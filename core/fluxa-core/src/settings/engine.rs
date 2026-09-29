use crate::headless_engine::helpers::normalize_error;
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// `values` is a genuinely dynamic settings bag keyed by whatever the platform sends
// in SettingsChanged — there's no fixed schema to type beyond the container.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct SettingsState {
    pub(crate) values: Value,
    last_write_error: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WriteSettingsPayload {
    key: String,
    value: Value,
}

pub(crate) fn dispatch(
    engine: &mut HeadlessEngine,
    key: String,
    value: Value,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Settings);
    if !engine.state.settings.values.is_object() {
        engine.state.settings.values = serde_json::json!({});
    }
    if let Some(values) = engine.state.settings.values.as_object_mut() {
        values.insert(key.clone(), value.clone());
    }
    vec![engine.effect(
        EffectKind::WriteSettings,
        generation,
        WriteSettingsPayload { key, value },
    )]
}

pub(crate) fn complete(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    if generation == engine.state.runtime.get(GenerationKey::Settings) {
        if !result.status.is_ok() {
            engine.state.settings.last_write_error = normalize_error(result.error.clone());
        } else {
            engine.state.settings.last_write_error = Value::Null;
        }
    }
    vec![]
}
