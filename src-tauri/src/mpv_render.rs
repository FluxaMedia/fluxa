use base64::{engine::general_purpose, Engine as _};
use libloading::Library;
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::path::PathBuf;
use std::ptr;
use std::sync::OnceLock;

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
const MPV_RENDER_PARAM_SW_SIZE: c_int = 17;
const MPV_RENDER_PARAM_SW_FORMAT: c_int = 18;
const MPV_RENDER_PARAM_SW_STRIDE: c_int = 19;
const MPV_RENDER_PARAM_SW_POINTER: c_int = 20;

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

type MpvCreate = unsafe extern "C" fn() -> *mut MpvHandle;
type MpvInitialize = unsafe extern "C" fn(*mut MpvHandle) -> c_int;
type MpvTerminateDestroy = unsafe extern "C" fn(*mut MpvHandle);
type MpvSetOptionString =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, *const c_char) -> c_int;
type MpvCommandString = unsafe extern "C" fn(*mut MpvHandle, *const c_char) -> c_int;
type MpvGetProperty =
    unsafe extern "C" fn(*mut MpvHandle, *const c_char, c_int, *mut c_void) -> c_int;
type MpvFree = unsafe extern "C" fn(*mut c_void);
type MpvErrorString = unsafe extern "C" fn(c_int) -> *const c_char;
type MpvRenderContextCreate =
    unsafe extern "C" fn(*mut *mut MpvRenderContext, *mut MpvHandle, *mut MpvRenderParam) -> c_int;
type MpvRenderContextRender = unsafe extern "C" fn(*mut MpvRenderContext, *mut MpvRenderParam);
type MpvRenderContextFree = unsafe extern "C" fn(*mut MpvRenderContext);

struct MpvApi {
    _library: Library,
    mpv_create: MpvCreate,
    mpv_initialize: MpvInitialize,
    mpv_terminate_destroy: MpvTerminateDestroy,
    mpv_set_option_string: MpvSetOptionString,
    mpv_command_string: MpvCommandString,
    mpv_get_property: MpvGetProperty,
    mpv_free: MpvFree,
    mpv_error_string: MpvErrorString,
    mpv_render_context_create: MpvRenderContextCreate,
    mpv_render_context_render: MpvRenderContextRender,
    mpv_render_context_free: MpvRenderContextFree,
}

unsafe impl Send for MpvApi {}

impl MpvApi {
    fn load() -> Result<Self, String> {
        let lib_path = find_libmpv_path();
        let library = unsafe { Library::new(&lib_path) }
            .map_err(|error| format!("failed to load libmpv from '{lib_path}': {error}"))?;

        unsafe {
            let mpv_create = *library
                .get::<MpvCreate>(b"mpv_create\0")
                .map_err(load_error)?;
            let mpv_initialize = *library
                .get::<MpvInitialize>(b"mpv_initialize\0")
                .map_err(load_error)?;
            let mpv_terminate_destroy = *library
                .get::<MpvTerminateDestroy>(b"mpv_terminate_destroy\0")
                .map_err(load_error)?;
            let mpv_set_option_string = *library
                .get::<MpvSetOptionString>(b"mpv_set_option_string\0")
                .map_err(load_error)?;
            let mpv_command_string = *library
                .get::<MpvCommandString>(b"mpv_command_string\0")
                .map_err(load_error)?;
            let mpv_get_property = *library
                .get::<MpvGetProperty>(b"mpv_get_property\0")
                .map_err(load_error)?;
            let mpv_free = *library.get::<MpvFree>(b"mpv_free\0").map_err(load_error)?;
            let mpv_error_string = *library
                .get::<MpvErrorString>(b"mpv_error_string\0")
                .map_err(load_error)?;
            let mpv_render_context_create = *library
                .get::<MpvRenderContextCreate>(b"mpv_render_context_create\0")
                .map_err(load_error)?;
            let mpv_render_context_render = *library
                .get::<MpvRenderContextRender>(b"mpv_render_context_render\0")
                .map_err(load_error)?;
            let mpv_render_context_free = *library
                .get::<MpvRenderContextFree>(b"mpv_render_context_free\0")
                .map_err(load_error)?;

            Ok(Self {
                _library: library,
                mpv_create,
                mpv_initialize,
                mpv_terminate_destroy,
                mpv_set_option_string,
                mpv_command_string,
                mpv_get_property,
                mpv_free,
                mpv_error_string,
                mpv_render_context_create,
                mpv_render_context_render,
                mpv_render_context_free,
            })
        }
    }

    fn error_string(&self, code: c_int) -> String {
        let ptr = unsafe { (self.mpv_error_string)(code) };
        if ptr.is_null() {
            return format!("mpv error {code}");
        }
        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
    }
}

pub struct MpvRenderer {
    api: MpvApi,
    handle: *mut MpvHandle,
    render_context: *mut MpvRenderContext,
    buffer: Vec<u8>,
    width: i32,
    height: i32,
    loaded: bool,
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
    loaded: bool,
    path: Option<String>,
    media_title: Option<String>,
    time_pos: Option<String>,
    duration: Option<String>,
    percent_pos: Option<String>,
    pause: Option<String>,
    mute: Option<String>,
    volume: Option<String>,
    core_idle: Option<String>,
    eof_reached: Option<String>,
    vo_configured: Option<String>,
    video_format: Option<String>,
    width: Option<String>,
    height: Option<String>,
    cache_speed: Option<String>,
    demuxer_cache_duration: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerTrackOption {
    pub id: String,
    pub label: String,
    pub selected: bool,
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
        };

        renderer.set_option("terminal", "no")?;
        renderer.set_option("config", "no")?;
        renderer.set_option("vo", "libmpv")?;
        renderer.set_option("idle", "yes")?;
        renderer.set_option("keep-open", "yes")?;
        renderer.set_option("osc", "no")?;
        renderer.set_option("osd-level", "0")?;
        renderer.set_option("osd-bar", "no")?;
        renderer.set_option("input-default-bindings", "yes")?;

        // GPU decode — auto-safe tries every platform API (VAAPI/NVDEC/DXVA2/VideoToolbox)
        // and falls back to software decode silently if none is available.
        renderer.set_option("hwdec", "auto-safe")?;
        renderer.set_option("hwdec-codecs", "all")?;

        // video-sync=audio is the correct mode for the libmpv render API.
        // display-resample requires MPV to calibrate display timing from render call
        // intervals, which takes several seconds and holds frames during calibration —
        // causing a black screen and no audio at stream start.
        renderer.set_option("video-sync", "audio")?;

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

        Ok(renderer)
    }

    pub fn load(&mut self, url: &str, start_at: Option<u64>) -> Result<(), String> {
        let escaped = url.replace('\\', "\\\\").replace('"', "\\\"");
        self.loaded = false;
        // Pass start position as a per-file option directly in the loadfile command.
        // This is the most reliable way to seek on open — no timing dependency.
        if let Some(secs) = start_at.filter(|&s| s > 0) {
            self.command_string(&format!("loadfile \"{escaped}\" replace 0 start={secs}"))?;
        } else {
            self.command_string(&format!("loadfile \"{escaped}\" replace"))?;
        }
        self.command_string("set pause no")?;
        self.loaded = true;
        Ok(())
    }

    pub fn add_subtitle(
        &self,
        url: &str,
        title: Option<&str>,
        language: Option<&str>,
    ) -> Result<(), String> {
        let escaped_url = command_quote(url);
        let escaped_title = command_quote(title.unwrap_or("Subtitle"));
        let escaped_language = command_quote(language.unwrap_or(""));
        self.command_string(&format!(
            "sub-add {escaped_url} auto {escaped_title} {escaped_language}"
        ))
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

        unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr());
        }

        for alpha in self.buffer.iter_mut().skip(3).step_by(4) {
            *alpha = 255;
        }

        Ok(PlayerFrame {
            width,
            height,
            pixels_base64: general_purpose::STANDARD.encode(&self.buffer),
        })
    }

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    pub fn render_opengl_frame(&mut self, width: i32, height: i32) -> Result<(), String> {
        if self.render_context.is_null() {
            self.create_opengl_context()?;
        }

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

        unsafe {
            (self.api.mpv_render_context_render)(self.render_context, params.as_mut_ptr());
        }
        Ok(())
    }

    pub fn title(&self) -> Option<String> {
        self.get_string_property("media-title")
    }

    pub fn status(&self) -> PlayerStatus {
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
            video_format: self.get_string_property("video-format"),
            width: self.get_string_property("width"),
            height: self.get_string_property("height"),
            cache_speed: self.get_string_property("cache-speed"),
            demuxer_cache_duration: self.get_string_property("demuxer-cache-duration"),
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
            let selected = self
                .get_string_property(&format!("track-list/{index}/selected"))
                .map(|value| value == "yes")
                .unwrap_or(false);
            let fallback = if track_type == "audio" {
                format!("Audio {}", tracks.len() + 1)
            } else {
                format!("Subtitle {}", tracks.len() + 1)
            };
            tracks.push(PlayerTrackOption {
                id,
                label: title.or(external_filename).or(lang).unwrap_or(fallback),
                selected,
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

    pub fn apply_options(&self, options: &[(String, String)]) -> Result<(), String> {
        for (name, value) in options {
            if let Err(error) = self.set_option(name, value) {
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
        let mut params = [
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_API_TYPE,
                data: api_type.as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                param_type: MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,
                data: (&mut init_params as *mut MpvOpenGlInitParams).cast(),
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

fn command_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

// ── Linux-only OpenGL proc address resolution ──────────────────────────────────

#[cfg(target_os = "linux")]
type GlProcFn = unsafe extern "C" fn(*const c_uchar) -> *mut c_void;

#[cfg(target_os = "linux")]
type GlGetIntegerv = unsafe extern "C" fn(pname: c_uint, params: *mut c_int);

#[cfg(target_os = "linux")]
const GL_DRAW_FRAMEBUFFER_BINDING: c_uint = 0x8CA6;

/// Read the current OpenGL draw framebuffer as RGBA pixels (bottom-to-top row order).
/// Must be called while a GL context is current (i.e. inside GLArea's render callback).
/// Returns width × height × 4 bytes, or None if GL is unavailable.
#[cfg(target_os = "linux")]
pub fn read_gl_pixels_rgba(w: i32, h: i32) -> Option<Vec<u8>> {
    type GlReadPixels = unsafe extern "C" fn(
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        format: c_uint,
        ty: c_uint,
        data: *mut c_void,
    );
    const GL_RGBA: c_uint = 0x1908;
    const GL_UNSIGNED_BYTE: c_uint = 0x1401;

    let Ok(name) = CString::new("glReadPixels") else {
        return None;
    };
    let ptr = unsafe { get_gl_proc_address(ptr::null_mut(), name.as_ptr()) };
    if ptr.is_null() {
        return None;
    }
    let read_pixels: GlReadPixels = unsafe { std::mem::transmute(ptr) };
    let mut buf = vec![0u8; (w * h * 4) as usize];
    unsafe {
        read_pixels(
            0,
            0,
            w,
            h,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            buf.as_mut_ptr() as *mut c_void,
        )
    };

    // OpenGL fills rows bottom-to-top; flip to top-to-bottom for display
    let stride = (w * 4) as usize;
    let rows = h as usize;
    for row in 0..(rows / 2) {
        let mirror = rows - 1 - row;
        for col in 0..stride {
            buf.swap(row * stride + col, mirror * stride + col);
        }
    }
    Some(buf)
}

#[cfg(target_os = "linux")]
fn query_draw_fbo() -> c_int {
    let Ok(name) = CString::new("glGetIntegerv") else {
        return 0;
    };
    let ptr = unsafe { get_gl_proc_address(ptr::null_mut(), name.as_ptr()) };
    if ptr.is_null() {
        return 0;
    }
    let get_integerv: GlGetIntegerv = unsafe { std::mem::transmute(ptr) };
    let mut fbo: c_int = 0;
    unsafe { get_integerv(GL_DRAW_FRAMEBUFFER_BINDING, &mut fbo) };
    fbo
}

#[cfg(target_os = "linux")]
static GL_PROC_FN: OnceLock<Option<GlProcFn>> = OnceLock::new();

#[cfg(target_os = "linux")]
unsafe extern "C" fn get_gl_proc_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void {
    if name.is_null() {
        return ptr::null_mut();
    }
    let Some(f) = *GL_PROC_FN.get_or_init(load_linux_gl_proc_fn) else {
        return ptr::null_mut();
    };
    unsafe { f(name.cast::<c_uchar>()) }
}

/// Resolves the platform's GL proc-address function.
/// Tries EGL first (works on both X11/EGL and Wayland), then falls back to GLX.
#[cfg(target_os = "linux")]
fn load_linux_gl_proc_fn() -> Option<GlProcFn> {
    // EGL covers both Wayland and X11-with-EGL (Mesa default on modern distros)
    for lib_name in &["libEGL.so.1", "libEGL.so"] {
        let Ok(name) = CString::new(*lib_name) else {
            continue;
        };
        let Ok(sym) = CString::new("eglGetProcAddress") else {
            continue;
        };
        let handle = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_LAZY | libc::RTLD_GLOBAL) };
        if handle.is_null() {
            continue;
        }
        let ptr = unsafe { libc::dlsym(handle, sym.as_ptr()) };
        if !ptr.is_null() {
            return Some(unsafe { std::mem::transmute::<*mut c_void, GlProcFn>(ptr) });
        }
    }

    // GLX fallback for pure X11 setups
    for lib_name in &["libGL.so.1", "libGL.so"] {
        let Ok(name) = CString::new(*lib_name) else {
            continue;
        };
        let Ok(sym) = CString::new("glXGetProcAddressARB") else {
            continue;
        };
        let handle = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_LAZY | libc::RTLD_GLOBAL) };
        if handle.is_null() {
            continue;
        }
        let ptr = unsafe { libc::dlsym(handle, sym.as_ptr()) };
        if !ptr.is_null() {
            return Some(unsafe { std::mem::transmute::<*mut c_void, GlProcFn>(ptr) });
        }
    }

    None
}

// ── Windows OpenGL proc address resolution ────────────────────────────────────

#[cfg(target_os = "windows")]
static OPENGL32_HANDLE: OnceLock<isize> = OnceLock::new();

#[cfg(target_os = "windows")]
unsafe extern "C" fn get_gl_proc_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void {
    use windows_sys::Win32::Graphics::OpenGL::wglGetProcAddress;
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
    if name.is_null() {
        return ptr::null_mut();
    }
    // wglGetProcAddress covers extension functions and core GL >= 1.2 on modern drivers.
    let proc = wglGetProcAddress(name as *const u8);
    if let Some(f) = proc {
        let addr = f as usize;
        // Drivers return 1/2/3/-1 as error sentinels.
        if addr > 3 && addr != usize::MAX {
            return addr as *mut c_void;
        }
    }
    // Fallback to opengl32.dll for core GL 1.1 functions (glViewport, glClear, etc.).
    let module = *OPENGL32_HANDLE.get_or_init(|| {
        LoadLibraryA(b"opengl32.dll\0".as_ptr()) as isize
    });
    if module == 0 {
        return ptr::null_mut();
    }
    match GetProcAddress(module as _, name as *const u8) {
        Some(f) => f as *mut c_void,
        None => ptr::null_mut(),
    }
}

// ── macOS OpenGL proc address resolution ──────────────────────────────────────
// Store the handle as usize (Sync-safe) and cast back to pointer when needed.

#[cfg(target_os = "macos")]
static OPENGL_FW_HANDLE: OnceLock<usize> = OnceLock::new();

#[cfg(target_os = "macos")]
unsafe extern "C" fn get_gl_proc_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void {
    if name.is_null() {
        return ptr::null_mut();
    }
    let handle_addr = *OPENGL_FW_HANDLE.get_or_init(|| {
        let path = match std::ffi::CString::new(
            "/System/Library/Frameworks/OpenGL.framework/OpenGL",
        ) {
            Ok(s) => s,
            Err(_) => return 0usize,
        };
        libc::dlopen(path.as_ptr(), libc::RTLD_LAZY | libc::RTLD_GLOBAL) as usize
    });
    if handle_addr == 0 {
        return ptr::null_mut();
    }
    libc::dlsym(handle_addr as *mut c_void, name)
}

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
