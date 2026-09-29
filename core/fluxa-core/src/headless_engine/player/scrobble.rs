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

pub(in crate::headless_engine) fn dispatch_scrobble(
    engine: &mut HeadlessEngine,
    token: String,
    meta_type: String,
    item_id: String,
    progress: f64,
    action_name: String,
    profile: Option<Value>,
    meta: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Player);
    vec![engine.effect(
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
    )]
}
