use crate::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BufferTargetsRequest {
    #[serde(default)]
    forward_buffer_seconds: Option<i64>,
    #[serde(default)]
    back_buffer_seconds: Option<i64>,
    #[serde(default)]
    cache_size_mb: Option<i64>,
    #[serde(default)]
    is_torrent: bool,
    #[serde(default)]
    mobile_data_usage: Option<String>,
}

/// Return safe buffer and cache targets for ExoPlayer given preferences and stream type.
pub(crate) fn player_buffer_targets_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<BufferTargetsRequest>(request_json)
        .map_err(|e| CoreError::BadInput {
            context: "player_buffer_targets_json",
            detail: e.to_string(),
        })
        .log_discard()?;
    let mobile_data_usage = request.mobile_data_usage.as_deref().unwrap_or("medium");

    // On mobile data, reduce buffers
    let data_factor: f64 = match mobile_data_usage {
        "low" => 0.5,
        "high" => 1.5,
        _ => 1.0,
    };

    let base_forward_ms =
        request.forward_buffer_seconds.unwrap_or(120).clamp(10, 600) as f64 * 1000.0 * data_factor;
    let base_back_ms = request.back_buffer_seconds.unwrap_or(30).clamp(5, 120) as f64 * 1000.0;

    // Torrent streams need smaller buffers to avoid filling the local proxy
    let (forward_ms, back_ms) = if request.is_torrent {
        (base_forward_ms.min(30_000.0), base_back_ms.min(15_000.0))
    } else {
        (base_forward_ms, base_back_ms)
    };

    const UNLIMITED_CACHE_BYTES: i64 = 64_000 * 1_000_000;
    let cache_bytes = match request.cache_size_mb {
        Some(mb) if mb < 0 => UNLIMITED_CACHE_BYTES,
        Some(mb) => mb.clamp(10, 2000) * 1_000_000,
        None => 100 * 1_000_000,
    };

    serde_json::to_string(&json!({
        "forwardBufferMs": forward_ms as i64,
        "backBufferMs": back_ms as i64,
        "cacheSizeBytes": cache_bytes
    }))
    .ok()
}
