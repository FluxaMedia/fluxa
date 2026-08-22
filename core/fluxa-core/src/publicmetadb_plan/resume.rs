use super::helpers::{body_from_keys, build_url, extract_query, parse_args, plan};
use serde_json::Value;

const RESUME_QUERY_KEYS: &[&str] = &[
    "tmdb_id",
    "media_type",
    "season",
    "episode",
    "id_type",
    "id_value",
    "page",
    "perPage",
];

pub(crate) fn publicmetadb_resume_url(query_json: &str) -> String {
    build_url(
        "/resume",
        &extract_query(&parse_args(query_json), RESUME_QUERY_KEYS),
    )
}

pub(crate) fn publicmetadb_resume_save_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let body = body_from_keys(
        &args,
        &["media_type", "position_ms", "runtime_ms"],
        &["tmdb_id", "season", "episode", "id_type", "id_value"],
    )?;
    plan("POST", build_url("/resume", &[]), Some(body))
}

pub(crate) fn publicmetadb_resume_batch_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let items = args.get("items")?.as_array()?;
    if items.is_empty() || items.len() > 50 {
        return None;
    }
    let body = body_from_keys(&args, &["items"], &[])?;
    plan("POST", build_url("/resume/batch", &[]), Some(body))
}

pub(crate) fn publicmetadb_resume_delete_plan(id: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    plan("DELETE", build_url(&format!("/resume/{id}"), &[]), None)
}
