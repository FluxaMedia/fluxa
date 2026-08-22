use crate::external_sync::trakt_id_from_source;
use serde_json::{Value, json};

pub(crate) fn simkl_history_request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let imdb_id = args.get("imdbId")?.as_str()?.trim();
    if imdb_id.is_empty() {
        return None;
    }
    let is_series = args.get("isSeries")?.as_bool()?;
    let ids = json!({ "imdb": imdb_id });
    if !is_series {
        return serde_json::to_string(&json!({ "movies": [{ "ids": ids }] })).ok();
    }
    let seasons = args
        .get("episodesBySeasonNumber")
        .and_then(Value::as_object)
        .map(|seasons| {
            seasons
                .iter()
                .filter_map(|(season, episodes)| {
                    let season = season.parse::<i64>().ok()?;
                    let episodes = episodes
                        .as_array()?
                        .iter()
                        .filter_map(Value::as_i64)
                        .map(|number| json!({ "number": number }))
                        .collect::<Vec<_>>();
                    Some(json!({ "number": season, "episodes": episodes }))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    serde_json::to_string(&json!({ "shows": [{ "ids": ids, "seasons": seasons }] })).ok()
}

pub(crate) fn simkl_watchlist_request_json(args_json: &str, remove: bool) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let imdb_id = args.get("imdbId")?.as_str()?.trim();
    if imdb_id.is_empty() {
        return None;
    }
    let is_series = args.get("isSeries")?.as_bool()?;
    let item = if remove {
        json!({ "ids": { "imdb": imdb_id } })
    } else {
        json!({ "ids": { "imdb": imdb_id }, "to": "plantowatch" })
    };
    serde_json::to_string(&if is_series {
        json!({ "shows": [item] })
    } else {
        json!({ "movies": [item] })
    })
    .ok()
}

pub(crate) fn simkl_playback_delete_ids_json(args_json: &str) -> Option<String> {
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

pub(crate) fn simkl_playback_item_to_continue_meta_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let item = args.get("item")?;
    serde_json::to_string(&simkl_playback_item_to_continue_meta(item)?).ok()
}

fn simkl_playback_item_to_continue_meta(item: &Value) -> Option<Value> {
    let movie = item.get("movie");
    let show = item.get("show").or_else(|| item.get("anime"));
    let episode = item.get("episode");
    let source = movie.or(show)?;
    let id = trakt_id_from_source(source)?;
    let progress_percent = item
        .get("progress")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .clamp(0.0, 100.0);
    let title = source
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Untitled");
    let episode_title = episode.and_then(|e| e.get("title")).and_then(Value::as_str);
    let content_type = if movie.is_some() { "movie" } else { "series" };
    let last_video_id = episode.map(|e| {
        let season = e.get("season").and_then(Value::as_i64).unwrap_or(1);
        let number = e.get("number").and_then(Value::as_i64).unwrap_or(1);
        format!("{id}:{season}:{number}")
    });
    let saved_at = item.get("paused_at").and_then(Value::as_str).unwrap_or("");
    Some(json!({
        "id": id,
        "name": title,
        "type": content_type,
        "resumeProgressPercent": progress_percent,
        "lastVideoId": last_video_id,
        "lastEpisodeName": episode_title,
        "savedAt": saved_at,
        "reason": "Simkl"
    }))
}
