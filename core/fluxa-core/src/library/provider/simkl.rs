use super::*;

pub(super) fn simkl_bucket(responses: &Value, status: &str) -> (String, String) {
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



pub(crate) fn simkl_library_snapshot(_args: &Value, responses: &Value) -> Value {
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

pub(crate) fn simkl_toggle_watchlist(
    args: &Value,
    item: &Value,
    id: &str,
    remove: bool,
) -> Option<Value> {
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
    Some(request(
        "simkl",
        args,
        "POST",
        format!("{SIMKL_API}/{path}"),
        serde_json::from_str(&body).ok()?,
    ))
}

pub(crate) fn simkl_mark_watched(args: &Value, change: &WatchedChange) -> Option<Value> {
    let suffix = if change.watched { "" } else { "/remove" };
    let rewatch = change.rewatch && change.watched;
    Some(request(
        "simkl",
        args,
        "POST",
        format!(
            "{SIMKL_API}/sync/history{suffix}{}",
            if rewatch { "?allow_rewatch=yes" } else { "" }
        ),
        serde_json::from_str(&simkl_mark_watched_body_json(
            &json!({
                "videoIds": change.video_ids,
                "providerIds": change.command.get("providerIds"),
                "rewatch": rewatch,
                "rewatchId": change.command.get("rewatchId"),
                "meta": {"type": if change.is_series { "series" } else { "movie" }},
            })
            .to_string(),
        )?)
        .ok()?,
    ))
}

pub(crate) fn simkl_auth_request(args: &Value, operation: &str) -> Option<Value> {
    let client_id = str_field(args, "clientId");
    let (path, body) = match operation {
        "start" => ("device", json!({"client_id": client_id, "scope": SIMKL_SCOPE})),
        "exchange" => (
            "token",
            json!({
                "grant_type": "authorization_code",
                "code": str_field(args, "code"),
                "code_verifier": str_field(args, "codeVerifier"),
                "redirect_uri": SIMKL_REDIRECT_URI,
                "client_id": client_id,
            }),
        ),
        "poll" => (
            "token",
            json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": str_field(args, "code"),
                "client_id": client_id,
            }),
        ),
        "refresh" => (
            "token",
            json!({
                "grant_type": "refresh_token",
                "refresh_token": str_field(args, "refreshToken"),
                "client_id": client_id,
            }),
        ),
        _ => return None,
    };
    Some(request(
        "simkl",
        args,
        "POST",
        format!("{SIMKL_API}/oauth2/{path}"),
        body,
    ))
}

pub(crate) fn simkl_token_state(status: u16, body: &Value) -> &'static str {
    if (200..300).contains(&status) && body.get("access_token").is_some() {
        return "success";
    }
    match str_field(body, "error") {
        "authorization_pending" => "pending",
        "slow_down" => "slow_down",
        _ => "error",
    }
}

pub(crate) fn simkl_authorize_url(args: &Value) -> Option<String> {
    use base64::Engine;
    use sha2::{Digest, Sha256};
    let verifier = str_field(args, "codeVerifier");
    if verifier.len() < 43 {
        return None;
    }
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(verifier.as_bytes()));
    let url = url::Url::parse_with_params(
        "https://simkl.com/oauth2/authorize",
        [
            ("response_type", "code"),
            ("client_id", str_field(args, "clientId")),
            ("redirect_uri", SIMKL_REDIRECT_URI),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("state", str_field(args, "state")),
            ("scope", SIMKL_SCOPE),
        ],
    )
    .ok()?;
    Some(url.to_string())
}
