use crate::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RetryPolicyRequest {
    error_code: String,
    #[serde(default)]
    retry_count: i32,
    #[serde(default)]
    is_torrent: bool,
}

/// Return the retry/fallback policy given an error code and retry history.
pub(crate) fn player_retry_policy_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<RetryPolicyRequest>(request_json)
        .map_err(|e| CoreError::BadInput {
            context: "player_retry_policy_json",
            detail: e.to_string(),
        })
        .log_discard()?;
    let error_code = request.error_code.as_str();
    let retry_count = request.retry_count;

    // Non-retryable errors
    let is_fatal = matches!(
        error_code,
        "no_source"
            | "drm_not_supported"
            | "drm_session_error"
            | "format_unsupported"
            | "missing_profile"
    );

    if is_fatal || retry_count >= 3 {
        return serde_json::to_string(&json!({
            "shouldRetry": false,
            "fallbackAction": if is_fatal { "show_error" } else { "show_error_with_retry_button" },
            "delayMs": 0
        }))
        .ok();
    }

    // Torrent errors get a longer delay
    let (should_retry, delay_ms, fallback_action) = if request.is_torrent {
        match error_code {
            "timeout" | "connection_error" | "buffer_timeout" => {
                (true, 2000u64 * (retry_count as u64 + 1), "retry_stream")
            }
            "torrent_no_file" | "torrent_file_validation_failed" => {
                (true, 1000, "try_fallback_file")
            }
            _ => (false, 0, "show_error"),
        }
    } else {
        match error_code {
            "timeout" | "connection_error" | "io_error" => {
                (true, 1000u64 * (retry_count as u64 + 1), "retry_stream")
            }
            "renderer_error" | "decode_error" => (true, 500, "retry_with_sw_decoder"),
            _ => (false, 0, "show_error"),
        }
    };

    serde_json::to_string(&json!({
        "shouldRetry": should_retry,
        "fallbackAction": fallback_action,
        "delayMs": delay_ms,
        "retryCount": retry_count
    }))
    .ok()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NextRetrySourceRequest {
    current_stream: Value,
    candidates: Vec<Value>,
    attempted_keys: Vec<String>,
    auto_retry: bool,
    force: bool,
    try_binge_group: bool,
    p2p_enabled: bool,
}

#[expect(
    clippy::indexing_slicing,
    reason = "candidate vector is checked non-empty before modulo indexing"
)]
pub(crate) fn next_retry_source_plan_json(request_json: &str) -> Option<String> {
    let request: NextRetrySourceRequest = serde_json::from_str(request_json).ok()?;
    if !request.force && !request.auto_retry {
        return Some(json!({"stream": null, "attemptedKeys": request.attempted_keys}).to_string());
    }
    if request.candidates.len() < 2 {
        return Some(json!({"stream": null, "attemptedKeys": request.attempted_keys}).to_string());
    }
    let current_key = retry_stream_key(&request.current_stream);
    let mut attempted = request.attempted_keys;
    if !current_key.is_empty() && !attempted.contains(&current_key) {
        attempted.push(current_key.clone());
    }
    let start = request
        .candidates
        .iter()
        .position(|candidate| retry_stream_key(candidate) == current_key)
        .unwrap_or(0);
    let binge_group = request
        .try_binge_group
        .then(|| behavior_text(&request.current_stream, "bingeGroup"))
        .flatten();
    for offset in 1..=request.candidates.len() {
        let candidate = &request.candidates[(start + offset) % request.candidates.len()];
        let key = retry_stream_key(candidate);
        if key.is_empty() || attempted.contains(&key) {
            continue;
        }
        if binge_group.is_some() && behavior_text(candidate, "bingeGroup") != binge_group {
            continue;
        }
        if !request.p2p_enabled && stream_is_p2p(candidate) {
            attempted.push(key);
            continue;
        }
        attempted.push(key);
        return Some(json!({"stream": candidate, "attemptedKeys": attempted}).to_string());
    }
    Some(json!({"stream": null, "attemptedKeys": attempted}).to_string())
}

fn retry_stream_key(stream: &Value) -> String {
    ["url", "playableUrl", "infoHash", "fileIdx", "title", "name"]
        .iter()
        .map(|key| stream.get(key).map(value_key_part).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("|")
}

fn with_referer_fallback(
    headers: Option<Value>,
    url: Option<&str>,
    is_torrent: bool,
) -> Option<Value> {
    if is_torrent {
        return headers;
    }
    let url = url?;
    let has_referer = headers
        .as_ref()
        .and_then(Value::as_object)
        .is_some_and(|map| map.keys().any(|key| key.eq_ignore_ascii_case("referer")));
    if has_referer {
        return headers;
    }
    let referer = crate::stream_policy::stream_request_referer(url)?;
    let mut map = headers
        .as_ref()
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    map.insert("Referer".to_string(), json!(referer));
    Some(Value::Object(map))
}

pub(crate) fn stream_shell_plan_json(input: &str) -> Option<String> {
    let stream: Value = serde_json::from_str(input).ok()?;
    let hints = stream.get("behaviorHints");
    let headers = hints
        .and_then(|value| value.get("requestHeaders"))
        .or_else(|| hints.and_then(|value| value.pointer("/proxyHeaders/request")))
        .filter(|value| value.as_object().is_some_and(|map| !map.is_empty()))
        .cloned();
    let source_link = stream
        .get("url")
        .or_else(|| stream.get("infoHash"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty());
    let download_link = stream
        .get("playableUrl")
        .or_else(|| stream.get("url"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty());
    let is_torrent = stream_is_p2p(&stream)
        || stream
            .get("url")
            .and_then(Value::as_str)
            .is_some_and(|url| {
                let lower = url.to_ascii_lowercase();
                lower.starts_with("magnet:")
                    || lower.starts_with("stremio://torrent/")
                    || lower.starts_with("infohash:")
            });
    let headers = with_referer_fallback(headers, download_link.or(source_link), is_torrent);
    serde_json::to_string(&json!({
        "identityKey": retry_stream_key(&stream),
        "isTorrent": is_torrent,
        "requestHeaders": headers,
        "sourceLink": source_link,
        "downloadLink": download_link,
    }))
    .ok()
}

pub(crate) fn order_streams_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let streams = request.get("streams")?.as_array()?;
    let prefs = request.get("prefs")?;
    if prefs
        .get("streamSourceSelectionMode")
        .and_then(Value::as_str)
        != Some("regex")
    {
        return serde_json::to_string(streams).ok();
    }
    let pattern = prefs
        .get("streamSourceRegexPattern")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if pattern.is_empty() {
        return serde_json::to_string(streams).ok();
    }
    let regex = regex::RegexBuilder::new(pattern)
        .case_insensitive(true)
        .build()
        .ok()?;
    let mut ordered = streams.clone();
    ordered.sort_by_key(|stream| {
        let text = [
            "name",
            "title",
            "description",
            "url",
            "playableUrl",
            "infoHash",
        ]
        .iter()
        .filter_map(|key| stream.get(*key).and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join(" ");
        std::cmp::Reverse(regex.is_match(&text))
    });
    serde_json::to_string(&ordered).ok()
}

fn value_key_part(value: &Value) -> String {
    value.as_str().map(str::to_string).unwrap_or_else(|| {
        if value.is_null() {
            String::new()
        } else {
            value.to_string().trim_matches('"').to_string()
        }
    })
}

fn behavior_text<'a>(stream: &'a Value, key: &str) -> Option<&'a str> {
    stream
        .get("behaviorHints")
        .and_then(|hints| hints.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn stream_is_p2p(stream: &Value) -> bool {
    stream.get("isTorrent").and_then(Value::as_bool) == Some(true)
        || stream
            .get("infoHash")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
}
