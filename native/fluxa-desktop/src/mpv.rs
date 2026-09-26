use std::ffi::{CStr, c_void};
use std::sync::{
    Arc,
    mpsc::{self, Receiver, Sender, TryRecvError},
};
use std::time::{Duration, Instant};

use ash::vk::Handle as _;
use fluxa_host::{DeviceOpener, Thumbnail, VideoBackend, VideoCommand, VideoStatus};

const FRAME_SIZE: [u32; 2] = [1920, 1080];
const THUMBNAIL_SIZE: [usize; 2] = [320, 180];
const CHAPTER_REFRESH: Duration = Duration::from_secs(2);

pub struct MpvBackend {
    pending: Option<Receiver<Result<MpvPlayer, String>>>,
    player: Option<MpvPlayer>,
    error: Option<String>,
    url: Option<String>,
    thumbnails: Option<ThumbnailWorker>,
}

impl MpvBackend {
    pub fn new() -> Self {
        Self {
            pending: None,
            player: None,
            error: None,
            url: None,
            thumbnails: None,
        }
    }

    fn start(&mut self, instance: &wgpu::Instance, device: &wgpu::Device, url: &str, preview: bool) {
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

    fn stop(&mut self) {
        if let Some(player) = self.player.take() {
            let _ = player.client.command(&["stop"]);
        }
        self.pending = None;
        self.error = None;
        self.url = None;
        self.thumbnails = None;
    }

    fn command(&mut self, command: VideoCommand) {
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
            chapters: player.chapters().to_vec(),
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
        let worker = self.thumbnails.as_ref()?;
        let mut latest = None;
        while let Ok(thumbnail) = worker.results.try_recv() {
            latest = Some(thumbnail);
        }
        latest
    }
}

fn thumbnail_url(url: &str) -> String {
    if url.contains("/stream/fname") {
        format!("{url}&role=auxiliary")
    } else {
        url.to_owned()
    }
}

struct ThumbnailWorker {
    requests: Sender<f64>,
    results: Receiver<Thumbnail>,
}

impl ThumbnailWorker {
    fn spawn(url: String) -> Self {
        let (requests, request_rx) = mpsc::channel::<f64>();
        let (result_tx, results) = mpsc::channel();
        std::thread::spawn(move || {
            let mut renderer = match fluxa_mpv::MpvThumbnailRenderer::new()
                .and_then(|mut renderer| renderer.load_thumbnail(&url).map(|_| renderer))
            {
                Ok(renderer) => renderer,
                Err(error) => {
                    eprintln!("[fluxa-desktop] seek thumbnails unavailable: {error}");
                    return;
                }
            };
            while let Ok(mut time) = request_rx.recv() {
                while let Ok(next) = request_rx.try_recv() {
                    time = next;
                }
                match render_thumbnail(&mut renderer, time) {
                    Ok(rgba) => {
                        let thumbnail = Thumbnail {
                            time,
                            size: THUMBNAIL_SIZE,
                            rgba,
                        };
                        if result_tx.send(thumbnail).is_err() {
                            return;
                        }
                    }
                    Err(error) => eprintln!("[fluxa-desktop] seek thumbnail failed: {error}"),
                }
            }
        });
        Self { requests, results }
    }
}

fn render_thumbnail(
    renderer: &mut fluxa_mpv::MpvThumbnailRenderer,
    time: f64,
) -> Result<Vec<u8>, String> {
    renderer.set_paused(false)?;
    renderer.seek_thumbnail_to(time)?;
    renderer.pump_events();
    std::thread::sleep(Duration::from_millis(50));
    renderer.set_paused(true)?;
    renderer.pump_events();
    std::thread::sleep(Duration::from_millis(20));
    renderer.render_thumbnail(THUMBNAIL_SIZE[0] as i32, THUMBNAIL_SIZE[1] as i32)
}

struct MpvPlayer {
    render: fluxa_mpv::MpvRenderState,
    client: fluxa_mpv::MpvClientHandle,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    image_layout: i32,
    sync: VulkanSync,
    first_frame: bool,
    chapters: Vec<(f64, String)>,
    chapters_at: Option<Instant>,
}

impl MpvPlayer {
    fn new(
        instance: &wgpu::Instance,
        device: &wgpu::Device,
        url: &str,
        preview: bool,
    ) -> Result<Self, String> {
        let local = preview
            || url.starts_with("http://127.0.0.1:")
            || url.starts_with("http://localhost:");
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
            chapters: Vec::new(),
            chapters_at: None,
        })
    }

    fn chapters(&mut self) -> &[(f64, String)] {
        if self.chapters_at.is_none_or(|at| at.elapsed() >= CHAPTER_REFRESH) {
            self.chapters_at = Some(Instant::now());
            self.chapters = self
                .client
                .chapters_json()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .and_then(|value| {
                    value.get("chapters")?.as_array().map(|chapters| {
                        chapters
                            .iter()
                            .filter_map(|chapter| {
                                Some((
                                    chapter.get("startMs")?.as_u64()? as f64 / 1000.0,
                                    chapter.get("title")?.as_str()?.to_owned(),
                                ))
                            })
                            .collect()
                    })
                })
                .unwrap_or_default();
        }
        &self.chapters
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

fn vulkan_handles(instance: &wgpu::Instance, device: &wgpu::Device) -> Result<VulkanHandles, String> {
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
        &[c"VK_KHR_external_memory_fd", c"VK_KHR_external_semaphore_fd"]
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
