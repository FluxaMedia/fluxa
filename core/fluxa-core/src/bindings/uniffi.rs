use crate::{app_state, core_contract, headless_engine};
use serde_json::json;
#[cfg(feature = "plugin-js-engine")]
use std::sync::Arc;

// A panic anywhere below must not unwind across the UniFFI boundary into
// Swift/Kotlin — that's undefined behavior, not a catchable exception there.
fn guard<T>(default: T, f: impl FnOnce() -> T) -> T {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(default)
}

#[uniffi::export]
pub fn fluxa_core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Funnel entry point — Swift calls this instead of binding each helper.
#[uniffi::export]
pub fn core_invoke(method: String, args_json: String) -> String {
    crate::ffi::core_invoke(&method, &args_json)
}

/// Returns the parsed subtitle cues for the platform subtitle picker.
///
/// These explicit UniFFI functions keep Swift consumers independent from the
/// generic `core_invoke` bridge while leaving the subtitle-sync policy in Rust.
#[uniffi::export]
pub fn subtitle_cue_list_json(subtitle_text: String) -> String {
    guard(String::new(), || {
        crate::subtitle_sync::subtitle_cue_list_json(
            &json!({ "subtitleText": subtitle_text }).to_string(),
        )
        .unwrap_or_default()
    })
}

#[uniffi::export]
pub fn subtitle_sync_capture_json(subtitle_text: String, current_time: f64) -> String {
    guard(String::new(), || {
        crate::subtitle_sync::subtitle_sync_capture_json(
            &json!({
                "subtitleText": subtitle_text,
                "currentTime": current_time,
            })
            .to_string(),
        )
        .unwrap_or_default()
    })
}

#[uniffi::export]
pub fn subtitle_sync_apply_json(captured_time: f64, cue_start: f64) -> String {
    guard(String::new(), || {
        crate::subtitle_sync::subtitle_sync_apply_json(
            &json!({
                "capturedTime": captured_time,
                "cueStart": cue_start,
            })
            .to_string(),
        )
        .unwrap_or_default()
    })
}

/// Structured alternatives to the legacy lifecycle calls below. They preserve
/// the common `{ ok, value | error }` contract without changing existing FFI
/// return types used by older Kotlin and Swift clients.
#[uniffi::export]
pub fn headless_engine_snapshot_result_json(handle: i64) -> String {
    crate::ffi::core_invoke("engine.snapshot", &json!({ "handle": handle }).to_string())
}

#[uniffi::export]
pub fn headless_engine_dispatch_result_json(handle: i64, action_json: String) -> String {
    let action = serde_json::from_str::<serde_json::Value>(&action_json)
        .unwrap_or(serde_json::Value::String(action_json));
    crate::ffi::core_invoke(
        "engine.dispatch",
        &json!({ "handle": handle, "action": action }).to_string(),
    )
}

#[uniffi::export]
pub fn headless_engine_complete_effect_result_json(handle: i64, result_json: String) -> String {
    let result = serde_json::from_str::<serde_json::Value>(&result_json)
        .unwrap_or(serde_json::Value::String(result_json));
    crate::ffi::core_invoke(
        "engine.completeEffect",
        &json!({ "handle": handle, "result": result }).to_string(),
    )
}

#[uniffi::export]
pub fn app_core_state_result_json(handle: i64) -> String {
    crate::ffi::core_invoke("app.state", &json!({ "handle": handle }).to_string())
}

#[uniffi::export]
pub fn app_core_dispatch_result_json(handle: i64, action_json: String) -> String {
    let action = serde_json::from_str::<serde_json::Value>(&action_json)
        .unwrap_or(serde_json::Value::String(action_json));
    crate::ffi::core_invoke(
        "app.dispatch",
        &json!({ "handle": handle, "action": action }).to_string(),
    )
}

#[uniffi::export]
pub fn create_headless_engine_json(initial_json: String) -> i64 {
    guard(0, || {
        headless_engine::create_headless_engine(&initial_json) as i64
    })
}

#[uniffi::export]
pub fn destroy_headless_engine_json(handle: i64) -> bool {
    handle > 0
        && guard(false, || {
            headless_engine::destroy_headless_engine(handle as u64)
        })
}

#[uniffi::export]
pub fn headless_engine_snapshot_json(handle: i64) -> String {
    if handle <= 0 {
        return String::new();
    }
    guard(String::new(), || {
        headless_engine::headless_engine_snapshot_json(handle as u64).unwrap_or_default()
    })
}

#[uniffi::export]
pub fn headless_engine_dispatch_json(handle: i64, action_json: String) -> String {
    if handle <= 0 {
        return String::new();
    }
    guard(String::new(), || {
        headless_engine::headless_engine_dispatch_json(handle as u64, &action_json)
            .unwrap_or_default()
    })
}

#[uniffi::export]
pub fn headless_engine_complete_effect_json(handle: i64, result_json: String) -> String {
    if handle <= 0 {
        return String::new();
    }
    guard(String::new(), || {
        headless_engine::headless_engine_complete_effect_json(handle as u64, &result_json)
            .unwrap_or_default()
    })
}

#[uniffi::export]
pub fn core_capabilities_json(portable: bool) -> String {
    guard(String::new(), || {
        core_contract::core_capabilities_json(portable)
    })
}

#[uniffi::export]
pub fn drain_core_error_log_json() -> String {
    guard(String::new(), crate::log_sink::drain_core_log_json)
}

#[uniffi::export]
pub fn create_app_core_state_json(initial_json: String) -> i64 {
    guard(0, || app_state::create_app_core_state(&initial_json) as i64)
}

#[uniffi::export]
pub fn destroy_app_core_state_json(handle: i64) -> bool {
    handle > 0 && guard(false, || app_state::destroy_app_core_state(handle as u64))
}

#[uniffi::export]
pub fn app_core_state_json(handle: i64) -> String {
    if handle <= 0 {
        return String::new();
    }
    guard(String::new(), || {
        app_state::app_core_state_json(handle as u64).unwrap_or_default()
    })
}

#[uniffi::export]
pub fn app_core_dispatch_json(handle: i64, action_json: String) -> String {
    if handle <= 0 {
        return String::new();
    }
    guard(String::new(), || {
        app_state::app_core_dispatch_json(handle as u64, &action_json).unwrap_or_default()
    })
}

#[uniffi::export]
pub fn app_core_dispatch_delta_json(handle: i64, action_json: String) -> String {
    if handle <= 0 {
        return String::new();
    }
    guard(String::new(), || {
        app_state::app_core_dispatch_delta_json(handle as u64, &action_json).unwrap_or_default()
    })
}

#[uniffi::export]
pub fn app_core_update_player(
    handle: i64,
    position_ms: i64,
    stream_index: i64,
    buffering: bool,
    playback_ended: bool,
    started: bool,
    rendered: bool,
) -> bool {
    handle > 0
        && guard(false, || {
            app_state::app_core_update_player(
                handle as u64,
                position_ms,
                stream_index,
                buffering,
                playback_ended,
                started,
                rendered,
            )
        })
}

#[cfg(feature = "plugin-js-engine")]
#[uniffi::export]
#[expect(
    clippy::too_many_arguments,
    reason = "UniFFI signature must preserve the existing platform ABI"
)]
pub fn execute_plugin_scraper(
    client: Box<dyn crate::plugin_runtime::PluginHttpClient>,
    code: String,
    repository_url: String,
    scraper_id: String,
    scraper_settings_json: String,
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
) -> String {
    guard("[]".to_string(), || {
        crate::plugin_runtime::execute_scraper(
            Arc::from(client),
            code,
            repository_url,
            scraper_id,
            scraper_settings_json,
            tmdb_id,
            media_type,
            season,
            episode,
        )
        .unwrap_or_else(|_| "[]".to_string())
    })
}

#[cfg(feature = "plugin-js-engine")]
#[uniffi::export]
pub fn get_plugin_scraper_settings_layout(code: String, scraper_id: String) -> String {
    guard("[]".to_string(), || {
        crate::plugin_runtime::get_settings_layout(code, scraper_id)
    })
}
