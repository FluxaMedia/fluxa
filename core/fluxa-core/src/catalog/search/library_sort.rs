use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibrarySortRequest {
    #[serde(default)]
    items: Vec<Value>,
    #[serde(default)]
    sort_by: Option<String>,
    #[serde(default)]
    ascending: bool,
    #[serde(default)]
    type_filter: Option<String>,
    #[serde(default)]
    status_filter: Option<String>,
}

pub(crate) fn library_sort_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<LibrarySortRequest>(request_json).ok()?;
    let type_filter = request.type_filter.as_deref().unwrap_or("").to_lowercase();
    let status_filter = request
        .status_filter
        .as_deref()
        .unwrap_or("")
        .to_lowercase();
    let sort_by = request.sort_by.as_deref().unwrap_or("added");

    let mut filtered: Vec<&Value> = request
        .items
        .iter()
        .filter(|item| {
            let type_ok = type_filter.is_empty()
                || item
                    .get("type")
                    .and_then(Value::as_str)
                    .is_some_and(|t| t.to_lowercase() == type_filter);
            let status_ok = status_filter.is_empty()
                || item
                    .get("status")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.to_lowercase() == status_filter);
            type_ok && status_ok
        })
        .collect();

    match sort_by {
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
        "progress" => {
            filtered.sort_by(|a, b| {
                let pa = a.get("timeOffset").and_then(Value::as_i64).unwrap_or(0);
                let pb = b.get("timeOffset").and_then(Value::as_i64).unwrap_or(0);
                if request.ascending {
                    pa.cmp(&pb)
                } else {
                    pb.cmp(&pa)
                }
            });
        }
        _ => {}
    }

    serde_json::to_string(&json!({
        "items": filtered,
        "sortBy": sort_by,
        "totalCount": filtered.len()
    }))
    .ok()
}
