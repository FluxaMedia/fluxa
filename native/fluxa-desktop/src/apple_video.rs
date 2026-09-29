use std::ffi::c_void;

use fluxa_host::{Thumbnail, VideoBackend, VideoCommand, VideoStatus};
use objc2::encode::{Encode, Encoding};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::AnyObject;
use objc2::{class, msg_send, msg_send_id};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

use crate::mpv_common::{Chapters, ThumbnailWorker, buffering, thumbnail_url};

#[link(name = "AVFoundation", kind = "framework")]
unsafe extern "C" {}

#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Size {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    origin: Point,
    size: Size,
}

unsafe impl Encode for Point {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}

unsafe impl Encode for Size {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

unsafe impl Encode for Rect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[Point::ENCODING, Size::ENCODING]);
}

const WIDTH_SIZABLE: usize = 2;
const HEIGHT_SIZABLE: usize = 16;
const WINDOW_BELOW: isize = -1;

pub struct VideoLayer {
    _view: Retained<AnyObject>,
    layer: Retained<AnyObject>,
}

impl VideoLayer {
    pub fn attach(window: &Window) -> Option<Self> {
        let RawWindowHandle::AppKit(handle) = window.window_handle().ok()?.as_raw() else {
            return None;
        };
        let content = handle.ns_view.as_ptr().cast::<AnyObject>();
        unsafe {
            let ns_window: *mut AnyObject = msg_send![content, window];
            let superview: *mut AnyObject = msg_send![content, superview];
            if ns_window.is_null() || superview.is_null() {
                return None;
            }
            let clear: *mut AnyObject = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![ns_window, setOpaque: false];
            let _: () = msg_send![ns_window, setBackgroundColor: clear];

            let frame: Rect = msg_send![content, frame];
            let alloc: Allocated<AnyObject> = msg_send_id![class!(NSView), alloc];
            let view: Retained<AnyObject> = msg_send_id![alloc, initWithFrame: frame];
            let layer: Retained<AnyObject> = msg_send_id![class!(AVSampleBufferDisplayLayer), new];
            let _: () = msg_send![&*layer, setContentsScale: window.scale_factor()];
            let _: () = msg_send![&*view, setLayer: &*layer];
            let _: () = msg_send![&*view, setWantsLayer: true];
            let _: () = msg_send![&*view, setAutoresizingMask: WIDTH_SIZABLE | HEIGHT_SIZABLE];
            let _: () = msg_send![
                superview,
                addSubview: &*view,
                positioned: WINDOW_BELOW,
                relativeTo: content
            ];
            Some(Self { _view: view, layer })
        }
    }

    pub fn pointer(&self) -> usize {
        Retained::as_ptr(&self.layer) as *const c_void as usize
    }

    pub fn set_scale(&self, scale: f64) {
        unsafe {
            let _: () = msg_send![&*self.layer, setContentsScale: scale];
        }
    }
}

pub struct AppleBackend {
    layer: usize,
    player: Option<Player>,
    error: Option<String>,
    url: Option<String>,
    shaders: Vec<String>,
    thumbnails: Option<ThumbnailWorker>,
}

struct Player {
    client: fluxa_mpv::MpvClientHandle,
    _render: fluxa_mpv::MpvRenderState,
    chapters: Chapters,
}

impl AppleBackend {
    pub fn new(layer: usize) -> Self {
        Self {
            layer,
            player: None,
            error: None,
            url: None,
            shaders: Vec::new(),
            thumbnails: None,
        }
    }

    fn start(&mut self, url: &str, start_at: Option<u64>) {
        self.stop();
        let result = fluxa_mpv::MpvClientHandle::new_with_apple_layer(self.layer, &self.shaders)
            .and_then(|(mut client, render)| {
                client.load(url, start_at)?;
                Ok(Player {
                    client,
                    _render: render,
                    chapters: Chapters::default(),
                })
            });
        match result {
            Ok(player) => {
                self.player = Some(player);
                self.url = Some(url.to_owned());
            }
            Err(error) => {
                eprintln!("[fluxa-desktop] apple_native setup failed: {error}");
                self.error = Some(error);
            }
        }
    }

    fn reload_with_shaders(&mut self) {
        let Some(url) = self.url.clone() else {
            return;
        };
        let position = self
            .player
            .as_ref()
            .and_then(|player| player.client.fast_position_status().time_pos)
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite() && *value > 0.0)
            .map(|value| value as u64);
        self.start(&url, position);
    }

    fn poll(&mut self) {
        let Some(player) = self.player.as_mut() else {
            return;
        };
        for event in player.client.poll_events() {
            if let fluxa_mpv::PlayerEvent::EndFile {
                error: Some(error), ..
            } = event
            {
                self.error = Some(error);
            }
        }
    }
}

impl VideoBackend for AppleBackend {
    fn load(&mut self, _instance: &wgpu::Instance, _device: &wgpu::Device, url: &str) {
        self.start(url, None);
    }

    fn stop(&mut self) {
        if let Some(player) = self.player.take() {
            let _ = player.client.command(&["stop"]);
        }
        self.error = None;
        self.url = None;
        self.thumbnails = None;
    }

    fn command(&mut self, command: VideoCommand) {
        if let VideoCommand::Shaders(shaders) = command {
            let chain = shader_paths(&shaders);
            if chain != self.shaders {
                self.shaders = chain;
                self.reload_with_shaders();
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
        self.poll();
        None
    }

    fn tracks(&mut self) -> Vec<fluxa_host::VideoTrack> {
        self.player
            .as_ref()
            .map(|player| crate::mpv_common::list_tracks(&player.client))
            .unwrap_or_default()
    }

    fn status(&mut self) -> VideoStatus {
        self.poll();
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
            has_frame: status.vo_configured.as_deref() == Some("yes"),
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

    fn passthrough(&self) -> bool {
        true
    }
}

fn shader_paths(shaders: &[String]) -> Vec<String> {
    let Some(dir) = crate::mpv_common::shader_dir() else {
        return Vec::new();
    };
    shaders
        .iter()
        .map(|shader| dir.join(shader).to_string_lossy().into_owned())
        .collect()
}
