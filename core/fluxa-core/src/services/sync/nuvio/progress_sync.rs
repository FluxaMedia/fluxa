use super::helpers::{parse, str_field};
use serde_json::{Map, Value, json};
use std::collections::HashSet;

const RESOLVED_HIGH_RATIO: f64 = 0.995;
const RESOLVED_MAX_POSITION_MS: f64 = 1000.0;

pub(crate) fn library_to_watchlist_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let library = args.get("library")?.as_array()?.clone();
    let watchlist: Vec<Value> = library
        .iter()
        .map(|item| {
            let mut out = Map::new();
            out.insert(
                "id".into(),
                item.get("content_id").cloned().unwrap_or(Value::Null),
            );
            out.insert(
                "name".into(),
                item.get("name").cloned().unwrap_or(Value::Null),
            );
            out.insert(
                "type".into(),
                item.get("content_type").cloned().unwrap_or(Value::Null),
            );
            for (dst, src) in [
                ("poster", "poster"),
                ("background", "background"),
                ("description", "description"),
                ("releaseInfo", "release_info"),
                ("imdbRating", "imdb_rating"),
            ] {
                if let Some(v) = item.get(src).filter(|v| !v.is_null()) {
                    out.insert(dst.into(), v.clone());
                }
            }
            if let Some(genres) = item.get("genres").and_then(Value::as_array)
                && !genres.is_empty()
            {
                out.insert("genres".into(), Value::Array(genres.clone()));
            }
            out.insert("inWatchlist".into(), Value::Bool(true));
            Value::Object(out)
        })
        .collect();
    Some(Value::Array(watchlist).to_string())
}

pub(super) fn episode_video<'a>(meta: &'a Value, entry: &Value) -> Option<&'a Value> {
    let video_id = entry.get("video_id").and_then(Value::as_str);
    let season = entry.get("season").and_then(Value::as_i64);
    let episode = entry.get("episode").and_then(Value::as_i64);
    meta.get("videos")?.as_array()?.iter().find(|video| {
        video_id.is_some() && video.get("id").and_then(Value::as_str) == video_id
            || season.is_some()
                && video.get("season").and_then(Value::as_i64) == season
                && video_episode_number(video) == episode
    })
}

pub(crate) fn progress_meta_needs_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let watch_progress = args.get("watchProgress")?.as_array()?.clone();
    let library = args
        .get("library")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let library_by_identity: std::collections::HashMap<String, &Value> = library
        .iter()
        .filter_map(|item| {
            let id = item.get("content_id")?.as_str()?.trim();
            let content_type = item.get("content_type")?.as_str()?.trim();
            (!id.is_empty() && !content_type.is_empty())
                .then(|| (format!("{}:{id}", content_type.to_ascii_lowercase()), item))
        })
        .collect();

    let mut seen = HashSet::new();
    let needs: Vec<Value> = watch_progress
        .iter()
        .filter_map(|e| {
            let content_id = e.get("content_id")?.as_str()?.trim();
            let content_type = e.get("content_type")?.as_str()?.trim();
            if content_id.is_empty() || content_type.is_empty() || !seen.insert((content_id, content_type)) {
                return None;
            }
            let library_item = library_by_identity.get(&format!("{}:{content_id}", content_type.to_ascii_lowercase()));
            let useful_name = library_item
                .and_then(|item| item.get("name"))
                .and_then(Value::as_str)
                .map(str::trim)
                .is_some_and(|name| !name.is_empty() && !name.eq_ignore_ascii_case(content_id));
            let has_artwork = library_item.is_some_and(|item| {
                ["poster", "background"].into_iter().any(|field| {
                    item.get(field).and_then(Value::as_str).is_some_and(|value| !value.trim().is_empty())
                })
            });
            let is_series = matches!(content_type.to_ascii_lowercase().as_str(), "series" | "show" | "tv" | "anime");
            let has_episode = e.get("video_id").and_then(Value::as_str).is_some_and(|value| !value.trim().is_empty())
                || (e.get("season").and_then(Value::as_i64).is_some() && e.get("episode").and_then(Value::as_i64).is_some());
            if useful_name && has_artwork && !(is_series && has_episode) {
                return None;
            }
            let progress_key = e.get("progress_key").and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| match (e.get("season").and_then(Value::as_i64), e.get("episode").and_then(Value::as_i64)) {
                    (Some(season), Some(episode)) => format!("{content_id}_s{season}e{episode}"),
                    _ => content_id.to_string(),
                });
            Some(json!({ "contentId": content_id, "contentType": content_type, "progressKey": progress_key }))
        })
        .collect();
    Some(Value::Array(needs).to_string())
}

fn is_resolved_up_next(position: f64, duration: f64) -> bool {
    if position < RESOLVED_MAX_POSITION_MS {
        return true;
    }
    duration > 0.0 && position / duration >= RESOLVED_HIGH_RATIO
}

fn video_episode_number(video: &Value) -> Option<i64> {
    video
        .get("episode")
        .or_else(|| video.get("number"))
        .and_then(Value::as_i64)
}

/// Finds the next released episode after `(current_season, current_episode)` in
/// an addon's episode list, matching Nuvio's own client-side Up Next resolution.
fn find_next_episode<'a>(
    current_season: i64,
    current_episode: i64,
    videos: &'a [Value],
) -> Option<&'a Value> {
    videos
        .iter()
        .filter(|video| {
            let season = video.get("season").and_then(Value::as_i64).unwrap_or(0);
            let episode = video_episode_number(video).unwrap_or(0);
            season > current_season || (season == current_season && episode > current_episode)
        })
        .min_by_key(|video| {
            (
                video.get("season").and_then(Value::as_i64).unwrap_or(0),
                video_episode_number(video).unwrap_or(0),
            )
        })
}

/// Resolves the live Continue Watching sync feed the same way `progress_entry`
/// resolves the one-time account import: an episode at/above the completion
/// ratio is rolled forward to the next released episode (Nuvio's own client
/// does this before ever surfacing a row, so a finished S1E2 never shows up as
/// an "almost done" resume card, it becomes a progress-less S1E3 Up Next row).
/// Entries still genuinely in progress pass through unchanged; a resolved
/// entry with no next episode in the addon's video list is dropped.
pub(crate) fn resolve_continue_watching_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let progress = args.get("progress")?.as_array()?.clone();
    let addon_metas = args
        .get("addonMetas")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let resolved: Vec<Value> = progress
        .into_iter()
        .filter_map(|entry| {
            let content_id = str_field(&entry, "content_id")?.to_string();
            let position = entry.get("position").and_then(Value::as_f64).unwrap_or(0.0);
            let duration = entry.get("duration").and_then(Value::as_f64).unwrap_or(0.0);
            if !is_resolved_up_next(position, duration) {
                return Some(entry);
            }

            let season = entry.get("season").and_then(Value::as_i64)?;
            let episode = entry.get("episode").and_then(Value::as_i64)?;
            let videos = addon_metas
                .get(&content_id)
                .and_then(|m| m.get("videos"))
                .and_then(Value::as_array)?;
            let next = find_next_episode(season, episode, videos)?;
            let next_season = next.get("season").and_then(Value::as_i64).unwrap_or(season);
            let next_episode = video_episode_number(next).unwrap_or(episode + 1);
            let next_video_id = str_field(next, "id")
                .map(str::to_string)
                .unwrap_or_else(|| format!("{content_id}:{next_season}:{next_episode}"));

            let mut out = entry.clone();
            if let Value::Object(map) = &mut out {
                map.insert("video_id".into(), Value::String(next_video_id));
                map.insert("season".into(), json!(next_season));
                map.insert("episode".into(), json!(next_episode));
                map.insert("position".into(), json!(0));
                map.insert("duration".into(), json!(0));
                map.insert(
                    "progress_key".into(),
                    Value::String(format!("{content_id}_s{next_season}e{next_episode}")),
                );
            }
            Some(out)
        })
        .collect();

    serde_json::to_string(&Value::Array(resolved)).ok()
}
