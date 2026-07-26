use base64::{engine::general_purpose, Engine as _};
use libloading::Library;
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::path::PathBuf;
use std::ptr;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::sync::OnceLock;

mod api;
use api::MpvApi;

#[cfg(target_os = "linux")]
use std::ffi::{c_uchar, c_uint};

type MpvHandle = c_void;
type MpvRenderContext = c_void;

const MPV_RENDER_PARAM_INVALID: c_int = 0;
const MPV_RENDER_PARAM_API_TYPE: c_int = 1;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
const MPV_RENDER_PARAM_OPENGL_INIT_PARAMS: c_int = 2;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
const MPV_RENDER_PARAM_OPENGL_FBO: c_int = 3;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
const MPV_RENDER_PARAM_FLIP_Y: c_int = 4;
const MPV_RENDER_PARAM_ICC_PROFILE: c_int = 6;
#[cfg(target_os = "linux")]
const MPV_RENDER_PARAM_X11_DISPLAY: c_int = 8;
#[cfg(target_os = "linux")]
const MPV_RENDER_PARAM_WL_DISPLAY: c_int = 9;
const MPV_RENDER_PARAM_SW_SIZE: c_int = 17;
const MPV_RENDER_PARAM_SW_FORMAT: c_int = 18;
const MPV_RENDER_PARAM_SW_STRIDE: c_int = 19;
const MPV_RENDER_PARAM_SW_POINTER: c_int = 20;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
const MPV_RENDER_PARAM_VULKAN_INIT_PARAMS: c_int = 21;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
const MPV_RENDER_PARAM_VULKAN_IMAGE: c_int = 22;
#[cfg(target_os = "windows")]
const MPV_RENDER_PARAM_D3D11_INIT_PARAMS: c_int = 23;
#[cfg(target_os = "windows")]
const MPV_RENDER_PARAM_D3D11_TARGET: c_int = 24;
const MPV_RENDER_UPDATE_FRAME: u64 = 1 << 0;

const MPV_EVENT_NONE: c_int = 0;
const MPV_EVENT_LOG_MESSAGE: c_int = 2;
const MPV_EVENT_COMMAND_REPLY: c_int = 5;
const MPV_EVENT_END_FILE: c_int = 7;
const MPV_EVENT_PLAYBACK_RESTART: c_int = 21;
const MPV_END_FILE_REASON_EOF: c_int = 0;
const MPV_END_FILE_REASON_ERROR: c_int = 4;

#[repr(C)]
struct MpvEvent {
    event_id: c_int,
    error: c_int,
    reply_userdata: u64,
    data: *mut c_void,
}

#[repr(C)]
struct MpvEventEndFile {
    reason: c_int,
    error: c_int,
    playlist_entry_id: i64,
    playlist_insert_id: c_int,
    playlist_insert_num_entries: c_int,
}

#[repr(C)]
struct MpvEventLogMessage {
    prefix: *const c_char,
    level: *const c_char,
    text: *const c_char,
    log_level: c_int,
}

pub enum PlayerEvent {
    EndFile { eof: bool, error: Option<String> },
}

#[repr(C)]
struct MpvRenderParam {
    param_type: c_int,
    data: *mut c_void,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
struct MpvOpenGlInitParams {
    get_proc_address:
        Option<unsafe extern "C" fn(ctx: *mut c_void, name: *const c_char) -> *mut c_void>,
    get_proc_address_ctx: *mut c_void,
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
#[repr(C)]
struct MpvOpenGlFbo {
    fbo: c_int,
    width: c_int,
    height: c_int,
    internal_format: c_int,
}

#[repr(C)]
struct MpvByteArray {
    data: *const u8,
    size: usize,
}

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
#[repr(C)]
struct MpvVulkanInitParams {
    instance: *mut c_void,
    phys_device: *mut c_void,
    device: *mut c_void,
    get_proc_address: *mut c_void,
    queue_graphics_index: u32,
    queue_graphics_count: u32,
    enabled_extensions: *const *const c_char,
    num_enabled_extensions: i32,
}

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
#[repr(C)]
pub struct VulkanTargetImage {
    pub image: u64,
    pub format: i32,
    pub w: c_int,
    pub h: c_int,
    pub usage: u32,
    pub layout: i32,
    pub wait_semaphore: u64,
    pub signal_semaphore: u64,
}

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
#[repr(C)]
struct MpvVulkanImageFfi {
    image: u64,
    format: i32,
    w: c_int,
    h: c_int,
    usage: u32,
    layout: i32,
    wait_semaphore: u64,
    signal_semaphore: u64,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct MpvD3d11InitParams {
    device: *mut c_void,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct MpvD3d11Target {
    tex: *mut c_void,
    format: i32,
    w: c_int,
    h: c_int,
}

type MpvCreate = unsafe extern "C" fn() -> *mut MpvHandle;
type MpvInitialize = unsafe extern "C" fn(*mut MpvHandle) -> c_int;
type MpvTerminateDestroy = unsafe extern "C" fn(*mut MpvHandle);
type MpvSetOptionString =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, *const c_char) -> c_int;
type MpvCommand = unsafe extern "C" fn(*mut MpvHandle, *const *const c_char) -> c_int;
type MpvCommandString = unsafe extern "C" fn(*mut MpvHandle, *const c_char) -> c_int;
type MpvCommandAsync = unsafe extern "C" fn(*mut MpvHandle, u64, *const *const c_char) -> c_int;
type MpvGetProperty =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, c_int, *mut c_void) -> c_int;
type MpvFree = unsafe extern "C" fn(*mut c_void);
type MpvErrorString = unsafe extern "C" fn(c_int) -> *const c_char;
type MpvRenderContextCreate =
    unsafe extern "C" fn(*mut *mut MpvRenderContext, *mut MpvHandle, *mut MpvRenderParam) -> c_int;
type MpvRenderContextRender =
    unsafe extern "C" fn(*mut MpvRenderContext, *mut MpvRenderParam) -> c_int;
type MpvRenderContextUpdate = unsafe extern "C" fn(*mut MpvRenderContext) -> u64;
type MpvRenderContextReportSwap = unsafe extern "C" fn(*mut MpvRenderContext);
type MpvRenderContextSetParameter =
    unsafe extern "C" fn(*mut MpvRenderContext, MpvRenderParam) -> c_int;
type MpvRenderContextFree = unsafe extern "C" fn(*mut MpvRenderContext);
type MpvWaitEvent = unsafe extern "C" fn(*mut MpvHandle, f64) -> *mut MpvEvent;
type MpvRequestLogMessages = unsafe extern "C" fn(*mut MpvHandle, *const c_char) -> c_int;

pub struct MpvRenderer {
    api: MpvApi,
    handle: *mut MpvHandle,
    render_context: *mut MpvRenderContext,
    buffer: Vec<u8>,
    width: i32,
    height: i32,
    loaded: bool,
    log_ring: std::collections::VecDeque<(c_int, String)>,
    frames_rendered: u64,
    first_frame_presented: bool,
    pending_unpause: bool,
    pending_seek_seconds: Option<f64>,
    waiting_for_seek_restart: bool,
    frame_ready_to_restore_audio: bool,
    #[cfg(target_os = "windows")]
    muted_until_first_frame: bool,
    #[cfg(target_os = "windows")]
    restore_mute: Option<bool>,
    current_url: Option<String>,
}

unsafe impl Send for MpvRenderer {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerFrame {
    width: i32,
    height: i32,
    pixels_base64: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatus {
    pub loaded: bool,
    pub path: Option<String>,
    pub media_title: Option<String>,
    pub time_pos: Option<String>,
    pub duration: Option<String>,
    pub percent_pos: Option<String>,
    pub pause: Option<String>,
    pub mute: Option<String>,
    pub volume: Option<String>,
    pub core_idle: Option<String>,
    pub eof_reached: Option<String>,
    pub vo_configured: Option<String>,
    pub video_codec: Option<String>,
    pub video_format: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub cache_speed: Option<String>,
    pub demuxer_cache_duration: Option<String>,
    pub hwdec_current: Option<String>,
    pub fps: Option<String>,
    pub frame_drop_count: Option<String>,
    pub decoder_frame_drop_count: Option<String>,
    pub avsync: Option<String>,
    pub video_bitrate: Option<String>,
    pub audio_bitrate: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_samplerate: Option<String>,
    pub audio_channels: Option<String>,
    pub color_primaries: Option<String>,
    pub color_matrix: Option<String>,
    pub color_gamma: Option<String>,
    pub video_out_primaries: Option<String>,
    pub video_out_matrix: Option<String>,
    pub video_out_gamma: Option<String>,
    pub sig_peak: Option<String>,
    pub container_fps: Option<String>,
    pub display_fps: Option<String>,
    pub mistimed_frame_count: Option<String>,
    pub vo_delayed_frame_count: Option<String>,
    pub paused_for_cache: Option<String>,
    pub cache_buffering_state: Option<String>,
    pub file_format: Option<String>,
    pub frames_rendered: u64,
    pub first_frame_presented: bool,
    pub has_video_track: bool,
    pub track_list_ready: bool,
    pub resuming: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerTrackOption {
    pub id: String,
    pub label: String,
    pub selected: bool,
    pub lang: Option<String>,
    pub source: Option<String>,
    pub external: bool,
    pub format: Option<String>,
}

fn subtitle_format_label(codec: &str) -> String {
    match codec {
        "subrip" | "srt" => "SRT".to_string(),
        "ass" => "ASS".to_string(),
        "ssa" => "SSA".to_string(),
        "webvtt" => "VTT".to_string(),
        "mov_text" => "MOV_TEXT".to_string(),
        "dvd_subtitle" => "VOBSUB".to_string(),
        "dvb_subtitle" => "DVB".to_string(),
        "hdmv_pgs_subtitle" | "pgssub" => "PGS".to_string(),
        other => other.to_uppercase(),
    }
}

fn audio_channel_label(channel_count: Option<u32>) -> Option<String> {
    match channel_count {
        Some(1) => Some("Mono".to_string()),
        Some(2) => Some("Stereo".to_string()),
        Some(6) => Some("5.1".to_string()),
        Some(8) => Some("7.1".to_string()),
        Some(n) => Some(format!("{n}ch")),
        None => None,
    }
}

impl PlayerStatus {
    pub fn time_pos(&self) -> Option<&str> {
        self.time_pos.as_deref()
    }

    pub fn duration(&self) -> Option<&str> {
        self.duration.as_deref()
    }

    pub fn pause(&self) -> Option<&str> {
        self.pause.as_deref()
    }

    pub fn mute(&self) -> Option<&str> {
        self.mute.as_deref()
    }

    pub fn volume(&self) -> Option<&str> {
        self.volume.as_deref()
    }

    pub fn vo_configured(&self) -> Option<&str> {
        self.vo_configured.as_deref()
    }

    pub fn demuxer_cache_duration(&self) -> Option<&str> {
        self.demuxer_cache_duration.as_deref()
    }

    pub fn eof_reached(&self) -> bool {
        self.eof_reached.as_deref() == Some("yes")
    }
}

impl MpvRenderer {
    pub fn new() -> Result<Self, String> {
        let api = MpvApi::load()?;
        let handle = unsafe { (api.mpv_create)() };
        if handle.is_null() {
            return Err("mpv_create returned null".to_string());
        }

        let renderer = Self {
            api,
            handle,
            render_context: ptr::null_mut(),
            buffer: Vec::new(),
            width: 0,
            height: 0,
            loaded: false,
            log_ring: std::collections::VecDeque::new(),
            frames_rendered: 0,
            first_frame_presented: false,
            pending_unpause: false,
            pending_seek_seconds: None,
            waiting_for_seek_restart: false,
            frame_ready_to_restore_audio: false,
            #[cfg(target_os = "windows")]
            muted_until_first_frame: false,
            #[cfg(target_os = "windows")]
            restore_mute: None,
            current_url: None,
        };

        renderer.set_option("terminal", "no")?;
        renderer.set_option("config", "no")?;
        renderer.set_option("vo", "libmpv")?;
        #[cfg(target_os = "windows")]
        renderer.set_option("gpu-api", "opengl")?;
        renderer.set_option("idle", "yes")?;
        renderer.set_option("keep-open", "yes")?;
        if let Err(error) = renderer.set_option("osc", "no") {
            log::warn!("set_option(osc, no) failed (mpv build without lua/OSC?): {error}");
        }
        renderer.set_option("osd-level", "0")?;
        renderer.set_option("osd-bar", "no")?;
        renderer.set_option("input-default-bindings", "yes")?;

        // GPU decode — auto-safe tries every platform API (VAAPI/NVDEC/DXVA2/VideoToolbox)
        // and falls back to software decode silently if none is available.
        renderer.set_option("hwdec", "auto-safe")?;
        renderer.set_option("hwdec-codecs", "all")?;

        renderer.set_option("video-sync", "audio")?;
        renderer.set_option("display-fps-override", "60")?;

        // Start playback immediately without waiting for the cache to fill.
        // cache-pause-initial=yes (default in many MPV builds) causes MPV to pause
        // at the beginning until demuxer-readahead-secs worth of data is buffered,
        // which looks like a frozen seekbar with no video or audio.
        renderer.set_option("cache-pause-initial", "no")?;

        // Network stream buffering (important for HLS/DASH/torrent)
        renderer.set_option("cache", "yes")?;
        renderer.set_option("cache-secs", "30")?;
        renderer.set_option("demuxer-max-bytes", "150MiB")?;
        renderer.set_option("demuxer-readahead-secs", "10")?;

        // Lower audio latency and proper app name for PulseAudio/PipeWire
        renderer.set_option("audio-buffer", "0.2")?;
        renderer.set_option("audio-client-name", "fluxa")?;

        let init_result = unsafe { (renderer.api.mpv_initialize)(renderer.handle) };
        if init_result < 0 {
            let message = renderer.api.error_string(init_result);
            unsafe { (renderer.api.mpv_terminate_destroy)(renderer.handle) };
            return Err(format!("mpv_initialize failed: {message}"));
        }

        let level = CString::new("v").unwrap();
        unsafe { (renderer.api.mpv_request_log_messages)(renderer.handle, level.as_ptr()) };

        Ok(renderer)
    }

    pub fn new_thumbnail() -> Result<Self, String> {
        let api = MpvApi::load()?;
        let handle = unsafe { (api.mpv_create)() };
        if handle.is_null() {
            return Err("mpv_create returned null".to_string());
        }

        let mut renderer = Self {
            api,
            handle,
            render_context: ptr::null_mut(),
            buffer: Vec::new(),
            width: 0,
            height: 0,
            loaded: false,
            log_ring: std::collections::VecDeque::new(),
            frames_rendered: 0,
            first_frame_presented: false,
            pending_unpause: false,
            pending_seek_seconds: None,
            waiting_for_seek_restart: false,
            frame_ready_to_restore_audio: false,
            #[cfg(target_os = "windows")]
            muted_until_first_frame: false,
            #[cfg(target_os = "windows")]
            restore_mute: None,
            current_url: None,
        };

        renderer.set_option("terminal", "no")?;
        renderer.set_option("config", "no")?;
        renderer.set_option("vo", "libmpv")?;
        renderer.set_option("ao", "null")?;
        renderer.set_option("audio", "no")?;
        renderer.set_option("idle", "yes")?;
        if let Err(error) = renderer.set_option("osc", "no") {
            log::warn!("set_option(osc, no) failed (mpv build without lua/OSC?): {error}");
        }
        renderer.set_option("osd-level", "0")?;
        renderer.set_option("hr-seek", "yes")?;
        renderer.set_option("pause", "yes")?;
        renderer.set_option("cache", "yes")?;
        renderer.set_option("cache-secs", "10")?;
        renderer.set_option("demuxer-max-bytes", "50MiB")?;
        renderer.set_option("demuxer-readahead-secs", "2")?;

        let init_result = unsafe { (renderer.api.mpv_initialize)(renderer.handle) };
        if init_result < 0 {
            let message = renderer.api.error_string(init_result);
            unsafe { (renderer.api.mpv_terminate_destroy)(renderer.handle) };
            return Err(format!("mpv_initialize failed: {message}"));
        }

        renderer.create_software_context()?;

        Ok(renderer)
    }

    pub fn load(&mut self, url: &str, start_at: Option<u64>) -> Result<(), String> {
        self.loaded = false;
        self.current_url = Some(url.to_string());
        self.log_ring.clear();
        self.frames_rendered = 0;
        self.first_frame_presented = false;
        self.pending_unpause = true;
        self.pending_seek_seconds = start_at.filter(|&s| s > 0).map(|s| s as f64);
        self.waiting_for_seek_restart = false;
        self.frame_ready_to_restore_audio = false;
        #[cfg(target_os = "windows")]
        {
            let restore_mute = if self.muted_until_first_frame {
                self.restore_mute.unwrap_or(false)
            } else {
                self.get_string_property("mute").as_deref() == Some("yes")
            };
            self.muted_until_first_frame = true;
            self.restore_mute = Some(restore_mute);
            self.command_string("set mute yes")?;
        }
        self.command(&["loadfile", url, "replace"])?;
        self.command_string("set pause yes")?;
        self.loaded = true;
        Ok(())
    }

    pub fn load_thumbnail(&mut self, url: &str) -> Result<(), String> {
        self.loaded = false;
        self.command(&["loadfile", url, "replace"])?;
        self.loaded = true;
        Ok(())
    }

    pub fn first_frame_presented(&self) -> bool {
        self.first_frame_presented
    }

    pub fn seek_to(&self, time_pos: f64) -> Result<(), String> {
        self.command_string(&format!("seek {time_pos:.3} absolute+exact"))
    }

    /// Dispatch a command originating from the UI. Startup resume/unpause work
    /// must never run later and overwrite an explicit user action.
    pub fn user_command(&mut self, command: &str) -> Result<(), String> {
        let command = command.trim();
        if command == "cycle pause" || command.starts_with("set pause ") {
            self.pending_unpause = false;
        }
        if command.starts_with("seek ") || command.starts_with("set time-pos ") {
            self.pending_seek_seconds = None;
            self.waiting_for_seek_restart = false;
        }
        self.command_string(command)
    }

    pub fn render_thumbnail(&mut self, width: i32, height: i32) -> Result<Vec<u8>, String> {
        if !self.loaded {
            return Err("not loaded".to_string());
        }
        if self.render_context.is_null() {
            self.create_software_context()?;
        }
        let width = width.clamp(2, 1920);
        let height = height.clamp(2, 1080);
        self.ensure_buffer(width, height);

        let mut size = [width, height];
        let format = CString::new("rgb0").unwrap();
        let mut stride = (width as usize) * 4;
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_SIZE,
                data: size.as_mut_ptr().cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_FORMAT,
                data: format.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_STRIDE,
                data: (&mut stride as *mut usize).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_POINTER,
                data: self.buffer.as_mut_ptr().cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render (thumbnail) failed: {}",
                self.api.error_string(result)
            ));
        }

        for alpha in self.buffer.iter_mut().skip(3).step_by(4) {
            *alpha = 255;
        }

        Ok(self.buffer.clone())
    }

    pub fn add_subtitle(
        &self,
        url: &str,
        title: Option<&str>,
        language: Option<&str>,
    ) -> Result<(), String> {
        let title = title.unwrap_or("Subtitle");
        match language.filter(|value| !value.is_empty()) {
            Some(language) => self.command_args(&["sub-add", url, "auto", title, language]),
            None => self.command_args(&["sub-add", url, "auto", title]),
        }
    }

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    pub fn needs_opengl_context(&self) -> bool {
        self.render_context.is_null()
    }

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    pub fn prepare_opengl_context(&mut self) -> Result<(), String> {
        if self.render_context.is_null() {
            self.create_opengl_context()?;
        }
        Ok(())
    }

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    pub fn needs_vulkan_context(&self) -> bool {
        self.render_context.is_null()
    }

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    pub fn vulkan_frame_ready(&self) -> bool {
        if self.render_context.is_null() {
            return false;
        }
        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };
        update_flags & MPV_RENDER_UPDATE_FRAME != 0
    }

    #[cfg(target_os = "windows")]
    pub fn needs_d3d11_context(&self) -> bool {
        self.render_context.is_null()
    }

    pub fn command_string(&self, command: &str) -> Result<(), String> {
        let c_command = CString::new(command).map_err(|error| error.to_string())?;
        let result = unsafe { (self.api.mpv_command_string)(self.handle, c_command.as_ptr()) };
        if result < 0 {
            Err(format!(
                "mpv command failed: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub fn command(&self, args: &[&str]) -> Result<(), String> {
        let c_args = args
            .iter()
            .map(|arg| CString::new(*arg).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let mut raw_args = c_args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
        raw_args.push(ptr::null());

        let result = unsafe { (self.api.mpv_command)(self.handle, raw_args.as_ptr()) };
        if result < 0 {
            Err(format!(
                "mpv command failed: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub fn command_args(&self, args: &[&str]) -> Result<(), String> {
        let c_args = args
            .iter()
            .map(|arg| CString::new(*arg).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let mut raw_args = c_args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
        raw_args.push(ptr::null());

        let result = unsafe { (self.api.mpv_command_async)(self.handle, 0, raw_args.as_ptr()) };
        if result < 0 {
            Err(format!(
                "mpv async command failed: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub fn render_frame(&mut self, width: i32, height: i32) -> Result<PlayerFrame, String> {
        if !self.loaded {
            return Err("player has not loaded media yet".to_string());
        }
        if self.render_context.is_null() {
            self.create_software_context()?;
        }
        let width = width.clamp(2, 1920);
        let height = height.clamp(2, 1080);
        self.ensure_buffer(width, height);

        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };

        let mut size = [width, height];
        let format = CString::new("rgb0").unwrap();
        let mut stride = (width as usize) * 4;
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_SIZE,
                data: size.as_mut_ptr().cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_FORMAT,
                data: format.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_STRIDE,
                data: (&mut stride as *mut usize).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_SW_POINTER,
                data: self.buffer.as_mut_ptr().cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render (sw) failed: {}",
                self.api.error_string(result)
            ));
        }

        for alpha in self.buffer.iter_mut().skip(3).step_by(4) {
            *alpha = 255;
        }

        if update_flags & MPV_RENDER_UPDATE_FRAME != 0 {
            self.frames_rendered = self.frames_rendered.saturating_add(1);
            self.frame_ready_to_restore_audio = true;
        }

        Ok(PlayerFrame {
            width,
            height,
            pixels_base64: general_purpose::STANDARD.encode(&self.buffer),
        })
    }

    pub fn reset_render_context(&mut self) {
        if self.render_context.is_null() {
            return;
        }
        unsafe {
            (self.api.mpv_render_context_free)(self.render_context);
        }
        self.render_context = ptr::null_mut();
        self.frame_ready_to_restore_audio = false;
    }

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    pub fn render_opengl_frame(&mut self, width: i32, height: i32) -> Result<(), String> {
        if self.render_context.is_null() {
            self.create_opengl_context()?;
        }

        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };

        // Linux/GTK: query the offscreen FBO that GTK's GLArea binds.
        // Windows/macOS: render into the default framebuffer (FBO 0).
        #[cfg(target_os = "linux")]
        let fbo_id = query_draw_fbo();
        #[cfg(not(target_os = "linux"))]
        let fbo_id: c_int = 0;

        let mut fbo = MpvOpenGlFbo {
            fbo: fbo_id,
            width: width.max(2),
            height: height.max(2),
            internal_format: 0,
        };
        let mut flip_y: c_int = 1;
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_OPENGL_FBO,
                data: (&mut fbo as *mut MpvOpenGlFbo).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_FLIP_Y,
                data: (&mut flip_y as *mut c_int).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render failed: {}",
                self.api.error_string(result)
            ));
        }
        if update_flags & MPV_RENDER_UPDATE_FRAME != 0 {
            self.frames_rendered = self.frames_rendered.saturating_add(1);
            self.frame_ready_to_restore_audio = true;
        }
        Ok(())
    }

    /// Call right after the buffer swap completes.
    pub fn report_swap(&mut self) {
        if self.render_context.is_null() {
            return;
        }
        unsafe {
            (self.api.mpv_render_context_report_swap)(self.render_context);
        }
        if self.frame_ready_to_restore_audio {
            self.first_frame_presented = true;
        }
        self.restore_audio_after_first_presented_frame();
    }

    fn restore_audio_after_first_presented_frame(&mut self) {
        #[cfg(target_os = "windows")]
        {
            if !self.muted_until_first_frame
                || !self.frame_ready_to_restore_audio
                || self.pending_seek_seconds.is_some()
                || self.waiting_for_seek_restart
            {
                return;
            }
            self.frame_ready_to_restore_audio = false;
            self.muted_until_first_frame = false;
            let mute = if self.restore_mute.take().unwrap_or(false) {
                "yes"
            } else {
                "no"
            };
            let _ = self.command_string(&format!("set mute {mute}"));
        }
    }

    fn restore_audio_only(&mut self) {
        #[cfg(target_os = "windows")]
        {
            if !self.muted_until_first_frame
                || self.pending_seek_seconds.is_some()
                || self.waiting_for_seek_restart
            {
                return;
            }
            let (has_video_track, track_list_ready) = self.track_list_status();
            if track_list_ready && !has_video_track {
                self.muted_until_first_frame = false;
                let mute = if self.restore_mute.take().unwrap_or(false) {
                    "yes"
                } else {
                    "no"
                };
                let _ = self.command_string(&format!("set mute {mute}"));
            }
        }
    }

    pub fn set_icc_profile(&self, data: &[u8]) -> Result<(), String> {
        if self.render_context.is_null() {
            return Err("render context not created yet".to_string());
        }
        let byte_array = MpvByteArray {
            data: data.as_ptr(),
            size: data.len(),
        };
        let param = MpvRenderParam {
            param_type: MPV_RENDER_PARAM_ICC_PROFILE,
            data: (&byte_array as *const MpvByteArray) as *mut c_void,
        };
        let result =
            unsafe { (self.api.mpv_render_context_set_parameter)(self.render_context, param) };
        if result < 0 {
            Err(format!(
                "failed to set ICC profile: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub fn title(&self) -> Option<String> {
        self.get_string_property("media-title")
    }

    fn error_log_details(&self) -> Vec<String> {
        const MPV_LOG_LEVEL_WARN: c_int = 30;
        const MAX_LINES: usize = 6;
        const MAX_LINE_LEN: usize = 220;

        let mut skip_indices = std::collections::HashSet::new();
        for (i, (_, text)) in self.log_ring.iter().enumerate() {
            let Some(disp_filename) = text
                .strip_prefix("Can not open external file ")
                .and_then(|rest| rest.strip_suffix('.'))
            else {
                continue;
            };
            skip_indices.insert(i);
            if i > 0 {
                if let Some((_, prev_text)) = self.log_ring.get(i - 1) {
                    if prev_text == &format!("Failed to open {disp_filename}.") {
                        skip_indices.insert(i - 1);
                    }
                }
            }
        }

        let mut lines: Vec<String> = Vec::new();
        for (i, (level, text)) in self.log_ring.iter().enumerate().rev() {
            if *level > MPV_LOG_LEVEL_WARN {
                continue;
            }
            if skip_indices.contains(&i) {
                continue;
            }
            let mut line = text.clone();
            if line.len() > MAX_LINE_LEN {
                let mut cut = MAX_LINE_LEN;
                while !line.is_char_boundary(cut) {
                    cut -= 1;
                }
                line.truncate(cut);
                line.push('…');
            }
            if lines.contains(&line) {
                continue;
            }
            lines.push(line);
            if lines.len() >= MAX_LINES {
                break;
            }
        }
        lines.reverse();
        lines
    }

    pub fn poll_events(&mut self) -> Vec<PlayerEvent> {
        let mut events = Vec::new();
        loop {
            let raw = unsafe { (self.api.mpv_wait_event)(self.handle, 0.0) };
            if raw.is_null() {
                break;
            }
            let event = unsafe { &*raw };
            match event.event_id {
                MPV_EVENT_NONE => break,
                MPV_EVENT_LOG_MESSAGE if !event.data.is_null() => {
                    let msg = unsafe { &*(event.data as *const MpvEventLogMessage) };
                    let text = unsafe { CStr::from_ptr(msg.text) }.to_string_lossy();
                    let text = text.trim_end();
                    if !text.is_empty() {
                        log::debug!("mpv: {text}");
                        if self.log_ring.len() >= 40 {
                            self.log_ring.pop_front();
                        }
                        self.log_ring.push_back((msg.log_level, text.to_string()));
                    }
                }
                MPV_EVENT_END_FILE if !event.data.is_null() => {
                    let end_file = unsafe { &*(event.data as *const MpvEventEndFile) };
                    if end_file.reason == MPV_END_FILE_REASON_ERROR {
                        let mut message = self.api.error_string(end_file.error);
                        let details = self.error_log_details();
                        if !details.is_empty() {
                            message.push('\n');
                            message.push_str(&details.join("\n"));
                        }
                        let url = self.current_url.clone();
                        sentry::with_scope(
                            |scope| {
                                scope.set_tag("mpv.error_code", end_file.error);
                                if let Some(url) = &url {
                                    scope.set_extra("mpv.url", url.clone().into());
                                }
                                if !details.is_empty() {
                                    scope.set_extra("mpv.log_tail", details.join("\n").into());
                                }
                            },
                            || sentry::capture_message(&message, sentry::Level::Error),
                        );
                        events.push(PlayerEvent::EndFile {
                            eof: false,
                            error: Some(message),
                        });
                    } else if end_file.reason == MPV_END_FILE_REASON_EOF {
                        events.push(PlayerEvent::EndFile {
                            eof: true,
                            error: None,
                        });
                    }
                }
                MPV_EVENT_COMMAND_REPLY if event.error < 0 => {
                    log::debug!(
                        "mpv async command failed: {}",
                        self.api.error_string(event.error)
                    );
                }
                MPV_EVENT_PLAYBACK_RESTART => {
                    if let Some(secs) = self.pending_seek_seconds.take() {
                        self.waiting_for_seek_restart = true;
                        self.frame_ready_to_restore_audio = false;
                        let _ = self.command_string(&format!("seek {secs:.3} absolute+exact"));
                    } else if self.waiting_for_seek_restart {
                        self.waiting_for_seek_restart = false;
                        self.frame_ready_to_restore_audio = false;
                    }
                    if self.pending_unpause && !self.waiting_for_seek_restart {
                        self.pending_unpause = false;
                        let _ = self.command_string("set pause no");
                    }
                }
                _ => {}
            }
        }
        self.restore_audio_only();
        events
    }

    fn track_list_status(&self) -> (bool, bool) {
        let count = self
            .get_string_property("track-list/count")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        if count == 0 {
            return (false, false);
        }
        for index in 0..count {
            if self
                .get_string_property(&format!("track-list/{index}/type"))
                .as_deref()
                == Some("video")
            {
                return (true, true);
            }
        }
        (false, true)
    }

    pub fn status(&self) -> PlayerStatus {
        let (has_video_track, track_list_ready) = self.track_list_status();
        PlayerStatus {
            loaded: self.loaded,
            path: self.get_string_property("path"),
            media_title: self.get_string_property("media-title"),
            time_pos: self.get_string_property("time-pos"),
            duration: self.get_string_property("duration"),
            percent_pos: self.get_string_property("percent-pos"),
            pause: self.get_string_property("pause"),
            mute: self.get_string_property("mute"),
            volume: self.get_string_property("volume"),
            core_idle: self.get_string_property("core-idle"),
            eof_reached: self.get_string_property("eof-reached"),
            vo_configured: self.get_string_property("vo-configured"),
            video_codec: self.get_string_property("video-codec"),
            video_format: self.get_string_property("video-format"),
            width: self.get_string_property("width"),
            height: self.get_string_property("height"),
            cache_speed: self.get_string_property("cache-speed"),
            demuxer_cache_duration: self.get_string_property("demuxer-cache-duration"),
            hwdec_current: self.get_string_property("hwdec-current"),
            fps: self.get_string_property("estimated-vf-fps"),
            frame_drop_count: self.get_string_property("frame-drop-count"),
            decoder_frame_drop_count: self.get_string_property("decoder-frame-drop-count"),
            avsync: self.get_string_property("avsync"),
            video_bitrate: self.get_string_property("video-bitrate"),
            audio_bitrate: self.get_string_property("audio-bitrate"),
            audio_codec: self.get_string_property("audio-codec-name"),
            audio_samplerate: self.get_string_property("audio-params/samplerate"),
            audio_channels: self.get_string_property("audio-params/channels"),
            color_primaries: self.get_string_property("video-params/primaries"),
            color_matrix: self.get_string_property("video-params/colormatrix"),
            color_gamma: self.get_string_property("video-params/gamma"),
            video_out_primaries: self.get_string_property("video-out-params/primaries"),
            video_out_matrix: self.get_string_property("video-out-params/colormatrix"),
            video_out_gamma: self.get_string_property("video-out-params/gamma"),
            sig_peak: self.get_string_property("video-params/sig-peak"),
            container_fps: self.get_string_property("container-fps"),
            display_fps: self.get_string_property("display-fps"),
            mistimed_frame_count: self.get_string_property("mistimed-frame-count"),
            vo_delayed_frame_count: self.get_string_property("vo-delayed-frame-count"),
            paused_for_cache: self.get_string_property("paused-for-cache"),
            cache_buffering_state: self.get_string_property("cache-buffering-state"),
            file_format: self.get_string_property("file-format"),
            frames_rendered: self.frames_rendered,
            first_frame_presented: self.first_frame_presented,
            has_video_track,
            track_list_ready,
            resuming: self.pending_seek_seconds.is_some() || self.waiting_for_seek_restart,
        }
    }

    pub fn track_options(&self, track_type: &str) -> Vec<PlayerTrackOption> {
        let count = self
            .get_string_property("track-list/count")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        let mut tracks = Vec::new();
        for index in 0..count {
            let Some(kind) = self.get_string_property(&format!("track-list/{index}/type")) else {
                continue;
            };
            if kind != track_type {
                continue;
            }
            let id = self
                .get_string_property(&format!("track-list/{index}/id"))
                .unwrap_or_else(|| (index + 1).to_string());
            let title = self
                .get_string_property(&format!("track-list/{index}/title"))
                .filter(|value| !value.trim().is_empty());
            let lang = self
                .get_string_property(&format!("track-list/{index}/lang"))
                .filter(|value| !value.trim().is_empty());
            let external_filename = self
                .get_string_property(&format!("track-list/{index}/external-filename"))
                .and_then(|value| {
                    std::path::Path::new(&value)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(str::to_string)
                })
                .filter(|value| !value.trim().is_empty());
            let external = self
                .get_string_property(&format!("track-list/{index}/external"))
                .map(|value| value == "yes")
                .unwrap_or(false);
            let codec = self
                .get_string_property(&format!("track-list/{index}/codec"))
                .filter(|value| !value.trim().is_empty());
            let selected = self
                .get_string_property(&format!("track-list/{index}/selected"))
                .map(|value| value == "yes")
                .unwrap_or(false);
            let fallback = if track_type == "audio" {
                format!("Audio {}", tracks.len() + 1)
            } else {
                format!("Subtitle {}", tracks.len() + 1)
            };
            let source = if external {
                title.clone().or_else(|| external_filename.clone())
            } else {
                title.clone()
            };
            let format = if track_type == "audio" {
                let channels = self
                    .get_string_property(&format!("track-list/{index}/demux-channel-count"))
                    .and_then(|value| value.parse::<u32>().ok());
                let channel_label = audio_channel_label(channels);
                match (channel_label, codec.as_deref()) {
                    (Some(ch), Some(c)) => Some(format!("{ch} · {}", c.to_uppercase())),
                    (Some(ch), None) => Some(ch),
                    (None, Some(c)) => Some(c.to_uppercase()),
                    (None, None) => None,
                }
            } else {
                codec.as_deref().map(subtitle_format_label)
            };
            tracks.push(PlayerTrackOption {
                id,
                label: title
                    .clone()
                    .or(external_filename)
                    .or(lang.clone())
                    .unwrap_or(fallback),
                selected,
                lang,
                source,
                external,
                format,
            });
        }
        tracks
    }

    pub fn set_option(&self, name: &str, value: &str) -> Result<(), String> {
        let c_name = CString::new(name).map_err(|error| error.to_string())?;
        let c_value = CString::new(value).map_err(|error| error.to_string())?;
        let result = unsafe {
            (self.api.mpv_set_option_string)(self.handle, c_name.as_ptr(), c_value.as_ptr())
        };
        if result < 0 {
            Err(format!(
                "mpv option '{name}' failed: {}",
                self.api.error_string(result)
            ))
        } else {
            Ok(())
        }
    }

    pub fn set_http_headers(&self, headers: &[(String, String)]) -> Result<(), String> {
        if headers.is_empty() {
            return self.set_option("http-header-fields", "");
        }
        let joined = headers
            .iter()
            .map(|(key, value)| format!("{key}: {value}").replace(',', "\\,"))
            .collect::<Vec<_>>()
            .join(",");
        self.set_option("http-header-fields", &joined)
    }

    pub fn apply_options(&self, options: &[(String, String)]) -> Result<(), String> {
        for (name, value) in options {
            let result = if name == "glsl-shaders" {
                if value.is_empty() {
                    self.command_args(&["change-list", name, "clr", ""])
                } else {
                    self.command_args(&["change-list", name, "set", value])
                }
            } else {
                self.command_args(&["set", name, value])
            };
            if let Err(error) = result {
                log::warn!("mpv preference skipped: {error}");
            }
        }
        Ok(())
    }

    fn create_software_context(&mut self) -> Result<(), String> {
        let api_type = CString::new("sw").unwrap();
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_API_TYPE,
                data: api_type.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let mut context: *mut MpvRenderContext = ptr::null_mut();
        let result = unsafe {
            (self.api.mpv_render_context_create)(&mut context, self.handle, params.as_mut_ptr())
        };
        if result < 0 {
            Err(format!(
                "mpv software render context failed: {}",
                self.api.error_string(result)
            ))
        } else if context.is_null() {
            Err("mpv software render context returned null".to_string())
        } else {
            self.render_context = context;
            Ok(())
        }
    }

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    fn create_opengl_context(&mut self) -> Result<(), String> {
        let api_type = CString::new("opengl").unwrap();
        let mut init_params = MpvOpenGlInitParams {
            get_proc_address: Some(get_gl_proc_address),
            get_proc_address_ctx: ptr::null_mut(),
        };
        let mut params = vec![
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_API_TYPE,
                data: api_type.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,
                data: (&mut init_params as *mut MpvOpenGlInitParams).cast(),
            },
        ];
        // Without one of these, mpv's gpu-next libmpv render backend has no way to
        // open a VADisplay, so VAAPI hwdec silently never initializes (falls back to copy).
        #[cfg(target_os = "linux")]
        if let Some(display) = wl_display_ptr() {
            params.push(MpvRenderParam {
                param_type: MPV_RENDER_PARAM_WL_DISPLAY,
                data: display,
            });
        } else if let Some(display) = x11_display_ptr() {
            params.push(MpvRenderParam {
                param_type: MPV_RENDER_PARAM_X11_DISPLAY,
                data: display,
            });
        }
        params.push(MpvRenderParam {
            param_type: MPV_RENDER_PARAM_INVALID,
            data: ptr::null_mut(),
        });
        let mut context: *mut MpvRenderContext = ptr::null_mut();
        let result = unsafe {
            (self.api.mpv_render_context_create)(&mut context, self.handle, params.as_mut_ptr())
        };
        if result < 0 {
            Err(format!(
                "mpv OpenGL render context failed: {}",
                self.api.error_string(result)
            ))
        } else if context.is_null() {
            Err("mpv OpenGL render context returned null".to_string())
        } else {
            self.render_context = context;
            Ok(())
        }
    }

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    pub fn create_vulkan_context(
        &mut self,
        instance: *mut c_void,
        phys_device: *mut c_void,
        device: *mut c_void,
        queue_graphics_index: u32,
        queue_graphics_count: u32,
        get_proc_address: *mut c_void,
        enabled_extension_ptrs: &[*const c_char],
    ) -> Result<(), String> {
        let api_type = CString::new("vulkan").unwrap();
        let mut init_params = MpvVulkanInitParams {
            instance,
            phys_device,
            device,
            get_proc_address,
            queue_graphics_index,
            queue_graphics_count,
            enabled_extensions: enabled_extension_ptrs.as_ptr(),
            num_enabled_extensions: enabled_extension_ptrs.len() as i32,
        };
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_API_TYPE,
                data: api_type.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_VULKAN_INIT_PARAMS,
                data: (&mut init_params as *mut MpvVulkanInitParams).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let mut context: *mut MpvRenderContext = ptr::null_mut();
        let result = unsafe {
            (self.api.mpv_render_context_create)(&mut context, self.handle, params.as_mut_ptr())
        };
        if result < 0 {
            Err(format!(
                "mpv Vulkan render context failed: {}",
                self.api.error_string(result)
            ))
        } else if context.is_null() {
            Err("mpv Vulkan render context returned null".to_string())
        } else {
            self.render_context = context;
            Ok(())
        }
    }

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    pub fn render_vulkan_frame(&mut self, image: &mut VulkanTargetImage) -> Result<(), String> {
        if self.render_context.is_null() {
            return Err("vulkan render context not created".to_string());
        }
        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };

        let mut ffi_image = MpvVulkanImageFfi {
            image: image.image,
            format: image.format,
            w: image.w,
            h: image.h,
            usage: image.usage,
            layout: image.layout,
            wait_semaphore: image.wait_semaphore,
            signal_semaphore: image.signal_semaphore,
        };
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_VULKAN_IMAGE,
                data: (&mut ffi_image as *mut MpvVulkanImageFfi).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        image.layout = ffi_image.layout;
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render (vulkan) failed: {}",
                self.api.error_string(result)
            ));
        }
        if update_flags & MPV_RENDER_UPDATE_FRAME != 0 {
            self.frames_rendered = self.frames_rendered.saturating_add(1);
            self.frame_ready_to_restore_audio = true;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn create_d3d11_context(&mut self, device: *mut c_void) -> Result<(), String> {
        let api_type = CString::new("d3d11").unwrap();
        let mut init_params = MpvD3d11InitParams { device };
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_API_TYPE,
                data: api_type.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_D3D11_INIT_PARAMS,
                data: (&mut init_params as *mut MpvD3d11InitParams).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let mut context: *mut MpvRenderContext = ptr::null_mut();
        let result = unsafe {
            (self.api.mpv_render_context_create)(&mut context, self.handle, params.as_mut_ptr())
        };
        if result < 0 {
            Err(format!(
                "mpv D3D11 render context failed: {}",
                self.api.error_string(result)
            ))
        } else if context.is_null() {
            Err("mpv D3D11 render context returned null".to_string())
        } else {
            self.render_context = context;
            Ok(())
        }
    }

    #[cfg(target_os = "windows")]
    pub fn render_d3d11_frame(
        &mut self,
        tex: *mut c_void,
        format: i32,
        w: i32,
        h: i32,
    ) -> Result<(), String> {
        if self.render_context.is_null() {
            return Err("d3d11 render context not created".to_string());
        }
        let update_flags = unsafe { (self.api.mpv_render_context_update)(self.render_context) };

        let mut target = MpvD3d11Target { tex, format, w, h };
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_D3D11_TARGET,
                data: (&mut target as *mut MpvD3d11Target).cast(),
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let result = unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr())
        };
        if result < 0 {
            return Err(format!(
                "mpv_render_context_render (d3d11) failed: {}",
                self.api.error_string(result)
            ));
        }
        if update_flags & MPV_RENDER_UPDATE_FRAME != 0 {
            self.frames_rendered = self.frames_rendered.saturating_add(1);
            self.frame_ready_to_restore_audio = true;
        }
        Ok(())
    }

    fn ensure_buffer(&mut self, width: i32, height: i32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        self.buffer
            .resize((width as usize) * (height as usize) * 4, 0);
    }

    pub fn query_property(&self, name: &str) -> Option<String> {
        self.get_string_property(name)
    }

    fn get_string_property(&self, name: &str) -> Option<String> {
        let c_name = CString::new(name).ok()?;
        let mut value: *mut c_char = ptr::null_mut();
        let result = unsafe {
            (self.api.mpv_get_property)(
                self.handle,
                c_name.as_ptr(),
                1,
                (&mut value as *mut *mut c_char).cast(),
            )
        };
        if result < 0 || value.is_null() {
            return None;
        }
        let text = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() };
        unsafe { (self.api.mpv_free)(value.cast()) };
        Some(text)
    }
}

// Linux-only OpenGL proc address resolution

#[cfg(target_os = "linux")]
type GlProcFn = unsafe extern "C" fn(*const c_uchar) -> *mut c_void;

#[cfg(target_os = "linux")]
type GlGetIntegerv = unsafe extern "C" fn(pname: c_uint, params: *mut c_int);

#[cfg(target_os = "linux")]
const GL_DRAW_FRAMEBUFFER_BINDING: c_uint = 0x8CA6;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod platform_gl;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use platform_gl::*;

impl Drop for MpvRenderer {
    fn drop(&mut self) {
        unsafe {
            if !self.render_context.is_null() {
                (self.api.mpv_render_context_free)(self.render_context);
            }
            if !self.handle.is_null() {
                (self.api.mpv_terminate_destroy)(self.handle);
            }
        }
    }
}

fn load_error(error: libloading::Error) -> String {
    error.to_string()
}

#[cfg(target_os = "windows")]
fn load_library(path: &str) -> Result<Library, String> {
    use libloading::os::windows::Library as WinLibrary;
    use std::error::Error as _;
    const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: u32 = 0x0000_1000;
    const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: u32 = 0x0000_0100;
    unsafe {
        WinLibrary::load_with_flags(
            path,
            LOAD_LIBRARY_SEARCH_DEFAULT_DIRS | LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
        )
    }
    .map(Library::from)
    .map_err(|error| match error.source() {
        Some(source) => format!("{error} ({source})"),
        None => error.to_string(),
    })
}

#[cfg(not(target_os = "windows"))]
fn load_library(path: &str) -> Result<Library, String> {
    unsafe { Library::new(path) }.map_err(|error| error.to_string())
}

pub(crate) fn find_libmpv_path() -> String {
    #[cfg(target_os = "windows")]
    let lib_names: &[&str] = &["mpv-2.dll", "libmpv-2.dll", "libmpv.dll"];
    #[cfg(target_os = "macos")]
    let lib_names: &[&str] = &["libmpv.dylib", "libmpv.2.dylib", "libmpv.1.dylib"];
    #[cfg(target_os = "linux")]
    let lib_names: &[&str] = &["libmpv.so.2.5.0", "libmpv.so.2", "libmpv.so"];
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let lib_names: &[&str] = &[];

    let mut search_dirs: Vec<PathBuf> = Vec::new();

    // Beside the executable (bundled distribution)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            search_dirs.push(exe_dir.to_path_buf());
            search_dirs.push(exe_dir.join("lib"));

            #[cfg(target_os = "macos")]
            if let Some(contents_dir) = exe_dir.parent() {
                search_dirs.push(contents_dir.join("Resources").join("lib"));
            }

            #[cfg(target_os = "linux")]
            if let Some(prefix_dir) = exe_dir.parent() {
                let lib_dir = prefix_dir.join("lib");
                if let Ok(entries) = std::fs::read_dir(&lib_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            search_dirs.push(path.join("lib"));
                        }
                    }
                }
            }
        }
    }

    // Cargo dev builds
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        search_dirs.push(PathBuf::from(&manifest_dir).join("lib"));
    }

    // macOS: Homebrew (Intel and Apple Silicon)
    #[cfg(target_os = "macos")]
    {
        search_dirs.push(PathBuf::from("/opt/homebrew/lib"));
        search_dirs.push(PathBuf::from("/usr/local/lib"));
    }

    for dir in &search_dirs {
        for lib_name in lib_names {
            let path = dir.join(lib_name);
            if path.exists() {
                return path.to_string_lossy().into_owned();
            }
        }
    }

    // Fall back to the system dynamic linker
    #[cfg(target_os = "windows")]
    return "mpv-2.dll".to_string();
    #[cfg(target_os = "macos")]
    return "libmpv.dylib".to_string();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return "libmpv.so.2".to_string();
}
