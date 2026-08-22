use super::helpers::normalize_error;
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct OfflineState {
    last_request: Value,
    last_enqueued: Value,
    error: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnqueueOfflineDownloadPayload {
    meta: Value,
    stream: Value,
    video_id: Option<String>,
    video: Option<Value>,
    subtitle: Option<Value>,
    profile_id: Option<String>,
    language: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    engine: &mut HeadlessEngine,
    meta: Value,
    stream: Value,
    video_id: Option<String>,
    video: Option<Value>,
    subtitle: Option<Value>,
    profile_id: Option<String>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Offline);
    let payload = EnqueueOfflineDownloadPayload {
        meta,
        stream,
        video_id,
        video,
        subtitle,
        profile_id,
        language,
    };
    engine.state.offline.last_request = serde_json::to_value(&payload).unwrap_or(Value::Null);
    vec![engine.effect(EffectKind::EnqueueOfflineDownload, generation, payload)]
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    if generation == engine.state.runtime.get(GenerationKey::Offline) {
        if result.status.is_ok() {
            engine.state.offline.last_enqueued = result.value.clone();
            engine.state.offline.error = Value::Null;
        } else {
            engine.state.offline.error = normalize_error(result.error.clone());
        }
    }
    vec![]
}
