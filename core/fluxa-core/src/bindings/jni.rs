#![allow(clippy::missing_safety_doc)]

use crate::app_state::*;
use crate::headless_engine::*;
pub(crate) use jni::JNIEnv;
use jni::objects::JClass;
pub(crate) use jni::objects::JString;
use jni::sys::{jboolean, jlong, jstring};
use std::ptr;

pub(crate) type JBoolean = jboolean;
pub(crate) type JLong = jlong;
pub(crate) type JObject<'local> = JClass<'local>;
pub(crate) type JStringReturn = jstring;

pub(crate) fn read_jstring(env: &mut JNIEnv<'_>, value: &JString<'_>) -> Option<String> {
    env.get_string(value)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
}

pub(crate) fn write_jstring(env: &mut JNIEnv<'_>, value: Option<String>) -> JStringReturn {
    let Some(value) = value else {
        return ptr::null_mut();
    };
    env.new_string(value)
        .map(JString::into_raw)
        .unwrap_or_else(|_| ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_coreInvokeNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    method: JString<'_>,
    args_json: JString<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = read_jstring(&mut env, &method).and_then(|method| {
            let args_json = read_jstring(&mut env, &args_json)?;
            Some(crate::ffi::core_invoke(&method, &args_json))
        });
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_createAppCoreStateNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    initial_json: JString<'_>,
) -> JLong {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        read_jstring(&mut env, &initial_json)
            .map(|initial_json| create_app_core_state(&initial_json) as JLong)
            .unwrap_or(0)
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_destroyAppCoreStateNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if handle > 0 && destroy_app_core_state(handle as u64) {
            1
        } else {
            0
        }
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreStateJsonNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = if handle > 0 {
            app_core_state_json(handle as u64)
        } else {
            None
        };
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreDispatchJsonNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    action_json: JString<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = if handle > 0 {
            read_jstring(&mut env, &action_json)
                .and_then(|action_json| app_core_dispatch_json(handle as u64, &action_json))
        } else {
            None
        };
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreDispatchDeltaJsonNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    action_json: JString<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = if handle > 0 {
            read_jstring(&mut env, &action_json)
                .and_then(|action_json| app_core_dispatch_delta_json(handle as u64, &action_json))
        } else {
            None
        };
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreSetPlayerPositionNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    position_ms: JLong,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0 && app_core_set_player_position(handle as u64, position_ms)) as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreSetPlayerBufferingNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    buffering: JBoolean,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0 && app_core_set_player_buffering(handle as u64, buffering != 0)) as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreSetPlayerStreamIndexNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    stream_index: JLong,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0 && app_core_set_player_stream_index(handle as u64, stream_index)) as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreSetPlayerPlaybackEndedNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    ended: JBoolean,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0 && app_core_set_player_playback_ended(handle as u64, ended != 0)) as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreSetPlayerVideoRenderedNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    rendered: JBoolean,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0 && app_core_set_player_video_rendered(handle as u64, rendered != 0)) as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreSetPlayerStartedNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    started: JBoolean,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0 && app_core_set_player_started(handle as u64, started != 0)) as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_appCoreUpdatePlayerNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    position_ms: JLong,
    stream_index: JLong,
    buffering: JBoolean,
    playback_ended: JBoolean,
    started: JBoolean,
    rendered: JBoolean,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0
            && app_core_update_player(
                handle as u64,
                position_ms,
                stream_index,
                buffering != 0,
                playback_ended != 0,
                started != 0,
                rendered != 0,
            )) as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_createHeadlessEngineNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    initial_json: JString<'_>,
) -> JLong {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        read_jstring(&mut env, &initial_json)
            .map(|initial_json| create_headless_engine(&initial_json) as JLong)
            .unwrap_or(0)
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_destroyHeadlessEngineNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if handle > 0 && destroy_headless_engine(handle as u64) {
            1
        } else {
            0
        }
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_headlessEngineSnapshotJsonNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = if handle > 0 {
            headless_engine_snapshot_json(handle as u64)
        } else {
            None
        };
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_headlessEngineDispatchJsonNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    action_json: JString<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = if handle > 0 {
            read_jstring(&mut env, &action_json)
                .and_then(|action_json| headless_engine_dispatch_json(handle as u64, &action_json))
        } else {
            None
        };
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_headlessEngineSetPlayerBufferingNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    buffering: JBoolean,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0 && headless_engine_set_player_buffering(handle as u64, buffering != 0))
            as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_headlessEngineSetPlayerStreamIndexNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    stream_index: JLong,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0 && headless_engine_set_player_stream_index(handle as u64, stream_index))
            as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_headlessEngineSetPlayerPositionNative(
    _env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    position_ms: JLong,
) -> JBoolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (handle > 0 && headless_engine_set_player_position(handle as u64, position_ms)) as JBoolean
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_headlessEngineCompleteEffectJsonNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
    handle: JLong,
    result_json: JString<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let output = if handle > 0 {
            read_jstring(&mut env, &result_json).and_then(|result_json| {
                headless_engine_complete_effect_json(handle as u64, &result_json)
            })
        } else {
            None
        };
        write_jstring(&mut env, output)
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_com_fluxa_app_core_rust_FluxaCoreNative_drainCoreErrorLogJsonNative(
    mut env: JNIEnv<'_>,
    _class: JObject<'_>,
) -> JStringReturn {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        write_jstring(&mut env, Some(crate::log_sink::drain_core_log_json()))
    }))
    .unwrap_or(ptr::null_mut())
}
