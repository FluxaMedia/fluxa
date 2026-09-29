use super::*;

pub(super) fn mdblist_entries(body: &Value, kind: &str) -> Vec<Value> {
    body.get(kind)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

pub(super) fn mdblist_items(body: &Value) -> Value {
    let mut items = Vec::new();
    for (kind, key, content_type) in [("movies", "movie", "movie"), ("shows", "show", "series")] {
        for entry in mdblist_entries(body, kind) {
            let source = entry.get(key).unwrap_or(&entry);
            let Some(id) = trakt_id_from_source(source) else {
                continue;
            };
            items.push(json!({
                "id": id,
                "type": content_type,
                "name": source.get("title").and_then(Value::as_str).unwrap_or(""),
                "source": "mdblist",
            }));
        }
    }
    Value::Array(items)
}

pub(super) fn mdblist_watched(body: &Value) -> (Value, Value) {
    let mut completed = mdblist_items(&json!({"movies": mdblist_entries(body, "movies")}))
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut watched = watched_map(&Value::Array(completed.clone()))
        .as_object()
        .cloned()
        .unwrap_or_default();
    let episodes: Vec<Value> = mdblist_entries(body, "episodes")
        .into_iter()
        .filter_map(|entry| {
            let episode = entry.get("episode")?;
            Some(json!({"show": episode.get("show")?, "episode": episode}))
        })
        .collect();
    let ids: Value = serde_json::from_str(
        &trakt_history_episodes_to_ids_json(&Value::Array(episodes).to_string())
            .unwrap_or_default(),
    )
    .unwrap_or(json!({}));
    watched.extend(ids.as_object().cloned().unwrap_or_default());
    for entry in mdblist_entries(body, "shows") {
        let Some(show) = entry.get("show") else {
            continue;
        };
        let Some(id) = trakt_id_from_source(show) else {
            continue;
        };
        let total = show
            .get("total_aired_episodes")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let prefix = format!("{id}:");
        let seen = watched
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .count() as i64;
        if total > 0 && seen >= total {
            completed.push(json!({
                "id": id,
                "type": "series",
                "name": show.get("title").and_then(Value::as_str).unwrap_or(""),
                "source": "mdblist",
            }));
        }
    }
    (Value::Array(completed), Value::Object(watched))
}

pub(super) fn mdblist_playback(entries: &Value) -> Value {
    let entries: Vec<Value> = entries
        .as_array()
        .into_iter()
        .flatten()
        .cloned()
        .map(|mut entry| {
            let progress = match entry.get("progress") {
                Some(Value::String(text)) => text.parse::<f64>().unwrap_or(0.0),
                Some(other) => other.as_f64().unwrap_or(0.0),
                None => 0.0,
            };
            entry["progress"] = json!(progress);
            if let Some(fields) = entry.as_object_mut() {
                fields.retain(|_, value| !value.is_null());
            }
            if let Some(name) = entry.pointer("/episode/name").cloned() {
                entry["episode"]["title"] = name;
            }
            entry
        })
        .collect();
    let mut items = parsed(trakt_playback_items_to_library_json(
        &Value::Array(entries).to_string(),
    ));
    for item in items.as_array_mut().into_iter().flatten() {
        item["reason"] = json!("mdblist");
    }
    items
}

pub(super) fn mdblist_up_next(upnext: &Value, watched: &Value) -> Value {
    let mut imdb_by_tmdb = std::collections::HashMap::new();
    let shows = mdblist_entries(watched, "shows")
        .into_iter()
        .filter_map(|entry| entry.get("show").cloned());
    for show in shows {
        if let (Some(tmdb), Some(imdb)) = (
            show.pointer("/ids/tmdb").and_then(Value::as_i64),
            show.pointer("/ids/imdb").and_then(Value::as_str),
        ) {
            imdb_by_tmdb.insert(tmdb, imdb.to_string());
        }
    }
    let entries: Vec<Value> = mdblist_entries(upnext, "items")
        .into_iter()
        .filter_map(|entry| {
            let mut show = entry.get("show")?.clone();
            let tmdb = show.pointer("/ids/tmdb").and_then(Value::as_i64)?;
            if let Some(imdb) = imdb_by_tmdb.get(&tmdb) {
                show["ids"]["imdb"] = json!(imdb);
            }
            let next = entry.get("next_episode")?;
            Some(json!({
                "show": show,
                "progress": {
                    "last_watched_at": entry.get("last_watched_at"),
                    "next_episode": {
                        "season": next.get("season"),
                        "number": next.get("episode"),
                        "title": next.get("title"),
                    },
                },
            }))
        })
        .collect();
    let mut items = parsed(trakt_up_next_to_items_json(
        &Value::Array(entries).to_string(),
    ));
    for item in items.as_array_mut().into_iter().flatten() {
        item["reason"] = json!("mdblist");
    }
    items
}


pub(crate) fn mdblist_calendar_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let year = args.get("year")?.as_i64()? as i32;
    let month = args.get("month")?.as_u64()? as u32;
    let start = chrono::NaiveDate::from_ymd_opt(year, month, 1)?;
    let next = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    let end = next.pred_opt()?;
    let mut plan = request(
        "mdblist",
        &args,
        "GET",
        format!("{MDBLIST_API}/calendar/events?start={start}&end={end}&limit=1000&favorite_cast=false"),
        Value::Null,
    );
    plan["key"] = json!("events");
    serde_json::to_string(&json!([plan])).ok()
}


pub(super) fn mdblist_items_json(id: &str, content_type: &str) -> Option<String> {
    let ids: Value = serde_json::from_str(&trakt_ids_from_content_id_json(id)?).ok()?;
    let mut item = ids.as_object()?.clone();
    item.insert("type".into(), json!(content_type));
    Some(json!([item]).to_string())
}



pub(crate) fn mdblist_library_requests(args: &Value) -> Option<Vec<Value>> {
    let get = |key: &str, url: String| {
        let mut plan = request("mdblist", args, "GET", url, Value::Null);
        plan["key"] = json!(key);
        plan
    };
    Some(vec![
        get(
            "watchlist",
            catalog::mdblist::mdblist_watchlist_items_url(None, "{}"),
        ),
        get(
            "watched",
            catalog::mdblist::mdblist_sync_get_url("watched", r#"{"limit":1000}"#)?,
        ),
        get(
            "dropped",
            catalog::mdblist::mdblist_sync_get_url("dropped", "{}")?,
        ),
        get(
            "playback",
            catalog::mdblist::mdblist_sync_get_url("playback", "{}")?,
        ),
        get("upnext", catalog::mdblist::mdblist_upnext_url(None, "{}")?),
    ])
}

pub(crate) fn mdblist_library_snapshot(args: &Value, responses: &Value) -> Value {
    let empty = json!({});
    let watched_body = responses.get("watched").unwrap_or(&empty);
    let (completed, watched) = mdblist_watched(watched_body);
    let playback = responses
        .get("playback")
        .map(|body| {
            concat(&[
                body.get("movies").cloned().unwrap_or(json!([])),
                body.get("episodes").cloned().unwrap_or(json!([])),
                body.get("playback").cloned().unwrap_or(json!([])),
                if body.is_array() {
                    body.clone()
                } else {
                    json!([])
                },
            ])
        })
        .unwrap_or(json!([]));
    let playback = mdblist_playback(&playback);
    let up_next = mdblist_up_next(responses.get("upnext").unwrap_or(&empty), watched_body);
    json!({
        "watchlist": parsed(catalog::mdblist::mdblist_list_items_response_to_metas_json(&response_str(&responses, "watchlist"))),
        "liked": [],
        "completed": completed,
        "watched": watched,
        "dropped": mdblist_items(responses.get("dropped").unwrap_or(&empty)),
        "continueWatching": trakt_continue_watching(&playback, &up_next),
    })
}

pub(crate) fn mdblist_toggle_watchlist(
    args: &Value,
    item: &Value,
    id: &str,
    remove: bool,
) -> Option<Value> {
    from_mdblist_plan(
        args,
        catalog::mdblist::mdblist_watchlist_mutate_plan(
            if remove { "remove" } else { "add" },
            &mdblist_items_json(id, content_type(item))?,
        ),
    )
}

pub(crate) fn mdblist_mark_watched(args: &Value, change: &WatchedChange) -> Option<Value> {
    let plan = if change.is_series {
        let episodes: Vec<&str> = change
            .video_ids
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|id| id.matches(':').count() >= 2)
            .collect();
        let body = trakt_mark_watched_body_json(&json!({"videoIds": episodes}).to_string())?;
        catalog::mdblist::mdblist_watched_body_plan(
            !change.watched,
            serde_json::from_str(&body).ok()?,
        )
    } else {
        catalog::mdblist::mdblist_sync_mutate_plan(
            "watched",
            !change.watched,
            &mdblist_items_json(change.series_id, "movie")?,
        )
    };
    from_mdblist_plan(args, plan)
}

pub(crate) fn mdblist_auth_request(args: &Value, operation: &str) -> Option<Value> {
    let (path, mut body) = match operation {
        "start" => ("device-authorization", json!({"scope": "write"})),
        "poll" => (
            "token",
            json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": str_field(args, "code"),
                "scope": "write",
            }),
        ),
        "refresh" => (
            "token",
            json!({
                "grant_type": "refresh_token",
                "refresh_token": str_field(args, "refreshToken"),
            }),
        ),
        _ => return None,
    };
    body["client_id"] = json!(str_field(args, "clientId"));
    Some(json!({
        "method": "POST",
        "url": format!("{MDBLIST_API}/oauth/{path}/"),
        "headers": {
            "Content-Type": "application/x-www-form-urlencoded",
            "User-Agent": SIMKL_USER_AGENT,
        },
        "body": body,
    }))
}

pub(crate) fn mdblist_token_state(status: u16, body: &Value) -> &'static str {
    if (200..300).contains(&status) && body.get("access_token").is_some() {
        return "success";
    }
    match str_field(body, "error") {
        "authorization_pending" => "pending",
        "slow_down" => "slow_down",
        _ => "error",
    }
}
