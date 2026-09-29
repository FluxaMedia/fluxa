use super::*;

pub(super) fn meta_field(meta: Option<&Value>, name: &str) -> Value {
    meta.and_then(|meta| meta.get(name))
        .cloned()
        .unwrap_or(Value::Null)
}

pub(super) fn episode_in(
    meta: Option<&Value>,
    season: Option<i64>,
    episode: Option<i64>,
) -> Option<Value> {
    let (season, episode) = (season?, episode?);
    meta?
        .get("videos")?
        .as_array()?
        .iter()
        .find(|video| {
            video.get("season").and_then(Value::as_i64) == Some(season)
                && video
                    .get("episode")
                    .or_else(|| video.get("number"))
                    .and_then(Value::as_i64)
                    == Some(episode)
        })
        .cloned()
}

pub(super) fn episode_field(video: Option<&Value>, names: &[&str]) -> Value {
    names
        .iter()
        .find_map(|name| {
            video
                .and_then(|video| video.get(*name))
                .filter(|value| value.as_str().is_none_or(|text| !text.trim().is_empty()))
                .filter(|value| !value.is_null())
        })
        .cloned()
        .unwrap_or(Value::Null)
}

pub(super) fn in_progress_item(entry: &Entry, meta: Option<&Value>) -> Value {
    let video = episode_in(meta, entry.season, entry.episode);
    json!({
        "id": entry.content_id,
        "_id": entry.content_id,
        "type": if entry.content_type.is_empty() { Value::String("series".into()) } else { Value::String(entry.content_type.clone()) },
        "name": meta_field(meta, "name"),
        "poster": meta_field(meta, "poster"),
        "background": meta_field(meta, "background"),
        "logo": meta_field(meta, "logo"),
        "timeOffset": (entry.position_ms / 1000.0).ceil() as i64,
        "duration": (entry.duration_ms / 1000.0).floor() as i64,
        "lastVideoId": entry.video_id,
        "lastEpisodeName": episode_field(video.as_ref(), &["name", "title"]),
        "lastEpisodeSeason": entry.season,
        "lastEpisodeNumber": entry.episode,
        "lastEpisodeThumbnail": episode_field(video.as_ref(), &["thumbnail"]),
        "progressFraction": entry.fraction(),
        "continueWatchingBadge": Value::Null,
        "continueWatchingEpisodeResolved": Value::Null,
        "savedAt": iso(entry.last_updated_ms),
        "source": "nuvio",
    })
}

pub(super) fn next_up_item(seed: &Seed, next: &Value, meta: Option<&Value>) -> Value {
    let season = next.get("season").cloned().unwrap_or(Value::Null);
    let episode = next
        .get("episode")
        .or_else(|| next.get("number"))
        .cloned()
        .unwrap_or(Value::Null);
    let video_id = match (season.as_i64(), episode.as_i64()) {
        (Some(season), Some(episode)) => format!("{}:{season}:{episode}", seed.content_id),
        _ => next
            .get("id")
            .or_else(|| next.get("_id"))
            .and_then(Value::as_str)
            .unwrap_or(&seed.content_id)
            .to_string(),
    };
    json!({
        "id": seed.content_id,
        "_id": seed.content_id,
        "type": if seed.content_type.is_empty() { Value::String("series".into()) } else { Value::String(seed.content_type.clone()) },
        "name": meta_field(meta, "name"),
        "poster": meta_field(meta, "poster"),
        "background": meta_field(meta, "background"),
        "logo": meta_field(meta, "logo"),
        "timeOffset": 0,
        "duration": 0,
        "lastVideoId": video_id,
        "lastEpisodeName": next.get("name").or_else(|| next.get("title")).cloned().unwrap_or(Value::Null),
        "lastEpisodeSeason": season,
        "lastEpisodeNumber": episode,
        "lastEpisodeThumbnail": next.get("thumbnail").cloned().unwrap_or(Value::Null),
        "progressFraction": 0.0,
        "continueWatchingBadge": "upNext",
        "continueWatchingEpisodeResolved": true,
        "newEpisodeReleasedAt": next.get("released").cloned().unwrap_or(Value::Null),
        "savedAt": iso(seed.marked_at_ms),
        "source": "nuvio",
    })
}

pub(super) fn iso(ms: i64) -> Value {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|value| Value::String(value.to_rfc3339()))
        .unwrap_or(Value::Null)
}
