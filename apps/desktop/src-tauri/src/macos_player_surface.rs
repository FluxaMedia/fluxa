// macOS native player surface — libmpv Vulkan render API via CAMetalLayer.
//
// A child NSView is inserted behind the Tauri WKWebView using ObjC messaging.
// Vulkan renders mpv frames into a CAMetalLayer behind the WebView.
// Window size is queried via Tauri (no NSRect stret needed).

use crate::DesktopState;
use crate::macos_vulkan::VulkanContext;
use crate::mpv_render::VulkanTargetImage;
use crate::playback_engine::{PlaybackEngine, PlayerEngine};
use crate::render_backend::RenderBackend;
use std::ffi::{CString, c_void};
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

#[link(name = "QuartzCore", kind = "framework")]
unsafe extern "C" {}

// ObjC runtime types

type Id = *mut c_void;

// All ObjC calls go through these two entry points.
unsafe extern "C" {
    fn objc_getClass(name: *const i8) -> Id;
    fn objc_msgSend(receiver: Id, sel: Id, ...) -> Id;
    fn sel_registerName(name: *const i8) -> Id;
    fn object_getClassName(obj: Id) -> *const i8;
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
unsafe fn msg1_usize(obj: Id, sel_name: &str, v: usize) {
    type Fn = unsafe extern "C" fn(Id, Id, usize);
    let f: Fn = unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _) };
    unsafe { f(obj, sel(sel_name), v) }
}

unsafe fn msg1_usize_ret(obj: Id, sel_name: &str, v: usize) -> Id {
    type Fn = unsafe extern "C" fn(Id, Id, usize) -> Id;
    let f: Fn = unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _) };
    unsafe { f(obj, sel(sel_name), v) }
}

unsafe fn msg0_usize(obj: Id, sel_name: &str) -> usize {
    type Fn = unsafe extern "C" fn(Id, Id) -> usize;
    let f: Fn = unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _) };
    unsafe { f(obj, sel(sel_name)) }
}

unsafe fn msg1_f64(obj: Id, sel_name: &str, v: f64) {
    type Fn = unsafe extern "C" fn(Id, Id, f64);
    let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
    f(obj, sel(sel_name), v)
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
    Shutdown {
        ack: mpsc::Sender<()>,
    },
}

#[derive(Clone)]
pub struct NativePlayerSurface {
    sender: mpsc::Sender<SurfaceCommand>,
    backend: RenderBackend,
    thread: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
    app: AppHandle,
}

impl NativePlayerSurface {
    pub fn backend(&self) -> RenderBackend {
        self.backend
    }
}

impl crate::player_surface::PlayerSurface for NativePlayerSurface {
    fn backend_name(&self) -> &'static str {
        self.backend.name()
    }

    fn load(
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

    fn hide(&self) {
        let _ = self.sender.send(SurfaceCommand::Hide);
    }

    fn shutdown(&self) -> Result<(), String> {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.sender
            .send(SurfaceCommand::Shutdown { ack: ack_tx })
            .map_err(|e| format!("surface unavailable: {e}"))?;
        ack_rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| format!("surface shutdown timed out: {e}"))?;
        let thread = self
            .thread
            .lock()
            .map_err(|_| "surface thread lock poisoned".to_string())?
            .take();
        if let Some(thread) = thread {
            thread
                .join()
                .map_err(|_| "surface render thread panicked".to_string())?;
        }
        Ok(())
    }

    fn show_loading(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::ShowLoading {
            title,
            episode_title,
        });
    }

    fn set_title(&self, title: String, episode_title: Option<String>) {
        let _ = self.sender.send(SurfaceCommand::SetTitle {
            title,
            episode_title,
        });
    }

    fn set_artwork(
        &self,
        title: String,
        episode_title: Option<String>,
        background: crate::player_surface::Artwork,
        logo: crate::player_surface::Artwork,
    ) {
        let _ = self.sender.send(SurfaceCommand::SetArtwork {
            title,
            episode_title,
            background,
            logo,
        });
    }

    fn set_cursor_visible(&self, _visible: bool) {}

    fn command(&self, command: String) -> Result<(), String> {
        crate::player_surface_events::engine_command(&self.app, command)
    }

    fn command_args(&self, commands: Vec<Vec<String>>) -> Result<(), String> {
        crate::player_surface_events::engine_command_args(&self.app, commands)
    }

    fn status(&self) -> Result<crate::mpv_render::PlayerStatus, String> {
        crate::player_surface_events::engine_status(&self.app)
    }

    fn track_options(
        &self,
        track_type: String,
    ) -> Result<Vec<crate::mpv_render::PlayerTrackOption>, String> {
        crate::player_surface_events::engine_track_options(&self.app, track_type)
    }

    // sub-add loads the file synchronously, so this must never run on the render thread.
    fn add_subtitle(
        &self,
        url: String,
        title: Option<String>,
        language: Option<String>,
    ) -> Result<(), String> {
        crate::player_surface_events::engine_add_subtitle(&self.app, url, title, language)
    }
}

// install

pub fn install(app_handle: AppHandle) -> Result<NativePlayerSurface, String> {
    install_with_backend(app_handle)
}

fn install_with_backend(app_handle: AppHandle) -> Result<NativePlayerSurface, String> {
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
    let (view_tx, view_rx) = mpsc::channel::<Result<(SendId, Option<SendId>), String>>();
    let parent_ptr = ns_view as usize;
    let frame_w = init_w as f64 / scale;
    let frame_h = init_h as f64 / scale;
    let layer_request = Some((scale, init_w, init_h));
    run_on_main(move || {
        let parent = SendId(parent_ptr as Id);
        let _ =
            view_tx.send(unsafe { create_render_subview(parent, frame_w, frame_h, layer_request) });
    });

    let (render_view, prepared_layer) = view_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "macOS render view creation timed out".to_string())
        .and_then(|r| r)?;

    log::info!(
        "macos_player_surface: requested render backend={} arch={} scale={} drawable={}x{}",
        "vulkan",
        std::env::consts::ARCH,
        scale,
        init_w,
        init_h
    );

    enum MacRenderTarget {
        Vulkan {
            ctx: VulkanContext,
            metal_layer: usize,
        },
    }

    let render_target = {
        let metal_layer =
            prepared_layer.ok_or_else(|| "CAMetalLayer was not created".to_string())?;

        let vk_ctx =
            crate::macos_vulkan::create_context(metal_layer.0 as *const c_void, init_w, init_h)
                .map_err(|e| format!("Vulkan context creation failed: {e}"))?;

        let state = app_handle.state::<DesktopState>();
        let mut render_guard = state.player_render_state.lock().unwrap();
        let mut client_guard = state.player_mpv_client.lock().unwrap();
        if client_guard.is_none() {
            match crate::mpv_render::MpvClientHandle::new_with_scripts(
                crate::player::mpv_script_paths(&app_handle),
            ) {
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
            if let Some(client) = client_guard.as_ref()
                && let Err(e) = client.set_log_level("v")
            {
                log::warn!("mpv: could not raise log level for Vulkan init: {e}");
            }
            let (instance, phys_device, device, queue_index, queue_count, get_proc_addr) =
                vk_ctx.device_handles();
            let enabled_extensions = vk_ctx.enabled_device_extension_ptrs();
            r.create_vulkan_context(
                instance,
                phys_device,
                device,
                queue_index,
                queue_count,
                get_proc_addr,
                &enabled_extensions,
            )
            .map_err(|e| {
                if let Some(client) = client_guard.as_mut() {
                    let _ = client.poll_events();
                }
                format!("mpv Vulkan context failed: {e}")
            })?;
            if let Some(client) = client_guard.as_ref() {
                let _ = client.set_log_level("info");
            }
            if let Some(icc) = query_colorsync_icc_profile() {
                if let Err(e) = r.set_icc_profile(&icc) {
                    log::warn!("failed to set ICC profile: {e}");
                }
            }
        }
        MacRenderTarget::Vulkan {
            ctx: vk_ctx,
            metal_layer: metal_layer.0 as usize,
        }
    };
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
    let app = app_handle.clone();

    // usize is Send; this is the standard pattern for passing ObjC raw pointers
    // across a thread boundary when the caller guarantees exclusive access.
    let render_view_usize: usize = render_view.0 as usize;

    let thread = std::thread::spawn(move || {
        let rv: *mut c_void = render_view_usize as _;
        let mut render_target = render_target;

        let _ = ready_tx.send(Ok(()));

        let mut visible = false;
        let mut last_size = (init_w, init_h);
        let mut last_vk_render_error: Option<String> = None;

        'render: loop {
            while let Ok(cmd) = receiver.try_recv() {
                match cmd {
                    SurfaceCommand::Load { url, start_at, .. } => {
                        let view = rv as usize;
                        run_on_main(move || unsafe {
                            msg1_bool(view as Id, "setHidden:", 0);
                        });
                        visible = true;
                        let state = app.state::<DesktopState>();
                        crate::player::reset_playback_state(&state);
                        let _ = app.emit("native-player-show", ());
                        if *state.active_player_engine.lock().unwrap() == PlayerEngine::Vlc {
                            let result = crate::player::load_libvlc_for_surface(
                                &state,
                                &url,
                                start_at,
                                |player| player.attach_nsobject(rv),
                            );
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
                        let hdr = match &render_target {
                            MacRenderTarget::Vulkan { ctx, metal_layer } => {
                                unsafe { log_surface_geometry("load", rv, *metal_layer as Id) };
                                ctx.is_hdr()
                            }
                        };
                        let mut r = state.player_mpv_client.lock().unwrap();
                        if let Some(renderer) = r.as_mut() {
                            if let Err(e) =
                                crate::player::load_mpv_engine(renderer, &url, start_at, hdr)
                            {
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
                    SurfaceCommand::Shutdown { ack } => {
                        visible = false;
                        let state = app.state::<DesktopState>();
                        if let Some(renderer) = state.player_mpv_client.lock().unwrap().as_ref() {
                            let _ = renderer.command_args(&["stop"]);
                        }
                        state
                            .player_render_state
                            .lock()
                            .unwrap()
                            .as_mut()
                            .map(|renderer| renderer.reset_render_context());
                        let view = rv as usize;
                        let (cleanup_tx, cleanup_rx) = mpsc::channel();
                        run_on_main(move || unsafe {
                            msg1_bool(view as Id, "setHidden:", 1);
                            msg1_id(view as Id, "setLayer:", std::ptr::null_mut());
                            msg0(view as Id, "removeFromSuperview");
                            let _ = cleanup_tx.send(());
                        });
                        let _ = cleanup_rx.recv_timeout(Duration::from_secs(5));
                        drop(render_target);
                        let _ = ack.send(());
                        break 'render;
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
                    MacRenderTarget::Vulkan { ctx, .. } => {
                        if resized {
                            if let Err(e) = ctx.resize(last_size.0, last_size.1) {
                                log::warn!("macos_player_surface: Vulkan resize failed: {e}");
                            }
                        }
                        let state = app.state::<DesktopState>();

                        let mut renderer = state.player_render_state.lock().unwrap();
                        let mut rebuild_failed = false;
                        if let Some(r) = renderer.as_mut()
                            && r.needs_vulkan_context()
                        {
                            let (
                                instance,
                                phys_device,
                                device,
                                queue_index,
                                queue_count,
                                get_proc_addr,
                            ) = ctx.device_handles();
                            let enabled_extensions = ctx.enabled_device_extension_ptrs();
                            match r.create_vulkan_context(
                                instance,
                                phys_device,
                                device,
                                queue_index,
                                queue_count,
                                get_proc_addr,
                                &enabled_extensions,
                            ) {
                                Ok(()) => {
                                    log::info!(
                                        "macos_player_surface: Vulkan render context rebuilt"
                                    );
                                    last_vk_render_error = None;
                                }
                                Err(e) => {
                                    if last_vk_render_error.as_deref() != Some(e.as_str()) {
                                        log::error!(
                                            "macos_player_surface: Vulkan context rebuild failed: {e}"
                                        );
                                        last_vk_render_error = Some(e);
                                    }
                                    rebuild_failed = true;
                                }
                            }
                        }
                        if rebuild_failed {
                            drop(renderer);
                            std::thread::sleep(Duration::from_millis(16));
                            continue;
                        }
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
                                Ok(()) => {
                                    r.report_swap();
                                    last_vk_render_error = None;
                                }
                                Err(e) => {
                                    log::warn!("macos_player_surface: Vulkan render failed: {e}");
                                    if last_vk_render_error.as_deref() != Some(e.as_str()) {
                                        crate::diagnostics::report(
                                            &app,
                                            format!(
                                                "macos_player_surface: Vulkan render failed: {e}"
                                            ),
                                            sentry::Level::Error,
                                        );
                                        last_vk_render_error = Some(e.clone());
                                    }
                                }
                            }
                        }
                        std::thread::sleep(Duration::from_millis(16));
                    }
                }

                crate::player_surface_events::check_player_events(&app);
            } else {
                std::thread::sleep(Duration::from_millis(16));
            }
        }
    });

    let ready = ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "macOS render thread setup timed out".to_string())
        .and_then(|r| r);
    if let Err(error) = ready {
        let (ack_tx, ack_rx) = mpsc::channel();
        let _ = sender.send(SurfaceCommand::Shutdown { ack: ack_tx });
        let _ = ack_rx.recv_timeout(Duration::from_secs(5));
        let _ = thread.join();
        return Err(error);
    }
    Ok(NativePlayerSurface {
        sender,
        backend: RenderBackend::Vulkan,
        thread: Arc::new(Mutex::new(Some(thread))),
        app: app_handle,
    })
}

// ObjC helpers

unsafe fn create_render_subview(
    parent: SendId,
    w: f64,
    h: f64,
    metal_layer: Option<(f64, i32, i32)>,
) -> Result<(SendId, Option<SendId>), String> {
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

    let layer = match metal_layer {
        Some((contents_scale, pixel_w, pixel_h)) => {
            let layer = create_metal_layer(contents_scale, pixel_w, pixel_h)?;
            msg1_id(view, "setLayer:", layer);
            Some(SendId(layer))
        }
        None => None,
    };
    msg1_bool(view, "setWantsLayer:", 1);

    let mode = surface_mode();
    log::info!("macos_player_surface: placement mode={mode}");
    if mode == "above" || mode == "solid" {
        let host = host_view_for(parent.0);
        let container = match host {
            HostView::Sibling { content_view, .. } => content_view,
            HostView::Container(c) => c,
        };
        msg3_positioned(container, view, 1, std::ptr::null_mut());
        msg1_usize(view, "setAutoresizingMask:", 2 | 16);
        if mode == "solid" {
            paint_layer(view, 0.0, 1.0, 0.0);
        }
        msg1_bool(view, "setHidden:", 1);
        return Ok((SendId(view), layer));
    }

    let host = host_view_for(parent.0);
    match host {
        HostView::Sibling { content_view, below } => {
            msg3_positioned(content_view, view, -1, below);
        }
        HostView::Container(container) => {
            msg3_positioned(container, view, -1, std::ptr::null_mut());
        }
    }
    msg1_usize(view, "setAutoresizingMask:", 2 | 16);
    if mode != "opaque" {
        make_webviews_transparent(parent.0);
    }

    // Hidden until playback starts.
    msg1_bool(view, "setHidden:", 1);

    Ok((SendId(view), layer))
}

unsafe extern "C" {
    fn CGColorCreateGenericRGB(red: f64, green: f64, blue: f64, alpha: f64) -> *mut c_void;
}

unsafe fn msg0_rect(obj: Id, sel_name: &str) -> NSRect {
    type Fn = unsafe extern "C" fn(Id, Id) -> NSRect;
    let f: Fn = unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _) };
    unsafe { f(obj, sel(sel_name)) }
}

unsafe fn msg0_bool(obj: Id, sel_name: &str) -> bool {
    type Fn = unsafe extern "C" fn(Id, Id) -> i8;
    let f: Fn = unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _) };
    unsafe { f(obj, sel(sel_name)) != 0 }
}

pub(crate) unsafe fn log_surface_geometry(tag: &str, view: Id, layer: Id) {
    unsafe {
        let frame = msg0_rect(view, "frame");
        let hidden = msg0_bool(view, "isHidden");
        let superview = msg0(view, "superview");
        let window = msg0(view, "window");
        let drawable = msg0_size(layer, "drawableSize");
        let opaque = msg0_bool(layer, "isOpaque");
        log::info!(
            "macos_player_surface[{tag}]: view frame={}x{}@{},{} hidden={} superview={} window={} layer_drawable={}x{} layer_opaque={}",
            frame.size.width,
            frame.size.height,
            frame.origin.x,
            frame.origin.y,
            hidden,
            !superview.is_null(),
            !window.is_null(),
            drawable.width,
            drawable.height,
            opaque
        );
    }
}

unsafe fn msg0_size(obj: Id, sel_name: &str) -> NSSize {
    type Fn = unsafe extern "C" fn(Id, Id) -> NSSize;
    let f: Fn = unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _) };
    unsafe { f(obj, sel(sel_name)) }
}

enum HostView {
    Sibling { content_view: Id, below: Id },
    Container(Id),
}

unsafe fn class_name(obj: Id) -> String {
    if obj.is_null() {
        return "nil".to_string();
    }
    unsafe {
        let name = object_getClassName(obj);
        if name.is_null() {
            "?".to_string()
        } else {
            std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned()
        }
    }
}

// Tauri hands back the WKWebView, not the window's content view. A subview of
// the web view sits under its layer tree and its opaque backdrop, so the video
// has to become a sibling underneath it instead.
unsafe fn host_view_for(parent: Id) -> HostView {
    unsafe {
        let window = msg0(parent, "window");
        let content_view = if window.is_null() {
            std::ptr::null_mut()
        } else {
            msg0(window, "contentView")
        };
        log::info!(
            "macos_player_surface: parent={} content_view={}",
            class_name(parent),
            class_name(content_view)
        );
        if !content_view.is_null() && content_view != parent {
            let mut below = parent;
            loop {
                let superview = msg0(below, "superview");
                if superview.is_null() || superview == content_view {
                    break;
                }
                below = superview;
            }
            return HostView::Sibling {
                content_view,
                below,
            };
        }
        HostView::Container(parent)
    }
}

// wry subclasses WKWebView as WryWebView, so match on the class hierarchy
// rather than the name.
unsafe fn is_kind_of(obj: Id, class: &str) -> bool {
    unsafe {
        let target = cls(class);
        if obj.is_null() || target.is_null() {
            return false;
        }
        type Fn = unsafe extern "C" fn(Id, Id, Id) -> i8;
        let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
        f(obj, sel("isKindOfClass:"), target) != 0
    }
}

fn surface_mode() -> String {
    std::env::var("FLUXA_MAC_SURFACE").unwrap_or_else(|_| "below".to_string())
}

unsafe fn paint_layer(view: Id, r: f64, g: f64, b: f64) {
    unsafe {
        let layer = msg0(view, "layer");
        if layer.is_null() {
            return;
        }
        let color = CGColorCreateGenericRGB(r, g, b, 1.0);
        if !color.is_null() {
            msg1_id(layer, "setBackgroundColor:", color);
        }
    }
}

unsafe fn responds_to(obj: Id, sel_name: &str) -> bool {
    unsafe {
        if obj.is_null() {
            return false;
        }
        type Fn = unsafe extern "C" fn(Id, Id, Id) -> i8;
        let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
        f(obj, sel("respondsToSelector:"), sel(sel_name)) != 0
    }
}

unsafe fn nsstring(value: &str) -> Id {
    unsafe {
        let cls_str = cls("NSString");
        let c = CString::new(value).unwrap();
        type Fn = unsafe extern "C" fn(Id, Id, *const i8) -> Id;
        let f: Fn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
        f(cls_str, sel("stringWithUTF8String:"), c.as_ptr())
    }
}

// WKWebView on macOS keeps painting its own backdrop until this private key is
// cleared; setOpaque: alone is not enough.
unsafe fn set_draws_background_off(view: Id) {
    unsafe {
        if !responds_to(view, "setValue:forKey:") {
            return;
        }
        let number_cls = cls("NSNumber");
        type NumFn = unsafe extern "C" fn(Id, Id, i8) -> Id;
        let num: NumFn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
        let no = num(number_cls, sel("numberWithBool:"), 0);
        type SetFn = unsafe extern "C" fn(Id, Id, Id, Id) -> Id;
        let set: SetFn = std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _);
        set(view, sel("setValue:forKey:"), no, nsstring("drawsBackground"));
    }
}

unsafe fn make_webviews_transparent(root: Id) {
    unsafe {
        let window = msg0(root, "window");
        let start = if window.is_null() {
            root
        } else {
            let content = msg0(window, "contentView");
            if content.is_null() { root } else { content }
        };
        if !window.is_null() && responds_to(window, "setOpaque:") {
            msg1_bool(window, "setOpaque:", 0);
        }
        walk_and_clear(start, 0);
    }
}

unsafe fn walk_and_clear(view: Id, depth: usize) {
    unsafe {
        if view.is_null() || depth > 6 {
            return;
        }
        let name = class_name(view);
        let is_web = is_kind_of(view, "WKWebView");
        log::info!(
            "macos_player_surface: view tree depth={depth} class={name} is_webview={is_web}"
        );
        if is_web {
            if responds_to(view, "setOpaque:") {
                msg1_bool(view, "setOpaque:", 0);
            }
            if responds_to(view, "setUnderPageBackgroundColor:") {
                let clear = msg0(cls("NSColor"), "clearColor");
                if !clear.is_null() {
                    msg1_id(view, "setUnderPageBackgroundColor:", clear);
                }
            }
            set_draws_background_off(view);
            log::info!("macos_player_surface: made {name} non-opaque");
        }
        let subviews = msg0(view, "subviews");
        if subviews.is_null() {
            return;
        }
        let count = msg0_usize(subviews, "count");
        for i in 0..count {
            walk_and_clear(msg1_usize_ret(subviews, "objectAtIndex:", i), depth + 1);
        }
    }
}

unsafe fn create_metal_layer(contents_scale: f64, w: i32, h: i32) -> Result<Id, String> {
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
    msg_set_drawable_size(
        layer,
        NSSize {
            width: w as f64,
            height: h as f64,
        },
    );
    let magenta = unsafe { CGColorCreateGenericRGB(1.0, 0.0, 1.0, 1.0) };
    if !magenta.is_null() {
        msg1_id(layer, "setBackgroundColor:", magenta);
    }
    Ok(layer)
}

// Player event polling

#[link(name = "WebKit", kind = "framework")]
unsafe extern "C" {}

unsafe fn msg0_bool_pub(obj: Id, sel_name: &str) -> bool {
    unsafe { msg0_bool(obj, sel_name) }
}

unsafe fn msg_init_window(obj: Id, rect: NSRect, style: usize, backing: usize, defer: i8) -> Id {
    type Fn = unsafe extern "C" fn(Id, Id, NSRect, usize, usize, i8) -> Id;
    let f: Fn = unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _) };
    unsafe {
        f(
            obj,
            sel("initWithContentRect:styleMask:backing:defer:"),
            rect,
            style,
            backing,
            defer,
        )
    }
}

unsafe fn msg_init_webview(obj: Id, rect: NSRect, config: Id) -> Id {
    type Fn = unsafe extern "C" fn(Id, Id, NSRect, Id) -> Id;
    let f: Fn = unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn(_, _, ...) -> _) };
    unsafe { f(obj, sel("initWithFrame:configuration:"), rect, config) }
}

pub fn smoke_check_placement() -> Result<String, String> {
    unsafe {
        let app = msg0(cls("NSApplication"), "sharedApplication");
        if app.is_null() {
            return Err("NSApplication sharedApplication returned nil".into());
        }
        msg1_usize(app, "setActivationPolicy:", 1);

        let rect = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: 800.0,
                height: 600.0,
            },
        };
        let window = msg_init_window(msg0(cls("NSWindow"), "alloc"), rect, 15, 2, 0);
        if window.is_null() {
            return Err("NSWindow creation failed".into());
        }
        let content_view = msg0(window, "contentView");
        if content_view.is_null() {
            return Err("window has no contentView".into());
        }

        let config = msg0(msg0(cls("WKWebViewConfiguration"), "alloc"), "init");
        let webview = msg_init_webview(msg0(cls("WKWebView"), "alloc"), rect, config);
        if webview.is_null() {
            return Err("WKWebView creation failed".into());
        }
        msg1_id(content_view, "addSubview:", webview);

        let (view, layer) = create_render_subview(SendId(content_view), 800.0, 600.0, Some((2.0, 1600, 1200)))?;
        msg1_bool(view.0, "setHidden:", 0);

        let superview = msg0(view.0, "superview");
        if superview != content_view {
            return Err(format!(
                "player view landed under {} instead of the window content view",
                class_name(superview)
            ));
        }
        if msg0_bool_pub(view.0, "isHiddenOrHasHiddenAncestor") {
            return Err("player view is hidden after unhide".into());
        }
        if layer.is_none() {
            return Err("player view has no CAMetalLayer".into());
        }
        if msg0_bool_pub(webview, "isOpaque") {
            return Err("web view stayed opaque, video would be covered".into());
        }
        let draws = msg1_id(webview, "valueForKey:", nsstring("drawsBackground"));
        if !draws.is_null() && msg0_bool_pub(draws, "boolValue") {
            return Err("web view still draws its background".into());
        }

        Ok(format!(
            "placement ok: view under {}, webview opaque=false drawsBackground=false, metal layer present",
            class_name(content_view)
        ))
    }
}
