use serde_json::{Value, json};

pub(crate) fn remote_collection_request_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let source = request.get("source")?;
    let provider = source.get("provider").and_then(Value::as_str)?;
    let page = request
        .get("page")
        .and_then(Value::as_i64)
        .unwrap_or(1)
        .max(1);
    if provider == "trakt" {
        let list_id = source.get("traktListId").and_then(Value::as_i64)?;
        let client_id = request
            .get("clientId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())?;
        let media_type = if source
            .get("mediaType")
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case("TV"))
        {
            "show"
        } else {
            "movie"
        };
        let mut params = serde_json::Map::from_iter([
            ("extended".into(), json!("full,images")),
            ("page".into(), json!(page)),
            ("limit".into(), json!(50)),
        ]);
        for (input, output) in [("sortBy", "sort_by"), ("sortHow", "sort_how")] {
            if let Some(value) = source
                .get(input)
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
            {
                params.insert(output.into(), json!(value));
            }
        }
        return serde_json::to_string(&json!({
            "url": format!("https://api.trakt.tv/lists/{list_id}/items/{media_type}"), "params": params,
            "headers": {"trakt-api-version": "2", "trakt-api-key": client_id}, "responseKind": "trakt", "requestedType": if media_type == "show" { "series" } else { "movie" }
        })).ok();
    }
    if provider != "tmdb" {
        return None;
    }
    let api_key = request
        .get("apiKey")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let language = request
        .get("language")
        .and_then(Value::as_str)
        .unwrap_or("en")
        .replace('_', "-");
    let source_type = source
        .get("tmdbSourceType")
        .and_then(Value::as_str)
        .unwrap_or("DISCOVER");
    let source_id = source.get("tmdbId").and_then(Value::as_i64);
    let media_type = if source
        .get("mediaType")
        .and_then(Value::as_str)
        .is_some_and(|value| value.eq_ignore_ascii_case("TV"))
    {
        "tv"
    } else {
        "movie"
    };
    let requested_type = if media_type == "tv" {
        "series"
    } else {
        "movie"
    };
    let actual_type = if source_type == "NETWORK" {
        "tv"
    } else {
        media_type
    };
    let mut params = serde_json::Map::from_iter([
        ("api_key".into(), json!(api_key)),
        ("language".into(), json!(language)),
        ("page".into(), json!(page)),
    ]);
    let path = match (source_type, source_id) {
        ("LIST", Some(id)) => format!("3/list/{id}"),
        ("COLLECTION", Some(id)) => {
            params.remove("page");
            format!("3/collection/{id}")
        }
        ("PERSON" | "DIRECTOR", Some(id)) => {
            params.remove("page");
            format!("3/person/{id}/combined_credits")
        }
        _ => {
            params.insert(
                "sort_by".into(),
                source
                    .get("sortBy")
                    .cloned()
                    .unwrap_or_else(|| json!("popularity.desc")),
            );
            if source_type == "COMPANY"
                && let Some(id) = source_id
            {
                params.insert("with_companies".into(), json!(id));
            }
            if source_type == "NETWORK"
                && let Some(id) = source_id
            {
                params.insert("with_networks".into(), json!(id));
            }
            let filters = source.get("filters").and_then(Value::as_object);
            for (input, output) in [
                (
                    "year",
                    if actual_type == "tv" {
                        "first_air_date_year"
                    } else {
                        "year"
                    },
                ),
                ("withGenres", "with_genres"),
                ("watchRegion", "watch_region"),
                ("voteCountGte", "vote_count.gte"),
                ("withKeywords", "with_keywords"),
                ("withNetworks", "with_networks"),
                ("withCompanies", "with_companies"),
                (
                    "releaseDateGte",
                    if actual_type == "tv" {
                        "first_air_date.gte"
                    } else {
                        "primary_release_date.gte"
                    },
                ),
                (
                    "releaseDateLte",
                    if actual_type == "tv" {
                        "first_air_date.lte"
                    } else {
                        "primary_release_date.lte"
                    },
                ),
                ("voteAverageGte", "vote_average.gte"),
                ("voteAverageLte", "vote_average.lte"),
                ("withOriginCountry", "with_origin_country"),
                ("withWatchProviders", "with_watch_providers"),
                ("withOriginalLanguage", "with_original_language"),
            ] {
                if let Some(value) = filters
                    .and_then(|values| values.get(input))
                    .filter(|value| value.is_string() || value.is_number())
                {
                    params.insert(output.into(), value.clone());
                }
            }
            format!("3/discover/{actual_type}")
        }
    };
    serde_json::to_string(&json!({"url": format!("https://api.themoviedb.org/{path}"), "params": params, "headers": {}, "responseKind": "tmdb", "sourceType": source_type, "mediaType": media_type, "requestedType": requested_type, "language": language})).ok()
}

fn first_image_url(images: Option<&Value>, field: &str) -> Option<String> {
    images?
        .get(field)?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .find(|url| !url.is_empty())
        .map(|url| {
            if url.starts_with("//") {
                format!("https:{url}")
            } else if url.starts_with("http://") {
                format!("https://{}", &url[7..])
            } else if url.starts_with("trakt.tv/") || url.starts_with("images.trakt.tv/") {
                format!("https://{url}")
            } else {
                url.to_string()
            }
        })
}

pub(crate) fn remote_collection_response_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let plan = request.get("plan")?;
    let data = request.get("data")?;
    if plan.get("responseKind").and_then(Value::as_str) == Some("trakt") {
        let requested_type = plan
            .get("requestedType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let metas = data.as_array()?.iter().filter_map(|item| {
            let value = item.get(if requested_type == "series" { "show" } else { "movie" })?;
            let title = value.get("title").and_then(Value::as_str)?;
            let ids = value.get("ids")?;
            let id = ids
                .get("imdb")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| ids.get("tmdb").and_then(Value::as_i64).map(|id| format!("tmdb:{id}")))
                .or_else(|| ids.get("trakt").and_then(Value::as_i64).map(|id| format!("trakt:{id}")))?;
            let images = value.get("images");
            let poster = first_image_url(images, "poster").or_else(|| first_image_url(images, "fanart"));
            let background = first_image_url(images, "fanart")
                .or_else(|| first_image_url(images, "banner"))
                .or_else(|| first_image_url(images, "thumb"))
                .or_else(|| poster.clone());
            let logo = first_image_url(images, "logo").or_else(|| first_image_url(images, "clearart"));
            Some(json!({
                "id": id,
                "type": requested_type,
                "name": title,
                "poster": poster,
                "background": background,
                "logo": logo,
                "description": value.get("overview"),
                "releaseInfo": value.get("year").and_then(Value::as_i64).map(|year| year.to_string())
            }))
        }).collect::<Vec<_>>();
        return serde_json::to_string(&metas).ok();
    }
    let source_type = plan
        .get("sourceType")
        .and_then(Value::as_str)
        .unwrap_or("DISCOVER");
    let media_type = plan
        .get("mediaType")
        .and_then(Value::as_str)
        .unwrap_or("movie");
    let language = plan.get("language").and_then(Value::as_str).unwrap_or("en");
    let items = match source_type {
        "COLLECTION" => data
            .get("parts")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        "LIST" => data
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        "PERSON" => data
            .get("cast")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter(|item| {
                        item.get("media_type").and_then(Value::as_str) == Some(media_type)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default(),
        "DIRECTOR" => data
            .get("crew")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter(|item| {
                        item.get("job").and_then(Value::as_str) == Some("Director")
                            && item.get("media_type").and_then(Value::as_str) == Some(media_type)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default(),
        _ => data
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    };
    if source_type == "LIST" {
        let movies = items
            .iter()
            .filter(|item| item.get("media_type").and_then(Value::as_str) != Some("tv"))
            .cloned()
            .collect::<Vec<_>>();
        let series = items
            .iter()
            .filter(|item| item.get("media_type").and_then(Value::as_str) == Some("tv"))
            .cloned()
            .collect::<Vec<_>>();
        let mut metas: Vec<Value> =
            serde_json::from_str(&crate::tmdb_plan::tmdb_bulk_metas_to_metas_json(
                &Value::Array(movies).to_string(),
                "movie",
                language,
            )?)
            .ok()?;
        let mut series_metas: Vec<Value> =
            serde_json::from_str(&crate::tmdb_plan::tmdb_bulk_metas_to_metas_json(
                &Value::Array(series).to_string(),
                "series",
                language,
            )?)
            .ok()?;
        metas.append(&mut series_metas);
        return serde_json::to_string(&metas).ok();
    }
    crate::tmdb_plan::tmdb_bulk_metas_to_metas_json(
        &Value::Array(items).to_string(),
        plan.get("requestedType")
            .and_then(Value::as_str)
            .unwrap_or("movie"),
        language,
    )
}
