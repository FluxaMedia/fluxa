use crate::content_identity::{base_content_id, imdb_regex, parse_episode_locator};
use serde_json::{Map, Value, json};

const TRAKT_API_BASE_URL: &str = "https://api.trakt.tv";

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

pub(crate) fn trakt_sync_item_to_meta_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let item = args.get("item")?;
    let summary = item.get("movie").or_else(|| item.get("show"))?;
    let id = trakt_content_id_from_ids_json(&summary.get("ids")?.to_string())?;
    let year = summary.get("year").and_then(Value::as_i64);
    serde_json::to_string(&json!({"id":id,"name":summary.get("title").and_then(Value::as_str).filter(|name| !name.trim().is_empty()).unwrap_or_else(|| args.get("unknownName").and_then(Value::as_str).unwrap_or("Unknown")),"type":args.get("type")?.as_str()?,"poster":Value::Null,"releaseInfo":year.map(|year| year.to_string()),"released":year.map(|year| format!("{year}-01-01"))})).ok()
}

pub(crate) fn trakt_has_client(api_key: &str) -> bool {
    !api_key.trim().is_empty()
}

pub(crate) fn trakt_bearer(token: &str) -> String {
    format!("Bearer {token}")
}

pub(crate) fn trakt_scrobble_url(action: &str) -> Option<String> {
    match action.trim() {
        "start" => Some(format!("{TRAKT_API_BASE_URL}/scrobble/start")),
        "pause" => Some(format!("{TRAKT_API_BASE_URL}/scrobble/pause")),
        "stop" => Some(format!("{TRAKT_API_BASE_URL}/scrobble/stop")),
        _ => None,
    }
}

pub(crate) fn trakt_playback_url(content_type: Option<&str>) -> Option<String> {
    let suffix = match content_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        None => None,
        Some("movie" | "movies") => Some("movies"),
        Some("series" | "show" | "shows" | "episode" | "episodes") => Some("episodes"),
        Some(_) => return None,
    };
    Some(match suffix {
        Some(suffix) => format!("{TRAKT_API_BASE_URL}/sync/playback/{suffix}"),
        None => format!("{TRAKT_API_BASE_URL}/sync/playback"),
    })
}

pub(crate) fn trakt_token_expires_at(created_at_seconds: i64, expires_in_seconds: i64) -> i64 {
    let refresh_buffer_seconds = 5 * 60;
    let effective_expires_in = (expires_in_seconds - refresh_buffer_seconds).max(0);
    created_at_seconds + effective_expires_in
}

fn number_to_i32(value: &Value) -> Option<i32> {
    value.as_i64().and_then(|value| i32::try_from(value).ok())
}

pub(crate) fn trakt_content_id_from_ids_json(ids_json: &str) -> Option<String> {
    let ids: Value = serde_json::from_str(ids_json).ok()?;
    ids.get("imdb")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            ids.get("tmdb")
                .and_then(number_to_i32)
                .map(|id| format!("tmdb:{id}"))
        })
        .or_else(|| {
            ids.get("tvdb")
                .and_then(number_to_i32)
                .map(|id| format!("tvdb:{id}"))
        })
        .or_else(|| {
            ids.get("slug")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(|slug| format!("trakt:{slug}"))
        })
        .or_else(|| {
            ids.get("trakt")
                .and_then(number_to_i32)
                .map(|id| format!("trakt:{id}"))
        })
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

pub(crate) fn trakt_comments_request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let content_id = args.get("contentId")?.as_str()?;
    let content_type = args.get("contentType")?.as_str()?;
    let resource = match content_type {
        "movie" => "movies",
        "series" => "shows",
        _ => return None,
    };
    let ids_json = trakt_ids_from_content_id_json(content_id)?;
    let ids: Value = serde_json::from_str(&ids_json).ok()?;
    let (lookup_type, id) = ids
        .get("imdb")
        .and_then(Value::as_str)
        .map(|value| ("imdb", value.to_string()))
        .or_else(|| {
            ids.get("tmdb")
                .and_then(Value::as_i64)
                .map(|value| ("tmdb", value.to_string()))
        })
        .or_else(|| {
            ids.get("tvdb")
                .and_then(Value::as_i64)
                .map(|value| ("tvdb", value.to_string()))
        })
        .or_else(|| {
            ids.get("trakt")
                .and_then(Value::as_i64)
                .map(|value| ("trakt", value.to_string()))
        })?;
    serde_json::to_string(&json!({ "resource": resource, "id": id, "lookupType": lookup_type, "wantType": if resource == "shows" { "show" } else { "movie" } })).ok()
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

pub(crate) fn trakt_scrobble_media_id(
    parent_id: &str,
    video_id: Option<&str>,
    media_type: &str,
) -> String {
    if media_type != "series" {
        return video_id.unwrap_or(parent_id).to_string();
    }
    let Some(video_id) = video_id.filter(|value| !value.is_empty()) else {
        return parent_id.to_string();
    };
    let Some((_, season, episode)) = parse_episode_locator(video_id) else {
        return video_id.to_string();
    };
    format!("{parent_id}:{season}:{episode}")
}

pub(crate) fn trakt_oauth_error_code(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    value
        .get("error")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn episode_season_number(episode: &Value) -> Option<(i32, i32)> {
    let parsed = episode
        .get("id")
        .and_then(Value::as_str)
        .and_then(parse_episode_locator);
    let season = episode
        .get("season")
        .and_then(number_to_i32)
        .or_else(|| parsed.as_ref().map(|(_, season, _)| *season));
    let number = episode
        .get("number")
        .and_then(number_to_i32)
        .or_else(|| parsed.as_ref().map(|(_, _, episode)| *episode));
    season.zip(number)
}

pub(crate) fn trakt_history_request_json(meta_json: &str, episodes_json: &str) -> Option<String> {
    let meta: Value = serde_json::from_str(meta_json).ok()?;
    let episodes: Vec<Value> = serde_json::from_str(episodes_json).unwrap_or_default();
    let meta_id = meta.get("id").and_then(Value::as_str).unwrap_or("");
    let ids_json = trakt_ids_from_content_id_json(meta_id).or_else(|| {
        episodes
            .first()
            .and_then(|episode| episode.get("id").and_then(Value::as_str))
            .and_then(trakt_ids_from_content_id_json)
    })?;
    let ids: Value = serde_json::from_str(&ids_json).ok()?;

    if meta.get("type").and_then(Value::as_str) == Some("movie") {
        return serde_json::to_string(&json!({
            "movies": [{ "ids": ids }]
        }))
        .ok();
    }

    let target_episodes = if episodes.is_empty() {
        meta.get("lastVideoId")
            .and_then(Value::as_str)
            .or_else(|| meta.get("id").and_then(Value::as_str))
            .and_then(parse_episode_locator)
            .map(|(_, season, episode)| {
                vec![json!({
                    "season": season,
                    "number": episode
                })]
            })
            .unwrap_or_default()
    } else {
        episodes
    };

    let mut seasons = std::collections::BTreeMap::<i32, Vec<i32>>::new();
    for episode in target_episodes.iter().filter_map(episode_season_number) {
        seasons.entry(episode.0).or_default().push(episode.1);
    }
    if seasons.is_empty() {
        return None;
    }

    let seasons = seasons
        .into_iter()
        .map(|(season, mut episodes)| {
            episodes.sort_unstable();
            episodes.dedup();
            json!({
                "number": season,
                "episodes": episodes.into_iter().map(|number| json!({ "number": number })).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();

    serde_json::to_string(&json!({
        "shows": [{
            "ids": ids,
            "seasons": seasons
        }]
    }))
    .ok()
}

pub(crate) fn trakt_id_from_source(source: &Value) -> Option<String> {
    let ids = source.get("ids")?;
    ids.get("imdb")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            ids.get("tmdb")
                .and_then(Value::as_i64)
                .map(|n| format!("tmdb:{n}"))
        })
}

pub(crate) fn trakt_playback_delete_ids_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let content_id = args.get("contentId")?.as_str()?;
    let ids = args
        .get("items")?
        .as_array()?
        .iter()
        .filter_map(|item| {
            let source = item.get("show").or_else(|| item.get("movie"))?;
            (trakt_id_from_source(source).as_deref() == Some(content_id))
                .then(|| item.get("id")?.as_i64())
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&ids).ok()
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

pub(crate) fn trakt_watched_to_ids_json(movies_json: &str, shows_json: &str) -> Option<String> {
    let movies: Vec<Value> = serde_json::from_str(movies_json).unwrap_or_default();
    let shows: Vec<Value> = serde_json::from_str(shows_json).unwrap_or_default();
    let mut ids: serde_json::Map<String, Value> = serde_json::Map::new();
    for entry in &movies {
        if let Some(id) = entry.get("movie").and_then(trakt_id_from_source) {
            ids.insert(id, Value::Bool(true));
        }
    }
    for entry in &shows {
        let show_id = match entry.get("show").and_then(trakt_id_from_source) {
            Some(id) => id,
            None => continue,
        };
        let seasons = entry
            .get("seasons")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for season in &seasons {
            let s_num = season.get("number").and_then(Value::as_i64).unwrap_or(0);
            let episodes = season
                .get("episodes")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for ep in &episodes {
                let e_num = ep.get("number").and_then(Value::as_i64).unwrap_or(0);
                if s_num > 0 && e_num > 0 {
                    ids.insert(format!("{show_id}:{s_num}:{e_num}"), Value::Bool(true));
                }
            }
        }
    }
    serde_json::to_string(&Value::Object(ids)).ok()
}
