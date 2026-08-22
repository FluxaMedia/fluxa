use super::magnet::normalize_torrent_link;
use super::meta::form_encode;
use super::torrent_files::{
    TorrentFileStat, resolve_torrent_file_index, torrent_fallback_file_indexes,
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TorrentRuntimeRequest {
    link: String,
    title: String,
    requested_file_idx: Option<i32>,
    preferred_filename: Option<String>,
    sources: Vec<String>,
    file_stats: Vec<TorrentFileStat>,
    rejected_index: Option<i32>,
    base_url: String,
    play: bool,
    stat: bool,
    duration_ms: Option<u64>,
}

pub(crate) fn query_encode(value: &str) -> String {
    form_encode(value).replace('+', "%20")
}
pub(crate) fn build_torrent_stream_url(
    base_url: &str,
    link: &str,
    title: &str,
    file_idx: Option<i32>,
    play: bool,
    stat: bool,
    duration_ms: Option<u64>,
) -> String {
    let base = format!("{}/stream/fname", base_url.trim_end_matches('/'));
    let mut query = format!("link={}", query_encode(link));
    if let Some(index) = file_idx {
        query.push_str(&format!("&index={index}"));
    }
    if play {
        query.push_str("&play");
    }
    if stat {
        query.push_str("&stat");
    }
    if let Some(duration_ms) = duration_ms.filter(|duration| *duration > 0) {
        query.push_str(&format!("&durationMs={duration_ms}"));
    }
    query.push_str(&format!("&title={}", query_encode(title)));
    format!("{base}?{query}")
}
pub(crate) fn torrent_runtime_info_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<TorrentRuntimeRequest>(request_json).ok()?;
    let normalized_link = normalize_torrent_link(&request.link, &request.sources);
    let (selected_file_idx, selected_reason) = resolve_torrent_file_index(
        &request.title,
        request.requested_file_idx,
        request.preferred_filename.as_deref(),
        &request.file_stats,
    );
    let fallback_file_indexes =
        torrent_fallback_file_indexes(&request.title, request.rejected_index, &request.file_stats);
    let stream_url = build_torrent_stream_url(
        &request.base_url,
        &normalized_link,
        &request.title,
        selected_file_idx,
        request.play,
        request.stat,
        request.duration_ms,
    );
    serde_json::to_string(&json!({
        "normalizedLink": normalized_link,
        "selectedFileIdx": selected_file_idx,
        "selectedReason": selected_reason,
        "fallbackFileIndexes": fallback_file_indexes,
        "streamUrl": stream_url
    }))
    .ok()
}
pub(crate) fn torrent_buffer_progress(status: &Value) -> i32 {
    let stat = status.get("stat").and_then(Value::as_i64).unwrap_or(0);
    let preload = status.get("preload").and_then(Value::as_i64).unwrap_or(0);
    let loaded_size = status
        .get("loaded_size")
        .or_else(|| status.get("loadedSize"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let preload_size = status
        .get("preload_size")
        .or_else(|| status.get("preloadSize"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let progress = status
        .get("progress")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let value = if stat >= 3 {
        100
    } else if preload > 0 {
        preload as i32
    } else if preload_size > 0 {
        ((loaded_size as f64 / preload_size as f64) * 100.0) as i32
    } else if loaded_size > 0 {
        ((loaded_size as f64 / (512.0 * 1024.0)) * 100.0) as i32
    } else {
        progress as i32
    };
    value.clamp(0, 100)
}
pub(crate) fn torrent_is_playable_enough(status: &Value) -> bool {
    let stat = status.get("stat").and_then(Value::as_i64).unwrap_or(0);
    let preload = status.get("preload").and_then(Value::as_i64).unwrap_or(0);
    let loaded_size = status
        .get("loaded_size")
        .or_else(|| status.get("loadedSize"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let preload_size = status
        .get("preload_size")
        .or_else(|| status.get("preloadSize"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let target = if preload_size > 0 {
        preload_size.min(4 * 1024 * 1024)
    } else {
        512 * 1024
    };
    stat >= 3 || preload >= 100 || loaded_size >= target
}
pub(crate) fn torrent_status_key(status: &Value) -> &'static str {
    match status.get("stat").and_then(Value::as_i64).unwrap_or(0) {
        1 => "player.torrent_status.preloading",
        2 => "player.torrent_status.downloading",
        3 => "player.torrent_status.ready",
        _ => "player.torrent_status.loading_metadata",
    }
}
pub(crate) fn torrent_status_info_json(status_json: &str) -> Option<String> {
    let status = serde_json::from_str::<Value>(status_json).ok()?;
    serde_json::to_string(&json!({
        "bufferProgress": torrent_buffer_progress(&status),
        "isPlayableEnough": torrent_is_playable_enough(&status),
        "statusKey": torrent_status_key(&status)
    }))
    .ok()
}
pub(crate) fn torrent_ready_budget_json() -> String {
    serde_json::json!({
        "firstAttemptMs": 15_000,
        "retryBudgetMs": 45_000,
        "hardLimitMs": 120_000,
        "stallExtensionMs": 20_000,
        "maxPeerRetriesWithAlternatives": 1,
        "maxPeerRetriesSingleSource": 2,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::build_torrent_stream_url;

    #[test]
    fn stream_url_preserves_positive_duration_for_scheduler() {
        let url = build_torrent_stream_url(
            "http://127.0.0.1:8090",
            "magnet:?xt=urn:btih:abc",
            "Example",
            Some(3),
            true,
            false,
            Some(5_400_000),
        );

        assert!(url.contains("durationMs=5400000"), "{url}");
    }

    #[test]
    fn stream_url_omits_missing_or_invalid_duration() {
        let missing = build_torrent_stream_url(
            "http://127.0.0.1:8090",
            "magnet:?xt=urn:btih:abc",
            "Example",
            None,
            true,
            false,
            None,
        );
        let zero = build_torrent_stream_url(
            "http://127.0.0.1:8090",
            "magnet:?xt=urn:btih:abc",
            "Example",
            None,
            true,
            false,
            Some(0),
        );

        assert!(!missing.contains("durationMs="));
        assert!(!zero.contains("durationMs="));
    }
}
