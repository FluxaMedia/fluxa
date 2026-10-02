use std::sync::Arc;
use std::time::{Duration, Instant};

use fluxa_host::{FluxaHost, GamepadButton, Key, KeyInput, MouseButton, NativeSurface, egui};
use fluxa_renderer::platform::GraphicsBackend;
use gilrs::{Axis, Button, EventType, Gilrs};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
    window::{CursorIcon, Fullscreen, Icon, Window, WindowId},
};

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
#[cfg(not(target_os = "macos"))]
const BACKENDS: &[GraphicsBackend] = &[GraphicsBackend::Vulkan, GraphicsBackend::Gles];

const LINE_HEIGHT: f32 = 40.0;
const STICK_DEADZONE: f32 = 0.2;
const STICK_SCROLL: f32 = 40.0;
const REPEAT_DELAY: Duration = Duration::from_millis(400);
const REPEAT_RATE: Duration = Duration::from_millis(100);
const GAMEPAD_POLL: Duration = Duration::from_millis(50);

struct App {
    host: Option<FluxaHost>,
    window: Option<Arc<Window>>,
    gamepad: Option<Gilrs>,
    axis_dirs: [i8; 2],
    right_stick_y: f32,
    held: Option<(GamepadButton, Instant)>,
    cursor_hidden: bool,
    modifiers: ModifiersState,
    cursor: egui::CursorIcon,
    pointer: [f32; 2],
    #[cfg(target_os = "macos")]
    video_layer: Option<apple_video::VideoLayer>,
}

impl App {
    fn new() -> Self {
        Self {
            host: None,
            window: None,
            gamepad: Gilrs::new().ok(),
            axis_dirs: [0; 2],
            right_stick_y: 0.0,
            held: None,
            cursor_hidden: false,
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
                window.request_redraw();
            }
        }
    }

    fn key(&self, host: &FluxaHost, key: &WinitKey, text: Option<&str>, pressed: bool) {
        let egui_key = egui_key(key);
        if host.wants_keyboard() {
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

    fn after_frame(&mut self) {
        let (Some(host), Some(window)) = (self.host.as_ref(), self.window.as_ref()) else {
            return;
        };
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

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Fluxa")
                        .with_transparent(cfg!(target_os = "macos"))
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0))
                        .with_min_inner_size(winit::dpi::LogicalSize::new(640.0, 480.0)),
                )
                .expect("create Fluxa window"),
        );
        let data_dir = fluxa_effects::storage::data_dir().ok();
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
        host.set_video_backend(Box::new(mpv::MpvBackend::new()));
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
        let surface = unsafe {
            NativeSurface::new(BACKENDS, move || {
                wgpu::SurfaceTargetUnsafe::from_display_and_window(&*target, &*target)
                    .map_err(|error| error.to_string())
            })
        };
        host.surface_created(surface, size.width, size.height);
        window.request_redraw();
        self.host = Some(host);
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.poll_gamepad();
        let (Some(host), Some(window)) = (self.host.as_ref(), self.window.as_ref()) else {
            return;
        };
        let mut wake = host.next_redraw();
        let now = Instant::now();
        if wake.is_some_and(|at| at <= now) {
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
        let Some(host) = self.host.clone() else {
            return;
        };
        let scale = self.scale();
        if !matches!(event, WindowEvent::RedrawRequested)
            && let Some(window) = self.window.as_ref()
        {
            window.request_redraw();
        }
        match event {
            WindowEvent::CloseRequested => {
                host.surface_destroyed();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => host.surface_changed(size.width, size.height),
            #[cfg(target_os = "macos")]
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(layer) = self.video_layer.as_ref() {
                    layer.set_scale(scale_factor);
                }
            }
            WindowEvent::RedrawRequested => {
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
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && let Some(chord) = shortcut_chord(&event.logical_key, self.modifiers)
                    && host.shortcut(&chord, event.repeat)
                {
                    if let Some(window) = self.window.as_ref() {
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
    EventLoop::new()?.run_app(&mut App::new())
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
