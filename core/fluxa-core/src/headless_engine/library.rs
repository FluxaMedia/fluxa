use super::detail;
use super::helpers::{active_profile_id, normalize_error, should_sync_watched_state};
use super::home;
use super::profile;
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct LibraryState {
    active_profile_id: String,
    is_loading: bool,
    watchlist: Value,
    continue_watching: Value,
    liked: Value,
    watched: Value,
    dropped: Value,
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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncWatchedStatePayload {
    profile: Value,
    meta: Value,
    episodes: Vec<Value>,
    watched: bool,
}

pub(super) fn set_active_profile_id(engine: &mut HeadlessEngine, id: &str) {
    engine.state.library.active_profile_id = id.to_string();
}

pub(super) fn dispatch_profile_activated(
    engine: &mut HeadlessEngine,
    profile: crate::types::Profile,
) -> Vec<EffectEnvelope> {
    profile::activate(engine, profile);
    vec![]
}

pub(super) fn dispatch_hydrate(
    engine: &mut HeadlessEngine,
    profile_id: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Library);
    let resolved_profile_id =
        profile_id.unwrap_or_else(|| active_profile_id(&engine.state, &Value::Null));
    engine.state.library.active_profile_id = resolved_profile_id.clone();
    engine.state.library.is_loading = true;
    engine.state.library.error = Value::Null;
    engine.state.library.generation = generation;
    vec![engine.effect(
        EffectKind::ReadLibraryState,
        generation,
        ReadLibraryStatePayload {
            profile_id: resolved_profile_id,
        },
    )]
}

pub(super) fn dispatch_toggle_watchlist(
    engine: &mut HeadlessEngine,
    item: Value,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Library);
    let profile_id = active_profile_id(&engine.state, &profile.unwrap_or(Value::Null));
    let command = ToggleWatchlistCommand {
        kind: "toggleWatchlist",
        item,
    };
    let command_value = serde_json::to_value(&command).unwrap_or(Value::Null);
    engine.state.library.last_command = command_value.clone();
    vec![engine.effect(
        EffectKind::WriteLibraryCommand,
        generation,
        WriteLibraryCommandPayload {
            profile_id,
            command: command_value,
        },
    )]
}

pub(super) fn dispatch_toggle_status(
    engine: &mut HeadlessEngine,
    list: String,
    item: Value,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Library);
    let profile_id = active_profile_id(&engine.state, &Value::Null);
    let command = ToggleLibraryStatusCommand {
        kind: "toggleLibraryStatus",
        list,
        item,
    };
    let command_value = serde_json::to_value(&command).unwrap_or(Value::Null);
    engine.state.library.last_command = command_value.clone();
    vec![engine.effect(
        EffectKind::WriteLibraryCommand,
        generation,
        WriteLibraryCommandPayload {
            profile_id,
            command: command_value,
        },
    )]
}

pub(super) fn dispatch_set_feedback(
    engine: &mut HeadlessEngine,
    id: String,
    value: Option<bool>,
    meta: Value,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Library);
    vec![engine.effect(
        EffectKind::WriteFeedback,
        generation,
        WriteFeedbackPayload { id, value, meta },
    )]
}

pub(super) fn dispatch_clear_progress(
    engine: &mut HeadlessEngine,
    profile: Option<Value>,
    meta: Value,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Library);
    vec![engine.effect(
        EffectKind::ClearPlaybackProgress,
        generation,
        ClearPlaybackProgressPayload {
            profile: profile.unwrap_or(Value::Null),
            meta,
        },
    )]
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_save_progress(
    engine: &mut HeadlessEngine,
    profile: Option<Value>,
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
    scrobble_trakt_pause: Option<bool>,
    refresh_external_continue_watching: Option<bool>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Library);
    let profile_id = active_profile_id(&engine.state, &Value::Null);
    let progress = PlaybackProgress {
        meta,
        time_offset: time_offset.max(0),
        duration: duration.max(0),
        last_video_id,
        last_stream_index,
        last_episode_name,
        last_episode_season,
        last_episode_number,
        last_episode_thumbnail,
        last_stream_url,
        last_stream_title,
        last_audio_language,
        last_subtitle_language,
        source: "local",
        refresh_external_continue_watching: refresh_external_continue_watching.unwrap_or(false),
    };
    engine.state.library.pending_playback_progress =
        serde_json::to_value(&progress).unwrap_or(Value::Null);
    vec![engine.effect(
        EffectKind::WritePlaybackProgress,
        generation,
        WritePlaybackProgressPayload {
            profile_id,
            profile: profile.unwrap_or(Value::Null),
            scrobble_trakt_pause: scrobble_trakt_pause.unwrap_or(true),
            progress,
        },
    )]
}

pub(super) fn dispatch_mark_watched(
    engine: &mut HeadlessEngine,
    series_id: String,
    video_ids: Vec<String>,
    watched: Option<bool>,
    meta: Option<Value>,
    episodes: Option<Vec<Value>>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Library);
    let profile_id = active_profile_id(&engine.state, &Value::Null);
    let watched_value = watched.unwrap_or(true);
    let mut seen_video_ids = HashSet::with_capacity(video_ids.len());
    let clean_video_ids: Vec<String> = video_ids
        .into_iter()
        .filter(|value| !value.trim().is_empty() && seen_video_ids.insert(value.clone()))
        .collect();
    let command = MarkWatchedCommand {
        kind: "markWatched",
        series_id,
        video_ids: clean_video_ids,
        watched: watched_value,
    };
    let command_value = serde_json::to_value(&command).unwrap_or(Value::Null);
    engine.state.library.last_command = command_value.clone();
    let mut effects = vec![engine.effect(
        EffectKind::WriteLibraryCommand,
        generation,
        WriteLibraryCommandPayload {
            profile_id,
            command: command_value,
        },
    )];
    if should_sync_watched_state(profile.as_ref(), meta.as_ref()) {
        effects.push(engine.effect(
            EffectKind::SyncWatchedState,
            generation,
            SyncWatchedStatePayload {
                profile: profile.unwrap_or(Value::Null),
                meta: meta.unwrap_or(Value::Null),
                episodes: episodes.unwrap_or_default(),
                watched: watched_value,
            },
        ));
    }
    effects
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "readLibraryState" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                engine.state.library.is_loading = false;
                if result.status.is_ok() {
                    engine.state.library.watchlist = result
                        .value
                        .get("watchlist")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.continue_watching = result
                        .value
                        .get("continueWatching")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.liked = result
                        .value
                        .get("liked")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.watched = result
                        .value
                        .get("watched")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));
                    engine.state.library.dropped = result
                        .value
                        .get("dropped")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.completed = result
                        .value
                        .get("completed")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.error = Value::Null;
                } else {
                    engine.state.library.error = normalize_error(result.error.clone());
                }
            }
        }
        "writeLibraryCommand" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    engine.state.library.last_write = result.value.clone();
                    engine.state.library.last_write_error = Value::Null;
                    if let Some(value) = engine
                        .state
                        .library
                        .last_write
                        .get("isInWatchlist")
                        .cloned()
                    {
                        detail::set_is_in_watchlist(engine, value);
                    }
                    if let Some(value) = engine
                        .state
                        .library
                        .last_write
                        .get("localWatchedVideoIds")
                        .cloned()
                    {
                        detail::set_local_watched_video_ids(engine, value);
                    }
                } else {
                    engine.state.library.last_write_error = normalize_error(result.error.clone());
                }
            }
        }
        "writeFeedback" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    detail::set_feedback(
                        engine,
                        result.value.get("feedback").cloned().unwrap_or(Value::Null),
                    );
                    engine.state.library.last_write_error = Value::Null;
                } else {
                    engine.state.library.last_write_error = normalize_error(result.error.clone());
                }
            }
        }
        "clearPlaybackProgress" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    detail::clear_saved_playback(engine);
                    engine.state.library.last_write_error = Value::Null;
                    // Remove the dropped item from home.continueWatching so stale state
                    // doesn't reappear when the user navigates back to the home screen.
                    if let Some(dropped_id) = result.value.get("droppedId").and_then(Value::as_str)
                    {
                        home::remove_from_continue_watching(engine, dropped_id);
                    }
                } else {
                    engine.state.library.last_write_error = normalize_error(result.error.clone());
                }
            }
        }
        "writePlaybackProgress" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    engine.state.library.saved_playback_progress =
                        engine.state.library.pending_playback_progress.clone();
                    engine.state.library.pending_playback_progress = Value::Null;
                    engine.state.library.last_write_error = Value::Null;
                } else {
                    engine.state.library.last_write_error = normalize_error(result.error.clone());
                }
            }
        }
        "syncWatchedState" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    engine.state.library.last_watched_sync = result.value.clone();
                    engine.state.library.last_watched_sync_error = Value::Null;
                } else {
                    engine.state.library.last_watched_sync_error =
                        normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
