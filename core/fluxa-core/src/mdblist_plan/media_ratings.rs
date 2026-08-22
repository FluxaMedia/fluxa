use super::helpers::{build_url, extract_query, extract_repeated, plan};
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

pub(crate) fn mdblist_watchprovider_links_url(
    provider: &str,
    media_type: &str,
    media_id: &str,
) -> String {
    build_url(
        &format!("/{provider}/{media_type}/{media_id}/watchprovider_links/"),
        &[],
    )
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

pub(crate) fn mdblist_ratings_batch_plan(
    media_type: &str,
    return_rating: &str,
    provider: &str,
    ids: &[String],
) -> Option<String> {
    if ids.is_empty() {
        return None;
    }
    plan(
        "POST",
        build_url(&format!("/rating/{media_type}/{return_rating}"), &[]),
        Some(json!({ "provider": provider, "ids": ids })),
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

pub(crate) fn mdblist_search_url(media_type: &str, args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let params = extract_query(
        &args,
        &[
            "query",
            "quick_search",
            "limit_by_score",
            "sort_by_score",
            "year",
            "limit",
        ],
    );
    Some(build_url(&format!("/search/{media_type}"), &params))
}

pub(crate) fn mdblist_genres_url(anime: Option<bool>) -> String {
    let mut params = Vec::new();
    if let Some(anime) = anime {
        params.push((
            "anime".to_string(),
            if anime { "1" } else { "0" }.to_string(),
        ));
    }
    build_url("/genres", &params)
}

pub(crate) fn mdblist_catalog_url(media_type: &str, args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let mut params = extract_query(
        &args,
        &[
            "genre_mode",
            "country",
            "language",
            "score_min",
            "score_max",
            "released_from",
            "released_to",
            "year_min",
            "year_max",
            "runtime_min",
            "runtime_max",
            "sort",
            "sort_order",
            "limit",
            "cursor",
        ],
    );
    extract_repeated(&args, "genre", &mut params);
    Some(build_url(&format!("/catalog/{media_type}"), &params))
}
