use serde_json::{Value, json};

pub(crate) fn write_requests_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let profile = args.get("profileIndex").cloned().unwrap_or(json!(1));
    let now = args
        .get("nowMs")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let action = args.get("action")?;
    let requests = match action.get("kind").and_then(Value::as_str) {
        Some("watchlist") => {
            let item = action.get("item")?;
            if action.get("command").and_then(Value::as_str) == Some("remove") {
                vec![json!({"rpc": "sync_delete_library_items", "body": {
                    "p_profile_id": profile,
                    "p_keys": [{"content_id": item.get("id"), "content_type": item.get("type")}],
                }})]
            } else {
                let row: Value = serde_json::from_str(&library_item_request_json(
                    &json!({"item": item, "addedAt": now}).to_string(),
                )?)
                .ok()?;
                vec![
                    json!({"rpc": "sync_push_library_items", "body": {"p_profile_id": profile, "p_items": [row]}}),
                ]
            }
        }
        Some("watched") => watched_requests(action, &profile, now)?,
        Some("progress") => {
            let ms = |field: &str| {
                (action.get(field).and_then(Value::as_f64).unwrap_or(0.0) * 1000.0).round() as i64
            };
            vec![
                json!({"rpc": "sync_push_watch_progress", "body": {"p_profile_id": profile, "p_entries": [{
                    "content_id": action.get("contentId"),
                    "content_type": action.get("contentType"),
                    "video_id": action.get("videoId"),
                    "season": action.get("season"),
                    "episode": action.get("episode"),
                    "position": ms("positionSeconds"),
                    "duration": ms("durationSeconds"),
                    "last_watched": action.get("lastWatched").and_then(Value::as_i64).unwrap_or(now),
                }]}}),
            ]
        }
        _ => Vec::new(),
    };
    serde_json::to_string(&requests).ok()
}

fn watched_requests(action: &Value, profile: &Value, now: i64) -> Option<Vec<Value>> {
    let meta = action.get("meta").unwrap_or(&Value::Null);
    let content_id = action
        .get("seriesId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .or_else(|| meta.get("id").and_then(Value::as_str))?;
    let content_type = meta.get("type").and_then(Value::as_str).unwrap_or("movie");
    let title = meta.get("name").cloned().unwrap_or(Value::Null);
    let episodes = action
        .get("episodeInfos")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let keys: Vec<Value> = if episodes.is_empty() {
        vec![json!({"content_id": content_id})]
    } else {
        episodes
            .iter()
            .map(|e| json!({"content_id": content_id, "season": e.get("season"), "episode": e.get("episode")}))
            .collect()
    };
    if action.get("watched").and_then(Value::as_bool) == Some(false) {
        return Some(vec![
            json!({"rpc": "sync_delete_watched_items", "body": {"p_profile_id": profile, "p_keys": keys}}),
        ]);
    }
    let items: Vec<Value> = keys
        .into_iter()
        .map(|mut key| {
            key["content_type"] = json!(if key.get("season").is_some() {
                "series"
            } else {
                content_type
            });
            key["title"] = title.clone();
            key["watched_at"] = json!(now);
            key
        })
        .collect();
    Some(vec![
        json!({"rpc": "sync_push_watched_items", "body": {"p_profile_id": profile, "p_items": items}}),
    ])
}

fn library_item_request_json(args_json: &str) -> Option<String> {
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
