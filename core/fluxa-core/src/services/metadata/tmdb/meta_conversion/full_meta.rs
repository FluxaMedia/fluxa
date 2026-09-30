use super::*;

pub(crate) fn tmdb_full_meta_to_meta_json(
    details_json: &str,
    credits_json: &str,
    images_json: &str,
    external_ids_json: &str,
    extras_json: &str,
    requested_type: &str,
    language: &str,
) -> Option<String> {
    let details: Value = serde_json::from_str(details_json).ok()?;
    let credits: Value = serde_json::from_str(credits_json).unwrap_or_else(|_| json!({}));
    let images: Value = serde_json::from_str(images_json).unwrap_or_else(|_| json!({}));
    let external_ids: Value = serde_json::from_str(external_ids_json).unwrap_or_else(|_| json!({}));
    let extras: Value = serde_json::from_str(extras_json).unwrap_or_else(|_| json!({}));

    let tmdb_id = details.get("id").and_then(Value::as_i64)?;
    let has_tv = details.get("first_air_date").is_some() || details.get("name").is_some();
    let content_type = if requested_type == "series" || has_tv {
        "series"
    } else {
        "movie"
    };

    let imdb_id = external_ids
        .get("imdb_id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    let id = imdb_id
        .map(str::to_string)
        .unwrap_or_else(|| format!("tmdb:{tmdb_id}"));

    let name = details
        .get("title")
        .or_else(|| details.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("Unknown");
    let description = details
        .get("overview")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    let released = details
        .get("release_date")
        .or_else(|| details.get("first_air_date"))
        .and_then(Value::as_str);
    let runtime_minutes = details
        .get("runtime")
        .and_then(Value::as_i64)
        .filter(|m| *m > 0)
        .or_else(|| {
            details
                .get("episode_run_time")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(Value::as_i64)
        });
    let genres: Vec<String> = details
        .get("genres")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|g| g.get("name").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let poster = pick_image(&images, "posters", language, "w500")
        .or_else(|| tmdb_image_url(details.get("poster_path").and_then(Value::as_str), "w500"));
    let background = pick_image(&images, "backdrops", language, "original").or_else(|| {
        tmdb_image_url(
            details.get("backdrop_path").and_then(Value::as_str),
            "original",
        )
    });
    let logo = pick_logo(&images, language);
    let network = details
        .get("networks")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .or_else(|| {
            details
                .get("production_companies")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
        })
        .and_then(|c| c.get("name"))
        .and_then(Value::as_str);

    let cast: Vec<Value> = credits
        .get("cast")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .take(10)
                .filter_map(|c| {
                    let name = c.get("name").and_then(Value::as_str)?;
                    Some(json!({
                        "name": name,
                        "character": c.get("character").and_then(Value::as_str),
                        "profilePath": tmdb_image_url(c.get("profile_path").and_then(Value::as_str), "w185"),
                    }))
                })
                .collect()
        })
        .unwrap_or_default();
    let director: Vec<String> = credits
        .get("crew")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter(|c| c.get("job").and_then(Value::as_str) == Some("Director"))
                .filter_map(|c| c.get("name").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let tagline = details
        .get("tagline")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    let status = details.get("status").and_then(Value::as_str);
    let collection = details.get("belongs_to_collection").and_then(|c| {
        let name = c.get("name").and_then(Value::as_str)?;
        Some(json!({
            "name": name,
            "poster": tmdb_image_url(c.get("poster_path").and_then(Value::as_str), "w500"),
            "background": tmdb_image_url(c.get("backdrop_path").and_then(Value::as_str), "original"),
        }))
    });
    let original_language = details.get("original_language").and_then(Value::as_str);
    let production_countries: Vec<String> = details
        .get("production_countries")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.get("name").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let created_by: Vec<String> = details
        .get("created_by")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.get("name").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let episode_to_air = |key: &str| {
        details.get(key).and_then(|ep| {
            let season = ep.get("season_number").and_then(Value::as_i64)?;
            let episode = ep.get("episode_number").and_then(Value::as_i64)?;
            Some(json!({
                "season": season,
                "episode": episode,
                "airDate": ep.get("air_date").and_then(Value::as_str),
                "name": ep.get("name").and_then(Value::as_str),
            }))
        })
    };
    let next_episode_to_air = episode_to_air("next_episode_to_air");
    let last_episode_to_air = episode_to_air("last_episode_to_air");

    let region = tmdb_region_from_language(language);

    let keywords: Vec<String> = extras
        .get("keywords")
        .and_then(|k| {
            k.get(if content_type == "movie" {
                "keywords"
            } else {
                "results"
            })
            .and_then(Value::as_array)
        })
        .map(|arr| {
            arr.iter()
                .filter_map(|k| k.get("name").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let alternative_titles: Vec<String> = extras
        .get("alternativeTitles")
        .and_then(|t| {
            t.get(if content_type == "movie" {
                "titles"
            } else {
                "results"
            })
            .and_then(Value::as_array)
        })
        .map(|arr| {
            let mut titles: Vec<String> = arr
                .iter()
                .filter_map(|t| t.get("title").and_then(Value::as_str).map(str::to_string))
                .collect();
            titles.dedup();
            titles.truncate(10);
            titles
        })
        .unwrap_or_default();

    let certification = extras.get("contentRatings").and_then(|r| {
        if content_type == "movie" {
            r.get("results")
                .and_then(Value::as_array)?
                .iter()
                .find(|entry| {
                    entry.get("iso_3166_1").and_then(Value::as_str) == Some(region.as_str())
                })
                .and_then(|entry| entry.get("release_dates").and_then(Value::as_array))
                .and_then(|dates| {
                    dates
                        .iter()
                        .filter_map(|d| d.get("certification").and_then(Value::as_str))
                        .find(|c| !c.is_empty())
                })
        } else {
            r.get("results")
                .and_then(Value::as_array)?
                .iter()
                .find(|entry| {
                    entry.get("iso_3166_1").and_then(Value::as_str) == Some(region.as_str())
                })
                .and_then(|entry| entry.get("rating").and_then(Value::as_str))
                .filter(|c| !c.is_empty())
        }
    });

    let watch_providers = extras.get("watchProviders").and_then(|w| {
        let results = w.get("results")?;
        let (used_region, regional) = results
            .get(region.as_str())
            .map(|v| (region.as_str(), v))
            .or_else(|| results.get("US").map(|v| ("US", v)))?;
        let providers = |key: &str| -> Vec<Value> {
            regional
                .get(key)
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| {
                            let name = p.get("provider_name").and_then(Value::as_str)?;
                            Some(json!({
                                "name": name,
                                "logo": tmdb_image_url(p.get("logo_path").and_then(Value::as_str), "w92"),
                            }))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        Some(json!({
            "region": used_region,
            "link": regional.get("link").and_then(Value::as_str),
            "flatrate": providers("flatrate"),
            "rent": providers("rent"),
            "buy": providers("buy"),
        }))
    });

    serde_json::to_string(&json!({
        "id": id,
        "type": content_type,
        "name": name,
        "description": description,
        "tagline": tagline,
        "status": status,
        "poster": poster,
        "background": background,
        "logo": logo,
        "network": network,
        "collection": collection,
        "originalLanguage": original_language,
        "productionCountries": production_countries,
        "createdBy": created_by,
        "nextEpisodeToAir": next_episode_to_air,
        "lastEpisodeToAir": last_episode_to_air,
        "keywords": keywords,
        "alternativeTitles": alternative_titles,
        "certification": certification,
        "watchProviders": watch_providers,
        "releaseInfo": released.map(|r| r.get(..4).unwrap_or(r)),
        "runtime": runtime_minutes.map(|m| format!("{m} min")),
        "genres": genres,
        "imdbRating": details.get("vote_average").and_then(Value::as_f64),
        "cast": cast,
        "director": director,
    }))
    .ok()
}
