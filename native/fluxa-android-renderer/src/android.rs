use std::{
    ffi::{CString, c_char, c_void},
    path::PathBuf,
    ptr::NonNull,
    sync::{Arc, Mutex, OnceLock},
};

use fluxa_host::{FluxaHost, VideoStatus, GamepadButton, Key, KeyInput, NativeSurface, PointerPhase};
use fluxa_renderer::platform::ANDROID_BACKEND_ORDER;
use jni::{
    JNIEnv,
    objects::{JClass, JObject, JString},
    sys::{jboolean, jdouble, jfloat, jint, jlong},
};
use raw_window_handle::{AndroidDisplayHandle, AndroidNdkWindowHandle, RawWindowHandle};

#[link(name = "android")]
unsafe extern "C" {
    fn __android_log_write(priority: i32, tag: *const c_char, text: *const c_char) -> i32;
    fn ANativeWindow_fromSurface(
        env: *mut jni::sys::JNIEnv,
        surface: jni::sys::jobject,
    ) -> *mut c_void;
    fn ANativeWindow_release(window: *mut c_void);
}

fn android_log(message: &str) {
    let Ok(tag) = CString::new("FluxaNativeRenderer") else {
        return;
    };
    let Ok(message) = CString::new(message.replace('\0', "�")) else {
        return;
    };
    unsafe { __android_log_write(4, tag.as_ptr(), message.as_ptr()) };
}

struct NativeWindowRef(NonNull<c_void>);

unsafe impl Send for NativeWindowRef {}
unsafe impl Sync for NativeWindowRef {}

impl Drop for NativeWindowRef {
    fn drop(&mut self) {
        unsafe { ANativeWindow_release(self.0.as_ptr()) };
    }
}

fn video_bridge() -> &'static Arc<Mutex<fluxa_host::VideoBridge>> {
    static BRIDGE: OnceLock<Arc<Mutex<fluxa_host::VideoBridge>>> = OnceLock::new();
    BRIDGE.get_or_init(Default::default)
}

fn host(handle: jlong) -> Option<&'static FluxaHost> {
    (handle != 0).then(|| unsafe { &*(handle as *const FluxaHost) })
}

fn string(env: &mut JNIEnv<'_>, value: &JString<'_>) -> Option<String> {
    env.get_string(value)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
}

fn java_string(env: &JNIEnv<'_>, value: Option<String>) -> jni::sys::jstring {
    value
        .and_then(|value| env.new_string(value).ok())
        .map(|value| value.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

fn android_key(key_code: jint, shift: bool) -> Option<KeyInput> {
    let gamepad = match key_code {
        96 => Some(GamepadButton::South),
        97 => Some(GamepadButton::East),
        99 | 82 => Some(GamepadButton::West),
        100 => Some(GamepadButton::North),
        19 => Some(GamepadButton::DPadUp),
        20 => Some(GamepadButton::DPadDown),
        21 => Some(GamepadButton::DPadLeft),
        22 => Some(GamepadButton::DPadRight),
        108 => Some(GamepadButton::Start),
        109 => Some(GamepadButton::Select),
        _ => None,
    };
    if let Some(button) = gamepad {
        return Some(KeyInput::Gamepad(button));
    }
    let key = match key_code {
        67 => return Some(KeyInput::Backspace),
        23 | 66 => Key::Enter,
        4 => Key::Back,
        111 => Key::Escape,
        61 if shift => Key::ShiftTab,
        61 => Key::Tab,
        _ => return None,
    };
    Some(KeyInput::Key(key))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_createNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    density: jfloat,
    artwork_cache_dir: JString<'_>,
) -> jlong {
    fluxa_host::set_logger(android_log);
    let artwork_cache_dir = string(&mut env, &artwork_cache_dir).map(PathBuf::from);
    let host = FluxaHost::new(density, artwork_cache_dir);
    host.set_video_backend(Box::new(fluxa_host::BridgeVideo(video_bridge().clone())));
    Box::into_raw(Box::new(host)) as jlong
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_pollVideoNative(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jni::sys::jstring {
    let requests = video_bridge()
        .lock()
        .map(|mut bridge| bridge.take_requests())
        .unwrap_or_default();
    java_string(&env, Some(serde_json::Value::from(requests).to_string()))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_videoStatusNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    position: jdouble,
    duration: jdouble,
    paused: jboolean,
    muted: jboolean,
    has_frame: jboolean,
    buffering: jfloat,
    error: JString<'_>,
) {
    let error = (!error.is_null()).then(|| string(&mut env, &error)).flatten();
    let status = VideoStatus {
        position,
        duration,
        paused: paused != 0,
        muted: muted != 0,
        volume: if muted != 0 { 0.0 } else { 100.0 },
        has_frame: has_frame != 0,
        error,
        chapters: Vec::new(),
        buffering: (buffering >= 0.0).then_some(buffering),
    };
    if let Ok(mut bridge) = video_bridge().lock() {
        bridge.set_status(status);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_startSessionNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    data_dir: JString<'_>,
) -> jni::sys::jboolean {
    let (Some(host), Some(dir)) = (host(handle), string(&mut env, &data_dir)) else {
        return 0;
    };
    match host.start_session(PathBuf::from(dir)) {
        Ok(()) => 1,
        Err(error) => {
            android_log(&format!("core session failed to start: {error}"));
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_destroyNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    if handle != 0 {
        drop(unsafe { Box::from_raw(handle as *mut FluxaHost) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setSafeBottomInsetNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    inset_dp: jfloat,
) {
    if let Some(host) = host(handle) {
        host.set_safe_bottom_inset(inset_dp);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setFormFactorNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    form_factor: JString<'_>,
) {
    if let (Some(host), Some(form_factor)) = (host(handle), string(&mut env, &form_factor)) {
        host.set_form_factor(&form_factor);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setHomeStateNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    json: JString<'_>,
) {
    if let (Some(host), Some(json)) = (host(handle), string(&mut env, &json)) {
        host.set_home_state_json(&json);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setCoreSnapshotNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    snapshot_json: JString<'_>,
) {
    if let (Some(host), Some(json)) = (host(handle), string(&mut env, &snapshot_json)) {
        host.set_core_snapshot_json(&json);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_snapshotNative(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(&env, host(handle).and_then(FluxaHost::snapshot_json))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_pollActionsNative(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(&env, host(handle).and_then(FluxaHost::take_actions_json))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_scrollNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    delta_y: f32,
) {
    if let Some(host) = host(handle) {
        host.scroll(delta_y);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_pointerEventNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    action: jint,
    x: f32,
    y: f32,
) {
    let phase = match action {
        0 => PointerPhase::Move,
        1 => PointerPhase::Down,
        2 => PointerPhase::Up,
        _ => return,
    };
    if let Some(host) = host(handle) {
        host.pointer(phase, x, y);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_keyDownNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    key_code: jint,
    shift: jint,
) {
    if let (Some(host), Some(input)) = (host(handle), android_key(key_code, shift != 0)) {
        host.key_down(input);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_focusedNodeNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jlong {
    host(handle)
        .and_then(FluxaHost::focused_node)
        .map(|node| node as jlong)
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_isPlayingNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jboolean {
    host(handle).is_some_and(|host| host.is_playing()) as jni::sys::jboolean
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_focusNodeNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    node: jlong,
) {
    if let Some(host) = host(handle) {
        host.focus_node(node as u64);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_activateNodeNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    node: jlong,
) {
    if let Some(host) = host(handle) {
        host.activate_node(node as u64);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_textInputNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    text: JString<'_>,
) {
    if let (Some(host), Some(text)) = (host(handle), string(&mut env, &text)) {
        host.text_input(&text);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_surfaceCreatedNative(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    surface: JObject<'_>,
    width: jint,
    height: jint,
) {
    let Some(host) = host(handle) else { return };
    let pointer =
        unsafe { ANativeWindow_fromSurface(env.get_native_interface(), surface.as_raw()) };
    let Some(window) = NonNull::new(pointer).map(NativeWindowRef) else {
        android_log("ANativeWindow_fromSurface returned null");
        return;
    };
    let surface = unsafe {
        NativeSurface::new(&ANDROID_BACKEND_ORDER, move || {
            let window = &window;
            Ok(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(AndroidDisplayHandle::new().into()),
                raw_window_handle: RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(
                    window.0,
                )),
            })
        })
    };
    host.surface_created(surface, width.max(1) as u32, height.max(1) as u32);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_surfaceChangedNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    width: jint,
    height: jint,
) {
    if let Some(host) = host(handle) {
        host.surface_changed(width.max(1) as u32, height.max(1) as u32);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_surfaceDestroyedNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    if let Some(host) = host(handle) {
        host.surface_destroyed();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_renderNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    if let Some(host) = host(handle) {
        host.render();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_importLegacyNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    data_dir: JString<'_>,
    legacy: JString<'_>,
) -> jni::sys::jboolean {
    let (Some(dir), Some(legacy)) = (string(&mut env, &data_dir), string(&mut env, &legacy)) else {
        return 0;
    };
    match fluxa_host::import_legacy(PathBuf::from(dir), &legacy) {
        Ok(imported) => imported as jni::sys::jboolean,
        Err(error) => {
            android_log(&format!("legacy import failed: {error}"));
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_pushActionNative(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    action: JString<'_>,
) {
    let (Some(host), Some(action)) = (host(handle), string(&mut env, &action)) else {
        return;
    };
    if let Err(error) = host.push_action_json(&action) {
        android_log(&format!("invalid native action: {error}"));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_backNative(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jboolean {
    host(handle).is_some_and(|host| host.back()) as jni::sys::jboolean
}
