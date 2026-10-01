use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchGroupingRequest {
    #[serde(default)]
    results: Vec<Value>,
    #[serde(default)]
    query: String,
}

pub(crate) fn search_result_grouping_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<SearchGroupingRequest>(request_json).ok()?;
    let mut movies: Vec<&Value> = Vec::new();
    let mut series: Vec<&Value> = Vec::new();
    let mut other: Vec<&Value> = Vec::new();
    for item in &request.results {
        match item.get("type").and_then(Value::as_str).unwrap_or("") {
            "movie" => movies.push(item),
            "series" | "anime" => series.push(item),
            _ => other.push(item),
        }
    }
    let mut groups = Vec::new();
    if !movies.is_empty() {
        groups.push(json!({ "type": "movie", "items": movies }));
    }
    if !series.is_empty() {
        groups.push(json!({ "type": "series", "items": series }));
    }
    if !other.is_empty() {
        groups.push(json!({ "type": "other", "items": other }));
    }
    serde_json::to_string(&json!({
        "groups": groups,
        "totalCount": request.results.len(),
        "query": request.query
    }))
    .ok()
}

/// Merges per-source search result batches (one per addon catalog request, plus TMDB
/// builtin batches) into the flat results list and category descriptors the search
/// screen renders — dropping empty sources rather than surfacing zero-result categories.
pub(crate) fn merge_search_sources_json(sources_json: &str) -> Option<String> {
    let sources = serde_json::from_str::<Value>(sources_json).ok()?;
    serde_json::to_string(&merge_search_sources(&sources)?).ok()
}

pub(crate) fn merge_search_sources(sources: &Value) -> Option<Value> {
    let sources = sources.as_array()?;
    let mut categories: Vec<Value> = Vec::new();
    let mut results: Vec<Value> = Vec::new();
    for source in sources {
        let items = source
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if items.is_empty() {
            continue;
        }
        results.extend(items.iter().cloned());
        let name = source.get("name").cloned().unwrap_or(Value::Null);
        categories.push(json!({
            "id": source.get("id").cloned().unwrap_or(Value::Null),
            "name": name.clone(),
            "semanticName": source.get("semanticName").cloned().unwrap_or(name),
            "type": source.get("type").cloned().unwrap_or(Value::Null),
            "addonName": source.get("addonName").cloned().unwrap_or(Value::Null),
            "catalogId": source.get("catalogId").cloned().unwrap_or(Value::Null),
            "items": items,
        }));
    }
    Some(json!({ "results": results, "categories": categories }))
}
