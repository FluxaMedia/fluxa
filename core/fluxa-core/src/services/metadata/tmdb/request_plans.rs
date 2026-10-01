use super::helpers::{
    is_imdb_id, tmdb_api_url, tmdb_content_type,
    tmdb_resolve_id_hint
};
use serde_json::{Value, json};

fn tmdb_detail_request_urls(
    content_type: &str,
    tmdb_id: &str,
    api_key: &str,
    language: &str,
    endpoints: &[Value],
) -> Option<Value> {
    let tmdb_type = tmdb_content_type(content_type);
    let mut urls = serde_json::Map::new();
    for endpoint in endpoints {
        let endpoint = endpoint.as_str()?;
        if !matches!(
            endpoint,
            "details" | "videos" | "external_ids" | "recommendations" | "similar"
        ) {
            return None;
        }
        let path = if endpoint == "details" {
            format!("3/{tmdb_type}/{tmdb_id}")
        } else {
            format!("3/{tmdb_type}/{tmdb_id}/{endpoint}")
        };
        urls.insert(
            endpoint.to_string(),
            Value::String(tmdb_api_url(&path, api_key, language, &[])),
        );
    }
    Some(json!({ "tmdbId": tmdb_id, "urls": urls }))
}
pub(crate) fn tmdb_detail_request_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let content_type = args.get("contentType")?.as_str()?;
    let content_id = args.get("contentId")?.as_str()?;
    let api_key = args.get("apiKey")?.as_str()?;
    let language = args.get("language")?.as_str()?;
    let endpoints = args.get("endpoints")?.as_array()?;
    let (tmdb_id, resolved) = tmdb_resolve_id_hint(content_id);
    let value = if resolved {
        tmdb_detail_request_urls(content_type, &tmdb_id, api_key, language, endpoints)?
    } else if is_imdb_id(&tmdb_id) {
        json!({
            "findUrl": tmdb_api_url(&format!("3/find/{tmdb_id}"), api_key, language, &[("external_source", "imdb_id")])
        })
    } else {
        return None;
    };
    serde_json::to_string(&value).ok()
}
pub(crate) fn tmdb_detail_request_urls_from_find_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let find = args.get("find")?;
    let content_type = args.get("contentType")?.as_str()?;
    let api_key = args.get("apiKey")?.as_str()?;
    let language = args.get("language")?.as_str()?;
    let endpoints = args.get("endpoints")?.as_array()?;
    let result_key = if content_type == "series" {
        "tv_results"
    } else {
        "movie_results"
    };
    let tmdb_id = find.get(result_key)?.as_array()?.first()?.get("id")?;
    let tmdb_id = match tmdb_id {
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        _ => return None
    };
    serde_json::to_string(&tmdb_detail_request_urls(
        content_type,
        &tmdb_id,
        api_key,
        language,
        endpoints,
    )?)
    .ok()
}
