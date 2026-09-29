use super::helpers::{build_url, plan};
use serde_json::json;

const VOTE_RESOURCES: &[&str] = &[
    "skips",
    "ratings",
    "highlights",
    "mappings",
    "anime-seasons",
    "episode-ratings",
];

fn votes_path(resource: &str, item_id: &str) -> Option<String> {
    if item_id.is_empty() || !VOTE_RESOURCES.contains(&resource) {
        return None;
    }
    Some(format!("/{resource}/{item_id}/votes"))
}

pub(crate) fn publicmetadb_votes_url(resource: &str, item_id: &str, all: bool) -> Option<String> {
    let path = votes_path(resource, item_id)?;
    let params = if all {
        vec![("all".to_string(), "true".to_string())]
    } else {
        vec![]
    };
    Some(build_url(&path, &params))
}

pub(crate) fn publicmetadb_votes_create_plan(
    resource: &str,
    item_id: &str,
    vote: i64,
) -> Option<String> {
    let path = votes_path(resource, item_id)?;
    if !(-1..=1).contains(&vote) {
        return None;
    }
    plan("POST", build_url(&path, &[]), Some(json!({ "vote": vote })))
}

pub(crate) fn publicmetadb_votes_delete_plan(resource: &str, item_id: &str) -> Option<String> {
    let path = votes_path(resource, item_id)?;
    plan("DELETE", build_url(&path, &[]), None)
}
