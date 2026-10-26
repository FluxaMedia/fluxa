use super::helpers::{build_url, plan};
use serde_json::{Value, json};

pub(crate) const LIST_ITEMS_QUERY_KEYS: &[&str] = &[
    "cursor",
    "limit",
    "offset",
    "append_to_response",
    "extended",
    "mediatype",
    "filter_title",
    "filter_genre",
    "genre_operator",
    "released_from",
    "released_to",
    "filter_score_min",
    "filter_score_max",
    "sort",
    "order",
    "unified",
];

pub(crate) fn mdblist_content_type(content_type: &str) -> &'static str {
    if matches!(
        content_type.trim().to_ascii_lowercase().as_str(),
        "series" | "show" | "tv" | "anime"
    ) {
        "show"
    } else {
        "movie"
    }
}

pub(crate) fn mdblist_media_info_url(
    provider: &str,
    media_type: &str,
    media_id: &str,
    append_to_response: Option<&str>,
) -> String {
    let mut params = Vec::new();
    if let Some(fields) = append_to_response.filter(|value| !value.is_empty()) {
        params.push(("append_to_response".to_string(), fields.to_string()));
    }
    build_url(&format!("/{provider}/{media_type}/{media_id}/"), &params)
}

pub(crate) fn mdblist_media_info_batch_plan(
    provider: &str,
    media_type: &str,
    ids: &[String],
) -> Option<String> {
    if ids.is_empty() {
        return None;
    }
    plan(
        "POST",
        build_url(&format!("/{provider}/{media_type}/"), &[]),
        Some(json!({ "ids": ids })),
    )
}

pub(crate) fn mdblist_media_ratings_from_response_json(response_json: &str) -> Option<String> {
    let response: Value = serde_json::from_str(response_json).ok()?;
    let ratings = response.get("ratings")?.as_array()?;
    let mut normalized = serde_json::Map::new();
    for entry in ratings {
        let source = entry.get("source").and_then(Value::as_str)?;
        let value = entry
            .get("value")
            .filter(|value| !value.is_null())
            .or_else(|| entry.get("score").filter(|score| !score.is_null()));
        if let Some(value) = value {
            normalized.insert(source.to_string(), value.clone());
        }
    }
    serde_json::to_string(&Value::Object(normalized)).ok()
}
