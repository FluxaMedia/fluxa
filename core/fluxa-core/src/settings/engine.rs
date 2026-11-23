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
    let mut effects = vec![engine.effect(
        EffectKind::WriteSettings,
        generation,
        WriteSettingsPayload {
            key: key.clone(),
            value: value.clone(),
        },
    )];
    match key.as_str() {
        "integrationLibrarySource" => {
            effects.extend(crate::headless_engine::library::dispatch_hydrate(
                engine, None,
            ));
        }
        "continueWatchingSource" => {
            effects.extend(
                crate::headless_engine::home::dispatch_refresh_continue_watching(
                    engine, None, None,
                ),
            );
        }
        _ => {}
    }
    effects
}

const SOURCE_KEYS: [&str; 2] = ["integrationLibrarySource", "continueWatchingSource"];

fn source_token_field(source: &str) -> Option<&'static str> {
    match source {
        "nuvio" => Some("nuvioAccessToken"),
        "stremio" => Some("stremioAuthKey"),
        other => crate::services::registry::provider(other).map(|provider| provider.token_field),
    }
}

fn has_token(profile: &Value, field: &str) -> bool {
    profile
        .get(field)
        .and_then(Value::as_str)
        .is_some_and(|token| !token.trim().is_empty())
}

pub(crate) fn reset_disconnected_sources(
    engine: &mut HeadlessEngine,
    profile: &Value,
) -> Vec<EffectEnvelope> {
    let fallback = if has_token(profile, "nuvioAccessToken") {
        "nuvio"
    } else {
        "local"
    };
    let stale = SOURCE_KEYS
        .into_iter()
        .filter(|key| {
            engine
                .state
                .settings
                .values
                .get(key)
                .and_then(Value::as_str)
                .and_then(source_token_field)
                .is_some_and(|field| !has_token(profile, field))
        })
        .collect::<Vec<_>>();
    stale
        .into_iter()
        .flat_map(|key| dispatch(engine, key.to_owned(), Value::String(fallback.to_owned())))
        .collect()
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
            if crate::addons::engine::installed::kits_out_of_date(engine, &result.value) {
                return crate::addons::engine::installed::dispatch_refresh(
                    engine,
                    None,
                    Some(false),
                );
            }
        }
    }
    vec![]
}
