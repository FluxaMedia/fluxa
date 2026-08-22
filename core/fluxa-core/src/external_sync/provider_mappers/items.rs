use crate::external_sync::{trakt_artwork, trakt_id_from_source, trakt_image_url};
use serde_json::{Value, json};

fn simkl_entries(json: &str, key: &str) -> Vec<Value> {
    match serde_json::from_str::<Value>(json).ok() {
        Some(Value::Array(entries)) => entries,
        Some(Value::Object(response)) => response
            .get(key)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn simkl_numeric_id(source: &Value) -> Option<i64> {
    source.get("ids")?.get("simkl").and_then(Value::as_i64)
}

fn simkl_fanart_url(source: &Value) -> Option<String> {
    source
        .get("fanart")
        .and_then(Value::as_str)
        .map(|p| format!("https://simkl.in/fanart/{p}_m.jpg"))
}

fn simkl_episode_marker(entry: &Value, field: &str) -> Option<(i64, i64)> {
    entry
        .get(field)
        .and_then(Value::as_str)
        .and_then(|value| value.strip_prefix('S'))
        .and_then(|value| value.split_once('E'))
        .and_then(|(season, episode)| {
            Some((season.parse::<i64>().ok()?, episode.parse::<i64>().ok()?))
        })
        .filter(|(season, episode)| *season > 0 && *episode > 0)
}

fn simkl_last_watched_episode(entry: &Value) -> Option<(i64, i64)> {
    if let Some(episode) = simkl_episode_marker(entry, "last_watched") {
        return Some(episode);
    }
    entry
        .get("seasons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|season| {
            let season_number = season.get("number").and_then(Value::as_i64).unwrap_or(0);
            season
                .get("episodes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(move |episode| {
                    let episode_number = episode.get("number").and_then(Value::as_i64)?;
                    (season_number > 0 && episode_number > 0)
                        .then_some((season_number, episode_number))
                })
        })
        .max()
}

pub(crate) fn simkl_watching_to_items_json(shows_json: &str, movies_json: &str) -> Option<String> {
    let shows = simkl_entries(shows_json, "shows");
    let movies = simkl_entries(movies_json, "movies");
    let mut items: Vec<Value> = Vec::new();
    for entry in &shows {
        let Some(show) = entry.get("show") else {
            continue;
        };
        let Some(id) = trakt_id_from_source(show) else {
            continue;
        };
        let title = show.get("title").and_then(Value::as_str).unwrap_or("");
        let poster = show
            .get("poster")
            .and_then(Value::as_str)
            .map(|p| format!("https://simkl.in/posters/{p}_m.jpg"));
        let background = simkl_fanart_url(show);
        let saved_at = entry
            .get("last_watched_at")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let last_episode = simkl_last_watched_episode(entry);
        let next_episode = simkl_episode_marker(entry, "next_to_watch");
        let card_episode = next_episode.or(last_episode);
        let last_video_id =
            card_episode.map(|(season, episode)| format!("{id}:{season}:{episode}"));
        items.push(json!({
            "id": id, "type": "series", "name": title,
            "poster": poster,
            "background": background,
            "lastVideoId": last_video_id,
            "lastEpisodeSeason": card_episode.map(|(season, _)| season),
            "lastEpisodeNumber": card_episode.map(|(_, episode)| episode),
            "continueWatchingBadge": next_episode.map(|_| "upNext"),
            "savedAt": saved_at, "reason": "simkl",
            "simklId": simkl_numeric_id(show)
        }));
    }
    for entry in &movies {
        let Some(movie) = entry.get("movie") else {
            continue;
        };
        let Some(id) = trakt_id_from_source(movie) else {
            continue;
        };
        let title = movie.get("title").and_then(Value::as_str).unwrap_or("");
        let poster = movie
            .get("poster")
            .and_then(Value::as_str)
            .map(|p| format!("https://simkl.in/posters/{p}_m.jpg"));
        let background = simkl_fanart_url(movie);
        let saved_at = entry
            .get("last_watched")
            .and_then(Value::as_str)
            .unwrap_or_default();
        items.push(json!({
            "id": id, "type": "movie", "name": title,
            "poster": poster,
            "background": background,
            "savedAt": saved_at, "reason": "simkl"
        }));
    }
    serde_json::to_string(&items).ok()
}

struct SimklPlaybackEntry {
    id: String,
    episode: Option<(i64, i64)>,
    progress: f64,
    episode_title: Option<String>,
    simkl_id: Option<i64>,
    is_anime: bool,
}

fn simkl_playback_progress_entries(playback_json: &str) -> Vec<SimklPlaybackEntry> {
    simkl_entries(playback_json, "")
        .iter()
        .filter_map(|entry| {
            let progress = entry.get("progress").and_then(Value::as_f64).unwrap_or(0.0);
            if progress <= 0.0 {
                return None;
            }
            let is_anime = entry.get("anime").is_some();
            let source = entry
                .get("movie")
                .or_else(|| entry.get("show"))
                .or_else(|| entry.get("anime"))?;
            let id = trakt_id_from_source(source)?;
            let episode = entry.get("episode").and_then(|e| {
                Some((
                    e.get("season").and_then(Value::as_i64)?,
                    e.get("number").and_then(Value::as_i64)?,
                ))
            });
            let episode_title = entry
                .get("episode")
                .and_then(|e| e.get("title"))
                .and_then(Value::as_str)
                .filter(|title| !title.trim().is_empty())
                .map(str::to_string);
            Some(SimklPlaybackEntry {
                id,
                episode,
                progress: progress.clamp(0.0, 100.0),
                episode_title,
                simkl_id: simkl_numeric_id(source),
                is_anime,
            })
        })
        .collect()
}

pub(crate) fn simkl_merge_playback_progress_json(
    items_json: &str,
    playback_json: &str,
) -> Option<String> {
    let mut items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let playback = simkl_playback_progress_entries(playback_json);
    for entry in &playback {
        let SimklPlaybackEntry {
            id,
            episode,
            progress,
            episode_title,
            simkl_id,
            is_anime,
        } = entry;
        for item in items.iter_mut() {
            if item.get("id").and_then(Value::as_str) != Some(id.as_str()) {
                continue;
            }
            if let Some((season, number)) = episode {
                let matches_episode = item.get("lastEpisodeSeason").and_then(Value::as_i64)
                    == Some(*season)
                    && item.get("lastEpisodeNumber").and_then(Value::as_i64) == Some(*number);
                if !matches_episode {
                    continue;
                }
            }
            let Some(obj) = item.as_object_mut() else {
                continue;
            };
            if let Some(simkl_id) = simkl_id {
                obj.insert("simklId".to_string(), json!(simkl_id));
            }
            if *is_anime {
                obj.insert("isAnime".to_string(), json!(true));
            }
            obj.insert("resumeProgressPercent".to_string(), json!(progress));
            obj.insert("continueWatchingBadge".to_string(), Value::Null);
            if let Some(title) = episode_title {
                obj.insert("lastEpisodeName".to_string(), json!(title));
            }
        }
    }
    serde_json::to_string(&items).ok()
}

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

pub(crate) fn simkl_watchlist_to_items_json(shows_json: &str, movies_json: &str) -> Option<String> {
    let shows = simkl_entries(shows_json, "shows");
    let movies = simkl_entries(movies_json, "movies");
    let mut items: Vec<Value> = Vec::new();
    for entry in &shows {
        let Some(show) = entry.get("show") else {
            continue;
        };
        let Some(id) = trakt_id_from_source(show) else {
            continue;
        };
        let title = show.get("title").and_then(Value::as_str).unwrap_or("");
        let poster = show
            .get("poster")
            .and_then(Value::as_str)
            .map(|p| format!("https://simkl.in/posters/{p}_m.jpg"));
        let background = simkl_fanart_url(show);
        items.push(json!({ "id": id, "name": title, "type": "series", "source": "simkl", "poster": poster, "background": background }));
    }
    for entry in &movies {
        let Some(movie) = entry.get("movie") else {
            continue;
        };
        let Some(id) = trakt_id_from_source(movie) else {
            continue;
        };
        let title = movie.get("title").and_then(Value::as_str).unwrap_or("");
        let poster = movie
            .get("poster")
            .and_then(Value::as_str)
            .map(|p| format!("https://simkl.in/posters/{p}_m.jpg"));
        let background = simkl_fanart_url(movie);
        items.push(json!({ "id": id, "name": title, "type": "movie", "source": "simkl", "poster": poster, "background": background }));
    }
    serde_json::to_string(&items).ok()
}

pub(crate) fn simkl_watched_to_ids_json(shows_json: &str, movies_json: &str) -> Option<String> {
    let shows = simkl_entries(shows_json, "shows");
    let movies = simkl_entries(movies_json, "movies");
    let mut ids: serde_json::Map<String, Value> = serde_json::Map::new();
    for entry in &shows {
        if let Some(id) = entry.get("show").and_then(trakt_id_from_source) {
            ids.insert(id, Value::Bool(true));
        }
    }
    for entry in &movies {
        if let Some(id) = entry.get("movie").and_then(trakt_id_from_source) {
            ids.insert(id, Value::Bool(true));
        }
    }
    serde_json::to_string(&Value::Object(ids)).ok()
}
