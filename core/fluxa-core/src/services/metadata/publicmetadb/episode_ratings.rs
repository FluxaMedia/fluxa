use super::helpers::{body_from_keys, build_url, plan};
use serde_json::Value;

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
