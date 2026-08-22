use serde_json::{Value, json};
use std::collections::HashMap;

pub(crate) const VIDEO_FILE_EXTENSIONS: [&str; 7] =
    [".mkv", ".mp4", ".avi", ".webm", ".m4v", ".mov", ".ts"];
// Any value the platform sends that isn't "regex"/"first" behaves as "manual" —
// this mirrors the pre-existing catch-all match arm, just moved to the one
// place the raw string gets parsed instead of being re-compared everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SourceSelectionMode {
    Regex,
    First,
    #[default]
    Manual,
}
impl From<&str> for SourceSelectionMode {
    fn from(value: &str) -> Self {
        match value {
            "regex" => SourceSelectionMode::Regex,
            "first" => SourceSelectionMode::First,
            _ => SourceSelectionMode::Manual,
        }
    }
}
pub(crate) fn stream_behavior_text<'a>(stream: &'a Value, key: &str) -> Option<&'a str> {
    stream
        .get("behaviorHints")
        .and_then(|hints| hints.get(key))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
}
pub(crate) fn stream_text<'a>(stream: &'a Value, key: &str) -> Option<&'a str> {
    stream
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
}
pub(crate) fn stream_number(stream: &Value, key: &str) -> Option<i64> {
    stream.get(key).and_then(Value::as_i64).or_else(|| {
        stream
            .get("behaviorHints")
            .and_then(|hints| hints.get(key))
            .and_then(Value::as_i64)
    })
}
pub(crate) fn stream_playable_url(stream: &Value) -> Option<String> {
    if let Some(url) = stream_text(stream, "url") {
        return Some(url.to_string());
    }
    if let Some(yt_id) = stream_text(stream, "ytId") {
        return Some(format!("https://www.youtube.com/watch?v={yt_id}"));
    }
    if let Some(yt_id) = stream_text(stream, "yt_ID") {
        return Some(format!("https://www.youtube.com/watch?v={yt_id}"));
    }
    let info_hash = stream_text(stream, "infoHash")?;
    match stream.get("fileIdx").and_then(Value::as_i64) {
        Some(file_idx) => Some(format!("stremio://torrent/{info_hash}/{file_idx}")),
        None => Some(format!("stremio://torrent/{info_hash}")),
    }
}
pub(crate) fn stream_external_url(stream: &Value) -> Option<String> {
    stream_text(stream, "externalUrl")
        .or_else(|| stream_text(stream, "playerFrameUrl"))
        .map(str::to_string)
}
pub(crate) fn percent_decode_component(value: &str) -> String {
    let mut bytes = Vec::with_capacity(value.len());
    let raw = value.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        // Decode the two hex digits as raw bytes rather than slicing `value` —
        // a `%` next to a multi-byte UTF-8 character can put the slice bound
        // mid-character, which panics; byte-at-a-time reads can't.
        if raw.get(index) == Some(&b'%') && index + 2 < raw.len() {
            let hi = raw
                .get(index + 1)
                .and_then(|byte| (*byte as char).to_digit(16));
            let lo = raw
                .get(index + 2)
                .and_then(|byte| (*byte as char).to_digit(16));
            if let (Some(hi), Some(lo)) = (hi, lo) {
                bytes.push((hi * 16 + lo) as u8);
                index += 3;
                continue;
            }
        }
        if let Some(byte) = raw.get(index) {
            bytes.push(*byte);
        }
        index += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}
pub(crate) fn stream_effective_filename(
    stream: &Value,
    playable_url: Option<&str>,
) -> Option<String> {
    if let Some(filename) = stream_text(stream, "filename") {
        return Some(filename.to_string());
    }
    if let Some(filename) = stream_behavior_text(stream, "filename") {
        return Some(filename.to_string());
    }
    let url = stream_text(stream, "url").or(playable_url)?;
    let path = url
        .split('?')
        .next()
        .unwrap_or(url)
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("");
    if path.is_empty() {
        None
    } else {
        Some(percent_decode_component(path))
    }
}
pub(crate) fn form_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'*') {
            encoded.push(byte as char);
        } else if byte == b' ' {
            encoded.push('+');
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}
pub(crate) fn is_torrent_playback_url(value: &str) -> bool {
    value.starts_with("stremio://torrent/")
        || value.starts_with("magnet:")
        || value.starts_with("infohash:")
}
pub(crate) fn stream_is_likely_player_compatible(
    _stream: &Value,
    playable_url: Option<&str>,
    _effective_filename: Option<&str>,
) -> bool {
    let Some(candidate) = playable_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    let normalized = candidate.to_ascii_lowercase();
    if is_torrent_playback_url(&normalized) {
        return true;
    }
    if !normalized.starts_with("http://") && !normalized.starts_with("https://") {
        return false;
    }
    true
}
pub(crate) fn stream_playback_info_json(stream_json: &str) -> Option<String> {
    let stream = serde_json::from_str::<Value>(stream_json).ok()?;
    let playable_url = stream_playable_url(&stream);
    let effective_video_hash = stream_text(&stream, "videoHash")
        .or_else(|| stream_behavior_text(&stream, "videoHash"))
        .map(str::to_string);
    let effective_video_size =
        stream_number(&stream, "videoSize").or_else(|| stream_number(&stream, "size"));
    let effective_filename = stream_effective_filename(&stream, playable_url.as_deref());
    let subtitle_parts = [
        effective_video_hash
            .as_ref()
            .map(|value| ("videoHash", value.clone())),
        effective_video_size.map(|value| ("videoSize", value.to_string())),
        effective_filename
            .as_ref()
            .map(|value| ("filename", value.clone())),
    ]
    .into_iter()
    .flatten()
    .map(|(key, value)| format!("{}={}", form_encode(key), form_encode(&value)))
    .collect::<Vec<_>>();
    let is_torrent = playable_url
        .as_deref()
        .map(is_torrent_playback_url)
        .unwrap_or(false);
    let is_compatible = stream_is_likely_player_compatible(
        &stream,
        playable_url.as_deref(),
        effective_filename.as_deref(),
    );
    serde_json::to_string(&json!({
        "playableUrl": playable_url,
        "externalUrl": stream_external_url(&stream),
        "effectiveVideoHash": effective_video_hash,
        "effectiveVideoSize": effective_video_size,
        "effectiveFilename": effective_filename,
        "subtitleExtraArgs": subtitle_parts.join("&"),
        "isTorrentPlaybackUrl": is_torrent,
        "isLikelyPlayerCompatible": is_compatible
    }))
    .ok()
}
pub(crate) fn stream_request_headers_json(headers_json: &str) -> Option<String> {
    let headers = serde_json::from_str::<HashMap<String, String>>(headers_json).ok()?;
    let clean = headers
        .into_iter()
        .filter(|(key, value)| !key.trim().is_empty() && !value.trim().is_empty())
        .collect::<HashMap<_, _>>();
    serde_json::to_string(&clean).ok()
}
pub(crate) fn stream_request_referer(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://")?.1;
    let host_end = after_scheme
        .find(['/', '?', '#'])
        .unwrap_or(after_scheme.len());
    let authority = &after_scheme[..host_end];
    if authority.is_empty() {
        return None;
    }
    let scheme = url.split_once("://")?.0;
    if scheme != "http" && scheme != "https" {
        return None;
    }
    Some(format!("{scheme}://{authority}/"))
}
