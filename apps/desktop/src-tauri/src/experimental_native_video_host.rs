use crate::DesktopState;
use crate::linux_native_render::VulkanShared;
use crate::mpv_render::{MpvClientHandle, PlayerStatus, PlayerTrackOption};
use crate::player_surface::{Artwork, PlayerSurface};
use glib::translate::ToGlibPtr;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use webkit2gtk::WebViewExt;

static NATIVE_MINI_MODE: OnceLock<std::sync::Mutex<bool>> = OnceLock::new();

pub fn set_mini_mode(mini: bool) {
    *NATIVE_MINI_MODE
        .get_or_init(|| std::sync::Mutex::new(false))
        .lock()
        .unwrap() = mini;
}

fn mini_mode() -> bool {
    NATIVE_MINI_MODE
        .get_or_init(|| std::sync::Mutex::new(false))
        .lock()
        .map(|mode| *mode)
        .unwrap_or(false)
}

enum HostWindow {
    X11(gdk::Window),
    Wayland(crate::linux_wayland_subsurface::VideoSubsurface),
}

/// Wayland fullscreen treats the Tauri/WebKit toplevel as an opaque parent
/// for its subsurfaces on some compositors.  In that mode the HTML controls
/// need their own surface, while the Vulkan video can stay above the original
/// parent surface.  This keeps the existing React player intact and only
/// changes the native surface it is hosted in.
struct WebViewOverlaySurface {
    window: gtk::Window,
    webview: gtk::Widget,
    original_parent: gtk::Container,
    attached: bool,
}

impl WebViewOverlaySurface {
    fn new(top: &gtk::Window, webview: &gtk::Widget) -> Option<Self> {
        let Some(parent) = webview.parent() else {
            log::warn!("experimental-native: WebView overlay has no GTK parent");
            return None;
        };
        let original_parent = match parent.downcast::<gtk::Container>() {
            Ok(parent) => parent,
            Err(parent) => {
                log::warn!(
                    "experimental-native: WebView overlay parent is not a GTK Container type={}",
                    parent.type_().name()
                );
                return None;
            }
        };
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_skip_taskbar_hint(true);
        window.set_skip_pager_hint(true);
        window.set_accept_focus(false);
        window.set_focus_on_map(false);
        window.set_keep_above(true);
        window.set_app_paintable(true);
        window.set_transient_for(Some(top));
        if let Some(screen) = gtk::prelude::GtkWindowExt::screen(top) {
            window.set_screen(&screen);
        }
        if let Some(visual) = top.visual() {
            window.set_visual(Some(&visual));
        }
        window.hide();
        Some(Self {
            window,
            webview: webview.clone(),
            original_parent,
            attached: false,
        })
    }

    fn attach(&mut self) {
        if self.attached {
            return;
        }
        self.original_parent.remove(&self.webview);
        self.window.add(&self.webview);
        self.webview.show();
        self.window.show_all();
        self.attached = true;
        log::info!("experimental-native: React/WebView moved to separate overlay surface");
    }

    fn detach(&mut self) {
        if !self.attached {
            return;
        }
        self.window.remove(&self.webview);
        self.original_parent.add(&self.webview);
        self.webview.show();
        self.original_parent.show_all();
        self.window.hide();
        self.attached = false;
    }

    fn sync_geometry(&mut self, top: &gtk::Window, width: i32, height: i32) {
        if !self.attached {
            return;
        }
        let Some(parent_window) = top.window() else {
            return;
        };
        let monitor_geometry = parent_window
            .display()
            .monitor_at_window(&parent_window)
            .map(|monitor| monitor.geometry())
            .unwrap_or_else(|| {
                let (_, _, fallback_width, fallback_height) = parent_window.geometry();
                gdk::Rectangle::new(0, 0, fallback_width, fallback_height)
            });
        let _ = monitor_geometry;
        // Do not fullscreen this transient surface independently.  On
        // Hyprland that can unfullscreen the Tauri parent.  The parent owns
        // the real fullscreen state; this surface simply follows its size.
        self.window.resize(width.max(2), height.max(2));
    }
}

struct NativeControlsOverlaySurface {
    window: gtk::Window,
    root: gtk::Overlay,
    title: gtk::Label,
}

impl NativeControlsOverlaySurface {
    fn new(top: &gtk::Window, app: &AppHandle) -> Option<Self> {
        let window = gtk::Window::new(gtk::WindowType::Popup);
        window.set_decorated(false);
        window.set_resizable(true);
        window.set_skip_taskbar_hint(true);
        window.set_skip_pager_hint(true);
        window.set_keep_above(true);
        window.set_app_paintable(true);
        window.set_transient_for(Some(top));
        window.set_attached_to(Some(top));
        window.set_position(gtk::WindowPosition::None);
        window.set_accept_focus(true);
        if let Some(screen) = gtk::prelude::GtkWindowExt::screen(top) {
            window.set_screen(&screen);
        }
        if let Some(visual) = top.visual() {
            window.set_visual(Some(&visual));
        }
        window.set_default_size(top.allocated_width().max(2), top.allocated_height().max(2));

        let overlay = gtk::Overlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);
        overlay.set_size_request(top.allocated_width().max(2), top.allocated_height().max(2));

        let header = gtk::Label::new(Some("Fluxa  •  Native Wayland overlay"));
        header.set_halign(gtk::Align::Start);
        header.set_valign(gtk::Align::Start);
        header.set_margin_top(24);
        header.set_margin_start(28);
        header.set_margin_end(28);
        header.set_widget_name("fluxa-native-overlay-header");
        overlay.add_overlay(&header);

        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        controls.set_halign(gtk::Align::Start);
        controls.set_valign(gtk::Align::End);
        controls.set_margin_start(28);
        controls.set_margin_end(28);
        controls.set_margin_bottom(28);
        controls.set_widget_name("fluxa-native-overlay-controls");
        for (label, command) in [
            ("▶ / ❚❚", "cycle pause"),
            ("−10", "seek -10 relative"),
            ("+10", "seek 10 relative"),
        ] {
            let button = gtk::Button::with_label(label);
            button.set_widget_name("fluxa-native-overlay-button");
            let app_for_button = app.clone();
            button.connect_clicked(move |_| {
                if let Err(error) = crate::player_surface_events::engine_command(
                    &app_for_button,
                    command.to_string(),
                ) {
                    log::warn!("experimental-native: native overlay command failed: {error}");
                }
            });
            controls.pack_start(&button, false, false, 0);
        }
        let close_button = gtk::Button::with_label("×");
        close_button.set_widget_name("fluxa-native-overlay-button");
        let app_for_close = app.clone();
        close_button.connect_clicked(move |_| {
            let _ = app_for_close.emit("native-player-close-requested", ());
        });
        controls.pack_start(&close_button, false, false, 0);
        overlay.add_overlay(&controls);
        window.add(&overlay);

        let provider = gtk::CssProvider::new();
        if let Err(error) = provider.load_from_data(
            b"#fluxa-native-overlay-header, #fluxa-native-overlay-controls { background: transparent; color: white; } #fluxa-native-overlay-header { font-size: 16px; font-weight: 600; } #fluxa-native-overlay-button { color: white; background: rgba(0,0,0,0.72); border: 1px solid rgba(255,255,255,0.25); border-radius: 8px; padding: 8px 14px; }",
        ) {
            log::warn!("experimental-native: native overlay CSS failed: {error}");
        }
        if let Some(screen) = gtk::prelude::GtkWindowExt::screen(top) {
            gtk::StyleContext::add_provider_for_screen(
                &screen,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        window.hide();
        Some(Self {
            window,
            root: overlay,
            title: header,
        })
    }

    fn show(&self) {
        self.window.show_all();
        self.window.present();
        if let Some(gdk_window) = self.window.window() {
            unsafe {
                let transparent = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
                gdk::ffi::gdk_window_set_background_rgba(
                    gdk_window.to_glib_none().0,
                    transparent.to_glib_none().0,
                );
            }
        }
    }

    fn hide(&self) {
        self.window.hide();
    }

    fn set_title(&self, title: &str, episode: Option<&str>) {
        let text = match episode.filter(|episode| !episode.is_empty()) {
            Some(episode) => format!("{title}  •  {episode}"),
            None => title.to_string(),
        };
        self.title.set_text(&text);
    }

    fn sync_geometry(&mut self, top: &gtk::Window, width: i32, height: i32) {
        let Some(parent_window) = top.window() else {
            return;
        };
        let monitor_geometry = parent_window
            .display()
            .monitor_at_window(&parent_window)
            .map(|monitor| monitor.geometry())
            .unwrap_or_else(|| {
                let (_, _, fallback_width, fallback_height) = parent_window.geometry();
                gdk::Rectangle::new(0, 0, fallback_width, fallback_height)
            });
        let _ = monitor_geometry;
        // Keep only the Tauri parent in xdg_toplevel fullscreen.  The popup
        // remains anchored to that fullscreen surface and follows its size.
        self.root.set_size_request(width.max(2), height.max(2));
        self.window.resize(width.max(2), height.max(2));
    }
}

pub struct NativeVideoHost {
    parent: gdk::Window,
    window: HostWindow,
    surface: crate::linux_vulkan::NativeSurface,
    native_overlay: Option<crate::linux_wayland_native_overlay::NativeOverlaySurface>,
    video_above_parent: bool,
}

impl NativeVideoHost {
    pub fn attach(
        parent: &gtk::Window,
        width: i32,
        height: i32,
        stacking_surface: Option<*mut std::ffi::c_void>,
        video_above_parent: bool,
    ) -> Result<Self, String> {
        let parent_window = parent
            .window()
            .ok_or("native video host: parent window is not realized")?;
        let display = parent_window.display();
        if parent_window
            .clone()
            .downcast::<gdkwayland::WaylandWindow>()
            .is_ok()
        {
            let wayland_display: gdkwayland::WaylandDisplay = display
                .clone()
                .downcast()
                .map_err(|_| "native video host: Wayland display is unavailable".to_string())?;
            let wl_display: *mut std::ffi::c_void = unsafe {
                gdkwayland::ffi::gdk_wayland_display_get_wl_display(
                    wayland_display.to_glib_none().0,
                ) as *mut _
            };
            let wl_compositor: *mut std::ffi::c_void = unsafe {
                gdkwayland::ffi::gdk_wayland_display_get_wl_compositor(
                    wayland_display.to_glib_none().0,
                ) as *mut _
            };
            let parent_surface: *mut std::ffi::c_void = unsafe {
                gdkwayland::ffi::gdk_wayland_window_get_wl_surface(
                    parent_window
                        .clone()
                        .downcast::<gdkwayland::WaylandWindow>()
                        .map_err(|_| {
                            "native video host: Wayland parent surface is unavailable".to_string()
                        })?
                        .to_glib_none()
                        .0,
                )
            } as *mut _;
            if wl_display.is_null() || wl_compositor.is_null() || parent_surface.is_null() {
                return Err(
                    "native video host: Wayland compositor/surface is unavailable".to_string(),
                );
            }
            let subsurface = crate::linux_wayland_subsurface::VideoSubsurface::new(
                wl_display,
                wl_compositor,
                parent_surface,
                stacking_surface.filter(|surface| !surface.is_null() && *surface != parent_surface),
                video_above_parent,
            )?;
            let surface_ptr = subsurface.wl_surface();
            let native_overlay = if std::env::var_os("FLUXA_NATIVE_NATIVE_OVERLAY").is_some() {
                let (overlay_width, overlay_height) = parent_window
                    .display()
                    .monitor_at_window(&parent_window)
                    .map(|monitor| {
                        let geometry = monitor.geometry();
                        (geometry.width().max(width), geometry.height().max(height))
                    })
                    .unwrap_or((width, height));
                Some(crate::linux_wayland_native_overlay::NativeOverlaySurface::new(
                    wl_display,
                    wl_compositor,
                    parent_surface,
                    surface_ptr,
                    overlay_width,
                    overlay_height,
                )?)
            } else {
                None
            };
            return Ok(Self {
                parent: parent_window.clone(),
                window: HostWindow::Wayland(subsurface),
                surface: crate::linux_vulkan::NativeSurface::Wayland {
                    display: wl_display,
                    surface: surface_ptr,
                },
                native_overlay,
                video_above_parent,
            });
        }
        let x11_display: gdkx11::X11Display = display
            .downcast()
            .map_err(|_| "native video host: X11 display is required".to_string())?;
        // The Tauri window uses an ARGB visual for the transparent HTML
        // overlay. libmpv's X11 Vulkan swapchain needs a normal RGB child
        // visual, so do not inherit the transparent parent visual here.
        let rgb_visual = parent_window.screen().system_visual();
        let attrs = gdk::WindowAttr {
            window_type: gdk::WindowType::Child,
            wclass: gdk::WindowWindowClass::InputOutput,
            x: Some(0),
            y: Some(0),
            width: width.max(2),
            height: height.max(2),
            visual: rgb_visual,
            ..Default::default()
        };
        let window = gdk::Window::new(Some(&parent_window), &attrs);
        let x11_window: gdkx11::X11Window = window
            .clone()
            .downcast()
            .map_err(|_| "native video host: child is not X11".to_string())?;
        let xid = unsafe { gdkx11::ffi::gdk_x11_window_get_xid(x11_window.to_glib_none().0) };
        let xdisplay =
            unsafe { gdkx11::ffi::gdk_x11_display_get_xdisplay(x11_display.to_glib_none().0) };
        if xdisplay.is_null() {
            return Err("native video host: X11 display is unavailable".to_string());
        }
        window.show_unraised();
        window.lower();
        log::info!(
            "experimental-native: attaching X11 video child parent={}x{} xid={}",
            width,
            height,
            xid
        );
        Ok(Self {
            parent: parent_window,
            window: HostWindow::X11(window),
            surface: crate::linux_vulkan::NativeSurface::Xlib {
                display: xdisplay as *mut _,
                window: xid as u64,
            },
            native_overlay: None,
            video_above_parent: false,
        })
    }

    fn surface(&self) -> crate::linux_vulkan::NativeSurface {
        self.surface
    }

    fn x11_window_id(&self) -> Option<u64> {
        let HostWindow::X11(window) = &self.window else {
            return None;
        };
        let x11_window: gdkx11::X11Window = window.clone().downcast().ok()?;
        Some(unsafe { gdkx11::ffi::gdk_x11_window_get_xid(x11_window.to_glib_none().0) as u64 })
    }

    fn set_geometry(&self, x: i32, y: i32, width: i32, height: i32, above_webview: bool) {
        match &self.window {
            HostWindow::X11(window) => {
                window.move_resize(x, y, width.max(2), height.max(2));
                if above_webview {
                    window.raise();
                } else {
                    window.lower();
                }
            }
            HostWindow::Wayland(window) => {
                let _ = (width, height);
                window.set_position(x, y);
                window.set_above(above_webview);
                if let Some(native_overlay) = self.native_overlay.as_ref() {
                    native_overlay.resize(width, height);
                }
                // GTK/WebKit may recommit the toplevel with an opaque region
                // while entering fullscreen.  Keep the parent surface
                // transparent so the video subsurface remains visible below
                // the HTML controls.
                unsafe {
                    let rgba = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
                    gdk::ffi::gdk_window_set_background_rgba(
                        self.parent.to_glib_none().0,
                        rgba.to_glib_none().0,
                    );
                }
            }
        }
    }

    fn show(&self) {
        match &self.window {
            HostWindow::X11(window) => {
                window.show_unraised();
                window.lower();
            }
            HostWindow::Wayland(window) => {
                window.set_above(self.video_above_parent);
                if let Some(native_overlay) = self.native_overlay.as_ref() {
                    // The native surface is committed independently of the
                    // WebView parent and must be restored after a hide/show.
                    native_overlay.show();
                }
            }
        }
    }

    fn hide(&self) {
        match &self.window {
            HostWindow::X11(window) => window.hide(),
            HostWindow::Wayland(window) => {
                window.hide();
                if let Some(native_overlay) = self.native_overlay.as_ref() {
                    native_overlay.hide();
                }
            }
        }
    }
}

enum Command {
    Load(String, Option<u64>),
    Hide,
    Title(String, Option<String>),
}

pub struct ExperimentalNativePlayerSurface {
    sender: mpsc::Sender<Command>,
    app: AppHandle,
}

impl PlayerSurface for ExperimentalNativePlayerSurface {
    fn backend_name(&self) -> &'static str {
        "vulkan-native-experimental"
    }
    fn load(&self, url: String, start_at: Option<u64>, _: Option<u64>) -> Result<(), String> {
        self.sender
            .send(Command::Load(url, start_at))
            .map_err(|e| e.to_string())
    }
    fn hide(&self) {
        let _ = self.sender.send(Command::Hide);
    }
    fn shutdown(&self) -> Result<(), String> {
        self.hide();
        Ok(())
    }
    fn show_loading(&self, title: String, episode: Option<String>) {
        let _ = self.sender.send(Command::Title(title, episode));
    }
    fn set_title(&self, title: String, episode: Option<String>) {
        let _ = self.sender.send(Command::Title(title, episode));
    }
    fn set_artwork(&self, title: String, episode: Option<String>, _: Artwork, _: Artwork) {
        self.set_title(title, episode);
    }
    fn set_cursor_visible(&self, _: bool) {}
    fn command(&self, command: String) -> Result<(), String> {
        crate::player_surface_events::engine_command(&self.app, command)
    }
    fn command_args(&self, commands: Vec<Vec<String>>) -> Result<(), String> {
        crate::player_surface_events::engine_command_args(&self.app, commands)
    }
    fn status(&self) -> Result<PlayerStatus, String> {
        crate::player_surface_events::engine_status(&self.app)
    }
    fn track_options(&self, track_type: String) -> Result<Vec<PlayerTrackOption>, String> {
        crate::player_surface_events::engine_track_options(&self.app, track_type)
    }
    fn add_subtitle(
        &self,
        url: String,
        title: Option<String>,
        language: Option<String>,
    ) -> Result<(), String> {
        crate::player_surface_events::engine_add_subtitle(&self.app, url, title, language)
    }
}

fn load_renderer(
    app: &AppHandle,
    url: &str,
    start_at: Option<u64>,
    shared: &Arc<VulkanShared>,
    direct_x11: bool,
) -> Result<(), String> {
    log::debug!("experimental-native: load requested url={url} start_at={start_at:?}");
    let state = app.state::<DesktopState>();
    if direct_x11 {
        let mut client = state
            .player_mpv_client
            .try_lock()
            .map_err(|_| "player client busy".to_string())?;
        let result = crate::player::load_mpv_engine(
            client.as_mut().ok_or("player client unavailable")?,
            url,
            start_at,
            false,
        );
        log::info!("experimental-native: mpv X11 wid load result={result:?}");
        return result;
    }
    let mut render = state
        .player_render_state
        .try_lock()
        .map_err(|_| "player renderer busy".to_string())?;
    let mut client = state
        .player_mpv_client
        .try_lock()
        .map_err(|_| "player client busy".to_string())?;
    if client.is_none() {
        let (new_client, new_render) =
            MpvClientHandle::new_with_scripts(crate::player::mpv_script_paths(app))?;
        *render = Some(new_render);
        *client = Some(new_client);
    }
    if !shared.mpv_context_ready.load(Ordering::Acquire) {
        log::debug!("experimental-native: waiting for Vulkan/mpv context");
        return Err("vulkan render context pending".to_string());
    }
    let result = crate::player::load_mpv_engine(
        client.as_mut().ok_or("player client unavailable")?,
        url,
        start_at,
        shared.hdr.load(Ordering::Acquire),
    );
    log::warn!("experimental-native: mpv load result={result:?}");
    result
}

pub fn install(app: AppHandle) -> Result<ExperimentalNativePlayerSurface, String> {
    let window = app
        .get_webview_window("main")
        .ok_or("main webview window was not found")?;
    let (sender, receiver) = mpsc::channel();
    let (setup_tx, setup_rx) = mpsc::channel();
    let app_for_ui = app.clone();
    window
        .with_webview(move |platform_webview| {
            let webview = platform_webview.inner().upcast::<gtk::Widget>();
            if let Ok(webview) = webview.clone().downcast::<webkit2gtk::WebView>() {
                webview.set_background_color(&gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));
            }
            let top = webview
                .toplevel()
                .and_then(|widget| widget.downcast::<gtk::Window>().ok());
            let Some(top) = top else {
                let _ = setup_tx.send(Err("native video host: GTK window unavailable".to_string()));
                return;
            };
            top.set_app_paintable(true);
            webview.set_hexpand(true);
            webview.set_vexpand(true);
            let transparent = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
            let parent_visual_depth = top
                .window()
                .map(|window| unsafe {
                    let visual = gdk::ffi::gdk_window_get_visual(window.to_glib_none().0);
                    if visual.is_null() {
                        -1
                    } else {
                        gdk::ffi::gdk_visual_get_depth(visual)
                    }
                })
                .unwrap_or(-1);
            // WebKitGTK owns a child GDK window on Linux.  Setting the
            // WebKit page background alone does not clear that native
            // window's default black background, which can cover a valid
            // Vulkan subsurface after a fullscreen reconfigure.
            if let Some(window) = webview.window() {
                unsafe {
                    gdk::ffi::gdk_window_set_background_rgba(
                        window.to_glib_none().0,
                        transparent.to_glib_none().0,
                    );
                }
                let visual_depth = unsafe {
                    let visual = gdk::ffi::gdk_window_get_visual(window.to_glib_none().0);
                    if visual.is_null() {
                        -1
                    } else {
                        gdk::ffi::gdk_visual_get_depth(visual)
                    }
                };
                log::info!(
                    "experimental-native: GDK visual depths parent={} webview={}",
                    parent_visual_depth,
                    visual_depth
                );
            }
            let width = top.allocated_width().max(2);
            let height = top.allocated_height().max(2);
            let is_wayland = top
                .window()
                .and_then(|window| window.downcast::<gdkwayland::WaylandWindow>().ok())
                .is_some();
            let webview_overlay_enabled = is_wayland
                && std::env::var_os("FLUXA_NATIVE_WEBVIEW_OVERLAY").is_some();
            let native_wayland_overlay_enabled = is_wayland
                && std::env::var_os("FLUXA_NATIVE_NATIVE_OVERLAY").is_some();
            // Keep the old GTK experiment available for comparison, but do
            // not create it for the real native Wayland sibling path.
            let native_controls_overlay_enabled = is_wayland
                && std::env::var_os("FLUXA_NATIVE_GTK_OVERLAY").is_some();
            log::warn!(
                "experimental-native: overlay probe wayland={} react={} gtk_native={} wayland_native={}",
                is_wayland,
                webview_overlay_enabled,
                native_controls_overlay_enabled,
                native_wayland_overlay_enabled
            );
            let webview_overlay = if webview_overlay_enabled {
                WebViewOverlaySurface::new(&top, &webview)
            } else {
                None
            };
            let native_controls_overlay = if native_controls_overlay_enabled {
                NativeControlsOverlaySurface::new(&top, &app_for_ui)
            } else {
                None
            };
            let video_above_parent = webview_overlay.is_some()
                || native_controls_overlay.is_some()
                || native_wayland_overlay_enabled
                || std::env::var_os("FLUXA_NATIVE_AUTOTEST_VIDEO_ABOVE").is_some();
            // When React moves to its own surface, the video must be stacked
            // above the original Tauri parent. Otherwise the fullscreen
            // compositor can still hide it behind the parent buffer.
            let webview_surface = if webview_overlay.is_some() || native_wayland_overlay_enabled {
                None
            } else {
                webview.window().and_then(|window| {
                    let window_ptr = window.as_ptr() as *mut gdkwayland::ffi::GdkWaylandWindow;
                    Some(unsafe {
                        gdkwayland::ffi::gdk_wayland_window_get_wl_surface(window_ptr)
                    } as *mut std::ffi::c_void)
                })
            };
            let top_surface = top.window().and_then(|window| {
                let wayland_window = window.downcast::<gdkwayland::WaylandWindow>().ok()?;
                Some(unsafe {
                    gdkwayland::ffi::gdk_wayland_window_get_wl_surface(
                        wayland_window.to_glib_none().0,
                    )
                } as *mut std::ffi::c_void)
            });
            log::info!(
                "experimental-native: top surface={:?} webview surface={:?} same={} ",
                top_surface,
                webview_surface,
                top_surface == webview_surface
            );
            let host = match NativeVideoHost::attach(
                &top,
                width,
                height,
                webview_surface,
                video_above_parent,
            ) {
                Ok(host) => host,
                Err(error) => {
                    let _ = setup_tx.send(Err(error));
                    return;
                }
            };
            let x11_window_id = host.x11_window_id();
            let direct_x11 = x11_window_id.is_some();
            if let Some(window_id) = x11_window_id {
                let (client, _render) = match MpvClientHandle::new_with_scripts_x11_wid(
                    crate::player::mpv_script_paths(&app_for_ui),
                    window_id,
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        let _ = setup_tx.send(Err(format!(
                            "native video host: mpv X11 wid setup failed: {error}"
                        )));
                        return;
                    }
                };
                let state = app_for_ui.state::<DesktopState>();
                *state.player_render_state.lock().unwrap() = None;
                *state.player_mpv_client.lock().unwrap() = Some(client);
                log::info!(
                    "experimental-native: using libmpv gpu/vulkan with X11 wid={window_id}"
                );
            }
            let scale = if matches!(&host.window, HostWindow::Wayland(_)) {
                top.scale_factor().max(1)
            } else {
                1
            };
            let render_width = width.saturating_mul(scale);
            let render_height = height.saturating_mul(scale);
            let shared = Arc::new(VulkanShared {
                width: AtomicI32::new(render_width),
                height: AtomicI32::new(render_height),
                hdr: AtomicBool::new(false),
                mpv_context_ready: AtomicBool::new(direct_x11),
                load_in_progress: AtomicBool::new(false),
            });
            if !direct_x11 {
                let context = match crate::linux_vulkan::create_context(
                    host.surface(),
                    render_width,
                    render_height,
                ) {
                    Ok(context) => context,
                    Err(error) => {
                        let _ = setup_tx.send(Err(error));
                        return;
                    }
                };
                crate::linux_native_render::spawn_vulkan_render_thread(
                    app_for_ui.clone(),
                    context,
                    shared.clone(),
                );
            }
            let host = Rc::new(host);
            let webview_overlay = Rc::new(RefCell::new(webview_overlay));
            let native_controls_overlay = Rc::new(RefCell::new(native_controls_overlay));
            let pending = Rc::new(RefCell::new(None::<(String, Option<u64>)>));
            let mut last_geometry = None;
            let mut last_top_surface = top_surface;
            let _ = setup_tx.send(Ok(()));
            if std::env::var_os("FLUXA_NATIVE_AUTOTEST_GTK_FULLSCREEN").is_some() {
                let fullscreen_window = top.clone();
                glib::timeout_add_local_once(Duration::from_secs(4), move || {
                    log::warn!("experimental-native: autotest calling gtk_window_fullscreen");
                    fullscreen_window.fullscreen();
                });
            }
            glib::timeout_add_local(Duration::from_millis(16), move || {
                while let Ok(command) = receiver.try_recv() {
                    match command {
                        Command::Load(url, start_at) => {
                            log::debug!("experimental-native: Load command received");
                            pending.replace(Some((url, start_at)));
                            if let Some(overlay) = webview_overlay.borrow_mut().as_mut() {
                                overlay.attach();
                            }
                            if let Some(overlay) = native_controls_overlay.borrow().as_ref() {
                                overlay.show();
                            }
                            host.show();
                            let _ = app_for_ui.emit("native-player-show", ());
                        }
                        Command::Hide => {
                            pending.replace(None);
                            if let Some(overlay) = webview_overlay.borrow_mut().as_mut() {
                                overlay.detach();
                            }
                            if let Some(overlay) = native_controls_overlay.borrow().as_ref() {
                                overlay.hide();
                            }
                            host.hide();
                            let _ = app_for_ui.emit("native-player-hide", ());
                            if let Ok(client) = app_for_ui
                                .state::<DesktopState>()
                                .player_mpv_client
                                .try_lock()
                            {
                                if let Some(client) = client.as_ref() {
                                    let _ = client.command_args(&["stop"]);
                                }
                            }
                        }
                        Command::Title(title, episode) => {
                            if let Some(overlay) = native_controls_overlay.borrow().as_ref() {
                                overlay.set_title(&title, episode.as_deref());
                            }
                            let _ = app_for_ui.emit(
                                "native-player-title",
                                serde_json::json!({"title": title, "episodeTitle": episode}),
                            );
                        }
                    }
                }
                let (x, y, host_width, host_height) = if mini_mode() {
                    (24, 24, 448, 252)
                } else {
                    (
                        0,
                        0,
                        top.allocated_width().max(2),
                        top.allocated_height().max(2),
                    )
                };
                let scale = if matches!(&host.window, HostWindow::Wayland(_)) {
                    top.scale_factor().max(1)
                } else {
                    1
                };
                let width = host_width.saturating_mul(scale);
                let height = host_height.saturating_mul(scale);
                if let Some(current_top_surface) = top.window().and_then(|window| {
                    let wayland_window = window.downcast::<gdkwayland::WaylandWindow>().ok()?;
                    Some(unsafe {
                        gdkwayland::ffi::gdk_wayland_window_get_wl_surface(
                            wayland_window.to_glib_none().0,
                        )
                    } as *mut std::ffi::c_void)
                }) {
                    if Some(current_top_surface) != last_top_surface {
                        log::warn!(
                            "experimental-native: Wayland parent surface changed old={:?} new={:?} fullscreen_or_reconfigure=true",
                            last_top_surface,
                            current_top_surface
                        );
                        last_top_surface = Some(current_top_surface);
                    }
                }
                if last_geometry != Some((x, y, host_width, host_height, scale)) {
                    shared.width.store(width, Ordering::Release);
                    shared.height.store(height, Ordering::Release);
                    log::info!(
                        "experimental-native: geometry changed logical={}x{} render={}x{} offset={}x{} scale={} mini={}",
                        host_width,
                        host_height,
                        width,
                        height,
                        x,
                        y,
                        scale,
                        mini_mode()
                    );
                    let above_webview = std::env::var_os("FLUXA_NATIVE_AUTOTEST_VIDEO_ABOVE").is_some();
                    host.set_geometry(x, y, host_width, host_height, video_above_parent || above_webview);
                    last_geometry = Some((x, y, host_width, host_height, scale));
                }
                if let Some(overlay) = webview_overlay.borrow_mut().as_mut() {
                    overlay.sync_geometry(&top, host_width, host_height);
                }
                if let Some(overlay) = native_controls_overlay.borrow_mut().as_mut() {
                    overlay.sync_geometry(&top, host_width, host_height);
                }
                if let HostWindow::Wayland(window) = &host.window {
                    // Fullscreen transitions can make WebKit re-commit an
                    // opaque parent region after the geometry update. Keep
                    // the transparent overlay composited above Vulkan.
                    window.clear_parent_opaque_region();
                }
                crate::player_surface_events::check_player_events(&app_for_ui);
                let pending_load = { pending.borrow().clone() };
                if let Some((url, start_at)) = pending_load {
                    let result = load_renderer(
                        &app_for_ui,
                        &url,
                        start_at,
                        &shared,
                        direct_x11,
                    );
                    match result {
                        Ok(()) => {
                            pending.replace(None);
                        }
                        Err(error)
                            if error == "vulkan render context pending"
                                || error == "player renderer busy"
                                || error == "player client busy" => {}
                        Err(error) => {
                            let _ = app_for_ui.emit("native-player-error", error);
                            pending.replace(None);
                        }
                    }
                }
                glib::ControlFlow::Continue
            });
        })
        .map_err(|error| error.to_string())?;
    setup_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "native video host setup timed out".to_string())??;
    Ok(ExperimentalNativePlayerSurface { sender, app })
}
