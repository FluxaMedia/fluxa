use std::ffi::{CStr, c_void};
use std::sync::{
    Arc,
    mpsc::{self, Receiver, TryRecvError},
};

use ash::vk::Handle as _;
use fluxa_host::{DeviceOpener, Thumbnail, VideoBackend, VideoCommand, VideoStatus};

use crate::mpv_common::{Chapters, ThumbnailWorker, buffering, shader_dir, thumbnail_url};

const FRAME_SIZE: [u32; 2] = [1920, 1080];

pub struct MpvBackend {
    pending: Option<Receiver<Result<MpvPlayer, String>>>,
    player: Option<MpvPlayer>,
    error: Option<String>,
    url: Option<String>,
    thumbnails: Option<ThumbnailWorker>,
    shaders: Vec<String>,
    preview: bool,
}

impl MpvBackend {
    pub fn new() -> Self {
        Self {
            pending: None,
            player: None,
            error: None,
            url: None,
            thumbnails: None,
            shaders: Vec::new(),
            preview: false,
        }
    }

    fn start(
        &mut self,
        instance: &wgpu::Instance,
        device: &wgpu::Device,
        url: &str,
        preview: bool,
    ) {
        self.stop();
        let (sender, receiver) = mpsc::channel();
        let instance = instance.clone();
        let device = device.clone();
        let url = url.to_owned();
        let target = url.clone();
        std::thread::spawn(move || {
            let _ = sender.send(MpvPlayer::new(&instance, &device, &target, preview));
        });
        self.url = Some(url.clone());
        self.pending = Some(receiver);
        self.preview = preview;
    }
}

impl VideoBackend for MpvBackend {
    fn device_opener(&self) -> Option<DeviceOpener> {
        Some(Arc::new(open_shared_device))
    }

    fn load(&mut self, instance: &wgpu::Instance, device: &wgpu::Device, url: &str) {
        self.start(instance, device, url, false);
    }

    fn load_preview(&mut self, instance: &wgpu::Instance, device: &wgpu::Device, url: &str) {
        self.start(instance, device, url, true);
    }

    fn media_session(&mut self, plan: &serde_json::Value) {
        #[cfg(target_os = "linux")]
        crate::mpris::publish(plan);
        #[cfg(not(target_os = "linux"))]
        let _ = plan;
    }

    fn stop(&mut self) {
        #[cfg(target_os = "linux")]
        crate::mpris::clear();
        if let Some(player) = self.player.take() {
            let _ = player.client.command(&["stop"]);
        }
        self.pending = None;
        self.error = None;
        self.url = None;
        self.thumbnails = None;
    }

    fn command(&mut self, command: VideoCommand) {
        if let VideoCommand::Shaders(shaders) = command {
            self.shaders = shaders;
            if let Some(player) = self.player.as_ref().filter(|_| !self.preview) {
                apply_shaders(player, &self.shaders);
            }
            return;
        }
        let Some(player) = self.player.as_mut() else {
            return;
        };
        let result = match command {
            VideoCommand::TogglePause => player.client.command(&["cycle", "pause"]),
            VideoCommand::ToggleMute => player.client.command(&["cycle", "mute"]),
            VideoCommand::Seek(delta) => {
                player
                    .client
                    .command(&["seek", &delta.to_string(), "relative"])
            }
            VideoCommand::SeekTo(position) => player.client.seek_to(position),
            VideoCommand::SelectTracks(selection) => {
                crate::mpv_common::select_tracks(&player.client, &selection)
            }
            VideoCommand::SetVolume(volume) => {
                player
                    .client
                    .command(&["set", "volume", &volume.to_string()])
            }
            VideoCommand::SetSpeed(rate) => {
                player.client.command(&["set", "speed", &rate.to_string()])
            }
            VideoCommand::Shaders(_) => Ok(()),
        };
        if let Err(error) = result {
            eprintln!("[fluxa-desktop] mpv command failed: {error}");
        }
    }

    fn render(&mut self, _device: &wgpu::Device) -> Option<wgpu::TextureView> {
        if let Some(receiver) = self.pending.as_ref() {
            match receiver.try_recv() {
                Ok(Ok(player)) => {
                    self.pending = None;
                    if !self.preview && !self.shaders.is_empty() {
                        apply_shaders(&player, &self.shaders);
                    }
                    self.player = Some(player);
                }
                Ok(Err(error)) => {
                    eprintln!("[fluxa-desktop] mpv setup failed: {error}");
                    self.pending = None;
                    self.error = Some(error);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.error = Some("mpv setup worker stopped".to_owned());
                }
            }
        }
        let player = self.player.as_mut()?;
        match player.render() {
            Ok(first) => first.then(|| player.view.clone()),
            Err(error) => {
                eprintln!("[fluxa-desktop] mpv frame failed: {error}");
                self.player = None;
                self.error = Some(error);
                None
            }
        }
    }

    fn tracks(&mut self) -> Vec<fluxa_host::VideoTrack> {
        self.player
            .as_ref()
            .map(|player| crate::mpv_common::list_tracks(&player.client))
            .unwrap_or_default()
    }

    fn status(&mut self) -> VideoStatus {
        let Some(player) = self.player.as_mut() else {
            return VideoStatus {
                error: self.error.clone(),
                volume: 100.0,
                ..VideoStatus::default()
            };
        };
        let position = player.client.fast_position_status();
        let status = player.client.cached_static_status();
        let number = |value: Option<&str>| value.and_then(|value| value.parse().ok());
        VideoStatus {
            position: number(position.time_pos.as_deref()).unwrap_or(0.0),
            duration: number(status.duration.as_deref()).unwrap_or(0.0),
            paused: status.pause.as_deref() == Some("yes"),
            muted: status.mute.as_deref() == Some("yes"),
            volume: number(status.volume.as_deref()).unwrap_or(100.0),
            has_frame: player.first_frame,
            error: self.error.clone(),
            chapters: player.chapters.get(&player.client).to_vec(),
            buffering: buffering(&position),
        }
    }

    fn request_thumbnail(&mut self, time: f64) {
        let Some(url) = self.url.as_deref() else {
            return;
        };
        let worker = self
            .thumbnails
            .get_or_insert_with(|| ThumbnailWorker::spawn(thumbnail_url(url)));
        let _ = worker.requests.send(time);
    }

    fn take_thumbnail(&mut self) -> Option<Thumbnail> {
        self.thumbnails.as_ref()?.latest()
    }
}

fn apply_shaders(player: &MpvPlayer, shaders: &[String]) {
    let commands: Vec<Vec<String>> = match shader_dir().filter(|_| !shaders.is_empty()) {
        Some(dir) => {
            let chain = shaders
                .iter()
                .map(|shader| dir.join(shader).to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(if cfg!(windows) { ";" } else { ":" });
            vec![
                vec![
                    "change-list".into(),
                    "glsl-shaders".into(),
                    "set".into(),
                    chain,
                ],
                vec!["set".into(), "scale".into(), "ewa_lanczossharp".into()],
                vec!["set".into(), "cscale".into(), "ewa_lanczos".into()],
                vec!["set".into(), "dscale".into(), "mitchell".into()],
                vec!["set".into(), "correct-downscaling".into(), "yes".into()],
                vec!["set".into(), "linear-downscaling".into(), "yes".into()],
            ]
        }
        None => vec![
            vec![
                "change-list".into(),
                "glsl-shaders".into(),
                "clr".into(),
                String::new(),
            ],
            vec!["set".into(), "scale".into(), "bilinear".into()],
            vec!["set".into(), "cscale".into(), "bilinear".into()],
        ],
    };
    for command in commands {
        let args = command.iter().map(String::as_str).collect::<Vec<_>>();
        if let Err(error) = player.client.command_args(&args) {
            eprintln!("[fluxa-desktop] mpv shader command failed: {error}");
        }
    }
}

struct MpvPlayer {
    render: fluxa_mpv::MpvRenderState,
    client: fluxa_mpv::MpvClientHandle,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    image_layout: i32,
    sync: VulkanSync,
    first_frame: bool,
    chapters: Chapters,
}

impl MpvPlayer {
    fn new(
        instance: &wgpu::Instance,
        device: &wgpu::Device,
        url: &str,
        preview: bool,
    ) -> Result<Self, String> {
        let local =
            preview || url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:");
        let (mut client, mut render) = if local {
            fluxa_mpv::MpvClientHandle::new_without_ytdl()?
        } else {
            fluxa_mpv::MpvClientHandle::new()?
        };
        let handles = vulkan_handles(instance, device)?;
        if let Err(error) = render.create_vulkan_context(
            handles.instance,
            handles.phys_device,
            handles.device,
            handles.queue_index,
            1,
            handles.get_proc_address,
            &handles.extensions,
        ) {
            let _ = client.poll_events();
            for line in client.recent_log_lines() {
                eprintln!("[fluxa-desktop] mpv: {line}");
            }
            return Err(error);
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fluxa-mpv-frame"),
            size: wgpu::Extent3d {
                width: FRAME_SIZE[0],
                height: FRAME_SIZE[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sync = VulkanSync::new(device)?;
        if preview {
            client.apply_options(&[
                ("mute".to_owned(), "yes".to_owned()),
                ("loop-file".to_owned(), "inf".to_owned()),
                ("sid".to_owned(), "no".to_owned()),
            ])?;
        }
        client.load(url, None)?;
        Ok(Self {
            render,
            client,
            texture,
            view,
            image_layout: 0,
            sync,
            first_frame: false,
            chapters: Chapters::default(),
        })
    }

    fn render(&mut self) -> Result<bool, String> {
        for event in self.client.poll_events() {
            if let fluxa_mpv::PlayerEvent::EndFile {
                error: Some(error), ..
            } = event
            {
                return Err(error);
            }
        }
        if !self.render.vulkan_frame_ready() {
            return Ok(false);
        }
        let hal_texture = unsafe { self.texture.as_hal::<wgpu::hal::api::Vulkan>() }
            .ok_or_else(|| "wgpu Vulkan texture handle unavailable".to_owned())?;
        let image = unsafe { hal_texture.raw_handle() }.as_raw();
        let (wait, signal) = self.sync.next_pair();
        let mut target = fluxa_mpv::VulkanTargetImage {
            image,
            // VK_FORMAT_B8G8R8A8_UNORM
            format: 44,
            w: FRAME_SIZE[0] as i32,
            h: FRAME_SIZE[1] as i32,
            usage: (ash::vk::ImageUsageFlags::COLOR_ATTACHMENT | ash::vk::ImageUsageFlags::SAMPLED)
                .as_raw(),
            layout: self.image_layout,
            wait_semaphore: wait.as_raw(),
            signal_semaphore: signal.as_raw(),
        };
        self.render.render_vulkan_frame(&mut target)?;
        self.image_layout = target.layout;
        self.render.report_swap();
        let first = !self.first_frame;
        self.first_frame = true;
        Ok(first)
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.sync.device.device_wait_idle();
        }
    }
}

struct VulkanSync {
    device: ash::Device,
    semaphores: [ash::vk::Semaphore; 2],
    previous: Option<ash::vk::Semaphore>,
    next: usize,
}

impl VulkanSync {
    fn new(device: &wgpu::Device) -> Result<Self, String> {
        let hal_device = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }
            .ok_or_else(|| "wgpu Vulkan device handle unavailable".to_owned())?;
        let device = hal_device.raw_device().clone();
        let info = ash::vk::SemaphoreCreateInfo::default();
        let first = unsafe { device.create_semaphore(&info, None) }
            .map_err(|error| format!("create mpv semaphore: {error:?}"))?;
        let second = match unsafe { device.create_semaphore(&info, None) } {
            Ok(semaphore) => semaphore,
            Err(error) => {
                unsafe { device.destroy_semaphore(first, None) };
                return Err(format!("create mpv semaphore: {error:?}"));
            }
        };
        Ok(Self {
            device,
            semaphores: [first, second],
            previous: None,
            next: 0,
        })
    }

    fn next_pair(&mut self) -> (ash::vk::Semaphore, ash::vk::Semaphore) {
        let signal = self.semaphores[self.next];
        self.next = (self.next + 1) % self.semaphores.len();
        let wait = self
            .previous
            .replace(signal)
            .unwrap_or(ash::vk::Semaphore::null());
        (wait, signal)
    }
}

impl Drop for VulkanSync {
    fn drop(&mut self) {
        unsafe {
            // mpv and wgpu share this device; both queues must be idle before the semaphores go.
            let _ = self.device.device_wait_idle();
            for semaphore in self.semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }
        }
    }
}

struct VulkanHandles {
    instance: *mut c_void,
    phys_device: *mut c_void,
    device: *mut c_void,
    queue_index: u32,
    get_proc_address: *mut c_void,
    extensions: Vec<*const i8>,
}

fn vulkan_handles(
    instance: &wgpu::Instance,
    device: &wgpu::Device,
) -> Result<VulkanHandles, String> {
    let hal_instance = unsafe { instance.as_hal::<wgpu::hal::api::Vulkan>() }
        .ok_or_else(|| "wgpu is not using the Vulkan backend".to_owned())?;
    let hal_device = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }
        .ok_or_else(|| "wgpu Vulkan device handle unavailable".to_owned())?;
    let shared = hal_instance.shared_instance();
    Ok(VulkanHandles {
        instance: shared.raw_instance().handle().as_raw() as *mut c_void,
        phys_device: hal_device.raw_physical_device().as_raw() as *mut c_void,
        device: hal_device.raw_device().handle().as_raw() as *mut c_void,
        queue_index: hal_device.queue_family_index(),
        get_proc_address: shared.entry().static_fn().get_instance_proc_addr as *mut c_void,
        extensions: hal_device
            .enabled_device_extensions()
            .iter()
            .map(|extension| extension.as_ptr())
            .collect(),
    })
}

fn open_shared_device(
    adapter: &wgpu::Adapter,
    descriptor: &wgpu::DeviceDescriptor<'_>,
) -> Result<(wgpu::Device, wgpu::Queue), String> {
    let hal_adapter = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }
        .ok_or_else(|| "wgpu adapter is not backed by Vulkan".to_owned())?;
    let required: &[&CStr] = if cfg!(target_os = "linux") {
        &[
            c"VK_KHR_external_memory_fd",
            c"VK_KHR_external_semaphore_fd",
        ]
    } else {
        &[]
    };
    let capabilities = hal_adapter.physical_device_capabilities();
    let missing = required
        .iter()
        .filter(|extension| !capabilities.supports_extension(extension))
        .map(|extension| extension.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!("missing Vulkan extensions: {}", missing.join(", ")));
    }
    let callback = Box::new(
        move |args: wgpu::hal::vulkan::CreateDeviceCallbackArgs<'_, '_, '_>| {
            for &extension in required {
                if !args.extensions.contains(&extension) {
                    args.extensions.push(extension);
                }
            }
        },
    );
    let hal_device = unsafe {
        hal_adapter.open_with_callback(
            descriptor.required_features,
            &descriptor.required_limits,
            &descriptor.memory_hints,
            Some(callback),
        )
    }
    .map_err(|error| format!("open Vulkan device: {error}"))?;
    unsafe { adapter.create_device_from_hal(hal_device, descriptor) }
        .map_err(|error| format!("create wgpu device: {error}"))
}
