use super::stream_load::{dispatch_streams_failed, dispatch_streams_loaded};
use crate::headless_engine::helpers::{error_code, normalize_error};
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::EffectEnvelope;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrefetchedNextEpisode {
    video_id: Value,
    streams: Value,
}

pub(in crate::headless_engine) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "loadStreams" => {
            if generation == engine.state.runtime.get(GenerationKey::Player) {
                let pending = engine.state.player.pending_stream_load.clone();
                if result.status.is_ok() {
                    dispatch_streams_loaded(
                        engine,
                        result.value.as_array().cloned().unwrap_or_default(),
                        pending
                            .get("currentVideoId")
                            .and_then(Value::as_str)
                            .map(ToString::to_string),
                        Some(
                            pending
                                .get("initialStreamIndex")
                                .and_then(Value::as_i64)
                                .unwrap_or(0) as i32,
                        ),
                        pending
                            .get("savedUrl")
                            .and_then(Value::as_str)
                            .map(ToString::to_string),
                        pending
                            .get("savedTitle")
                            .and_then(Value::as_str)
                            .map(ToString::to_string),
                        pending
                            .get("sourceSelectionMode")
                            .and_then(Value::as_str)
                            .map(ToString::to_string),
                        pending
                            .get("regexPattern")
                            .and_then(Value::as_str)
                            .map(ToString::to_string),
                        pending
                            .get("preferredBingeGroup")
                            .and_then(Value::as_str)
                            .map(ToString::to_string),
                    );
                } else {
                    dispatch_streams_failed(engine, Some(error_code(&result.error)));
                }
                engine.state.player.pending_stream_load = Value::Null;
            }
        }
        "startTorrentStream" => {
            if generation == engine.state.runtime.get(GenerationKey::Player) {
                if result.status.is_ok() {
                    engine.state.player.resolved_url =
                        result.value.get("url").cloned().unwrap_or(Value::Null);
                    engine.state.player.is_buffering = false;
                    engine.state.player.player_error = Value::Null;
                } else {
                    engine.state.player.resolved_url = Value::Null;
                    engine.state.player.is_buffering = false;
                    engine.state.player.player_error = Value::String(error_code(&result.error));
                }
            }
        }
        "enqueueTraktScrobble" => {
            if generation == engine.state.runtime.get(GenerationKey::Player) {
                if result.status.is_ok() {
                    engine.state.player.last_scrobble = result.value.clone();
                    engine.state.player.player_error = Value::Null;
                } else {
                    engine.state.player.player_error = Value::String(error_code(&result.error));
                }
            }
        }
        "stopTorrent" => {
            if generation == engine.state.runtime.get(GenerationKey::Player)
                && !result.status.is_ok()
            {
                engine.state.player.stop_torrent_warning = normalize_error(result.error.clone());
            }
        }
        "fetchIntroSegments" => {
            if generation == engine.state.runtime.get(GenerationKey::Intro) {
                if result.status.is_ok() {
                    engine.state.player.intro_segments = result.value.clone();
                    engine.state.player.player_error = Value::Null;
                } else {
                    engine.state.player.intro_segments = serde_json::json!([]);
                    engine.state.player.player_error = Value::String(error_code(&result.error));
                }
            }
        }
        "resolveIntroImdbId" => {
            if generation == engine.state.runtime.get(GenerationKey::Intro) {
                if result.status.is_ok() {
                    engine.state.player.intro_imdb_id = result.value.clone();
                    engine.state.player.player_error = Value::Null;
                } else {
                    engine.state.player.intro_imdb_id = Value::Null;
                    engine.state.player.player_error = Value::String(error_code(&result.error));
                }
            }
        }
        "fetchSubtitles" => {
            if generation == engine.state.runtime.get(GenerationKey::Player) {
                engine.state.player.subtitle_loading = false;
                if result.status.is_ok() {
                    engine.state.player.subtitles = result
                        .value
                        .get("subtitles")
                        .cloned()
                        .unwrap_or_else(|| result.value.clone());
                    engine.state.player.player_error = Value::Null;
                } else {
                    engine.state.player.player_error = Value::String(error_code(&result.error));
                }
            }
        }
        "prefetchNextEpisodeStreams" => {
            // Only accept if the prefetch generation is still current (player hasn't moved on).
            if generation == engine.state.runtime.get(GenerationKey::Player) {
                if result.status.is_ok() {
                    let prefetched_video_id = engine.state.player.prefetching_next_video_id.clone();
                    engine.state.player.prefetched_next_episode =
                        serde_json::to_value(PrefetchedNextEpisode {
                            video_id: prefetched_video_id,
                            streams: result
                                .value
                                .get("streams")
                                .cloned()
                                .unwrap_or_else(|| Value::Array(vec![])),
                        })
                        .unwrap_or(Value::Null);
                }
                engine.state.player.prefetching_next_video_id = Value::Null;
            }
        }
        _ => {}
    }
    vec![]
}
