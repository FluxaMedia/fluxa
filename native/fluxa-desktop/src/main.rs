use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use fluxa_host::{FluxaHost, GamepadButton, Key, KeyInput, MouseButton, NativeSurface, egui};
use fluxa_renderer::platform::GraphicsBackend;
use gilrs::{Axis, Button, EventType, Gilrs};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, Ime, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
    window::{CursorIcon, Fullscreen, Icon, Window, WindowId},
};

mod access;
#[cfg(target_os = "macos")]
mod apple_video;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod emoji;
#[cfg(target_os = "linux")]
mod mpris;
#[cfg(any(target_os = "linux", target_os = "windows"))]
mod mpv;
mod mpv_common;

#[cfg(target_os = "macos")]
const BACKENDS: &[GraphicsBackend] = &[GraphicsBackend::Vulkan, GraphicsBackend::Metal];
#[cfg(target_os = "windows")]
const BACKENDS: &[GraphicsBackend] = &[
    GraphicsBackend::Vulkan,
    GraphicsBackend::Dx12,
    GraphicsBackend::Gles,
];
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const BACKENDS: &[GraphicsBackend] = &[GraphicsBackend::Vulkan, GraphicsBackend::Gles];

#[cfg(target_os = "macos")]
struct MetalLayer(*mut std::ffi::c_void);

#[cfg(target_os = "macos")]
impl MetalLayer {
    fn get(&self) -> *mut std::ffi::c_void {
        self.0
    }
}

#[cfg(target_os = "macos")]
unsafe impl Send for MetalLayer {}
#[cfg(target_os = "macos")]
unsafe impl Sync for MetalLayer {}

#[cfg(target_os = "macos")]
fn metal_layer(window: &Window) -> Result<*mut std::ffi::c_void, String> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let RawWindowHandle::AppKit(handle) = window
        .window_handle()
        .map_err(|error| error.to_string())?
        .as_raw()
    else {
        return Err("window is not an AppKit window".to_owned());
    };
    let layer = unsafe { raw_window_metal::Layer::from_ns_view(handle.ns_view) };
    Ok(layer.into_raw().as_ptr())
}

const LINE_HEIGHT: f32 = 40.0;
const STICK_DEADZONE: f32 = 0.2;
const STICK_SCROLL: f32 = 40.0;
const REPEAT_DELAY: Duration = Duration::from_millis(400);
const REPEAT_RATE: Duration = Duration::from_millis(100);
const ACCESS_INTERVAL: Duration = Duration::from_millis(200);
const GAMEPAD_POLL: Duration = Duration::from_millis(50);

struct AccessEvent(accesskit_winit::Event);

impl From<accesskit_winit::Event> for AccessEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        Self(event)
    }
}

struct App {
    proxy: EventLoopProxy<AccessEvent>,
    access: Option<accesskit_winit::Adapter>,
    access_tree: Option<fluxa_host::AccessTree>,
    access_at: Instant,
    host: Option<FluxaHost>,
    window: Option<Arc<Window>>,
    redraw_asked: Arc<AtomicBool>,
    gamepad: Option<Gilrs>,
    axis_dirs: [i8; 2],
    right_stick_y: f32,
    held: Option<(GamepadButton, Instant)>,
    cursor_hidden: bool,
    ime_allowed: bool,
    clipboard: Option<arboard::Clipboard>,
    modifiers: ModifiersState,
    cursor: egui::CursorIcon,
    pointer: [f32; 2],
    #[cfg(target_os = "macos")]
    video_layer: Option<apple_video::VideoLayer>,
}

impl App {
    fn new(proxy: EventLoopProxy<AccessEvent>) -> Self {
        Self {
            proxy,
            access: None,
            access_tree: None,
            access_at: Instant::now(),
            host: None,
            window: None,
            redraw_asked: Arc::default(),
            gamepad: Gilrs::new().ok(),
            axis_dirs: [0; 2],
            right_stick_y: 0.0,
            held: None,
            cursor_hidden: false,
            ime_allowed: false,
            clipboard: arboard::Clipboard::new().ok(),
            modifiers: ModifiersState::empty(),
            cursor: egui::CursorIcon::Default,
            pointer: [0.0, 0.0],
            #[cfg(target_os = "macos")]
            video_layer: None,
        }
    }

    fn scale(&self) -> f32 {
        self.window
            .as_ref()
            .map_or(1.0, |window| window.scale_factor() as f32)
    }

    fn poll_gamepad(&mut self) {
        let (Some(gamepad), Some(host)) = (self.gamepad.as_mut(), self.host.as_ref()) else {
            return;
        };
        let now = Instant::now();
        let mut active = false;
        while let Some(event) = gamepad.next_event() {
            let pressed = match event.event {
                EventType::ButtonPressed(button, _) => gamepad_button(button),
                EventType::ButtonReleased(button, _) => {
                    if gamepad_button(button)
                        .is_some_and(|button| self.held.is_some_and(|(held, _)| held == button))
                    {
                        self.held = None;
                    }
                    None
                }
                EventType::AxisChanged(Axis::RightStickY, value, _) => {
                    self.right_stick_y = if value.abs() < STICK_DEADZONE {
                        0.0
                    } else {
                        value
                    };
                    None
                }
                EventType::AxisChanged(axis, value, _) => {
                    let slot = axis_slot(axis);
                    let pressed = axis_press(&mut self.axis_dirs, axis, value);
                    if let Some(slot) = slot
                        && self.axis_dirs[slot] == 0
                        && self.held.is_some_and(|(held, _)| axis_owns(slot, held))
                    {
                        self.held = None;
                    }
                    pressed
                }
                _ => None,
            };
            if let Some(button) = pressed {
                host.key_down(KeyInput::Gamepad(button));
                active = true;
                if is_direction(button) {
                    self.held = Some((button, now + REPEAT_DELAY));
                }
            }
        }
        if let Some((button, at)) = self.held
            && now >= at
        {
            host.key_down(KeyInput::Gamepad(button));
            self.held = Some((button, now + REPEAT_RATE));
            active = true;
        }
        if self.right_stick_y != 0.0 {
            host.gamepad_scroll(-self.right_stick_y * STICK_SCROLL);
            active = true;
        }
        if active {
            if let Some(window) = self.window.as_ref() {
                if !self.cursor_hidden {
                    window.set_cursor_visible(false);
                    self.cursor_hidden = true;
                }
                self.redraw_asked.store(true, Ordering::Relaxed);
                window.request_redraw();
            }
        }
    }

    fn edit_chord(&mut self, host: &FluxaHost, key: &WinitKey) -> bool {
        if !(self.modifiers.control_key() || self.modifiers.super_key()) {
            return false;
        }
        let WinitKey::Character(text) = key else {
            return false;
        };
        match text.to_lowercase().as_str() {
            "v" => {
                if let Some(text) = self
                    .clipboard
                    .as_mut()
                    .and_then(|clipboard| clipboard.get_text().ok())
                {
                    host.egui_paste(&text);
                }
            }
            "c" => host.egui_event(egui::Event::Copy),
            "x" => host.egui_event(egui::Event::Cut),
            _ => return false,
        }
        true
    }

    fn key(&mut self, host: &FluxaHost, key: &WinitKey, text: Option<&str>, pressed: bool) {
        let egui_key = egui_key(key);
        if host.wants_keyboard() {
            if pressed && self.edit_chord(host, key) {
                return;
            }
            if let Some(egui_key) = egui_key {
                host.egui_key(egui_key, pressed);
            }
            if pressed
                && let Some(text) = text.filter(|text| !text.chars().any(char::is_control))
                && !self.modifiers.control_key()
                && !self.modifiers.super_key()
            {
                host.egui_text(text);
            }
            return;
        }
        if !pressed {
            if let Some(egui_key) = egui_key {
                host.egui_key(egui_key, false);
            }
            return;
        }
        if let Some(input) = key_input(key, self.modifiers.shift_key()) {
            host.key_down(input);
        } else if let Some(text) = text.filter(|text| !text.chars().any(char::is_control))
            && !self.modifiers.control_key()
        {
            host.text_input(text);
        }
    }

    fn push_access(&mut self, force: bool) {
        let (Some(host), Some(window), Some(adapter)) = (
            self.host.as_ref(),
            self.window.as_ref(),
            self.access.as_mut(),
        ) else {
            return;
        };
        if !force && self.access_at.elapsed() < ACCESS_INTERVAL {
            return;
        }
        self.access_at = Instant::now();
        let scale = window.scale_factor() as f32;
        let last = &mut self.access_tree;
        let Some(tree) = host.accessibility_tree() else {
            return;
        };
        if !force && last.as_ref() == Some(&tree) {
            return;
        }
        adapter.update_if_active(|| {
            let update = access::tree_update(&tree, scale, "Fluxa");
            *last = Some(tree);
            update
        });
    }

    fn after_frame(&mut self) {
        self.push_access(false);
        let (Some(host), Some(window)) = (self.host.as_ref(), self.window.as_ref()) else {
            return;
        };
        host.set_window_fullscreen(window.fullscreen().is_some());
        if host.take_fullscreen_toggle() {
            let fullscreen = window
                .fullscreen()
                .is_none()
                .then_some(Fullscreen::Borderless(None));
            window.set_fullscreen(fullscreen);
        }
        if let Some(id) = host.take_app_icon() {
            let icon = fluxa_host::app_icon_rgba(&id, 256).and_then(|image| {
                Icon::from_rgba(image.to_vec(), image.width(), image.height()).ok()
            });
            window.set_window_icon(icon);
        }
        if let Some(text) = host.take_copied()
            && let Some(clipboard) = self.clipboard.as_mut()
        {
            let _ = clipboard.set_text(text);
        }
        let editing = host.text_input_focused() || host.wants_keyboard();
        if editing != self.ime_allowed {
            self.ime_allowed = editing;
            window.set_ime_allowed(editing);
        }
        if editing
            && let Some([x, y, width, height]) =
                host.caret_bounds().or_else(|| host.focused_bounds())
        {
            let scale = window.scale_factor();
            window.set_ime_cursor_area(
                winit::dpi::LogicalPosition::new(x as f64, y as f64).to_physical::<u32>(scale),
                winit::dpi::LogicalSize::new(width as f64, height as f64).to_physical::<u32>(scale),
            );
        }
        let cursor = host.cursor();
        if cursor != self.cursor {
            self.cursor = cursor;
            window.set_cursor(match cursor {
                egui::CursorIcon::PointingHand => CursorIcon::Pointer,
                egui::CursorIcon::Text => CursorIcon::Text,
                _ => CursorIcon::Default,
            });
            window.set_cursor_visible(cursor != egui::CursorIcon::None);
        }
    }
}

#[cfg(not(target_os = "macos"))]
struct SurfaceHandles {
    display: winit::raw_window_handle::RawDisplayHandle,
    window: winit::raw_window_handle::RawWindowHandle,
}

#[cfg(not(target_os = "macos"))]
unsafe impl Send for SurfaceHandles {}
#[cfg(not(target_os = "macos"))]
unsafe impl Sync for SurfaceHandles {}

impl ApplicationHandler<AccessEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let (width, height) = fluxa_host::preview_size().unwrap_or((1280.0, 800.0));
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Fluxa")
                        .with_visible(false)
                        .with_transparent(cfg!(target_os = "macos"))
                        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
                        .with_min_inner_size(winit::dpi::LogicalSize::new(
                            width.min(640.0),
                            height.min(480.0),
                        )),
                )
                .expect("create Fluxa window"),
        );
        self.access = Some(accesskit_winit::Adapter::with_event_loop_proxy(
            event_loop,
            &window,
            self.proxy.clone(),
        ));
        window.set_visible(true);
        let data_dir = fluxa_effects::storage::data_dir().ok();
        if let Some(path) = data_dir
            .as_ref()
            .map(|directory| directory.join("theme.css"))
            && path.exists()
        {
            match fluxa_theme::css::set_user_theme_from_path(&path) {
                Ok(report) => report
                    .rejected
                    .iter()
                    .for_each(|entry| eprintln!("[fluxa-desktop] theme.css: {entry}")),
                Err(error) => eprintln!("[fluxa-desktop] theme.css: {error:?}"),
            }
        }
        let host = FluxaHost::new(
            window.scale_factor() as f32,
            data_dir
                .as_ref()
                .map(|directory| directory.join("artwork-cache")),
        );
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            emoji::warm_up();
            fluxa_host::set_emoji_rasterizer(emoji::rasterize);
        }
        host.set_form_factor("desktop");
        host.set_platform(std::env::consts::OS);
        host.set_image_picker(Box::new(pick_image));
        let notified = window.clone();
        host.set_pre_present(Box::new(move || notified.pre_present_notify()));
        #[cfg(target_os = "linux")]
        mpris::start(host.clone());
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            let window = window.clone();
            let asked = self.redraw_asked.clone();
            host.set_video_backend(Box::new(mpv::MpvBackend::new(Arc::new(move || {
                asked.store(true, Ordering::Relaxed);
                window.request_redraw();
            }))));
        }
        #[cfg(target_os = "macos")]
        {
            self.video_layer = apple_video::VideoLayer::attach(&window);
            match self.video_layer.as_ref() {
                Some(layer) => host
                    .set_video_backend(Box::new(apple_video::AppleBackend::new(layer.pointer()))),
                None => eprintln!("[fluxa-desktop] could not attach the video layer"),
            }
        }
        match data_dir {
            Some(directory) => {
                if let Err(error) = host.start_session(directory) {
                    eprintln!("[fluxa-desktop] session failed to start: {error}");
                }
            }
            None => eprintln!("[fluxa-desktop] no data directory; running without a session"),
        }
        let size = window.inner_size();
        let target = window.clone();
        #[cfg(target_os = "macos")]
        let surface = {
            let layer = metal_layer(&window).map(MetalLayer);
            unsafe {
                NativeSurface::new(BACKENDS, move || {
                    let _keep_alive = &target;
                    match &layer {
                        Ok(layer) => Ok(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer.get())),
                        Err(error) => Err(error.clone()),
                    }
                })
            }
        };
        #[cfg(not(target_os = "macos"))]
        let surface = {
            use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
            let handles = match (window.display_handle(), window.window_handle()) {
                (Ok(display), Ok(handle)) => Ok(SurfaceHandles {
                    display: display.as_raw(),
                    window: handle.as_raw(),
                }),
                (display, handle) => Err(format!(
                    "window handles unavailable: {:?} {:?}",
                    display.err(),
                    handle.err()
                )),
            };
            unsafe {
                NativeSurface::new(BACKENDS, move || {
                    let _keep_alive = &target;
                    match &handles {
                        Ok(handles) => Ok(wgpu::SurfaceTargetUnsafe::RawHandle {
                            raw_display_handle: Some(handles.display),
                            raw_window_handle: handles.window,
                        }),
                        Err(error) => Err(error.clone()),
                    }
                })
            }
        };
        host.surface_created(surface, size.width, size.height);
        self.redraw_asked.store(true, Ordering::Relaxed);
        window.request_redraw();
        self.host = Some(host);
        self.window = Some(window);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AccessEvent) {
        let Some(host) = self.host.as_ref() else {
            return;
        };
        match event.0.window_event {
            accesskit_winit::WindowEvent::InitialTreeRequested => {
                self.access_tree = None;
                self.push_access(true);
            }
            accesskit_winit::WindowEvent::ActionRequested(request) => {
                access::handle(host, request);
                if let Some(window) = self.window.as_ref() {
                    self.redraw_asked.store(true, Ordering::Relaxed);
                    window.request_redraw();
                }
            }
            accesskit_winit::WindowEvent::AccessibilityDeactivated => self.access_tree = None,
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.poll_gamepad();
        let (Some(host), Some(window)) = (self.host.as_ref(), self.window.as_ref()) else {
            return;
        };
        if host.take_exit_request() {
            event_loop.exit();
            return;
        }
        if host.take_minimize_request() {
            window.set_minimized(true);
        }
        let mut wake = host.next_redraw();
        let now = Instant::now();
        if host.waiting_to_exit() {
            let check = now + Duration::from_secs(1);
            wake = Some(wake.map_or(check, |at| at.min(check)));
        }
        if wake.is_some_and(|at| at <= now) {
            self.redraw_asked.store(true, Ordering::Relaxed);
            window.request_redraw();
            wake = None;
        }
        if self.gamepad.is_some() {
            let poll = now + GAMEPAD_POLL;
            wake = Some(wake.map_or(poll, |at| at.min(poll)));
        }
        event_loop.set_control_flow(wake.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(adapter), Some(window)) = (self.access.as_mut(), self.window.as_ref()) {
            adapter.process_event(window, &event);
        }
        let Some(host) = self.host.clone() else {
            return;
        };
        let scale = self.scale();
        if !matches!(event, WindowEvent::RedrawRequested) {
            host.invalidate();
        }
        match event {
            WindowEvent::CloseRequested => {
                if host.request_close() {
                    host.surface_destroyed();
                    event_loop.exit();
                }
            }
            WindowEvent::Focused(focused) => {
                if focused {
                    host.window_restored();
                }
                host.set_window_focused(focused);
            }
            WindowEvent::Resized(size) => host.surface_changed(size.width, size.height),
            #[cfg(target_os = "macos")]
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(layer) = self.video_layer.as_ref() {
                    layer.set_scale(scale_factor);
                }
            }
            WindowEvent::RedrawRequested => {
                if !self.redraw_asked.swap(false, Ordering::Relaxed) {
                    host.expose();
                }
                host.render();
                self.after_frame();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
                host.set_modifiers(egui::Modifiers {
                    alt: self.modifiers.alt_key(),
                    ctrl: self.modifiers.control_key(),
                    shift: self.modifiers.shift_key(),
                    mac_cmd: cfg!(target_os = "macos") && self.modifiers.super_key(),
                    command: if cfg!(target_os = "macos") {
                        self.modifiers.super_key()
                    } else {
                        self.modifiers.control_key()
                    },
                });
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = [position.x as f32 / scale, position.y as f32 / scale];
                host.mouse_moved(self.pointer[0], self.pointer[1]);
                if self.cursor_hidden {
                    if let Some(window) = self.window.as_ref() {
                        window.set_cursor_visible(true);
                    }
                    self.cursor_hidden = false;
                }
            }
            WindowEvent::CursorLeft { .. } => host.mouse_left(),
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    winit::event::MouseButton::Left => MouseButton::Primary,
                    winit::event::MouseButton::Right => MouseButton::Secondary,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    winit::event::MouseButton::Back if state == ElementState::Pressed => {
                        host.key_down(KeyInput::Key(Key::Back));
                        return;
                    }
                    _ => return,
                };
                host.mouse_button(
                    button,
                    state == ElementState::Pressed,
                    self.pointer[0],
                    self.pointer[1],
                );
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x * LINE_HEIGHT, y * LINE_HEIGHT),
                    MouseScrollDelta::PixelDelta(delta) => {
                        (delta.x as f32 / scale, delta.y as f32 / scale)
                    }
                };
                host.wheel(-x, -y);
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                if host.wants_keyboard() {
                    host.egui_text(&text);
                } else {
                    host.text_input(&text);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && let Some(chord) = shortcut_chord(&event.logical_key, self.modifiers)
                    && host.shortcut(&chord, event.repeat)
                {
                    if let Some(window) = self.window.as_ref() {
                        self.redraw_asked.store(true, Ordering::Relaxed);
                        window.request_redraw();
                    }
                    return;
                }
                if host.shortcut_recording() {
                    return;
                }
                if event.state == ElementState::Pressed
                    && !event.repeat
                    && let Some(command) = media_command(&event.logical_key)
                    && host.is_playing()
                {
                    host.media_command(command, 0.0);
                    return;
                }
                self.key(
                    &host,
                    &event.logical_key,
                    event.text.as_deref(),
                    event.state == ElementState::Pressed,
                );
            }
            _ => {}
        }
    }
}

fn key_input(key: &WinitKey, shift: bool) -> Option<KeyInput> {
    let WinitKey::Named(named) = key else {
        return None;
    };
    Some(KeyInput::Key(match named {
        NamedKey::ArrowUp => Key::Up,
        NamedKey::ArrowDown => Key::Down,
        NamedKey::ArrowLeft => Key::Left,
        NamedKey::ArrowRight => Key::Right,
        NamedKey::Enter => Key::Enter,
        NamedKey::Escape => Key::Escape,
        NamedKey::ContextMenu => return Some(KeyInput::Gamepad(GamepadButton::West)),
        NamedKey::BrowserBack | NamedKey::GoBack => Key::Back,
        NamedKey::Tab if shift => Key::ShiftTab,
        NamedKey::Tab => Key::Tab,
        NamedKey::Backspace => return Some(KeyInput::Backspace),
        NamedKey::Space | NamedKey::MediaPlayPause => {
            return Some(KeyInput::Gamepad(GamepadButton::Start));
        }
        _ => return None,
    }))
}

fn shortcut_chord(key: &WinitKey, modifiers: ModifiersState) -> Option<String> {
    let name = match key {
        WinitKey::Named(named) => match named {
            NamedKey::ArrowUp => "up".to_owned(),
            NamedKey::ArrowDown => "down".to_owned(),
            NamedKey::ArrowLeft => "left".to_owned(),
            NamedKey::ArrowRight => "right".to_owned(),
            NamedKey::Space => "space".to_owned(),
            NamedKey::Enter => "enter".to_owned(),
            NamedKey::Escape => "escape".to_owned(),
            NamedKey::Tab => "tab".to_owned(),
            NamedKey::Backspace => "backspace".to_owned(),
            NamedKey::Delete => "delete".to_owned(),
            NamedKey::Insert => "insert".to_owned(),
            NamedKey::Home => "home".to_owned(),
            NamedKey::End => "end".to_owned(),
            NamedKey::PageUp => "pageup".to_owned(),
            NamedKey::PageDown => "pagedown".to_owned(),
            other => {
                let name = format!("{other:?}");
                let number = name.strip_prefix('F')?;
                number.parse::<u8>().ok()?;
                name.to_lowercase()
            }
        },
        WinitKey::Character(text) => {
            let mut chars = text.chars();
            let first = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            match first {
                ',' => "comma".to_owned(),
                '+' => "plus".to_owned(),
                '-' => "minus".to_owned(),
                ' ' => "space".to_owned(),
                other => other.to_lowercase().collect(),
            }
        }
        _ => return None,
    };
    let literal =
        matches!(key, WinitKey::Character(text) if text.chars().all(|c| !c.is_alphabetic()));
    let mut parts = Vec::new();
    if modifiers.control_key() {
        parts.push("ctrl");
    }
    if modifiers.alt_key() {
        parts.push("alt");
    }
    if modifiers.shift_key() && !literal {
        parts.push("shift");
    }
    if modifiers.super_key() {
        parts.push("meta");
    }
    parts.push(&name);
    Some(parts.join("+"))
}

fn media_command(key: &WinitKey) -> Option<&'static str> {
    let WinitKey::Named(named) = key else {
        return None;
    };
    Some(match named {
        NamedKey::MediaPlayPause => "toggle",
        NamedKey::MediaPlay => "play",
        NamedKey::MediaPause => "pause",
        NamedKey::MediaStop => "stop",
        NamedKey::MediaTrackNext => "next",
        NamedKey::MediaTrackPrevious => "previous",
        NamedKey::MediaFastForward => "fastForward",
        NamedKey::MediaRewind => "rewind",
        _ => return None,
    })
}

fn egui_key(key: &WinitKey) -> Option<egui::Key> {
    match key {
        WinitKey::Named(named) => match named {
            NamedKey::ArrowUp => Some(egui::Key::ArrowUp),
            NamedKey::ArrowDown => Some(egui::Key::ArrowDown),
            NamedKey::ArrowLeft => Some(egui::Key::ArrowLeft),
            NamedKey::ArrowRight => Some(egui::Key::ArrowRight),
            NamedKey::Enter => Some(egui::Key::Enter),
            NamedKey::Escape => Some(egui::Key::Escape),
            NamedKey::Tab => Some(egui::Key::Tab),
            NamedKey::Backspace => Some(egui::Key::Backspace),
            NamedKey::Delete => Some(egui::Key::Delete),
            NamedKey::Home => Some(egui::Key::Home),
            NamedKey::End => Some(egui::Key::End),
            NamedKey::Space => Some(egui::Key::Space),
            _ => None,
        },
        WinitKey::Character(text) => egui::Key::from_name(text),
        _ => None,
    }
}

fn axis_slot(axis: Axis) -> Option<usize> {
    match axis {
        Axis::DPadX | Axis::LeftStickX => Some(0),
        Axis::DPadY | Axis::LeftStickY => Some(1),
        _ => None,
    }
}

fn axis_owns(slot: usize, button: GamepadButton) -> bool {
    match slot {
        0 => matches!(button, GamepadButton::DPadLeft | GamepadButton::DPadRight),
        _ => matches!(button, GamepadButton::DPadUp | GamepadButton::DPadDown),
    }
}

fn is_direction(button: GamepadButton) -> bool {
    matches!(
        button,
        GamepadButton::DPadUp
            | GamepadButton::DPadDown
            | GamepadButton::DPadLeft
            | GamepadButton::DPadRight
    )
}

fn axis_press(dirs: &mut [i8; 2], axis: Axis, value: f32) -> Option<GamepadButton> {
    let slot = axis_slot(axis)?;
    let dir = if value > 0.6 {
        1
    } else if value < -0.6 {
        -1
    } else {
        0
    };
    if dir == dirs[slot] {
        return None;
    }
    dirs[slot] = dir;
    Some(match (slot, dir) {
        (0, 1) => GamepadButton::DPadRight,
        (0, -1) => GamepadButton::DPadLeft,
        (1, 1) => GamepadButton::DPadUp,
        (1, -1) => GamepadButton::DPadDown,
        _ => return None,
    })
}

fn gamepad_button(button: Button) -> Option<GamepadButton> {
    Some(match button {
        Button::South => GamepadButton::South,
        Button::East => GamepadButton::East,
        Button::North => GamepadButton::North,
        Button::West => GamepadButton::West,
        Button::DPadUp => GamepadButton::DPadUp,
        Button::DPadDown => GamepadButton::DPadDown,
        Button::DPadLeft => GamepadButton::DPadLeft,
        Button::DPadRight => GamepadButton::DPadRight,
        Button::Start => GamepadButton::Start,
        Button::Select => GamepadButton::Select,
        _ => return None,
    })
}

fn main() -> Result<(), winit::error::EventLoopError> {
    // SAFETY: set before the event loop or any mpv handle exists.
    unsafe { std::env::set_var("MPV_LIBMPV_RENDER_BACKEND", "gpu-next") };
    fluxa_mpv::set_error_reporter(|error| {
        log::error!("native mpv: {}", error.message);
        sentry::with_scope(
            |scope| {
                scope.set_tag("mpv.error_code", error.error_code);
                if let Some(url) = &error.url {
                    scope.set_extra("mpv.url", url.clone().into());
                }
                if !error.log_tail.is_empty() {
                    scope.set_extra("mpv.log_tail", error.log_tail.clone().into());
                }
            },
            || sentry::capture_message(&error.message, sentry::Level::Error),
        );
    });
    let event_loop = EventLoop::<AccessEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    event_loop.run_app(&mut App::new(proxy))
}

fn pick_image() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    let output = std::process::Command::new("osascript")
        .args([
            "-e",
            "POSIX path of (choose file of type {\"public.image\"})",
        ])
        .output();
    #[cfg(not(target_os = "macos"))]
    let output = std::process::Command::new("zenity")
        .args([
            "--file-selection",
            "--file-filter=Images | *.png *.jpg *.jpeg *.webp *.gif",
        ])
        .output()
        .or_else(|_| {
            std::process::Command::new("kdialog")
                .args(["--getopenfilename", ".", "*.png *.jpg *.jpeg *.webp *.gif"])
                .output()
        });
    let output = output.ok().filter(|output| output.status.success())?;
    let path = String::from_utf8(output.stdout).ok()?;
    let path = path.trim();
    (!path.is_empty()).then(|| path.into())
}
