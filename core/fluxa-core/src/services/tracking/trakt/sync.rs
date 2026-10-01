use crate::catalog::identity::{base_content_id, imdb_regex, parse_episode_locator};
use serde_json::{Map, Value, json};

pub(crate) fn trakt_image_url(images: &Value, kind: &str) -> Option<String> {
    images
        .get(kind)?
        .as_array()?
        .first()?
        .as_str()
        .map(|path| format!("https://{path}"))
}

pub(crate) struct TraktArtwork {
    pub(crate) poster: Option<String>,
    pub(crate) background: Option<String>,
    pub(crate) logo: Option<String>,
}

pub(crate) fn trakt_artwork(source: &Value) -> TraktArtwork {
    match source.get("images") {
        Some(images) => TraktArtwork {
            poster: trakt_image_url(images, "poster"),
            background: trakt_image_url(images, "fanart"),
            logo: trakt_image_url(images, "logo"),
        },
        None => TraktArtwork {
            poster: None,
            background: None,
            logo: None,
        },
    }
}

pub(crate) fn trakt_ids_from_content_id_json(raw_id: &str) -> Option<String> {
    let imdb = imdb_regex().find(raw_id).map(|m| m.as_str().to_string());
    let mut ids = Map::new();
    if let Some(imdb) = imdb {
        ids.insert("imdb".to_string(), Value::String(imdb));
        return serde_json::to_string(&Value::Object(ids)).ok();
    }

    let prefix_number = |prefix: &str| {
        raw_id
            .strip_prefix(prefix)
            .and_then(|rest| rest.split(':').next())
            .and_then(|value| value.parse::<i32>().ok())
    };

    if let Some(tmdb) = prefix_number("tmdb:") {
        ids.insert("tmdb".to_string(), json!(tmdb));
    } else if let Some(tvdb) = prefix_number("tvdb:") {
        ids.insert("tvdb".to_string(), json!(tvdb));
    } else if let Some(trakt) = prefix_number("trakt:") {
        ids.insert("trakt".to_string(), json!(trakt));
    } else if let Some(slug) = raw_id.strip_prefix("slug:").filter(|slug| !slug.is_empty()) {
        ids.insert("slug".to_string(), json!(slug));
    } else if let Some(tmdb) = raw_id
        .split(':')
        .next()
        .and_then(|value| value.parse::<i32>().ok())
    {
        ids.insert("tmdb".to_string(), json!(tmdb));
    }

    if ids.is_empty() {
        None
    } else {
        serde_json::to_string(&Value::Object(ids)).ok()
    }
}

pub(crate) fn trakt_episode_locator_json(video_id: &str) -> Option<String> {
    let (_, season, episode) = parse_episode_locator(video_id)?;
    serde_json::to_string(&json!({
        "season": season,
        "episode": episode
    }))
    .ok()
}

pub(crate) fn trakt_show_id_from_episode_id(video_id: &str) -> String {
    if parse_episode_locator(video_id).is_some() {
        base_content_id(video_id)
    } else {
        video_id.to_string()
    }
}

pub(crate) fn trakt_collection_body_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let ids_json = args.get("idsJson")?.as_str()?;
    let ids: Value = serde_json::from_str(ids_json).ok()?;
    let content_type = args.get("contentType")?.as_str()?;
    let collection = if matches!(content_type, "series" | "show" | "anime") {
        "shows"
    } else {
        "movies"
    };
    serde_json::to_string(&json!({ collection: [{ "ids": ids }] })).ok()
}

pub(crate) fn trakt_id_from_source(source: &Value) -> Option<String> {
    let ids = source.get("ids")?;
    ids.get("imdb")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            ["tmdb", "tvdb", "trakt"].iter().find_map(|key| {
                ids.get(*key)
                    .and_then(Value::as_i64)
                    .map(|n| format!("{key}:{n}"))
            })
        })
}

pub(crate) fn trakt_playback_items_to_library_json(items_json: &str) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let result: Vec<Value> = items
        .iter()
        .filter_map(trakt_playback_item_to_library)
        .collect();
    serde_json::to_string(&result).ok()
}

pub(crate) fn trakt_playback_item_to_library(item: &Value) -> Option<Value> {
    let movie = item.get("movie");
    let show = item.get("show");
    let episode = item.get("episode");
    let source = movie.or(show)?;
    let id = trakt_id_from_source(source)?;
    let progress = item.get("progress").and_then(Value::as_f64).unwrap_or(0.0);
    if progress < 1.0 {
        return None;
    }
    let title = source
        .get("title")
        .or_else(|| source.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("Untitled");
    let episode_title = episode
        .and_then(|e| e.get("title"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let content_type = if movie.is_some() { "movie" } else { "series" };
    let last_video_id = if let Some(ep) = episode {
        let season = ep.get("season").and_then(Value::as_i64).unwrap_or(0);
        let number = ep.get("number").and_then(Value::as_i64).unwrap_or(0);
        format!("{id}:{season}:{number}")
    } else {
        id.clone()
    };
    let episode_season = episode
        .and_then(|e| e.get("season"))
        .and_then(Value::as_i64);
    let episode_number = episode
        .and_then(|e| e.get("number"))
        .and_then(Value::as_i64);
    let saved_at = item.get("paused_at").and_then(Value::as_str).unwrap_or("");
    let artwork = trakt_artwork(source);
    let episode_thumbnail = episode
        .and_then(|e| e.get("images"))
        .and_then(|images| trakt_image_url(images, "screenshot"));
    Some(json!({
        "id": id,
        "name": title,
        "type": content_type,
        "resumeProgressPercent": progress,
        "lastVideoId": last_video_id,
        "lastEpisodeName": if episode_title.is_empty() { Value::Null } else { Value::String(episode_title.to_string()) },
        "lastEpisodeSeason": episode_season,
        "lastEpisodeNumber": episode_number,
        "lastEpisodeThumbnail": episode_thumbnail,
        "savedAt": saved_at,
        "reason": "trakt",
        "poster": artwork.poster,
        "background": artwork.background,
        "logo": artwork.logo
    }))
}

pub(crate) fn trakt_watchlist_to_items_json(movies_json: &str, shows_json: &str) -> Option<String> {
    let movies: Vec<Value> = serde_json::from_str(movies_json).unwrap_or_default();
    let shows: Vec<Value> = serde_json::from_str(shows_json).unwrap_or_default();
    let mut items: Vec<Value> = Vec::new();
    for entry in &movies {
        let Some(movie) = entry.get("movie") else {
            continue;
        };
        let Some(id) = trakt_id_from_source(movie) else {
            continue;
        };
        let name = movie.get("title").and_then(Value::as_str).unwrap_or("");
        let artwork = trakt_artwork(movie);
        items.push(json!({
            "id": id, "name": name, "type": "movie", "source": "trakt",
            "poster": artwork.poster, "background": artwork.background, "logo": artwork.logo
        }));
    }
    for entry in &shows {
        let Some(show) = entry.get("show") else {
            continue;
        };
        let Some(id) = trakt_id_from_source(show) else {
            continue;
        };
        let name = show.get("title").and_then(Value::as_str).unwrap_or("");
        let artwork = trakt_artwork(show);
        items.push(json!({
            "id": id, "name": name, "type": "series", "source": "trakt",
            "poster": artwork.poster, "background": artwork.background, "logo": artwork.logo
        }));
    }
    serde_json::to_string(&items).ok()
}

pub(crate) fn trakt_history_episodes_to_ids_json(history_json: &str) -> Option<String> {
    let history: Vec<Value> = serde_json::from_str(history_json).unwrap_or_default();
    let ids: Map<String, Value> = history
        .iter()
        .filter_map(|entry| {
            let show_id = trakt_id_from_source(entry.get("show")?)?;
            let episode = entry.get("episode")?;
            let season = episode.get("season")?.as_i64().filter(|n| *n > 0)?;
            let number = episode.get("number")?.as_i64().filter(|n| *n > 0)?;
            Some((format!("{show_id}:{season}:{number}"), Value::Bool(true)))
        })
        .collect();
    serde_json::to_string(&Value::Object(ids)).ok()
}
