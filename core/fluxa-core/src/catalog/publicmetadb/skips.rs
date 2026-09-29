use super::helpers::{body_from_keys, build_url, extract_query, parse_args, plan};
use serde_json::Value;

pub(crate) fn publicmetadb_skips_url(query_json: &str) -> Option<String> {
    let args = parse_args(query_json);
    let params = extract_query(
        &args,
        &["tmdb_id", "media_type", "season", "episode", "source"],
    );
    if !params.iter().any(|(key, _)| key == "tmdb_id")
        || !params.iter().any(|(key, _)| key == "media_type")
    {
        return None;
    }
    Some(build_url("/skips", &params))
}

pub(crate) fn publicmetadb_skips_create_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let body = body_from_keys(
        &args,
        &["tmdb_id", "media_type", "season", "episode"],
        &[
            "source",
            "intro_start_ms",
            "intro_end_ms",
            "credits_start_ms",
            "credits_end_ms",
        ],
    )?;
    plan("POST", build_url("/skips", &[]), Some(body))
}

pub(crate) fn publicmetadb_skips_delete_plan(id: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    plan("DELETE", build_url(&format!("/skips/{id}"), &[]), None)
}
