use crate::accounts::external_sync::{
    simkl_library_to_items_json, simkl_mark_watched_body_json, simkl_merge_playback_progress_json,
    simkl_watched_to_ids_json, simkl_watching_to_items_json, simkl_watchlist_body_json,
    trakt_collection_body_json, trakt_history_episodes_to_ids_json, trakt_id_from_source,
    trakt_ids_from_content_id_json, trakt_mark_watched_body_json,
    trakt_playback_items_to_library_json, trakt_up_next_to_items_json,
    trakt_watchlist_to_items_json,
};
use crate::catalog;
use serde_json::{Map, Value, json};

const TRAKT_API: &str = "https://api.trakt.tv";
pub(super) const SIMKL_SCOPE: &str = "media:read media:write";
pub(super) const SIMKL_API: &str = "https://api.simkl.com";
const MDBLIST_API: &str = "https://api.mdblist.com";
const SIMKL_APP_NAME: &str = "fluxa";
const SIMKL_USER_AGENT: &str = concat!("fluxa/", env!("CARGO_PKG_VERSION"));

fn str_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn headers(provider: &str, args: &Value) -> Value {
    let token = str_field(args, "token");
    let client_id = str_field(args, "clientId");
    let mut headers = Map::new();
    headers.insert("Content-Type".into(), json!("application/json"));
    match provider {
        "trakt" => {
            headers.insert("User-Agent".into(), json!(SIMKL_USER_AGENT));
            headers.insert("trakt-api-version".into(), json!("2"));
            headers.insert("trakt-api-key".into(), json!(client_id));
        }
        "simkl" => {
            headers.insert("simkl-api-key".into(), json!(client_id));
            headers.insert("User-Agent".into(), json!(SIMKL_USER_AGENT));
        }
        _ => {}
    }
    if !token.is_empty() {
        headers.insert("Authorization".into(), json!(format!("Bearer {token}")));
    }
    Value::Object(headers)
}

fn with_api_key(url: &str, args: &Value) -> String {
    let key = str_field(args, "apiKey");
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}apikey={key}")
}

fn with_simkl_app(url: &str) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!(
        "{url}{separator}app-name={SIMKL_APP_NAME}&app-version={}",
        env!("CARGO_PKG_VERSION")
    )
}

pub(super) fn request(
    provider: &str,
    args: &Value,
    method: &str,
    url: String,
    body: Value,
) -> Value {
    let url = match provider {
        "mdblist" if str_field(args, "token").is_empty() => with_api_key(&url, args),
        "simkl" => with_simkl_app(&url),
        _ => url,
    };
    json!({"method": method, "url": url, "headers": headers(provider, args), "body": body})
}

fn from_mdblist_plan(args: &Value, plan: Option<String>) -> Option<Value> {
    let plan: Value = serde_json::from_str(&plan?).ok()?;
    Some(request(
        "mdblist",
        args,
        str_field(&plan, "method"),
        str_field(&plan, "url").to_string(),
        plan.get("body").cloned().unwrap_or(Value::Null),
    ))
}

pub(crate) fn provider_auth_request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let client_id = str_field(&args, "clientId");
    let plan = match (provider, str_field(&args, "operation")) {
        ("trakt", operation @ ("start" | "poll" | "refresh")) => {
            let operation = match operation {
                "start" => "device_start",
                "poll" => "device_poll",
                other => other,
            };
            let mut oauth = args.clone();
            oauth["service"] = json!("trakt");
            oauth["operation"] = json!(operation);
            let plan: Value = serde_json::from_str(
                &crate::accounts::oauth::oauth_request_plan_json(&oauth.to_string())?,
            )
            .ok()?;
            request(
                provider,
                &json!({"clientId": client_id}),
                "POST",
                str_field(&plan, "url").to_string(),
                plan["body"].clone(),
            )
        }
        ("simkl", "start") => request(
            provider,
            &args,
            "POST",
            format!("{SIMKL_API}/oauth2/device"),
            json!({"client_id": client_id, "scope": SIMKL_SCOPE}),
        ),
        ("simkl", "poll") => request(
            provider,
            &args,
            "POST",
            format!("{SIMKL_API}/oauth2/token"),
            json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": str_field(&args, "code"),
                "client_id": client_id,
            }),
        ),
        ("simkl", "refresh") => request(
            provider,
            &args,
            "POST",
            format!("{SIMKL_API}/oauth2/token"),
            json!({
                "grant_type": "refresh_token",
                "refresh_token": str_field(&args, "refreshToken"),
                "client_id": client_id,
            }),
        ),
        ("mdblist", operation @ ("start" | "poll" | "refresh")) => {
            let (path, mut body) = match operation {
                "start" => ("device-authorization", json!({"scope": "write"})),
                "poll" => (
                    "token",
                    json!({
                        "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                        "device_code": str_field(&args, "code"),
                        "scope": "write",
                    }),
                ),
                _ => (
                    "token",
                    json!({
                        "grant_type": "refresh_token",
                        "refresh_token": str_field(&args, "refreshToken"),
                    }),
                ),
            };
            body["client_id"] = json!(client_id);
            json!({
                "method": "POST",
                "url": format!("{MDBLIST_API}/oauth/{path}/"),
                "headers": {"Content-Type": "application/x-www-form-urlencoded"},
                "body": body,
            })
        }
        _ => return None,
    };
    serde_json::to_string(&plan).ok()
}

pub(crate) fn provider_auth_outcome_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let operation = str_field(&args, "operation");
    let status = args.get("status").and_then(Value::as_u64).unwrap_or(0) as u16;
    let body = args.get("body").cloned().unwrap_or(Value::Null);
    let now = args.get("nowSeconds").and_then(Value::as_i64).unwrap_or(0);
    let ok = (200..300).contains(&status);

    if operation == "start" {
        if !ok {
            return serde_json::to_string(&json!({"state": "error"})).ok();
        }
        let user_code = str_field(&body, "user_code");
        let device_code = str_field(&body, "device_code");
        return serde_json::to_string(&json!({
            "state": "pending",
            "device": {
                "userCode": user_code,
                "deviceCode": device_code,
                "verificationUrl": if provider != "trakt" {
                    str_field(&body, "verification_uri")
                } else {
                    str_field(&body, "verification_url")
                },
                "interval": body.get("interval").and_then(Value::as_i64).unwrap_or(5),
                "expiresAt": now + body.get("expires_in").and_then(Value::as_i64).unwrap_or(600),
            }
        }))
        .ok();
    }

    let state = match provider {
        "simkl" | "mdblist" if ok && body.get("access_token").is_some() => "success",
        "simkl" | "mdblist" => match str_field(&body, "error") {
            "authorization_pending" => "pending",
            "slow_down" => "slow_down",
            _ => "error",
        },
        "trakt" if status == 429 => "slow_down",
        "trakt" => crate::accounts::oauth::oauth_response_outcome("trakt", "device_poll", status),
        _ => "error",
    };
    let state = if operation == "refresh" && state != "success" {
        "error"
    } else {
        state
    };
    if state != "success" {
        return serde_json::to_string(&json!({"state": state})).ok();
    }
    let created_at = body
        .get("created_at")
        .and_then(Value::as_i64)
        .unwrap_or(now);
    let expires_at = body
        .get("expires_in")
        .and_then(Value::as_i64)
        .map(|expires_in| created_at + expires_in);
    serde_json::to_string(&json!({
        "state": "success",
        "auth": {
            "accessToken": body.get("access_token"),
            "refreshToken": body.get("refresh_token"),
            "expiresAt": expires_at,
        }
    }))
    .ok()
}

pub(crate) fn provider_library_requests_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let get = |key: &str, url: String| {
        let mut plan = request(provider, &args, "GET", url, Value::Null);
        plan["key"] = json!(key);
        plan
    };
    let requests = match provider {
        "trakt" => {
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
                format!(
                    "{TRAKT_API}/users/hidden/dropped?type=show&extended=full,images&limit=1000"
                ),
            ));
            requests.push(get(
                "playback",
                format!("{TRAKT_API}/sync/playback?extended=full,images"),
            ));
            requests
        }
        "mdblist" => vec![
            get(
                "watchlist",
                catalog::mdblist::mdblist_watchlist_items_url(None, "{}"),
            ),
            get(
                "watched",
                catalog::mdblist::mdblist_sync_get_url("watched", "{}")?,
            ),
            get(
                "dropped",
                catalog::mdblist::mdblist_sync_get_url("dropped", "{}")?,
            ),
            get(
                "playback",
                catalog::mdblist::mdblist_sync_get_url("playback", "{}")?,
            ),
        ],
        _ => return None,
    };
    serde_json::to_string(&requests).ok()
}

fn parsed(json: Option<String>) -> Value {
    json.and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_else(|| json!([]))
}

fn response_str(responses: &Value, key: &str) -> String {
    responses
        .get(key)
        .map(Value::to_string)
        .unwrap_or_else(|| "[]".to_string())
}

fn concat(lists: &[Value]) -> Value {
    Value::Array(
        lists
            .iter()
            .filter_map(Value::as_array)
            .flatten()
            .cloned()
            .collect(),
    )
}

fn merge_maps(maps: &[Value]) -> Value {
    let mut merged = Map::new();
    for map in maps.iter().filter_map(Value::as_object) {
        merged.extend(map.clone());
    }
    Value::Object(merged)
}

fn simkl_bucket(responses: &Value, status: &str) -> (String, String) {
    let shows = concat(&[
        responses
            .get(&format!("{status}_shows"))
            .and_then(|value| value.get("shows"))
            .cloned()
            .unwrap_or(json!([])),
        responses
            .get(&format!("{status}_anime"))
            .and_then(|value| value.get("anime"))
            .and_then(Value::as_array)
            .map(|anime| {
                Value::Array(
                    anime
                        .iter()
                        .map(|entry| {
                            let mut entry = entry.clone();
                            entry["anime"] = json!(true);
                            if let Some(show) = entry.get("show").or_else(|| entry.get("anime")) {
                                entry["show"] = show.clone();
                            }
                            entry
                        })
                        .collect(),
                )
            })
            .unwrap_or(json!([])),
    ]);
    let movies = responses
        .get(&format!("{status}_movies"))
        .and_then(|value| value.get("movies"))
        .cloned()
        .unwrap_or(json!([]));
    (shows.to_string(), movies.to_string())
}

fn mdblist_entries(body: &Value, kind: &str) -> Vec<Value> {
    body.get(kind)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn mdblist_items(body: &Value) -> Value {
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

fn trakt_progress_number(entry: &Value, key: &str) -> i64 {
    entry
        .pointer(&format!("/progress/{key}"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

fn trakt_show_finished(entry: &Value) -> bool {
    let aired = trakt_progress_number(entry, "aired");
    aired > 0 && trakt_progress_number(entry, "completed") >= aired
}

fn trakt_next_episode_aired(entry: &Value, now: i64) -> bool {
    entry
        .pointer("/progress/next_episode/first_aired")
        .and_then(Value::as_str)
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .is_some_and(|aired| aired.timestamp() <= now)
}

fn trakt_continue_watching(playback: &Value, up_next: &Value) -> Value {
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

fn watched_map(items: &Value) -> Value {
    Value::Object(
        items
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|item| Some((item.get("id")?.as_str()?.to_string(), json!(true))))
            .collect(),
    )
}

pub(crate) fn provider_library_snapshot_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let responses = args.get("responses").cloned().unwrap_or(json!({}));
    let snapshot = match str_field(&args, "provider") {
        "trakt" => {
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
        "simkl" => {
            let items = |status: &str| {
                let (shows, movies) = simkl_bucket(&responses, status);
                parsed(simkl_library_to_items_json(&shows, &movies))
            };
            let (completed_shows, completed_movies) = simkl_bucket(&responses, "completed");
            let (watching_shows, watching_movies) = simkl_bucket(&responses, "watching");
            let watching = parsed(simkl_watching_to_items_json(
                &watching_shows,
                &watching_movies,
            ));
            let continue_watching = parsed(simkl_merge_playback_progress_json(
                &watching.to_string(),
                &response_str(&responses, "playback"),
            ));
            json!({
                "watchlist": items("plantowatch"),
                "liked": [],
                "completed": items("completed"),
                "watched": parsed(simkl_watched_to_ids_json(&completed_shows, &completed_movies)),
                "dropped": items("dropped"),
                "onHold": items("hold"),
                "continueWatching": continue_watching,
            })
        }
        "mdblist" => {
            let empty = json!({});
            let watched = mdblist_items(responses.get("watched").unwrap_or(&empty));
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
            json!({
                "watchlist": parsed(catalog::mdblist::mdblist_list_items_response_to_metas_json(&response_str(&responses, "watchlist"))),
                "liked": [],
                "completed": watched,
                "watched": watched_map(&watched),
                "dropped": mdblist_items(responses.get("dropped").unwrap_or(&empty)),
                "continueWatching": parsed(trakt_playback_items_to_library_json(&playback.to_string())),
            })
        }
        _ => return None,
    };
    serde_json::to_string(&snapshot).ok()
}

fn content_type(item: &Value) -> &str {
    match str_field(item, "type") {
        "movie" => "movie",
        _ => "series",
    }
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

pub(crate) fn provider_write_requests_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let command = args.get("command")?;
    let post = |url: String, body: Value| request(provider, &args, "POST", url, body);
    let mut requests = Vec::new();
    match str_field(command, "type") {
        "simklAccount" if provider == "simkl" => {
            requests.push(post(format!("{SIMKL_API}/users/settings"), json!({})))
        }
        "traktSeasons" if provider == "trakt" => {
            let ids: Value = serde_json::from_str(&trakt_ids_from_content_id_json(str_field(
                command, "seriesId",
            ))?)
            .ok()?;
            let id = ids.as_object()?.values().next().map(|value| {
                value
                    .as_str()
                    .map_or_else(|| value.to_string(), str::to_owned)
            })?;
            requests.push(request(
                provider,
                &args,
                "GET",
                format!("{TRAKT_API}/shows/{id}/seasons?extended=episodes"),
                Value::Null,
            ));
        }
        "toggleWatchlist" => {
            let item = command.get("item")?;
            let id = str_field(item, "id");
            let remove = command
                .get("remove")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            match provider {
                "trakt" => {
                    let body = trakt_collection_body_json(
                        &json!({
                            "idsJson": trakt_ids_from_content_id_json(id)?,
                            "contentType": content_type(item),
                        })
                        .to_string(),
                    )?;
                    let suffix = if remove { "/remove" } else { "" };
                    requests.push(post(
                        format!("{TRAKT_API}/sync/watchlist{suffix}"),
                        serde_json::from_str(&body).ok()?,
                    ));
                }
                "simkl" => {
                    let body = simkl_watchlist_body_json(
                        &json!({
                            "id": id,
                            "providerIds": item.get("providerIds"),
                            "contentType": content_type(item),
                            "command": if remove { "remove" } else { "add" },
                        })
                        .to_string(),
                    )?;
                    let path = if remove {
                        "sync/history/remove"
                    } else {
                        "sync/add-to-list"
                    };
                    requests.push(post(
                        format!("{SIMKL_API}/{path}"),
                        serde_json::from_str(&body).ok()?,
                    ));
                }
                "mdblist" => requests.push(from_mdblist_plan(
                    &args,
                    catalog::mdblist::mdblist_watchlist_mutate_plan(
                        if remove { "remove" } else { "add" },
                        &mdblist_items_json(id, content_type(item))?,
                    ),
                )?),
                _ => return None,
            }
        }
        "markWatched" => {
            let series_id = str_field(command, "seriesId");
            let watched = command
                .get("watched")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let video_ids = command
                .get("videoIds")
                .cloned()
                .filter(|ids| ids.as_array().is_some_and(|ids| !ids.is_empty()))
                .unwrap_or_else(|| json!([series_id]));
            let is_series = video_ids
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .any(|id| id.matches(':').count() >= 2 && id != series_id);
            let rewatch = command
                .get("rewatch")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let suffix = if watched { "" } else { "/remove" };
            match provider {
                "trakt" => requests.push(post(
                    format!("{TRAKT_API}/sync/history{suffix}"),
                    serde_json::from_str(&trakt_mark_watched_body_json(
                        &json!({"videoIds": video_ids}).to_string(),
                    )?)
                    .ok()?,
                )),
                "simkl" => requests.push(post(
                    format!(
                        "{SIMKL_API}/sync/history{suffix}{}",
                        if rewatch && watched {
                            "?allow_rewatch=yes"
                        } else {
                            ""
                        }
                    ),
                    serde_json::from_str(&simkl_mark_watched_body_json(
                        &json!({
                            "videoIds": video_ids,
                            "providerIds": command.get("providerIds"),
                            "rewatch": rewatch && watched,
                            "rewatchId": command.get("rewatchId"),
                            "meta": {"type": if is_series { "series" } else { "movie" }},
                        })
                        .to_string(),
                    )?)
                    .ok()?,
                )),
                "mdblist" => requests.push(from_mdblist_plan(
                    &args,
                    catalog::mdblist::mdblist_sync_mutate_plan(
                        "watched",
                        !watched,
                        &mdblist_items_json(series_id, if is_series { "series" } else { "movie" })?,
                    ),
                )?),
                _ => return None,
            }
        }
        _ => return None,
    }
    serde_json::to_string(&requests).ok()
}

fn mdblist_items_json(id: &str, content_type: &str) -> Option<String> {
    let ids: Value = serde_json::from_str(&trakt_ids_from_content_id_json(id)?).ok()?;
    let mut item = ids.as_object()?.clone();
    item.insert("type".into(), json!(content_type));
    Some(json!([item]).to_string())
}

pub(crate) fn provider_scrobble_request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let action = match str_field(&args, "action") {
        action @ ("start" | "pause" | "stop") => action,
        _ => return None,
    };
    let item_id = str_field(&args, "itemId");
    let progress = args
        .get("progress")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .clamp(0.0, 100.0);
    if provider == "simkl" {
        let mut target = crate::accounts::external_sync::simkl_target(item_id)?;
        if let Some(ids) = args
            .get("providerIds")
            .and_then(crate::accounts::external_sync::simkl_ids_from_provider)
        {
            target.ids = ids;
        }
        let body = match target
            .episode
            .filter(|_| str_field(&args, "metaType") != "movie")
        {
            Some(episode) => json!({
                "show": {"ids": target.ids},
                "episode": {"season": target.season, "number": episode},
                "progress": progress,
            }),
            None => json!({"movie": {"ids": target.ids}, "progress": progress}),
        };
        let plan = request(
            provider,
            &args,
            "POST",
            format!("{SIMKL_API}/scrobble/{action}"),
            body,
        );
        return serde_json::to_string(&plan).ok();
    }
    let episode = crate::accounts::external_sync::trakt_episode_locator_json(item_id)
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .filter(|_| str_field(&args, "metaType") != "movie");
    let show_id = crate::accounts::external_sync::trakt_show_id_from_episode_id(item_id);
    let ids: Value = serde_json::from_str(&trakt_ids_from_content_id_json(&show_id)?).ok()?;
    let body = match &episode {
        Some(episode) => json!({
            "show": {"ids": ids},
            "episode": {"season": episode["season"], "number": episode["episode"]},
            "progress": progress,
        }),
        None => json!({"movie": {"ids": ids}, "progress": progress}),
    };
    let plan = match provider {
        "trakt" => request(
            provider,
            &args,
            "POST",
            format!("{TRAKT_API}/scrobble/{action}"),
            body,
        ),
        "mdblist" => from_mdblist_plan(
            &args,
            catalog::mdblist::mdblist_scrobble_plan(
                action,
                &json!({
                    "ids": ids,
                    "isEpisode": episode.is_some(),
                    "season": episode.as_ref().map(|episode| episode["season"].clone()),
                    "episode": episode.as_ref().map(|episode| episode["episode"].clone()),
                    "progress": progress,
                })
                .to_string(),
            ),
        )?,
        _ => return None,
    };
    serde_json::to_string(&plan).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(json: Option<String>) -> Value {
        serde_json::from_str(&json.unwrap()).unwrap()
    }

    #[test]
    fn trakt_calendar_plan_covers_the_whole_month() {
        let plan = value(trakt_calendar_plan_json(
            &json!({"token": "t", "clientId": "c", "year": 2026, "month": 2}).to_string(),
        ));
        assert_eq!(
            plan[0]["url"],
            "https://api.trakt.tv/calendars/my/shows/2026-02-01/28?extended=full,images"
        );
        assert_eq!(plan[1]["key"], "movies");
    }

    #[test]
    fn trakt_snapshot_dedupes_playback_and_derives_completed_from_progress() {
        let show =
            |imdb: &str, title: &str| json!({"title": title, "ids": {"imdb": imdb, "trakt": 1}});
        let episode = |n: i64| json!({"season": 1, "number": n, "title": "e", "ids": {"trakt": n}});
        let responses = json!({
            "playback": [
                {"id": 1, "type": "episode", "progress": 30.0, "paused_at": "2026-01-01T00:00:00.000Z", "show": show("tt1", "A"), "episode": episode(1)},
                {"id": 2, "type": "episode", "progress": 40.0, "paused_at": "2026-02-01T00:00:00.000Z", "show": show("tt1", "A"), "episode": episode(2)}
            ],
            "progress": [
                {"show": show("tt2", "Done"), "progress": {"aired": 3, "completed": 3, "last_watched_at": "2026-01-01T00:00:00.000Z"}},
                {"show": show("tt3", "Partial"), "progress": {"aired": 3, "completed": 1, "last_watched_at": "2026-01-01T00:00:00.000Z",
                    "next_episode": {"season": 1, "number": 2, "title": "n", "ids": {"trakt": 9}, "first_aired": "2026-01-02T00:00:00.000Z"}}},
                {"show": show("tt4", "Future"), "progress": {"aired": 3, "completed": 1, "last_watched_at": "2026-01-01T00:00:00.000Z",
                    "next_episode": {"season": 1, "number": 2, "title": "n", "ids": {"trakt": 10}, "first_aired": "2030-01-02T00:00:00.000Z"}}}
            ],
            "hidden_dropped": [{"show": show("tt5", "Dropped")}]
        });
        let snapshot = value(provider_library_snapshot_json(
            &json!({"provider": "trakt", "responses": responses, "nowSeconds": 1_800_000_000})
                .to_string(),
        ));
        let ids = |list: &str| -> Vec<String> {
            snapshot[list]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item["id"].as_str().unwrap().to_owned())
                .collect()
        };
        assert_eq!(ids("completed"), ["tt2"]);
        assert_eq!(ids("dropped"), ["tt5"]);
        assert_eq!(ids("continueWatching"), ["tt1", "tt3"]);
        assert_eq!(snapshot["continueWatching"][0]["lastEpisodeNumber"], 2);
    }

    #[test]
    fn simkl_allow_rewatch_is_sent_only_for_explicit_rewatches() {
        let request = |command: Value| {
            value(provider_write_requests_json(
                &json!({"provider":"simkl","token":"t","clientId":"c","command":command})
                    .to_string(),
            ))
        };
        let plain = request(
            json!({"type":"markWatched","seriesId":"tt1","videoIds":["tt1"],"watched":true}),
        );
        assert!(!plain[0]["url"].as_str().unwrap().contains("allow_rewatch"));
        let rewatch = request(
            json!({"type":"markWatched","seriesId":"tt1","videoIds":["tt1"],"watched":true,"rewatch":true}),
        );
        assert!(
            rewatch[0]["url"]
                .as_str()
                .unwrap()
                .contains("allow_rewatch=yes")
        );
        let unwatch = request(
            json!({"type":"markWatched","seriesId":"tt1","videoIds":["tt1"],"watched":false,"rewatch":true}),
        );
        assert!(
            !unwatch[0]["url"]
                .as_str()
                .unwrap()
                .contains("allow_rewatch")
        );
    }

    #[test]
    fn simkl_scrobble_uses_anime_ids_directly() {
        let plan = value(provider_scrobble_request_json(
            r#"{"provider":"simkl","accessToken":"t","action":"start","itemId":"kitsu:46474:5","metaType":"series","progress":12.5}"#,
        ));
        assert_eq!(plan["body"]["show"]["ids"], json!({"kitsu": 46474}));
        assert_eq!(plan["body"]["episode"], json!({"season": 1, "number": 5}));
    }

    #[test]
    fn simkl_device_poll_stays_pending_until_token_arrives() {
        let pending = value(provider_auth_outcome_json(
            r#"{"provider":"simkl","operation":"poll","status":400,"body":{"error":"authorization_pending"}}"#,
        ));
        assert_eq!(pending["state"], "pending");
        let slow = value(provider_auth_outcome_json(
            r#"{"provider":"simkl","operation":"poll","status":400,"body":{"error":"slow_down"}}"#,
        ));
        assert_eq!(slow["state"], "slow_down");
        let expired = value(provider_auth_outcome_json(
            r#"{"provider":"simkl","operation":"poll","status":400,"body":{"error":"expired_token"}}"#,
        ));
        assert_eq!(expired["state"], "error");
        let done = value(provider_auth_outcome_json(
            r#"{"provider":"simkl","operation":"poll","status":200,"nowSeconds":100,"body":{"access_token":"tok","refresh_token":"ref","expires_in":604800}}"#,
        ));
        assert_eq!(done["auth"]["accessToken"], "tok");
        assert_eq!(done["auth"]["refreshToken"], "ref");
        assert_eq!(done["auth"]["expiresAt"], 604900);
    }

    #[test]
    fn simkl_sign_in_uses_the_v2_endpoints_and_asks_for_write_scope() {
        let start = value(provider_auth_request_json(
            r#"{"provider":"simkl","operation":"start","clientId":"cid"}"#,
        ));
        assert!(
            start["url"]
                .as_str()
                .unwrap()
                .starts_with("https://api.simkl.com/oauth2/device")
        );
        assert_eq!(start["body"]["scope"], "media:read media:write");
        let poll = value(provider_auth_request_json(
            r#"{"provider":"simkl","operation":"poll","clientId":"cid","code":"dev"}"#,
        ));
        assert!(poll["url"].as_str().unwrap().contains("/oauth2/token"));
        assert_eq!(poll["body"]["device_code"], "dev");
        let refresh = value(provider_auth_request_json(
            r#"{"provider":"simkl","operation":"refresh","clientId":"cid","refreshToken":"ref"}"#,
        ));
        assert_eq!(refresh["body"]["grant_type"], "refresh_token");
        assert_eq!(refresh["body"]["refresh_token"], "ref");
    }

    #[test]
    fn mdblist_device_sign_in_posts_form_fields() {
        let start = value(provider_auth_request_json(
            r#"{"provider":"mdblist","operation":"start","clientId":"cid"}"#,
        ));
        assert_eq!(
            start["url"],
            "https://api.mdblist.com/oauth/device-authorization/"
        );
        assert_eq!(
            start["headers"]["Content-Type"],
            "application/x-www-form-urlencoded"
        );
        assert_eq!(start["body"]["client_id"], "cid");
        assert_eq!(start["body"]["scope"], "write");
        let poll = value(provider_auth_request_json(
            r#"{"provider":"mdblist","operation":"poll","clientId":"cid","code":"dev"}"#,
        ));
        assert_eq!(poll["body"]["device_code"], "dev");
    }

    #[test]
    fn mdblist_poll_outcomes() {
        let outcome = |status: u16, body: Value| {
            value(provider_auth_outcome_json(
                &json!({"provider": "mdblist", "operation": "poll", "status": status, "body": body, "nowSeconds": 100})
                    .to_string(),
            ))
        };
        assert_eq!(
            outcome(400, json!({"error": "authorization_pending"}))["state"],
            "pending"
        );
        assert_eq!(
            outcome(400, json!({"error": "slow_down"}))["state"],
            "slow_down"
        );
        let done = outcome(
            200,
            json!({"access_token": "a", "refresh_token": "r", "expires_in": 60}),
        );
        assert_eq!(done["auth"]["accessToken"], "a");
        assert_eq!(done["auth"]["expiresAt"], 160);
    }

    #[test]
    fn mdblist_bearer_token_replaces_the_api_key() {
        let requests = value(provider_library_requests_json(
            r#"{"provider":"mdblist","token":"tok","apiKey":"k"}"#,
        ));
        let first = &requests[0];
        assert_eq!(first["headers"]["Authorization"], "Bearer tok");
        assert!(!first["url"].as_str().unwrap().contains("apikey="));
        let legacy = value(provider_library_requests_json(
            r#"{"provider":"mdblist","apiKey":"k"}"#,
        ));
        assert!(legacy[0]["url"].as_str().unwrap().contains("apikey=k"));
    }

    #[test]
    fn trakt_watched_movies_count_as_completed_and_watched() {
        let snapshot = value(provider_library_snapshot_json(
            &json!({
                "provider": "trakt",
                "responses": {
                    "watched_movies": [{"movie": {"title": "Heat", "ids": {"imdb": "tt0113277"}}}],
                    "playback": [{"progress": 40.0, "movie": {"title": "Alien", "ids": {"imdb": "tt0078748"}}}],
                }
            })
            .to_string(),
        ));
        assert_eq!(snapshot["completed"][0]["id"], "tt0113277");
        assert_eq!(snapshot["watched"]["tt0113277"], true);
        assert_eq!(snapshot["continueWatching"][0]["id"], "tt0078748");
    }

    #[test]
    fn simkl_requests_identify_the_app() {
        let plan = value(provider_scrobble_request_json(
            r#"{"provider":"simkl","action":"start","itemId":"tt0078748","metaType":"movie","progress":1}"#,
        ));
        let url = plan["url"].as_str().unwrap();
        assert!(url.contains("app-name=fluxa&app-version="));
        assert!(
            plan["headers"]["User-Agent"]
                .as_str()
                .unwrap()
                .starts_with("fluxa/")
        );
    }

    #[test]
    fn mdblist_requests_carry_the_api_key() {
        let requests = value(provider_library_requests_json(
            r#"{"provider":"mdblist","apiKey":"k"}"#,
        ));
        assert!(
            requests
                .as_array()
                .unwrap()
                .iter()
                .all(|request| request["url"].as_str().unwrap().contains("apikey=k"))
        );
    }

    #[test]
    fn episode_scrobble_targets_the_show() {
        let plan = value(provider_scrobble_request_json(
            r#"{"provider":"trakt","action":"pause","itemId":"tt0903747:2:3","metaType":"series","progress":50}"#,
        ));
        assert_eq!(plan["body"]["show"]["ids"]["imdb"], "tt0903747");
        assert_eq!(plan["body"]["episode"]["number"], 3);
    }
}
