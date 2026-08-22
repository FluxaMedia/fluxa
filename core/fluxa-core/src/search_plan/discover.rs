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
