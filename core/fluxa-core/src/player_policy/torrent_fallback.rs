use super::dolby_vision::episode_path_matches_id;
use crate::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TorrentFallbackRequest {
    #[serde(default)]
    file_stats: Vec<Value>,
    #[serde(default)]
    rejected_index: Option<i32>,
    #[serde(default)]
    video_id: Option<String>,
}

pub(crate) fn torrent_fallback_file_policy_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<TorrentFallbackRequest>(request_json)
        .map_err(|e| CoreError::BadInput {
            context: "torrent_fallback_file_policy_json",
            detail: e.to_string(),
        })
        .log_discard()?;
    let rejected = request.rejected_index;
    let video_id = request.video_id.as_deref().unwrap_or("");

    // Collect video-likely files (by extension)
    let video_exts = [
        ".mkv", ".mp4", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v",
    ];
    let mut candidates: Vec<(i32, i64)> = request
        .file_stats
        .iter()
        .filter_map(|stat| {
            let id = stat.get("id").and_then(Value::as_i64)? as i32;
            if rejected == Some(id) {
                return None;
            }
            let path = stat
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_lowercase();
            let is_video = video_exts.iter().any(|ext| path.ends_with(ext));
            if !is_video {
                return None;
            }
            let length = stat.get("length").and_then(Value::as_i64).unwrap_or(0);
            // Skip tiny files (less than 1MB) unless it's the only candidate
            if length < 1_000_000 && request.file_stats.len() > 1 {
                return None;
            }
            Some((id, length))
        })
        .collect();

    // Sort by size descending (largest first as most likely the right video file)
    candidates.sort_by(|a, b| b.1.cmp(&a.1));

    // If we have a video_id hint, try to match by episode pattern
    let fallback_ids: Vec<i32> = if !video_id.is_empty() {
        // Episode-matched first, then size-sorted remainder
        let mut matched: Vec<(i32, i64)> = Vec::new();
        let mut unmatched: Vec<(i32, i64)> = Vec::new();
        for (id, length) in &candidates {
            let path = request
                .file_stats
                .iter()
                .find(|s| s.get("id").and_then(Value::as_i64) == Some(*id as i64))
                .and_then(|s| s.get("path").and_then(Value::as_str))
                .unwrap_or("");
            if episode_path_matches_id(path, video_id) {
                matched.push((*id, *length));
            } else {
                unmatched.push((*id, *length));
            }
        }
        matched
            .iter()
            .chain(unmatched.iter())
            .map(|(id, _)| *id)
            .collect()
    } else {
        candidates.iter().map(|(id, _)| *id).collect()
    };

    serde_json::to_string(&json!({
        "fallbackFileIndexes": fallback_ids,
        "rejectedIndex": rejected
    }))
    .ok()
}
