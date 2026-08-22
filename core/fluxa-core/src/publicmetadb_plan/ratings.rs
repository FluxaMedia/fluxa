use super::helpers::{body_from_keys, build_url, extract_query, parse_args, plan};
use serde_json::Value;

pub(crate) fn publicmetadb_ratings_url(query_json: &str) -> Option<String> {
    let args = parse_args(query_json);
    let params = extract_query(&args, &["tmdb_id", "media_type", "label"]);
    if !params.iter().any(|(key, _)| key == "tmdb_id")
        || !params.iter().any(|(key, _)| key == "media_type")
    {
        return None;
    }
    Some(build_url("/ratings", &params))
}

pub(crate) fn publicmetadb_ratings_create_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let body = body_from_keys(&args, &["tmdb_id", "media_type", "score"], &["label"])?;
    plan("POST", build_url("/ratings", &[]), Some(body))
}

pub(crate) fn publicmetadb_ratings_delete_plan(id: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    plan("DELETE", build_url(&format!("/ratings/{id}"), &[]), None)
}
