use super::helpers::{normalize_error, upsert_by_key};
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::addon_store::normalize_plugin_repository_url;
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct PluginsState {
    repositories: Value,
    scrapers: Value,
    adding_repository_url: Value,
    error: Value,
}

impl Default for PluginsState {
    fn default() -> Self {
        Self {
            repositories: json!([]),
            scrapers: json!([]),
            adding_repository_url: Value::Null,
            error: Value::Null,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchPluginManifestPayload {
    manifest_url: String,
}

pub(super) fn dispatch_add_repository(
    engine: &mut HeadlessEngine,
    manifest_url: String,
) -> Vec<EffectEnvelope> {
    let manifest_url = normalize_plugin_repository_url(&manifest_url);
    let generation = engine.bump_generation(GenerationKey::Plugins);
    engine.state.plugins.adding_repository_url = Value::String(manifest_url.clone());
    engine.state.plugins.error = Value::Null;
    vec![engine.effect(
        EffectKind::FetchPluginManifest,
        generation,
        FetchPluginManifestPayload { manifest_url },
    )]
}

pub(super) fn dispatch_remove_repository(
    engine: &mut HeadlessEngine,
    manifest_url: String,
) -> Vec<EffectEnvelope> {
    let manifest_url = normalize_plugin_repository_url(&manifest_url);
    if let Some(items) = engine.state.plugins.repositories.as_array_mut() {
        items.retain(|repo| repo["manifestUrl"].as_str() != Some(manifest_url.as_str()));
    }
    if let Some(items) = engine.state.plugins.scrapers.as_array_mut() {
        items.retain(|scraper| scraper["repositoryUrl"].as_str() != Some(manifest_url.as_str()));
    }
    vec![]
}

pub(super) fn dispatch_toggle_scraper(
    engine: &mut HeadlessEngine,
    scraper_id: String,
    enabled: bool,
) -> Vec<EffectEnvelope> {
    if let Some(items) = engine.state.plugins.scrapers.as_array_mut()
        && let Some(scraper) = items
            .iter_mut()
            .find(|scraper| scraper["id"].as_str() == Some(scraper_id.as_str()))
    {
        scraper["enabled"] = Value::Bool(enabled);
    }
    vec![]
}

pub(super) fn dispatch_update_scraper_settings(
    engine: &mut HeadlessEngine,
    scraper_id: String,
    settings: Value,
) -> Vec<EffectEnvelope> {
    if let Some(items) = engine.state.plugins.scrapers.as_array_mut()
        && let Some(scraper) = items
            .iter_mut()
            .find(|scraper| scraper["id"].as_str() == Some(scraper_id.as_str()))
    {
        scraper["settings"] = settings;
    }
    vec![]
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    if generation != engine.state.runtime.get(GenerationKey::Plugins) {
        return vec![];
    }
    engine.state.plugins.adding_repository_url = Value::Null;

    if !result.status.is_ok() {
        engine.state.plugins.error = normalize_error(result.error.clone());
        return vec![];
    }

    let manifest_url = result
        .value
        .get("manifestUrl")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let manifest = result.value.get("manifest").unwrap_or(&Value::Null);
    if manifest_url.is_empty() || !manifest.is_object() {
        engine.state.plugins.error = normalize_error(Value::Null);
        return vec![];
    }

    let scrapers = manifest
        .get("scrapers")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let repository_entry = json!({
        "manifestUrl": manifest_url,
        "name": manifest.get("name").cloned().unwrap_or(Value::Null),
        "description": manifest.get("description").cloned().unwrap_or(Value::Null),
        "version": manifest.get("version").cloned().unwrap_or(Value::Null),
        "scraperCount": scrapers.len(),
    });
    upsert_by_key(
        &mut engine.state.plugins.repositories,
        "manifestUrl",
        manifest_url,
        repository_entry,
    );

    let previous_by_id: std::collections::HashMap<String, Value> = engine
        .state
        .plugins
        .scrapers
        .as_array()
        .into_iter()
        .flatten()
        .filter(|scraper| {
            scraper.get("repositoryUrl").and_then(Value::as_str) == Some(manifest_url)
        })
        .filter_map(|scraper| {
            scraper
                .get("id")
                .and_then(Value::as_str)
                .map(|id| (id.to_string(), scraper.clone()))
        })
        .collect();

    if let Some(items) = engine.state.plugins.scrapers.as_array_mut() {
        items.retain(|scraper| {
            scraper.get("repositoryUrl").and_then(Value::as_str) != Some(manifest_url)
        });
    }
    for mut scraper in scrapers {
        let Some(scraper_fields) = scraper.as_object_mut() else {
            continue;
        };
        scraper_fields.insert(
            "repositoryUrl".to_string(),
            Value::String(manifest_url.to_string()),
        );
        let manifest_enabled = scraper_fields
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let scraper_id = scraper_fields
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string);
        if let Some(previous) = scraper_id.and_then(|id| previous_by_id.get(&id)) {
            if manifest_enabled
                && let Some(previous_enabled) = previous.get("enabled").and_then(Value::as_bool)
            {
                scraper_fields.insert("enabled".to_string(), Value::Bool(previous_enabled));
            }
            if let Some(previous_settings) = previous.get("settings") {
                scraper_fields.insert("settings".to_string(), previous_settings.clone());
            }
        }
        if !scraper_fields.contains_key("settings") {
            scraper_fields.insert("settings".to_string(), json!({}));
        }
        if let Some(items) = engine.state.plugins.scrapers.as_array_mut() {
            items.push(scraper);
        }
    }
    engine.state.plugins.error = Value::Null;
    vec![]
}
