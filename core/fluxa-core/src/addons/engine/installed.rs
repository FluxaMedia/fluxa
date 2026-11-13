use crate::headless_engine::helpers::{normalize_error, upsert_by_key};
use crate::headless_engine::home;
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct AddonsState {
    pub(crate) installed: Value,
    installing: Value,
    for_streamkit: bool,
    last_resource_request: Value,
    last_resource_result: Value,
    error: Value,
}

impl Default for AddonsState {
    fn default() -> Self {
        Self {
            installed: serde_json::json!([]),
            installing: Value::Null,
            for_streamkit: false,
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

pub(crate) fn dispatch_install(
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

pub(crate) fn dispatch_streamkit_install(
    engine: &mut HeadlessEngine,
    transport_url: String,
) -> Vec<EffectEnvelope> {
    let effects = dispatch_install(engine, transport_url, Some(true));
    engine.state.addons.for_streamkit = true;
    effects
}

pub(crate) fn dispatch_streamkit_remove(
    engine: &mut HeadlessEngine,
    transport_url: &str,
) -> Vec<EffectEnvelope> {
    let mut addons = crate::stream_pipeline::kit_addons(&engine.state.settings.values);
    addons.retain(|addon| addon_url(addon) != Some(transport_url));
    crate::settings::engine::dispatch(engine, "streamkitAddons".into(), Value::Array(addons))
}

fn finish_streamkit_install(engine: &mut HeadlessEngine, manifest: Value) -> Vec<EffectEnvelope> {
    engine.state.addons.for_streamkit = false;
    engine.state.addons.installing = Value::Null;
    let inner = manifest.get("manifest").unwrap_or(&manifest);
    let provides_streams = crate::addons::sources::protocol::supports_resource(
        &inner.to_string(),
        "stream",
        None,
        None,
    );
    let Some(url) = addon_url(&manifest)
        .map(ToOwned::to_owned)
        .filter(|_| provides_streams)
    else {
        engine.state.addons.error = serde_json::json!({ "code": "notStreamAddon" });
        return vec![];
    };
    engine.state.addons.error = Value::Null;
    let mut addons = crate::stream_pipeline::kit_addons(&engine.state.settings.values);
    addons.retain(|addon| addon_url(addon) != Some(url.as_str()));
    addons.push(manifest);
    crate::settings::engine::dispatch(engine, "streamkitAddons".into(), Value::Array(addons))
}

pub(crate) fn dispatch_refresh(
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

pub(crate) fn kits_out_of_date(engine: &HeadlessEngine, prefs: &Value) -> bool {
    let listed = |id: &str| {
        engine
            .state
            .addons
            .installed
            .as_array()
            .is_some_and(|items| {
                items
                    .iter()
                    .any(|addon| addon.pointer("/manifest/id").and_then(Value::as_str) == Some(id))
            })
    };
    listed("fluxa.metadatakit") != crate::stream_pipeline::metadatakit_enabled(prefs)
}

fn addon_url(addon: &Value) -> Option<&str> {
    addon
        .get("transportUrl")
        .or_else(|| addon.pointer("/manifest/transportUrl"))
        .and_then(Value::as_str)
}

pub(crate) fn dispatch_remove(
    engine: &mut HeadlessEngine,
    transport_url: &str,
) -> Vec<EffectEnvelope> {
    if let Some(items) = engine.state.addons.installed.as_array_mut() {
        items.retain(|addon| addon_url(addon) != Some(transport_url));
        let addons = engine.state.addons.installed.clone();
        home::set_user_addons(engine, addons);
        return home::reload(engine);
    }
    vec![]
}

pub(crate) fn dispatch_move(
    engine: &mut HeadlessEngine,
    transport_url: &str,
    offset: i64,
) -> Vec<EffectEnvelope> {
    let Some(items) = engine.state.addons.installed.as_array_mut() else {
        return vec![];
    };
    let Some(from) = items
        .iter()
        .position(|addon| addon_url(addon) == Some(transport_url))
    else {
        return vec![];
    };
    let to = (from as i64 + offset).clamp(0, items.len() as i64 - 1) as usize;
    let addon = items.remove(from);
    items.insert(to, addon);
    let addons = engine.state.addons.installed.clone();
    home::set_user_addons(engine, addons);
    home::reload(engine)
}

pub(crate) fn dispatch_toggle(
    engine: &mut HeadlessEngine,
    transport_url: &str,
    enabled: bool,
) -> Vec<EffectEnvelope> {
    let Some(addon) = engine
        .state
        .addons
        .installed
        .as_array_mut()
        .and_then(|items| {
            items
                .iter_mut()
                .find(|addon| addon_url(addon) == Some(transport_url))
        })
    else {
        return vec![];
    };
    addon["enabled"] = Value::Bool(enabled);
    let addons = engine.state.addons.installed.clone();
    home::set_user_addons(engine, addons);
    home::reload(engine)
}

pub(crate) fn dispatch_rename(
    engine: &mut HeadlessEngine,
    transport_url: &str,
    name: &str,
) -> Vec<EffectEnvelope> {
    let Some(addon) = engine
        .state
        .addons
        .installed
        .as_array_mut()
        .and_then(|items| {
            items
                .iter_mut()
                .find(|addon| addon_url(addon) == Some(transport_url))
        })
    else {
        return vec![];
    };
    let name = name.trim();
    if name.is_empty() {
        if let Some(object) = addon.as_object_mut() {
            object.remove("displayName");
        }
    } else {
        addon["displayName"] = Value::String(name.to_owned());
    }
    let addons = engine.state.addons.installed.clone();
    home::set_user_addons(engine, addons);
    home::reload(engine)
}

fn keep_enabled(previous: &Value, fresh: &mut Value) {
    for key in ["enabled", "displayName", "trackingConsent"] {
        if let Some(value) = previous.get(key) {
            fresh[key] = value.clone();
        }
    }
    require_tracking_consent(fresh);
}

fn require_tracking_consent(addon: &mut Value) {
    if super::events::declares_events(addon) && addon.get("trackingConsent").is_none() {
        addon["trackingConsent"] = Value::Bool(false);
    }
}

pub(crate) fn dispatch_resource(
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

pub(crate) fn complete(
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
                    if engine.state.addons.for_streamkit {
                        return finish_streamkit_install(engine, manifest);
                    }
                    let url = addon_url(&manifest).map(ToOwned::to_owned);
                    let existing = engine
                        .state
                        .addons
                        .installed
                        .as_array_mut()
                        .and_then(|items| {
                            items
                                .iter_mut()
                                .find(|addon| url.is_some() && addon_url(addon) == url.as_deref())
                        });
                    match existing {
                        Some(existing) => {
                            let mut manifest = manifest;
                            keep_enabled(existing, &mut manifest);
                            *existing = manifest;
                        }
                        None => {
                            let mut manifest = manifest;
                            require_tracking_consent(&mut manifest);
                            let id = url
                                .as_deref()
                                .or_else(|| manifest.get("id").and_then(Value::as_str))
                                .unwrap_or("unknown")
                                .to_string();
                            upsert_by_key(&mut engine.state.addons.installed, "id", &id, manifest);
                        }
                    }
                    engine.state.addons.installing = Value::Null;
                    engine.state.addons.error = Value::Null;
                    let addons = engine.state.addons.installed.clone();
                    home::set_user_addons(engine, addons);
                    return home::reload(engine);
                } else {
                    engine.state.addons.for_streamkit = false;
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
                    let mut addons = addons;
                    if let (Some(fresh), Some(previous)) = (
                        addons.as_array_mut(),
                        engine.state.addons.installed.as_array(),
                    ) {
                        for addon in fresh {
                            if let Some(old) = previous
                                .iter()
                                .find(|old| addon_url(old) == addon_url(addon))
                            {
                                keep_enabled(old, addon);
                            }
                        }
                    }
                    home::set_user_addons(engine, addons.clone());
                    engine.state.addons.installed = addons;
                    engine.state.addons.error = Value::Null;
                } else {
                    engine.state.addons.error = normalize_error(result.error.clone());
                }
            }
        }
        "fetchAddonResource" if generation == engine.state.runtime.get(GenerationKey::Addon) => {
            if result.status.is_ok() {
                engine.state.addons.last_resource_result = result.value.clone();
                engine.state.addons.error = Value::Null;
            } else {
                engine.state.addons.last_resource_result = Value::Null;
                engine.state.addons.error = normalize_error(result.error.clone());
            }
        }
        _ => {}
    }
    vec![]
}
