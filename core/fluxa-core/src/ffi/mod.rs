use serde_json::{Value, json};

mod args;
mod errors;
mod methods;
#[cfg(test)]
mod tests;
use crate::services::{
    anilist::route_anilist, mdblist::route_mdblist, mediaserver::routes::route_mediaserver,
    nuvio::route_nuvio, publicmetadb::route_publicmetadb, simkl::route_simkl,
    stremio::route_stremio, tmdb::route_tmdb, tracking::external_sync::route_external_sync,
    trakt::route_trakt,
};
use crate::{
    addons::routes::*, catalog::routes::*, headless_engine::routes::*, library::routes::*,
    player::routes::*, profile::routes::*, services::auth::route_device_auth,
    services::fluxa::route_fluxa_sync, services::provider_routes::*, settings::routes::*,
};

use crate::headless_engine::{self, app_state};
use crate::player;

pub(crate) use args::*;
pub use errors::ErrorKind;
pub(crate) use errors::{CallError, Outcome, fail, unknown_method};

pub fn core_invoke(method: &str, args_json: &str) -> String {
    if matches!(
        method,
        "app.dispatchDelta" | "engine.dispatch" | "engine.completeEffect"
    ) {
        return match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            raw_dispatch(method, args_json)
        })) {
            Ok(Ok(value)) => format!(r#"{{"ok":true,"value":{value}}}"#),
            Ok(Err(error)) => json!({
                "ok": false,
                "error": { "kind": error.kind.as_str(), "message": error.message, "method": method },
            })
            .to_string(),
            Err(_) => json!({
                "ok": false,
                "error": { "kind": ErrorKind::Internal.as_str(), "message": "internal panic", "method": method },
            })
            .to_string(),
        };
    }
    match guarded_route(method, args_json) {
        Ok(value) => json!({ "ok": true, "value": value }).to_string(),
        Err(e) => json!({
            "ok": false,
            "error": { "kind": e.kind.as_str(), "message": e.message, "method": method },
        })
        .to_string(),
    }
}

pub fn call(method: &str, args: &Value) -> Result<Value, String> {
    let direct =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| call_direct(method, args)));
    if let Ok(Some(value)) = direct {
        return Ok(value);
    }
    guarded_route(method, &args.to_string()).map_err(|e| match e.message.is_empty() {
        true => e.kind.as_str().to_owned(),
        false => format!("{}: {}", e.kind.as_str(), e.message),
    })
}

fn call_direct(method: &str, args: &Value) -> Option<Value> {
    match method {
        "homeHeroPlan" => Some(crate::home::ranking::home_hero_plan(args)),
        "normalizeLibraryDocument" => Some(crate::library::state::normalize_library_document(args)),
        "buildContinueWatchingFromProgress" => {
            crate::library::state::build_continue_watching_from_progress(args)
        }
        "mergeSearchSources" => crate::catalog::search::merge_search_sources(args),
        "mergeDiscoverSources" => crate::catalog::search::merge_discover_sources(args),
        _ => None,
    }
}

fn guarded_route(method: &str, args_json: &str) -> Outcome {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| route(method, args_json)))
        .unwrap_or_else(|_| Err(fail(ErrorKind::Internal, "internal panic")))
}

fn raw_dispatch(method: &str, args_json: &str) -> Result<String, CallError> {
    let args = object(args_json)?;
    let value = match method {
        "app.dispatchDelta" => app_state::app_core_dispatch_delta_json(
            field_u64(&args, "handle")?,
            &field(&args, "action")?.to_string(),
        ),
        "engine.dispatch" => headless_engine::headless_engine_dispatch_json(
            field_u64(&args, "handle")?,
            &field(&args, "action")?.to_string(),
        ),
        "engine.completeEffect" => headless_engine::headless_engine_complete_effect_json(
            field_u64(&args, "handle")?,
            &field(&args, "result")?.to_string(),
        ),
        _ => unreachable!(),
    }
    .ok_or_else(|| {
        fail(
            ErrorKind::NotFound,
            format!("`{method}` produced no result"),
        )
    })?;
    debug_assert!(serde_json::from_str::<&serde_json::value::RawValue>(&value).is_ok());
    Ok(value)
}

fn route(method: &str, args_json: &str) -> Outcome {
    match method {
        "subtitleCueList" => opt_json(player::subtitles::subtitle_sync::subtitle_cue_list_json(
            args_json,
        )),
        "subtitleSyncCapture" => {
            opt_json(player::subtitles::subtitle_sync::subtitle_sync_capture_json(args_json))
        }
        "subtitleSyncApply" => opt_json(
            player::subtitles::subtitle_sync::subtitle_sync_apply_json(args_json),
        ),
        _ => match methods::router_for(method) {
            Some(router) => router(method, args_json),
            None => Err(fail(
                ErrorKind::UnknownMethod,
                format!("no such method `{method}`"),
            )),
        },
    }
}
