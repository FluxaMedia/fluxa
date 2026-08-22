use super::helpers::{normalize_error, upsert_by_key};
use super::home;
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct AddonsState {
    installed: Value,
    installing: Value,
    last_resource_request: Value,
    last_resource_result: Value,
    error: Value,
}

impl Default for AddonsState {
    fn default() -> Self {
        Self {
            installed: serde_json::json!([]),
            installing: Value::Null,
            last_resource_request: Value::Null,
            last_resource_result: Value::Null,
            error: Value::Null,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchAddonManifestPayload {
    transport_url: String,
    force_refresh: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshInstalledAddonsPayload {
    profile: Value,
    force_refresh: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchAddonResourcePayload {
    transport_url: String,
    resource: String,
    content_type: String,
    id: String,
    extra: Value,
}

pub(super) fn dispatch_install(
    engine: &mut HeadlessEngine,
    transport_url: String,
    force_refresh: Option<bool>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Addon);
    engine.state.addons.installing = Value::String(transport_url.clone());
    engine.state.addons.error = Value::Null;
    vec![engine.effect(
        EffectKind::FetchAddonManifest,
        generation,
        FetchAddonManifestPayload {
            transport_url,
            force_refresh: force_refresh.unwrap_or(false),
        },
    )]
}

pub(super) fn dispatch_refresh(
    engine: &mut HeadlessEngine,
    profile: Option<Value>,
    force_refresh: Option<bool>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Addon);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    vec![engine.effect(
        EffectKind::RefreshInstalledAddons,
        generation,
        RefreshInstalledAddonsPayload {
            profile: profile_value,
            force_refresh: force_refresh.unwrap_or(true),
        },
    )]
}

pub(super) fn dispatch_resource(
    engine: &mut HeadlessEngine,
    transport_url: String,
    resource: String,
    content_type: String,
    id: String,
    extra: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Addon);
    let payload = FetchAddonResourcePayload {
        transport_url,
        resource,
        content_type,
        id,
        extra: extra.unwrap_or(Value::Null),
    };
    engine.state.addons.last_resource_request =
        serde_json::to_value(&payload).unwrap_or(Value::Null);
    vec![engine.effect(EffectKind::FetchAddonResource, generation, payload)]
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "fetchAddonManifest" => {
            if generation == engine.state.runtime.get(GenerationKey::Addon) {
                if result.status.is_ok() {
                    let manifest = result.value.clone();
                    let id = manifest
                        .get("transportUrl")
                        .and_then(Value::as_str)
                        .or_else(|| manifest.get("id").and_then(Value::as_str))
                        .unwrap_or("unknown")
                        .to_string();
                    upsert_by_key(&mut engine.state.addons.installed, "id", &id, manifest);
                    engine.state.addons.installing = Value::Null;
                    engine.state.addons.error = Value::Null;
                } else {
                    engine.state.addons.installing = Value::Null;
                    engine.state.addons.error = normalize_error(result.error.clone());
                }
            }
        }
        "refreshInstalledAddons" => {
            if generation == engine.state.runtime.get(GenerationKey::Addon) {
                if result.status.is_ok() {
                    let addons = result
                        .value
                        .get("addons")
                        .cloned()
                        .unwrap_or_else(|| result.value.clone());
                    home::set_user_addons(engine, addons.clone());
                    engine.state.addons.installed = addons;
                    engine.state.addons.error = Value::Null;
                } else {
                    engine.state.addons.error = normalize_error(result.error.clone());
                }
            }
        }
        "fetchAddonResource" => {
            if generation == engine.state.runtime.get(GenerationKey::Addon) {
                if result.status.is_ok() {
                    engine.state.addons.last_resource_result = result.value.clone();
                    engine.state.addons.error = Value::Null;
                } else {
                    engine.state.addons.last_resource_result = Value::Null;
                    engine.state.addons.error = normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
