use crate::catalog::identity::parse_video_id_json;
use crate::services::trakt::*;
use serde_json::{Value, json};

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
