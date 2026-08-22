use super::helpers::{TMDB_ID_PREFIX, push_unique};
use super::id::{
    base_content_id, episode_id, imdb_id, is_tmdb_like_content_id, normalize_series_lookup_id,
    parse_episode_locator,
};
use serde_json::{Map, Value, json};

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

pub(crate) fn playback_intro_lookup_content_id(id: &str) -> String {
    if let Some(imdb) = imdb_id(id) {
        return imdb;
    }
    base_content_id(id)
        .trim_start_matches(TMDB_ID_PREFIX)
        .to_string()
}

pub(crate) fn playback_stream_request_ids_json(
    content_type: &str,
    id: &str,
    detail_id: Option<&str>,
) -> Option<String> {
    let canonical_base_id = imdb_id(id).or_else(|| detail_id.and_then(imdb_id));
    serde_json::to_string(&stream_request_ids(
        content_type,
        id,
        detail_id,
        detail_id.map(normalize_series_lookup_id).as_deref(),
        canonical_base_id.as_deref(),
    ))
    .ok()
}

pub(crate) fn direct_playback_plan_json(
    meta_json: &str,
    detail_json: Option<&str>,
    today_iso: &str,
) -> Option<String> {
    let meta: Value = serde_json::from_str(meta_json).ok()?;
    let detail: Value = detail_json
        .and_then(|value| serde_json::from_str(value).ok())
        .unwrap_or(Value::Null);
    let has_detail = detail.as_object().is_some_and(|object| !object.is_empty());
    let playback_meta = if has_detail {
        home_playback_meta(&meta, &detail)
    } else {
        meta.clone()
    };
    let target_video_id = string_field(&meta, "lastVideoId")
        .or_else(|| select_direct_playback_video_id(&detail, today_iso));
    let lookup_id = target_video_id
        .clone()
        .or_else(|| string_field(&detail, "id"))
        .or_else(|| string_field(&meta, "id"))
        .unwrap_or_default();

    serde_json::to_string(&json!({
        "meta": playback_meta,
        "targetVideoId": target_video_id,
        "lookupId": lookup_id
    }))
    .ok()
}

fn home_playback_meta(fallback: &Value, detail: &Value) -> Value {
    let mut meta = Map::new();
    for key in [
        "id",
        "name",
        "type",
        "poster",
        "background",
        "logo",
        "description",
        "imdbRating",
        "ageRating",
        "ratings",
        "genres",
        "releaseInfo",
        "released",
        "runtime",
        "seasonsCount",
        "cast",
        "originalLanguage",
    ] {
        insert_detail_or_fallback(&mut meta, key, detail, fallback);
    }
    if meta
        .get("name")
        .and_then(Value::as_str)
        .is_some_and(|name| name.trim().is_empty())
    {
        meta.insert(
            "name".to_string(),
            fallback
                .get("name")
                .cloned()
                .unwrap_or(Value::String(String::new())),
        );
    }
    let episodes_count = detail
        .get("videos")
        .and_then(Value::as_array)
        .map(|videos| json!(videos.len()))
        .unwrap_or_else(|| {
            fallback
                .get("episodesCount")
                .cloned()
                .unwrap_or(Value::Null)
        });
    meta.insert("episodesCount".to_string(), episodes_count);
    for key in [
        "timeOffset",
        "duration",
        "lastVideoId",
        "lastStreamIndex",
        "lastEpisodeName",
        "lastStreamUrl",
        "lastStreamTitle",
        "lastAudioLanguage",
        "lastSubtitleLanguage",
        "awards",
        "rank",
        "reason",
        "homeBadge",
    ] {
        meta.insert(
            key.to_string(),
            fallback.get(key).cloned().unwrap_or(Value::Null),
        );
    }
    Value::Object(meta)
}

fn insert_detail_or_fallback(
    target: &mut Map<String, Value>,
    key: &str,
    detail: &Value,
    fallback: &Value,
) {
    target.insert(
        key.to_string(),
        detail
            .get(key)
            .filter(|value| !value.is_null())
            .cloned()
            .unwrap_or_else(|| fallback.get(key).cloned().unwrap_or(Value::Null)),
    );
}

fn select_direct_playback_video_id(detail: &Value, today_iso: &str) -> Option<String> {
    if detail.get("type").and_then(Value::as_str) != Some("series") {
        return None;
    }
    let mut videos = detail
        .get("videos")
        .and_then(Value::as_array)?
        .iter()
        .collect::<Vec<_>>();
    videos.sort_by_key(|video| {
        (
            number_field(video, "season").unwrap_or(i64::MAX),
            number_field(video, "number")
                .or_else(|| number_field(video, "episode"))
                .unwrap_or(i64::MAX),
        )
    });
    videos
        .iter()
        .find(|video| {
            !string_field(video, "released")
                .as_deref()
                .is_some_and(|released| is_upcoming_iso(released, today_iso))
        })
        .copied()
        .or_else(|| videos.first().copied())
        .and_then(|video| string_field(video, "id"))
}

fn is_upcoming_iso(value: &str, today_iso: &str) -> bool {
    let date = value.trim().get(0..10).unwrap_or("").to_string();
    date.len() == 10 && date.as_str() > today_iso
}

pub(crate) fn stream_discovery_episode_context_json(
    content_type: &str,
    request_id: &str,
    detail_json: Option<&str>,
    season_episodes_json: &str,
) -> Option<String> {
    let season_episodes: Vec<Value> =
        serde_json::from_str(season_episodes_json).unwrap_or_default();
    let detail: Value = detail_json
        .and_then(|value| serde_json::from_str(value).ok())
        .unwrap_or(Value::Null);

    let expected_episode_titles = if content_type == "series" {
        season_episodes
            .iter()
            .find(|episode| string_field(episode, "id").as_deref() == Some(request_id))
            .or_else(|| {
                detail
                    .get("videos")
                    .and_then(Value::as_array)
                    .and_then(|videos| {
                        videos.iter().find(|episode| {
                            string_field(episode, "id").as_deref() == Some(request_id)
                        })
                    })
            })
            .and_then(|episode| string_field(episode, "name"))
            .map(|title| vec![title])
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut season_episode_titles = Map::new();
    let mut season_episode_ids = Map::new();
    if content_type == "series" {
        for episode in &season_episodes {
            let Some(number) = number_field(episode, "number") else {
                continue;
            };
            if let Some(title) = string_field(episode, "name") {
                let key = number.to_string();
                let values = season_episode_titles
                    .entry(key)
                    .or_insert_with(|| Value::Array(Vec::new()));
                if let Some(values) = values.as_array_mut()
                    && !values
                        .iter()
                        .any(|value| value.as_str() == Some(title.as_str()))
                {
                    values.push(Value::String(title));
                }
            }
            if let Some(id) = string_field(episode, "id") {
                season_episode_ids
                    .entry(number.to_string())
                    .or_insert(Value::String(id));
            }
        }
    }

    serde_json::to_string(&serde_json::json!({
        "expectedEpisodeTitles": expected_episode_titles,
        "seasonEpisodeTitles": season_episode_titles,
        "seasonEpisodeIds": season_episode_ids
    }))
    .ok()
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn number_field(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}
