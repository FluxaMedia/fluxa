use super::helpers::parse;
use serde_json::{Value, json};

pub(crate) fn export_push_plan_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let library = args.get("library")?;
    let now_ms = args.get("nowMs").and_then(Value::as_i64).unwrap_or(0);
    let progress_entries: Vec<Value> = library
        .get("progress")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|entries| entries.iter())
        .filter_map(|(content_id, entry)| {
            let duration = entry.get("duration").and_then(Value::as_f64).unwrap_or(0.0);
            if duration <= 0.0 {
                return None;
            }
            Some(json!({
                "content_id": content_id,
                "content_type": entry.pointer("/meta/type").and_then(Value::as_str).unwrap_or("movie"),
                "video_id": entry.get("lastVideoId").and_then(Value::as_str).unwrap_or(content_id),
                "position": (entry.get("timeOffset").and_then(Value::as_f64).unwrap_or(0.0) * 1000.0).round() as i64,
                "duration": (duration * 1000.0).round() as i64,
                "last_watched": entry.get("savedAt").and_then(Value::as_str).and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok()).map(|value| value.timestamp_millis()).unwrap_or(now_ms),
                "season": entry.get("lastEpisodeSeason"),
                "episode": entry.get("lastEpisodeNumber"),
            }))
        })
        .collect();
    let library_items: Vec<Value> = library
        .get("watchlist")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| !id.is_empty())
        })
        .map(|item| {
            json!({
                "content_id": item.get("id"),
                "content_type": item.get("type").and_then(Value::as_str).unwrap_or("movie"),
                "name": item.get("name").and_then(Value::as_str).unwrap_or(""),
                "poster": item.get("poster"),
                "background": item.get("background"),
            })
        })
        .collect();
    let history_items: Vec<Value> = library
        .get("watched")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|entries| entries.iter())
        .filter(|(_, watched)| watched.as_bool() == Some(true))
        .map(|(video_id, _)| {
            if let Some((content_id, season, episode)) = crate::content_identity::parse_episode_locator(video_id) {
                json!({"content_id": content_id, "content_type": "series", "title": "", "season": season, "episode": episode, "watched_at": now_ms})
            } else {
                json!({"content_id": video_id, "content_type": "movie", "title": "", "watched_at": now_ms})
            }
        })
        .collect();
    serde_json::to_string(&json!({
        "progressEntries": progress_entries,
        "libraryItems": library_items,
        "historyItems": history_items,
    }))
    .ok()
}

pub(crate) fn library_item_request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let item = args.get("item")?;
    let added_at = args.get("addedAt").cloned().unwrap_or(Value::Null);
    serde_json::to_string(&json!({
        "content_id": item.get("id").or_else(|| item.get("contentId")),
        "content_type": item.get("type").or_else(|| item.get("contentType")),
        "name": item.get("name"), "poster": item.get("poster"), "background": item.get("background"),
        "description": item.get("description"), "release_info": item.get("releaseInfo"),
        "imdb_rating": item.get("imdbRating").and_then(Value::as_str).and_then(|v| v.parse::<f64>().ok()).or_else(|| item.get("imdbRating").cloned().and_then(|v| v.as_f64())),
        "genres": item.get("genres"), "poster_shape": item.get("posterShape").and_then(Value::as_str).unwrap_or("POSTER"),
        "addon_base_url": item.get("addonBaseUrl"), "added_at": added_at
    })).ok()
}

pub(crate) fn watched_items_request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let meta = args.get("meta")?;
    let at = args.get("watchedAt").and_then(Value::as_i64)?;
    if meta.get("type").and_then(Value::as_str) == Some("movie") {
        return serde_json::to_string(&json!([{ "content_id": meta.get("id"), "content_type": "movie", "title": meta.get("name"), "watched_at": at }])).ok();
    }
    let items = args.get("episodes").and_then(Value::as_array)?.iter().filter_map(|e| Some(json!({"content_id": meta.get("id"), "content_type": meta.get("type"), "title": meta.get("name"), "season": e.get("season")?.as_i64()?, "episode": e.get("number")?.as_i64()?, "watched_at": at}))).collect::<Vec<_>>();
    serde_json::to_string(&items).ok()
}

pub(crate) fn playback_progress_request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let meta = args.get("meta")?;
    let video = args
        .get("videoId")
        .and_then(Value::as_str)
        .unwrap_or_else(|| meta.get("id").and_then(Value::as_str).unwrap_or(""));
    let mut parts = video.split(':');
    let (season, episode) = match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(_), Some(season), Some(episode), None) => {
            (season.parse::<i64>().ok(), episode.parse::<i64>().ok())
        }
        _ => (None, None),
    };
    serde_json::to_string(&json!({"content_id": meta.get("id"), "content_type": meta.get("type"), "video_id": video, "position": args.get("position"), "duration": args.get("duration"), "last_watched": args.get("watchedAt"), "season": season, "episode": episode, "progress_key": if let (Some(s), Some(e)) = (season, episode) { format!("{}_s{s}e{e}", meta.get("id").and_then(Value::as_str).unwrap_or("")) } else { meta.get("id").and_then(Value::as_str).unwrap_or("").to_string() }})).ok()
}

pub(crate) fn collection_request_json(args_json: &str) -> Option<String> {
    let collection: Value = serde_json::from_str(args_json).ok()?;
    let folders = collection.get("folders").and_then(Value::as_array).cloned().unwrap_or_default().into_iter().map(|folder| {
        let sources = folder.get("sources").and_then(Value::as_array).cloned().filter(|items| !items.is_empty()).unwrap_or_else(|| folder.get("catalogSources").and_then(Value::as_array).cloned().unwrap_or_default().into_iter().map(|source| json!({"provider":"addon","addonId":source.get("addonId"),"catalogId":source.get("catalogId"),"type":source.get("type"),"genre":source.get("genre")})).collect());
        json!({"id":folder.get("id"),"title":folder.get("title"),"coverImageUrl":folder.get("coverImageUrl").or_else(|| folder.get("imageUrl")),"coverEmoji":folder.get("coverEmoji"),"focusGifUrl":folder.get("focusGifUrl"),"focusGifEnabled":folder.get("focusGifEnabled"),"titleLogoUrl":folder.get("titleLogoUrl"),"heroBackdropUrl":folder.get("heroBackdropUrl"),"heroVideoUrl":folder.get("heroVideoUrl"),"tileShape":folder.get("shape"),"hideTitle":folder.get("hideTitle"),"sources":sources})
    }).collect::<Vec<_>>();
    serde_json::to_string(&json!({"id":collection.get("id"),"title":collection.get("title"),"backdropImageUrl":collection.get("imageUrl"),"showOnHome":collection.get("showOnHome"),"pinToTop":collection.get("pinToTop"),"viewMode":collection.get("viewMode"),"showAllTab":collection.get("showAllTab"),"focusGlowEnabled":collection.get("focusGlowEnabled"),"community":collection.get("community"),"folders":folders})).ok()
}
