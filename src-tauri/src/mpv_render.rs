use base64::{engine::general_purpose, Engine as _};
use libloading::Library;
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
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
const MPV_EVENT_PROPERTY_CHANGE: c_int = 22;
const MPV_END_FILE_REASON_EOF: c_int = 0;
const MPV_END_FILE_REASON_ERROR: c_int = 4;
const MPV_FORMAT_FLAG: c_int = 3;
const PAUSE_OBSERVE_ID: u64 = 1001;

#[repr(C)]
struct MpvEvent {
    event_id: c_int,
    error: c_int,
    reply_userdata: u64,
    data: *mut c_void,
}

#[repr(C)]
struct MpvEventProperty {
    name: *const c_char,
    format: c_int,
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
    PauseChanged(bool),
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
type MpvObserveProperty =
    unsafe extern "C" fn(*mut MpvHandle, u64, *const c_char, c_int) -> c_int;

pub struct MpvFrameState {
    loaded: AtomicBool,
    frames_rendered: AtomicU64,
    first_frame_presented: AtomicBool,
    frame_ready_to_restore_audio: AtomicBool,
    waiting_for_seek_restart: AtomicBool,
    pending_seek_active: AtomicBool,
    #[cfg(target_os = "windows")]
    muted_until_first_frame: AtomicBool,
    #[cfg(target_os = "windows")]
    restore_mute_value: AtomicBool,
}

impl MpvFrameState {
    fn new() -> Self {
        Self {
            loaded: AtomicBool::new(false),
            frames_rendered: AtomicU64::new(0),
            first_frame_presented: AtomicBool::new(false),
            frame_ready_to_restore_audio: AtomicBool::new(false),
            waiting_for_seek_restart: AtomicBool::new(false),
            pending_seek_active: AtomicBool::new(false),
            #[cfg(target_os = "windows")]
            muted_until_first_frame: AtomicBool::new(false),
            #[cfg(target_os = "windows")]
            restore_mute_value: AtomicBool::new(false),
        }
    }
}

pub struct MpvClientHandle {
    api: Arc<MpvApi>,
    handle: *mut MpvHandle,
    frame_state: Arc<MpvFrameState>,
    log_ring: std::collections::VecDeque<(c_int, String)>,
    pending_unpause: bool,
    pending_seek_seconds: Option<f64>,
    current_url: Option<String>,
    next_async_command_id: AtomicU64,
}

unsafe impl Send for MpvClientHandle {}

pub struct MpvRenderState {
    api: Arc<MpvApi>,
    handle: *mut MpvHandle,
    render_context: *mut MpvRenderContext,
    buffer: Vec<u8>,
    width: i32,
    height: i32,
    frame_state: Arc<MpvFrameState>,
}

unsafe impl Send for MpvRenderState {}

pub struct MpvThumbnailRenderer {
    client: MpvClientHandle,
    render: MpvRenderState,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerFrame {
    width: i32,
    height: i32,
    pixels_base64: String,
}

#[derive(Clone, Serialize)]
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
    pub hdr_active: bool,
    pub container_fps: Option<String>,
    pub display_fps: Option<String>,
    pub mistimed_frame_count: Option<String>,
    pub vo_delayed_frame_count: Option<String>,
    pub paused_for_cache: Option<String>,
    pub cache_buffering_state: Option<String>,
    pub seeking: Option<String>,
    pub file_format: Option<String>,
    pub frames_rendered: u64,
    pub first_frame_presented: bool,
    pub has_video_track: bool,
    pub track_list_ready: bool,
    pub resuming: bool,
}

#[derive(Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStaticStatus {
    pub loaded: bool,
    pub path: Option<String>,
    pub media_title: Option<String>,
    pub duration: Option<String>,
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
    pub hwdec_current: Option<String>,
    pub fps: Option<String>,
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
    pub hdr_active: bool,
    pub container_fps: Option<String>,
    pub display_fps: Option<String>,
    pub file_format: Option<String>,
    pub first_frame_presented: bool,
    pub has_video_track: bool,
    pub track_list_ready: bool,
    pub resuming: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerPositionStatus {
    pub time_pos: Option<String>,
    pub percent_pos: Option<String>,
    pub cache_speed: Option<String>,
    pub demuxer_cache_duration: Option<String>,
    pub frame_drop_count: Option<String>,
    pub decoder_frame_drop_count: Option<String>,
    pub avsync: Option<String>,
    pub video_bitrate: Option<String>,
    pub audio_bitrate: Option<String>,
    pub mistimed_frame_count: Option<String>,
    pub vo_delayed_frame_count: Option<String>,
    pub paused_for_cache: Option<String>,
    pub cache_buffering_state: Option<String>,
    pub seeking: Option<String>,
    pub frames_rendered: u64,
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
    pub fn static_status(&self) -> PlayerStaticStatus {
        PlayerStaticStatus {
            loaded: self.loaded,
            path: self.path.clone(),
            media_title: self.media_title.clone(),
            duration: self.duration.clone(),
            pause: self.pause.clone(),
            mute: self.mute.clone(),
            volume: self.volume.clone(),
            core_idle: self.core_idle.clone(),
            eof_reached: self.eof_reached.clone(),
            vo_configured: self.vo_configured.clone(),
            video_codec: self.video_codec.clone(),
            video_format: self.video_format.clone(),
            width: self.width.clone(),
            height: self.height.clone(),
            hwdec_current: self.hwdec_current.clone(),
            fps: self.fps.clone(),
            audio_codec: self.audio_codec.clone(),
            audio_samplerate: self.audio_samplerate.clone(),
            audio_channels: self.audio_channels.clone(),
            color_primaries: self.color_primaries.clone(),
            color_matrix: self.color_matrix.clone(),
            color_gamma: self.color_gamma.clone(),
            video_out_primaries: self.video_out_primaries.clone(),
            video_out_matrix: self.video_out_matrix.clone(),
            video_out_gamma: self.video_out_gamma.clone(),
            sig_peak: self.sig_peak.clone(),
            hdr_active: self.hdr_active,
            container_fps: self.container_fps.clone(),
            display_fps: self.display_fps.clone(),
            file_format: self.file_format.clone(),
            first_frame_presented: self.first_frame_presented,
            has_video_track: self.has_video_track,
            track_list_ready: self.track_list_ready,
            resuming: self.resuming,
        }
    }

    pub fn position_status(&self) -> PlayerPositionStatus {
        PlayerPositionStatus {
            time_pos: self.time_pos.clone(),
            percent_pos: self.percent_pos.clone(),
            cache_speed: self.cache_speed.clone(),
            demuxer_cache_duration: self.demuxer_cache_duration.clone(),
            frame_drop_count: self.frame_drop_count.clone(),
            decoder_frame_drop_count: self.decoder_frame_drop_count.clone(),
            avsync: self.avsync.clone(),
            video_bitrate: self.video_bitrate.clone(),
            audio_bitrate: self.audio_bitrate.clone(),
            mistimed_frame_count: self.mistimed_frame_count.clone(),
            vo_delayed_frame_count: self.vo_delayed_frame_count.clone(),
            paused_for_cache: self.paused_for_cache.clone(),
            cache_buffering_state: self.cache_buffering_state.clone(),
            seeking: self.seeking.clone(),
            frames_rendered: self.frames_rendered,
        }
    }

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

mod commands;
mod context;
mod frame;
mod lifecycle;
mod status;

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
