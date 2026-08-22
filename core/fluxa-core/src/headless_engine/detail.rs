use super::helpers::{
    active_profile_id, normalize_error, normalize_meta_trailers, value_array_is_empty,
    visible_streams,
};
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct DetailState {
    content_type: String,
    id: String,
    language: String,
    profile: Value,
    is_loading: bool,
    is_loading_streams: bool,
    meta: Value,
    streams: Value,
    visible_streams: Value,
    selected_addon: Value,
    available_addons: Value,
    loading_addon_names: Value,
    season_episodes: Value,
    season_loading: Value,
    saved_playback: Value,
    watched_video_ids: Value,
    local_watched_video_ids: Value,
    is_in_watchlist: Value,
    feedback: Value,
    user_addons: Value,
    similar_items: Value,
    trailers: Value,
    omdb_ratings: Value,
    mdblist_ratings: Value,
    fanart_artwork: Value,
    has_stream_providers: Value,
    last_prefetch: Value,
    last_prefetch_error: Value,
    resolved_request_id: Value,
    streams_error: Value,
    failed_addons: Value,
    error: Value,
    generation: u64,
    #[serde(skip)]
    streams_request_key: String,
}

impl Default for DetailState {
    fn default() -> Self {
        Self {
            content_type: String::new(),
            id: String::new(),
            language: "en".to_string(),
            profile: Value::Null,
            is_loading: false,
            is_loading_streams: false,
            meta: Value::Null,
            streams: serde_json::json!([]),
            visible_streams: serde_json::json!([]),
            selected_addon: Value::Null,
            available_addons: serde_json::json!([]),
            loading_addon_names: serde_json::json!([]),
            season_episodes: serde_json::json!([]),
            season_loading: Value::Null,
            saved_playback: Value::Null,
            watched_video_ids: serde_json::json!([]),
            local_watched_video_ids: serde_json::json!([]),
            is_in_watchlist: Value::Null,
            feedback: Value::Null,
            user_addons: serde_json::json!([]),
            similar_items: serde_json::json!([]),
            trailers: serde_json::json!([]),
            omdb_ratings: Value::Null,
            mdblist_ratings: Value::Null,
            fanart_artwork: Value::Null,
            has_stream_providers: Value::Null,
            last_prefetch: Value::Null,
            last_prefetch_error: Value::Null,
            resolved_request_id: Value::Null,
            streams_error: Value::Null,
            failed_addons: serde_json::json!([]),
            error: Value::Null,
            generation: 0,
            streams_request_key: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct LookupState {
    trailers: Value,
    meta_detail: Value,
    error: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchMetaDetailPayload {
    content_type: String,
    id: String,
    language: String,
    source_addon_transport_url: String,
    source_addon_catalog_type: String,
    profile: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadPlaybackProgressPayload {
    id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadDetailLocalStatePayload {
    primary_id: String,
    fallback_id: Option<String>,
    content_type: String,
    profile: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchDetailSecondaryPayload {
    content_type: String,
    id: String,
    language: String,
    profile: Value,
    similar_titles_source: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrefetchDetailStreamsPayload {
    content_type: String,
    id: String,
    stream_lookup_id: String,
    title: String,
    original_name: Option<String>,
    year: Option<i32>,
    language: String,
    profile: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchDetailStreamsPayload {
    content_type: String,
    request_ids: Vec<String>,
    detail: Value,
    season_episodes: Vec<Value>,
    language: String,
    profile: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchMetaDetailLookupPayload {
    content_type: String,
    id: String,
    language: String,
    profile: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchSeasonEpisodesPayload {
    series_id: String,
    season: i32,
    profile_id: String,
    profile: Value,
    language: String,
}

pub(super) fn set_is_in_watchlist(engine: &mut HeadlessEngine, value: Value) {
    engine.state.detail.is_in_watchlist = value;
}

pub(super) fn set_local_watched_video_ids(engine: &mut HeadlessEngine, value: Value) {
    engine.state.detail.local_watched_video_ids = value;
}

pub(super) fn set_feedback(engine: &mut HeadlessEngine, value: Value) {
    engine.state.detail.feedback = value;
}

pub(super) fn clear_saved_playback(engine: &mut HeadlessEngine) {
    engine.state.detail.saved_playback = Value::Null;
}

pub(super) fn dispatch_load(
    engine: &mut HeadlessEngine,
    content_type: String,
    id: String,
    language: Option<String>,
    source_addon_transport_url: Option<String>,
    source_addon_catalog_type: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Detail);
    let language = language.unwrap_or_else(|| "en".to_string());
    *engine.state.detail = DetailState {
        content_type: content_type.clone(),
        id: id.clone(),
        language: language.clone(),
        profile: profile.clone().unwrap_or(Value::Null),
        is_loading: true,
        generation,
        ..DetailState::default()
    };
    vec![
        engine.effect(
            EffectKind::FetchMetaDetail,
            generation,
            FetchMetaDetailPayload {
                content_type,
                id: id.clone(),
                language,
                source_addon_transport_url: source_addon_transport_url.unwrap_or_default(),
                source_addon_catalog_type: source_addon_catalog_type.unwrap_or_default(),
                profile: profile.unwrap_or(Value::Null),
            },
        ),
        engine.effect(
            EffectKind::ReadPlaybackProgress,
            generation,
            ReadPlaybackProgressPayload { id },
        ),
    ]
}

pub(super) fn dispatch_local_state(
    engine: &mut HeadlessEngine,
    primary_id: String,
    fallback_id: Option<String>,
    content_type: String,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.runtime.get(GenerationKey::Detail);
    vec![engine.effect(
        EffectKind::ReadDetailLocalState,
        generation,
        ReadDetailLocalStatePayload {
            primary_id,
            fallback_id,
            content_type,
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

pub(super) fn dispatch_secondary(
    engine: &mut HeadlessEngine,
    content_type: String,
    id: String,
    language: Option<String>,
    profile: Option<Value>,
    similar_titles_source: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.runtime.get(GenerationKey::Detail);
    vec![engine.effect(
        EffectKind::FetchDetailSecondary,
        generation,
        FetchDetailSecondaryPayload {
            content_type,
            id,
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
            similar_titles_source,
        },
    )]
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_prefetch(
    engine: &mut HeadlessEngine,
    content_type: String,
    id: String,
    stream_lookup_id: String,
    title: Option<String>,
    original_name: Option<String>,
    year: Option<i32>,
    language: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.runtime.get(GenerationKey::Detail);
    vec![engine.effect(
        EffectKind::PrefetchDetailStreams,
        generation,
        PrefetchDetailStreamsPayload {
            content_type,
            id,
            stream_lookup_id,
            title: title.unwrap_or_default(),
            original_name,
            year,
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

pub(super) fn dispatch_streams(
    engine: &mut HeadlessEngine,
    content_type: String,
    request_ids: Vec<String>,
    detail: Option<Value>,
    season_episodes: Option<Vec<Value>>,
    language: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let request_key = format!("{content_type}:{}", request_ids.join(","));
    if engine.state.detail.is_loading_streams
        && engine.state.detail.streams_request_key == request_key
    {
        // Same episode already in flight — ignore the redundant request instead of
        // bumping the generation, which would orphan any addon results still on the way.
        return vec![];
    }
    engine.state.detail.streams_request_key = request_key;
    let generation = engine.bump_generation(GenerationKey::DetailStreams);
    engine.state.detail.is_loading_streams = true;
    engine.state.detail.streams = serde_json::json!([]);
    engine.state.detail.visible_streams = serde_json::json!([]);
    engine.state.detail.selected_addon = Value::Null;
    engine.state.detail.available_addons = serde_json::json!([]);
    engine.state.detail.loading_addon_names = serde_json::json!([]);
    engine.state.detail.failed_addons = serde_json::json!([]);
    vec![engine.effect(
        EffectKind::FetchDetailStreams,
        generation,
        FetchDetailStreamsPayload {
            content_type,
            request_ids,
            detail: detail.unwrap_or(Value::Null),
            season_episodes: season_episodes.unwrap_or_default(),
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

pub(super) fn dispatch_streams_appended(
    engine: &mut HeadlessEngine,
    streams: Vec<Value>,
    available_addons: Vec<String>,
    generation: Option<u64>,
) -> Vec<EffectEnvelope> {
    if !engine.state.detail.is_loading_streams {
        return vec![];
    }
    if let Some(generation) = generation
        && generation != engine.state.runtime.get(GenerationKey::DetailStreams)
    {
        return vec![];
    }
    let mut merged: Vec<Value> = engine
        .state
        .detail
        .streams
        .as_array()
        .cloned()
        .unwrap_or_default();
    merged.extend(streams);
    engine.state.detail.streams = serde_json::json!(merged);
    engine.state.detail.visible_streams = visible_streams(
        &engine.state.detail.streams,
        engine.state.detail.selected_addon.as_str(),
    );
    let mut all_addons: Vec<String> = engine
        .state
        .detail
        .available_addons
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    for addon in available_addons {
        if !all_addons.contains(&addon) {
            all_addons.push(addon);
        }
    }
    engine.state.detail.available_addons = serde_json::json!(all_addons);
    engine.state.detail.has_stream_providers =
        Value::Bool(!value_array_is_empty(&engine.state.detail.streams));
    vec![]
}

pub(super) fn dispatch_selected_addon_changed(
    engine: &mut HeadlessEngine,
    addon: Option<String>,
) -> Vec<EffectEnvelope> {
    let selected = addon.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    engine.state.detail.selected_addon = selected
        .as_ref()
        .map(|value| Value::String(value.clone()))
        .unwrap_or(Value::Null);
    engine.state.detail.visible_streams =
        visible_streams(&engine.state.detail.streams, selected.as_deref());
    vec![]
}

pub(super) fn dispatch_meta_detail(
    engine: &mut HeadlessEngine,
    content_type: String,
    id: String,
    language: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Lookup);
    vec![engine.effect(
        EffectKind::FetchMetaDetailLookup,
        generation,
        FetchMetaDetailLookupPayload {
            content_type,
            id,
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

pub(super) fn dispatch_season(
    engine: &mut HeadlessEngine,
    series_id: String,
    season: i32,
    profile: Option<Value>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.runtime.get(GenerationKey::Detail);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    engine.state.detail.season_loading = Value::from(season);
    vec![engine.effect(
        EffectKind::FetchSeasonEpisodes,
        generation,
        FetchSeasonEpisodesPayload {
            series_id,
            season: season.max(0),
            profile_id,
            profile: profile_value,
            language: language.unwrap_or_else(|| "en".to_string()),
        },
    )]
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "fetchMetaDetail" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                engine.state.detail.is_loading = false;
                if result.status.is_ok() {
                    let meta = result.value.get("meta").cloned().unwrap_or(Value::Null);
                    engine.state.detail.trailers = normalize_meta_trailers(&meta);
                    engine.state.detail.mdblist_ratings = result
                        .value
                        .get("mdblistRatings")
                        .cloned()
                        .unwrap_or(Value::Null);
                    engine.state.detail.meta = meta;
                    engine.state.detail.error = Value::Null;
                } else {
                    engine.state.detail.error = normalize_error(result.error.clone());
                }
            }
        }
        "readPlaybackProgress" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                engine.state.detail.saved_playback = if result.status.is_ok() {
                    result.value.clone()
                } else {
                    Value::Null
                };
            }
        }
        "readDetailLocalState" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                if result.status.is_ok() {
                    engine.state.detail.saved_playback = result
                        .value
                        .get("savedPlayback")
                        .cloned()
                        .unwrap_or(Value::Null);
                    engine.state.detail.local_watched_video_ids = result
                        .value
                        .get("localWatchedVideoIds")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.is_in_watchlist = result
                        .value
                        .get("isInWatchlist")
                        .cloned()
                        .unwrap_or(Value::Bool(false));
                    engine.state.detail.feedback =
                        result.value.get("feedback").cloned().unwrap_or(Value::Null);
                    engine.state.detail.has_stream_providers = result
                        .value
                        .get("hasStreamProviders")
                        .cloned()
                        .unwrap_or(Value::Bool(false));
                    engine.state.detail.user_addons = result
                        .value
                        .get("userAddons")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                } else {
                    engine.state.detail.error = normalize_error(result.error.clone());
                }
            }
        }
        "fetchDetailSecondary" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                if result.status.is_ok() {
                    engine.state.detail.watched_video_ids = result
                        .value
                        .get("watchedVideoIds")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.similar_items = result
                        .value
                        .get("similarItems")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    if value_array_is_empty(&engine.state.detail.trailers) {
                        engine.state.detail.trailers = result
                            .value
                            .get("trailers")
                            .cloned()
                            .unwrap_or_else(|| serde_json::json!([]));
                    }
                    engine.state.detail.omdb_ratings = result
                        .value
                        .get("omdbRatings")
                        .cloned()
                        .unwrap_or(Value::Null);
                    engine.state.detail.fanart_artwork = result
                        .value
                        .get("fanartArtwork")
                        .cloned()
                        .unwrap_or(Value::Null);
                } else {
                    engine.state.detail.error = normalize_error(result.error.clone());
                }
            }
        }
        "prefetchDetailStreams" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                if result.status.is_ok() {
                    engine.state.detail.last_prefetch = result.value.clone();
                } else {
                    engine.state.detail.last_prefetch_error = normalize_error(result.error.clone());
                }
            }
        }
        "fetchDetailStreams" => {
            if generation == engine.state.runtime.get(GenerationKey::DetailStreams) {
                engine.state.detail.is_loading_streams = false;
                if result.status.is_ok() {
                    engine.state.detail.streams = result
                        .value
                        .get("streams")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.visible_streams = visible_streams(
                        &engine.state.detail.streams,
                        engine.state.detail.selected_addon.as_str(),
                    );
                    engine.state.detail.available_addons = result
                        .value
                        .get("availableAddons")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.loading_addon_names = serde_json::json!([]);
                    engine.state.detail.failed_addons = result
                        .value
                        .get("failedAddons")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.resolved_request_id = result
                        .value
                        .get("resolvedRequestId")
                        .cloned()
                        .unwrap_or(Value::Null);
                    engine.state.detail.has_stream_providers = result
                        .value
                        .get("hasStreamProviders")
                        .cloned()
                        .unwrap_or(Value::Bool(false));
                    engine.state.detail.streams_error = Value::Null;
                } else {
                    engine.state.detail.streams_error = normalize_error(result.error.clone());
                    engine.state.detail.loading_addon_names = serde_json::json!([]);
                }
            }
        }
        "fetchMetaDetailLookup" => {
            if generation == engine.state.runtime.get(GenerationKey::Lookup) {
                if result.status.is_ok() {
                    *engine.state.lookup = LookupState {
                        trailers: normalize_meta_trailers(&result.value),
                        meta_detail: result.value.clone(),
                        error: Value::Null,
                    };
                } else {
                    *engine.state.lookup = LookupState {
                        trailers: serde_json::json!([]),
                        meta_detail: Value::Null,
                        error: normalize_error(result.error.clone()),
                    };
                }
            }
        }
        "fetchSeasonEpisodes" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                engine.state.detail.season_loading = Value::Null;
                if result.status.is_ok() {
                    engine.state.detail.season_episodes = result
                        .value
                        .get("episodes")
                        .cloned()
                        .unwrap_or_else(|| result.value.clone());
                    engine.state.detail.error = Value::Null;
                } else {
                    engine.state.detail.error = normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
