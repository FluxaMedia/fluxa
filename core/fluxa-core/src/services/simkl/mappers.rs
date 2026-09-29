use crate::catalog::identity::parse_video_id_json;
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

pub(crate) fn simkl_watchlist_to_items_json(shows_json: &str, movies_json: &str) -> Option<String> {
    simkl_library_to_items_json(shows_json, movies_json)
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

const SIMKL_ANIME_ID_KEYS: [&str; 7] = [
    "kitsu",
    "mal",
    "anilist",
    "anidb",
    "anisearch",
    "livechart",
    "animeplanet",
];
const SIMKL_TEXT_ID_KEYS: [&str; 4] = ["tvdb", "animeplanet", "letterboxd", "traktslug"];
const SIMKL_ID_KEYS: [&str; 14] = [
    "simkl",
    "imdb",
    "tmdb",
    "tvdb",
    "mal",
    "anidb",
    "anilist",
    "kitsu",
    "anisearch",
    "animeplanet",
    "livechart",
    "letterboxd",
    "netflix",
    "traktslug",
];

pub(crate) struct SimklTarget {
    pub(crate) ids: Value,
    pub(crate) season: i64,
    pub(crate) episode: Option<i64>,
}

pub(crate) fn simkl_ids_from_provider(ids: &Value) -> Option<Value> {
    let known: serde_json::Map<String, Value> = ids
        .as_object()?
        .iter()
        .filter(|(key, value)| {
            SIMKL_ID_KEYS.contains(&key.as_str())
                && (value.as_i64().is_some() || value.as_str().is_some_and(|v| !v.is_empty()))
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    (!known.is_empty()).then(|| Value::Object(known))
}

pub(crate) fn simkl_target(video_id: &str) -> Option<SimklTarget> {
    let parts: Vec<&str> = video_id.split(':').collect();
    if let Some(key) = parts
        .first()
        .filter(|key| SIMKL_ID_KEYS.contains(key) && **key != "tmdb" && **key != "imdb")
    {
        let raw = *parts.get(1)?;
        let id = match raw.parse::<i64>() {
            Ok(number) => json!(number),
            Err(_) if SIMKL_TEXT_ID_KEYS.contains(key) && !raw.is_empty() => json!(raw),
            Err(_) => return None,
        };
        let number = |index: usize| parts.get(index).and_then(|part| part.parse::<i64>().ok());
        let (season, episode) = if SIMKL_ANIME_ID_KEYS.contains(key) {
            match parts.len() {
                0..=2 => (1, None),
                3 => (1, number(2)),
                _ => (number(2).unwrap_or(1), number(3)),
            }
        } else {
            (number(2).unwrap_or(1), number(3))
        };
        return Some(SimklTarget {
            ids: json!({ *key: id }),
            season,
            episode,
        });
    }
    let parsed: Value = serde_json::from_str(&parse_video_id_json(video_id)).ok()?;
    let ids = parsed
        .get("imdb")
        .and_then(Value::as_str)
        .map(|id| json!({"imdb": id}))
        .or_else(|| {
            parsed
                .get("tmdb")
                .and_then(Value::as_str)
                .and_then(|id| id.parse::<i64>().ok())
                .map(|id| json!({"tmdb": id}))
        })?;
    let is_episode = parsed.get("isEpisode").and_then(Value::as_bool) == Some(true);
    Some(SimklTarget {
        ids,
        season: parsed.get("season").and_then(Value::as_i64).unwrap_or(1),
        episode: is_episode.then(|| parsed.get("episode").and_then(Value::as_i64).unwrap_or(1)),
    })
}

pub(crate) fn simkl_mark_watched_body_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let video_ids = args.get("videoIds")?.as_array()?;
    let meta_type = args
        .pointer("/meta/type")
        .and_then(Value::as_str)
        .unwrap_or("movie");
    let watched_at = args
        .get("watchedAtMs")
        .and_then(Value::as_i64)
        .and_then(chrono::DateTime::from_timestamp_millis)
        .map(|value| value.to_rfc3339());
    let provider_ids = args.get("providerIds").and_then(simkl_ids_from_provider);
    let rewatch = args
        .get("rewatch")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let rewatch_id = args.get("rewatchId").and_then(Value::as_i64);
    let mut movies = Vec::new();
    let mut shows: std::collections::HashMap<
        String,
        (Value, std::collections::BTreeMap<i64, Vec<i64>>),
    > = std::collections::HashMap::new();
    for video_id in video_ids.iter().filter_map(Value::as_str) {
        let Some(target) = simkl_target(video_id) else {
            continue;
        };
        let ids = provider_ids.clone().unwrap_or(target.ids);
        if let Some(episode) = target.episode {
            let season = target.season;
            let key = ids.to_string();
            let entry = shows
                .entry(key)
                .or_insert_with(|| (ids, std::collections::BTreeMap::new()));
            entry.1.entry(season).or_default().push(episode);
        } else if meta_type == "series" {
            shows
                .entry(ids.to_string())
                .or_insert_with(|| (ids, std::collections::BTreeMap::new()));
        } else {
            let mut movie = json!({"ids": ids, "watched_at": watched_at.clone().unwrap_or_else(|| "now".to_string())});
            mark_rewatch(&mut movie, rewatch, rewatch_id);
            movies.push(movie);
        }
    }
    let show_values = shows
        .into_values()
        .map(|(ids, seasons)| {
            let tvdb_seasons = ids.get("imdb").is_some() || ids.get("tmdb").is_some();
            let mut show = if seasons.is_empty() {
                json!({"ids": ids, "use_tvdb_anime_seasons": tvdb_seasons})
            } else {
            json!({"ids": ids, "use_tvdb_anime_seasons": tvdb_seasons, "seasons": seasons.into_iter().map(|(number, mut episodes)| {
            episodes.sort_unstable(); episodes.dedup();
            let episodes = episodes.into_iter().map(|number| {
                let mut episode = json!({"number": number});
                if let Some(timestamp) = watched_at.as_ref()
                    && let Some(object) = episode.as_object_mut() {
                        object.insert("watched_at".to_string(), Value::String(timestamp.clone()));
                    }
                episode
            }).collect::<Vec<_>>();
            json!({"number": number, "episodes": episodes})
        }).collect::<Vec<_>>()})
            };
            mark_rewatch(&mut show, rewatch, rewatch_id);
            show
        })
        .collect::<Vec<_>>();
    if movies.is_empty() && show_values.is_empty() {
        return None;
    }
    serde_json::to_string(&json!({"movies": movies, "shows": show_values})).ok()
}

fn mark_rewatch(entry: &mut Value, rewatch: bool, rewatch_id: Option<i64>) {
    if !rewatch {
        return;
    }
    entry["is_rewatch"] = json!(true);
    if let Some(id) = rewatch_id {
        entry["rewatch_id"] = json!(id);
    }
}

pub(crate) fn simkl_watchlist_body_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let id = args.get("id")?.as_str()?;
    let ids = match args.get("providerIds").and_then(simkl_ids_from_provider) {
        Some(ids) => ids,
        None => simkl_target(id)?.ids,
    };
    let entry = if args.get("command").and_then(Value::as_str) == Some("remove") {
        json!({"ids": ids})
    } else {
        json!({"ids": ids, "to": "plantowatch"})
    };
    let body = if args.get("contentType").and_then(Value::as_str) == Some("series") {
        json!({"shows": [entry]})
    } else {
        json!({"movies": [entry]})
    };
    serde_json::to_string(&body).ok()
}

pub(crate) fn simkl_match_episode_json(episodes_json: &str, target_json: &str) -> Option<String> {
    let episodes: Vec<Value> = serde_json::from_str(episodes_json).ok()?;
    let target: Value = serde_json::from_str(target_json).ok()?;
    let release_date = target
        .get("releaseDate")
        .and_then(Value::as_str)
        .unwrap_or("");
    let title = target
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_lowercase();
    let title = title.trim();

    let matched = if !release_date.is_empty() {
        episodes.iter().find(|ep| {
            ep.get("date")
                .and_then(Value::as_str)
                .is_some_and(|d| d.starts_with(release_date))
        })
    } else {
        None
    };

    let matched = matched.or_else(|| {
        if title.is_empty() {
            return None;
        }
        episodes.iter().find(|ep| {
            ep.get("title")
                .and_then(Value::as_str)
                .is_some_and(|t| t.to_lowercase().trim() == title)
        })
    })?;

    let season = matched.get("season").and_then(Value::as_i64)?;
    let episode = matched.get("episode").and_then(Value::as_i64)?;
    serde_json::to_string(&json!({ "season": season, "episode": episode })).ok()
}

pub(crate) fn simkl_lookup_id_for_type(lookup_json: &str, want_type: &str) -> Option<i64> {
    let lookup: Vec<Value> = serde_json::from_str(lookup_json).ok()?;
    lookup
        .iter()
        .find(|item| item.get("type").and_then(Value::as_str) == Some(want_type))
        .and_then(|item| item.get("ids")?.get("simkl")?.as_i64())
}

pub(crate) fn simkl_recommendation_candidates_json(detail_json: &str) -> Option<String> {
    let detail: Value = serde_json::from_str(detail_json).ok()?;
    let recs = detail.get("users_recommendations")?.as_array()?;
    let candidates: Vec<Value> = recs.iter().take(15).cloned().collect();
    serde_json::to_string(&candidates).ok()
}

pub(crate) fn simkl_poster_url(path: &str) -> String {
    format!("https://wsrv.nl/?url=https://simkl.in/posters/{path}_c.webp&q=90")
}

pub(crate) fn simkl_recommendation_to_meta_json(
    rec_json: &str,
    resolved_imdb: &str,
) -> Option<String> {
    let rec: Value = serde_json::from_str(rec_json).ok()?;
    let title = rec.get("title").and_then(Value::as_str)?;
    let type_str = rec.get("type").and_then(Value::as_str).unwrap_or("movie");
    let meta_type = if type_str == "tv" { "series" } else { "movie" };
    let mut meta = json!({ "id": resolved_imdb, "type": meta_type, "name": title });
    if let Some(poster) = rec.get("poster").and_then(Value::as_str) {
        meta.as_object_mut()?
            .insert("poster".to_string(), json!(simkl_poster_url(poster)));
    }
    if let Some(fanart) = rec.get("fanart").and_then(Value::as_str) {
        meta.as_object_mut()?.insert(
            "background".to_string(),
            json!(format!("https://simkl.in/fanart/{fanart}_b.jpg")),
        );
    }
    for (source, target) in [("overview", "description"), ("description", "description")] {
        if let Some(value) = rec.get(source).and_then(Value::as_str) {
            meta.as_object_mut()?
                .insert(target.to_string(), json!(value));
            break;
        }
    }
    if let Some(genres) = rec.get("genres").and_then(Value::as_array) {
        meta.as_object_mut()?
            .insert("genres".to_string(), json!(genres));
    }
    if let Some(rating) = rec.get("rating").and_then(Value::as_f64) {
        meta.as_object_mut()?
            .insert("imdbRating".to_string(), json!(rating));
    }
    if let Some(year) = rec.get("year").and_then(Value::as_i64) {
        meta.as_object_mut()?
            .insert("releaseInfo".to_string(), json!(year.to_string()));
    }
    serde_json::to_string(&meta).ok()
}
