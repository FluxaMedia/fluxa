use super::helpers::{body_from_keys, build_url, extract_query, parse_args, plan};
use serde_json::Value;

pub(crate) fn publicmetadb_watched_url(query_json: &str) -> String {
    build_url(
        "/watched",
        &extract_query(&parse_args(query_json), &["page", "perPage"]),
    )
}

pub(crate) fn publicmetadb_watched_mark_plan(args_json: &str, dedupe: bool) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let body = body_from_keys(
        &args,
        &["tmdb_id", "media_type"],
        &["season", "episode", "watched_at"],
    )?;
    let params = if dedupe {
        vec![("dedupe".to_string(), "true".to_string())]
    } else {
        vec![]
    };
    plan("POST", build_url("/watched", &params), Some(body))
}

pub(crate) fn publicmetadb_watched_edit_date_plan(id: &str, args_json: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    let args: Value = serde_json::from_str(args_json).ok()?;
    let body = body_from_keys(&args, &["watched_at"], &[])?;
    plan(
        "PATCH",
        build_url(&format!("/watched/{id}"), &[]),
        Some(body),
    )
}

pub(crate) fn publicmetadb_watched_delete_plan(id: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    plan("DELETE", build_url(&format!("/watched/{id}"), &[]), None)
}

pub(crate) fn publicmetadb_watched_bulk_delete_plan(query_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(query_json).ok()?;
    let params = extract_query(&args, &["tmdb_id", "media_type", "season", "episode"]);
    if !params.iter().any(|(key, _)| key == "tmdb_id")
        || !params.iter().any(|(key, _)| key == "media_type")
    {
        return None;
    }
    plan("DELETE", build_url("/watched", &params), None)
}
