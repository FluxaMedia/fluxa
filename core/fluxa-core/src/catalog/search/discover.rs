use super::addon_catalog::discover_catalog_request_types_json;
use serde_json::{Value, json};
use std::collections::HashSet;

// Discover aggregates results from every installed addon's catalogs — with enough
// addons installed, that's thousands of items in one IPC payload. Cap it after
// dedup/sort so a single discover fetch can't balloon into multi-megabyte responses.

/// Merge Discover's per-catalog batches once in Core so every host preserves
/// the same source identity and deduplication order for a selected catalog.
pub(crate) fn merge_discover_sources_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<Value>(request_json).ok()?;
    serde_json::to_string(&merge_discover_sources(&request)?).ok()
}

pub(crate) fn merge_discover_sources(request: &Value) -> Option<Value> {
    let sources = request.get("sources")?.as_array()?;
    let mut results = Vec::new();
    let mut result_sources = serde_json::Map::new();
    let mut seen = HashSet::new();

    for source in sources {
        let items = source.get("items").and_then(Value::as_array)?;
        let identity = json!({
            "transportUrl": source.get("transportUrl").cloned().unwrap_or(Value::Null),
            "catalogId": source.get("catalogId").cloned().unwrap_or(Value::Null),
            "type": source.get("type").cloned().unwrap_or(Value::Null),
            "genre": source.get("genre").cloned().unwrap_or(Value::Null),
        });
        for item in items {
            let id = item.get("id").and_then(Value::as_str).unwrap_or("");
            let content_type = item
                .get("type")
                .and_then(Value::as_str)
                .or_else(|| source.get("type").and_then(Value::as_str))
                .unwrap_or("");
            if id.is_empty() || !seen.insert((content_type.to_owned(), id.to_owned())) {
                continue;
            }
            results.push(item.clone());
            result_sources
                .entry(format!("{content_type}:{id}"))
                .or_insert_with(|| identity.clone());
            result_sources
                .entry(id.to_owned())
                .or_insert(identity.clone());
        }
    }
    Some(json!({"results": results, "resultSources": result_sources}))
}

/// Resolve the concrete catalog requests for a Discover effect in one place.
/// Hosts should only perform transport for these requests; filter aliases,
/// catalog type expansion, and source identity stay consistent across them.
pub(crate) fn discover_source_requests_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let filters = request.get("filters")?;
    let transport_url = filters
        .get("transportUrl")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())?;
    let catalog_id = filters
        .get("catalogId")
        .or_else(|| filters.get("id"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())?;
    let content_type = request
        .get("contentType")
        .and_then(Value::as_str)
        .unwrap_or("movie");
    let catalog_type = filters
        .get("catalogType")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(content_type);
    let request_types: Vec<String> =
        serde_json::from_str(&discover_catalog_request_types_json(catalog_type)?).ok()?;
    let extra = filters
        .get("extra")
        .filter(|value| value.is_object())
        .cloned()
        .unwrap_or_else(|| {
            let mut extra = serde_json::Map::new();
            for key in ["genre", "search", "skip"] {
                if let Some(value) = filters.get(key) {
                    extra.insert(key.to_owned(), value.clone());
                }
            }
            Value::Object(extra)
        });
    let genre = extra.get("genre").cloned().unwrap_or(Value::Null);
    let requests = request_types
        .into_iter()
        .map(|content_type| {
            json!({
                "transportUrl": transport_url,
                "catalogId": catalog_id,
                "type": content_type,
                "genre": genre.clone(),
                "extra": extra.clone(),
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&requests).ok()
}

#[cfg(test)]
mod tests {
    use super::{discover_source_requests_json, merge_discover_sources_json};
    use serde_json::{Value, json};

    #[test]
    fn discover_source_merge_deduplicates_and_keeps_first_source_for_all_hosts() {
        let merged = merge_discover_sources_json(
            &json!({"sources": [
                {"transportUrl":"https://one.test/manifest.json", "catalogId":"featured", "type":"movie", "genre":"Drama", "items":[
                    {"id":"same", "type":"movie", "name":"First"},
                    {"id":"episode", "type":"series", "name":"Series"}
                ]},
                {"transportUrl":"https://two.test/manifest.json", "catalogId":"featured", "type":"movie", "items":[
                    {"id":"same", "type":"movie", "name":"Duplicate"},
                    {"id":"unique", "type":"movie", "name":"Unique"}
                ]}
            ]}).to_string(),
        ).and_then(|json| serde_json::from_str::<Value>(&json).ok()).expect("merged result");

        assert_eq!(merged["results"].as_array().map(Vec::len), Some(3));
        assert_eq!(merged["results"][0]["name"], "First");
        assert_eq!(
            merged["resultSources"]["movie:same"]["transportUrl"],
            "https://one.test/manifest.json"
        );
        assert_eq!(merged["resultSources"]["same"]["catalogId"], "featured");
        assert_eq!(merged["resultSources"]["series:episode"]["type"], "movie");
        assert_eq!(
            merged["resultSources"]["unique"]["transportUrl"],
            "https://two.test/manifest.json"
        );
    }

    #[test]
    fn discover_requests_normalize_catalog_identity_and_expand_all_type_in_core() {
        let requests = discover_source_requests_json(
            &json!({
                "contentType": "all",
                "filters": {
                    "transportUrl": "https://addon.test/manifest.json",
                    "id": "top",
                    "extra": {"genre": "Drama", "search": "pilot"}
                }
            })
            .to_string(),
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("shared request plan");

        assert_eq!(requests.as_array().map(Vec::len), Some(2));
        assert_eq!(requests[0]["type"], "movie");
        assert_eq!(requests[1]["type"], "series");
        assert_eq!(requests[0]["catalogId"], "top");
        assert_eq!(requests[0]["genre"], "Drama");
        assert_eq!(requests[1]["extra"]["search"], "pilot");
    }
}
