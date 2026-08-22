use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibraryLocalStateRequest {
    #[serde(default)]
    library: Value,
    #[serde(default)]
    primary_id: Option<String>,
    #[serde(default)]
    fallback_id: Option<String>,
}

pub(crate) fn library_local_state_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<LibraryLocalStateRequest>(request_json).ok()?;
    let id = request
        .primary_id
        .as_deref()
        .or(request.fallback_id.as_deref())
        .unwrap_or("");
    let progress = request
        .library
        .get("progress")
        .and_then(|value| value.get(id))
        .cloned()
        .unwrap_or(Value::Null);
    let is_in_watchlist = request
        .library
        .get("watchlist")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items
                .iter()
                .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
        });
    let watched_video_ids = request
        .library
        .get("watched")
        .and_then(Value::as_object)
        .map(|watched| {
            watched
                .iter()
                .filter(|(key, value)| key.starts_with(id) && value.as_bool().unwrap_or(false))
                .map(|(key, _)| Value::String(key.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    serde_json::to_string(&json!({
        "progress": progress,
        "isInWatchlist": is_in_watchlist,
        "watchedVideoIds": watched_video_ids
    }))
    .ok()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreferenceUpdateRequest {
    #[serde(default)]
    existing: Map<String, Value>,
    key: String,
    value: Value,
}

pub(crate) fn preferences_schema_json() -> String {
    json!({
        "keys": [
            "language",
            "startPage",
            "preferredPlayer",
            "externalPlayerTarget",
            "streamSourceSelectionMode",
            "streamSourceRegexPattern",
            "preferredAudioLanguage",
            "secondaryAudioLanguage",
            "audioProcessingMode",
            "preferredSubtitleLanguage",
            "secondarySubtitleLanguage",
            "subtitleSize",
            "playbackSpeed",
            "torrentSpeedPreset",
            "torrentCachePreset",
            "downloadSourceSelectionMode",
            "downloadSubtitleLanguage"
        ]
    })
    .to_string()
}

pub(crate) fn apply_preference_update_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<PreferenceUpdateRequest>(request_json).ok()?;
    let mut updated = request.existing;
    let value = normalize_preference_value(&request.key, request.value);
    updated.insert(request.key, value);
    serde_json::to_string(&Value::Object(updated)).ok()
}

fn normalize_preference_value(key: &str, value: Value) -> Value {
    match key {
        "preferredPlayer" => enum_string(value, &["mpv", "exoplayer", "external"], "mpv"),
        "audioProcessingMode" => enum_string(value, &["reference", "balanced", "night"], "reference"),
        "externalPlayerTarget" => Value::String(value.as_str().unwrap_or("mpv").trim().to_string()),
        "streamSourceSelectionMode" | "downloadSourceSelectionMode" => {
            enum_string(value, &["manual", "first", "best", "regex"], "manual")
        }
        "downloadSubtitleLanguage" => enum_string(
            value,
            &["off", "preferred", "tr", "en", "ja", "es", "fr", "de"],
            "preferred",
        ),
        "torrentSpeedPreset" => enum_string(value, &["default", "fast", "ultra_fast"], "default"),
        "torrentCachePreset" => {
            enum_string(value, &["auto", "2gb", "5gb", "10gb", "unlimited"], "auto")
        }
        "subtitleSize" => enum_string(value, &["50", "75", "100", "125", "150", "200"], "100"),
        _ => value,
    }
}
fn enum_string(value: Value, allowed: &[&str], fallback: &str) -> Value {
    let text = value.as_str().unwrap_or(fallback);
    if allowed.contains(&text) {
        Value::String(text.to_string())
    } else {
        Value::String(fallback.to_string())
    }
}
