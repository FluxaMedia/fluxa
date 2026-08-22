use super::helpers::usable_artwork;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseRowsRequest {
    meta: Value,
    #[serde(default)]
    detail: Value,
    #[serde(default)]
    videos: Vec<Value>,
    month_prefix: String,
    movie_label: String,
}

pub(crate) fn calendar_release_rows_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<ReleaseRowsRequest>(request_json).ok()?;
    let meta = &request.meta;
    let content_type = meta
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let meta_id = meta.get("id").and_then(Value::as_str).unwrap_or("");
    let title = meta.get("name").and_then(Value::as_str).unwrap_or(meta_id);
    if ["movie", "film"].contains(&content_type.as_str()) {
        let date_iso = meta
            .get("released")
            .and_then(Value::as_str)
            .and_then(|value| value.get(..10))?;
        if !date_iso.starts_with(&request.month_prefix) {
            return serde_json::to_string(&json!([])).ok();
        }
        let poster = usable_artwork(meta.get("poster").and_then(Value::as_str));
        return serde_json::to_string(&json!([{
            "dateIso": date_iso,
            "meta": meta,
            "title": title,
            "subtitle": request.movie_label,
            "poster": poster
        }]))
        .ok();
    }
    if !["series", "tv", "show", "anime"].contains(&content_type.as_str()) {
        return serde_json::to_string(&json!([])).ok();
    }
    let detail = &request.detail;
    let fallback_artwork = [
        meta.get("poster").and_then(Value::as_str),
        detail.get("poster").and_then(Value::as_str),
        meta.get("continueWatchingPoster").and_then(Value::as_str),
        meta.get("background").and_then(Value::as_str),
        meta.get("continueWatchingBackground")
            .and_then(Value::as_str),
        detail.get("background").and_then(Value::as_str),
    ]
    .into_iter()
    .flatten()
    .find_map(|url| usable_artwork(Some(url)));
    let rows: Vec<Value> = request
        .videos
        .iter()
        .filter_map(|video| {
            let date_iso = video
                .get("released")
                .and_then(Value::as_str)
                .and_then(|value| value.get(..10))?;
            if !date_iso.starts_with(&request.month_prefix) {
                return None;
            }
            let season = video.get("season").and_then(Value::as_i64);
            let episode = video
                .get("number")
                .or_else(|| video.get("episode"))
                .and_then(Value::as_i64);
            let episode_code = match (season, episode) {
                (Some(s), Some(e)) => Some(format!("S{s}:E{e}")),
                (Some(s), None) => Some(format!("S{s}")),
                (None, Some(e)) => Some(format!("E{e}")),
                _ => None,
            };
            let episode_title = video
                .get("name")
                .or_else(|| video.get("title"))
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty());
            let subtitle = [episode_code.as_deref(), episode_title]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ");
            let episode_thumbnail = video
                .get("thumbnail")
                .and_then(Value::as_str)
                .and_then(|url| usable_artwork(Some(url)));
            let episode_poster = episode_thumbnail
                .clone()
                .or_else(|| fallback_artwork.clone());
            Some(json!({
                "dateIso": date_iso,
                "meta": meta,
                "title": title,
                "subtitle": subtitle,
                "poster": fallback_artwork,
                "episodePoster": episode_poster,
                "seasonNumber": season,
                "episodeNumber": episode,
                "episodeTitle": episode_title
            }))
        })
        .collect();
    serde_json::to_string(&rows).ok()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeasonCandidatesRequest {
    #[serde(default)]
    seasons_count: Option<i32>,
    #[serde(default)]
    last_video_id: Option<String>,
}

pub(crate) fn calendar_season_candidates_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<SeasonCandidatesRequest>(request_json).ok()?;
    let seasons_count = request.seasons_count.unwrap_or(1).max(1);
    let watched_season = request
        .last_video_id
        .as_deref()
        .and_then(|id| id.split(':').nth(1))
        .and_then(|s| s.parse::<i32>().ok());
    let focused: Vec<i32> = [
        watched_season,
        watched_season.map(|s| s + 1),
        Some(seasons_count),
    ]
    .into_iter()
    .flatten()
    .filter(|&s| s > 0 && s <= seasons_count)
    .collect::<std::collections::BTreeSet<_>>()
    .into_iter()
    .collect();
    let full: Vec<i32> = if seasons_count <= 8 {
        (1..=seasons_count).collect()
    } else {
        focused.clone()
    };
    let mut result: Vec<i32> = focused
        .into_iter()
        .chain(full)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    result.truncate(12);
    serde_json::to_string(&result).ok()
}
