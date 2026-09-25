use std::{
    ffi::{CStr, CString, c_char, c_void},
    path::PathBuf,
    ptr::NonNull,
};

use fluxa_mobile_host::{GamepadButton, Key, KeyInput, MobileHost, NativeSurface, PointerPhase};
use fluxa_renderer::platform::APPLE_BACKEND_ORDER;

pub type FluxaRenderer = MobileHost;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(value: *const c_void);
}

struct MetalLayer(NonNull<c_void>);

unsafe impl Send for MetalLayer {}
unsafe impl Sync for MetalLayer {}

impl Drop for MetalLayer {
    fn drop(&mut self) {
        unsafe { CFRelease(self.0.as_ptr()) };
    }
}

unsafe fn text(value: *const c_char) -> Option<String> {
    (!value.is_null()).then(|| {
        unsafe { CStr::from_ptr(value) }
            .to_string_lossy()
            .into_owned()
    })
}

fn owned(value: Option<String>) -> *mut c_char {
    value
        .and_then(|value| CString::new(value).ok())
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

fn key_input(code: u32) -> Option<KeyInput> {
    let key = match code {
        1 => Key::Up,
        2 => Key::Down,
        3 => Key::Left,
        4 => Key::Right,
        5 => Key::Enter,
        6 => Key::Back,
        7 => Key::Escape,
        8 => Key::Tab,
        9 => Key::ShiftTab,
        10 => return Some(KeyInput::Backspace),
        20 => return Some(KeyInput::Gamepad(GamepadButton::South)),
        21 => return Some(KeyInput::Gamepad(GamepadButton::East)),
        22 => return Some(KeyInput::Gamepad(GamepadButton::West)),
        23 => return Some(KeyInput::Gamepad(GamepadButton::North)),
        24 => return Some(KeyInput::Gamepad(GamepadButton::DPadUp)),
        25 => return Some(KeyInput::Gamepad(GamepadButton::DPadDown)),
        26 => return Some(KeyInput::Gamepad(GamepadButton::DPadLeft)),
        27 => return Some(KeyInput::Gamepad(GamepadButton::DPadRight)),
        28 => return Some(KeyInput::Gamepad(GamepadButton::Start)),
        29 => return Some(KeyInput::Gamepad(GamepadButton::Select)),
        _ => return None,
    };
    Some(KeyInput::Key(key))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_create(
    density: f32,
    artwork_cache_dir: *const c_char,
) -> *mut FluxaRenderer {
    let cache = unsafe { text(artwork_cache_dir) }.map(PathBuf::from);
    Box::into_raw(Box::new(MobileHost::new(density, cache)))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_destroy(renderer: *mut FluxaRenderer) {
    if !renderer.is_null() {
        drop(unsafe { Box::from_raw(renderer) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_surface_created(
    renderer: *const FluxaRenderer,
    metal_layer: *mut c_void,
    width: u32,
    height: u32,
) {
    let (Some(renderer), Some(layer)) = (unsafe { renderer.as_ref() }, NonNull::new(metal_layer))
    else {
        return;
    };
    let layer = MetalLayer(layer);
    let surface = unsafe {
        NativeSurface::new(&APPLE_BACKEND_ORDER, move || {
            let layer = &layer;
            Ok(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(
                layer.0.as_ptr(),
            ))
        })
    };
    renderer.surface_created(surface, width, height);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_surface_changed(
    renderer: *const FluxaRenderer,
    width: u32,
    height: u32,
) {
    if let Some(renderer) = unsafe { renderer.as_ref() } {
        renderer.surface_changed(width, height);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_surface_destroyed(renderer: *const FluxaRenderer) {
    if let Some(renderer) = unsafe { renderer.as_ref() } {
        renderer.surface_destroyed();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_render(renderer: *const FluxaRenderer) {
    if let Some(renderer) = unsafe { renderer.as_ref() } {
        renderer.render();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_set_safe_bottom_inset(
    renderer: *const FluxaRenderer,
    inset: f32,
) {
    if let Some(renderer) = unsafe { renderer.as_ref() } {
        renderer.set_safe_bottom_inset(inset);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_set_form_factor(
    renderer: *const FluxaRenderer,
    form_factor: *const c_char,
) {
    if let (Some(renderer), Some(form_factor)) =
        (unsafe { renderer.as_ref() }, unsafe { text(form_factor) })
    {
        renderer.set_form_factor(&form_factor);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_set_home_state(
    renderer: *const FluxaRenderer,
    json: *const c_char,
) {
    if let (Some(renderer), Some(json)) = (unsafe { renderer.as_ref() }, unsafe { text(json) }) {
        renderer.set_home_state_json(&json);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_set_core_snapshot(
    renderer: *const FluxaRenderer,
    json: *const c_char,
) {
    if let (Some(renderer), Some(json)) = (unsafe { renderer.as_ref() }, unsafe { text(json) }) {
        renderer.set_core_snapshot_json(&json);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_snapshot(renderer: *const FluxaRenderer) -> *mut c_char {
    owned(unsafe { renderer.as_ref() }.and_then(MobileHost::snapshot_json))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_poll_actions(
    renderer: *const FluxaRenderer,
) -> *mut c_char {
    owned(unsafe { renderer.as_ref() }.and_then(MobileHost::take_actions_json))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_scroll(renderer: *const FluxaRenderer, delta_y: f32) {
    if let Some(renderer) = unsafe { renderer.as_ref() } {
        renderer.scroll(delta_y);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_pointer(
    renderer: *const FluxaRenderer,
    phase: u32,
    x: f32,
    y: f32,
) {
    let phase = match phase {
        0 => PointerPhase::Move,
        1 => PointerPhase::Down,
        2 => PointerPhase::Up,
        _ => return,
    };
    if let Some(renderer) = unsafe { renderer.as_ref() } {
        renderer.pointer(phase, x, y);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_key_down(renderer: *const FluxaRenderer, key: u32) {
    if let (Some(renderer), Some(input)) = (unsafe { renderer.as_ref() }, key_input(key)) {
        renderer.key_down(input);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_text_input(
    renderer: *const FluxaRenderer,
    value: *const c_char,
) {
    if let (Some(renderer), Some(value)) = (unsafe { renderer.as_ref() }, unsafe { text(value) }) {
        renderer.text_input(&value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_focused_node(renderer: *const FluxaRenderer) -> i64 {
    unsafe { renderer.as_ref() }
        .and_then(MobileHost::focused_node)
        .map(|node| node as i64)
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_focus_node(renderer: *const FluxaRenderer, node: u64) {
    if let Some(renderer) = unsafe { renderer.as_ref() } {
        renderer.focus_node(node);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fluxa_renderer_activate_node(renderer: *const FluxaRenderer, node: u64) {
    if let Some(renderer) = unsafe { renderer.as_ref() } {
        renderer.activate_node(node);
    }
}
