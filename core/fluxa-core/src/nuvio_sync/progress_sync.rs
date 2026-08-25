use super::helpers::{iso_from_ms, parse, str_field};
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

fn progress_entry(entry: &Value, lib_item: Option<&Value>, addon_meta: Option<&Value>) -> Value {
    let position = entry.get("position").and_then(Value::as_f64).unwrap_or(0.0);
    let duration = entry.get("duration").and_then(Value::as_f64).unwrap_or(0.0);
    let is_resolved_up_next = is_resolved_up_next(position, duration);

    let season = entry.get("season").filter(|v| !v.is_null());
    let episode = entry.get("episode").filter(|v| !v.is_null());
    let num_eq = |a: Option<&Value>, b: Option<&Value>| match (
        a.and_then(Value::as_f64),
        b.and_then(Value::as_f64),
    ) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    };
    let videos = addon_meta
        .and_then(|m| m.get("videos"))
        .and_then(Value::as_array);
    let ep_meta = match (season, episode, videos) {
        (Some(s), Some(e), Some(videos)) => videos
            .iter()
            .find(|v| num_eq(v.get("season"), Some(s)) && num_eq(v.get("episode"), Some(e))),
        _ => None,
    };
    let next_ep_meta = if is_resolved_up_next {
        match (
            season.and_then(Value::as_i64),
            episode.and_then(Value::as_i64),
            videos,
        ) {
            (Some(current_season), Some(current_episode), Some(videos)) => {
                find_next_episode(current_season, current_episode, videos)
            }
            _ => None,
        }
    } else {
        None
    };
    let target_ep_meta = next_ep_meta.or(ep_meta);

    let pick = |field: &str| -> Value {
        lib_item
            .and_then(|i| i.get(field))
            .filter(|v| !v.is_null())
            .or_else(|| {
                addon_meta
                    .and_then(|m| m.get(field))
                    .filter(|v| !v.is_null())
            })
            .cloned()
            .unwrap_or(Value::Null)
    };

    let mut out = Map::new();
    let mut meta = Map::new();
    meta.insert(
        "id".into(),
        entry.get("content_id").cloned().unwrap_or(Value::Null),
    );
    meta.insert(
        "type".into(),
        entry.get("content_type").cloned().unwrap_or(Value::Null),
    );
    meta.insert("name".into(), pick("name"));
    for field in ["poster", "background"] {
        let v = pick(field);
        if !v.is_null() {
            meta.insert(field.into(), v);
        }
    }
    out.insert("meta".into(), Value::Object(meta));
    out.insert(
        "timeOffset".into(),
        json!((position / 1000.0).round() as i64),
    );
    out.insert("duration".into(), json!((duration / 1000.0).round() as i64));
    out.insert(
        "lastVideoId".into(),
        next_ep_meta
            .and_then(|video| video.get("id").or_else(|| video.get("_id")))
            .cloned()
            .or_else(|| entry.get("video_id").cloned())
            .unwrap_or(Value::Null),
    );
    if let Some(s) = target_ep_meta
        .and_then(|video| video.get("season"))
        .or(season)
    {
        out.insert("lastEpisodeSeason".into(), s.clone());
    }
    if let Some(e) = target_ep_meta
        .and_then(|video| video.get("episode").or_else(|| video.get("number")))
        .or(episode)
    {
        out.insert("lastEpisodeNumber".into(), e.clone());
    }
    if let Some(ep) = target_ep_meta {
        if let Some(title) = str_field(ep, "title").or_else(|| str_field(ep, "name")) {
            out.insert("lastEpisodeName".into(), Value::String(title.to_string()));
        }
        if let Some(thumb) = str_field(ep, "thumbnail") {
            out.insert(
                "lastEpisodeThumbnail".into(),
                Value::String(thumb.to_string()),
            );
        }
    }
    if is_resolved_up_next {
        out.insert(
            "continueWatchingBadge".into(),
            Value::String("upNext".into()),
        );
        out.insert("continueWatchingEpisodeResolved".into(), Value::Bool(true));
    }
    let last_watched = entry
        .get("last_watched")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    out.insert("savedAt".into(), Value::String(iso_from_ms(last_watched)));
    out.insert("source".into(), Value::String("nuvio".into()));
    Value::Object(out)
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

pub(crate) fn import_merge_plan_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let categories: Option<Vec<&str>> = args
        .get("categories")
        .filter(|v| !v.is_null())
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect());
    let dry_run = args.get("dryRun").and_then(Value::as_bool).unwrap_or(false);
    let wants = |c: &str| categories.as_ref().is_none_or(|cats| cats.contains(&c));
    let progress_count = args
        .get("watchProgress")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    let watched_count = args
        .get("watchHistory")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    let mut progress = args
        .get("progress")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut watched = args
        .get("watched")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let library = args
        .get("library")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let addon_metas = args
        .get("addonMetas")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut lib_by_id: Map<String, Value> = Map::new();
    for item in library {
        if let Some(id) = str_field(&item, "content_id") {
            lib_by_id.insert(id.to_string(), item.clone());
        }
    }

    let mut active_remote_ids: Vec<String> = Vec::new();
    if let Some(watch_progress) = args.get("watchProgress").and_then(Value::as_array) {
        let mut sorted = watch_progress.clone();
        sorted.sort_by_key(|e| e.get("last_watched").and_then(Value::as_i64).unwrap_or(0));
        for entry in &sorted {
            let Some(content_id) = str_field(entry, "content_id") else {
                continue;
            };
            progress.insert(
                content_id.to_string(),
                progress_entry(
                    entry,
                    lib_by_id.get(content_id),
                    addon_metas.get(content_id),
                ),
            );
            if let Some(video_id) = str_field(entry, "video_id") {
                active_remote_ids.push(video_id.to_string());
            }
            if let (Some(s), Some(e)) = (
                entry.get("season").and_then(Value::as_i64),
                entry.get("episode").and_then(Value::as_i64),
            ) {
                active_remote_ids.push(format!("{content_id}:{s}:{e}"));
            }
        }
    }

    if let Some(watch_history) = args.get("watchHistory").and_then(Value::as_array) {
        for item in watch_history {
            let Some(content_id) = str_field(item, "content_id") else {
                continue;
            };
            if str_field(item, "content_type") == Some("movie") {
                watched.insert(content_id.to_string(), Value::Bool(true));
            } else if let (Some(s), Some(e)) = (
                item.get("season").and_then(Value::as_i64),
                item.get("episode").and_then(Value::as_i64),
            ) {
                watched.insert(format!("{content_id}:{s}:{e}"), Value::Bool(true));
            }
        }
        for id in &active_remote_ids {
            watched.remove(id);
        }
    }

    let is_watched = |watched: &Map<String, Value>, key: &str| {
        watched.get(key).and_then(Value::as_bool).unwrap_or(false)
    };
    let mut to_remove: Vec<String> = Vec::new();
    for (content_id, entry) in &progress {
        let video_watched = str_field(entry, "lastVideoId")
            .map(|id| is_watched(&watched, id))
            .unwrap_or(false);
        let episode_watched = match (
            entry.get("lastEpisodeSeason").and_then(Value::as_i64),
            entry.get("lastEpisodeNumber").and_then(Value::as_i64),
        ) {
            (Some(s), Some(e)) => is_watched(&watched, &format!("{content_id}:{s}:{e}")),
            _ => false,
        };
        if video_watched || episode_watched {
            to_remove.push(content_id.clone());
        }
    }
    for id in to_remove {
        progress.remove(&id);
    }

    let progress_out = if wants("continueWatching") && !dry_run {
        Some(progress)
    } else {
        None
    };
    let watched_out = if wants("watched") && !dry_run {
        Some(watched)
    } else {
        None
    };

    Some(
        json!({
            "progress": progress_out,
            "progressCount": progress_count,
            "watched": watched_out,
            "watchedCount": watched_count,
        })
        .to_string(),
    )
}
