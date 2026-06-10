// macOS native player surface — libmpv OpenGL render API via NSOpenGLContext.
//
// A child NSView is inserted behind the Tauri WKWebView using ObjC messaging.
// NSOpenGLContext renders mpv frames into it every 16 ms.
// Window size is queried via Tauri (no NSRect stret needed).

use crate::DesktopState;
use std::ffi::{c_void, CString};
use std::sync::{mpsc, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

// ── ObjC runtime types ─────────────────────────────────────────────────────────

type Id = *mut c_void;

// All ObjC calls go through these two entry points.
extern "C" {
    fn objc_getClass(name: *const i8) -> Id;
    fn objc_msgSend(receiver: Id, sel: Id, ...) -> Id;
    fn sel_registerName(name: *const i8) -> Id;
}

unsafe fn cls(name: &str) -> Id {
    let s = CString::new(name).unwrap();
    objc_getClass(s.as_ptr())
}
unsafe fn sel(name: &str) -> Id {
    let s = CString::new(name).unwrap();
    sel_registerName(s.as_ptr())
}
// msg0: send with no extra args, return id
unsafe fn msg0(obj: Id, sel_name: &str) -> Id {
    objc_msgSend(obj, sel(sel_name))
}
// msg1_id: send with one id arg
unsafe fn msg1_id(obj: Id, sel_name: &str, arg: Id) -> Id {
    type Fn = unsafe extern "C" fn(Id, Id, Id) -> Id;
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel(sel_name), arg)
}
// msg1_bool: send with one BOOL arg
unsafe fn msg1_bool(obj: Id, sel_name: &str, b: i8) -> Id {
    type Fn = unsafe extern "C" fn(Id, Id, i8) -> Id;
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel(sel_name), b)
}
// msg2_id_id: send with two id args
unsafe fn msg2_id_id(obj: Id, sel_name: &str, a: Id, b: Id) -> Id {
    type Fn = unsafe extern "C" fn(Id, Id, Id, Id) -> Id;
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel(sel_name), a, b)
}
// msg3_id_isize_id: addSubview:positioned:relativeTo:
unsafe fn msg3_positioned(obj: Id, sub: Id, order: isize, rel: Id) -> Id {
    type Fn = unsafe extern "C" fn(Id, Id, Id, isize, Id) -> Id;
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel("addSubview:positioned:relativeTo:"), sub, order, rel)
}

// NSRect passed BY VALUE to initWithFrame: and setFrame: — no stret needed.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct NSPoint {
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct NSSize {
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

unsafe fn msg_init_with_frame(obj: Id, frame: NSRect) -> Id {
    type Fn = unsafe extern "C" fn(Id, Id, NSRect) -> Id;
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel("initWithFrame:"), frame)
}
unsafe fn msg_set_frame(obj: Id, frame: NSRect) {
    type Fn = unsafe extern "C" fn(Id, Id, NSRect);
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel("setFrame:"), frame)
}

// NSOpenGLPixelFormatAttribute constants.
const NSOpenGLPFADoubleBuffer: u32 = 5;
const NSOpenGLPFAAccelerated: u32 = 73;
const NSOpenGLPFAColorSize: u32 = 8;
const NSOpenGLPFADepthSize: u32 = 12;
const NSOpenGLPFAOpenGLProfile: u32 = 99;
const NSOpenGLProfileVersionLegacy: u32 = 0x1000;

// ── Send wrappers ──────────────────────────────────────────────────────────────

#[derive(Clone)]
struct SendId(Id);
unsafe impl Send for SendId {}
unsafe impl Sync for SendId {}

// ── GCD dispatch helpers ───────────────────────────────────────────────────────

// Load libdispatch symbols once.
fn libdispatch() -> *mut c_void {
    static HANDLE: OnceLock<usize> = OnceLock::new();
    let h = *HANDLE.get_or_init(|| unsafe {
        libc::dlopen(
            b"/usr/lib/system/libdispatch.dylib\0".as_ptr() as _,
            libc::RTLD_LAZY,
        ) as usize
    });
    h as *mut c_void
}

unsafe fn gcd_main_queue() -> *mut c_void {
    let lib = libdispatch();
    if lib.is_null() { return std::ptr::null_mut(); }
    let f: unsafe extern "C" fn() -> *mut c_void = std::mem::transmute(
        libc::dlsym(lib, b"dispatch_get_main_queue\0".as_ptr() as _),
    );
    f()
}

unsafe fn gcd_async_f(
    queue: *mut c_void,
    ctx: *mut c_void,
    work: unsafe extern "C" fn(*mut c_void),
) {
    let lib = libdispatch();
    if lib.is_null() { return; }
    let f: unsafe extern "C" fn(*mut c_void, *mut c_void, unsafe extern "C" fn(*mut c_void)) =
        std::mem::transmute(libc::dlsym(lib, b"dispatch_async_f\0".as_ptr() as _));
    f(queue, ctx, work);
}

// ── Surface commands ───────────────────────────────────────────────────────────

enum SurfaceCommand {
    Load { url: String, start_at: Option<u64>, total_duration: Option<u64> },
    Hide,
    ShowLoading { title: String, episode_title: Option<String> },
    SetTitle { title: String, episode_title: Option<String> },
    SetArtwork {
        title: String,
        episode_title: Option<String>,
        background: Option<(Vec<u8>, i32, i32)>,
        logo: Option<(Vec<u8>, i32, i32)>,
    },
}

#[derive(Clone)]
pub struct NativePlayerSurface {
    sender: mpsc::Sender<SurfaceCommand>,
}

impl NativePlayerSurface {
    pub fn load(&self, url: String, start_at: Option<u64>, total_duration: Option<u64>) -> Result<(), String> {
        self.sender
            .send(SurfaceCommand::Load { url, start_at, total_duration })
            .map_err(|e| format!("surface unavailable: {e}"))
    }
    pub fn hide(&self) { let _ = self.sender.send(SurfaceCommand::Hide); }
    pub fn show_loading(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::ShowLoading { title, episode_title });
    }
    pub fn set_title(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::SetTitle { title, episode_title });
    }
    pub fn set_artwork(
        &self,
        title: String,
        episode_title: Option<String>,
        background: Option<(Vec<u8>, i32, i32)>,
        logo: Option<(Vec<u8>, i32, i32)>,
    ) {
        let _ = self.sender.send(SurfaceCommand::SetArtwork {
            title, episode_title, background, logo,
        });
    }
}

// ── install ────────────────────────────────────────────────────────────────────

pub fn install(app_handle: AppHandle) -> Result<NativePlayerSurface, String> {
    let (sender, receiver) = mpsc::channel::<SurfaceCommand>();

    // Get the Tauri window's NSView.
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let ns_view: Id = match window
        .window_handle()
        .map_err(|e| e.to_string())?
        .as_ref()
    {
        RawWindowHandle::AppKit(h) => h.ns_view.as_ptr() as Id,
        _ => return Err("unexpected window handle type on macOS".to_string()),
    };

    // Get the initial window size via Tauri (avoids NSRect stret).
    let init_size = window.inner_size().unwrap_or(tauri::PhysicalSize::new(1280, 720));
    let init_w = init_size.width.max(2) as i32;
    let init_h = init_size.height.max(2) as i32;

    // Create the render subview on the main thread, collect result.
    let (view_tx, view_rx) = mpsc::channel::<Result<SendId, String>>();
    let ctx_struct = Box::new((
        SendId(ns_view),
        init_w as f64,
        init_h as f64,
        Box::into_raw(Box::new(view_tx)) as usize,
    ));
    let ctx_ptr = Box::into_raw(ctx_struct) as *mut c_void;

    extern "C" fn create_on_main(ctx: *mut c_void) {
        unsafe {
            let (parent, w, h, tx_addr) =
                *Box::from_raw(ctx as *mut (SendId, f64, f64, usize));
            let tx = Box::from_raw(
                tx_addr as *mut mpsc::Sender<Result<SendId, String>>,
            );
            let result = create_render_subview(parent, w, h);
            let _ = tx.send(result);
        }
    }

    unsafe {
        let queue = gcd_main_queue();
        if queue.is_null() {
            // Fallback: run inline (works if we're already on the main thread).
            create_on_main(ctx_ptr);
        } else {
            gcd_async_f(queue, ctx_ptr, create_on_main);
        }
    }

    let render_view = view_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "macOS render view creation timed out".to_string())
        .and_then(|r| r)?;

    // Create NSOpenGLContext.
    let gl_ctx = unsafe { create_gl_context(render_view.clone())? };

    // Make it current on this thread and init mpv render context.
    unsafe {
        msg0(gl_ctx.0, "makeCurrentContext");
    }

    {
        let state = app_handle.state::<DesktopState>();
        let mut renderer = state.player_renderer.lock().unwrap();
        if renderer.is_none() {
            match crate::mpv_render::MpvRenderer::new() {
                Ok(r) => *renderer = Some(r),
                Err(e) => {
                    return Err(format!("mpv init failed: {e}"));
                }
            }
        }
        if let Some(r) = renderer.as_mut() {
            r.prepare_opengl_context()
                .map_err(|e| format!("mpv GL context failed: {e}"))?;
        }
    }

    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
    let app = app_handle.clone();

    // usize is Send; this is the standard pattern for passing ObjC raw pointers
    // across a thread boundary when the caller guarantees exclusive access.
    let gl_ctx_usize: usize = gl_ctx.0 as usize;
    let render_view_usize: usize = render_view.0 as usize;

    std::thread::spawn(move || {
        let gl: *mut c_void = gl_ctx_usize as _;
        let rv: *mut c_void = render_view_usize as _;

        // Make context current on this render thread.
        unsafe { msg0(gl, "makeCurrentContext") };
        let _ = ready_tx.send(Ok(()));

        let mut visible = false;
        let mut last_size = (init_w, init_h);

        loop {
            while let Ok(cmd) = receiver.try_recv() {
                match cmd {
                    SurfaceCommand::Load { url, start_at, .. } => {
                        unsafe { msg1_bool(rv, "setHidden:", 0) };
                        visible = true;
                        let _ = app.emit("native-player-show", ());
                        let state = app.state::<DesktopState>();
                        *state.eof_next_fired.lock().unwrap() = false;
                        let mut r = state.player_renderer.lock().unwrap();
                        if let Some(renderer) = r.as_mut() {
                            if let Err(e) = renderer.load(&url, start_at) {
                                drop(r);
                                let _ = app.emit("native-player-error", e);
                                visible = false;
                                unsafe { msg1_bool(rv, "setHidden:", 1) };
                            }
                        }
                    }
                    SurfaceCommand::Hide => {
                        visible = false;
                        unsafe { msg1_bool(rv, "setHidden:", 1) };
                        let _ = app.emit("native-player-hide", ());
                        let state = app.state::<DesktopState>();
                        let guard = state.player_renderer.lock().unwrap();
                        if let Some(r) = guard.as_ref() {
                            let _ = r.command_string("stop");
                        }
                    }
                    SurfaceCommand::ShowLoading { title, episode_title } => {
                        let _ = app.emit(
                            "native-player-title",
                            serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                        );
                    }
                    SurfaceCommand::SetTitle { title, episode_title } => {
                        let _ = app.emit(
                            "native-player-title",
                            serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                        );
                    }
                    SurfaceCommand::SetArtwork { title, .. } => {
                        let _ = app.emit(
                            "native-player-title",
                            serde_json::json!({ "title": title }),
                        );
                    }
                }
            }

            if visible {
                // Track window size changes via Tauri.
                if let Some(win) = app.get_webview_window("main") {
                    if let Ok(sz) = win.inner_size() {
                        let nw = (sz.width as i32).max(2);
                        let nh = (sz.height as i32).max(2);
                        if (nw, nh) != last_size {
                            last_size = (nw, nh);
                            unsafe {
                                msg_set_frame(
                                    rv,
                                    NSRect {
                                        origin: NSPoint { x: 0.0, y: 0.0 },
                                        size: NSSize {
                                            width: nw as f64,
                                            height: nh as f64,
                                        },
                                    },
                                );
                                // Notify context that the view was resized.
                                msg0(gl, "update");
                            }
                        }
                    }
                }

                {
                    let state = app.state::<DesktopState>();
                    let mut renderer = state.player_renderer.lock().unwrap();
                    if let Some(r) = renderer.as_mut() {
                        let _ = r.render_opengl_frame(last_size.0, last_size.1);
                    }
                }
                unsafe { msg0(gl, "flushBuffer") };

                check_player_events(&app);
            }

            std::thread::sleep(Duration::from_millis(16));
        }
    });

    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "macOS render thread setup timed out".to_string())
        .and_then(|r| r)
        .map(|()| NativePlayerSurface { sender })
}

// ── ObjC helpers ───────────────────────────────────────────────────────────────

unsafe fn create_render_subview(parent: SendId, w: f64, h: f64) -> Result<SendId, String> {
    let ns_view_cls = cls("NSView");
    if ns_view_cls.is_null() {
        return Err("NSView class not found".to_string());
    }

    let frame = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: NSSize { width: w, height: h },
    };
    let alloc: Id = msg0(ns_view_cls, "alloc");
    let view: Id = msg_init_with_frame(alloc, frame);
    if view.is_null() {
        return Err("NSView initWithFrame: failed".to_string());
    }

    // wantsLayer = YES allows us to control z-ordering.
    msg1_bool(view, "setWantsLayer:", 1);

    // Insert at back (NSWindowBelow = 1), relative to nil = behind everything.
    msg3_positioned(parent.0, view, 1, std::ptr::null_mut());

    // Hidden until playback starts.
    msg1_bool(view, "setHidden:", 1);

    Ok(SendId(view))
}

unsafe fn create_gl_context(render_view: SendId) -> Result<SendId, String> {
    let attribs: [u32; 9] = [
        NSOpenGLPFADoubleBuffer,
        NSOpenGLPFAAccelerated,
        NSOpenGLPFAColorSize, 32,
        NSOpenGLPFADepthSize, 24,
        NSOpenGLPFAOpenGLProfile, NSOpenGLProfileVersionLegacy,
        0,
    ];

    let pf_cls = cls("NSOpenGLPixelFormat");
    let pf_alloc: Id = msg0(pf_cls, "alloc");
    // initWithAttributes: takes a pointer arg, not a struct.
    type InitAttrFn = unsafe extern "C" fn(Id, Id, *const u32) -> Id;
    let init_attr: InitAttrFn = std::mem::transmute(
        objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _,
    );
    let pf: Id = init_attr(pf_alloc, sel("initWithAttributes:"), attribs.as_ptr());
    if pf.is_null() {
        return Err("NSOpenGLPixelFormat init failed (try legacy profile)".to_string());
    }

    let ctx_cls = cls("NSOpenGLContext");
    let ctx_alloc: Id = msg0(ctx_cls, "alloc");
    let ctx: Id = msg2_id_id(
        ctx_alloc,
        "initWithFormat:shareContext:",
        pf,
        std::ptr::null_mut(),
    );
    if ctx.is_null() {
        return Err("NSOpenGLContext init failed".to_string());
    }

    // Attach to the render view.
    msg1_id(ctx, "setView:", render_view.0);

    Ok(SendId(ctx))
}

// ── Player event polling ───────────────────────────────────────────────────────

fn check_player_events(app: &AppHandle) {
    let state = app.state::<DesktopState>();
    let status = {
        let renderer = state.player_renderer.lock().unwrap();
        renderer.as_ref().map(|r| r.status())
    };
    let Some(status) = status else { return };
    if !status.eof_reached() { return; }
    let mut fired = state.eof_next_fired.lock().unwrap();
    if *fired { return; }
    *fired = true;
    drop(fired);

    let next_sub = state.next_ep_subtitle.lock().unwrap().clone();
    let auto_play = *state.auto_play_next_episode.lock().unwrap();
    if !next_sub.is_empty() && auto_play {
        let _ = app.emit("native-player-next-episode", ());
    } else {
        let _ = app.emit("native-player-close-requested", ());
    }
}

// ── Artwork helpers ────────────────────────────────────────────────────────────

pub fn scale_artwork_cover(
    bytes: Vec<u8>,
    target_w: u32,
    target_h: u32,
) -> Option<(Vec<u8>, i32, i32)> {
    let img = image::load_from_memory(&bytes).ok()?;
    let filled = img.resize_to_fill(target_w, target_h, image::imageops::FilterType::Triangle);
    let rgba = filled.to_rgba8();
    Some((rgba.into_raw(), target_w as i32, target_h as i32))
}

pub fn scale_artwork_fit(
    bytes: Vec<u8>,
    max_w: u32,
    max_h: u32,
) -> Option<(Vec<u8>, i32, i32)> {
    let img = image::load_from_memory(&bytes).ok()?;
    let resized = img.resize(max_w, max_h, image::imageops::FilterType::Triangle);
    let (rw, rh) = (resized.width(), resized.height());
    let rgba = resized.to_rgba8();
    Some((rgba.into_raw(), rw as i32, rh as i32))
}
