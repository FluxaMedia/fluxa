// macOS native player surface — libmpv OpenGL render API via NSOpenGLContext.
//
// A child NSView is inserted behind the Tauri WKWebView using ObjC messaging.
// NSOpenGLContext renders mpv frames into it every 16 ms.
// Window size is queried via Tauri (no NSRect stret needed).

use crate::macos_vulkan::VulkanContext;
use crate::mpv_render::VulkanTargetImage;
use crate::playback_engine::{PlaybackEngine, PlayerEngine};
use crate::DesktopState;
use fluxa_core::FluxaCore;
use std::ffi::{c_void, CString};
use std::sync::{mpsc, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

#[link(name = "QuartzCore", kind = "framework")]
unsafe extern "C" {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RenderBackend {
    OpenGl,
    Vulkan,
}

fn read_render_backend(app: &AppHandle) -> RenderBackend {
    let state = app.state::<DesktopState>();
    match crate::storage::read_pref_field(state, "renderBackend").as_deref() {
        Some("vulkan") => RenderBackend::Vulkan,
        _ => RenderBackend::OpenGl,
    }
}

// ObjC runtime types

type Id = *mut c_void;

// All ObjC calls go through these two entry points.
unsafe extern "C" {
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
unsafe fn msg1_f64(obj: Id, sel_name: &str, v: f64) {
    type Fn = unsafe extern "C" fn(Id, Id, f64);
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel(sel_name), v)
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
    f(
        obj,
        sel("addSubview:positioned:relativeTo:"),
        sub,
        order,
        rel,
    )
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
unsafe fn msg_set_drawable_size(obj: Id, size: NSSize) {
    type Fn = unsafe extern "C" fn(Id, Id, NSSize);
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel("setDrawableSize:"), size)
}

// NSOpenGLPixelFormatAttribute constants.
const NSOpenGLPFADoubleBuffer: u32 = 5;
const NSOpenGLPFAAccelerated: u32 = 73;
const NSOpenGLPFAColorSize: u32 = 8;
const NSOpenGLPFADepthSize: u32 = 12;
const NSOpenGLPFAOpenGLProfile: u32 = 99;
const NSOpenGLProfileVersion3_2Core: u32 = 0x3200;

// NSOpenGLContextParameter.
const NS_OPENGL_CP_SWAP_INTERVAL: isize = 222;

unsafe fn msg_set_values(obj: Id, vals: *const i32, param: isize) {
    type Fn = unsafe extern "C" fn(Id, Id, *const i32, isize);
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel("setValues:forParameter:"), vals, param)
}

unsafe fn enable_vsync(gl_ctx: Id) {
    let interval: i32 = 1;
    msg_set_values(gl_ctx, &interval as *const i32, NS_OPENGL_CP_SWAP_INTERVAL);
}

// Send wrappers

#[derive(Clone)]
struct SendId(Id);
unsafe impl Send for SendId {}
unsafe impl Sync for SendId {}

// ColorSync / CoreGraphics: ICC profile of the main display

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGMainDisplayID() -> u32;
    fn CGDisplayCopyColorSpace(display: u32) -> Id;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CGColorSpaceCopyICCData(space: Id) -> Id; // CFDataRef
    fn CFDataGetLength(data: Id) -> isize;
    fn CFDataGetBytePtr(data: Id) -> *const u8;
    fn CFRelease(obj: Id);
}

fn query_colorsync_icc_profile() -> Option<Vec<u8>> {
    unsafe {
        let space = CGDisplayCopyColorSpace(CGMainDisplayID());
        if space.is_null() {
            return None;
        }
        let data = CGColorSpaceCopyICCData(space);
        CFRelease(space);
        if data.is_null() {
            return None;
        }
        let len = CFDataGetLength(data);
        let ptr = CFDataGetBytePtr(data);
        let result = if len > 0 && !ptr.is_null() {
            Some(std::slice::from_raw_parts(ptr, len as usize).to_vec())
        } else {
            None
        };
        CFRelease(data);
        result
    }
}

// GCD dispatch helpers

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
    if lib.is_null() {
        return std::ptr::null_mut();
    }
    libc::dlsym(lib, b"_dispatch_main_q\0".as_ptr() as _)
}

unsafe fn gcd_async_f(
    queue: *mut c_void,
    ctx: *mut c_void,
    work: unsafe extern "C" fn(*mut c_void),
) {
    let lib = libdispatch();
    if lib.is_null() {
        return;
    }
    let f: unsafe extern "C" fn(*mut c_void, *mut c_void, unsafe extern "C" fn(*mut c_void)) =
        std::mem::transmute(libc::dlsym(lib, b"dispatch_async_f\0".as_ptr() as _));
    f(queue, ctx, work);
}

fn run_on_main(f: impl FnOnce() + Send + 'static) {
    unsafe extern "C" fn trampoline(ctx: *mut c_void) {
        let f = unsafe { Box::from_raw(ctx as *mut Box<dyn FnOnce() + Send>) };
        f();
    }
    let ctx = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce() + Send>)) as *mut c_void;
    unsafe {
        let queue = gcd_main_queue();
        if queue.is_null() || libc::pthread_main_np() == 1 {
            trampoline(ctx);
        } else {
            gcd_async_f(queue, ctx, trampoline);
        }
    }
}

// Surface commands

enum SurfaceCommand {
    Load {
        url: String,
        start_at: Option<u64>,
        total_duration: Option<u64>,
    },
    Hide,
    ShowLoading {
        title: String,
        episode_title: Option<String>,
    },
    SetTitle {
        title: String,
        episode_title: Option<String>,
    },
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
    pub fn load(
        &self,
        url: String,
        start_at: Option<u64>,
        total_duration: Option<u64>,
    ) -> Result<(), String> {
        self.sender
            .send(SurfaceCommand::Load {
                url,
                start_at,
                total_duration,
            })
            .map_err(|e| format!("surface unavailable: {e}"))
    }
    pub fn hide(&self) {
        let _ = self.sender.send(SurfaceCommand::Hide);
    }
    pub fn show_loading(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::ShowLoading {
            title,
            episode_title,
        });
    }
    pub fn set_title(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::SetTitle {
            title,
            episode_title,
        });
    }
    pub fn set_artwork(
        &self,
        title: String,
        episode_title: Option<String>,
        background: Option<(Vec<u8>, i32, i32)>,
        logo: Option<(Vec<u8>, i32, i32)>,
    ) {
        let _ = self.sender.send(SurfaceCommand::SetArtwork {
            title,
            episode_title,
            background,
            logo,
        });
    }
}

// install

pub fn install(app_handle: AppHandle) -> Result<NativePlayerSurface, String> {
    let (sender, receiver) = mpsc::channel::<SurfaceCommand>();

    // Get the Tauri window's NSView.
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let ns_view: Id = match window.window_handle().map_err(|e| e.to_string())?.as_ref() {
        RawWindowHandle::AppKit(h) => h.ns_view.as_ptr() as Id,
        _ => return Err("unexpected window handle type on macOS".to_string()),
    };

    // Get the initial window size via Tauri (avoids NSRect stret).
    let init_size = window
        .inner_size()
        .unwrap_or(tauri::PhysicalSize::new(1280, 720));
    let init_w = init_size.width.max(2) as i32;
    let init_h = init_size.height.max(2) as i32;
    let scale = window.scale_factor().unwrap_or(1.0).max(1.0);

    // Create the render subview on the main thread, collect result.
    // NSView frames are in points; inner_size() is physical pixels.
    let (view_tx, view_rx) = mpsc::channel::<Result<SendId, String>>();
    let parent_ptr = ns_view as usize;
    let frame_w = init_w as f64 / scale;
    let frame_h = init_h as f64 / scale;
    run_on_main(move || {
        let parent = SendId(parent_ptr as Id);
        let _ = view_tx.send(unsafe { create_render_subview(parent, frame_w, frame_h) });
    });

    let render_view = view_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "macOS render view creation timed out".to_string())
        .and_then(|r| r)?;

    let backend = read_render_backend(&app_handle);
    log::info!(
        "macos_player_surface: experimental render backend = {}",
        match backend {
            RenderBackend::OpenGl => "opengl",
            RenderBackend::Vulkan => "vulkan",
        }
    );

    enum MacRenderTarget {
        Gl {
            gl_ctx: usize,
        },
        Vulkan {
            ctx: VulkanContext,
            metal_layer: usize,
        },
    }

    let render_target = match backend {
        RenderBackend::OpenGl => {
            let gl_ctx = unsafe { create_gl_context()? };

            let (attach_tx, attach_rx) = mpsc::channel::<()>();
            let attach_ctx = gl_ctx.0 as usize;
            let attach_view = render_view.0 as usize;
            run_on_main(move || {
                unsafe { msg1_id(attach_ctx as Id, "setView:", attach_view as Id) };
                let _ = attach_tx.send(());
            });
            attach_rx
                .recv_timeout(Duration::from_secs(5))
                .map_err(|_| "attaching GL context to render view timed out".to_string())?;

            unsafe {
                msg0(gl_ctx.0, "makeCurrentContext");
                enable_vsync(gl_ctx.0);
            }

            {
                let state = app_handle.state::<DesktopState>();
                let mut render_guard = state.player_render_state.lock().unwrap();
                let mut client_guard = state.player_mpv_client.lock().unwrap();
                if client_guard.is_none() {
                    match crate::mpv_render::MpvClientHandle::new() {
                        Ok((client, render)) => {
                            *render_guard = Some(render);
                            *client_guard = Some(client);
                        }
                        Err(e) => {
                            return Err(format!("mpv init failed: {e}"));
                        }
                    }
                }
                if let Some(r) = render_guard.as_mut() {
                    r.prepare_opengl_context()
                        .map_err(|e| format!("mpv GL context failed: {e}"))?;
                    if let Some(icc) = query_colorsync_icc_profile() {
                        if let Err(e) = r.set_icc_profile(&icc) {
                            log::warn!("failed to set ICC profile: {e}");
                        }
                    }
                }
            }

            unsafe { msg0(cls("NSOpenGLContext"), "clearCurrentContext") };

            MacRenderTarget::Gl {
                gl_ctx: gl_ctx.0 as usize,
            }
        }
        RenderBackend::Vulkan => {
            let (layer_tx, layer_rx) = mpsc::channel::<Result<SendId, String>>();
            let layer_view = render_view.0 as usize;
            run_on_main(move || {
                let _ = layer_tx.send(unsafe {
                    create_metal_layer(layer_view as Id, scale, init_w, init_h).map(SendId)
                });
            });
            let metal_layer = layer_rx
                .recv_timeout(Duration::from_secs(5))
                .map_err(|_| "macOS Metal layer creation timed out".to_string())
                .and_then(|r| r)?;

            let vk_ctx = VulkanContext::new(metal_layer.0 as *const c_void, init_w, init_h)
                .map_err(|e| format!("Vulkan context creation failed: {e}"))?;

            let state = app_handle.state::<DesktopState>();
            let mut render_guard = state.player_render_state.lock().unwrap();
            let mut client_guard = state.player_mpv_client.lock().unwrap();
            if client_guard.is_none() {
                match crate::mpv_render::MpvClientHandle::new() {
                    Ok((client, render)) => {
                        *render_guard = Some(render);
                        *client_guard = Some(client);
                    }
                    Err(e) => {
                        return Err(format!("mpv init failed: {e}"));
                    }
                }
            }
            if let Some(r) = render_guard.as_mut() {
                let (instance, phys_device, device, queue_index, queue_count) =
                    vk_ctx.device_handles();
                r.create_vulkan_context(
                    instance,
                    phys_device,
                    device,
                    queue_index,
                    queue_count,
                    std::ptr::null_mut(),
                    &[],
                )
                .map_err(|e| format!("mpv Vulkan context failed: {e}"))?;
                if let Some(icc) = query_colorsync_icc_profile() {
                    if let Err(e) = r.set_icc_profile(&icc) {
                        log::warn!("failed to set ICC profile: {e}");
                    }
                }
            }
            if vk_ctx.is_hdr() {
                if let Some(c) = client_guard.as_ref() {
                    let _ = c.set_option("target-trc", "linear");
                    let _ = c.set_option("target-prim", "bt.709");
                }
            }

            MacRenderTarget::Vulkan {
                ctx: vk_ctx,
                metal_layer: metal_layer.0 as usize,
            }
        }
    };

    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
    let app = app_handle.clone();

    // usize is Send; this is the standard pattern for passing ObjC raw pointers
    // across a thread boundary when the caller guarantees exclusive access.
    let render_view_usize: usize = render_view.0 as usize;

    std::thread::spawn(move || {
        let rv: *mut c_void = render_view_usize as _;
        let mut render_target = render_target;

        if let MacRenderTarget::Gl { gl_ctx } = &render_target {
            unsafe { msg0(*gl_ctx as Id, "makeCurrentContext") };
        }
        let _ = ready_tx.send(Ok(()));

        let mut visible = false;
        let mut last_size = (init_w, init_h);

        loop {
            while let Ok(cmd) = receiver.try_recv() {
                match cmd {
                    SurfaceCommand::Load { url, start_at, .. } => {
                        let view = rv as usize;
                        run_on_main(move || unsafe {
                            msg1_bool(view as Id, "setHidden:", 0);
                        });
                        visible = true;
                        app.state::<DesktopState>()
                            .pending_hide
                            .store(false, std::sync::atomic::Ordering::Release);
                        let _ = app.emit("native-player-show", ());
                        let state = app.state::<DesktopState>();

                        state.player_overlay.lock().unwrap().eof_next_fired = false;
                        if *state.active_player_engine.lock().unwrap() == PlayerEngine::Vlc {
                            let result = (|| {
                                let mut players = state.player_renderer_vlc.lock().unwrap();
                                if players.is_none() {
                                    *players = Some(crate::libvlc_render::LibvlcPlayer::new()?);
                                }
                                let player = players
                                    .as_mut()
                                    .ok_or_else(|| "libVLC player is unavailable".to_string())?;
                                player.attach_nsobject(rv)?;
                                player.load(&url, start_at)
                            })();
                            if let Err(error) = result {
                                let _ = app.emit("native-player-error", error);
                                visible = false;
                                let view = rv as usize;
                                run_on_main(move || unsafe {
                                    msg1_bool(view as Id, "setHidden:", 1);
                                });
                            }
                            continue;
                        }
                        let mut r = state.player_mpv_client.lock().unwrap();
                        if let Some(renderer) = r.as_mut() {
                            if let Err(e) = renderer.load(&url, start_at) {
                                drop(r);
                                let _ = app.emit("native-player-error", e);
                                visible = false;
                                let view = rv as usize;
                                run_on_main(move || unsafe {
                                    msg1_bool(view as Id, "setHidden:", 1);
                                });
                            }
                        }
                    }
                    SurfaceCommand::Hide => {
                        visible = false;
                        let view = rv as usize;
                        run_on_main(move || unsafe {
                            msg1_bool(view as Id, "setHidden:", 1);
                        });
                        let _ = app.emit("native-player-hide", ());
                        let state = app.state::<DesktopState>();

                        let guard = state.player_mpv_client.lock().unwrap();
                        if let Some(r) = guard.as_ref() {
                            let _ = r.command_args(&["stop"]);
                        }
                    }
                    SurfaceCommand::ShowLoading {
                        title,
                        episode_title,
                    } => {
                        let _ = app.emit(
                            "native-player-title",
                            serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                        );
                    }
                    SurfaceCommand::SetTitle {
                        title,
                        episode_title,
                    } => {
                        let _ = app.emit(
                            "native-player-title",
                            serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                        );
                    }
                    SurfaceCommand::SetArtwork { title, .. } => {
                        let _ =
                            app.emit("native-player-title", serde_json::json!({ "title": title }));
                    }
                }
            }

            if visible {
                if app
                    .state::<DesktopState>()
                    .pending_hide
                    .load(std::sync::atomic::Ordering::Acquire)
                {
                    std::thread::sleep(Duration::from_millis(16));
                    continue;
                }

                let mut resized = false;
                if let Some(win) = app.get_webview_window("main") {
                    if let Ok(sz) = win.inner_size() {
                        let scale = win.scale_factor().unwrap_or(1.0).max(1.0);
                        let nw = (sz.width as i32).max(2);
                        let nh = (sz.height as i32).max(2);
                        if (nw, nh) != last_size {
                            last_size = (nw, nh);
                            resized = true;
                            let view = rv as usize;
                            let frame_w = nw as f64 / scale;
                            let frame_h = nh as f64 / scale;
                            match &render_target {
                                MacRenderTarget::Gl { gl_ctx } => {
                                    let ctx = *gl_ctx;
                                    run_on_main(move || unsafe {
                                        msg_set_frame(
                                            view as Id,
                                            NSRect {
                                                origin: NSPoint { x: 0.0, y: 0.0 },
                                                size: NSSize {
                                                    width: frame_w,
                                                    height: frame_h,
                                                },
                                            },
                                        );
                                        msg0(ctx as Id, "update");
                                    });
                                }
                                MacRenderTarget::Vulkan { metal_layer, .. } => {
                                    let layer = *metal_layer;
                                    run_on_main(move || unsafe {
                                        msg_set_frame(
                                            view as Id,
                                            NSRect {
                                                origin: NSPoint { x: 0.0, y: 0.0 },
                                                size: NSSize {
                                                    width: frame_w,
                                                    height: frame_h,
                                                },
                                            },
                                        );
                                        msg_set_drawable_size(
                                            layer as Id,
                                            NSSize {
                                                width: nw as f64,
                                                height: nh as f64,
                                            },
                                        );
                                    });
                                }
                            }
                        }
                    }
                }

                if *app
                    .state::<DesktopState>()
                    .active_player_engine
                    .lock()
                    .unwrap()
                    == PlayerEngine::Vlc
                {
                    std::thread::sleep(Duration::from_millis(16));
                    continue;
                }

                match &mut render_target {
                    MacRenderTarget::Gl { gl_ctx } => {
                        {
                            let state = app.state::<DesktopState>();

                            let mut renderer = state.player_render_state.lock().unwrap();
                            if let Some(r) = renderer.as_mut() {
                                let _ = r.render_opengl_frame(last_size.0, last_size.1);
                            }
                        }
                        let flush_start = std::time::Instant::now();
                        unsafe { msg0(*gl_ctx as Id, "flushBuffer") };
                        if flush_start.elapsed() < Duration::from_millis(4) {
                            std::thread::sleep(Duration::from_millis(16));
                        }
                        {
                            let state = app.state::<DesktopState>();

                            let mut renderer = state.player_render_state.lock().unwrap();
                            if let Some(r) = renderer.as_mut() {
                                r.report_swap();
                            };
                        }
                    }
                    MacRenderTarget::Vulkan { ctx, .. } => {
                        if resized {
                            if let Err(e) = ctx.resize(last_size.0, last_size.1) {
                                log::warn!("macos_player_surface: Vulkan resize failed: {e}");
                            }
                        }
                        let state = app.state::<DesktopState>();

                        let mut renderer = state.player_render_state.lock().unwrap();
                        if let Some(r) = renderer.as_mut() {
                            let image_usage = ctx.image_usage();
                            let result = ctx.render_and_present(
                                |image, format, iw, ih, wait_semaphore, signal_semaphore| {
                                    let mut target = VulkanTargetImage {
                                        image,
                                        format,
                                        w: iw as i32,
                                        h: ih as i32,
                                        usage: image_usage,
                                        layout: 0,
                                        wait_semaphore,
                                        signal_semaphore,
                                    };
                                    r.render_vulkan_frame(&mut target).map(|_| target.layout)
                                },
                            );
                            match result {
                                Ok(()) => r.report_swap(),
                                Err(e) => {
                                    log::warn!("macos_player_surface: Vulkan render failed: {e}")
                                }
                            }
                        }
                        std::thread::sleep(Duration::from_millis(16));
                    }
                }

                check_player_events(&app);
            } else {
                std::thread::sleep(Duration::from_millis(16));
            }
        }
    });

    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "macOS render thread setup timed out".to_string())
        .and_then(|r| r)
        .map(|()| NativePlayerSurface { sender })
}

// ObjC helpers

unsafe fn create_render_subview(parent: SendId, w: f64, h: f64) -> Result<SendId, String> {
    let ns_view_cls = cls("NSView");
    if ns_view_cls.is_null() {
        return Err("NSView class not found".to_string());
    }

    let frame = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: NSSize {
            width: w,
            height: h,
        },
    };
    let alloc: Id = msg0(ns_view_cls, "alloc");
    let view: Id = msg_init_with_frame(alloc, frame);
    if view.is_null() {
        return Err("NSView initWithFrame: failed".to_string());
    }

    // wantsLayer = YES allows us to control z-ordering.
    msg1_bool(view, "setWantsLayer:", 1);
    msg1_bool(view, "setWantsBestResolutionOpenGLSurface:", 1);

    // Insert at back (NSWindowBelow = -1), relative to nil = behind everything.
    msg3_positioned(parent.0, view, -1, std::ptr::null_mut());

    // Hidden until playback starts.
    msg1_bool(view, "setHidden:", 1);

    Ok(SendId(view))
}

unsafe fn create_metal_layer(view: Id, contents_scale: f64, w: i32, h: i32) -> Result<Id, String> {
    let layer_cls = cls("CAMetalLayer");
    if layer_cls.is_null() {
        return Err("CAMetalLayer class not found".to_string());
    }
    let alloc: Id = msg0(layer_cls, "alloc");
    let layer: Id = msg0(alloc, "init");
    if layer.is_null() {
        return Err("CAMetalLayer init failed".to_string());
    }
    msg1_f64(layer, "setContentsScale:", contents_scale);
    msg1_bool(layer, "setWantsExtendedDynamicRangeContent:", 1);
    msg_set_drawable_size(
        layer,
        NSSize {
            width: w as f64,
            height: h as f64,
        },
    );
    msg1_id(view, "setLayer:", layer);
    Ok(layer)
}

unsafe fn create_gl_context() -> Result<SendId, String> {
    let attribs: [u32; 9] = [
        NSOpenGLPFADoubleBuffer,
        NSOpenGLPFAAccelerated,
        NSOpenGLPFAColorSize,
        32,
        NSOpenGLPFADepthSize,
        24,
        NSOpenGLPFAOpenGLProfile,
        NSOpenGLProfileVersion3_2Core,
        0,
    ];

    let pf_cls = cls("NSOpenGLPixelFormat");
    let pf_alloc: Id = msg0(pf_cls, "alloc");
    // initWithAttributes: takes a pointer arg, not a struct.
    type InitAttrFn = unsafe extern "C" fn(Id, Id, *const u32) -> Id;
    let init_attr: InitAttrFn =
        std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    let pf: Id = init_attr(pf_alloc, sel("initWithAttributes:"), attribs.as_ptr());
    if pf.is_null() {
        return Err("NSOpenGLPixelFormat init failed (3.2 core profile)".to_string());
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

    Ok(SendId(ctx))
}

// Player event polling

fn check_player_events(app: &AppHandle) {
    let state = app.state::<DesktopState>();
    let (events, status) = {
        let mut renderer = state.player_mpv_client.lock().unwrap();
        match renderer.as_mut() {
            Some(r) => (r.poll_events(), r.status()),
            None => return,
        }
    };
    for event in events {
        let crate::mpv_render::PlayerEvent::EndFile { eof: _, error } = event;
        if let Some(message) = error {
            log::error!("macos player surface: stream failed to play: {message}");
            let _ = app.emit("native-player-error", message);
        }
    }
    if !status.eof_reached() {
        return;
    }
    let mut overlay = state.player_overlay.lock().unwrap();
    if !overlay.take_eof_next() {
        return;
    }

    let next_sub = overlay.next_ep_subtitle.clone();
    let auto_play = overlay.auto_play_next_episode;
    drop(overlay);
    if FluxaCore::should_play_next_episode(!next_sub.is_empty(), auto_play) {
        let _ = app.emit("native-player-next-episode", ());
    } else {
        let _ = app.emit("native-player-close-requested", ());
    }
}

// Artwork helpers

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

pub fn scale_artwork_fit(bytes: Vec<u8>, max_w: u32, max_h: u32) -> Option<(Vec<u8>, i32, i32)> {
    let img = image::load_from_memory(&bytes).ok()?;
    let resized = img.resize(max_w, max_h, image::imageops::FilterType::Triangle);
    let (rw, rh) = (resized.width(), resized.height());
    let rgba = resized.to_rgba8();
    Some((rgba.into_raw(), rw as i32, rh as i32))
}
