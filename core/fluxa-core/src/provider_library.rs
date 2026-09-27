use crate::external_sync::{
    simkl_library_to_items_json, simkl_mark_watched_body_json, simkl_merge_playback_progress_json,
    simkl_watched_to_ids_json, simkl_watching_to_items_json, simkl_watchlist_body_json,
    trakt_collection_body_json, trakt_id_from_source, trakt_ids_from_content_id_json,
    trakt_mark_watched_body_json, trakt_playback_items_to_library_json, trakt_watched_to_ids_json,
    trakt_watchlist_to_items_json,
};
use crate::mdblist_plan;
use serde_json::{Map, Value, json};

const TRAKT_API: &str = "https://api.trakt.tv";
const SIMKL_API: &str = "https://api.simkl.com";

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
            headers.insert("trakt-api-version".into(), json!("2"));
            headers.insert("trakt-api-key".into(), json!(client_id));
        }
        "simkl" => {
            headers.insert("simkl-api-key".into(), json!(client_id));
        }
        _ => {}
    }
    if !token.is_empty() && provider != "mdblist" {
        headers.insert("Authorization".into(), json!(format!("Bearer {token}")));
    }
    Value::Object(headers)
}

fn with_api_key(url: &str, args: &Value) -> String {
    let key = str_field(args, "apiKey");
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}apikey={key}")
}

fn request(provider: &str, args: &Value, method: &str, url: String, body: Value) -> Value {
    let url = if provider == "mdblist" {
        with_api_key(&url, args)
    } else {
        url
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
            let plan: Value =
                serde_json::from_str(&crate::oauth_plan::oauth_request_plan_json(&oauth.to_string())?)
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
            "GET",
            format!("{SIMKL_API}/oauth/pin?client_id={client_id}"),
            Value::Null,
        ),
        ("simkl", "poll") => request(
            provider,
            &args,
            "GET",
            format!(
                "{SIMKL_API}/oauth/pin/{}?client_id={client_id}",
                str_field(&args, "code")
            ),
            Value::Null,
        ),
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
        let result = str_field(&body, "result");
        if !ok || (provider == "simkl" && result != "OK") {
            return serde_json::to_string(&json!({"state": "error"})).ok();
        }
        let user_code = str_field(&body, "user_code");
        let device_code = if provider == "simkl" {
            user_code
        } else {
            str_field(&body, "device_code")
        };
        return serde_json::to_string(&json!({
            "state": "pending",
            "device": {
                "userCode": user_code,
                "deviceCode": device_code,
                "verificationUrl": str_field(&body, "verification_url"),
                "interval": body.get("interval").and_then(Value::as_i64).unwrap_or(5),
                "expiresAt": now + body.get("expires_in").and_then(Value::as_i64).unwrap_or(600),
            }
        }))
        .ok();
    }

    let state = match provider {
        "simkl" if ok && body.get("access_token").is_some() => "success",
        "simkl" if ok => "pending",
        "trakt" if status == 429 => "slow_down",
        "trakt" => crate::oauth_plan::oauth_response_outcome("trakt", "device_poll", status),
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
    let created_at = body.get("created_at").and_then(Value::as_i64).unwrap_or(now);
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
                requests.push(get(
                    &format!("watched_{kind}"),
                    format!("{TRAKT_API}/sync/watched/{kind}?extended=full,images"),
                ));
            }
            requests.push(get(
                "playback",
                format!("{TRAKT_API}/sync/playback?extended=full,images"),
            ));
            requests
        }
        "simkl" => {
            let mut requests = Vec::new();
            for kind in ["shows", "movies", "anime"] {
                for status in ["plantowatch", "watching", "completed", "dropped"] {
                    requests.push(get(
                        &format!("{status}_{kind}"),
                        format!("{SIMKL_API}/sync/all-items/{kind}/{status}?extended=full"),
                    ));
                }
            }
            requests.push(get("playback", format!("{SIMKL_API}/sync/playback")));
            requests
        }
        "mdblist" => vec![
            get(
                "watchlist",
                mdblist_plan::mdblist_watchlist_items_url(None, "{}"),
            ),
            get("watched", mdblist_plan::mdblist_sync_get_url("watched", "{}")?),
            get("dropped", mdblist_plan::mdblist_sync_get_url("dropped", "{}")?),
            get("playback", mdblist_plan::mdblist_sync_get_url("playback", "{}")?),
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
            let (watchlist_movies, watchlist_shows) = pair("watchlist");
            let (favorite_movies, favorite_shows) = pair("favorites");
            let (watched_movies, watched_shows) = pair("watched");
            json!({
                "watchlist": parsed(trakt_watchlist_to_items_json(&watchlist_movies, &watchlist_shows)),
                "liked": parsed(trakt_watchlist_to_items_json(&favorite_movies, &favorite_shows)),
                "completed": parsed(trakt_watchlist_to_items_json(&watched_movies, &watched_shows)),
                "watched": merge_maps(&[
                    watched_map(&parsed(trakt_watchlist_to_items_json(&watched_movies, "[]"))),
                    parsed(trakt_watched_to_ids_json(&watched_movies, &watched_shows)),
                ]),
                "dropped": [],
                "continueWatching": parsed(trakt_playback_items_to_library_json(&response_str(&responses, "playback"))),
            })
        }
        "simkl" => {
            let items = |status: &str| {
                let (shows, movies) = simkl_bucket(&responses, status);
                parsed(simkl_library_to_items_json(&shows, &movies))
            };
            let (completed_shows, completed_movies) = simkl_bucket(&responses, "completed");
            let (watching_shows, watching_movies) = simkl_bucket(&responses, "watching");
            let watching = parsed(simkl_watching_to_items_json(&watching_shows, &watching_movies));
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
                        if body.is_array() { body.clone() } else { json!([]) },
                    ])
                })
                .unwrap_or(json!([]));
            json!({
                "watchlist": parsed(mdblist_plan::mdblist_list_items_response_to_metas_json(&response_str(&responses, "watchlist"))),
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

pub(crate) fn provider_write_requests_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let command = args.get("command")?;
    let post = |url: String, body: Value| request(provider, &args, "POST", url, body);
    let mut requests = Vec::new();
    match str_field(command, "type") {
        "toggleWatchlist" => {
            let item = command.get("item")?;
            let id = str_field(item, "id");
            let remove = command.get("remove").and_then(Value::as_bool).unwrap_or(false);
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
                            "contentType": content_type(item),
                            "command": if remove { "remove" } else { "add" },
                        })
                        .to_string(),
                    )?;
                    let path = if remove { "sync/history/remove" } else { "sync/add-to-list" };
                    requests.push(post(
                        format!("{SIMKL_API}/{path}"),
                        serde_json::from_str(&body).ok()?,
                    ));
                }
                "mdblist" => requests.push(from_mdblist_plan(
                    &args,
                    mdblist_plan::mdblist_watchlist_mutate_plan(
                        if remove { "remove" } else { "add" },
                        &mdblist_items_json(id, content_type(item))?,
                    ),
                )?),
                _ => return None,
            }
        }
        "markWatched" => {
            let series_id = str_field(command, "seriesId");
            let watched = command.get("watched").and_then(Value::as_bool).unwrap_or(true);
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
                    format!("{SIMKL_API}/sync/history{suffix}"),
                    serde_json::from_str(&simkl_mark_watched_body_json(
                        &json!({
                            "videoIds": video_ids,
                            "meta": {"type": if is_series { "series" } else { "movie" }},
                        })
                        .to_string(),
                    )?)
                    .ok()?,
                )),
                "mdblist" => requests.push(from_mdblist_plan(
                    &args,
                    mdblist_plan::mdblist_sync_mutate_plan(
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
    let episode = crate::external_sync::trakt_episode_locator_json(item_id)
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .filter(|_| str_field(&args, "metaType") != "movie");
    let show_id = crate::external_sync::trakt_show_id_from_episode_id(item_id);
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
        "simkl" => request(
            provider,
            &args,
            "POST",
            format!("{SIMKL_API}/scrobble/{action}"),
            body,
        ),
        "mdblist" => from_mdblist_plan(
            &args,
            mdblist_plan::mdblist_scrobble_plan(
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
    fn simkl_pin_poll_stays_pending_until_token_arrives() {
        let pending = value(provider_auth_outcome_json(
            r#"{"provider":"simkl","operation":"poll","status":200,"body":{"result":"KO","message":"Authorization pending"}}"#,
        ));
        assert_eq!(pending["state"], "pending");
        let done = value(provider_auth_outcome_json(
            r#"{"provider":"simkl","operation":"poll","status":200,"body":{"result":"OK","access_token":"tok"}}"#,
        ));
        assert_eq!(done["auth"]["accessToken"], "tok");
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
