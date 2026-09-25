use super::addon_catalog::discover_catalog_request_types_json;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;

// Discover aggregates results from every installed addon's catalogs — with enough
// addons installed, that's thousands of items in one IPC payload. Cap it after
// dedup/sort so a single discover fetch can't balloon into multi-megabyte responses.
const DISCOVER_MAX_ITEMS: usize = 400;

pub(crate) fn merge_discover_pages_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let base = request.get("baseItems")?.as_array()?;
    let existing = request.get("existingItems")?.as_array()?;
    let incoming = request.get("incomingItems")?.as_array()?;
    let mut seen = HashSet::new();
    let mut merged = Vec::new();
    for item in base.iter().chain(existing).chain(incoming) {
        let id = item.get("id").and_then(Value::as_str).unwrap_or("");
        if !id.is_empty() && seen.insert(id) {
            merged.push(item.clone());
        }
    }
    let existing_ids: HashSet<&str> = base
        .iter()
        .chain(existing)
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .collect();
    let mut appended_seen = existing_ids.clone();
    let appended: Vec<&Value> = incoming
        .iter()
        .filter(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| appended_seen.insert(id))
        })
        .collect();
    serde_json::to_string(&json!({
        "items": merged,
        "appendedItems": appended,
        "exhausted": incoming.is_empty() || appended.is_empty(),
    }))
    .ok()
}

/// Merge Discover's per-catalog batches once in Core so every host preserves
/// the same source identity and deduplication order for a selected catalog.
pub(crate) fn merge_discover_sources_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
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
    serde_json::to_string(&json!({"results": results, "resultSources": result_sources})).ok()
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscoverSortRequest {
    #[serde(default)]
    items: Vec<Value>,
    #[serde(default)]
    sort_by: Option<String>,
    #[serde(default)]
    ascending: bool,
    #[serde(default)]
    content_type_filter: Option<String>,
    #[serde(default)]
    genre_filter: Option<String>,
}

pub(crate) fn discover_selection_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let content_type = request
        .get("contentType")
        .and_then(Value::as_str)
        .unwrap_or("movie");
    let catalogs = request
        .get("catalogs")
        .and_then(Value::as_array)?
        .iter()
        .filter(|catalog| catalog.get("type").and_then(Value::as_str) == Some(content_type))
        .cloned()
        .collect::<Vec<_>>();
    let requested_key = request.get("selectedCatalogKey").and_then(Value::as_str);
    let catalog = requested_key
        .and_then(|key| {
            catalogs
                .iter()
                .find(|catalog| catalog.get("key").and_then(Value::as_str) == Some(key))
        })
        .or_else(|| catalogs.first())
        .cloned();
    let selected_key = catalog
        .as_ref()
        .and_then(|value| value.get("key"))
        .and_then(Value::as_str);
    let extra = catalog
        .as_ref()
        .and_then(|value| value.get("extras"))
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .cloned();
    let extra_options_contains = |value: &str| {
        extra
            .as_ref()
            .and_then(|extra| extra.get("options"))
            .and_then(Value::as_array)
            .is_some_and(|options| options.iter().any(|option| option.as_str() == Some(value)))
    };
    let extra_required = extra
        .as_ref()
        .and_then(|extra| extra.get("isRequired"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let requested_extra = request.get("extraValue").and_then(Value::as_str);
    let extra_value = requested_extra
        .filter(|value| extra_options_contains(value))
        .or_else(|| {
            extra
                .as_ref()
                .and_then(|extra| extra.get("default"))
                .and_then(Value::as_str)
                .filter(|value| extra_required && extra_options_contains(value))
        })
        .or_else(|| {
            extra_required
                .then(|| {
                    extra
                        .as_ref()
                        .and_then(|extra| extra.get("options"))
                        .and_then(Value::as_array)
                        .and_then(|options| options.first())
                        .and_then(Value::as_str)
                })
                .flatten()
        });
    let extra_name = extra
        .as_ref()
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str);
    let key = format!(
        "{}|{}|{}",
        selected_key.unwrap_or(""),
        extra_name.unwrap_or(""),
        extra_value.unwrap_or("")
    );
    serde_json::to_string(&json!({"catalogs": catalogs, "selectedCatalogKey": selected_key, "selectedCatalog": catalog, "selectedExtra": extra, "extraValue": extra_value, "key": key})).ok()
}

pub(crate) fn discover_catalog_candidates_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let content_type = request
        .get("contentType")
        .and_then(Value::as_str)
        .unwrap_or("all");
    let genre = request
        .get("genre")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty());
    let provider_terms = request
        .get("provider")
        .and_then(Value::as_str)
        .map(crate::content_identity::provider_search_terms)
        .unwrap_or_default();
    let limit = request
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(5)
        .clamp(1, 20) as usize;
    let requested_key = request.get("catalogKey").and_then(Value::as_str);
    let catalogs = request.get("catalogs")?.as_array()?;
    let matches = |catalog: &&Value| {
        let kind = catalog.get("type").and_then(Value::as_str).unwrap_or("");
        let type_ok = content_type == "all" || kind == content_type || kind == "all";
        let genre_ok = genre.is_none_or(|wanted| {
            catalog
                .get("genres")
                .and_then(Value::as_array)
                .is_some_and(|genres| {
                    genres.iter().any(|value| {
                        value
                            .as_str()
                            .is_some_and(|value| value.eq_ignore_ascii_case(wanted))
                    })
                })
        });
        let provider_ok = provider_terms.is_empty() || {
            let haystack = format!(
                "{} {} {}",
                catalog.get("label").and_then(Value::as_str).unwrap_or(""),
                catalog
                    .get("transportUrl")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                catalog.get("id").and_then(Value::as_str).unwrap_or("")
            )
            .to_lowercase();
            provider_terms.iter().any(|term| haystack.contains(term))
        };
        type_ok && genre_ok && provider_ok
    };
    let selected = requested_key.and_then(|key| {
        catalogs.iter().find(|catalog| {
            catalog.get("key").and_then(Value::as_str) == Some(key) && matches(&catalog)
        })
    });
    let selected_or_candidates = selected.into_iter().chain(catalogs.iter().filter(matches));
    let keys = selected_or_candidates
        .filter_map(|catalog| catalog.get("key").and_then(Value::as_str))
        .take(limit)
        .map(str::to_string)
        .collect::<Vec<_>>();
    Some(json!(keys).to_string())
}

pub(crate) fn discover_sort_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<DiscoverSortRequest>(request_json).ok()?;
    let content_type = request.content_type_filter.as_deref().unwrap_or("");
    let genre = request.genre_filter.as_deref().unwrap_or("").to_lowercase();
    let sort_by = match request.sort_by.as_deref().unwrap_or("default") {
        "top" => "rating",
        "newest" => "year",
        other => other,
    };

    let mut filtered: Vec<&Value> = request
        .items
        .iter()
        .filter(|item| {
            let type_ok = content_type.is_empty()
                || content_type == "anime"
                || item
                    .get("type")
                    .and_then(Value::as_str)
                    .is_some_and(|t| t == content_type);
            let genre_ok = genre.is_empty()
                || item
                    .get("genres")
                    .and_then(Value::as_array)
                    .is_some_and(|g| {
                        g.iter()
                            .any(|gv| gv.as_str().is_some_and(|s| s.to_lowercase() == genre))
                    });
            type_ok && genre_ok
        })
        .collect();

    let mut seen_ids: HashSet<&str> = HashSet::with_capacity(filtered.len());
    filtered.retain(|item| match item.get("id").and_then(Value::as_str) {
        Some(id) => seen_ids.insert(id),
        None => true,
    });

    match sort_by {
        "year" => {
            filtered.sort_by(|a, b| {
                let ya = a
                    .get("releaseInfo")
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                let yb = b
                    .get("releaseInfo")
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                if request.ascending {
                    ya.cmp(&yb)
                } else {
                    yb.cmp(&ya)
                }
            });
        }
        "rating" => {
            filtered.sort_by(|a, b| {
                let ra = a.get("imdbRating").and_then(Value::as_f64).unwrap_or(0.0);
                let rb = b.get("imdbRating").and_then(Value::as_f64).unwrap_or(0.0);
                if request.ascending {
                    ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
                }
            });
        }
        "name" => {
            filtered.sort_by(|a, b| {
                let na = a.get("name").and_then(Value::as_str).unwrap_or("");
                let nb = b.get("name").and_then(Value::as_str).unwrap_or("");
                if request.ascending {
                    na.cmp(nb)
                } else {
                    nb.cmp(na)
                }
            });
        }
        _ => {}
    }

    let total_count = filtered.len();
    filtered.truncate(DISCOVER_MAX_ITEMS);

    serde_json::to_string(&json!({
        "items": filtered,
        "sortBy": sort_by,
        "ascending": request.ascending,
        "totalCount": total_count
    }))
    .ok()
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
