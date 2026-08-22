use super::helpers::{
    is_imdb_id, normalize_person_name, tmdb_api_url, tmdb_content_type, tmdb_image_url,
    tmdb_resolve_id_hint,
};
use serde_json::{Value, json};

fn tmdb_content_ratings_path(tmdb_type: &str, tmdb_id: &str) -> String {
    if tmdb_type == "tv" {
        format!("3/tv/{tmdb_id}/content_ratings")
    } else {
        format!("3/movie/{tmdb_id}/release_dates")
    }
}
pub(crate) fn tmdb_builtin_meta_request_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let content_type = args.get("contentType")?.as_str()?;
    let content_id = args.get("contentId")?.as_str()?;
    let api_key = args.get("apiKey")?.as_str()?;
    let language = args.get("language")?.as_str()?;
    let (tmdb_id, already_resolved) = tmdb_resolve_id_hint(content_id);
    let tmdb_type = tmdb_content_type(content_type);
    let mut plan = json!({
        "tmdbId": tmdb_id,
        "contentType": tmdb_type,
        "alreadyResolved": already_resolved,
    });
    let plan_object = plan.as_object_mut()?;
    if already_resolved {
        plan_object.insert(
            "detailsUrl".to_string(),
            json!(tmdb_api_url(
                &format!("3/{tmdb_type}/{tmdb_id}"),
                api_key,
                language,
                &[]
            )),
        );
        plan_object.insert(
            "creditsUrl".to_string(),
            json!(tmdb_api_url(
                &format!("3/{tmdb_type}/{tmdb_id}/credits"),
                api_key,
                language,
                &[]
            )),
        );
        plan_object.insert(
            "imagesUrl".to_string(),
            json!(tmdb_api_url(
                &format!("3/{tmdb_type}/{tmdb_id}/images"),
                api_key,
                language,
                &[("include_image_language", "en,null")]
            )),
        );
        plan_object.insert(
            "externalIdsUrl".to_string(),
            json!(tmdb_api_url(
                &format!("3/{tmdb_type}/{tmdb_id}/external_ids"),
                api_key,
                language,
                &[]
            )),
        );
        plan_object.insert(
            "keywordsUrl".to_string(),
            json!(tmdb_api_url(
                &format!("3/{tmdb_type}/{tmdb_id}/keywords"),
                api_key,
                language,
                &[]
            )),
        );
        plan_object.insert(
            "alternativeTitlesUrl".to_string(),
            json!(tmdb_api_url(
                &format!("3/{tmdb_type}/{tmdb_id}/alternative_titles"),
                api_key,
                language,
                &[]
            )),
        );
        plan_object.insert(
            "contentRatingsUrl".to_string(),
            json!(tmdb_api_url(
                &tmdb_content_ratings_path(tmdb_type, &tmdb_id),
                api_key,
                language,
                &[]
            )),
        );
        plan_object.insert(
            "watchProvidersUrl".to_string(),
            json!(tmdb_api_url(
                &format!("3/{tmdb_type}/{tmdb_id}/watch/providers"),
                api_key,
                language,
                &[]
            )),
        );
    } else if is_imdb_id(&tmdb_id) {
        plan_object.insert(
            "findUrl".to_string(),
            json!(tmdb_api_url(
                &format!("3/find/{tmdb_id}"),
                api_key,
                language,
                &[("external_source", "imdb_id")]
            )),
        );
    } else {
        return None;
    }
    serde_json::to_string(&plan).ok()
}
pub(crate) fn tmdb_builtin_meta_urls_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let content_type = args.get("contentType")?.as_str()?;
    let tmdb_id = args.get("tmdbId")?.as_str()?;
    let api_key = args.get("apiKey")?.as_str()?;
    let language = args.get("language")?.as_str()?;
    let tmdb_type = tmdb_content_type(content_type);
    serde_json::to_string(&json!({
        "tmdbId": tmdb_id,
        "detailsUrl": tmdb_api_url(&format!("3/{tmdb_type}/{tmdb_id}"), api_key, language, &[]),
        "creditsUrl": tmdb_api_url(&format!("3/{tmdb_type}/{tmdb_id}/credits"), api_key, language, &[]),
        "imagesUrl": tmdb_api_url(&format!("3/{tmdb_type}/{tmdb_id}/images"), api_key, language, &[("include_image_language", "en,null")]),
        "externalIdsUrl": tmdb_api_url(&format!("3/{tmdb_type}/{tmdb_id}/external_ids"), api_key, language, &[]),
        "keywordsUrl": tmdb_api_url(&format!("3/{tmdb_type}/{tmdb_id}/keywords"), api_key, language, &[]),
        "alternativeTitlesUrl": tmdb_api_url(&format!("3/{tmdb_type}/{tmdb_id}/alternative_titles"), api_key, language, &[]),
        "contentRatingsUrl": tmdb_api_url(&tmdb_content_ratings_path(tmdb_type, tmdb_id), api_key, language, &[]),
        "watchProvidersUrl": tmdb_api_url(&format!("3/{tmdb_type}/{tmdb_id}/watch/providers"), api_key, language, &[]),
    })).ok()
}
pub(crate) fn tmdb_builtin_meta_urls_from_find_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let find = args.get("find")?;
    let content_type = args.get("contentType")?.as_str()?;
    let api_key = args.get("apiKey")?.as_str()?;
    let language = args.get("language")?.as_str()?;
    let result_key = if content_type == "series" {
        "tv_results"
    } else {
        "movie_results"
    };
    let tmdb_id = find.get(result_key)?.as_array()?.first()?.get("id")?;
    let tmdb_id = match tmdb_id {
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        _ => return None,
    };
    tmdb_builtin_meta_urls_json(
        &json!({
            "contentType": content_type,
            "tmdbId": tmdb_id,
            "apiKey": api_key,
            "language": language,
        })
        .to_string(),
    )
}
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
            "findUrl": tmdb_api_url(&format!("3/find/{tmdb_id}"), api_key, language, &[("external_source", "imdb_id")]),
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
        _ => return None,
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
pub(crate) fn tmdb_season_request_url(
    content_id: &str,
    season: i64,
    api_key: &str,
    language: &str,
) -> Option<String> {
    let (tmdb_id, already_resolved) = tmdb_resolve_id_hint(content_id);
    already_resolved.then(|| {
        tmdb_api_url(
            &format!("3/tv/{tmdb_id}/season/{season}"),
            api_key,
            language,
            &[],
        )
    })
}
fn tmdb_credits_url(content_type: &str, tmdb_id: &str, api_key: &str, language: &str) -> String {
    tmdb_api_url(
        &format!("3/{}/{tmdb_id}/credits", tmdb_content_type(content_type)),
        api_key,
        language,
        &[],
    )
}
pub(crate) fn tmdb_people_request_plan(meta: &Value, api_key: &str, language: &str) -> Value {
    let id = meta.get("id").and_then(Value::as_str).unwrap_or("");
    let content_type = meta.get("type").and_then(Value::as_str).unwrap_or("");

    let (base_id, resolved) = tmdb_resolve_id_hint(id);
    if resolved {
        return json!({ "creditsUrl": tmdb_credits_url(content_type, &base_id, api_key, language) });
    }

    let imdb_id = id.split(':').next().unwrap_or("");
    if !is_imdb_id(imdb_id) {
        return json!({});
    }
    json!({
        "findUrl": tmdb_api_url(
            &format!("3/find/{imdb_id}"),
            api_key,
            language,
            &[("external_source", "imdb_id")],
        ),
    })
}
pub(crate) fn tmdb_credits_url_from_find(
    find: &Value,
    meta: &Value,
    api_key: &str,
    language: &str,
) -> Option<String> {
    let content_type = meta.get("type").and_then(Value::as_str).unwrap_or("");
    let key = if content_type == "series" {
        "tv_results"
    } else {
        "movie_results"
    };
    let result_id = find.get(key)?.as_array()?.first()?.get("id")?;
    let tmdb_id = match result_id {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => return None,
    };
    Some(tmdb_credits_url(content_type, &tmdb_id, api_key, language))
}
pub(crate) fn tmdb_people_images_from_credits(credits: &Value, links: &[Value]) -> Value {
    let mut wanted: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for link in links {
        if let Some(name) = link.get("name").and_then(Value::as_str) {
            wanted.insert(normalize_person_name(name), name.to_string());
        }
    }

    let empty: Vec<Value> = Vec::new();
    let cast = credits
        .get("cast")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    let crew = credits
        .get("crew")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    let mut images = serde_json::Map::new();
    for person in cast.iter().chain(crew.iter()) {
        let name = person.get("name").and_then(Value::as_str).unwrap_or("");
        let Some(canonical) = wanted.get(&normalize_person_name(name)) else {
            continue;
        };
        if images.contains_key(canonical) {
            continue;
        }
        if let Some(image) =
            tmdb_image_url(person.get("profile_path").and_then(Value::as_str), "w185")
        {
            images.insert(canonical.clone(), Value::String(image));
        }
    }

    Value::Object(images)
}
