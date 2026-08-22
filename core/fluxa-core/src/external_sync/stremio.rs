use serde_json::{Value, json};

pub(crate) fn stremio_library_mutation_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let kind = args.get("kind")?.as_str()?;
    let meta = args.get("meta").or_else(|| args.get("item"));
    let now_ms = args
        .get("nowMs")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let watched_at =
        chrono::DateTime::from_timestamp_millis(now_ms).map(|value| value.to_rfc3339());
    let item_value = |source: &Value, state: Value, extra: Value| {
        let id = source.get("id").and_then(Value::as_str).unwrap_or("");
        if id.is_empty() {
            return None;
        }
        let mut item = json!({
            "_id": id,
            "name": source.get("name").and_then(Value::as_str).unwrap_or(""),
            "type": source.get("type").and_then(Value::as_str).unwrap_or("movie"),
            "poster": source.get("poster"), "background": source.get("background"), "logo": source.get("logo"),
            "state": state,
        });
        if let (Some(target), Some(fields)) = (item.as_object_mut(), extra.as_object()) {
            target.extend(fields.clone());
        }
        Some(item)
    };
    let changes: Vec<Value> = match kind {
        "watchlist" => {
            let source = meta?;
            let removed = args.get("command").and_then(Value::as_str) == Some("remove");
            item_value(source, json!({"lastWatched": null, "timeOffset": 0, "duration": 0, "videoId": null, "timesWatched": 0, "flaggedWatched": 0}), json!({"removed": if removed { 1 } else { 0 }})).into_iter().collect()
        }
        "progress" => {
            let source = meta?;
            let progress = args.get("progress")?;
            let last_watched = progress
                .get("lastWatched")
                .and_then(Value::as_i64)
                .and_then(chrono::DateTime::from_timestamp_millis)
                .map(|value| value.to_rfc3339());
            item_value(source, json!({
                "lastWatched": last_watched,
                "timeOffset": progress.get("positionSeconds").and_then(Value::as_f64).unwrap_or_default().max(0.0).round() as i64,
                "duration": progress.get("durationSeconds").and_then(Value::as_f64).unwrap_or_default().max(0.0).round() as i64,
                "videoId": progress.get("videoId"),
            }), json!({})).into_iter().collect()
        }
        "watched" => {
            let watched = args
                .get("watched")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let timestamp = watched.then_some(watched_at).flatten();
            let episodes = args
                .get("episodes")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if episodes.is_empty() {
                meta.and_then(|source| item_value(source, json!({"lastWatched": timestamp, "timeOffset": 0, "duration": 0, "videoId": null, "timesWatched": if watched { 1 } else { 0 }, "flaggedWatched": if watched { 1 } else { 0 }}), json!({"lastWatched": timestamp}))).into_iter().collect()
            } else {
                episodes.iter().map(|episode| {
                    let content_id = episode.get("contentId").and_then(Value::as_str).unwrap_or("");
                    let video_id = episode.get("videoId").and_then(Value::as_str).filter(|value| !value.is_empty()).map(str::to_string)
                        .unwrap_or_else(|| format!("{content_id}:{}:{}", episode.get("season").and_then(Value::as_i64).unwrap_or_default(), episode.get("episode").and_then(Value::as_i64).unwrap_or_default()));
                    json!({
                        "_id": video_id, "name": episode.get("title").and_then(Value::as_str).or_else(|| meta.and_then(|value| value.get("name")).and_then(Value::as_str)).unwrap_or(""),
                        "type": episode.get("contentType"), "poster": meta.and_then(|value| value.get("poster")), "background": meta.and_then(|value| value.get("background")), "logo": meta.and_then(|value| value.get("logo")),
                        "state": {"lastWatched": timestamp, "timeOffset": 0, "duration": 0, "videoId": video_id, "timesWatched": if watched { 1 } else { 0 }, "flaggedWatched": if watched { 1 } else { 0 }},
                        "lastWatched": timestamp,
                    })
                }).collect()
            }
        }
        _ => return None,
    };
    serde_json::to_string(&changes).ok()
}

pub(crate) fn stremio_watchlist_to_items_json(items_json: &str) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let out: Vec<Value> = items
        .iter()
        .filter(|item| {
            item.get("removed").and_then(Value::as_bool) != Some(true)
                && item.get("temp").and_then(Value::as_bool) != Some(true)
        })
        .filter_map(|item| {
            let id = item
                .get("_id")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())?;
            if crate::content_identity::parse_episode_locator(id).is_some() {
                return None;
            }
            let name = item.get("name").and_then(Value::as_str).unwrap_or("");
            let kind = match item.get("type").and_then(Value::as_str) {
                Some("movie") => "movie",
                _ => "series",
            };
            let mut entry = json!({ "id": id, "name": name, "type": kind, "source": "stremio" });
            if let Some(poster) = item
                .get("poster")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                entry
                    .as_object_mut()?
                    .insert("poster".to_string(), json!(poster));
            }
            Some(entry)
        })
        .collect();
    serde_json::to_string(&out).ok()
}

pub(crate) fn stremio_watched_to_ids_json(items_json: &str) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let mut ids: serde_json::Map<String, Value> = serde_json::Map::new();
    for item in &items {
        let flagged = item.get("state").is_some_and(|s| {
            s.get("flaggedWatched").and_then(Value::as_i64).unwrap_or(0) == 1
                || s.get("timesWatched").and_then(Value::as_i64).unwrap_or(0) > 0
        });
        if !flagged {
            continue;
        }
        if let Some(id) = item
            .get("_id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            ids.insert(id.to_string(), Value::Bool(true));
        }
    }
    serde_json::to_string(&Value::Object(ids)).ok()
}
