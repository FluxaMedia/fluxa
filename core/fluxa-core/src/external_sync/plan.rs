use super::{merge_external_watched_json, merge_external_watchlist_json};
use serde_json::{Map, Value, json};

pub(crate) fn external_sync_response_action(_provider: &str, status_code: i64) -> &'static str {
    if (200..300).contains(&status_code) {
        "stamp_success"
    } else if status_code == 401 {
        "clear_credentials"
    } else {
        "keep_credentials"
    }
}

pub(crate) fn external_sync_refresh_retry_action(status_code: Option<i64>) -> &'static str {
    match status_code {
        Some(code) if (200..300).contains(&code) => "stamp_success",
        Some(401) => "clear_credentials",
        _ => "keep_credentials",
    }
}

pub(crate) fn provider_pagination_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let base_url = args.get("baseUrl")?.as_str()?;
    let limit = args
        .get("limit")
        .and_then(Value::as_i64)
        .filter(|value| *value > 0)?;
    let page = args.get("page").and_then(Value::as_i64).unwrap_or(0);
    let mut items = args
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let page_items = args
        .get("pageItems")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    items.extend(page_items.iter().cloned());
    let page_count = args
        .get("pageCount")
        .and_then(Value::as_i64)
        .filter(|value| *value > 0);
    let response_ok = args
        .get("responseOk")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let done = !response_ok
        || (page > 0 && page_items.is_empty())
        || page >= 100
        || page_count.is_some_and(|count| page >= count)
        || (page > 0 && page_items.len() < limit as usize);
    let next_page = if page <= 0 { 1 } else { page + 1 };
    let separator = if base_url.contains('?') { '&' } else { '?' };
    serde_json::to_string(&json!({
        "items": items,
        "done": done,
        "page": next_page,
        "requestUrl": (!done).then(|| format!("{base_url}{separator}page={next_page}&limit={limit}")),
    })).ok()
}

pub(crate) fn promote_external_progress_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let source = args.get("source")?.as_str()?;
    let mut progress = args.get("progress")?.as_object()?.clone();
    let mut promotions = Vec::new();
    for item in args.get("items")?.as_array()? {
        let id = item.get("id").and_then(Value::as_str).unwrap_or("");
        let video_id = item
            .get("lastVideoId")
            .and_then(Value::as_str)
            .unwrap_or("");
        let duration = item.get("duration").and_then(Value::as_f64).unwrap_or(0.0);
        let offset = item
            .get("timeOffset")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let saved_at = item.get("savedAt").and_then(Value::as_str).unwrap_or("");
        let Some(saved_ms) = chrono::DateTime::parse_from_rfc3339(saved_at)
            .ok()
            .map(|value| value.timestamp_millis())
        else {
            continue;
        };
        if id.is_empty() || video_id.is_empty() || duration <= 0.0 {
            continue;
        }
        let existing_ms = progress
            .get(id)
            .and_then(|value| value.get("savedAt"))
            .and_then(Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp_millis())
            .unwrap_or(0);
        if existing_ms >= saved_ms {
            continue;
        }
        let existing = progress
            .get(id)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let existing_meta = existing
            .get("meta")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let item_fields: Map<String, Value> = item
            .as_object()?
            .iter()
            .filter(|(_, value)| !value.is_null())
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        let mut next = existing;
        next.extend(item_fields.clone());
        let mut merged_meta = existing_meta;
        merged_meta.extend(item_fields);
        next.insert("meta".to_string(), Value::Object(merged_meta));
        next.insert("source".to_string(), Value::String(source.to_string()));
        next.insert("savedAt".to_string(), Value::String(saved_at.to_string()));
        progress.insert(id.to_string(), Value::Object(next));
        let content_type = item.get("type").and_then(Value::as_str).unwrap_or("movie");
        let season = item.get("lastEpisodeSeason").and_then(Value::as_i64);
        let episode_number = item.get("lastEpisodeNumber").and_then(Value::as_i64);
        promotions.push(json!({
            "item": item,
            "externalProgress": {
                "contentId": id,
                "contentType": content_type,
                "videoId": video_id,
                "positionSeconds": offset,
                "durationSeconds": duration,
                "lastWatched": saved_ms,
                "season": season,
                "episode": episode_number,
            },
            "meta": {"id": id, "type": content_type, "name": item.get("name").and_then(Value::as_str).unwrap_or("")},
            "episode": match (season, episode_number) {
                (Some(season), Some(episode)) => json!({"id": video_id, "season": season, "episode": episode, "number": episode}),
                _ => Value::Null,
            },
            "scrobbleTrakt": source != "trakt",
            "scrobbleSimkl": source != "simkl",
        }));
    }
    serde_json::to_string(&json!({"progress": progress, "promotions": promotions})).ok()
}

pub(crate) fn external_provider_action_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let kind = args.get("kind")?.as_str()?;
    if kind == "sync" {
        let provider = args
            .get("provider")
            .and_then(Value::as_str)
            .unwrap_or("trakt")
            .to_ascii_lowercase();
        let supported = matches!(
            provider.as_str(),
            "anilist" | "simkl" | "trakt" | "stremio" | "nuvio"
        );
        return Some(json!({
            "provider": provider,
            "supported": supported,
            "error": (!supported).then(|| format!("Unsupported external sync provider: {provider}")),
        }).to_string());
    }
    let profile = args.get("profile").filter(|value| !value.is_null())?;
    let now_ms = args.get("nowMs").and_then(Value::as_i64).unwrap_or(0);
    let has = |key: &str| {
        profile
            .get(key)
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
    };
    let trakt = has("traktAccessToken")
        && profile
            .get("traktTokenExpiresAt")
            .and_then(Value::as_i64)
            .is_none_or(|expires| now_ms / 1000 <= expires);
    let simkl = has("simklAccessToken");
    let anilist = has("anilistAccessToken");
    let stremio = has("stremioAuthKey");
    let nuvio = has("nuvioAccessToken");
    match kind {
        "markWatched" => {
            let watched = args.get("watched").and_then(Value::as_bool).unwrap_or(true);
            let episode_infos: Vec<Value> = match args.get("episodeInfo") {
                Some(Value::Array(values)) => values.clone(),
                Some(value) if !value.is_null() => vec![value.clone()],
                _ => Vec::new(),
            }
            .into_iter()
            .filter(|info| {
                info.get("contentId")
                    .and_then(Value::as_str)
                    .is_some_and(|id| !id.is_empty())
            })
            .collect();
            let meta = args.get("meta").cloned().unwrap_or(Value::Null);
            let video_ids = args
                .get("videoIds")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let fallback_id = meta
                .get("id")
                .and_then(Value::as_str)
                .or_else(|| video_ids.first().and_then(Value::as_str))
                .unwrap_or("");
            let watched_keys: Vec<Value> = if episode_infos.is_empty() {
                (!fallback_id.is_empty()).then(|| json!({"content_id": fallback_id, "season": Value::Null, "episode": Value::Null})).into_iter().collect()
            } else {
                episode_infos.iter().map(|info| json!({"content_id": info.get("contentId"), "season": info.get("season"), "episode": info.get("episode")})).collect()
            };
            let history_items: Vec<Value> = episode_infos.iter().map(|info| json!({
                "content_id": info.get("contentId"), "content_type": info.get("contentType"), "title": info.get("title").and_then(Value::as_str).unwrap_or(""),
                "season": info.get("season"), "episode": info.get("episode"), "watched_at": now_ms,
            })).collect();
            let progress_entry = args
                .get("progressInfo")
                .filter(|value| {
                    value.get("contentId").is_some()
                        && value.get("videoId").is_some()
                        && value
                            .get("durationSeconds")
                            .and_then(Value::as_f64)
                            .unwrap_or(0.0)
                            > 0.0
                })
                .map(progress_to_nuvio);
            let anime_episode = episode_infos.last().cloned().unwrap_or(Value::Null);
            Some(json!({
                "trakt": trakt, "simkl": simkl, "anilist": anilist && watched, "stremio": stremio, "nuvio": nuvio,
                "animeEpisode": anime_episode, "animeProgressEpisode": args.pointer("/progressInfo/episode").cloned().or_else(|| anime_episode.get("episode").cloned()),
                "episodes": episode_infos, "watchedKeys": watched_keys, "historyItems": history_items, "progressEntry": progress_entry,
            }).to_string())
        }
        "watchlist" => {
            let command = args.get("command").and_then(Value::as_str).unwrap_or("add");
            Some(json!({"trakt": trakt, "simkl": simkl && command == "add", "anilist": anilist, "stremio": stremio, "nuvio": nuvio}).to_string())
        }
        "progress" => {
            let progress = args.get("progress")?;
            let valid = progress
                .get("durationSeconds")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                > 0.0;
            Some(json!({"trakt": trakt && valid, "simkl": simkl && valid, "stremio": stremio && valid, "nuvio": nuvio && valid, "progressEntry": valid.then(|| progress_to_nuvio(progress))}).to_string())
        }
        "status" => Some(json!({"anilist": anilist}).to_string()),
        "favorite" => Some(json!({"trakt": trakt}).to_string()),
        "dropProgress" => {
            let reason = args
                .pointer("/item/reason")
                .and_then(Value::as_str)
                .unwrap_or("");
            Some(
                json!({
                    "dropTrakt": reason.eq_ignore_ascii_case("trakt"),
                    "dropSimkl": reason.eq_ignore_ascii_case("simkl"),
                })
                .to_string(),
            )
        }
        _ => None,
    }
}

fn progress_to_nuvio(progress: &Value) -> Value {
    json!({
        "content_id": progress.get("contentId"), "content_type": progress.get("contentType"), "video_id": progress.get("videoId"),
        "position": (progress.get("positionSeconds").and_then(Value::as_f64).unwrap_or(0.0) * 1000.0).round() as i64,
        "duration": (progress.get("durationSeconds").and_then(Value::as_f64).unwrap_or(0.0) * 1000.0).round() as i64,
        "last_watched": progress.get("lastWatched"), "season": progress.get("season"), "episode": progress.get("episode")
    })
}

/// Decides what an "import" should actually apply for a provider that follows the
/// common watchlist+watched(+continueWatching) shape (Trakt, Simkl, Stremio): given
/// already-fetched provider data, local before-state, the selected import
/// categories, and whether this is a dry run, returns per-category counts (always)
/// and merged results (only when that category was selected and this isn't a dry
/// run) — so host platforms don't need to re-derive "should I apply this category"
/// themselves beyond passing the user's selection through.
pub(crate) fn import_apply_plan_json(request_json: &str) -> Option<String> {
    let req: Value = serde_json::from_str(request_json).ok()?;
    let categories: Option<Vec<&str>> = req
        .get("categories")
        .filter(|v| !v.is_null())
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect());
    let dry_run = req.get("dryRun").and_then(Value::as_bool).unwrap_or(false);
    let wants = |c: &str| categories.as_ref().is_none_or(|cats| cats.contains(&c));

    let local_watchlist = req.get("localWatchlist").cloned().unwrap_or(json!([]));
    let external_watchlist = req.get("externalWatchlist").cloned().unwrap_or(json!([]));
    let watchlist_count = external_watchlist.as_array().map(|a| a.len()).unwrap_or(0);
    let watchlist_merged = if wants("watchlist") && !dry_run {
        serde_json::from_str::<Value>(&merge_external_watchlist_json(
            &local_watchlist.to_string(),
            &external_watchlist.to_string(),
        ))
        .ok()
    } else {
        None
    };

    let local_watched = req.get("localWatched").cloned().unwrap_or(json!({}));
    let external_watched = req.get("externalWatched").cloned().unwrap_or(json!({}));
    let watched_count = external_watched
        .as_object()
        .map(|m| m.values().filter(|v| v.as_bool() == Some(true)).count())
        .unwrap_or(0);
    let watched_merged = if wants("watched") && !dry_run {
        serde_json::from_str::<Value>(&merge_external_watched_json(
            &local_watched.to_string(),
            &external_watched.to_string(),
        ))
        .ok()
    } else {
        None
    };

    let continue_watching_apply = wants("continueWatching") && !dry_run;

    Some(
        json!({
            "watchlist": watchlist_merged,
            "watchlistCount": watchlist_count,
            "watched": watched_merged,
            "watchedCount": watched_count,
            "continueWatchingApply": continue_watching_apply,
        })
        .to_string(),
    )
}

fn item_str<'a>(item: &'a Value, key: &str) -> Option<&'a str> {
    item.get(key).and_then(Value::as_str)
}

/// Given a destination provider, the requested import categories, and the local
/// library snapshot for those categories, decides which push operations are needed
/// and in what shape — so every platform's host code (TS, Kotlin, ...) can dispatch
/// to its own existing per-provider push functions without re-deriving this
/// provider/category capability matrix itself. Only fields relevant to the chosen
/// destination + categories are populated; the rest are omitted.
pub(crate) fn push_plan_json(request_json: &str) -> Option<String> {
    let req: Value = serde_json::from_str(request_json).ok()?;
    let destination = req.get("destination")?.as_str()?;
    let categories: Vec<&str> = req
        .get("categories")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let wants = |c: &str| categories.contains(&c);

    let watchlist = req
        .get("watchlist")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let completed = req
        .get("completed")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let dropped = req
        .get("dropped")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let continue_watching = req
        .get("continueWatching")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let now_sec = req.get("nowSec").and_then(Value::as_i64).unwrap_or(0);

    let mut plan = Map::new();

    if wants("watchlist") {
        match destination {
            "trakt" | "simkl" | "anilist" | "stremio" => {
                let items: Vec<Value> = watchlist
                    .iter()
                    .filter_map(|item| {
                        let id = item_str(item, "id")?;
                        Some(json!({ "id": id, "contentType": item_str(item, "type").unwrap_or("movie") }))
                    })
                    .collect();
                plan.insert("watchlistItems".into(), json!(items));
            }
            "nuvio" => {
                let items: Vec<Value> = watchlist
                    .iter()
                    .filter_map(|item| {
                        let id = item_str(item, "id")?;
                        Some(json!({
                            "contentId": id,
                            "contentType": item_str(item, "type").unwrap_or("movie"),
                            "name": item.get("name"),
                            "poster": item.get("poster"),
                            "background": item.get("background"),
                        }))
                    })
                    .collect();
                plan.insert("watchlistNuvioItems".into(), json!(items));
            }
            _ => {}
        }
    }

    if wants("watched") {
        let all_watched: Vec<&Value> = completed.iter().chain(dropped.iter()).collect();
        match destination {
            "trakt" | "simkl" => {
                let video_ids: Vec<&str> = all_watched
                    .iter()
                    .filter_map(|item| {
                        item_str(item, "lastVideoId").or_else(|| item_str(item, "id"))
                    })
                    .collect();
                plan.insert("watchedVideoIds".into(), json!(video_ids));
            }
            "anilist" => {
                let mut items: Vec<Value> = Vec::new();
                for item in &completed {
                    if let Some(id) = item_str(item, "id") {
                        items.push(json!({ "id": id, "status": "completed" }));
                    }
                }
                for item in &dropped {
                    if let Some(id) = item_str(item, "id") {
                        items.push(json!({ "id": id, "status": "dropped" }));
                    }
                }
                plan.insert("watchedStatusItems".into(), json!(items));
            }
            "stremio" => {
                let ids: Vec<&str> = all_watched
                    .iter()
                    .filter_map(|item| item_str(item, "id"))
                    .collect();
                plan.insert("watchedItemIds".into(), json!(ids));
            }
            "nuvio" => {
                let items: Vec<Value> = all_watched
                    .iter()
                    .filter_map(|item| {
                        let id = item_str(item, "id")?;
                        Some(json!({
                            "contentId": id,
                            "contentType": item_str(item, "type").unwrap_or("movie"),
                            "title": item.get("name"),
                            "season": item.get("lastEpisodeSeason"),
                            "episode": item.get("lastEpisodeNumber"),
                            "watchedAt": now_sec,
                        }))
                    })
                    .collect();
                plan.insert("watchedNuvioItems".into(), json!(items));
            }
            _ => {}
        }
    }

    if wants("continueWatching") {
        match destination {
            "trakt" | "simkl" | "anilist" | "stremio" => {
                let ids: Vec<&str> = continue_watching
                    .iter()
                    .filter(|item| {
                        item.get("duration").and_then(Value::as_f64).unwrap_or(0.0) > 0.0
                    })
                    .filter_map(|item| item_str(item, "id"))
                    .collect();
                plan.insert("progressItemIds".into(), json!(ids));
            }
            "nuvio" => {
                let entries: Vec<Value> = continue_watching
                    .iter()
                    .filter(|item| {
                        item.get("duration").and_then(Value::as_f64).unwrap_or(0.0) > 0.0
                    })
                    .filter_map(|item| {
                        let id = item_str(item, "id")?;
                        let video_id = item_str(item, "lastVideoId")?;
                        Some(json!({
                            "contentId": id,
                            "contentType": item_str(item, "type").unwrap_or("movie"),
                            "videoId": video_id,
                            "position": item.get("timeOffset").cloned().unwrap_or(json!(0)),
                            "duration": item.get("duration").cloned().unwrap_or(json!(0)),
                            "lastWatched": now_sec,
                            "season": item.get("lastEpisodeSeason"),
                            "episode": item.get("lastEpisodeNumber"),
                        }))
                    })
                    .collect();
                plan.insert("progressNuvioEntries".into(), json!(entries));
            }
            _ => {}
        }
    }

    Some(Value::Object(plan).to_string())
}
