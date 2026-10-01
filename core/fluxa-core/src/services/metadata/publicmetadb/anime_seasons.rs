use super::helpers::{build_url, extract_query, parse_args, plan};

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
