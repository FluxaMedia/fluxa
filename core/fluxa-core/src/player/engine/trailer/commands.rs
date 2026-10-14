use super::clients;
use super::effects::{dispatch_player, player_script_effect, watch_config_effect};
use super::state::TrailerRequest;
use super::stream_resolution::{
    player_script_url, requires_player_script, resolve_player_response,
};
use super::watch_config::parse_watch_config;
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::EffectEnvelope;
use serde_json::{Value, json};

const WATCH_CONFIG_RETRIES: usize = 2;

pub(crate) fn dispatch_resolve(
    engine: &mut HeadlessEngine,
    request_id: String,
    video_id: String,
    max_height: Option<u32>,
    client_index: Option<usize>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Trailer);
    engine.state.trailer.resolutions.remove(&request_id);
    let start = client_index.unwrap_or(engine.state.trailer.preferred_client);
    engine.state.trailer.requests.insert(
        request_id.clone(),
        TrailerRequest {
            video_id,
            max_height,
            player_response: None,
            start,
            attempt: 0,
        },
    );
    if engine.state.trailer.watch_config.is_some() {
        return dispatch_player(engine, generation, &request_id);
    }
    vec![watch_config_effect(engine, generation, Some(request_id))]
}

pub(crate) fn dispatch_prewarm(engine: &mut HeadlessEngine) -> Vec<EffectEnvelope> {
    if engine.state.trailer.watch_config.is_some() {
        return vec![];
    }
    let generation = engine.bump_generation(GenerationKey::Trailer);
    vec![watch_config_effect(engine, generation, None)]
}

pub(crate) fn complete(
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
            let parsed = result
                .status
                .is_ok()
                .then(|| parse_watch_config(&result.value));
            let missing_visitor = parsed
                .as_ref()
                .is_none_or(|config| config.visitor_data.is_none());
            if missing_visitor && engine.state.trailer.watch_config_retries < WATCH_CONFIG_RETRIES {
                engine.state.trailer.watch_config_retries += 1;
                return vec![watch_config_effect(engine, generation, request_id)];
            }
            engine.state.trailer.watch_config_retries = 0;
            if let Some(config) = parsed {
                engine.state.trailer.watch_config = Some(config);
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
            finish(engine, generation, &request_id, resolution)
        }
        "fetchYoutubeTrailerPlayerScript" => {
            let Some(request_id) = request_id else {
                return vec![];
            };
            let resolution = engine
                .state
                .trailer
                .requests
                .get(&request_id)
                .and_then(|request| {
                    let player_response = request.player_response.as_ref()?;
                    let player_js = result.value.get("body")?.as_str()?;
                    Some(resolve_player_response(
                        player_response,
                        request.max_height,
                        Some(player_js),
                    ))
                })
                .unwrap_or(Value::Null);
            finish(engine, generation, &request_id, resolution)
        }
        _ => vec![],
    }
}

fn finish(
    engine: &mut HeadlessEngine,
    generation: u64,
    request_id: &str,
    mut resolution: Value,
) -> Vec<EffectEnvelope> {
    let Some(request) = engine.state.trailer.requests.get_mut(request_id) else {
        return vec![];
    };
    if resolution.is_null() && request.attempt + 1 < clients::COUNT {
        request.attempt += 1;
        request.player_response = None;
        return dispatch_player(engine, generation, request_id);
    }
    let index = clients::rotated_index(request.start, request.attempt);
    engine.state.trailer.requests.remove(request_id);
    if let Some(object) = resolution.as_object_mut() {
        object.insert("clientIndex".to_string(), json!(index));
        engine.state.trailer.preferred_client = index;
    }
    engine
        .state
        .trailer
        .resolutions
        .insert(request_id.to_string(), resolution);
    vec![]
}
