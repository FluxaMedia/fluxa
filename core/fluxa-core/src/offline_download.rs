use crate::stream_policy::{
    is_torrent_playback_url, stream_effective_filename, stream_playable_url, stream_text,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfflineDownloadPlanRequest {
    meta: Value,
    #[serde(default)]
    video: Option<Value>,
    #[serde(default)]
    video_id: Option<String>,
    stream: Value,
    #[serde(default)]
    subtitle_url: Option<String>,
    download_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OfflineDownloadPlan {
    supported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
    is_torrent: bool,
    playback_url: String,
    base_name: String,
    video_file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle_file_name: Option<String>,
    poster_file_name: String,
    background_file_name: String,
    logo_file_name: String,
    video_id: Option<String>,
    stream_title: Option<String>,
}

pub fn offline_download_plan_json(request_json: &str) -> Option<String> {
    let request: OfflineDownloadPlanRequest = serde_json::from_str(request_json).ok()?;
    serde_json::to_string(&offline_download_plan(request)).ok()
}

fn offline_download_plan(request: OfflineDownloadPlanRequest) -> OfflineDownloadPlan {
    let playback_url = stream_playable_url(&request.stream).unwrap_or_default();
    let effective_filename = stream_effective_filename(&request.stream, Some(&playback_url));
    let support_error = downloadable_error(&playback_url, &request.stream);
    let base_name = offline_base_name(&request.meta, request.video.as_ref(), &request.download_id);
    let extension = file_extension(&playback_url, effective_filename.as_deref());
    let stream_title = raw_stream_display_title(&request.stream, &playback_url);
    let subtitle_extension = request
        .subtitle_url
        .as_deref()
        .map(subtitle_extension)
        .unwrap_or_else(|| "srt".to_string());

    OfflineDownloadPlan {
        supported: support_error.is_none(),
        reason: support_error,
        is_torrent: is_torrent_playback_url(&playback_url),
        playback_url,
        video_file_name: format!("{base_name}-{}.{extension}", request.download_id),
        subtitle_file_name: request
            .subtitle_url
            .as_ref()
            .map(|_| format!("{base_name}-{}.{}", request.download_id, subtitle_extension)),
        poster_file_name: format!("{base_name}-{}-poster.jpg", request.download_id),
        background_file_name: format!("{base_name}-{}-background.jpg", request.download_id),
        logo_file_name: format!("{base_name}-{}-logo.png", request.download_id),
        video_id: request
            .video
            .as_ref()
            .and_then(|video| text(video, "id").map(ToOwned::to_owned))
            .or(request.video_id),
        stream_title,
        base_name,
    }
}

fn downloadable_error(url: &str, stream: &Value) -> Option<&'static str> {
    let is_http = url.starts_with("http://") || url.starts_with("https://");
    let is_torrent = is_torrent_playback_url(url);
    if !is_http && !is_torrent {
        return Some("unsupported_source");
    }
    if is_http {
        let normalized_path = url.split('?').next().unwrap_or(url).to_lowercase();
        if [".srt", ".vtt", ".ass", ".ssa", ".ttml", ".sub"]
            .iter()
            .any(|ext| normalized_path.ends_with(ext))
        {
            return Some("unsupported_source");
        }
        if [".m3u8", ".mpd"]
            .iter()
            .any(|ext| normalized_path.ends_with(ext))
        {
            return Some("unsupported_source");
        }
    }
    let source_text = ["name", "title", "description", "addonName"]
        .iter()
        .filter_map(|key| stream_text(stream, key))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if source_text.contains("opensubtitles")
        || source_text.contains("subtitle")
        || source_text.contains("altyazi")
        || source_text.contains("altyazı")
    {
        return Some("unsupported_source");
    }
    None
}

fn offline_base_name(meta: &Value, video: Option<&Value>, fallback: &str) -> String {
    let mut parts = vec![];
    if let Some(name) = text(meta, "name") {
        parts.push(name.to_string());
    }
    if let Some(season) = video.and_then(|v| number(v, "season")) {
        parts.push(format!("S{season}"));
    }
    if let Some(episode) = video.and_then(|v| number(v, "number")) {
        parts.push(format!("E{episode}"));
    }
    let sanitized = sanitize_file_name(&parts.join(" "));
    if sanitized.is_empty() {
        fallback.to_string()
    } else {
        sanitized
    }
}

fn sanitize_file_name(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut previous_space = false;
    for character in value.chars() {
        let allowed =
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | ' ');
        let next = if allowed { character } else { ' ' };
        if next.is_whitespace() {
            if !previous_space {
                output.push(' ');
            }
            previous_space = true;
        } else {
            output.push(next);
            previous_space = false;
        }
    }
    output.trim().chars().take(80).collect()
}

fn file_extension(url: &str, filename: Option<&str>) -> String {
    if let Some(ext) = filename
        .and_then(extension_part)
        .filter(|ext| !ext.is_empty() && ext.len() <= 5)
    {
        return ext;
    }
    extension_part(url)
        .filter(|ext| (2..=5).contains(&ext.len()))
        .unwrap_or_else(|| "mp4".to_string())
}

fn subtitle_extension(url: &str) -> String {
    let ext = extension_part(url).unwrap_or_default();
    if matches!(ext.as_str(), "srt" | "vtt" | "ass" | "ssa" | "ttml") {
        ext
    } else {
        "srt".to_string()
    }
}

fn extension_part(value: &str) -> Option<String> {
    let path = value.split('?').next().unwrap_or(value);
    path.rsplit('.').next().and_then(|ext| {
        if ext == path {
            None
        } else {
            Some(ext.to_lowercase())
        }
    })
}

fn raw_stream_display_title(stream: &Value, playable_url: &str) -> Option<String> {
    stream_text(stream, "name")
        .or_else(|| stream_text(stream, "title"))
        .or_else(|| stream_text(stream, "description"))
        .or_else(|| stream_text(stream, "addonName"))
        .map(ToOwned::to_owned)
        .or_else(|| {
            if playable_url.is_empty() {
                None
            } else {
                Some(playable_url.to_string())
            }
        })
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

fn number(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plans_downloadable_http_streams() {
        let request = json!({
            "downloadId": "id1",
            "meta": {"id": "tt1", "name": "Movie: Name", "type": "movie"},
            "video": {"id": "tt1:1:2", "season": 1, "number": 2},
            "stream": {"url": "https://cdn.example/video.mkv", "title": "1080p"},
            "subtitleUrl": "https://sub.example/file.vtt"
        });

        let value: Value =
            serde_json::from_str(&offline_download_plan_json(&request.to_string()).unwrap())
                .unwrap();

        assert_eq!(value["supported"], true);
        assert_eq!(value["playbackUrl"], "https://cdn.example/video.mkv");
        assert_eq!(value["videoFileName"], "Movie Name S1 E2-id1.mkv");
        assert_eq!(value["subtitleFileName"], "Movie Name S1 E2-id1.vtt");
        assert_eq!(value["videoId"], "tt1:1:2");
        assert_eq!(value["streamTitle"], "1080p");
    }

    #[test]
    fn rejects_manifests_and_subtitle_sources() {
        for stream in [
            json!({"url": "https://cdn.example/list.m3u8"}),
            json!({"url": "https://cdn.example/file.srt"}),
            json!({"url": "https://cdn.example/video.mp4", "addonName": "OpenSubtitles"}),
        ] {
            let request = json!({
                "downloadId": "id1",
                "meta": {"name": "Movie"},
                "stream": stream
            });
            let value: Value =
                serde_json::from_str(&offline_download_plan_json(&request.to_string()).unwrap())
                    .unwrap();
            assert_eq!(value["supported"], false);
            assert_eq!(value["reason"], "unsupported_source");
        }
    }

    #[test]
    fn accepts_torrent_sourced_streams() {
        for stream in [
            json!({"url": "magnet:?xt=urn:btih:abc"}),
            json!({"infoHash": "abcdef1234567890abcdef1234567890abcdef12"}),
        ] {
            let request = json!({
                "downloadId": "id1",
                "meta": {"name": "Movie"},
                "stream": stream
            });
            let value: Value =
                serde_json::from_str(&offline_download_plan_json(&request.to_string()).unwrap())
                    .unwrap();
            assert_eq!(value["supported"], true);
            assert_eq!(value["isTorrent"], true);
        }
    }
}
