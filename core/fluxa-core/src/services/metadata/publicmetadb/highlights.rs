use super::helpers::{body_from_keys, build_url, extract_query, parse_args, plan};
use serde_json::Value;

pub(crate) fn publicmetadb_highlights_url(query_json: &str) -> Option<String> {
    let args = parse_args(query_json);
    let params = extract_query(&args, &["tmdb_id", "media_type", "season", "episode"]);
    if !params.iter().any(|(key, _)| key == "tmdb_id")
        || !params.iter().any(|(key, _)| key == "media_type")
    {
        return None;
    }
    Some(build_url("/highlights", &params))
}

pub(crate) fn publicmetadb_highlights_create_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let body = body_from_keys(
        &args,
        &[
            "tmdb_id",
            "media_type",
            "highlight_start_ms",
            "highlight_end_ms",
        ],
        &["season", "episode", "description"],
    )?;
    plan("POST", build_url("/highlights", &[]), Some(body))
}

pub(crate) fn publicmetadb_highlights_delete_plan(id: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    plan("DELETE", build_url(&format!("/highlights/{id}"), &[]), None)
}
