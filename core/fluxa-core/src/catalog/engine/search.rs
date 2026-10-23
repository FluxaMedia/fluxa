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
    pending: u32,
    plan: Vec<String>,
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
        pending: 0,
        plan: Vec::new(),
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
    payload: &Value,
) -> Vec<EffectEnvelope> {
    if generation != engine.state.runtime.get(GenerationKey::Search) {
        return vec![];
    }
    let appending = engine.state.search.is_loading_more;
    if payload.get("request").is_some() {
        if result.status.is_ok() {
            merge_source(&mut engine.state.search, &result.value);
        }
        settle(&mut engine.state.search);
        return vec![];
    }
    if !result.status.is_ok() {
        engine.state.search.is_loading = false;
        engine.state.search.is_loading_more = false;
        if !appending {
            engine.state.search.error = normalize_error(result.error.clone());
        }
        return vec![];
    }
    if let Some(requests) = result.value.get("requests").and_then(Value::as_array) {
        return fan_out(engine, generation, payload, requests, appending);
    }
    engine.state.search.is_loading = false;
    engine.state.search.is_loading_more = false;
    let results = result
        .value
        .get("results")
        .cloned()
        .unwrap_or_else(|| result.value.clone());
    let search = &mut engine.state.search;
    search.error = Value::Null;
    search.results = results;
    search.categories = result
        .value
        .get("categories")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    search.grouping = result.value.get("grouping").cloned().unwrap_or(Value::Null);
    search.sources = Vec::new();
    search.has_more = false;
    vec![]
}

fn fan_out(
    engine: &mut HeadlessEngine,
    generation: u64,
    payload: &Value,
    requests: &[Value],
    appending: bool,
) -> Vec<EffectEnvelope> {
    let search = &mut engine.state.search;
    search.error = Value::Null;
    search.pending = requests.len() as u32;
    search.has_more = false;
    if requests.is_empty() {
        search.is_loading = false;
        search.is_loading_more = false;
        return vec![];
    }
    if appending {
        for source in &mut search.sources {
            source.has_more = false;
        }
    } else {
        search.plan = requests
            .iter()
            .filter_map(|request| request.get("categoryId").and_then(Value::as_str))
            .map(str::to_owned)
            .collect();
    }
    requests
        .iter()
        .map(|request| {
            let mut payload = payload.clone();
            payload["request"] = request.clone();
            engine.effect(EffectKind::RunSearch, generation, payload)
        })
        .collect()
}

fn merge_source(search: &mut SearchState, value: &Value) {
    if let Some(page) = value
        .get("page")
        .and_then(|page| serde_json::from_value::<SearchSource>(page.clone()).ok())
    {
        match search.sources.iter_mut().find(|known| known.id == page.id) {
            Some(known) => {
                known.loaded += page.loaded;
                known.has_more = page.has_more;
            }
            None => search.sources.push(page),
        }
    }
    let Some(source) = value.get("source").filter(|source| source.is_object()) else {
        return;
    };
    let Some(merged) = crate::catalog::search::merge_search_sources(&serde_json::json!([source]))
    else {
        return;
    };
    let mut categories = search.categories.as_array().cloned().unwrap_or_default();
    for category in merged
        .get("categories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        insert_category(&mut categories, category.clone(), &search.plan);
    }
    let mut results = Value::Array(Vec::new());
    for category in &categories {
        results = append_unique(&results, category.get("items").unwrap_or(&Value::Null));
    }
    search.results = results;
    search.categories = Value::Array(categories);
    search.grouping = crate::catalog::search::search_result_grouping_json(
        &serde_json::json!({"query": search.query, "results": search.results}).to_string(),
    )
    .and_then(|json| serde_json::from_str(&json).ok())
    .unwrap_or(Value::Null);
}

fn settle(search: &mut SearchState) {
    search.pending = search.pending.saturating_sub(1);
    if search.pending > 0 {
        return;
    }
    search.is_loading = false;
    search.is_loading_more = false;
    search.has_more = search.sources.iter().any(|source| source.has_more);
}

fn insert_category(categories: &mut Vec<Value>, category: Value, plan: &[String]) {
    let id = category.get("id").cloned();
    if let Some(known) = categories
        .iter_mut()
        .find(|known| known.get("id") == id.as_ref())
    {
        let items = append_unique(
            known.get("items").unwrap_or(&Value::Null),
            category.get("items").unwrap_or(&Value::Null),
        );
        known["items"] = items;
        return;
    }
    let rank = |category: &Value| {
        plan.iter()
            .position(|id| category.get("id").and_then(Value::as_str) == Some(id.as_str()))
            .unwrap_or(usize::MAX)
    };
    let at = categories
        .iter()
        .position(|known| rank(known) > rank(&category))
        .unwrap_or(categories.len());
    categories.insert(at, category);
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
