use super::*;
use serde_json::{Value, json};

/// Given a library JSON and a set of just-watched video IDs, update `lastWatchedEpisodes`.
/// Returns the updated library as JSON.
pub(crate) fn remember_last_watched_episodes_json(
    lib_json: &str,
    watched_ids_json: &str,
) -> String {
    let mut lib: Value = serde_json::from_str(lib_json).unwrap_or(json!({}));
    let watched_ids: std::collections::HashSet<String> = serde_json::from_str(watched_ids_json)
        .ok()
        .and_then(|v: Value| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            })
        })
        .unwrap_or_default();
    let progress = lib
        .get("progress")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut last_watched = lib
        .get("lastWatchedEpisodes")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (series_id, raw) in &progress {
        let video_id = raw.get("lastVideoId").and_then(Value::as_str).unwrap_or("");
        if video_id.is_empty() || !watched_ids.contains(video_id) {
            continue;
        }
        let meta = match raw.get("meta") {
            Some(m) if m.get("type").and_then(Value::as_str) == Some("series") => m,
            _ => continue,
        };
        last_watched.insert(series_id.clone(), json!({
            "meta": meta,
            "lastVideoId": video_id,
            "lastEpisodeName": raw.get("lastEpisodeName").cloned().unwrap_or(Value::Null),
            "lastEpisodeSeason": raw.get("lastEpisodeSeason").cloned().unwrap_or(Value::Null),
            "lastEpisodeNumber": raw.get("lastEpisodeNumber").cloned().unwrap_or(Value::Null),
            "lastEpisodeThumbnail": raw.get("lastEpisodeThumbnail").cloned().unwrap_or(Value::Null),
            "watchedAt": chrono::Utc::now().to_rfc3339(),
        }));
    }
    if let Some(obj) = lib.as_object_mut() {
        obj.insert(
            "lastWatchedEpisodes".to_string(),
            Value::Object(last_watched),
        );
    }
    serde_json::to_string(&lib).unwrap_or_else(|_| lib_json.to_string())
}

/// Returns the next episode after (current_season, current_episode).
/// If released_only is true, episodes whose `released` date is in the future
/// (relative to now_ms) are excluded.
pub(crate) fn resolve_next_episode_json(
    videos_json: &str,
    current_season: i64,
    current_episode: i64,
    now_ms: i64,
    released_only: bool,
) -> Option<String> {
    let videos: Vec<Value> = serde_json::from_str(videos_json).ok()?;
    let filtered: Vec<&Value> = videos
        .iter()
        .filter(|v| !released_only || is_episode_released(v, now_ms))
        .collect();
    let next = first_episode_after(
        &filtered.into_iter().cloned().collect::<Vec<_>>(),
        current_season,
        current_episode,
    )?;
    serde_json::to_string(&next).ok()
}

/// Formats a "S1:E2 Episode Name" line from the episode progress fields.
/// Falls back to parsing season/episode from lastVideoId when the explicit
/// season/episode numbers are absent.
pub(crate) fn format_episode_line_json(
    last_episode_name: Option<&str>,
    last_episode_season: Option<i64>,
    last_episode_number: Option<i64>,
    last_video_id: Option<&str>,
) -> String {
    let mut season = last_episode_season;
    let mut episode = last_episode_number;

    if (season.is_none() || episode.is_none())
        && let Some(id) = last_video_id.filter(|id| !id.is_empty())
    {
        let mut parts = id.rsplitn(3, ':');
        if let (Some(episode_part), Some(season_part), Some(_)) =
            (parts.next(), parts.next(), parts.next())
            && let (Ok(s), Ok(e)) = (season_part.parse::<i64>(), episode_part.parse::<i64>())
            && s > 0
            && e > 0
        {
            if season.is_none() {
                season = Some(s);
            }
            if episode.is_none() {
                episode = Some(e);
            }
        }
    }

    let code = match (season, episode) {
        (Some(s), Some(e)) => format!("S{s}:E{e}"),
        _ => String::new(),
    };
    let mut name = last_episode_name.map(str::trim).unwrap_or("").to_string();
    if !code.is_empty() {
        while let Some(rest) = name.strip_prefix(&format!("{code} ")) {
            name = rest.trim_start().to_string();
        }
    }
    [code, name]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
