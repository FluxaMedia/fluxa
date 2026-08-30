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

pub struct NativeVideoHost {
    window: HostWindow,
    surface: crate::linux_vulkan::NativeSurface,
}

impl NativeVideoHost {
    pub fn attach(parent: &gtk::Window, width: i32, height: i32) -> Result<Self, String> {
        let parent_window = parent
            .window()
            .ok_or("native video host: parent window is not realized")?;
        let display = parent_window.display();
        if let Ok(wayland_parent) = parent_window
            .clone()
            .downcast::<gdkwayland::WaylandWindow>()
        {
            let parent_surface = unsafe {
                gdkwayland::ffi::gdk_wayland_window_get_wl_surface(wayland_parent.to_glib_none().0)
            };
            let wayland_display: gdkwayland::WaylandDisplay = display
                .clone()
                .downcast()
                .map_err(|_| "native video host: Wayland display is unavailable".to_string())?;
            let wl_display = unsafe {
                gdkwayland::ffi::gdk_wayland_display_get_wl_display(
                    wayland_display.to_glib_none().0,
                )
            };
            let wl_compositor = unsafe {
                gdkwayland::ffi::gdk_wayland_display_get_wl_compositor(
                    wayland_display.to_glib_none().0,
                )
            };
            let host = crate::linux_wayland_subsurface::VideoSubsurface::new(
                wl_display as *mut _,
                wl_compositor as *mut _,
                parent_surface as *mut _,
            )?;
            let surface_ptr = host.wl_surface();
            return Ok(Self {
                window: HostWindow::Wayland(host),
                surface: crate::linux_vulkan::NativeSurface::Wayland {
                    display: wl_display as *mut _,
                    surface: surface_ptr,
                },
            });
        }
        let x11_display: gdkx11::X11Display = display
            .downcast()
            .map_err(|_| "native video host: X11 display is required".to_string())?;
        let attrs = gdk::WindowAttr {
            window_type: gdk::WindowType::Child,
            wclass: gdk::WindowWindowClass::InputOutput,
            x: Some(0),
            y: Some(0),
            width: width.max(2),
            height: height.max(2),
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
        Ok(Self {
            window: HostWindow::X11(window),
            surface: crate::linux_vulkan::NativeSurface::Xlib {
                display: xdisplay as *mut _,
                window: xid as u64,
            },
        })
    }

    fn surface(&self) -> crate::linux_vulkan::NativeSurface {
        self.surface
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
                window.set_position(x, y);
                window.set_above(above_webview);
            }
        }
    }

    fn show(&self) {
        if let HostWindow::X11(window) = &self.window {
            window.show_unraised();
            window.lower();
        }
    }

    fn hide(&self) {
        match &self.window {
            HostWindow::X11(window) => window.hide(),
            HostWindow::Wayland(window) => window.hide(),
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
) -> Result<(), String> {
    log::debug!("experimental-native: load requested url={url} start_at={start_at:?}");
    let state = app.state::<DesktopState>();
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
                webview.set_background_color(&gdk::RGBA::new(0.0, 0.0, 0.0, 1.0 / 255.0));
            }
            let top = webview
                .toplevel()
                .and_then(|widget| widget.downcast::<gtk::Window>().ok());
            let Some(top) = top else {
                let _ = setup_tx.send(Err("native video host: GTK window unavailable".to_string()));
                return;
            };
            webview.set_hexpand(true);
            webview.set_vexpand(true);
            let width = top.allocated_width().max(2);
            let height = top.allocated_height().max(2);
            let host = match NativeVideoHost::attach(&top, width, height) {
                Ok(host) => host,
                Err(error) => {
                    let _ = setup_tx.send(Err(error));
                    return;
                }
            };
            let scale = if matches!(&host.window, HostWindow::Wayland(_)) {
                top.scale_factor().max(1)
            } else {
                1
            };
            let render_width = width.saturating_mul(scale);
            let render_height = height.saturating_mul(scale);
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
            let shared = Arc::new(VulkanShared {
                width: AtomicI32::new(render_width),
                height: AtomicI32::new(render_height),
                hdr: AtomicBool::new(false),
                mpv_context_ready: AtomicBool::new(false),
                load_in_progress: AtomicBool::new(false),
            });
            crate::linux_native_render::spawn_vulkan_render_thread(
                app_for_ui.clone(),
                context,
                shared.clone(),
            );
            let host = Rc::new(host);
            let pending = Rc::new(RefCell::new(None::<(String, Option<u64>)>));
            let mut last_geometry = None;
            let _ = setup_tx.send(Ok(()));
            glib::timeout_add_local(Duration::from_millis(16), move || {
                while let Ok(command) = receiver.try_recv() {
                    match command {
                        Command::Load(url, start_at) => {
                            log::debug!("experimental-native: Load command received");
                            pending.replace(Some((url, start_at)));
                            host.show();
                            let _ = app_for_ui.emit("native-player-show", ());
                        }
                        Command::Hide => {
                            pending.replace(None);
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
                if last_geometry != Some((x, y, host_width, host_height, scale)) {
                    shared.width.store(width, Ordering::Release);
                    shared.height.store(height, Ordering::Release);
                    host.set_geometry(x, y, host_width, host_height, false);
                    last_geometry = Some((x, y, host_width, host_height, scale));
                }
                crate::player_surface_events::check_player_events(&app_for_ui);
                let pending_load = { pending.borrow().clone() };
                if let Some((url, start_at)) = pending_load {
                    let result = load_renderer(&app_for_ui, &url, start_at, &shared);
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
