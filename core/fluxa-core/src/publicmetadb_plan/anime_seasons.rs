use super::helpers::{body_from_keys, build_url, extract_query, parse_args, plan};
use serde_json::Value;

pub(crate) fn publicmetadb_anime_seasons_url(query_json: &str) -> Option<String> {
    let args = parse_args(query_json);
    let params = extract_query(&args, &["tmdb_id"]);
    if params.is_empty() {
        return None;
    }
    Some(build_url("/anime-seasons", &params))
}

fn chunks_are_valid(chunks: &[Value]) -> bool {
    !chunks.is_empty()
        && chunks
            .iter()
            .all(|chunk| chunk.get("tmdb_season").is_some())
}

pub(crate) fn publicmetadb_anime_seasons_submit_plan(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let chunks = args.get("chunks")?.as_array()?;
    if !chunks_are_valid(chunks) {
        return None;
    }
    let body = body_from_keys(
        &args,
        &["tmdb_id", "season_number", "chunks"],
        &["season_name"],
    )?;
    plan("POST", build_url("/anime-seasons", &[]), Some(body))
}

pub(crate) fn publicmetadb_anime_seasons_delete_mapping_plan(query_json: &str) -> Option<String> {
    let args = parse_args(query_json);
    let params = extract_query(&args, &["tmdb_id", "season_number"]);
    if params.len() != 2 {
        return None;
    }
    plan("DELETE", build_url("/anime-seasons", &params), None)
}

pub(crate) fn publicmetadb_anime_seasons_delete_chunk_plan(id: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    plan(
        "DELETE",
        build_url(&format!("/anime-seasons/{id}"), &[]),
        None,
    )
}
