use super::super::state::GenerationKey;
use super::super::{EffectResultInput, HeadlessEngine};
use super::effects::{dispatch_player, player_script_effect, watch_config_effect};
use super::state::TrailerRequest;
use super::stream_resolution::{
    player_script_url, requires_player_script, resolve_player_response,
};
use super::watch_config::parse_watch_config;
use crate::runtime::EffectEnvelope;
use serde_json::Value;

pub(in crate::headless_engine) fn dispatch_resolve(
    engine: &mut HeadlessEngine,
    request_id: String,
    video_id: String,
    max_height: Option<u32>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Trailer);
    engine.state.trailer.resolutions.remove(&request_id);
    engine.state.trailer.requests.insert(
        request_id.clone(),
        TrailerRequest {
            video_id,
            max_height,
            player_response: None,
        },
    );
    if engine.state.trailer.watch_config.is_some() {
        return dispatch_player(engine, generation, &request_id);
    }
    vec![watch_config_effect(engine, generation, Some(request_id))]
}

pub(in crate::headless_engine) fn dispatch_prewarm(
    engine: &mut HeadlessEngine,
) -> Vec<EffectEnvelope> {
    if engine.state.trailer.watch_config.is_some() {
        return vec![];
    }
    let generation = engine.bump_generation(GenerationKey::Trailer);
    vec![watch_config_effect(engine, generation, None)]
}

pub(in crate::headless_engine) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    effect: &EffectEnvelope,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    if generation != engine.state.runtime.get(GenerationKey::Trailer) {
        return vec![];
    }
    let request_id = effect
        .payload
        .get("requestId")
        .and_then(Value::as_str)
        .map(str::to_owned);
    match effect_type {
        "fetchYoutubeTrailerWatchConfig" => {
            if result.status.is_ok() {
                engine.state.trailer.watch_config = Some(parse_watch_config(&result.value));
            }
            request_id
                .as_deref()
                .map(|id| dispatch_player(engine, generation, id))
                .unwrap_or_default()
        }
        "fetchYoutubeTrailerPlayer" => {
            let Some(request_id) = request_id else {
                return vec![];
            };
            if result.status.is_ok()
                && requires_player_script(&result.value)
                && let Some(script_url) = player_script_url(&result.value).or_else(|| {
                    engine
                        .state
                        .trailer
                        .watch_config
                        .as_ref()
                        .and_then(|config| config.player_script_url.clone())
                })
            {
                if let Some(request) = engine.state.trailer.requests.get_mut(&request_id) {
                    request.player_response = Some(result.value.clone());
                }
                return player_script_effect(engine, generation, &request_id, script_url);
            }
            let resolution = if result.status.is_ok() {
                let max_height = engine
                    .state
                    .trailer
                    .requests
                    .get(&request_id)
                    .and_then(|request| request.max_height);
                resolve_player_response(&result.value, max_height, None)
            } else {
                Value::Null
            };
            engine.state.trailer.requests.remove(&request_id);
            engine
                .state
                .trailer
                .resolutions
                .insert(request_id, resolution);
            vec![]
        }
        "fetchYoutubeTrailerPlayerScript" => {
            let Some(request_id) = request_id else {
                return vec![];
            };
            let request = engine.state.trailer.requests.remove(&request_id);
            let resolution = request
                .and_then(|request| {
                    let player_response = request.player_response?;
                    let player_js = result.value.get("body")?.as_str()?;
                    Some(resolve_player_response(
                        &player_response,
                        request.max_height,
                        Some(player_js),
                    ))
                })
                .unwrap_or(Value::Null);
            engine
                .state
                .trailer
                .resolutions
                .insert(request_id, resolution);
            vec![]
        }
        _ => vec![],
    }
}
