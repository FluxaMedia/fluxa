use crate::addons::engine::events;
use crate::headless_engine::HeadlessEngine;
use crate::headless_engine::library::addon_episodes;
use crate::headless_engine::state::GenerationKey;
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnqueueTraktScrobblePayload {
    token: String,
    meta_type: String,
    item_id: String,
    progress: f64,
    action_name: String,
    profile: Value,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    addon_episodes: Vec<Value>,
}

pub(crate) struct AddonPlayerEvent {
    pub(crate) position_ms: Option<i64>,
    pub(crate) duration_ms: Option<i64>,
    pub(crate) closing: bool,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_scrobble(
    engine: &mut HeadlessEngine,
    token: String,
    meta_type: String,
    item_id: String,
    progress: f64,
    action_name: String,
    profile: Option<Value>,
    meta: Option<Value>,
    addon_event: AddonPlayerEvent,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Player);
    let mut effects = addon_player_event(engine, &meta_type, &item_id, &action_name, &addon_event);
    effects.push(engine.effect(
        EffectKind::EnqueueTraktScrobble,
        generation,
        EnqueueTraktScrobblePayload {
            token,
            meta_type,
            item_id,
            progress,
            action_name,
            profile: profile.unwrap_or(Value::Null),
            addon_episodes: addon_episodes(meta.as_ref()),
        },
    ));
    effects
}

fn addon_player_event(
    engine: &mut HeadlessEngine,
    meta_type: &str,
    video_id: &str,
    action: &str,
    event: &AddonPlayerEvent,
) -> Vec<EffectEnvelope> {
    let action = match action {
        "start" | "pause" => action,
        "stop" if event.closing => "stop",
        _ => return vec![],
    };
    let (Some(position_ms), Some(duration_ms)) = (event.position_ms, event.duration_ms) else {
        return vec![];
    };
    events::player_event(
        engine,
        meta_type,
        video_id,
        action,
        position_ms,
        duration_ms,
    )
}
