use serde_json::{Value, json};

pub(crate) fn provider_calendar_items_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = args.get("provider")?.as_str()?;
    let shows = args.get("shows").and_then(Value::as_array);
    let movies = args.get("movies").and_then(Value::as_array);
    let entries = args.get("entries").and_then(Value::as_array);
    let mut items = Vec::new();
    if provider == "anilist" {
        for entry in entries.into_iter().flatten() {
            let Some(media) = entry.get("media") else {
                continue;
            };
            let Some(next) = media.get("nextAiringEpisode") else {
                continue;
            };
            let Some(media_id) = media.get("id").and_then(Value::as_i64) else {
                continue;
            };
            let Some(episode) = next.get("episode").and_then(Value::as_i64) else {
                continue;
            };
            let Some(airing_at) = next.get("airingAt").and_then(Value::as_i64) else {
                continue;
            };
            let content_id = format!("anilist:{media_id}");
            let title = media
                .pointer("/title/english")
                .or_else(|| media.pointer("/title/romaji"));
            let Some(date_iso) =
                chrono::DateTime::from_timestamp(airing_at, 0).map(|value| value.to_rfc3339())
            else {
                continue;
            };
            items.push(json!({
                "id": format!("{content_id}:{episode}"),
                "title": title,
                "dateIso": date_iso,
                "contentId": content_id,
                "seriesId": content_id,
            }));
        }
        return serde_json::to_string(&items).ok();
    }
    if provider == "mdblist" {
        for event in args
            .get("events")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let is_movie = event.get("type").and_then(Value::as_str) == Some("movie");
            let Some(tmdb) = event
                .get(if is_movie { "tmdb" } else { "show_tmdb" })
                .and_then(Value::as_i64)
            else {
                continue;
            };
            let Some(date) = event.get("start").and_then(Value::as_str) else {
                continue;
            };
            let content_id = format!("tmdb:{tmdb}");
            let poster = event
                .get("poster")
                .and_then(Value::as_str)
                .filter(|url| !url.is_empty());
            if is_movie {
                items.push(json!({
                    "id": event.get("id").and_then(Value::as_str).unwrap_or(&content_id),
                    "title": event.get("title"),
                    "dateIso": date,
                    "contentId": content_id,
                    "metaType": "movie",
                    "poster": poster,
                }));
                continue;
            }
            let season = event.get("season_number").and_then(Value::as_i64);
            let number = event.get("episode_number").and_then(Value::as_i64);
            let still = event
                .get("image")
                .and_then(Value::as_str)
                .filter(|path| path.starts_with('/'))
                .map(|path| format!("https://image.tmdb.org/t/p/w500{path}"));
            let finale = if event["is_season_finale"] == true {
                Some("season")
            } else if event["is_mid_season_finale"] == true {
                Some("mid_season")
            } else {
                None
            };
            items.push(json!({
                "id": format!("{content_id}:{}:{}", season.unwrap_or_default(), number.unwrap_or_default()),
                "title": event.get("title"),
                "episodeTitle": event.get("episode_title"),
                "seasonNumber": season,
                "episodeNumber": number,
                "dateIso": date,
                "contentId": content_id,
                "seriesId": content_id,
                "metaType": "series",
                "poster": still.as_deref().or(poster),
                "episodePoster": still,
                "seriesPoster": poster,
                "finaleType": finale,
            }));
        }
        return serde_json::to_string(&items).ok();
    }
    if provider == "simkl" && args.get("shows").is_some_and(Value::is_object) {
        let allowed: std::collections::HashSet<&str> = args
            .get("allowedContentIds")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        for (catalog, is_movie) in [("shows", false), ("anime", false), ("movies", true)] {
            let block = args.get(catalog);
            let metadata = block.and_then(|value| value.get("metadata"));
            let calendar = block
                .and_then(|value| value.get("calendar"))
                .and_then(Value::as_array);
            for entry in calendar.into_iter().flatten() {
                let Some(media) = entry
                    .get("simkl_id")
                    .and_then(|id| id.as_i64().map(|id| id.to_string()))
                    .and_then(|key| metadata?.get(&key))
                else {
                    continue;
                };
                let Some(content_id) =
                    crate::accounts::external_sync::provider_mappers::simkl_content_id(media)
                        .filter(|id| allowed.contains(id.as_str()))
                else {
                    continue;
                };
                let Some(date) = entry.get("date").and_then(Value::as_str) else {
                    continue;
                };
                let poster = media
                    .get("poster")
                    .and_then(Value::as_str)
                    .filter(|path| !path.is_empty())
                    .map(|path| {
                        if path.starts_with("http") {
                            path.to_owned()
                        } else {
                            format!("https://simkl.in/posters/{path}_m.jpg")
                        }
                    });
                if is_movie {
                    items.push(json!({
                        "id": content_id,
                        "title": media.get("title"),
                        "dateIso": date,
                        "contentId": content_id,
                        "metaType": "movie",
                        "poster": poster,
                    }));
                    continue;
                }
                let episode = entry.get("episode").unwrap_or(&Value::Null);
                let season = episode.get("season").and_then(Value::as_i64).unwrap_or(1);
                let number = episode.get("episode").and_then(Value::as_i64);
                items.push(json!({
                    "id": format!("{content_id}:{season}:{}", number.unwrap_or_default()),
                    "title": media.get("title"),
                    "episodeTitle": episode.get("title"),
                    "seasonNumber": season,
                    "episodeNumber": number,
                    "dateIso": date,
                    "contentId": content_id,
                    "seriesId": content_id,
                    "metaType": "series",
                    "poster": poster,
                    "seriesPoster": poster,
                    "isAnime": catalog == "anime",
                    "finaleType": entry.get("finale_type"),
                }));
            }
        }
        return serde_json::to_string(&items).ok();
    }
    for entry in shows.into_iter().flatten() {
        let Some(show) = entry.get("show") else {
            continue;
        };
        let Some(episode) = entry.get("episode") else {
            continue;
        };
        let Some(ids) = show.get("ids") else { continue };
        let Some(series_id) = ids
            .get("imdb")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                ids.get("tmdb")
                    .and_then(Value::as_i64)
                    .map(|id| format!("tmdb:{id}"))
            })
        else {
            continue;
        };
        let Some(date) = entry
            .get(if provider == "trakt" {
                "first_aired"
            } else {
                "date"
            })
            .and_then(Value::as_str)
        else {
            continue;
        };
        let season = episode
            .get("season")
            .or_else(|| episode.get("season_number"))
            .and_then(Value::as_i64);
        let number = episode
            .get("number")
            .or_else(|| episode.get("episode"))
            .or_else(|| episode.get("episode_number"))
            .and_then(Value::as_i64);
        let episode_poster = provider_image_url(episode, "screenshot");
        let series_poster = provider_image_url(show, "poster");
        items.push(json!({
            "id": format!("{series_id}:{}:{}", season.unwrap_or_default(), number.unwrap_or_default()),
            "title": show.get("title"),
            "episodeTitle": episode.get("title"),
            "seasonNumber": season,
            "episodeNumber": number,
            "dateIso": date,
            "contentId": series_id,
            "seriesId": series_id,
            "metaType": "series",
            "poster": episode_poster.as_ref().or(series_poster.as_ref()),
            "episodePoster": episode_poster,
            "seriesPoster": series_poster,
        }));
    }
    for entry in movies.into_iter().flatten() {
        let Some(movie) = entry.get("movie") else {
            continue;
        };
        let Some(ids) = movie.get("ids") else {
            continue;
        };
        let Some(content_id) = ids
            .get("imdb")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                ids.get("tmdb")
                    .and_then(Value::as_i64)
                    .map(|id| format!("tmdb:{id}"))
            })
        else {
            continue;
        };
        let Some(date) = entry
            .get(if provider == "trakt" {
                "released"
            } else {
                "date"
            })
            .and_then(Value::as_str)
        else {
            continue;
        };
        let poster = provider_image_url(movie, "poster");
        items.push(json!({
            "id": content_id,
            "title": movie.get("title"),
            "dateIso": date,
            "contentId": content_id,
            "metaType": "movie",
            "poster": poster,
        }));
    }
    serde_json::to_string(&items).ok()
}

fn provider_image_url(media: &Value, image_type: &str) -> Option<String> {
    let image = media
        .get("images")?
        .get(image_type)
        .and_then(|value| {
            value
                .as_array()
                .and_then(|images| images.first())
                .or(Some(value))
        })
        .and_then(Value::as_str)?;
    if image.starts_with("https://") || image.starts_with("http://") {
        Some(image.to_string())
    } else {
        Some(format!("https://{image}"))
    }
}
