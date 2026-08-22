use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;

pub(crate) fn search_suggestions_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let needle = request.get("needle")?.as_str()?.trim().to_ascii_lowercase();
    if needle.len() < 2 {
        return Some("[]".to_string());
    }
    let limit = request.get("limit").and_then(Value::as_u64).unwrap_or(8) as usize;
    let mut seen_ids = HashSet::new();
    let mut seen_names = HashSet::new();
    let mut prefix = Vec::new();
    let mut contains = Vec::new();
    let values: Vec<&Value> =
        if let Some(categories) = request.get("categories").and_then(Value::as_array) {
            categories
                .iter()
                .flat_map(|category| {
                    category
                        .get("items")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                })
                .collect()
        } else {
            request
                .get("items")
                .and_then(Value::as_array)?
                .iter()
                .collect()
        };
    for item in values {
        let id = item.get("id").and_then(Value::as_str).unwrap_or("");
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase();
        if !name.contains(&needle) || !seen_ids.insert(id) || !seen_names.insert(name.clone()) {
            continue;
        }
        if name.starts_with(&needle) {
            prefix.push(item.clone());
        } else {
            contains.push(item.clone());
        }
    }
    prefix.extend(contains);
    prefix.truncate(limit);
    serde_json::to_string(&prefix).ok()
}

pub(crate) fn search_screen_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let query = request
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let search_query = request
        .get("searchQuery")
        .and_then(Value::as_str)
        .unwrap_or("");
    let search_categories = request
        .get("searchCategories")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let cached_categories = request
        .get("cachedCategories")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let has_cache = request
        .get("hasCache")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let search_loading = request
        .get("searchLoading")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let matches = search_query == query;
    let raw = if matches && !search_categories.is_empty() {
        search_categories.clone()
    } else {
        cached_categories
    };
    let type_filter = request
        .get("typeFilter")
        .and_then(Value::as_str)
        .unwrap_or("");
    let categories = raw
        .into_iter()
        .filter_map(|mut category| {
            let items = category.get("items")?.as_array()?;
            let visible = items
                .iter()
                .filter(|item| {
                    type_filter.is_empty()
                        || item.get("type").and_then(Value::as_str) == Some(type_filter)
                })
                .cloned()
                .collect::<Vec<_>>();
            if visible.is_empty() {
                return None;
            }
            category
                .as_object_mut()?
                .insert("items".to_string(), Value::Array(visible));
            Some(category)
        })
        .collect::<Vec<_>>();
    let result_count = categories
        .iter()
        .filter_map(|category| category.get("items").and_then(Value::as_array))
        .map(Vec::len)
        .sum::<usize>();
    serde_json::to_string(&json!({
        "query": query,
        "queryEligible": query.chars().count() >= 2,
        "shouldDispatch": query.chars().count() >= 2 && !has_cache && !(matches && search_loading),
        "shouldCache": matches && !search_categories.is_empty(),
        "categories": categories,
        "resultCount": result_count,
        "categoryCount": categories.len(),
        "isLoading": search_loading && !has_cache,
    }))
    .ok()
}

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
    let sources: Vec<Value> = serde_json::from_str(sources_json).ok()?;
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
    serde_json::to_string(&json!({ "results": results, "categories": categories })).ok()
}

pub(crate) fn recent_searches_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let operation = request
        .get("operation")
        .and_then(Value::as_str)
        .unwrap_or("normalize");
    let mut items = request
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    match operation {
        "add" => {
            let query = request
                .get("query")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if query.chars().count() >= 2 {
                items.retain(|item| {
                    item.get("query")
                        .and_then(Value::as_str)
                        .is_none_or(|value| !value.eq_ignore_ascii_case(query))
                });
                let mut item = json!({"query": query});
                if let Some(meta) = request.get("meta").filter(|value| !value.is_null())
                    && let Some(item) = item.as_object_mut()
                {
                    item.insert("meta".to_string(), meta.clone());
                }
                items.insert(0, item);
            }
        }
        "remove" => {
            let query = request.get("query").and_then(Value::as_str).unwrap_or("");
            items.retain(|item| item.get("query").and_then(Value::as_str) != Some(query));
        }
        "clear" => items.clear(),
        "normalize" => {}
        _ => return None,
    }
    let mut seen = std::collections::HashSet::new();
    let normalized = items
        .into_iter()
        .filter_map(|item| {
            let (query, meta) = match item {
                Value::String(query) => (query.trim().to_string(), None),
                Value::Object(object) => (
                    object.get("query")?.as_str()?.trim().to_string(),
                    object
                        .get("meta")
                        .filter(|value| value.is_object())
                        .cloned(),
                ),
                _ => return None,
            };
            if query.is_empty() || !seen.insert(query.to_lowercase()) {
                return None;
            }
            Some(match meta {
                Some(meta) => json!({"query": query, "meta": meta}),
                None => json!({"query": query}),
            })
        })
        .take(8)
        .collect::<Vec<_>>();
    serde_json::to_string(&normalized).ok()
}
