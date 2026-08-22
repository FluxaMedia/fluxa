use super::state::PlayerState;
use crate::headless_engine::HeadlessEngine;
use crate::headless_engine::library;
use crate::headless_engine::state::GenerationKey;
use crate::player_flow::{self, PlayerFlowAction};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutgoingProgress {
    time_offset: i64,
    duration: i64,
    last_stream_index: Option<i32>,
    last_episode_name: Option<String>,
    last_episode_season: Option<i64>,
    last_episode_number: Option<i64>,
    last_episode_thumbnail: Option<String>,
    last_stream_url: Option<String>,
    last_stream_title: Option<String>,
    last_audio_language: Option<String>,
    last_subtitle_language: Option<String>,
    #[serde(default)]
    meta: Value,
    scrobble_trakt_pause: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrefetchNextEpisodeStreamsPayload {
    content_type: String,
    series_id: String,
    next_video_id: String,
    title: String,
    original_name: Option<String>,
    year: Option<i32>,
    language: String,
    profile: Value,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PendingStreamLoad {
    saved_url: Option<String>,
    saved_title: Option<String>,
    source_selection_mode: String,
    regex_pattern: Option<String>,
    preferred_binge_group: Option<String>,
    initial_streams: Vec<Value>,
    initial_stream_index: i32,
    current_video_id: Option<String>,
    title: Option<String>,
    original_name: Option<String>,
    year: Option<i32>,
    language: String,
    profile: Value,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::headless_engine) fn dispatch_next_episode_prefetch(
    engine: &mut HeadlessEngine,
    content_type: String,
    series_id: String,
    next_video_id: String,
    title: Option<String>,
    original_name: Option<String>,
    year: Option<i32>,
    language: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let already_prefetching = engine
        .state
        .player
        .prefetching_next_video_id
        .as_str()
        .is_some_and(|v| v == next_video_id);
    let already_cached = engine
        .state
        .player
        .prefetched_next_episode
        .get("videoId")
        .and_then(Value::as_str)
        .is_some_and(|v| v == next_video_id);
    if already_prefetching || already_cached {
        return vec![];
    }

    let generation = engine.state.runtime.get(GenerationKey::Player);
    engine.state.player.prefetching_next_video_id = Value::String(next_video_id.clone());
    vec![engine.effect(
        EffectKind::PrefetchNextEpisodeStreams,
        generation,
        PrefetchNextEpisodeStreamsPayload {
            content_type,
            series_id,
            next_video_id,
            title: title.unwrap_or_default(),
            original_name,
            year,
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

#[allow(clippy::too_many_arguments)]
pub(in crate::headless_engine) fn dispatch_load_streams(
    engine: &mut HeadlessEngine,
    content_type: String,
    id: String,
    current_video_id: Option<String>,
    initial_video_id: Option<String>,
    initial_streams: Option<Vec<Value>>,
    initial_stream_index: Option<i32>,
    saved_url: Option<String>,
    saved_title: Option<String>,
    source_selection_mode: Option<String>,
    regex_pattern: Option<String>,
    preferred_binge_group: Option<String>,
    title: Option<String>,
    original_name: Option<String>,
    year: Option<i32>,
    language: Option<String>,
    profile: Option<Value>,
    outgoing_progress: Option<Value>,
) -> Vec<EffectEnvelope> {
    let mut save_effects = Vec::new();
    if let (Some(outgoing_video_id), Some(raw)) = (current_video_id.clone(), outgoing_progress)
        && outgoing_video_id != initial_video_id.clone().unwrap_or_default()
        && let Ok(progress) = serde_json::from_value::<OutgoingProgress>(raw)
    {
        save_effects.extend(library::dispatch_save_progress(
            engine,
            profile.clone(),
            progress.meta,
            progress.time_offset,
            progress.duration,
            Some(outgoing_video_id),
            progress.last_stream_index,
            progress.last_episode_name,
            progress.last_episode_season,
            progress.last_episode_number,
            progress.last_episode_thumbnail,
            progress.last_stream_url,
            progress.last_stream_title,
            progress.last_audio_language,
            progress.last_subtitle_language,
            progress.scrobble_trakt_pause,
            None,
        ));
    }

    let generation = engine.bump_generation(GenerationKey::Player);
    let mut initial_streams = initial_streams.unwrap_or_default();
    let initial_stream_index = initial_stream_index.unwrap_or(0);

    // If no initial streams were provided by the caller but we have a prefetch
    // cache hit for this video_id, inject those streams so playback can start
    // without waiting for a fresh fetch.
    let mut effective_initial_video_id = initial_video_id.clone();
    if initial_streams.is_empty() {
        let prefetched = engine.state.player.prefetched_next_episode.clone();
        let cached_video_id = prefetched
            .get("videoId")
            .and_then(Value::as_str)
            .map(str::to_string);
        if cached_video_id.is_some() && cached_video_id == current_video_id {
            initial_streams = prefetched
                .get("streams")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            effective_initial_video_id = cached_video_id;
            engine.state.player.prefetched_next_episode = Value::Null;
        }
    }

    let action = PlayerFlowAction::LoadStreamsRequested {
        content_type: content_type.clone(),
        id: id.clone(),
        current_video_id: current_video_id.clone(),
        initial_video_id: effective_initial_video_id,
        initial_streams: initial_streams.clone(),
        initial_stream_index,
    };
    let mut flow_state = engine.state.player.to_flow_state();
    let effects = player_flow::dispatch(&mut flow_state, action);
    *engine.state.player = PlayerState::from_flow_state(flow_state);

    let pending = PendingStreamLoad {
        saved_url,
        saved_title,
        source_selection_mode: source_selection_mode.unwrap_or_else(|| "manual".to_string()),
        regex_pattern,
        preferred_binge_group,
        initial_streams,
        initial_stream_index,
        current_video_id,
        title,
        original_name,
        year,
        language: language.unwrap_or_else(|| "en".to_string()),
        profile: profile.unwrap_or(Value::Null),
    };
    let pending_value = serde_json::to_value(&pending).unwrap_or(Value::Null);
    engine.state.player.pending_stream_load = pending_value.clone();

    save_effects.extend(effects.into_iter().filter_map(|effect| {
        let mut payload = serde_json::to_value(&effect).unwrap_or(Value::Null);
        let kind = payload
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        if kind == "loadStreams"
            && let Value::Object(map) = &mut payload
        {
            for key in [
                "initialStreams",
                "title",
                "originalName",
                "year",
                "language",
                "profile",
            ] {
                map.insert(
                    key.to_string(),
                    pending_value.get(key).cloned().unwrap_or(Value::Null),
                );
            }
        }
        let kind = EffectKind::from_str(&kind)?;
        Some(engine.effect(kind, generation, payload))
    }));
    save_effects
}

#[allow(clippy::too_many_arguments)]
pub(in crate::headless_engine) fn dispatch_streams_loaded(
    engine: &mut HeadlessEngine,
    streams: Vec<Value>,
    current_video_id: Option<String>,
    initial_stream_index: Option<i32>,
    saved_url: Option<String>,
    saved_title: Option<String>,
    source_selection_mode: Option<String>,
    regex_pattern: Option<String>,
    preferred_binge_group: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.runtime.get(GenerationKey::Player);
    let action = PlayerFlowAction::StreamsLoaded {
        streams,
        current_video_id,
        initial_stream_index: initial_stream_index.unwrap_or(0),
        saved_url,
        saved_title,
        source_selection_mode,
        regex_pattern,
        preferred_binge_group,
    };
    let mut flow_state = engine.state.player.to_flow_state();
    let _ = player_flow::dispatch(&mut flow_state, action);
    *engine.state.player = PlayerState::from_flow_state(flow_state);
    engine.state.player.generation = generation;
    vec![]
}

pub(in crate::headless_engine) fn dispatch_streams_failed(
    engine: &mut HeadlessEngine,
    err_code: Option<String>,
) -> Vec<EffectEnvelope> {
    let action = PlayerFlowAction::StreamsFailed {
        error_code: err_code,
    };
    let mut flow_state = engine.state.player.to_flow_state();
    let _ = player_flow::dispatch(&mut flow_state, action);
    *engine.state.player = PlayerState::from_flow_state(flow_state);
    vec![]
}
