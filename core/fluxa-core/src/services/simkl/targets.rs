use crate::catalog::identity::parse_video_id_json;
use serde_json::{Value, json};

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
