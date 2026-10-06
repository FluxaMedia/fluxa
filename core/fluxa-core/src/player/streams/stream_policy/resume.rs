use serde_json::Value;

const SOURCE_KEYS: [&str; 5] = ["url", "playableUrl", "infoHash", "ytId", "yt_ID"];

fn non_empty(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|text| !text.trim().is_empty())
}

pub(crate) fn saved_stream_reusable(item: &Value, profile: Option<&Value>) -> bool {
    let prefs = profile.unwrap_or(&Value::Null);
    if prefs.get("reuseLastStream").and_then(Value::as_bool) == Some(false) {
        return false;
    }
    let ttl_hours = prefs
        .get("reuseLastStreamTtlHours")
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
        .unwrap_or(24);
    let Some(saved_at) = item
        .get("savedAt")
        .and_then(Value::as_str)
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
    else {
        return true;
    };
    chrono::Utc::now().signed_duration_since(saved_at).num_hours() < ttl_hours
}

pub fn resumes_saved_stream(item: &Value, profile: Option<&Value>) -> bool {
    if item.get("pickSource").is_some() {
        return false;
    }
    let saved = non_empty(item.get("lastStreamUrl"))
        || item
            .get("lastStream")
            .is_some_and(|stream| SOURCE_KEYS.iter().any(|key| non_empty(stream.get(*key))));
    saved && saved_stream_reusable(item, profile)
}
