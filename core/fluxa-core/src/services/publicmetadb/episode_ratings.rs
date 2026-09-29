use super::helpers::{body_from_keys, build_url, extract_query, parse_args, plan};
use serde_json::Value;

pub(crate) fn publicmetadb_episode_ratings_url(query_json: &str) -> Option<String> {
    let args = parse_args(query_json);
    let params = extract_query(
        &args,
        &[
            "tmdb_id",
            "media_type",
            "season",
            "episode",
            "label",
            "perPage",
        ],
    );
    if !params.iter().any(|(key, _)| key == "tmdb_id")
        || !params.iter().any(|(key, _)| key == "media_type")
    {
        return None;
    }
    Some(build_url("/episode-ratings", &params))
}

pub(crate) fn publicmetadb_episode_ratings_create_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let body = body_from_keys(
        &args,
        &["tmdb_id", "media_type", "season", "episode", "score"],
        &["label"],
    )?;
    plan("POST", build_url("/episode-ratings", &[]), Some(body))
}

pub(crate) fn publicmetadb_episode_ratings_delete_plan(id: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    plan(
        "DELETE",
        build_url(&format!("/episode-ratings/{id}"), &[]),
        None,
    )
}

pub(crate) fn publicmetadb_episode_ratings_batch_url(query_json: &str) -> Option<String> {
    let args = parse_args(query_json);
    let params = extract_query(&args, &["tmdb_id", "media_type", "season", "label"]);
    if !params.iter().any(|(key, _)| key == "tmdb_id")
        || !params.iter().any(|(key, _)| key == "media_type")
        || !params.iter().any(|(key, _)| key == "season")
    {
        return None;
    }
    Some(build_url("/episode-ratings/batch", &params))
}

pub(crate) fn publicmetadb_episode_ratings_batch_create_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let ratings = args.get("ratings")?.as_array()?;
    if ratings.is_empty() || ratings.len() > 50 {
        return None;
    }
    let body = body_from_keys(
        &args,
        &["tmdb_id", "media_type", "season", "ratings"],
        &["label"],
    )?;
    plan("POST", build_url("/episode-ratings/batch", &[]), Some(body))
}

pub(crate) fn publicmetadb_episode_ratings_batch_delete_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let ids = args.get("ids")?.as_array()?;
    if ids.is_empty() || ids.len() > 50 {
        return None;
    }
    let body = body_from_keys(&args, &["ids"], &[])?;
    plan(
        "DELETE",
        build_url("/episode-ratings/batch", &[]),
        Some(body),
    )
}
