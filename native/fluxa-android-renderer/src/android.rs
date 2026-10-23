use std::{
    ffi::{CString, c_char, c_void},
    path::PathBuf,
    ptr::NonNull,
    sync::{Arc, Mutex, OnceLock},
};

use fluxa_host::{
    FluxaHost, GamepadButton, Key, KeyInput, NativeSurface, PointerPhase, VideoStatus,
};
use fluxa_renderer::platform::ANDROID_BACKEND_ORDER;
use jni::{
    Env, EnvUnowned, JavaVM, Outcome, jni_str,
    objects::{Global, JByteArray, JClass, JIntArray, JObject, JString, JValue},
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

fn attached<T>(
    env: &mut EnvUnowned<'_>,
    run: impl FnOnce(&mut Env<'_>) -> jni::errors::Result<T>,
) -> Option<T> {
    match env.with_env(run).into_outcome() {
        Outcome::Ok(value) => Some(value),
        _ => None,
    }
}

fn string(env: &mut EnvUnowned<'_>, value: &JString<'_>) -> Option<String> {
    attached(env, |env| value.try_to_string(env))
}

fn java_string(env: &mut EnvUnowned<'_>, value: Option<String>) -> jni::sys::jstring {
    value
        .and_then(|value| attached(env, |env| JString::from_str(env, value).map(|s| s.as_raw())))
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

static EMOJI: OnceLock<Global<JClass<'static>>> = OnceLock::new();

fn rasterize_emoji(text: &str, size: u32) -> Option<(u32, u32, Vec<u8>)> {
    let class = EMOJI.get()?;
    let pixels = JavaVM::singleton()
        .ok()?
        .attach_current_thread(|env| -> jni::errors::Result<Option<Vec<i32>>> {
            let text = JString::from_str(env, text)?;
            let result = env
                .call_static_method(
                    class,
                    jni_str!("render"),
                    jni::jni_sig!("(Ljava/lang/String;I)[I"),
                    &[JValue::Object(&text), JValue::Int(size as i32)],
                )
                .and_then(|value| value.l());
            let array = match result {
                Ok(array) if !array.is_null() => env.cast_local::<JIntArray>(array)?,
                _ => {
                    env.exception_clear();
                    return Ok(None);
                }
            };
            let len = env.get_array_length(&array)? as usize;
            let mut pixels = vec![0i32; len];
            env.get_int_array_region(&array, 0, &mut pixels)?;
            Ok(Some(pixels))
        })
        .ok()??;
    if pixels.len() < 2 {
        return None;
    }
    let (width, height) = (pixels[0] as u32, pixels[1] as u32);
    let rgba = pixels[2..]
        .iter()
        .flat_map(|&argb| {
            let [a, r, g, b] = (argb as u32).to_be_bytes();
            [r, g, b, a]
        })
        .collect();
    Some((width, height, rgba))
}

static INPUT_FOCUS: OnceLock<Global<JClass<'static>>> = OnceLock::new();

fn input_focus_changed() {
    let Some(class) = INPUT_FOCUS.get() else {
        return;
    };
    let Ok(vm) = JavaVM::singleton() else {
        return;
    };
    let _ = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
        if env
            .call_static_method(class, jni_str!("changed"), jni::jni_sig!("()V"), &[])
            .is_err()
        {
            env.exception_clear();
        }
        Ok(())
    });
}

static ARTWORK: OnceLock<Global<JClass<'static>>> = OnceLock::new();

fn decode_artwork(bytes: &[u8], target: [u32; 2]) -> Option<(u32, u32, Vec<u8>)> {
    let class = ARTWORK.get()?;
    let mut raw = JavaVM::singleton()
        .ok()?
        .attach_current_thread(|env| -> jni::errors::Result<Option<Vec<u8>>> {
            let input = env.byte_array_from_slice(bytes)?;
            let result = env
                .call_static_method(
                    class,
                    jni_str!("decode"),
                    jni::jni_sig!("([BII)[B"),
                    &[
                        JValue::Object(&input),
                        JValue::Int(target[0] as i32),
                        JValue::Int(target[1] as i32),
                    ],
                )
                .and_then(|value| value.l());
            let array = match result {
                Ok(array) if !array.is_null() => env.cast_local::<JByteArray>(array)?,
                _ => {
                    env.exception_clear();
                    return Ok(None);
                }
            };
            env.convert_byte_array(&array).map(Some)
        })
        .ok()??;
    if raw.len() < 8 {
        return None;
    }
    let width = u32::from_be_bytes(raw[0..4].try_into().ok()?);
    let height = u32::from_be_bytes(raw[4..8].try_into().ok()?);
    raw.drain(..8);
    (raw.len() == width as usize * height as usize * 4).then_some((width, height, raw))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_createNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    density: jfloat,
    artwork_cache_dir: JString<'_>,
) -> jlong {
    fluxa_host::set_logger(android_log);
    let classes = attached(&mut env, |env| {
        let mut global = |name: &jni::strings::JNIStr| {
            env.find_class(name)
                .and_then(|class| env.new_global_ref(class))
        };
        Ok((
            global(jni_str!("com/fluxa/app/ui/rust/EmojiRasterizer")),
            global(jni_str!("com/fluxa/app/ui/rust/ArtworkDecoder")),
            global(jni_str!("com/fluxa/app/ui/rust/NativeInputFocus")),
        ))
    });
    if let Some((emoji, artwork, input_focus)) = classes {
        if let Ok(class) = emoji {
            let _ = EMOJI.set(class);
            fluxa_host::set_emoji_rasterizer(rasterize_emoji);
        }
        if let Ok(class) = artwork {
            let _ = ARTWORK.set(class);
            fluxa_host::set_artwork_decoder(decode_artwork);
        }
        if let Ok(class) = input_focus {
            let _ = INPUT_FOCUS.set(class);
            fluxa_host::set_input_focus_listener(input_focus_changed);
        }
    }
    let artwork_cache_dir = string(&mut env, &artwork_cache_dir).map(PathBuf::from);
    let host = FluxaHost::new(density, artwork_cache_dir);
    host.set_platform("android");
    host.set_video_backend(Box::new(fluxa_host::BridgeVideo(video_bridge().clone())));
    Box::into_raw(Box::new(host)) as jlong
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_pollVideoNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
) -> jni::sys::jstring {
    let requests = video_bridge()
        .lock()
        .map(|mut bridge| bridge.take_requests())
        .unwrap_or_default();
    java_string(
        &mut env,
        Some(serde_json::Value::from(requests).to_string()),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_videoStatusNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    position: jdouble,
    duration: jdouble,
    paused: jboolean,
    muted: jboolean,
    has_frame: jboolean,
    buffering: jfloat,
    error: JString<'_>,
) {
    let error = (!error.is_null())
        .then(|| string(&mut env, &error))
        .flatten();
    let status = VideoStatus {
        position,
        duration,
        paused: paused,
        muted: muted,
        volume: if muted { 0.0 } else { 100.0 },
        has_frame: has_frame,
        error,
        chapters: Vec::new(),
        buffering: (buffering >= 0.0).then_some(buffering),
        stalled: buffering >= 0.0,
        buffered: 0.0,
    };
    if let Ok(mut bridge) = video_bridge().lock() {
        bridge.set_status(status);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_videoTracksNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    json: JString<'_>,
) {
    let Some(text) = string(&mut env, &json) else {
        return;
    };
    let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&text) else {
        return;
    };
    let text_of = |item: &serde_json::Value, key: &str| {
        item.get(key)
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    let flag = |item: &serde_json::Value, key: &str| {
        item.get(key)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    };
    let tracks = items
        .iter()
        .filter_map(|item| {
            Some(fluxa_host::VideoTrack {
                id: text_of(item, "id")?,
                subtitle: flag(item, "subtitle"),
                language: text_of(item, "language"),
                title: text_of(item, "title"),
                selected: flag(item, "selected"),
                external: flag(item, "external"),
                forced: flag(item, "forced"),
            })
        })
        .collect();
    if let Ok(mut bridge) = video_bridge().lock() {
        bridge.set_tracks(tracks);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_videoChaptersNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    json: JString<'_>,
) {
    let Some(text) = string(&mut env, &json) else {
        return;
    };
    let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(&text) else {
        return;
    };
    let chapters = items
        .iter()
        .filter_map(|item| {
            Some((
                item.get("startMs")?.as_u64()? as f64 / 1000.0,
                item.get("title")?.as_str()?.to_owned(),
            ))
        })
        .collect();
    if let Ok(mut bridge) = video_bridge().lock() {
        bridge.set_chapters(chapters);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_videoThumbnailNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    data: JByteArray<'_>,
) {
    let Some(bytes) = attached(&mut env, |env| env.convert_byte_array(&data)) else {
        return;
    };
    let Some((head, rgba)) = bytes.split_at_checked(12) else {
        return;
    };
    let field = |index: usize| {
        i32::from_le_bytes(
            head[index * 4..index * 4 + 4]
                .try_into()
                .unwrap_or_default(),
        )
    };
    let (width, height) = (field(0).max(0) as usize, field(1).max(0) as usize);
    if width == 0 || height == 0 || rgba.len() != width * height * 4 {
        return;
    }
    let thumbnail = fluxa_host::Thumbnail {
        time: field(2) as f64 / 1000.0,
        size: [width, height],
        rgba: rgba.to_vec(),
    };
    if let Ok(mut bridge) = video_bridge().lock() {
        bridge.set_thumbnail(thumbnail);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_videoThumbnailProgressNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    done: jint,
    count: jint,
) {
    if let Ok(mut bridge) = video_bridge().lock() {
        bridge.set_thumbnail_progress(fluxa_host::ThumbnailProgress {
            done: done.max(0) as usize,
            count: count.max(0) as usize,
        });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_startSessionNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
    data_dir: JString<'_>,
) -> jni::sys::jboolean {
    let (Some(host), Some(dir)) = (host(handle), string(&mut env, &data_dir)) else {
        return false;
    };
    match host.start_session(PathBuf::from(dir)) {
        Ok(()) => true,
        Err(error) => {
            android_log(&format!("core session failed to start: {error}"));
            false
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_destroyNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    if handle != 0 {
        drop(unsafe { Box::from_raw(handle as *mut FluxaHost) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setSafeBottomInsetNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
    inset_dp: jfloat,
) {
    if let Some(host) = host(handle) {
        host.set_safe_bottom_inset(inset_dp);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setSafeTopInsetNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
    inset_dp: jfloat,
) {
    if let Some(host) = host(handle) {
        host.set_safe_top_inset(inset_dp);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setWindowFocusedNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
    focused: jboolean,
) {
    if let Some(host) = host(handle) {
        host.set_window_focused(focused);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setFormFactorNative(
    mut env: EnvUnowned<'_>,
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
    mut env: EnvUnowned<'_>,
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
    mut env: EnvUnowned<'_>,
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
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(&mut env, host(handle).and_then(FluxaHost::snapshot_json))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_accessibilityTreeNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(
        &mut env,
        host(handle).and_then(FluxaHost::accessibility_json),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_pollActionsNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(
        &mut env,
        host(handle).and_then(FluxaHost::take_actions_json),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_takeOpenUrlNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(&mut env, host(handle).and_then(FluxaHost::take_open_url))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_takeExternalPlayNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(
        &mut env,
        host(handle).and_then(FluxaHost::take_external_play),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_takeInstallApkNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(&mut env, host(handle).and_then(FluxaHost::take_install_apk))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_launcherFeedNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(
        &mut env,
        host(handle).and_then(FluxaHost::launcher_feed_json),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_takeAppIconNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(&mut env, host(handle).and_then(FluxaHost::take_app_icon))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_scrollNative(
    _env: EnvUnowned<'_>,
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
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
    action: jint,
    x: f32,
    y: f32,
    event_ms: jlong,
) {
    let phase = match action {
        0 => PointerPhase::Move,
        1 => PointerPhase::Down,
        2 => PointerPhase::Up,
        _ => return,
    };
    if let Some(host) = host(handle) {
        host.pointer_at(phase, x, y, event_ms.max(0) as u64);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_keyDownNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
    key_code: jint,
    shift: jboolean,
) {
    if let (Some(host), Some(input)) = (host(handle), android_key(key_code, shift)) {
        host.key_down(input);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_focusedNodeNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jlong {
    host(handle)
        .and_then(FluxaHost::focused_node)
        .map(|node| node as jlong)
        .unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_textInputFocusedNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jboolean {
    host(handle).is_some_and(FluxaHost::text_input_focused)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_isPlayingNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jboolean {
    host(handle).is_some_and(|host| host.is_playing())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_pipStateNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jint {
    host(handle).map_or(0, FluxaHost::pip_state)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setPipNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
    on: jni::sys::jboolean,
) {
    if let Some(host) = host(handle) {
        host.set_pip(on);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_keepAwakeNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jboolean {
    host(handle).is_some_and(|host| host.keep_awake())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_wantsLandscapeNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jboolean {
    host(handle).is_some_and(|host| host.wants_landscape())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_focusNodeNative(
    _env: EnvUnowned<'_>,
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
    _env: EnvUnowned<'_>,
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
    mut env: EnvUnowned<'_>,
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
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
    surface: JObject<'_>,
    width: jint,
    height: jint,
) {
    let Some(host) = host(handle) else { return };
    let pointer = attached(&mut env, |env| Ok(env.get_raw()))
        .map_or(std::ptr::null_mut(), |raw| unsafe {
            ANativeWindow_fromSurface(raw, surface.as_raw())
        });
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
    _env: EnvUnowned<'_>,
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
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    if let Some(host) = host(handle) {
        host.surface_destroyed();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_renderNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    if let Some(host) = host(handle) {
        host.render_due();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_importLegacyNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    data_dir: JString<'_>,
    legacy: JString<'_>,
) -> jni::sys::jboolean {
    let (Some(dir), Some(legacy)) = (string(&mut env, &data_dir), string(&mut env, &legacy)) else {
        return false;
    };
    match fluxa_host::import_legacy(PathBuf::from(dir), &legacy) {
        Ok(imported) => imported,
        Err(error) => {
            android_log(&format!("legacy import failed: {error}"));
            false
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_pushActionNative(
    mut env: EnvUnowned<'_>,
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
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jboolean {
    host(handle).is_some_and(|host| host.back())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_focusedTextNative<
    'local,
>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jni::sys::jstring {
    let text = host(handle)
        .and_then(FluxaHost::focused_text)
        .unwrap_or_default();
    java_string(&mut env, Some(text))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_setFieldTextNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
    field: jlong,
    text: JString<'_>,
) {
    if let (Some(host), Some(text)) = (host(handle), string(&mut env, &text)) {
        host.set_field_text(field as u64, &text);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_focusedFieldIdNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jlong {
    host(handle).map_or(0, FluxaHost::focused_field_id) as jlong
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_textFieldNative(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jni::sys::jstring {
    java_string(&mut env, host(handle).and_then(FluxaHost::text_field_json))
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_ui_rust_NativeRenderer_blurTextInputNative(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    if let Some(host) = host(handle) {
        host.blur_text_input();
    }
}
