use crate::services::*;

pub(crate) fn trakt_progress_number(entry: &Value, key: &str) -> i64 {
    entry
        .pointer(&format!("/progress/{key}"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

pub(crate) fn trakt_show_finished(entry: &Value) -> bool {
    let aired = trakt_progress_number(entry, "aired");
    aired > 0 && trakt_progress_number(entry, "completed") >= aired
}

pub(crate) fn trakt_next_episode_aired(entry: &Value, now: i64) -> bool {
    entry
        .pointer("/progress/next_episode/first_aired")
        .and_then(Value::as_str)
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .is_some_and(|aired| aired.timestamp() <= now)
}

pub(crate) fn trakt_continue_watching(playback: &Value, up_next: &Value) -> Value {
    let mut items: Vec<Value> = Vec::new();
    for item in playback.as_array().into_iter().flatten() {
        let id = str_field(item, "id");
        let saved_at = str_field(item, "savedAt");
        match items.iter_mut().find(|kept| str_field(kept, "id") == id) {
            Some(kept) if saved_at > str_field(kept, "savedAt") => *kept = item.clone(),
            Some(_) => {}
            None => items.push(item.clone()),
        }
    }
    for item in up_next.as_array().into_iter().flatten() {
        if !items
            .iter()
            .any(|kept| str_field(kept, "id") == str_field(item, "id"))
        {
            items.push(item.clone());
        }
    }
    items.sort_by(|a, b| str_field(b, "savedAt").cmp(str_field(a, "savedAt")));
    Value::Array(items)
}

pub(crate) fn trakt_calendar_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let year = args.get("year")?.as_i64()? as i32;
    let month = args.get("month")?.as_u64()? as u32;
    let start = chrono::NaiveDate::from_ymd_opt(year, month, 1)?;
    let next = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    let days = (next - start).num_days();
    let requests: Vec<Value> = ["shows", "movies"]
        .into_iter()
        .map(|kind| {
            let mut plan = request(
                "trakt",
                &args,
                "GET",
                format!("{TRAKT_API}/calendars/my/{kind}/{start}/{days}?extended=full,images"),
                Value::Null,
            );
            plan["key"] = json!(kind);
            plan
        })
        .collect();
    serde_json::to_string(&requests).ok()
}

pub(crate) fn trakt_library_requests(args: &Value) -> Option<Vec<Value>> {
    let get = |key: &str, url: String| {
        let mut plan = request("trakt", args, "GET", url, Value::Null);
        plan["key"] = json!(key);
        plan
    };
    let mut requests = Vec::new();
    for kind in ["movies", "shows"] {
        requests.push(get(
            &format!("watchlist_{kind}"),
            format!("{TRAKT_API}/sync/watchlist/{kind}?extended=full,images"),
        ));
        requests.push(get(
            &format!("favorites_{kind}"),
            format!("{TRAKT_API}/sync/favorites/{kind}?extended=full,images"),
        ));
    }
    requests.push(get(
        "watched_movies",
        format!("{TRAKT_API}/sync/watched/movies?extended=full,images"),
    ));
    requests.push(get(
        "progress",
        format!("{TRAKT_API}/sync/progress/watched?extended=full,images&limit=1000"),
    ));
    requests.push(get(
        "history",
        format!("{TRAKT_API}/sync/history/episodes?page=1&limit=100"),
    ));
    requests.push(get(
        "hidden_dropped",
        format!("{TRAKT_API}/users/hidden/dropped?type=show&extended=full,images&limit=1000"),
    ));
    requests.push(get(
        "playback",
        format!("{TRAKT_API}/sync/playback?extended=full,images"),
    ));
    Some(requests)
}

pub(crate) fn trakt_library_snapshot(args: &Value, responses: &Value) -> Value {
    let pair = |prefix: &str| {
        (
            response_str(&responses, &format!("{prefix}_movies")),
            response_str(&responses, &format!("{prefix}_shows")),
        )
    };
    let now = args.get("nowSeconds").and_then(Value::as_i64).unwrap_or(0);
    let (watchlist_movies, watchlist_shows) = pair("watchlist");
    let (favorite_movies, favorite_shows) = pair("favorites");
    let watched_movies = response_str(&responses, "watched_movies");
    let progress: Vec<Value> = responses
        .get("progress")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let finished: Vec<Value> = progress
        .iter()
        .filter(|entry| trakt_show_finished(entry))
        .cloned()
        .collect();
    let started: Vec<Value> = progress
        .iter()
        .filter(|entry| trakt_next_episode_aired(entry, now))
        .cloned()
        .collect();
    let movies_done = parsed(trakt_watchlist_to_items_json(&watched_movies, "[]"));
    let shows_done = parsed(trakt_watchlist_to_items_json(
        "[]",
        &Value::Array(finished).to_string(),
    ));
    let playback = parsed(trakt_playback_items_to_library_json(&response_str(
        &responses, "playback",
    )));
    let up_next = parsed(trakt_up_next_to_items_json(
        &Value::Array(started).to_string(),
    ));
    json!({
        "watchlist": parsed(trakt_watchlist_to_items_json(&watchlist_movies, &watchlist_shows)),
        "liked": parsed(trakt_watchlist_to_items_json(&favorite_movies, &favorite_shows)),
        "completed": concat(&[movies_done.clone(), shows_done]),
        "watched": merge_maps(&[
            watched_map(&movies_done),
            parsed(trakt_history_episodes_to_ids_json(&response_str(&responses, "history"))),
        ]),
        "dropped": parsed(trakt_watchlist_to_items_json("[]", &response_str(&responses, "hidden_dropped"))),
        "continueWatching": trakt_continue_watching(&playback, &up_next),
    })
}

pub(crate) fn trakt_toggle_watchlist(
    args: &Value,
    item: &Value,
    id: &str,
    remove: bool,
) -> Option<Value> {
    let body = trakt_collection_body_json(
        &json!({
            "idsJson": trakt_ids_from_content_id_json(id)?,
            "contentType": content_type(item),
        })
        .to_string(),
    )?;
    let suffix = if remove { "/remove" } else { "" };
    Some(request(
        "trakt",
        args,
        "POST",
        format!("{TRAKT_API}/sync/watchlist{suffix}"),
        serde_json::from_str(&body).ok()?,
    ))
}

pub(crate) fn trakt_mark_watched(args: &Value, change: &WatchedChange) -> Option<Value> {
    let suffix = if change.watched { "" } else { "/remove" };
    Some(request(
        "trakt",
        args,
        "POST",
        format!("{TRAKT_API}/sync/history{suffix}"),
        serde_json::from_str(&trakt_mark_watched_body_json(
            &json!({"videoIds": change.video_ids}).to_string(),
        )?)
        .ok()?,
    ))
}

pub(crate) fn trakt_auth_request(args: &Value, operation: &str) -> Option<Value> {
    let operation = match operation {
        "start" => "device_start",
        "poll" => "device_poll",
        "refresh" => "refresh",
        _ => return None,
    };
    let mut oauth = args.clone();
    oauth["service"] = json!("trakt");
    oauth["operation"] = json!(operation);
    let plan: Value = serde_json::from_str(&crate::accounts::oauth::oauth_request_plan_json(
        &oauth.to_string(),
    )?)
    .ok()?;
    Some(request(
        "trakt",
        &json!({"clientId": str_field(args, "clientId")}),
        "POST",
        str_field(&plan, "url").to_string(),
        plan["body"].clone(),
    ))
}

pub(crate) fn trakt_token_state(status: u16, _body: &Value) -> &'static str {
    if status == 429 {
        "slow_down"
    } else {
        crate::accounts::oauth::oauth_response_outcome("trakt", "device_poll", status)
    }
}
