pub(crate) mod provider_routes;
use serde_json::{Map, Value, json};
use simkl::*;

pub(crate) mod anilist;
pub(crate) mod auth;
pub(crate) mod fluxa;
pub(crate) mod mdblist;
pub(crate) mod mediaserver;
pub(crate) mod nuvio;
pub(crate) mod publicmetadb;
pub(crate) mod registry;
pub(crate) mod simkl;
pub(crate) mod stremio;
pub(crate) mod tmdb;
pub(crate) mod trakt;

use trakt::*;

pub(crate) const USER_AGENT: &str = concat!("fluxa/", env!("CARGO_PKG_VERSION"));

fn str_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn headers(provider: &str, args: &Value) -> Value {
    let token = str_field(args, "token");
    let mut headers = Map::new();
    headers.insert("Content-Type".into(), json!("application/json"));
    if let Some(extra) = registry::provider(provider).and_then(|provider| provider.headers) {
        for (name, value) in extra(str_field(args, "clientId")) {
            headers.insert(name.into(), json!(value));
        }
    }
    if !token.is_empty() {
        headers.insert("Authorization".into(), json!(format!("Bearer {token}")));
    }
    Value::Object(headers)
}

pub(crate) fn request(
    provider: &str,
    args: &Value,
    method: &str,
    url: String,
    body: Value,
) -> Value {
    let url = match registry::provider(provider).and_then(|provider| provider.prepare_url) {
        Some(prepare) => prepare(url, args),
        None => url,
    };
    json!({"method": method, "url": url, "headers": headers(provider, args), "body": body})
}

pub(crate) fn provider_auth_request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let operation = str_field(&args, "operation");
    let plan = (registry::provider(provider)?.auth_request?)(&args, operation)?;
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
                "verificationUrl": str_field(
                    &body,
                    registry::provider(provider)
                        .map_or("verification_uri", |provider| provider.verification_key),
                ),
                "interval": body.get("interval").and_then(Value::as_i64).unwrap_or(5),
                "expiresAt": now + body.get("expires_in").and_then(Value::as_i64).unwrap_or(600),
            }
        }))
        .ok();
    }

    let state = match registry::provider(provider).and_then(|provider| provider.token_state) {
        Some(token_state) => token_state(status, &body),
        None => "error",
    };
    let state = if matches!(operation, "refresh" | "exchange") && state != "success" {
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

pub(crate) fn provider_authorize_url_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let url = (registry::provider(str_field(&args, "provider"))?.authorize_url?)(&args)?;
    serde_json::to_string(&json!({"url": url})).ok()
}

pub(crate) fn provider_auth_callback_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let url = url::Url::parse(str_field(&args, "url")).ok()?;
    if url.scheme() != "fluxa" || url.host_str() != Some("oauth") {
        return None;
    }
    let provider = url.path().trim_matches('/').to_owned();
    let query = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    };
    let code = query("code").filter(|code| !code.is_empty());
    serde_json::to_string(&json!({
        "provider": provider,
        "code": code,
        "state": query("state"),
        "error": query("error"),
    }))
    .ok()
}

pub(crate) fn provider_library_requests_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let requests = (registry::provider(provider)?.library_requests?)(&args)?;
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

pub(crate) fn provider_calendar_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    (registry::provider(str_field(&args, "provider"))?.calendar_plan?)(args_json)
}

pub(crate) fn provider_library_snapshot_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let responses = args.get("responses").cloned().unwrap_or(json!({}));
    let snapshot = match str_field(&args, "provider") {
        provider => (registry::provider(provider)?.snapshot?)(&args, &responses),
    };
    serde_json::to_string(&snapshot).ok()
}

fn content_type(item: &Value) -> &str {
    match str_field(item, "type") {
        "movie" => "movie",
        _ => "series",
    }
}

pub(crate) struct WatchedChange<'a> {
    pub command: &'a Value,
    pub series_id: &'a str,
    pub video_ids: &'a Value,
    pub watched: bool,
    pub rewatch: bool,
    pub is_series: bool,
}

pub(crate) fn provider_write_requests_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let command = args.get("command")?;
    let mut requests = Vec::new();
    match str_field(command, "type") {
        "toggleWatchlist" => {
            let item = command.get("item")?;
            let id = str_field(item, "id");
            let remove = command
                .get("remove")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let toggle = registry::provider(provider)?.toggle_watchlist?;
            requests.push(toggle(&args, item, id, remove)?);
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
            let mark = registry::provider(provider)?.mark_watched?;
            requests.push(mark(
                &args,
                &WatchedChange {
                    command,
                    series_id,
                    video_ids: &video_ids,
                    watched,
                    rewatch,
                    is_series,
                },
            )?);
        }
        _ => requests.extend((registry::provider(provider)?.write?)(&args, command)?),
    }
    serde_json::to_string(&requests).ok()
}

pub(crate) fn provider_scrobble_request_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let provider = str_field(&args, "provider");
    let action = match str_field(&args, "action") {
        action @ ("start" | "pause" | "stop") => action,
        _ => return None,
    };
    let progress = args
        .get("progress")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .clamp(0.0, 100.0);
    let plan = (registry::provider(provider)?.scrobble?)(&args, action, progress)?;
    serde_json::to_string(&plan).ok()
}

#[cfg(test)]
mod tests;
