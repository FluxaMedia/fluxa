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
    if provider == "simkl"
        && args
            .get("shows")
            .and_then(|value| value.get("calendar"))
            .is_some()
    {
        let allowed_content_ids: std::collections::HashSet<&str> = args
            .get("allowedContentIds")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        for (calendar, metadata, is_movie) in [
            (
                args.get("shows")
                    .and_then(|value| value.get("calendar"))
                    .and_then(Value::as_array),
                args.get("shows")
                    .and_then(|value| value.get("metadata"))
                    .and_then(Value::as_object),
                false,
            ),
            (
                args.get("movies")
                    .and_then(|value| value.get("calendar"))
                    .and_then(Value::as_array),
                args.get("movies")
                    .and_then(|value| value.get("metadata"))
                    .and_then(Value::as_object),
                true,
            ),
        ] {
            for entry in calendar.into_iter().flatten() {
                let Some(simkl_id) = entry.get("simkl_id") else {
                    continue;
                };
                let simkl_key = simkl_id
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| simkl_id.as_i64().map(|value| value.to_string()));
                let Some(media) = simkl_key
                    .as_deref()
                    .and_then(|key| metadata.and_then(|value| value.get(key)))
                else {
                    continue;
                };
                let ids = media.get("ids").unwrap_or(&Value::Null);
                let content_id = ids
                    .get("imdb")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| {
                        ids.get("tmdb")
                            .and_then(Value::as_i64)
                            .map(|id| format!("tmdb:{id}"))
                            .or_else(|| {
                                ids.get("tmdb")
                                    .and_then(Value::as_str)
                                    .filter(|id| !id.is_empty())
                                    .map(|id| format!("tmdb:{id}"))
                            })
                    });
                let Some(content_id) = content_id else {
                    continue;
                };
                if !allowed_content_ids.contains(content_id.as_str()) {
                    continue;
                }
                let Some(date) = entry.get("date").and_then(Value::as_str) else {
                    continue;
                };
                if is_movie {
                    items.push(json!({
                        "id": content_id,
                        "title": media.get("title"),
                        "dateIso": date,
                        "contentId": content_id,
                        "metaType": "movie",
                        "poster": media.get("poster"),
                    }));
                    continue;
                }
                let episode = entry.get("episode").unwrap_or(&Value::Null);
                let season = episode.get("season").and_then(Value::as_i64);
                let number = episode.get("episode").and_then(Value::as_i64);
                items.push(json!({
                    "id": format!("{content_id}:{}:{}", season.unwrap_or_default(), number.unwrap_or_default()),
                    "title": media.get("title"),
                    "episodeTitle": episode.get("title"),
                    "seasonNumber": season,
                    "episodeNumber": number,
                    "dateIso": date,
                    "contentId": content_id,
                    "seriesId": content_id,
                    "metaType": "series",
                    "poster": media.get("poster"),
                    "seriesPoster": media.get("poster"),
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
