use super::helpers::{build_url, extract_query, plan};
use super::lists::items_by_type_body;
use serde_json::{Value, json};

const WATCHLIST_QUERY_KEYS: &[&str] = &[
    "cursor",
    "limit",
    "offset",
    "append_to_response",
    "extended",
    "sort",
    "order",
];

pub(crate) fn mdblist_watchlist_items_url(media_type: Option<&str>, args_json: &str) -> String {
    let args: Value = serde_json::from_str(args_json).unwrap_or(json!({}));
    let params = extract_query(&args, WATCHLIST_QUERY_KEYS);
    match media_type.filter(|value| !value.is_empty()) {
        Some(media_type) => build_url(&format!("/watchlist/items/{media_type}"), &params),
        None => build_url("/watchlist/items", &params),
    }
}

pub(crate) fn mdblist_watchlist_mutate_plan(action: &str, items_json: &str) -> Option<String> {
    let body = items_by_type_body(items_json)?;
    plan(
        "POST",
        build_url(&format!("/watchlist/items/{action}"), &[]),
        Some(body),
    )
}

const SYNC_LIST_QUERY_KEYS: &[&str] = &[
    "cursor",
    "limit",
    "offset",
    "since",
    "page",
    "mediatype",
    "sort",
    "order",
    "unified",
    "append_to_response",
    "extended",
];

pub(crate) fn mdblist_sync_get_url(category: &str, args_json: &str) -> Option<String> {
    let path = match category {
        "watched" | "ratings" | "collection" | "dropped" | "journal" | "last_activities"
        | "now-playing" | "playback" => format!("/sync/{category}"),
        _ => return None,
    };
    let args: Value = serde_json::from_str(args_json).unwrap_or(json!({}));
    let keys: &[&str] = match category {
        "journal" => &["since", "limit", "cursor"],
        "last_activities" | "now-playing" | "playback" => &[],
        _ => SYNC_LIST_QUERY_KEYS,
    };
    Some(build_url(&path, &extract_query(&args, keys)))
}

pub(crate) fn mdblist_sync_mutate_plan(
    category: &str,
    remove: bool,
    items_json: &str,
) -> Option<String> {
    if !["watched", "ratings", "collection", "dropped"].contains(&category) {
        return None;
    }
    let body = items_by_type_body(items_json)?;
    let path = if remove {
        format!("/sync/{category}/remove")
    } else {
        format!("/sync/{category}")
    };
    plan("POST", build_url(&path, &[]), Some(body))
}

const UPNEXT_QUERY_KEYS: &[&str] = &["limit", "offset", "hide_unreleased", "days"];

pub(crate) fn mdblist_upnext_url(section: Option<&str>, args_json: &str) -> Option<String> {
    let path = match section {
        None => "/upnext".to_string(),
        Some("upcoming") => "/upnext/upcoming".to_string(),
        Some("watchlist") => "/upnext/watchlist".to_string(),
        Some(_) => return None,
    };
    let args: Value = serde_json::from_str(args_json).unwrap_or(json!({}));
    Some(build_url(&path, &extract_query(&args, UPNEXT_QUERY_KEYS)))
}
