use crate::headless_engine::helpers::{
    active_profile_id, normalize_error, normalize_meta_trailers, value_array_is_empty,
    visible_streams,
};
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct DetailState {
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
    #[serde(skip_serializing_if = "Value::is_null")]
    provider_watched_video_ids: Value,
    #[serde(skip_serializing_if = "Value::is_null")]
    trakt_seasons: Value,
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
            provider_watched_video_ids: Value::Null,
            trakt_seasons: Value::Null,
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
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct LookupState {
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

pub(crate) fn set_is_in_watchlist(engine: &mut HeadlessEngine, value: Value) {
    engine.state.detail.is_in_watchlist = value;
}

pub(crate) fn set_local_watched_video_ids(engine: &mut HeadlessEngine, value: Value) {
    engine.state.detail.local_watched_video_ids = value;
    engine.state.detail.provider_watched_video_ids = Value::Null;
    engine.state.detail.trakt_seasons = Value::Null;
}

fn remap_local_watched(engine: &mut HeadlessEngine) {
    let detail = &mut engine.state.detail;
    if detail.trakt_seasons.is_null() {
        return;
    }
    let episodes = crate::headless_engine::library::addon_episodes(Some(&detail.meta));
    if episodes.is_empty() {
        return;
    }
    let args = serde_json::json!({
        "direction": "toAddon",
        "videoIds": detail.provider_watched_video_ids,
        "addonEpisodes": episodes,
        "traktSeasons": detail.trakt_seasons,
    });
    if let Some(mapped) = crate::services::trakt::trakt_remap_video_ids_json(&args.to_string())
        .and_then(|mapped| serde_json::from_str(&mapped).ok())
    {
        detail.local_watched_video_ids = mapped;
    }
}

pub(crate) fn set_feedback(engine: &mut HeadlessEngine, value: Value) {
    engine.state.detail.feedback = value;
}

pub(crate) fn clear_saved_playback(engine: &mut HeadlessEngine) {
    engine.state.detail.saved_playback = Value::Null;
}

mod complete;
mod dispatch;

pub(crate) use complete::complete;
pub(crate) use dispatch::*;
