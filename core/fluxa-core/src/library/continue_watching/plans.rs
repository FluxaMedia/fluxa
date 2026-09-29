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

pub(crate) fn resolve_next_after_watched_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let watched = request.get("watchedEpisodes")?.as_array()?;
    let last = watched.iter().max_by_key(|episode| {
        episode.get("season").and_then(Value::as_i64).unwrap_or(1) * 10_000
            + episode
                .get("episode")
                .or_else(|| episode.get("number"))
                .and_then(Value::as_i64)
                .unwrap_or(0)
    })?;
    resolve_next_episode_json(
        &request.get("videos")?.to_string(),
        last.get("season").and_then(Value::as_i64).unwrap_or(1),
        last.get("episode")
            .or_else(|| last.get("number"))
            .and_then(Value::as_i64)
            .unwrap_or(0),
        request.get("nowMs").and_then(Value::as_i64).unwrap_or(0),
        false,
    )
}

pub(crate) fn continue_watching_resume_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let item = request.get("item")?;
    let last_video_id = item
        .get("lastVideoId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty());
    let videos = request
        .get("videos")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let matched = last_video_id.and_then(|id| {
        videos
            .iter()
            .find(|video| video.get("id").and_then(Value::as_str) == Some(id))
    });
    let episode = last_video_id.map(|id| {
        let mut result = matched.cloned().unwrap_or_else(|| json!({"id": id}));
        if let Some(object) = result.as_object_mut() {
            if !object.contains_key("name") || object.get("name").is_some_and(Value::is_null) {
                if let Some(name) = item.get("lastEpisodeName") {
                    object.insert("name".to_string(), name.clone());
                }
            }
            if !object.contains_key("season") || object.get("season").is_some_and(Value::is_null) {
                if let Some(season) = item.get("lastEpisodeSeason") {
                    object.insert("season".to_string(), season.clone());
                }
            }
            if !object.contains_key("episode") || object.get("episode").is_some_and(Value::is_null)
            {
                if let Some(number) = item.get("lastEpisodeNumber") {
                    object.insert("episode".to_string(), number.clone());
                }
            }
            if !object.contains_key("number") || object.get("number").is_some_and(Value::is_null) {
                if let Some(number) = item.get("lastEpisodeNumber") {
                    object.insert("number".to_string(), number.clone());
                }
            }
            if !object.contains_key("thumbnail")
                || object.get("thumbnail").is_some_and(Value::is_null)
            {
                if let Some(thumbnail) = item.get("lastEpisodeThumbnail") {
                    object.insert("thumbnail".to_string(), thumbnail.clone());
                }
            }
        }
        result
    });
    let override_value = request.get("resumeAtOverride");
    let has_override = override_value.is_some() && !override_value.is_some_and(Value::is_null);
    let resume_percent = if has_override {
        None
    } else {
        item.get("resumeProgressPercent")
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite())
            .or_else(|| {
                let offset = item.get("timeOffset").and_then(Value::as_f64)?;
                let duration = item.get("duration").and_then(Value::as_f64)?;
                (duration > 0.0).then_some(offset / duration * 100.0)
            })
    };
    let resume_at = if has_override {
        override_value.and_then(Value::as_f64)
    } else if resume_percent.is_none() {
        item.get("timeOffset").and_then(Value::as_f64)
    } else {
        None
    };
    serde_json::to_string(&json!({
        "episode": episode,
        "resumeAt": resume_at,
        "resumePercent": resume_percent,
        "duration": item.get("duration").and_then(Value::as_f64),
    }))
    .ok()
}

pub(crate) fn next_progress_info_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let next: Value = serde_json::from_str(&resolve_next_after_watched_json(request_json)?).ok()?;
    let content_id = request.get("contentId")?.as_str()?.trim();
    if content_id.is_empty() {
        return None;
    }
    serde_json::to_string(&json!({
        "contentId": content_id,
        "contentType": request.get("contentType").and_then(Value::as_str).unwrap_or("series"),
        "videoId": next.get("id")?,
        "positionSeconds": UP_NEXT_POSITION_SECONDS,
        "durationSeconds": UP_NEXT_DURATION_SECONDS,
        "lastWatched": request.get("nowMs").and_then(Value::as_i64).unwrap_or(0),
        "season": next.get("season"),
        "episode": next.get("episode").or_else(|| next.get("number")),
    }))
    .ok()
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
