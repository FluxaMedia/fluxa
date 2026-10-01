use super::helpers::push_unique;
use super::id::{
    base_content_id, episode_id, is_tmdb_like_content_id, normalize_series_lookup_id,
    parse_episode_locator,
};
use serde_json::Value;

pub(crate) fn stream_request_ids(
    content_type: &str,
    id: &str,
    detail_id: Option<&str>,
    current_series_lookup_id: Option<&str>,
    canonical_base_id: Option<&str>,
) -> Vec<String> {
    let mut ids = Vec::new();
    if content_type != "series" {
        if is_tmdb_like_content_id(id)
            && let Some(canonical) = canonical_base_id
        {
            push_unique(&mut ids, canonical.to_string());
        }
        push_unique(&mut ids, id.to_string());
        if let Some(detail) = detail_id {
            push_unique(&mut ids, detail.to_string());
        }
        if let Some(canonical) = canonical_base_id {
            push_unique(&mut ids, canonical.to_string());
        }
        return ids;
    }

    let locator = parse_episode_locator(id);
    let normalized_series_id = current_series_lookup_id
        .map(str::to_string)
        .or_else(|| detail_id.map(normalize_series_lookup_id));
    let normalized_detail_base_id = detail_id.map(base_content_id);

    if let Some((_, season, episode)) = locator {
        push_unique(&mut ids, id.to_string());
        if let Some(series_id) = normalized_series_id {
            push_unique(&mut ids, episode_id(&series_id, season, episode));
        }
        if let Some(detail_base_id) = normalized_detail_base_id {
            push_unique(&mut ids, episode_id(&detail_base_id, season, episode));
        }
        push_unique(&mut ids, episode_id(&base_content_id(id), season, episode));
        if let Some(canonical) = canonical_base_id {
            push_unique(&mut ids, episode_id(canonical, season, episode));
        }
    } else {
        push_unique(&mut ids, id.to_string());
        if let Some(series_id) = normalized_series_id {
            push_unique(&mut ids, series_id);
        }
        if let Some(detail) = detail_id {
            push_unique(&mut ids, detail.to_string());
        }
        if let Some(canonical) = canonical_base_id {
            push_unique(&mut ids, canonical.to_string());
        }
    }

    ids
}

pub(crate) fn is_live_channel(meta: &Value) -> bool {
    let hint = |key| {
        meta.get("behaviorHints")
            .and_then(|hints| hints.get(key))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    };
    let default_video = meta
        .pointer("/behaviorHints/defaultVideoId")
        .and_then(Value::as_str)
        .is_some();
    meta.get("type").and_then(Value::as_str) == Some("tv")
        || hint("isLive")
        || (hint("hasScheduledVideos") && default_video)
}

pub(crate) fn split_channel_schedule(mut meta: Value) -> Value {
    if !is_live_channel(&meta) {
        return meta;
    }
    if let Some(object) = meta.as_object_mut()
        && let Some(videos) = object.remove("videos")
    {
        object.insert("schedule".to_string(), videos);
    }
    meta
}
