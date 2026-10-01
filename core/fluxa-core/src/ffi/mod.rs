use serde_json::{Value, json};

mod args;
mod errors;
mod methods;
#[cfg(test)]
mod tests;
use crate::services::{
    mdblist::route_mdblist, mediaserver::routes::route_mediaserver,
    nuvio::route_nuvio, publicmetadb::route_publicmetadb, simkl::route_simkl, tmdb::route_tmdb,
    tracking::external_sync::route_external_sync, trakt::route_trakt,
};
use crate::{
    addons::routes::*, catalog::routes::*, library::routes::*,
    player::routes::*, profile::routes::*, services::provider_routes::*, settings::routes::*,
};

use crate::player;

pub(crate) use args::*;
pub use errors::ErrorKind;
pub(crate) use errors::{CallError, Outcome, fail, unknown_method};

pub fn core_invoke(method: &str, args_json: &str) -> String {
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
