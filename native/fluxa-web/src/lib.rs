#![cfg(target_arch = "wasm32")]

use std::{cell::RefCell, ffi::c_void, path::PathBuf, ptr::NonNull, rc::Rc};

use fluxa_host::{FluxaHost, GamepadButton, Key, KeyInput, NativeSurface, PointerPhase};
use fluxa_renderer::platform::GraphicsBackend;
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WebCanvasWindowHandle, WebDisplayHandle,
};
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use web_sys::{HtmlCanvasElement, KeyboardEvent, PointerEvent, WheelEvent};

struct CanvasHandle(NonNull<c_void>);

unsafe impl Send for CanvasHandle {}
unsafe impl Sync for CanvasHandle {}

fn log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

fn key_input(event: &KeyboardEvent) -> Option<KeyInput> {
    let key = match event.key_code() {
        38 => Key::Up,
        40 => Key::Down,
        37 => Key::Left,
        39 => Key::Right,
        13 => Key::Enter,
        27 => Key::Escape,
        461 | 10009 => Key::Back,
        9 if event.shift_key() => Key::ShiftTab,
        9 => Key::Tab,
        8 => return Some(KeyInput::Backspace),
        415 | 19 => return Some(KeyInput::Gamepad(GamepadButton::Start)),
        _ => return None,
    };
    Some(KeyInput::Key(key))
}

#[wasm_bindgen]
pub struct FluxaWeb {
    host: FluxaHost,
    canvas: HtmlCanvasElement,
    _listeners: Vec<Closure<dyn FnMut(JsValue)>>,
}

#[wasm_bindgen]
impl FluxaWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas_id: &str,
        form_factor: &str,
        webgpu: bool,
        image_proxy: Option<String>,
    ) -> Result<FluxaWeb, JsValue> {
        fluxa_host::set_logger(log);
        if let Some(proxy) = image_proxy.filter(|proxy| !proxy.is_empty()) {
            fluxa_artwork::set_fetch_proxy(&proxy);
        }
        let window = web_sys::window().ok_or("window is not available")?;
        let canvas: HtmlCanvasElement = window
            .document()
            .and_then(|document| document.get_element_by_id(canvas_id))
            .ok_or("canvas not found")?
            .dyn_into()?;
        let host = FluxaHost::new(window.device_pixel_ratio() as f32, None);
        host.set_form_factor(form_factor);
        host.start_session(PathBuf::from("fluxa"))
            .map_err(|error| JsValue::from_str(&error))?;
        let mut web = FluxaWeb {
            host,
            canvas,
            _listeners: Vec::new(),
        };
        let [width, height] = web.physical_size();
        let pointer = Box::leak(Box::new(JsValue::from(web.canvas.clone())));
        let handle = CanvasHandle(NonNull::from(pointer).cast());
        let surface = unsafe {
            NativeSurface::new(
                if webgpu {
                    &[GraphicsBackend::WebGpu]
                } else {
                    &[GraphicsBackend::Gles]
                },
                move || {
                    let handle = &handle;
                    Ok(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle: Some(RawDisplayHandle::Web(WebDisplayHandle::new())),
                        raw_window_handle: RawWindowHandle::WebCanvas(WebCanvasWindowHandle::new(
                            handle.0,
                        )),
                    })
                },
            )
        };
        web.host.surface_created(surface, width, height);
        web.install_listeners()?;
        Ok(web)
    }

    pub fn resize(&self) {
        let [width, height] = self.physical_size();
        self.host.surface_changed(width, height);
    }

    pub fn frame(&self) -> Option<String> {
        self.host.render();
        self.host
            .take_actions_json()
            .filter(|actions| actions != "[]")
    }

    fn physical_size(&self) -> [u32; 2] {
        let window = web_sys::window();
        let ratio = window
            .as_ref()
            .map_or(1.0, |window| window.device_pixel_ratio());
        let css_size = |client: i32, fallback: Option<f64>| {
            if client > 0 {
                f64::from(client)
            } else {
                fallback.unwrap_or(1.0)
            }
        };
        let inner = |value: Result<JsValue, JsValue>| value.ok().and_then(|value| value.as_f64());
        let width = css_size(
            self.canvas.client_width(),
            window
                .as_ref()
                .and_then(|window| inner(window.inner_width())),
        );
        let height = css_size(
            self.canvas.client_height(),
            window
                .as_ref()
                .and_then(|window| inner(window.inner_height())),
        );
        let width = (width * ratio).round().max(1.0) as u32;
        let height = (height * ratio).round().max(1.0) as u32;
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        [width, height]
    }

    fn listen(
        &mut self,
        target: &web_sys::EventTarget,
        event: &str,
        handler: impl FnMut(JsValue) + 'static,
    ) -> Result<(), JsValue> {
        let closure = Closure::<dyn FnMut(JsValue)>::new(handler);
        target.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())?;
        self._listeners.push(closure);
        Ok(())
    }

    fn install_listeners(&mut self) -> Result<(), JsValue> {
        let canvas: web_sys::EventTarget = self.canvas.clone().into();
        let window: web_sys::EventTarget =
            web_sys::window().ok_or("window is not available")?.into();
        let pressed = Rc::new(RefCell::new(false));
        for (name, phase) in [
            ("pointerdown", PointerPhase::Down),
            ("pointermove", PointerPhase::Move),
            ("pointerup", PointerPhase::Up),
            ("pointercancel", PointerPhase::Up),
        ] {
            let host = self.host.clone();
            let pressed = pressed.clone();
            let canvas_element = self.canvas.clone();
            self.listen(&canvas, name, move |event| {
                let Ok(event) = event.dyn_into::<PointerEvent>() else {
                    return;
                };
                match phase {
                    PointerPhase::Down => {
                        *pressed.borrow_mut() = true;
                        let _ = canvas_element.set_pointer_capture(event.pointer_id());
                    }
                    PointerPhase::Up => *pressed.borrow_mut() = false,
                    PointerPhase::Move => {}
                }
                event.prevent_default();
                host.pointer(phase, event.offset_x() as f32, event.offset_y() as f32);
            })?;
        }
        let host = self.host.clone();
        self.listen(&canvas, "wheel", move |event| {
            let Ok(event) = event.dyn_into::<WheelEvent>() else {
                return;
            };
            event.prevent_default();
            let delta = match event.delta_mode() {
                1 => event.delta_y() * 40.0,
                2 => event.delta_y() * 800.0,
                _ => event.delta_y(),
            };
            host.scroll(delta as f32);
        })?;
        let host = self.host.clone();
        self.listen(&window, "keydown", move |event| {
            let Ok(event) = event.dyn_into::<KeyboardEvent>() else {
                return;
            };
            if let Some(input) = key_input(&event) {
                event.prevent_default();
                host.key_down(input);
            } else if event.key().chars().count() == 1 && !event.ctrl_key() && !event.meta_key() {
                host.text_input(&event.key());
            }
        })?;
        Ok(())
    }
}
