use super::helpers::{end_of_current_week_ms, parse_date_ms};
use serde_json::{Value, json};

pub(crate) fn calendar_items_from_meta_json(meta_json: &str, month_prefix: &str) -> Option<String> {
    let meta: Value = serde_json::from_str(meta_json).ok()?;
    let meta_id = meta.get("id").and_then(Value::as_str).unwrap_or("");
    let meta_name = meta.get("name").and_then(Value::as_str).unwrap_or("");
    let meta_poster = meta
        .get("poster")
        .and_then(Value::as_str)
        .or_else(|| meta.get("background").and_then(Value::as_str));
    let videos = meta.get("videos").and_then(Value::as_array)?;
    let mut items: Vec<Value> = Vec::new();
    for video in videos {
        let released = video.get("released").and_then(Value::as_str).unwrap_or("");
        let date_iso = match released.get(..10) {
            Some(d) => d,
            None => continue,
        };
        if !month_prefix.is_empty() && !date_iso.starts_with(month_prefix) {
            continue;
        }
        let season = video.get("season").and_then(Value::as_i64);
        let episode = video
            .get("episode")
            .or_else(|| video.get("number"))
            .and_then(Value::as_i64);
        let episode_code = match (season, episode) {
            (Some(s), Some(e)) => Some(format!("S{s}:E{e}")),
            _ => None,
        };
        let video_name = video
            .get("name")
            .or_else(|| video.get("title"))
            .and_then(Value::as_str);
        let subtitle = [episode_code.as_deref(), video_name]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        let poster = video
            .get("thumbnail")
            .and_then(Value::as_str)
            .or(meta_poster);
        let video_id = video.get("id").and_then(Value::as_str).unwrap_or("");
        let key = format!("{meta_id}:{video_id}:{date_iso}");
        items.push(json!({
            "id": key,
            "title": meta_name,
            "name": video_name.unwrap_or(meta_name),
            "subtitle": subtitle,
            "dateIso": date_iso,
            "poster": poster,
            "contentId": meta_id,
            "seriesId": meta_id,
            "metaType": meta.get("type"),
        }));
    }
    serde_json::to_string(&items).ok()
}

/// Earliest video whose `released` date is strictly in the future, or None
/// if every video is already released, missing a date, or there are no videos.
/// Purely date-based (no current watch position needed) — unlike
/// `library_state::resolve_next_episode_json`, this works for items that
/// were never started.
pub(crate) fn next_unaired_episode_json(videos_json: &str, now_ms: i64) -> Option<String> {
    let videos: Vec<Value> = serde_json::from_str(videos_json).ok()?;
    let mut future: Vec<Value> = videos
        .into_iter()
        .filter(|v| v.get("released").and_then(Value::as_str).is_some())
        .filter(|v| !crate::library_state::is_episode_released(v, now_ms))
        .collect();
    future.sort_by(|a, b| {
        let ar = a.get("released").and_then(Value::as_str).unwrap_or("");
        let br = b.get("released").and_then(Value::as_str).unwrap_or("");
        ar.cmp(br)
    });
    let next = future.into_iter().next()?;
    serde_json::to_string(&next).ok()
}

pub(crate) fn partition_this_week_json(
    items_json: &str,
    now_ms: i64,
    keep_scheduled: bool,
) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let week_end = end_of_current_week_ms(now_ms);

    let mut this_week: Vec<Value> = Vec::new();
    let mut this_week_ids = std::collections::HashSet::new();
    for item in &items {
        if item.get("continueWatchingBadge").and_then(Value::as_str) != Some("scheduledEpisode") {
            continue;
        }
        let Some(released_at) = item.get("newEpisodeReleasedAt").and_then(Value::as_str) else {
            continue;
        };
        let Some(released_ms) = parse_date_ms(released_at) else {
            continue;
        };
        if released_ms <= week_end {
            this_week.push(item.clone());
            if let Some(id) = item.get("id").and_then(Value::as_str) {
                this_week_ids.insert(id.to_string());
            }
        }
    }

    let continue_watching: Vec<Value> = if keep_scheduled {
        items
    } else {
        items
            .into_iter()
            .filter(|m| {
                let id = m.get("id").and_then(Value::as_str).unwrap_or("");
                !this_week_ids.contains(id)
            })
            .collect()
    };

    serde_json::to_string(&json!({ "thisWeek": this_week, "continueWatching": continue_watching }))
        .ok()
}

pub(crate) fn calendar_item_matches_month_json(item_json: &str, month_prefix: &str) -> bool {
    if month_prefix.is_empty() {
        return true;
    }
    serde_json::from_str::<Value>(item_json)
        .ok()
        .and_then(|v| {
            v.get("dateIso")
                .and_then(Value::as_str)
                .map(|d| d.starts_with(month_prefix))
        })
        .unwrap_or(false)
}
