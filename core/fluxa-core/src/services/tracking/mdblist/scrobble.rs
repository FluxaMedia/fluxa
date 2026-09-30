use super::helpers::{build_url, plan};
use serde_json::{Value, json};

// Confirmed against a real account: the body nests under "movie" or "show"
// (Trakt-shaped), not a flat {ids, type}. Season/episode live inside the
// "show" object itself, not a separate "episode" object.
fn scrobble_target_body(args: &Value) -> Option<serde_json::Map<String, Value>> {
    let ids = args.get("ids").filter(|value| value.is_object())?.clone();
    let mut body = serde_json::Map::new();
    if args.get("isEpisode").and_then(Value::as_bool) == Some(true) {
        let mut show = serde_json::Map::new();
        show.insert("ids".to_string(), ids);
        if let Some(season) = args.get("season") {
            show.insert("season".to_string(), season.clone());
        }
        if let Some(episode) = args.get("episode") {
            show.insert("episode".to_string(), episode.clone());
        }
        body.insert("show".to_string(), Value::Object(show));
    } else {
        body.insert("movie".to_string(), json!({ "ids": ids }));
    }
    Some(body)
}

pub(crate) fn mdblist_scrobble_plan(action: &str, args_json: &str) -> Option<String> {
    let path = match action {
        "start" => "/scrobble/start",
        "pause" => "/scrobble/pause",
        "stop" => "/scrobble/stop",
        "clear" => "/scrobble/clear",
        _ => return None,
    };
    let args: Value = serde_json::from_str(args_json).ok()?;
    let mut body = scrobble_target_body(&args)?;
    if let Some(progress) = args.get("progress") {
        body.insert("progress".to_string(), progress.clone());
    }
    plan("POST", build_url(path, &[]), Some(Value::Object(body)))
}

pub(crate) fn mdblist_checkin_plan(method: &str, args_json: &str) -> Option<String> {
    let body = if method == "POST" {
        let args: Value = serde_json::from_str(args_json).ok()?;
        Some(Value::Object(scrobble_target_body(&args)?))
    } else {
        None
    };
    match method {
        "GET" | "POST" | "DELETE" => plan(method, build_url("/checkin", &[]), body),
        _ => None,
    }
}
