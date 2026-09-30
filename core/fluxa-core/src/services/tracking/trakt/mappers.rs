use crate::catalog::identity::parse_video_id_json;
use crate::services::trakt::*;
use serde_json::{Value, json};

pub(crate) fn trakt_watched_shows_to_items_json(shows_json: &str) -> Option<String> {
    let shows: Vec<Value> = serde_json::from_str(shows_json).unwrap_or_default();
    let mut items = Vec::new();

    for entry in shows {
        let Some(show) = entry.get("show") else {
            continue;
        };
        let Some(id) = trakt_id_from_source(show) else {
            continue;
        };
        let aired_episodes = show.get("aired_episodes").and_then(Value::as_i64);
        let completed = entry.get("completed").and_then(Value::as_i64);
        if aired_episodes.is_some_and(|aired| completed.unwrap_or(0) >= aired) {
            continue;
        }

        let mut last_episode: Option<(i64, i64, Option<String>)> = None;
        for season in entry
            .get("seasons")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let season_number = season.get("number").and_then(Value::as_i64).unwrap_or(0);
            if season_number <= 0 {
                continue;
            }
            for episode in season
                .get("episodes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let number = episode.get("number").and_then(Value::as_i64).unwrap_or(0);
                if number <= 0 {
                    continue;
                }
                let watched_at = episode
                    .get("last_watched_at")
                    .or_else(|| episode.get("watched_at"))
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string);
                let rank = (season_number, number);
                let replace =
                    last_episode
                        .as_ref()
                        .is_none_or(|(previous_season, previous_number, _)| {
                            rank > (*previous_season, *previous_number)
                        });
                if replace {
                    last_episode = Some((season_number, number, watched_at));
                }
            }
        }
        let Some((season, number, watched_at)) = last_episode else {
            continue;
        };
        let saved_at = entry
            .get("last_watched_at")
            .and_then(Value::as_str)
            .or(watched_at.as_deref())
            .unwrap_or("");
        let title = show.get("title").and_then(Value::as_str).unwrap_or("");
        items.push(json!({
            "id": id,
            "type": "series",
            "name": title,
            "continueWatchingBadge": "upNext",
            "lastVideoId": format!("{id}:{season}:{number}"),
            "lastEpisodeSeason": season,
            "lastEpisodeNumber": number,
            "timeOffset": 1,
            "duration": 1,
            "savedAt": saved_at,
            "reason": "trakt"
        }));
    }

    serde_json::to_string(&items).ok()
}

pub(crate) fn trakt_up_next_to_items_json(items_json: &str) -> Option<String> {
    let entries: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let mut items = Vec::new();
    for entry in entries {
        let Some(show) = entry.get("show") else {
            continue;
        };
        let Some(id) = trakt_id_from_source(show) else {
            continue;
        };
        let Some(next_episode) = entry
            .get("progress")
            .and_then(|progress| progress.get("next_episode"))
        else {
            continue;
        };
        let Some(season) = next_episode
            .get("season")
            .and_then(Value::as_i64)
            .filter(|value| *value > 0)
        else {
            continue;
        };
        let Some(number) = next_episode
            .get("number")
            .and_then(Value::as_i64)
            .filter(|value| *value > 0)
        else {
            continue;
        };
        let saved_at = entry
            .get("progress")
            .and_then(|progress| progress.get("last_watched_at"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let artwork = trakt_artwork(show);
        let episode_thumbnail = next_episode
            .get("images")
            .and_then(|images| trakt_image_url(images, "screenshot"));
        items.push(json!({
            "id": id,
            "type": "series",
            "name": show.get("title").and_then(Value::as_str).unwrap_or(""),
            "lastVideoId": format!("{id}:{season}:{number}"),
            "lastEpisodeSeason": season,
            "lastEpisodeNumber": number,
            "lastEpisodeName": next_episode.get("title").cloned().unwrap_or(Value::Null),
            "lastEpisodeThumbnail": episode_thumbnail,
            "continueWatchingBadge": "upNext",
            "savedAt": saved_at,
            "reason": "trakt",
            "poster": artwork.poster,
            "background": artwork.background,
            "logo": artwork.logo
        }));
    }
    serde_json::to_string(&items).ok()
}

pub(crate) fn trakt_mark_watched_body_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let video_ids: Vec<String> = request
        .as_array()
        .cloned()
        .and_then(|value| serde_json::from_value(Value::Array(value)).ok())
        .or_else(|| {
            request
                .get("videoIds")
                .cloned()
                .and_then(|value| serde_json::from_value(value).ok())
        })?;
    let watched_at = request
        .get("watchedAtMs")
        .and_then(Value::as_i64)
        .and_then(chrono::DateTime::from_timestamp_millis)
        .map(|value| value.to_rfc3339());
    let mut movie_ids: Vec<Value> = Vec::new();
    let mut shows: std::collections::HashMap<
        String,
        (Value, std::collections::BTreeMap<i64, Vec<i64>>),
    > = std::collections::HashMap::new();

    for vid in &video_ids {
        let parsed_json = parse_video_id_json(vid);
        let parsed: Value = match serde_json::from_str(&parsed_json) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let ids_json = match trakt_ids_from_content_id_json(vid) {
            Some(j) => j,
            None => continue,
        };
        let ids: Value = match serde_json::from_str(&ids_json) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if parsed
            .get("isEpisode")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            let season = parsed.get("season").and_then(Value::as_i64).unwrap_or(1);
            let episode = parsed.get("episode").and_then(Value::as_i64).unwrap_or(1);
            let show_id = parsed
                .get("imdb")
                .or_else(|| parsed.get("tmdb"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if show_id.is_empty() {
                continue;
            }
            let entry = shows
                .entry(show_id)
                .or_insert_with(|| (ids, std::collections::BTreeMap::new()));
            entry.1.entry(season).or_default().push(episode);
        } else {
            movie_ids.push(json!({ "ids": ids, "watched_at": watched_at }));
        }
    }

    let show_entries: Vec<Value> = shows
        .into_values()
        .map(|(ids, seasons)| {
            let seasons_arr: Vec<Value> = seasons
                .into_iter()
                .map(|(season, mut episodes)| {
                    episodes.sort_unstable();
                    episodes.dedup();
                    json!({
                        "number": season,
                        "episodes": episodes.into_iter().map(|n| json!({ "number": n, "watched_at": watched_at })).collect::<Vec<_>>()
                    })
                })
                .collect();
            json!({ "ids": ids, "seasons": seasons_arr })
        })
        .collect();

    let mut body = serde_json::Map::new();
    if !movie_ids.is_empty() {
        body.insert("movies".into(), movie_ids.into());
    }
    if !show_entries.is_empty() {
        body.insert("shows".into(), show_entries.into());
    }
    if body.is_empty() {
        return None;
    }
    serde_json::to_string(&Value::Object(body)).ok()
}

pub(crate) fn trakt_related_lookup_slug(lookup_json: &str, want_type: &str) -> Option<String> {
    let lookup: Vec<Value> = serde_json::from_str(lookup_json).ok()?;
    lookup
        .first()?
        .get(want_type)?
        .get("ids")?
        .get("slug")?
        .as_str()
        .map(|s| s.to_string())
}

pub(crate) fn trakt_related_items_to_metas_json(
    related_json: &str,
    content_type: &str,
) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(related_json).ok()?;
    let metas: Vec<Value> = items
        .iter()
        .filter_map(|item| {
            let ids = item.get("ids")?;
            let id = ids
                .get("imdb")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .or_else(|| {
                    ids.get("tmdb")
                        .and_then(Value::as_i64)
                        .map(|t| format!("tmdb:{t}"))
                })?;
            let name = item.get("title").and_then(Value::as_str)?;
            let mut meta = json!({ "id": id, "type": content_type, "name": name });
            if let Some(images) = item.get("images") {
                let object = meta.as_object_mut()?;
                if let Some(poster) = trakt_image_url(images, "poster") {
                    object.insert("poster".to_string(), json!(poster));
                }
                if let Some(background) = trakt_image_url(images, "fanart") {
                    object.insert("background".to_string(), json!(background));
                }
                if let Some(logo) = trakt_image_url(images, "logo") {
                    object.insert("logo".to_string(), json!(logo));
                }
            }
            if let Some(overview) = item.get("overview").and_then(Value::as_str) {
                meta.as_object_mut()?
                    .insert("description".to_string(), json!(overview));
            }
            if let Some(genres) = item.get("genres").and_then(Value::as_array) {
                meta.as_object_mut()?
                    .insert("genres".to_string(), json!(genres));
            }
            if let Some(rating) = item.get("rating").and_then(Value::as_f64) {
                meta.as_object_mut()?
                    .insert("imdbRating".to_string(), json!(rating));
            }
            if let Some(year) = item.get("year").and_then(Value::as_i64) {
                meta.as_object_mut()?
                    .insert("releaseInfo".to_string(), json!(year.to_string()));
            }
            Some(meta)
        })
        .collect();
    if metas.is_empty() {
        return None;
    }
    serde_json::to_string(&metas).ok()
}

fn activity_at(activities: Option<&Value>, group: &str, field: &str) -> Option<String> {
    activities?
        .get(group)?
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub(crate) fn trakt_activity_diff_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let current = request.get("current").filter(|v| !v.is_null());
    let previous = request.get("previous").filter(|v| !v.is_null());
    let has = |key: &str| request.get(key).and_then(Value::as_bool).unwrap_or(false);
    let changed = |group: &str, field: &str| {
        current.is_none()
            || activity_at(current, group, field) != activity_at(previous, group, field)
    };

    let result = json!({
        "playbackChanged": !has("hasPlayback") || changed("movies", "paused_at") || changed("episodes", "paused_at"),
        "watchlistMoviesChanged": !has("hasWatchlistMovies") || changed("movies", "watchlisted_at"),
        "watchlistShowsChanged": !has("hasWatchlistShows") || changed("shows", "watchlisted_at"),
        "watchedMoviesChanged": !has("hasWatchedMovies") || changed("movies", "watched_at"),
        "watchedShowsChanged": !has("hasWatchedShows") || changed("episodes", "watched_at"),
    });
    serde_json::to_string(&result).ok()
}

pub(crate) fn trakt_playback_items_dedup_json(items_json: &str) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;

    fn saved_at_str(item: &Value) -> &str {
        item.get("savedAt").and_then(Value::as_str).unwrap_or("")
    }

    fn episode_rank(item: &Value) -> Option<(i64, i64)> {
        Some((
            item.get("lastEpisodeSeason").and_then(Value::as_i64)?,
            item.get("lastEpisodeNumber").and_then(Value::as_i64)?,
        ))
    }

    let mut best: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for item in items {
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if id.is_empty() {
            continue;
        }
        let cur = saved_at_str(&item).to_string();
        match best.get(&id) {
            None => {
                best.insert(id, item);
            }
            Some(existing) => {
                let incoming_rank = episode_rank(&item);
                let existing_rank = episode_rank(existing);
                let incoming_is_watched =
                    item.get("continueWatchingBadge").and_then(Value::as_str) == Some("upNext");
                let existing_is_watched = existing
                    .get("continueWatchingBadge")
                    .and_then(Value::as_str)
                    == Some("upNext");
                let incoming_wins = match (incoming_rank, existing_rank) {
                    (Some(incoming_rank), Some(existing_rank))
                        if incoming_is_watched || existing_is_watched =>
                    {
                        incoming_rank > existing_rank
                    }
                    _ => cur.as_str() > saved_at_str(existing),
                };
                if incoming_wins {
                    best.insert(id, item);
                }
            }
        }
    }

    let mut deduped: Vec<Value> = best.into_values().collect();
    deduped.sort_by(|a, b| saved_at_str(b).cmp(saved_at_str(a)));
    serde_json::to_string(&deduped).ok()
}
