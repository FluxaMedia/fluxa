use super::helpers::{body_from_keys, build_url, extract_query, parse_args, plan};
use serde_json::Value;

pub(crate) fn publicmetadb_lists_url(query_json: &str) -> String {
    build_url(
        "/lists",
        &extract_query(&parse_args(query_json), &["page", "perPage"]),
    )
}

pub(crate) fn publicmetadb_lists_create_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let body = body_from_keys(&args, &["name"], &["description", "is_public", "type"])?;
    plan("POST", build_url("/lists", &[]), Some(body))
}

pub(crate) fn publicmetadb_lists_delete_plan(list_id: &str) -> Option<String> {
    if list_id.is_empty() {
        return None;
    }
    plan("DELETE", build_url(&format!("/lists/{list_id}"), &[]), None)
}

pub(crate) fn publicmetadb_list_items_url(list_id: &str, query_json: &str) -> Option<String> {
    if list_id.is_empty() {
        return None;
    }
    let params = extract_query(&parse_args(query_json), &["page", "perPage"]);
    Some(build_url(&format!("/lists/{list_id}/items"), &params))
}

pub(crate) fn publicmetadb_list_items_add_plan(list_id: &str, args_json: &str) -> Option<String> {
    if list_id.is_empty() {
        return None;
    }
    let args: Value = serde_json::from_str(args_json).ok()?;
    let body = body_from_keys(&args, &["tmdb_id", "media_type"], &[])?;
    plan(
        "POST",
        build_url(&format!("/lists/{list_id}/items"), &[]),
        Some(body),
    )
}

pub(crate) fn publicmetadb_list_items_remove_plan(list_id: &str, item_id: &str) -> Option<String> {
    if list_id.is_empty() || item_id.is_empty() {
        return None;
    }
    plan(
        "DELETE",
        build_url(&format!("/lists/{list_id}/items/{item_id}"), &[]),
        None,
    )
}
