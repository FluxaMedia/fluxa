// Linux native player surface — libmpv OpenGL render API via GLX/EGL.
//
// Architecture:
//   • A GtkGLArea is placed BEHIND the WebView inside a GtkOverlay.
//   • The WebView stays visible with a transparent background (Tauri transparent=true).
//   • The React overlay (ReactPlayerOverlay.tsx) renders on top and handles all
//     player controls — no GTK controls are built here.
//   • On Load/Hide we emit native-player-show / native-player-hide so the React
//     overlay activates / deactivates, mirroring the Windows and macOS surfaces.

use crate::linux_vulkan::{NativeSurface, VulkanContext};
use crate::mpv_render::VulkanTargetImage;
use crate::playback_engine::{PlaybackEngine, PlayerEngine};
use crate::DesktopState;
use fluxa_core::FluxaCore;
use glib::ControlFlow;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use webkit2gtk::WebViewExt;

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

enum VulkanSurfaceHandle {
    Wayland(crate::linux_wayland_subsurface::VideoSubsurface),
    X11(gdk::Window),
}

impl VulkanSurfaceHandle {
    fn hide(&self) {
        match self {
            Self::Wayland(sub) => sub.hide(),
            Self::X11(window) => window.hide(),
        }
    }

    fn show(&self) {
        match self {
            Self::Wayland(_) => {}
            Self::X11(window) => {
                window.show_unraised();
                window.lower();
            }
        }
    }

    fn resize(&self, width: i32, height: i32) {
        if let Self::X11(window) = self {
            window.move_resize(0, 0, width, height);
        }
    }
}

struct VulkanShared {
    width: AtomicI32,
    height: AtomicI32,
    hdr: AtomicBool,
    mpv_context_ready: AtomicBool,
}

struct VulkanState {
    shared: Arc<VulkanShared>,
    surface_handle: VulkanSurfaceHandle,
}

const VULKAN_CONTEXT_PENDING: &str = "vulkan render context pending";

const PRESENT_WATCHDOG_INTERVAL: Duration = Duration::from_millis(200);

fn spawn_vulkan_render_thread(app: AppHandle, mut ctx: VulkanContext, shared: Arc<VulkanShared>) {
    std::thread::Builder::new()
        .name("vulkan-player-render".into())
        .spawn(move || {
            let mut last_present = Instant::now();
            let mut last_context_error: Option<String> = None;
            let mut last_render_error: Option<String> = None;
            loop {
                let w = shared.width.load(Ordering::Acquire);
                let h = shared.height.load(Ordering::Acquire);
                if w > 1 && h > 1 {
                    if let Err(e) = ctx.resize(w, h) {
                        log::warn!("linux_player_surface: Vulkan resize failed: {e}");
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
                let forced = last_present.elapsed() >= PRESENT_WATCHDOG_INTERVAL;
                shared.hdr.store(ctx.is_hdr(), Ordering::Release);

                let state = app.state::<DesktopState>();
                let mut wire_error = false;
                let ready = {
                    let Ok(mut guard) = state.player_render_state.try_lock() else {
                        std::thread::sleep(Duration::from_millis(2));
                        continue;
                    };
                    match guard.as_mut() {
                        None => {
                            shared.mpv_context_ready.store(false, Ordering::Release);
                            false
                        }
                        Some(renderer) => {
                            if renderer.needs_vulkan_context() {
                                shared.mpv_context_ready.store(false, Ordering::Release);
                                let (
                                    instance,
                                    phys_device,
                                    device,
                                    queue_index,
                                    queue_count,
                                    get_proc_addr,
                                ) = ctx.device_handles();
                                let ext_ptrs = ctx.enabled_device_extension_ptrs();
                                match renderer.create_vulkan_context(
                                    instance,
                                    phys_device,
                                    device,
                                    queue_index,
                                    queue_count,
                                    get_proc_addr,
                                    &ext_ptrs,
                                ) {
                                    Ok(()) => {
                                        last_context_error = None;
                                        shared.mpv_context_ready.store(true, Ordering::Release)
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "linux_player_surface: mpv Vulkan context failed: {e}"
                                        );
                                        if last_context_error.as_deref() != Some(e.as_str()) {
                                            crate::diagnostics::report(
                                                &app,
                                                format!("linux_player_surface: mpv Vulkan context failed: {e}"),
                                                sentry::Level::Error,
                                            );
                                            last_context_error = Some(e.clone());
                                        }
                                        wire_error = true;
                                    }
                                }
                            } else {
                                shared.mpv_context_ready.store(true, Ordering::Release);
                            }
                            !wire_error && (forced || renderer.vulkan_frame_ready())
                        }
                    }
                };
                if wire_error {
                    std::thread::sleep(Duration::from_millis(250));
                    continue;
                }
                if !ready {
                    std::thread::sleep(Duration::from_millis(4));
                    continue;
                }

                let image_usage = ctx.image_usage();
                let result = ctx.render_and_present(
                    |image, format, iw, ih, wait_semaphore, signal_semaphore| {
                        let mut guard = state
                            .player_render_state
                            .lock()
                            .map_err(|_| "player renderer lock poisoned".to_string())?;
                        let renderer = guard
                            .as_mut()
                            .ok_or_else(|| "player renderer destroyed".to_string())?;
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
                        renderer
                            .render_vulkan_frame(&mut target)
                            .map(|_| target.layout)
                    },
                );
                match result {
                    Ok(()) => {
                        last_present = Instant::now();
                        last_render_error = None;
                        if let Ok(mut guard) = state.player_render_state.lock() {
                            if let Some(r) = guard.as_mut() {
                                r.report_swap();
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("linux_player_surface: Vulkan render failed: {e}");
                        if last_render_error.as_deref() != Some(e.as_str()) {
                            crate::diagnostics::report(
                                &app,
                                format!("linux_player_surface: Vulkan render failed: {e}"),
                                sentry::Level::Error,
                            );
                            last_render_error = Some(e.clone());
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        })
        .expect("failed to spawn vulkan render thread");
}

fn create_vulkan_surface(
    anchor_widget: &impl IsA<gtk::Widget>,
    width: i32,
    height: i32,
) -> Result<(VulkanSurfaceHandle, NativeSurface), String> {
    use glib::translate::ToGlibPtr;

    let toplevel = anchor_widget
        .toplevel()
        .ok_or("Vulkan surface setup: widget has no toplevel ancestor")?;
    let toplevel_window: gtk::Window = toplevel
        .downcast()
        .map_err(|_| "Vulkan surface setup: toplevel ancestor is not a gtk::Window".to_string())?;
    let parent_gdk_window = toplevel_window
        .window()
        .ok_or("Vulkan surface setup: toplevel window is not realized yet")?;

    if let Ok(wl_parent) = parent_gdk_window
        .clone()
        .downcast::<gdkwayland::WaylandWindow>()
    {
        let parent_wl_surface = unsafe {
            gdkwayland::ffi::gdk_wayland_window_get_wl_surface(wl_parent.to_glib_none().0)
        };
        let display = gdk::Display::default().ok_or("Vulkan surface setup: no default display")?;
        let wayland_display: gdkwayland::WaylandDisplay = display
            .downcast()
            .map_err(|_| "Vulkan surface setup: default display is not Wayland".to_string())?;
        let wl_display = unsafe {
            gdkwayland::ffi::gdk_wayland_display_get_wl_display(wayland_display.to_glib_none().0)
        };
        let wl_compositor = unsafe {
            gdkwayland::ffi::gdk_wayland_display_get_wl_compositor(wayland_display.to_glib_none().0)
        };
        if parent_wl_surface.is_null() || wl_display.is_null() || wl_compositor.is_null() {
            return Err("Vulkan surface setup: could not resolve Wayland handles".into());
        }
        log::debug!("linux_player_surface: creating subsurface with parent_wl_surface={parent_wl_surface:p}");
        let subsurface = crate::linux_wayland_subsurface::VideoSubsurface::new(
            wl_display as *mut _,
            wl_compositor as *mut _,
            parent_wl_surface as *mut _,
        )?;
        let native_surface = NativeSurface::Wayland {
            display: wl_display as *mut _,
            surface: subsurface.wl_surface(),
        };
        return Ok((VulkanSurfaceHandle::Wayland(subsurface), native_surface));
    }

    let attrs = gdk::WindowAttr {
        window_type: gdk::WindowType::Child,
        wclass: gdk::WindowWindowClass::InputOutput,
        x: Some(0),
        y: Some(0),
        width: width.max(2),
        height: height.max(2),
        ..Default::default()
    };
    let window = gdk::Window::new(Some(&parent_gdk_window), &attrs);
    window.show_unraised();
    window.lower();
    let native_surface = native_surface_for_gdk_window(&window)
        .ok_or("Vulkan surface setup: could not resolve a native surface for the X11 window")?;
    Ok((VulkanSurfaceHandle::X11(window), native_surface))
}

fn log_current_toplevel_wl_surface(gl_area: &gtk::GLArea, label: &str) {
    use glib::translate::ToGlibPtr;
    let Some(toplevel) = gl_area.toplevel() else {
        return;
    };
    let Ok(toplevel_window) = toplevel.downcast::<gtk::Window>() else {
        return;
    };
    let Some(gdk_window) = toplevel_window.window() else {
        return;
    };
    let Ok(wl_window) = gdk_window.downcast::<gdkwayland::WaylandWindow>() else {
        return;
    };
    let wl_surface =
        unsafe { gdkwayland::ffi::gdk_wayland_window_get_wl_surface(wl_window.to_glib_none().0) };
    log::debug!("linux_player_surface: {label} current toplevel wl_surface={wl_surface:p}");
}

fn reassert_webview_transparency(webview_widget: &gtk::Widget) {
    if let Ok(webview) = webview_widget.clone().downcast::<webkit2gtk::WebView>() {
        webview.set_background_color(&gdk::RGBA::new(0.0, 0.0, 0.0, 1.0 / 255.0));
    }
}

fn vulkan_surface_size(gl_area: &gtk::GLArea, webview_widget: &gtk::Widget) -> (i32, i32) {
    let scale = webview_widget.scale_factor().max(1);
    let (w, h) = gl_area
        .toplevel()
        .map(|t| (t.allocated_width(), t.allocated_height()))
        .unwrap_or_else(|| {
            (
                webview_widget.allocated_width(),
                webview_widget.allocated_height(),
            )
        });
    (w.max(2) * scale, h.max(2) * scale)
}

fn native_surface_for_gdk_window(window: &gdk::Window) -> Option<NativeSurface> {
    use glib::translate::ToGlibPtr;

    if let Ok(wl_window) = window.clone().downcast::<gdkwayland::WaylandWindow>() {
        let wl_surface = unsafe {
            gdkwayland::ffi::gdk_wayland_window_get_wl_surface(wl_window.to_glib_none().0)
        };
        let display = gdk::Display::default()?;
        let wayland_display: gdkwayland::WaylandDisplay = display.downcast().ok()?;
        let wl_display = unsafe {
            gdkwayland::ffi::gdk_wayland_display_get_wl_display(wayland_display.to_glib_none().0)
        };
        if wl_surface.is_null() || wl_display.is_null() {
            return None;
        }
        return Some(NativeSurface::Wayland {
            display: wl_display as *mut _,
            surface: wl_surface as *mut _,
        });
    }
    if let Ok(x11_window) = window.clone().downcast::<gdkx11::X11Window>() {
        let xid = unsafe { gdkx11::ffi::gdk_x11_window_get_xid(x11_window.to_glib_none().0) };
        let display = gdk::Display::default()?;
        let x11_display: gdkx11::X11Display = display.downcast().ok()?;
        let xdisplay =
            unsafe { gdkx11::ffi::gdk_x11_display_get_xdisplay(x11_display.to_glib_none().0) };
        if xdisplay.is_null() {
            return None;
        }
        return Some(NativeSurface::Xlib {
            display: xdisplay as *mut _,
            window: xid as u64,
        });
    }
    None
}

// Chapter / skip-segment utilities (polled by player_get_playback_info)

struct Chapter {
    title: String,
    start_ms: i64,
}

fn read_mpv_chapters(renderer: &crate::mpv_render::MpvClientHandle) -> Vec<Chapter> {
    let count: usize = renderer
        .query_property("chapters")
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    if count == 0 {
        return vec![];
    }
    (0..count)
        .filter_map(|i| {
            let title = renderer
                .query_property(&format!("chapter-list/{i}/title"))
                .unwrap_or_default();
            let start_secs: f64 = renderer
                .query_property(&format!("chapter-list/{i}/time"))
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0.0);
            Some(Chapter {
                title,
                start_ms: (start_secs * 1000.0).round() as i64,
            })
        })
        .collect()
}

fn chapters_to_json(chapters: &[Chapter]) -> String {
    let arr: Vec<serde_json::Value> = chapters
        .iter()
        .map(|c| serde_json::json!({ "title": c.title, "startTime": c.start_ms }))
        .collect();
    serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
}

// Surface commands

enum SurfaceCommand {
    Load {
        url: String,
        start_at: Option<u64>,
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
        _total_duration: Option<u64>,
    ) -> Result<(), String> {
        self.sender
            .send(SurfaceCommand::Load { url, start_at })
            .map_err(|e| format!("native player surface is not available: {e}"))
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
        _background: Option<(Vec<u8>, i32, i32)>,
        _logo: Option<(Vec<u8>, i32, i32)>,
    ) {
        let _ = self.sender.send(SurfaceCommand::SetArtwork {
            title,
            episode_title,
        });
    }
}

// Per-tick state for the GTK main-loop timer that drives surface commands,
// render-context warmup, deferred loads, and playing-state housekeeping.
// Bundled into a struct (rather than closure-captured locals) so each
// concern below is a named, independently readable method.
struct SurfaceTick {
    receiver: mpsc::Receiver<SurfaceCommand>,
    app: AppHandle,
    gl_area: gtk::GLArea,
    webview_widget: gtk::Widget,
    backend: RenderBackend,
    visible: Rc<Cell<bool>>,
    vulkan_state: Rc<RefCell<Option<VulkanState>>>,
    pending_load: Option<(String, Option<u64>)>,
    pending_load_retries: u32,
    latch_grace_ticks: u32,
    chapters_native_loaded: bool,
    transparency_reassert_ticks: u32,
    screenshot_countdown_ticks: i32,
    screenshot_seq: u32,
}

impl SurfaceTick {
    fn on_tick(&mut self) -> ControlFlow {
        self.drain_commands();
        self.warm_up_render_context();
        self.retry_pending_load();
        if self.visible.get() {
            self.run_frame_tasks();
        }
        ControlFlow::Continue
    }

    fn drain_commands(&mut self) {
        while let Ok(command) = self.receiver.try_recv() {
            match command {
                SurfaceCommand::Load { url, start_at } => {
                    self.app.state::<DesktopState>().pending_hide
                        .store(false, std::sync::atomic::Ordering::Release);
                    let state = self.app.state::<DesktopState>();
                    let mut overlay = state.player_overlay.lock().unwrap();
                    overlay.eof_next_fired = false;
                    overlay.chapters_json = None;
                    if *self.app.state::<DesktopState>().active_player_engine.lock().unwrap() == PlayerEngine::Vlc {
                        self.gl_area.set_auto_render(false);
                        self.gl_area.show();
                        let gl_area = &self.gl_area;
                        let app = &self.app;
                        let result = (|| {
                            use glib::translate::ToGlibPtr;
                            let window = gl_area.window().ok_or_else(|| "libVLC requires a realized X11 video surface".to_string())?;
                            let x11_window = window.downcast::<gdkx11::X11Window>().map_err(|_| "embedded libVLC currently requires an X11 session; Wayland needs the libVLC video-callback renderer".to_string())?;
                            let xid = unsafe { gdkx11::ffi::gdk_x11_window_get_xid(x11_window.to_glib_none().0) };
                            let state = app.state::<DesktopState>();
                            let mut players = state.player_renderer_vlc.lock().unwrap();
                            if players.is_none() {
                                *players = Some(crate::libvlc_render::LibvlcPlayer::new()?);
                            }
                            let player = players.as_mut().ok_or_else(|| "libVLC player is unavailable".to_string())?;
                            player.attach_xwindow(xid as u32)?;
                            player.load(&url, start_at)
                        })();
                        if let Err(error) = result {
                            self.visible.set(false);
                            self.gl_area.hide();
                            let _ = self.app.emit("native-player-error", error);
                        } else {
                            self.visible.set(true);
                            let _ = self.app.emit("native-player-show", ());
                        }
                        continue;
                    }
                    if self.backend == RenderBackend::OpenGl {
                        self.gl_area.set_auto_render(true);
                        self.gl_area.show();
                    } else if let Some(state) = self.vulkan_state.borrow().as_ref() {
                        state.surface_handle.show();
                    }
                    self.visible.set(true);
                    self.latch_grace_ticks = 10;
                    self.chapters_native_loaded = false;
                    self.pending_load = Some((url, start_at));
                    self.pending_load_retries = 0;
                    let emit_result = self.app.emit("native-player-show", ());
                    log::info!("linux_player_surface: emitted native-player-show, result={emit_result:?}");
                }
                SurfaceCommand::Hide => {
                    self.visible.set(false);
                    self.pending_load = None;
                    self.pending_load_retries = 0;
                    self.app.state::<DesktopState>().pending_hide
                        .store(true, std::sync::atomic::Ordering::Release);
                    if self.backend == RenderBackend::OpenGl {
                        self.gl_area.set_auto_render(false);
                        self.gl_area.hide();
                    } else if let Some(state) = self.vulkan_state.borrow().as_ref() {
                        state.surface_handle.hide();
                    }
                    let _ = self.app.emit("native-player-hide", ());
                    if let Ok(guard) = self.app.state::<DesktopState>().player_mpv_client.try_lock() {
                        if let Some(r) = guard.as_ref() {
                            let _ = r.command_args(&["stop"]);
                        }
                    }
                }
                SurfaceCommand::ShowLoading { title, episode_title } => {
                    self.app.state::<DesktopState>().pending_hide
                        .store(false, std::sync::atomic::Ordering::Release);
                    let _ = self.app.emit(
                        "native-player-title",
                        serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                    );
                }
                SurfaceCommand::SetTitle { title, episode_title } => {
                    let _ = self.app.emit(
                        "native-player-title",
                        serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                    );
                }
                SurfaceCommand::SetArtwork { title, episode_title } => {
                    // Artwork display is handled by PlayerLoadingOverlay in the WebView.
                    // We just forward the title so ReactPlayerOverlay can show it.
                    let _ = self.app.emit(
                        "native-player-title",
                        serde_json::json!({ "title": title, "episodeTitle": episode_title }),
                    );
                }
            }
        }
    }

    // Warm up the render context before first play. Blocking work here (idle,
    // no user waiting) is better than doing it inside prepare_and_load when the
    // loading screen is shown.
    fn warm_up_render_context(&mut self) {
        let warmup_ready = match self.backend {
            RenderBackend::OpenGl => self.gl_area.is_realized(),
            RenderBackend::Vulkan => self.gl_area
                .toplevel()
                .map(|t| t.is_realized())
                .unwrap_or(false),
        };
        if self.pending_load.is_none() && warmup_ready {
            match self.backend {
                RenderBackend::OpenGl => {
                    if let Ok(mut guard) = self.app.state::<DesktopState>().player_render_state.try_lock() {
                        if let Some(renderer) = guard.as_mut() {
                            if renderer.needs_opengl_context() {
                                self.gl_area.make_current();
                                if self.gl_area.error().is_none() {
                                    let _ = renderer.prepare_opengl_context();
                                }
                            }
                        }
                    }
                }
                RenderBackend::Vulkan => {
                    if self.vulkan_state.borrow().is_none() {
                        let (w, h) = vulkan_surface_size(&self.gl_area, &self.webview_widget);
                        match create_vulkan_surface(&self.gl_area, w, h) {
                            Ok((surface_handle, native_surface)) => {
                                match crate::linux_vulkan::create_context(native_surface, w, h) {
                                    Ok(ctx) => {
                                        let shared = Arc::new(VulkanShared {
                                            width: AtomicI32::new(w),
                                            height: AtomicI32::new(h),
                                            hdr: AtomicBool::new(false),
                                            mpv_context_ready: AtomicBool::new(false),
                                        });
                                        spawn_vulkan_render_thread(
                                            self.app.clone(),
                                            ctx,
                                            shared.clone(),
                                        );
                                        *self.vulkan_state.borrow_mut() =
                                            Some(VulkanState { shared, surface_handle })
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "linux_player_surface: Vulkan context creation failed: {e}"
                                        );
                                        crate::diagnostics::report(
                                            &self.app,
                                            format!("linux_player_surface: Vulkan context creation failed: {e}"),
                                            sentry::Level::Error,
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("linux_player_surface: {e}");
                                crate::diagnostics::report(
                                    &self.app,
                                    format!("linux_player_surface: {e}"),
                                    sentry::Level::Error,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn retry_pending_load(&mut self) {
        let Some((url, start_at)) = self.pending_load.clone() else {
            return;
        };
        let state = self.app.state::<DesktopState>();
        if state.player_mpv_client.try_lock().is_ok() {
            match prepare_and_load(&self.app, &self.gl_area, self.backend, &self.vulkan_state, &url, start_at) {
                Ok(()) => {
                    self.pending_load = None;
                    self.latch_grace_ticks = 10;
                    let hide_pending = self.app.state::<DesktopState>()
                        .pending_hide.load(std::sync::atomic::Ordering::Acquire);
                    if hide_pending {
                        self.visible.set(false);
                        self.gl_area.hide();
                    }
                }
                Err(e) if e == VULKAN_CONTEXT_PENDING => {
                    self.pending_load_retries += 1;
                    if self.pending_load_retries > 300 {
                        self.pending_load = None;
                        self.visible.set(false);
                        self.gl_area.hide();
                        let _ = self.app.emit("native-player-error", e);
                    }
                }
                Err(e) => {
                    self.pending_load = None;
                    self.visible.set(false);
                    self.gl_area.hide();
                    log::warn!("native player load failed: {e}");
                    let _ = self.app.emit("native-player-error", e);
                }
            }
        } else {
            self.pending_load_retries += 1;
            if self.pending_load_retries > 300 {
                self.pending_load = None;
                self.visible.set(false);
                self.gl_area.hide();
                let _ = self.app.emit("native-player-error", "player renderer busy".to_string());
            }
        }
    }

    fn run_frame_tasks(&mut self) {
        let in_grace = if self.latch_grace_ticks > 0 {
            self.latch_grace_ticks -= 1;
            true
        } else {
            false
        };

        // Read native chapters from mpv once playback has started.
        // Stored in DesktopState so player_get_playback_info() can return them
        // to the React overlay for chapter-segmented seekbar rendering.
        if !in_grace && !self.chapters_native_loaded {
            let state = self.app.state::<DesktopState>();
            let pos = state.player_mpv_client.try_lock().ok()
                .and_then(|g| g.as_ref()
                    .and_then(|r| r.query_property("time-pos")
                        .and_then(|v| v.trim().parse::<f64>().ok())))
                .unwrap_or(0.0);
            if pos > 0.05 {
                self.chapters_native_loaded = true;
                let chapters_already_set = state.player_overlay.lock().unwrap().chapters_json.is_some();
                if !chapters_already_set {
                    if let Ok(guard) = state.player_mpv_client.try_lock() {
                        if let Some(renderer) = guard.as_ref() {
                            let native = read_mpv_chapters(renderer);
                            if !native.is_empty() {
                                let chapters_json = chapters_to_json(&native);
                                let mut overlay = state.player_overlay.lock().unwrap();
                                overlay.chapters_json = Some(chapters_json.clone());
                                let skip_already_set = overlay.skip_segments_json.is_some();
                                let chapter_skip_enabled = overlay.use_chapter_skip;
                                if !skip_already_set && chapter_skip_enabled {
                                    let derived = FluxaCore::chapter_skip_segments_json(&chapters_json);
                                    if derived != "[]" {
                                        overlay.skip_segments_json = Some(derived);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if self.backend == RenderBackend::Vulkan {
            let (w, h) = vulkan_surface_size(&self.gl_area, &self.webview_widget);
            if let Some(state) = self.vulkan_state.borrow().as_ref() {
                state.surface_handle.resize(w, h);
                let prev_w = state.shared.width.swap(w, Ordering::AcqRel);
                let prev_h = state.shared.height.swap(h, Ordering::AcqRel);
                self.transparency_reassert_ticks += 1;
                if prev_w != w || prev_h != h || self.transparency_reassert_ticks >= 60 {
                    self.transparency_reassert_ticks = 0;
                    reassert_webview_transparency(&self.webview_widget);
                }
                if prev_w != w || prev_h != h {
                    log_current_toplevel_wl_surface(&self.gl_area, "resize");
                    self.screenshot_countdown_ticks = 30;
                }
            }
        }

        if self.screenshot_countdown_ticks >= 0 {
            self.screenshot_countdown_ticks -= 1;
            if self.screenshot_countdown_ticks == 0 {
                if let Ok(guard) = self.app.state::<DesktopState>().player_mpv_client.try_lock() {
                    if let Some(r) = guard.as_ref() {
                        self.screenshot_seq += 1;
                        let path = format!("/tmp/fluxa_mpv_screenshot_{}.png", self.screenshot_seq);
                        let _ = r.command_args(&["screenshot-to-file", &path]);
                        log::debug!("linux_player_surface: mpv screenshot-to-file requested at {path}");
                    }
                }
            }
        }

        check_player_events(&self.app);
    }
}

// Install

pub fn install(app_handle: AppHandle) -> Result<NativePlayerSurface, String> {
    let (sender, receiver) = mpsc::channel();
    let (setup_tx, setup_rx) = mpsc::channel::<Result<(), String>>();

    let backend = read_render_backend(&app_handle);
    log::info!(
        "linux_player_surface: experimental render backend = {}",
        match backend {
            RenderBackend::OpenGl => "opengl",
            RenderBackend::Vulkan => "vulkan",
        }
    );

    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "main webview window was not found".to_string())?;

    window
        .with_webview(move |platform_webview| {
            let webview_widget = platform_webview.inner().upcast::<gtk::Widget>();
            let Some(parent_widget) = webview_widget.parent() else {
                let _ = setup_tx.send(Err("WebView parent widget not found — cannot attach player surface".to_string()));
                return;
            };
            let Ok(parent_box) = parent_widget.downcast::<gtk::Box>() else {
                let _ = setup_tx.send(Err(
                    "WebView parent is not a GTK Box; widget hierarchy may differ on this Tauri build".to_string(),
                ));
                return;
            };

            // GLArea for mpv OpenGL rendering — initially hidden, shown only when playing.
            let gl_area = gtk::GLArea::new();
            gl_area.set_hexpand(true);
            gl_area.set_vexpand(true);
            gl_area.set_has_alpha(false);
            gl_area.set_auto_render(false);
            gl_area.set_required_version(3, 2);
            // no_show_all prevents parent show_all() from revealing the video prematurely.
            gl_area.set_no_show_all(true);

            // Wrap in a GtkOverlay: GLArea (base/background) + WebView (overlay/foreground).
            // The WebView is transparent when the player is active (see App.tsx + transparent=true
            // in tauri.conf.json), so the video shows through from the GLArea behind it.
            let video_overlay = gtk::Overlay::new();
            video_overlay.set_hexpand(true);
            video_overlay.set_vexpand(true);

            webview_widget.set_hexpand(true);
            webview_widget.set_vexpand(true);

            // Reparent: move the WebView from parent_box into the overlay (on top of GLArea).
            parent_box.remove(&webview_widget);
            video_overlay.add(&gl_area);
            video_overlay.add_overlay(&webview_widget);
            webview_widget.set_halign(gtk::Align::Fill);
            webview_widget.set_valign(gtk::Align::Fill);
            parent_box.pack_start(&video_overlay, true, true, 0);
            parent_box.show_all();
            webview_widget.show();
            webview_widget.queue_resize();
            webview_widget.queue_draw();
            reassert_webview_transparency(&webview_widget);
            video_overlay.queue_resize();
            video_overlay.queue_draw();
            // gl_area stays hidden (no_show_all) until the first Load command.

            // Render callback: called by GTK when gl_area.queue_render() is invoked.
            if backend == RenderBackend::OpenGl {
                let render_app = app_handle.clone();
                let last_gl_render_error: RefCell<Option<String>> = RefCell::new(None);
                gl_area.connect_render(move |area, _ctx| {
                    let state = render_app.state::<DesktopState>();
                    if state.pending_hide.load(std::sync::atomic::Ordering::Acquire) {
                        // Returning Proceed (unhandled) with auto_render=false skips GTK's
                        // gdk_cairo_draw_from_gl readback path — prevents a multi-second freeze
                        // that occurs on some composited systems when the last video frame is read.
                        area.set_auto_render(false);
                        return glib::Propagation::Proceed;
                    }
                    area.make_current();
                    if area.error().is_some() {
                        return glib::Propagation::Stop;
                    }
                    let scale = area.scale_factor().max(1);
                    let w = area.allocated_width().max(2) * scale;
                    let h = area.allocated_height().max(2) * scale;
                    let Ok(mut renderer) = state.player_render_state.try_lock() else {
                        return glib::Propagation::Stop;
                    };
                    if let Some(r) = renderer.as_mut() {
                        if let Err(e) = r.render_opengl_frame(w, h) {
                            log::warn!("mpv OpenGL render failed: {e}");
                            if last_gl_render_error.borrow().as_deref() != Some(e.as_str()) {
                                crate::diagnostics::report(
                                    &render_app,
                                    format!("mpv OpenGL render failed: {e}"),
                                    sentry::Level::Error,
                                );
                                *last_gl_render_error.borrow_mut() = Some(e.clone());
                            }
                        } else {
                            *last_gl_render_error.borrow_mut() = None;
                        }
                        r.report_swap();
                    }
                    glib::Propagation::Stop
                });
            }

            let _ = setup_tx.send(Ok(()));
            // Shared with the GdkFrameClock tick callback below, both on the GTK main thread.
            let visible = Rc::new(Cell::new(false));
            let tick_visible = visible.clone();
            let tick_gl_area = gl_area.clone();
            gl_area.add_tick_callback(move |_area, _frame_clock| {
                if tick_visible.get() && backend == RenderBackend::OpenGl {
                    tick_gl_area.queue_render();
                }
                glib::ControlFlow::Continue
            });

            let mut tick = SurfaceTick {
                receiver,
                app: app_handle.clone(),
                gl_area: gl_area.clone(),
                webview_widget: webview_widget.clone(),
                backend,
                visible,
                vulkan_state: Rc::new(RefCell::new(None)),
                pending_load: None,
                pending_load_retries: 0,
                latch_grace_ticks: 0,
                chapters_native_loaded: false,
                transparency_reassert_ticks: 0,
                screenshot_countdown_ticks: -1,
                screenshot_seq: 0,
            };
            glib::timeout_add_local(Duration::from_millis(16), move || tick.on_tick());
        })
        .map_err(|e| e.to_string())?;

    setup_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "native player surface setup timed out".to_string())
        .and_then(|r| r)
        .map(|()| NativePlayerSurface { sender })
}

// Helpers

fn prepare_and_load(
    app_handle: &AppHandle,
    gl_area: &gtk::GLArea,
    backend: RenderBackend,
    vulkan_state: &Rc<RefCell<Option<VulkanState>>>,
    url: &str,
    start_at: Option<u64>,
) -> Result<(), String> {
    if backend == RenderBackend::OpenGl && !gl_area.is_realized() {
        gl_area.realize();
    }

    let state = app_handle.state::<DesktopState>();
    let mut render_guard = state
        .player_render_state
        .try_lock()
        .map_err(|_| "player renderer busy — load deferred".to_string())?;
    let mut client_guard = state
        .player_mpv_client
        .try_lock()
        .map_err(|_| "player renderer busy — load deferred".to_string())?;
    if client_guard.is_none() {
        let (client, render) = crate::mpv_render::MpvClientHandle::new()?;
        *render_guard = Some(render);
        *client_guard = Some(client);
    }
    let render = render_guard
        .as_mut()
        .ok_or_else(|| "player renderer is not initialized".to_string())?;
    let client = client_guard
        .as_mut()
        .ok_or_else(|| "player renderer is not initialized".to_string())?;

    match backend {
        RenderBackend::OpenGl => {
            gl_area.make_current();
            if let Some(error) = gl_area.error() {
                return Err(format!("OpenGL player surface context error: {error}"));
            }
            render.prepare_opengl_context()?;
        }
        RenderBackend::Vulkan => {
            let shared = vulkan_state
                .borrow()
                .as_ref()
                .map(|vs| vs.shared.clone())
                .ok_or_else(|| VULKAN_CONTEXT_PENDING.to_string())?;
            if render.needs_vulkan_context() && !shared.mpv_context_ready.load(Ordering::Acquire)
            {
                return Err(VULKAN_CONTEXT_PENDING.to_string());
            }
            if shared.hdr.load(Ordering::Acquire) {
                let _ = client.set_option("target-trc", "linear");
                let _ = client.set_option("target-prim", "bt.709");
            }
        }
    }

    if let Some(fps) = monitor_refresh_fps(gl_area) {
        if let Err(e) = client.set_option("display-fps-override", &format!("{fps:.3}")) {
            log::warn!("failed to set display-fps-override: {e}");
        }
    }
    if let Some(icc) = query_x11_icc_profile() {
        if let Err(e) = render.set_icc_profile(&icc) {
            log::warn!("failed to set ICC profile: {e}");
        }
    }
    client.load(url, start_at)
}

fn monitor_refresh_fps(gl_area: &gtk::GLArea) -> Option<f64> {
    let window = gl_area.window()?;
    let monitor = gl_area.display().monitor_at_window(&window)?;
    let mhz = monitor.refresh_rate();
    (mhz > 0).then(|| f64::from(mhz) / 1000.0)
}

// X11-only; no equivalent mechanism under Wayland.
fn query_x11_icc_profile() -> Option<Vec<u8>> {
    use gdkx11::x11::xlib;
    use glib::translate::ToGlibPtr;
    use std::ffi::CString;

    let display = gdk::Display::default()?;
    let x11_display: gdkx11::X11Display = display.downcast().ok()?;
    let xdisplay =
        unsafe { gdkx11::ffi::gdk_x11_display_get_xdisplay(x11_display.to_glib_none().0) };
    if xdisplay.is_null() {
        return None;
    }

    unsafe {
        let root = xlib::XDefaultRootWindow(xdisplay);
        let atom_name = CString::new("_ICC_PROFILE").ok()?;
        let atom = xlib::XInternAtom(xdisplay, atom_name.as_ptr(), 1 /* only_if_exists */);
        if atom == 0 {
            return None;
        }

        let mut actual_type: xlib::Atom = 0;
        let mut actual_format: i32 = 0;
        let mut nitems: std::os::raw::c_ulong = 0;
        let mut bytes_after: std::os::raw::c_ulong = 0;
        let mut prop: *mut std::os::raw::c_uchar = std::ptr::null_mut();
        let result = xlib::XGetWindowProperty(
            xdisplay,
            root,
            atom,
            0,
            i64::MAX / 4,
            0,
            xlib::AnyPropertyType as std::os::raw::c_ulong,
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut prop,
        );
        if result != 0 || prop.is_null() || nitems == 0 {
            return None;
        }
        let data = std::slice::from_raw_parts(prop, nitems as usize).to_vec();
        xlib::XFree(prop as *mut std::os::raw::c_void);
        Some(data)
    }
}

fn check_player_events(app: &AppHandle) {
    let state = app.state::<DesktopState>();
    let (events, eof) = {
        let Ok(mut renderer) = state.player_mpv_client.try_lock() else {
            return;
        };
        let Some(r) = renderer.as_mut() else {
            return;
        };
        let events = r.poll_events();
        let eof = r.query_property("eof-reached").as_deref() == Some("yes");
        (events, eof)
    };
    crate::player_surface_events::drain_player_events(app, events);
    if !eof {
        crate::player_surface_events::clear_eof_latch(app);
        return;
    }
    crate::player_surface_events::fire_eof_transition(app);
}
