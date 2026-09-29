use crate::headless_engine::detail;
use crate::headless_engine::helpers::{
    active_profile_id, normalize_error, should_sync_watched_state,
};
use crate::headless_engine::home;
use crate::headless_engine::profile;
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

mod complete;
mod dispatch;

pub(crate) use complete::*;
pub(crate) use dispatch::*;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct LibraryState {
    active_profile_id: String,
    is_loading: bool,
    watchlist: Value,
    continue_watching: Value,
    liked: Value,
    watched: Value,
    dropped: Value,
    on_hold: Value,
    completed: Value,
    last_command: Value,
    last_write: Value,
    last_write_error: Value,
    pending_playback_progress: Value,
    saved_playback_progress: Value,
    last_watched_sync: Value,
    last_watched_sync_error: Value,
    error: Value,
    generation: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadLibraryStatePayload {
    profile_id: String,
    source: String,
    profile: Value,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToggleWatchlistCommand {
    #[serde(rename = "type")]
    kind: &'static str,
    item: Value,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToggleLibraryStatusCommand {
    #[serde(rename = "type")]
    kind: &'static str,
    list: String,
    item: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WriteLibraryCommandPayload {
    profile_id: String,
    source: String,
    command: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WriteFeedbackPayload {
    id: String,
    value: Option<bool>,
    meta: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClearPlaybackProgressPayload {
    profile: Value,
    meta: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackProgress {
    meta: Value,
    time_offset: i64,
    duration: i64,
    last_video_id: Option<String>,
    last_stream_index: Option<i32>,
    last_episode_name: Option<String>,
    last_episode_season: Option<i64>,
    last_episode_number: Option<i64>,
    last_episode_thumbnail: Option<String>,
    last_stream_url: Option<String>,
    last_stream_title: Option<String>,
    last_audio_language: Option<String>,
    last_subtitle_language: Option<String>,
    source: &'static str,
    refresh_external_continue_watching: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WritePlaybackProgressPayload {
    profile_id: String,
    profile: Value,
    scrobble_trakt_pause: bool,
    progress: PlaybackProgress,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MarkWatchedCommand {
    #[serde(rename = "type")]
    kind: &'static str,
    series_id: String,
    video_ids: Vec<String>,
    watched: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    rewatch: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    addon_episodes: Vec<Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncWatchedStatePayload {
    profile: Value,
    meta: Value,
    episodes: Vec<Value>,
    watched: bool,
}
