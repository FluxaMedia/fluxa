use super::effects::DEFAULT_API_KEY;
use super::state::WatchConfig;
use super::stream_resolution::normalize_youtube_url;
use serde_json::Value;

pub(super) fn parse_watch_config(response: &Value) -> WatchConfig {
    let html = response
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or_default();
    WatchConfig {
        api_key: extract_json_string_field(html, "INNERTUBE_API_KEY")
            .unwrap_or_else(|| DEFAULT_API_KEY.to_string()),
        visitor_data: extract_json_string_field(html, "VISITOR_DATA"),
        player_script_url: extract_json_string_field(html, "PLAYER_JS_URL")
            .or_else(|| extract_json_string_field(html, "jsUrl"))
            .map(|url| normalize_youtube_url(&url)),
    }
}

fn extract_json_string_field(html: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let rest = html.get(html.find(&needle)? + needle.len()..)?;
    let end = rest
        .as_bytes()
        .iter()
        .enumerate()
        .find_map(|(index, byte)| {
            (*byte == b'"'
                && (index == 0
                    || rest
                        .as_bytes()
                        .get(index - 1)
                        .is_none_or(|previous| *previous != b'\\')))
            .then_some(index)
        })?;
    Some(rest[..end].to_string())
}
