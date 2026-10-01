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

fn simkl_is_anime(entry: &Value, _source: &Value) -> bool {
    entry.get("anime").and_then(Value::as_bool) == Some(true) || entry.get("anime_type").is_some()
}

fn simkl_anime_id(source: &Value) -> Option<String> {
    let ids = source.get("ids")?;
    ["kitsu", "mal", "anilist", "anidb", "anisearch", "livechart"]
        .iter()
        .find_map(|key| {
            let id = ids.get(key)?;
            let id = id
                .as_str()
                .map(str::to_string)
                .or_else(|| id.as_i64().map(|n| n.to_string()))?;
            (!id.is_empty()).then(|| format!("{key}:{id}"))
        })
}

pub(crate) fn simkl_content_id(source: &Value) -> Option<String> {
    let ids = source.get("ids")?;
    if let Some(id) = simkl_anime_id(source) {
        return Some(id);
    }
    ids.get("imdb")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            ids.get("tmdb").and_then(|value| {
                value
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .map(|value| format!("tmdb:{value}"))
                    .or_else(|| value.as_i64().map(|value| format!("tmdb:{value}")))
            })
        })
        .or_else(|| {
            ids.get("tvdb").and_then(|value| {
                value
                    .as_i64()
                    .map(|value| value.to_string())
                    .or_else(|| value.as_str().filter(|v| !v.is_empty()).map(str::to_string))
                    .map(|value| format!("tvdb:{value}"))
            })
        })
        .or_else(|| {
            ids.get("simkl")
                .and_then(Value::as_i64)
                .map(|value| format!("simkl:{value}"))
        })
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
        let Some(id) = simkl_content_id(show) else {
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
            "source": "simkl",
            "isAnime": simkl_is_anime(entry, show),
            "providerIds": show.get("ids"),
            "simklId": simkl_numeric_id(show)
        }));
    }
    for entry in &movies {
        let Some(movie) = entry.get("movie") else {
            continue;
        };
        let Some(id) = simkl_content_id(movie) else {
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
            "savedAt": saved_at, "reason": "simkl",
            "source": "simkl",
            "providerIds": movie.get("ids")
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
            let id = simkl_content_id(source)?;
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

pub(crate) fn simkl_library_to_items_json(shows_json: &str, movies_json: &str) -> Option<String> {
    let shows = simkl_entries(shows_json, "shows");
    let movies = simkl_entries(movies_json, "movies");
    let mut items: Vec<Value> = Vec::new();
    for entry in &shows {
        let Some(show) = entry.get("show") else {
            continue;
        };
        let anime = simkl_is_anime(entry, show);
        let Some(id) = simkl_content_id(show) else {
            continue;
        };
        let title = show.get("title").and_then(Value::as_str).unwrap_or("");
        let poster = show
            .get("poster")
            .and_then(Value::as_str)
            .map(|p| format!("https://simkl.in/posters/{p}_m.jpg"));
        let background = simkl_fanart_url(show);
        let kind = if entry.get("anime_type").and_then(Value::as_str) == Some("movie") {
            "movie"
        } else {
            "series"
        };
        items.push(json!({ "id": id, "name": title, "type": kind, "isAnime": anime, "source": "simkl", "poster": poster, "background": background, "providerIds": show.get("ids") }));
    }
    for entry in &movies {
        let Some(movie) = entry.get("movie") else {
            continue;
        };
        let Some(id) = simkl_content_id(movie) else {
            continue;
        };
        let title = movie.get("title").and_then(Value::as_str).unwrap_or("");
        let poster = movie
            .get("poster")
            .and_then(Value::as_str)
            .map(|p| format!("https://simkl.in/posters/{p}_m.jpg"));
        let background = simkl_fanart_url(movie);
        items.push(json!({ "id": id, "name": title, "type": "movie", "source": "simkl", "poster": poster, "background": background, "providerIds": movie.get("ids") }));
    }
    serde_json::to_string(&items).ok()
}

pub(crate) fn simkl_watched_to_ids_json(shows_json: &str, movies_json: &str) -> Option<String> {
    let shows = simkl_entries(shows_json, "shows");
    let movies = simkl_entries(movies_json, "movies");
    let mut ids: serde_json::Map<String, Value> = serde_json::Map::new();
    for entry in &shows {
        if let Some(id) = entry.get("show").and_then(simkl_content_id) {
            ids.insert(id, Value::Bool(true));
        }
    }
    for entry in &movies {
        if let Some(id) = entry.get("movie").and_then(simkl_content_id) {
            ids.insert(id, Value::Bool(true));
        }
    }
    serde_json::to_string(&Value::Object(ids)).ok()
}
