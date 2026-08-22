use super::meta::{form_encode, is_torrent_playback_url, stream_playable_url, stream_text};
use serde_json::Value;

pub(crate) fn is_bare_info_hash(value: &str) -> bool {
    let length = value.len();
    matches!(length, 32 | 40 | 64) && value.chars().all(|ch| ch.is_ascii_hexdigit())
}
pub(crate) fn normalize_torrent_link(link: &str, sources: &[String]) -> String {
    let trimmed = link.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("stremio://torrent/") {
        let rest = &trimmed["stremio://torrent/".len()..];
        let hash = rest.split('/').next().unwrap_or("").trim();
        if hash.is_empty() {
            return trimmed.to_string();
        }
        return build_magnet(hash, sources);
    }
    if lower.starts_with("infohash:") {
        return build_magnet(
            trimmed
                .split_once(':')
                .map(|(_, value)| value)
                .unwrap_or(""),
            sources,
        );
    }
    if is_bare_info_hash(trimmed) {
        return build_magnet(trimmed, sources);
    }
    trimmed.to_string()
}
// Popular fallback trackers always added to magnets so a bare info_hash (no
// addon-provided sources) doesn't have to round-trip DHT for peer discovery.
// Kept short — duplicates from `sources` are filtered out below.
const FALLBACK_TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://exodus.desync.com:6969/announce",
    "udp://open.demonii.com:1337/announce",
];
pub(crate) fn build_magnet(hash: &str, sources: &[String]) -> String {
    let mut trackers = Vec::new();
    for source in sources {
        let tracker = source.strip_prefix("tracker:").unwrap_or(source).trim();
        if (tracker.starts_with("udp://")
            || tracker.starts_with("http://")
            || tracker.starts_with("https://"))
            && !trackers.contains(&tracker.to_string())
        {
            trackers.push(tracker.to_string());
        }
    }
    for fallback in FALLBACK_TRACKERS {
        let tracker = fallback.to_string();
        if !trackers.contains(&tracker) {
            trackers.push(tracker);
        }
    }
    let tracker_query = trackers
        .iter()
        .map(|tracker| format!("&tr={}", form_encode(tracker)))
        .collect::<String>();
    format!(
        "magnet:?xt=urn:btih:{}{}",
        hash.to_ascii_lowercase(),
        tracker_query
    )
}
pub(crate) fn stream_magnet_link(stream: &Value) -> Option<String> {
    let sources = stream
        .get("sources")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(hash) = stream_text(stream, "infoHash") {
        return Some(build_magnet(hash, &sources));
    }
    let playable = stream_playable_url(stream)?;
    if playable.to_ascii_lowercase().starts_with("magnet:") {
        return Some(playable);
    }
    if is_torrent_playback_url(&playable) {
        return Some(normalize_torrent_link(&playable, &sources));
    }
    None
}
pub(crate) fn stream_magnet_link_json(stream_json: &str) -> Option<String> {
    let stream: Value = serde_json::from_str(stream_json).ok()?;
    stream_magnet_link(&stream)
}
