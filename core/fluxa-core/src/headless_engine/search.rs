use super::helpers::{active_profile_id, normalize_error};
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct SearchState {
    query: String,
    is_loading: bool,
    results: Value,
    categories: Value,
    grouping: Value,
    error: Value,
    generation: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunSearchPayload {
    query: String,
    profile_id: String,
    profile: Value,
    language: String,
}

pub(super) fn dispatch(
    engine: &mut HeadlessEngine,
    query: String,
    profile: Option<Value>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Search);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    *engine.state.search = SearchState {
        query: query.clone(),
        is_loading: true,
        results: serde_json::json!([]),
        categories: Value::Null,
        grouping: Value::Null,
        error: Value::Null,
        generation,
    };
    vec![engine.effect(
        EffectKind::RunSearch,
        generation,
        RunSearchPayload {
            query,
            profile_id,
            profile: profile_value,
            language: language.unwrap_or_else(|| "en".to_string()),
        },
    )]
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    if generation == engine.state.runtime.get(GenerationKey::Search) {
        engine.state.search.is_loading = false;
        if result.status.is_ok() {
            engine.state.search.results = result
                .value
                .get("results")
                .cloned()
                .unwrap_or_else(|| result.value.clone());
            engine.state.search.categories = result
                .value
                .get("categories")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([]));
            engine.state.search.grouping =
                result.value.get("grouping").cloned().unwrap_or(Value::Null);
            engine.state.search.error = Value::Null;
        } else {
            engine.state.search.error = normalize_error(result.error.clone());
        }
    }
    vec![]
}
