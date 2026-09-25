use std::sync::Arc;
use std::time::{Duration, Instant};

use fluxa_host::{FluxaHost, GamepadButton, Key, KeyInput, MouseButton, NativeSurface, egui};
use fluxa_renderer::platform::GraphicsBackend;
use gilrs::{Button, EventType, Gilrs};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
    window::{CursorIcon, Fullscreen, Window, WindowId},
};

#[cfg(target_os = "linux")]
mod mpv;

#[cfg(target_os = "macos")]
const BACKENDS: &[GraphicsBackend] = &[GraphicsBackend::Metal];
#[cfg(not(target_os = "macos"))]
const BACKENDS: &[GraphicsBackend] = &[GraphicsBackend::Vulkan, GraphicsBackend::Gles];

const LINE_HEIGHT: f32 = 40.0;
const GAMEPAD_POLL: Duration = Duration::from_millis(50);

struct App {
    host: Option<FluxaHost>,
    window: Option<Arc<Window>>,
    gamepad: Option<Gilrs>,
    modifiers: ModifiersState,
    cursor: egui::CursorIcon,
    pointer: [f32; 2],
}

impl App {
    fn new() -> Self {
        Self {
            host: None,
            window: None,
            gamepad: Gilrs::new().ok(),
            modifiers: ModifiersState::empty(),
            cursor: egui::CursorIcon::Default,
            pointer: [0.0, 0.0],
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
        while let Some(event) = gamepad.next_event() {
            if let EventType::ButtonPressed(button, _) = event.event
                && let Some(button) = gamepad_button(button)
            {
                host.key_down(KeyInput::Gamepad(button));
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
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
            let fullscreen = window.fullscreen().is_none().then_some(Fullscreen::Borderless(None));
            window.set_fullscreen(fullscreen);
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
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0))
                        .with_min_inner_size(winit::dpi::LogicalSize::new(640.0, 480.0)),
                )
                .expect("create Fluxa window"),
        );
        let data_dir = fluxa_effects::storage::data_dir().ok();
        let host = FluxaHost::new(
            window.scale_factor() as f32,
            data_dir.as_ref().map(|directory| directory.join("artwork-cache")),
        );
        host.set_form_factor("desktop");
        #[cfg(target_os = "linux")]
        host.set_video_backend(Box::new(mpv::MpvBackend::new()));
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
        let now = Instant::now();
        let mut wake = host.next_redraw();
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
                if event.logical_key == WinitKey::Named(NamedKey::F11)
                    && event.state == ElementState::Pressed
                    && let Some(window) = self.window.as_ref()
                {
                    let fullscreen =
                        window.fullscreen().is_none().then_some(Fullscreen::Borderless(None));
                    window.set_fullscreen(fullscreen);
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
