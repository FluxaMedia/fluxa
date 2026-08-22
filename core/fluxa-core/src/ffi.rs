use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

mod addon_protocol_routes;
mod addon_resource_routes;
mod addon_support_routes;
mod anime_nuvio_routes;
mod calendar_routes;
mod content_identity_routes;
mod content_warning_routes;
mod core_addon_store_routes;
mod engine_routes;
mod external_sync_routes;
mod fluxa_sync_routes;
mod intro_plugins_routes;
mod library_routes;
mod local_media_routes;
mod mdblist_routes;
mod plan_misc_routes;
mod player_policy_routes;
mod player_scrobble_routes;
mod profile_routes;
mod publicmetadb_routes;
mod resource_plan_routes;
mod search_plan_routes;
mod stream_badge_routes;
mod stream_policy_routes;
mod tmdb_routes;
mod watchlist_offline_routes;
mod watch_together_routes;
use addon_protocol_routes::route_addon_protocol;
use addon_resource_routes::route_addon_resource;
use addon_support_routes::{route_addon_uptime, route_trailer_subtitles};
use anime_nuvio_routes::{route_anime_detection, route_nuvio_pin, route_nuvio_sync};
use calendar_routes::route_calendar;
use content_identity_routes::route_content_identity;
use content_warning_routes::route_content_warnings;
use core_addon_store_routes::{route_addon_store, route_core_contract, route_profile_avatar_pack};
use engine_routes::route_engine_lifecycle;
use fluxa_sync_routes::route_fluxa_sync;
use external_sync_routes::{
    route_external_sync_anilist, route_external_sync_simkl, route_external_sync_trakt,
};
use intro_plugins_routes::{route_intro_segments, route_plugins};
use library_routes::route_library_state;
use local_media_routes::route_local_media;
use mdblist_routes::route_mdblist;
#[cfg(feature = "dv-codec")]
use plan_misc_routes::route_dolby_vision_rpu;
use plan_misc_routes::{
    route_data_policy, route_device_resource, route_discovery_plan, route_headless_adapter_plan,
    route_player_flow,
};
use player_policy_routes::route_player_policy;
use player_scrobble_routes::route_player_scrobble;
use profile_routes::{route_profile_contract, route_profile_prefs};
use publicmetadb_routes::route_publicmetadb;
use resource_plan_routes::route_resource_plan;
use search_plan_routes::route_search_plan;
use stream_badge_routes::route_stream_badges;
use stream_policy_routes::route_stream_policy;
use tmdb_routes::route_tmdb;
use watchlist_offline_routes::{route_offline, route_watchlist};
use watch_together_routes::route_watch_together;

#[cfg(feature = "dv-codec")]
use crate::dolby_vision_rpu;
#[cfg(feature = "dv-codec")]
use crate::dolby_vision_sample;
use crate::{
    addon_protocol, addon_resource, addon_store, addon_uptime, anime_detection, app_state,
    calendar_plan, content_identity, content_warnings, core_contract, data_policy,
    desktop_playback, device_resource, discovery_plan, external_sync, fluxa_sync,
    headless_adapter_plan,
    headless_engine, home_ranking, integration_settings, intro_segments, library_persistence,
    library_state, mdblist_plan, nuvio_sync, offline_download, platform_plan, player_flow,
    player_policy, player_scrobble, plugins, profile_avatar_pack, profile_contract, profile_prefs,
    publicmetadb_plan, repository_flow, search_plan, stream_badges, stream_policy, tmdb_plan,
    trailer_subtitles, subtitle_sync, watchlist_plan,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    UnknownMethod,
    InvalidArgs,
    NotFound,
    Internal,
}

impl ErrorKind {
    fn as_str(self) -> &'static str {
        match self {
            ErrorKind::UnknownMethod => "unknown_method",
            ErrorKind::InvalidArgs => "invalid_args",
            ErrorKind::NotFound => "not_found",
            ErrorKind::Internal => "internal",
        }
    }
}

struct CallError {
    kind: ErrorKind,
    message: String,
}

fn fail(kind: ErrorKind, message: impl Into<String>) -> CallError {
    CallError {
        kind,
        message: message.into(),
    }
}

fn unknown_method() -> CallError {
    CallError {
        kind: ErrorKind::UnknownMethod,
        message: String::new(),
    }
}

type Outcome = Result<Value, CallError>;

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
    // A panic anywhere in route()/the domain modules must not take the host
    // process down with it — catch it here and hand back the same error
    // envelope shape callers already handle for any other failure.
    let outcome =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| route(method, args_json)));
    match outcome {
        Ok(Ok(value)) => json!({ "ok": true, "value": value }).to_string(),
        Ok(Err(e)) => json!({
            "ok": false,
            "error": { "kind": e.kind.as_str(), "message": e.message, "method": method },
        })
        .to_string(),
        Err(_) => json!({
            "ok": false,
            "error": { "kind": ErrorKind::Internal.as_str(), "message": "internal panic", "method": method },
        })
        .to_string(),
    }
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
    serde_json::from_str::<&serde_json::value::RawValue>(&value).map_err(|error| {
        fail(
            ErrorKind::Internal,
            format!("core produced invalid JSON: {error}"),
        )
    })?;
    Ok(value)
}

// Each route_* function owns one domain's method names. `route` tries them in
// turn and moves to the next as long as a function reports the method isn't
// one of its own (signaled by the UnknownMethod error its catch-all arm
// produces) — so every method is still handled by exactly one place, just
// grouped by domain instead of one 500+ line match.
const ROUTERS: &[fn(&str, &str) -> Outcome] = &[
    route_engine_lifecycle,
    route_addon_protocol,
    route_addon_uptime,
    route_addon_resource,
    route_resource_plan,
    route_stream_policy,
    route_stream_badges,
    route_search_plan,
    route_player_policy,
    route_watchlist,
    route_offline,
    route_content_identity,
    route_content_warnings,
    route_calendar,
    route_external_sync_trakt,
    route_external_sync_simkl,
    route_external_sync_anilist,
    route_mdblist,
    route_publicmetadb,
    route_anime_detection,
    route_library_state,
    route_local_media,
    route_nuvio_sync,
    route_nuvio_pin,
    route_tmdb,
    route_intro_segments,
    route_core_contract,
    route_plugins,
    route_addon_store,
    route_profile_avatar_pack,
    route_profile_contract,
    route_profile_prefs,
    route_headless_adapter_plan,
    route_discovery_plan,
    route_data_policy,
    route_device_resource,
    #[cfg(feature = "dv-codec")]
    route_dolby_vision_rpu,
    route_player_flow,
    route_player_scrobble,
    route_trailer_subtitles,
    route_watch_together,
    route_fluxa_sync,
];

static ROUTE_CACHE: OnceLock<Mutex<HashMap<String, Option<usize>>>> = OnceLock::new();

fn route(method: &str, args_json: &str) -> Outcome {
    match method {
        "engine.dispatch"
        | "engine.completeEffect"
        | "app.dispatch"
        | "app.dispatchDelta"
        | "streamPlaybackInfo"
        | "torrentRuntimeInfo"
        | "torrentStatusInfo"
        | "playerTrackState"
        | "curateHomeItems"
        | "prioritizeHomeRows"
        | "optimizeHomeRows" => {
            return match method {
                "engine.dispatch"
                | "engine.completeEffect"
                | "app.dispatch"
                | "app.dispatchDelta" => route_engine_lifecycle(method, args_json),
                "streamPlaybackInfo" | "torrentRuntimeInfo" | "torrentStatusInfo"
                | "playerTrackState" => route_stream_policy(method, args_json),
                _ => route_library_state(method, args_json),
            };
        }
        _ => {}
    }
    let cache = ROUTE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    match method {
        "subtitleCueList" => {
            return opt_json(subtitle_sync::subtitle_cue_list_json(args_json));
        }
        "subtitleSyncCapture" => {
            return opt_json(subtitle_sync::subtitle_sync_capture_json(args_json));
        }
        "subtitleSyncApply" => {
            return opt_json(subtitle_sync::subtitle_sync_apply_json(args_json));
        }
        _ => {}
    }
    if let Some(index) = cache
        .lock()
        .ok()
        .and_then(|routes| routes.get(method).copied())
    {
        return index
            .map(|index| ROUTERS[index](method, args_json))
            .unwrap_or_else(|| {
                Err(fail(
                    ErrorKind::UnknownMethod,
                    format!("no such method `{method}`"),
                ))
            });
    }
    for (index, router) in ROUTERS.iter().enumerate() {
        match router(method, args_json) {
            Err(CallError {
                kind: ErrorKind::UnknownMethod,
                ..
            }) => continue,
            result => {
                if let Ok(mut routes) = cache.lock() {
                    routes.insert(method.to_string(), Some(index));
                }
                return result;
            }
        }
    }
    if let Ok(mut routes) = cache.lock() {
        routes.insert(method.to_string(), None);
    }
    Err(fail(
        ErrorKind::UnknownMethod,
        format!("no such method `{method}`"),
    ))
}

fn opt_str(value: Option<String>) -> Outcome {
    Ok(value.map(Value::String).unwrap_or(Value::Null))
}

fn opt_json(value: Option<String>) -> Outcome {
    Ok(match value {
        Some(s) => serde_json::from_str(&s).map_err(|e| {
            fail(
                ErrorKind::Internal,
                format!("core produced invalid JSON: {e}"),
            )
        })?,
        None => Value::Null,
    })
}

fn object(args_json: &str) -> Result<Value, CallError> {
    let value: Value = serde_json::from_str(args_json).map_err(|e| {
        fail(
            ErrorKind::InvalidArgs,
            format!("args is not valid JSON: {e}"),
        )
    })?;
    if value.is_object() {
        Ok(value)
    } else {
        Err(fail(ErrorKind::InvalidArgs, "args must be a JSON object"))
    }
}

fn arg_str(args_json: &str, name: &str) -> Result<String, CallError> {
    let args = object(args_json)?;
    Ok(field_str(&args, name)?.to_string())
}

fn field<'a>(args: &'a Value, name: &str) -> Result<&'a Value, CallError> {
    args.get(name)
        .ok_or_else(|| fail(ErrorKind::InvalidArgs, format!("missing field `{name}`")))
}

fn field_str<'a>(args: &'a Value, name: &str) -> Result<&'a str, CallError> {
    field(args, name)?.as_str().ok_or_else(|| {
        fail(
            ErrorKind::InvalidArgs,
            format!("field `{name}` must be a string"),
        )
    })
}

fn field_u64(args: &Value, name: &str) -> Result<u64, CallError> {
    field(args, name)?.as_u64().ok_or_else(|| {
        fail(
            ErrorKind::InvalidArgs,
            format!("field `{name}` must be a non-negative integer"),
        )
    })
}

fn field_i64(args: &Value, name: &str) -> Result<i64, CallError> {
    field(args, name)?.as_i64().ok_or_else(|| {
        fail(
            ErrorKind::InvalidArgs,
            format!("field `{name}` must be an integer"),
        )
    })
}

fn handle(args_json: &str) -> Result<u64, CallError> {
    let value: Value = serde_json::from_str(args_json).map_err(|e| {
        fail(
            ErrorKind::InvalidArgs,
            format!("args is not valid JSON: {e}"),
        )
    })?;
    value
        .as_u64()
        .or_else(|| value.get("handle").and_then(Value::as_u64))
        .ok_or_else(|| {
            fail(
                ErrorKind::InvalidArgs,
                "expected a handle (number or { handle })",
            )
        })
}

fn result_json(value: Option<String>, method: &str) -> Outcome {
    match value {
        Some(s) => into_json(s),
        None => Err(fail(
            ErrorKind::NotFound,
            format!("`{method}` produced no result"),
        )),
    }
}

fn into_json(s: String) -> Outcome {
    serde_json::from_str(&s).map_err(|e| {
        fail(
            ErrorKind::Internal,
            format!("core produced invalid JSON: {e}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Value {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn unknown_method_reports_kind_and_name() {
        let env = parse(&core_invoke("nope.doesNotExist", "{}"));
        assert_eq!(env["ok"], json!(false));
        assert_eq!(env["error"]["kind"], json!("unknown_method"));
        assert_eq!(env["error"]["method"], json!("nope.doesNotExist"));
    }

    #[test]
    fn invalid_args_distinguished_from_empty_result() {
        let bad_json = parse(&core_invoke("identity", "{ not json"));
        assert_eq!(bad_json["error"]["kind"], json!("invalid_args"));

        let missing_field = parse(&core_invoke("identity", "{}"));
        assert_eq!(missing_field["error"]["kind"], json!("invalid_args"));
    }

    #[test]
    fn stateless_helper_returns_ok_value() {
        let env = parse(&core_invoke("parseVideoId", r#"{"id":"tt123:1:2"}"#));
        assert_eq!(env["ok"], json!(true));
        assert_eq!(env["value"]["imdb"], json!("tt123"));
        assert_eq!(env["value"]["isEpisode"], json!(true));
    }

    #[test]
    fn new_sync_and_detection_methods_are_routed() {
        let detect = parse(&core_invoke(
            "detectAnimePlayback",
            r#"{"meta":{"genres":["Anime"]},"episode":null,"stream":null,"addons":[]}"#,
        ));
        assert_eq!(detect["ok"], json!(true));
        assert_eq!(detect["value"]["confidence"], json!(65));

        let sync = parse(&core_invoke(
            "anilistEntriesToSync",
            r#"{"entries":[],"nowMs":0}"#,
        ));
        assert_eq!(sync["ok"], json!(true));
        assert_eq!(sync["value"]["watchlist"], json!([]));

        let merged = parse(&core_invoke(
            "mergeLibraryItemsById",
            r#"{"local":[],"incoming":[{"id":"a"}]}"#,
        ));
        assert_eq!(merged["value"][0]["id"], json!("a"));

        let plan = parse(&core_invoke(
            "tmdbPeopleRequestPlan",
            r#"{"meta":{"id":"tt123","type":"movie"},"apiKey":"k","language":"en"}"#,
        ));
        assert_eq!(
            plan["value"]["findUrl"],
            json!(
                "https://api.themoviedb.org/3/find/tt123?api_key=k&language=en-US&external_source=imdb_id"
            )
        );

        let images = parse(&core_invoke(
            "tmdbPeopleImagesFromCredits",
            r#"{"credits":{"cast":[{"name":"Jane Doe","profile_path":"/x.jpg"}]},"links":[{"name":"jane  doe"}]}"#,
        ));
        assert_eq!(
            images["value"]["jane  doe"],
            json!("https://image.tmdb.org/t/p/w185/x.jpg")
        );
    }

    #[test]
    fn engine_roundtrips_through_the_funnel() {
        let created = parse(&core_invoke("engine.create", "{}"));
        let h = created["value"].as_i64().unwrap();
        assert!(h > 0);

        let snap = parse(&core_invoke("engine.snapshot", &h.to_string()));
        assert_eq!(snap["ok"], json!(true));

        let destroyed = parse(&core_invoke("engine.destroy", &h.to_string()));
        assert_eq!(destroyed["ok"], json!(true));
        assert_eq!(destroyed["value"], json!(true));
    }

    #[test]
    fn lifecycle_create_rejects_malformed_initial_state() {
        for method in ["engine.create", "app.create"] {
            let result = parse(&core_invoke(method, "{ malformed"));
            assert_eq!(result["ok"], json!(false));
            assert_eq!(result["error"]["kind"], json!("invalid_args"));
        }
    }

    #[test]
    fn app_delta_keeps_the_same_wire_value_without_reparsing() {
        let created = parse(&core_invoke("app.create", "{}"));
        let handle = created["value"].as_i64().unwrap();
        let delta = parse(&core_invoke(
            "app.dispatchDelta",
            &format!(r#"{{"handle":{handle},"action":{{"type":"setHomeLoading","value":true}}}}"#),
        ));
        assert_eq!(delta["ok"], json!(true));
        assert_eq!(delta["value"]["patch"]["home"]["isLoading"], json!(true));
        assert_eq!(
            parse(&core_invoke("app.destroy", &handle.to_string()))["value"],
            json!(true)
        );
    }

    #[test]
    fn calendar_plan_methods_route_and_compute() {
        let candidates = parse(&core_invoke(
            "calendarSeasonCandidates",
            r#"{"seasonsCount":10,"lastVideoId":"tt1:2:3"}"#,
        ));
        assert_eq!(candidates["ok"], json!(true));
        assert_eq!(candidates["value"], json!([2, 3, 10]));

        let rows = parse(&core_invoke(
            "calendarWidgetRows",
            r#"{"items":[{"dateIso":"2026-07-18","title":"Show","seasonNumber":1,"episodeNumber":2}],"maxRows":4}"#,
        ));
        assert_eq!(rows["value"][0]["episodeText"], json!("S1E2"));

        let content = parse(&core_invoke(
            "calendarContentPlan",
            r#"{"items":[{"dateIso":"2026-07-18","metaId":"tt1","title":"Show"}],"monthPrefix":"2026-07"}"#,
        ));
        assert_eq!(content["value"][0]["metaId"], json!("tt1"));

        let notifications = parse(&core_invoke(
            "calendarNotificationContent",
            r#"{"items":[{"dateIso":"2026-07-18","metaId":"tt1","metaType":"series","title":"Show","seasonNumber":1,"episodeNumber":1}],"todayIso":"2026-07-18","alreadyNotifiedKeys":[]}"#,
        ));
        assert_eq!(
            notifications["value"]["items"][0]["titleKey"],
            json!("notification.new_season_released")
        );
        assert_eq!(notifications["value"]["keys"].as_array().unwrap().len(), 1);

        let released = parse(&core_invoke(
            "calendarReleaseDetection",
            r#"{"items":[{"dateIso":"2026-07-18"},{"dateIso":"2026-07-19"}],"todayIso":"2026-07-18"}"#,
        ));
        assert_eq!(released["value"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn newly_routed_modules_compute() {
        let input_type = parse(&core_invoke(
            "addonStoreInputType",
            r#"{"input":"https://example.com/addon/manifest.json"}"#,
        ));
        assert_eq!(input_type["value"], json!("stremio_manifest"));

        let secure = parse(&core_invoke(
            "isSecureRemoteUrl",
            r#"{"url":"http://example.com"}"#,
        ));
        assert_eq!(secure["value"], json!(false));

        let same = parse(&core_invoke(
            "samePluginRepositoryUrl",
            r#"{"left":"https://Example.com/repo/","right":"http://example.com/repo"}"#,
        ));
        assert_eq!(same["value"], json!(true));

        let buffer = parse(&core_invoke("safePlayerBufferCacheMb", r#"{"value":50}"#));
        assert_eq!(buffer["value"], json!(100));

        let dv_mode = parse(&core_invoke(
            "safeDolbyVisionFallbackMode",
            r#"{"mode":"dv8"}"#,
        ));
        assert_eq!(dv_mode["value"], json!("dv8"));

        let source_mode = parse(&core_invoke(
            "safeStreamSourceSelectionMode",
            r#"{"mode":"regex"}"#,
        ));
        assert_eq!(source_mode["value"], json!("regex"));

        let policy = parse(&core_invoke("directPlaybackPolicy", "{}"));
        assert_eq!(policy["value"]["metaDetailTimeoutMs"], json!(3500));

        let prefix = parse(&core_invoke(
            "streamDiscoveryCachePrefix",
            r#"{"contentType":"movie","id":"tt1","language":"en"}"#,
        ));
        assert_eq!(prefix["value"], json!("movie|tt1|en"));
    }

    #[test]
    fn gap_filled_routes_compute() {
        let bearer = parse(&core_invoke("traktBearer", r#"{"token":"abc"}"#));
        assert_eq!(bearer["value"], json!("Bearer abc"));

        let has_client = parse(&core_invoke("traktHasClient", r#"{"apiKey":""}"#));
        assert_eq!(has_client["value"], json!(false));

        let expires_at = parse(&core_invoke(
            "traktTokenExpiresAt",
            r#"{"createdAtSeconds":1000,"expiresInSeconds":7200}"#,
        ));
        assert_eq!(expires_at["value"], json!(1000 + 6900));

        let show_id = parse(&core_invoke(
            "traktShowIdFromEpisodeId",
            r#"{"videoId":"tt1:2:3"}"#,
        ));
        assert_eq!(show_id["value"], json!("tt1"));

        let episode_matches = parse(&core_invoke(
            "episodeTextMatches",
            r#"{"text":"Show S01E02","season":1,"episode":2}"#,
        ));
        assert_eq!(episode_matches["value"], json!(true));

        let stream_matches = parse(&core_invoke(
            "streamMatchesEpisode",
            r#"{"videoId":"tt1:1:2","title":"","name":"","description":"","filename":"Show.S01E02.mkv","effectiveFilename":""}"#,
        ));
        assert_eq!(stream_matches["value"], json!(true));

        let content_type = parse(&core_invoke("normalizeContentType", r#"{"value":"tv"}"#));
        assert_eq!(content_type["value"], json!("series"));

        let feed_part = parse(&core_invoke("stableFeedPart", r#"{"value":"Foo Bar!"}"#));
        assert_eq!(feed_part["value"], json!("foo_bar"));

        let base = parse(&core_invoke(
            "baseUrl",
            r#"{"url":"https://example.com/addon/manifest.json"}"#,
        ));
        assert_eq!(base["value"], json!("https://example.com/addon/"));

        let progress = parse(&core_invoke(
            "playerProgressPercent",
            r#"{"positionMs":50,"durationMs":100}"#,
        ));
        assert_eq!(progress["value"], json!(50.0));

        let should_save = parse(&core_invoke(
            "playerShouldSaveOnDispose",
            r#"{"positionMs":6000}"#,
        ));
        assert_eq!(should_save["value"], json!(true));

        let category_json =
            r#"{\"id\":\"a\",\"name\":\"A\",\"type\":\"movie\",\"items\":[{\"id\":\"tt1\"}]}"#;
        let overlap = parse(&core_invoke(
            "homeOverlapRatio",
            &format!(r#"{{"firstJson":"{category_json}","secondJson":"{category_json}"}}"#),
        ));
        assert_eq!(overlap["value"], json!(1.0));

        let select = parse(&core_invoke(
            "selectStreamIndex",
            r#"{"streamsJson":"[]","currentVideoId":"tt1","initialStreamIndex":0,"sourceSelectionMode":"manual"}"#,
        ));
        assert_eq!(select["value"], json!(-1));

        let ids = parse(&core_invoke(
            "streamRequestIds",
            r#"{"contentType":"movie","id":"tt1"}"#,
        ));
        assert_eq!(ids["value"], json!(["tt1"]));
    }

    #[test]
    fn last_gap_filled_routes_compute() {
        let locator = parse(&core_invoke(
            "parseEpisodeLocator",
            r#"{"input":"tt1:2:3"}"#,
        ));
        assert_eq!(locator["value"]["baseId"], json!("tt1"));
        assert_eq!(locator["value"]["season"], json!(2));
        assert_eq!(locator["value"]["episode"], json!(3));

        let no_locator = parse(&core_invoke("parseEpisodeLocator", r#"{"input":"nope"}"#));
        assert_eq!(no_locator["value"], Value::Null);

        let audio = parse(&core_invoke(
            "resolvePreferredAudioLanguage",
            r#"{"lastAudioLanguage":null,"preferredAudioLanguage":"en","originalLanguage":"ja"}"#,
        ));
        assert_eq!(audio["value"], json!("ja"));

        let subtitle_match = parse(&core_invoke(
            "subtitleLanguageMatches",
            r#"{"label":"english","language":null,"preferredLanguage":"en"}"#,
        ));
        assert_eq!(subtitle_match["value"], json!(true));

        let toggled = parse(&core_invoke(
            "toggleMetadataFeed",
            r#"{"selectedKeys":"[]","availableKeys":"[\"a\"]","key":"a"}"#,
        ));
        assert_eq!(toggled["value"], json!(["a"]));

        let manifest_request = json!({
            "body": json!({"resources": ["catalog"], "types": ["movie"]}).to_string(),
            "transportUrl": "https://example.com/manifest.json",
            "unknownName": "Unknown Addon"
        });
        let manifest = parse(&core_invoke("parseManifest", &manifest_request.to_string()));
        assert_eq!(manifest["ok"], json!(true));
        assert_eq!(
            manifest["value"]["manifest"]["name"],
            json!("Unknown Addon")
        );
    }

    // tests/wire/core_invoke_methods.txt is a checked-in list of every method
    // name core_invoke routes. It exists so renaming or removing one shows up
    // as a failure in this repo (a diff in this fixture is the review
    // artifact for an intentional rename) instead of as a runtime
    // "no such method" discovered on a platform we can't see from here. This
    // doesn't verify each method's business logic — just that the name is
    // still recognized rather than falling through every router to
    // UnknownMethod.
    #[test]
    fn every_known_core_invoke_method_still_routes() {
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/wire/core_invoke_methods.txt");
        let methods = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|_| panic!("missing fixture {fixture_path:?}"));
        let methods: Vec<&str> = methods
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        assert!(!methods.is_empty(), "fixture list must not be empty");

        for method in methods {
            let result = parse(&core_invoke(method, "{}"));
            let kind = result["error"]["kind"].as_str().unwrap_or("");
            assert_ne!(
                kind, "unknown_method",
                "{method} no longer routes anywhere — renamed or removed?"
            );
        }
    }
}
