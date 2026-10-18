use crate::headless_engine::helpers::{active_profile_id, normalize_error};
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct SearchState {
    query: String,
    language: String,
    is_loading: bool,
    results: Value,
    categories: Value,
    grouping: Value,
    error: Value,
    generation: u64,
    sources: Vec<SearchSource>,
    has_more: bool,
    is_loading_more: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
struct SearchSource {
    id: String,
    loaded: u64,
    has_more: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunSearchPayload {
    query: String,
    profile_id: String,
    profile: Value,
    language: String,
    skips: serde_json::Map<String, Value>,
}

pub(crate) fn dispatch(
    engine: &mut HeadlessEngine,
    query: String,
    profile: Option<Value>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Search);
    let language = language.unwrap_or_else(|| "en".to_string());
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    *engine.state.search = SearchState {
        query: query.clone(),
        language: language.clone(),
        is_loading: true,
        results: serde_json::json!([]),
        categories: Value::Null,
        grouping: Value::Null,
        error: Value::Null,
        generation,
        sources: Vec::new(),
        has_more: false,
        is_loading_more: false,
    };
    vec![engine.effect(
        EffectKind::RunSearch,
        generation,
        RunSearchPayload {
            query,
            profile_id,
            profile: profile_value,
            language,
            skips: serde_json::Map::new(),
        },
    )]
}

pub(crate) fn dispatch_page(
    engine: &mut HeadlessEngine,
    profile: Option<Value>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let search = &engine.state.search;
    if search.query.trim().is_empty()
        || !search.has_more
        || search.is_loading
        || search.is_loading_more
    {
        return vec![];
    }
    let skips = search
        .sources
        .iter()
        .filter(|source| source.has_more)
        .map(|source| (source.id.clone(), Value::from(source.loaded)))
        .collect();
    let query = search.query.clone();
    let language = language.unwrap_or_else(|| search.language.clone());
    let generation = engine.state.runtime.get(GenerationKey::Search);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    engine.state.search.is_loading_more = true;
    vec![engine.effect(
        EffectKind::RunSearch,
        generation,
        RunSearchPayload {
            query,
            profile_id,
            profile: profile_value,
            language,
            skips,
        },
    )]
}

pub(crate) fn complete(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    if generation != engine.state.runtime.get(GenerationKey::Search) {
        return vec![];
    }
    let appending = engine.state.search.is_loading_more;
    engine.state.search.is_loading = false;
    engine.state.search.is_loading_more = false;
    if !result.status.is_ok() {
        if !appending {
            engine.state.search.error = normalize_error(result.error.clone());
        }
        return vec![];
    }
    let results = result
        .value
        .get("results")
        .cloned()
        .unwrap_or_else(|| result.value.clone());
    let categories = result
        .value
        .get("categories")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    let fetched: Vec<SearchSource> = result
        .value
        .get("sources")
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default();
    let search = &mut engine.state.search;
    search.error = Value::Null;
    if appending {
        search.results = append_unique(&search.results, &results);
        search.categories = append_categories(&search.categories, &categories);
        for source in &mut search.sources {
            if let Some(page) = fetched.iter().find(|page| page.id == source.id) {
                source.loaded += page.loaded;
                source.has_more = page.has_more;
            } else if source.has_more {
                source.has_more = false;
            }
        }
        search.grouping = crate::catalog::search::search_result_grouping_json(
            &serde_json::json!({"query": search.query, "results": search.results}).to_string(),
        )
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or(Value::Null);
    } else {
        search.results = results;
        search.categories = categories;
        search.grouping = result.value.get("grouping").cloned().unwrap_or(Value::Null);
        search.sources = fetched;
    }
    search.has_more = search.sources.iter().any(|source| source.has_more);
    vec![]
}

fn item_key(item: &Value) -> String {
    let field = |name| item.get(name).and_then(Value::as_str).unwrap_or_default();
    format!("{}:{}", field("type"), field("id"))
}

fn append_unique(existing: &Value, page: &Value) -> Value {
    let mut merged = existing.as_array().cloned().unwrap_or_default();
    let mut seen: std::collections::HashSet<String> = merged.iter().map(item_key).collect();
    for item in page.as_array().into_iter().flatten() {
        if seen.insert(item_key(item)) {
            merged.push(item.clone());
        }
    }
    Value::Array(merged)
}

fn append_categories(existing: &Value, page: &Value) -> Value {
    let mut merged = existing.as_array().cloned().unwrap_or_default();
    for category in page.as_array().into_iter().flatten() {
        let id = category.get("id");
        if !merged.iter().any(|known| known.get("id") == id) {
            merged.push(category.clone());
        }
    }
    Value::Array(merged)
}
