use super::helpers::{active_profile_id, normalize_error};
use super::home;
use super::profile;
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct SyncState {
    provider: String,
    is_loading: bool,
    snapshot: Value,
    error: Value,
    generation: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunExternalSyncPayload {
    provider: String,
    profile_id: String,
    profile: Value,
    language: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncExternalIntegrationPayload {
    provider: String,
    profile: Value,
    language: String,
}

pub(super) fn dispatch_external_sync(
    engine: &mut HeadlessEngine,
    provider: String,
    profile: Option<Value>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let provider = provider.trim().to_ascii_lowercase();
    let generation = engine.bump_generation(GenerationKey::Sync);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    *engine.state.sync = SyncState {
        provider: provider.clone(),
        is_loading: true,
        snapshot: Value::Null,
        error: Value::Null,
        generation,
    };
    if provider == "nuvio"
        && profile_value
            .get("nuvioAccessToken")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
    {
        engine.state.sync.is_loading = false;
        engine.state.sync.error = serde_json::json!({ "code": "nuvio_not_connected" });
        return vec![];
    }
    vec![engine.effect(
        EffectKind::RunExternalSync,
        generation,
        RunExternalSyncPayload {
            provider,
            profile_id,
            profile: profile_value,
            language: language.unwrap_or_else(|| "en".to_string()),
        },
    )]
}

pub(super) fn dispatch_integration_sync(
    engine: &mut HeadlessEngine,
    provider: String,
    profile: Value,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Sync);
    *engine.state.sync = SyncState {
        provider: provider.clone(),
        is_loading: true,
        snapshot: Value::Null,
        error: Value::Null,
        generation,
    };
    vec![engine.effect(
        EffectKind::SyncExternalIntegration,
        generation,
        SyncExternalIntegrationPayload {
            provider,
            profile,
            language: language.unwrap_or_else(|| "en".to_string()),
        },
    )]
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "runExternalSync" => {
            if generation == engine.state.runtime.get(GenerationKey::Sync) {
                engine.state.sync.is_loading = false;
                if result.status.is_ok() {
                    engine.state.sync.snapshot = result.value.clone();
                    engine.state.sync.error = Value::Null;
                } else {
                    engine.state.sync.error = normalize_error(result.error.clone());
                }
            }
        }
        "syncExternalIntegration" => {
            if generation == engine.state.runtime.get(GenerationKey::Sync) {
                engine.state.sync.is_loading = false;
                if result.status.is_ok() {
                    let updated_profile =
                        result.value.get("profile").cloned().unwrap_or(Value::Null);
                    engine.state.sync.snapshot =
                        result.value.get("snapshot").cloned().unwrap_or(Value::Null);
                    if !updated_profile.is_null() {
                        profile::update_active(engine, updated_profile);
                    }
                    let external_continue_watching = result
                        .value
                        .get("externalContinueWatching")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    home::set_external_continue_watching(engine, external_continue_watching);
                    engine.state.sync.error = Value::Null;
                } else {
                    engine.state.sync.error = normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
