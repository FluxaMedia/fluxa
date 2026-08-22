use crate::headless_engine::HeadlessEngine;
use crate::headless_engine::state::GenerationKey;
use crate::runtime::{EffectEnvelope, EffectKind};
use crate::stream_policy;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StartTorrentStreamPayload {
    url: String,
    stream: Value,
    current_video_id: Option<String>,
    title: String,
    file_idx: Option<i64>,
    preferred_filename: Option<String>,
    sources: Vec<Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StopTorrentPayload {
    reason: &'static str,
}

pub(in crate::headless_engine) fn complete_direct_playback(
    engine: &mut HeadlessEngine,
    value: Value,
    error: Value,
) {
    if error.is_null() {
        engine.state.player.direct_playback_target = value;
        engine.state.player.player_error = Value::Null;
    } else {
        engine.state.player.direct_playback_target = Value::Null;
        engine.state.player.player_error = error;
    }
}

pub(in crate::headless_engine) fn dispatch_resolve_playback(
    engine: &mut HeadlessEngine,
    url: String,
    stream: Option<Value>,
    current_video_id: Option<String>,
    title: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Player);
    engine.state.player.current_url = Value::String(url.clone());
    engine.state.player.resolved_url = Value::Null;
    engine.state.player.is_buffering = true;
    engine.state.player.is_video_rendered = false;
    engine.state.player.player_error = Value::Null;
    if stream_policy::is_torrent_playback_url(&url) {
        let stream_value = stream.unwrap_or(Value::Null);
        let file_idx = stream_value.get("fileIdx").and_then(Value::as_i64);
        let preferred_filename = stream_value
            .get("effectiveFilename")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let sources = stream_value
            .get("sources")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        vec![engine.effect(
            EffectKind::StartTorrentStream,
            generation,
            StartTorrentStreamPayload {
                url,
                stream: stream_value,
                current_video_id,
                title: title.unwrap_or_else(|| "Fluxa".to_string()),
                file_idx,
                preferred_filename,
                sources,
            },
        )]
    } else {
        engine.state.player.resolved_url = engine.state.player.current_url.clone();
        engine.state.player.is_buffering = false;
        vec![engine.effect(
            EffectKind::StopTorrent,
            generation,
            StopTorrentPayload {
                reason: "directPlayback",
            },
        )]
    }
}
