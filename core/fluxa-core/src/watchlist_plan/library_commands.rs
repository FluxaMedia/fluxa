use super::plans::{playback_progress_merge_plan_json, watchlist_toggle_plan_json};
use serde_json::{Value, json};

pub(crate) fn library_apply_mark_watched_json(
    lib_json: &str,
    video_ids_json: &str,
) -> Option<String> {
    use crate::library_state::{
        build_continue_watching_from_progress_json, remember_last_watched_episodes_json,
    };

    let updated_lib_str = remember_last_watched_episodes_json(lib_json, video_ids_json);
    let mut lib: serde_json::Map<String, Value> = serde_json::from_str(&updated_lib_str).ok()?;

    let video_ids: Vec<String> = serde_json::from_str(video_ids_json).unwrap_or_default();
    let watched: std::collections::HashSet<&str> = video_ids.iter().map(String::as_str).collect();

    if let Some(ext_cw) = lib
        .get("externalContinueWatching")
        .and_then(Value::as_array)
        .cloned()
    {
        let filtered: Vec<Value> = ext_cw
            .into_iter()
            .filter(|item| {
                let last_vid = item
                    .get("lastVideoId")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                last_vid.is_empty() || !watched.contains(last_vid)
            })
            .collect();
        lib.insert("externalContinueWatching".into(), filtered.into());
    }

    let progress_map = lib
        .get("progress")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let cleaned: serde_json::Map<String, Value> = progress_map
        .into_iter()
        .filter(|(_, entry)| {
            let last_vid = entry
                .get("lastVideoId")
                .and_then(Value::as_str)
                .unwrap_or("");
            last_vid.is_empty() || !watched.contains(last_vid)
        })
        .collect();

    let progress_json = serde_json::to_string(&cleaned).unwrap_or_else(|_| "{}".to_string());
    let cw = build_continue_watching_from_progress_json(&progress_json)
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .unwrap_or_else(|| Value::Array(Vec::new()));

    lib.insert("progress".into(), Value::Object(cleaned));
    lib.insert("continueWatching".into(), cw);

    serde_json::to_string(&Value::Object(lib)).ok()
}

pub(crate) fn merge_progress_meta_json(
    incoming_meta_json: &str,
    existing_meta_json: &str,
) -> String {
    let incoming: Value = serde_json::from_str(incoming_meta_json).unwrap_or(json!({}));
    let existing: Value = serde_json::from_str(existing_meta_json).unwrap_or(json!({}));

    let is_present = |v: &Value| !v.is_null() && v.as_str() != Some("");

    let pick = |key: &str| -> Value {
        incoming
            .get(key)
            .filter(|v| is_present(v))
            .cloned()
            .or_else(|| existing.get(key).cloned())
            .unwrap_or(Value::Null)
    };

    let mut merged = incoming.clone();
    if let Some(obj) = merged.as_object_mut() {
        obj.insert("poster".into(), pick("poster"));
        obj.insert("background".into(), pick("background"));
        obj.insert("logo".into(), pick("logo"));
    }
    serde_json::to_string(&merged).unwrap_or_else(|_| incoming_meta_json.to_string())
}

pub(crate) fn library_command_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let mut library = request.get("library")?.as_object()?.clone();
    let command = request.get("command")?.as_object()?;
    let command_type = command.get("type")?.as_str()?;
    let now_iso = request.get("nowIso").and_then(Value::as_str).unwrap_or("");
    let mut status_mutation = Value::Null;
    let external_action;

    if command_type == "toggleWatchlist" {
        let item = command.get("item")?.clone();
        let id = item.get("id")?.as_str()?;
        let list = library
            .get("watchlist")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let exists = list
            .iter()
            .any(|entry| entry.get("id").and_then(Value::as_str) == Some(id));
        let toggle_plan: Value = watchlist_toggle_plan_json(
            &json!({"item": item.clone(), "isCurrentlyInWatchlist": exists}).to_string(),
        )
        .and_then(|value| serde_json::from_str(&value).ok())?;
        if toggle_plan.get("command").and_then(Value::as_str) == Some("remove") {
            library.insert(
                "watchlist".into(),
                Value::Array(
                    list.into_iter()
                        .filter(|entry| entry.get("id").and_then(Value::as_str) != Some(id))
                        .collect(),
                ),
            );
            status_mutation = json!({"mediaId": id, "status": null, "item": null});
            external_action = json!({"kind": "watchlist", "command": "remove", "item": item});
        } else {
            let mut next = vec![item.clone()];
            next.extend(list);
            library.insert("watchlist".into(), Value::Array(next));
            for field in ["completed", "dropped"] {
                let filtered = library
                    .get(field)
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|entry| entry.get("id").and_then(Value::as_str) != Some(id))
                    .collect();
                library.insert(field.into(), Value::Array(filtered));
            }
            status_mutation = json!({"mediaId": id, "status": "watchlist", "item": item});
            external_action =
                json!({"kind": "watchlist", "command": "add", "item": command.get("item")});
        }
    } else if command_type == "toggleLibraryStatus" {
        let field = command.get("list")?.as_str()?;
        if !matches!(field, "completed" | "dropped") {
            return None;
        }
        let mut item = command.get("item")?.clone();
        let id = item.get("id")?.as_str()?.to_string();
        let list = library
            .get(field)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let exists = list
            .iter()
            .any(|entry| entry.get("id").and_then(Value::as_str) == Some(&id));
        if exists {
            library.insert(
                field.into(),
                Value::Array(
                    list.into_iter()
                        .filter(|entry| entry.get("id").and_then(Value::as_str) != Some(&id))
                        .collect(),
                ),
            );
            status_mutation = json!({"mediaId": id, "status": null, "item": null});
            external_action =
                json!({"kind": "status", "list": field, "command": "remove", "item": item});
        } else {
            if let Some(object) = item.as_object_mut() {
                object.insert("statusChangedAt".into(), Value::String(now_iso.to_string()));
            }
            let mut next = vec![item.clone()];
            next.extend(list);
            library.insert(field.into(), Value::Array(next));
            for other in [
                "watchlist",
                if field == "completed" {
                    "dropped"
                } else {
                    "completed"
                },
            ] {
                let filtered = library
                    .get(other)
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|entry| entry.get("id").and_then(Value::as_str) != Some(&id))
                    .collect();
                library.insert(other.into(), Value::Array(filtered));
            }
            let mut progress = library
                .get("progress")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            progress.remove(&id);
            let progress_json = Value::Object(progress.clone()).to_string();
            library.insert("progress".into(), Value::Object(progress));
            library.insert(
                "continueWatching".into(),
                crate::library_state::build_continue_watching_from_progress_json(&progress_json)
                    .and_then(|value| serde_json::from_str(&value).ok())
                    .unwrap_or_else(|| json!([])),
            );
            status_mutation = json!({"mediaId": id, "status": field, "item": item});
            external_action = json!({"kind": "status", "list": field, "command": "add", "item": command.get("item")});
        }
    } else if command_type == "markWatched" {
        let ids = command
            .get("videoIds")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let watched_value = command
            .get("watched")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let mut watched = library
            .get("watched")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        for id in &ids {
            if let Some(id) = id.as_str() {
                watched.insert(id.into(), Value::Bool(watched_value));
            }
        }
        library.insert("watched".into(), Value::Object(watched));
        if watched_value {
            let updated = library_apply_mark_watched_json(
                &Value::Object(library.clone()).to_string(),
                &Value::Array(ids.clone()).to_string(),
            )?;
            library = serde_json::from_str::<Value>(&updated)
                .ok()?
                .as_object()?
                .clone();
        }
        let series_id = command
            .get("seriesId")
            .and_then(Value::as_str)
            .unwrap_or("");
        let meta = command
            .get("meta")
            .or_else(|| command.get("item"))
            .cloned()
            .unwrap_or(Value::Null);
        let content_type = meta.get("type").and_then(Value::as_str).unwrap_or("series");
        let episode_infos = command.get("episodes").and_then(Value::as_array).into_iter().flatten().enumerate().filter_map(|(index, episode)| {
            let season = episode.get("season").and_then(Value::as_i64)?;
            let number = episode.get("episode").or_else(|| episode.get("number")).and_then(Value::as_i64)?;
            Some(json!({
                "contentId": series_id,
                "contentType": content_type,
                "videoId": episode.get("id"),
                "season": season,
                "episode": number,
                "title": episode.get("name").or_else(|| episode.get("title")).cloned().unwrap_or_else(|| ids.get(index).cloned().unwrap_or(Value::Null)),
            }))
        }).collect::<Vec<_>>();
        let progress_info = library.get("progress").and_then(|value| value.get(series_id)).and_then(|progress| {
            Some(json!({
                "contentId": series_id,
                "contentType": progress.get("meta").and_then(|value| value.get("type")).and_then(Value::as_str).unwrap_or(content_type),
                "videoId": progress.get("lastVideoId")?,
                "positionSeconds": progress.get("timeOffset").cloned().unwrap_or_else(|| json!(0)),
                "durationSeconds": progress.get("duration").cloned().unwrap_or_else(|| json!(0)),
                "lastWatched": progress.get("savedAt"),
                "season": progress.get("lastEpisodeSeason"),
                "episode": progress.get("lastEpisodeNumber"),
            }))
        });
        external_action = json!({"kind": "watched", "watched": watched_value, "videoIds": ids, "seriesId": series_id, "episodeInfos": episode_infos, "meta": meta, "progressInfo": progress_info});
    } else {
        return None;
    }
    Some(json!({"library": library, "statusMutation": status_mutation, "externalAction": external_action}).to_string())
}

pub(crate) fn playback_progress_write_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let mut library = request.get("library")?.as_object()?.clone();
    let progress = request.get("progress")?.as_object()?.clone();
    let meta = progress.get("meta")?.as_object()?.clone();
    let content_id = meta.get("id")?.as_str()?.to_string();
    let mut progress_map = library
        .get("progress")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let existing = progress_map
        .get(&content_id)
        .cloned()
        .unwrap_or_else(|| json!({}));
    let merge_request = json!({"existing": existing, "incoming": progress});
    let merge_plan: Value = playback_progress_merge_plan_json(&merge_request.to_string())
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or_else(|| json!({}));
    let merged_meta: Value = serde_json::from_str(&merge_progress_meta_json(
        &Value::Object(meta.clone()).to_string(),
        &existing
            .get("meta")
            .cloned()
            .unwrap_or_else(|| json!({}))
            .to_string(),
    ))
    .unwrap_or_else(|_| Value::Object(meta.clone()));
    let mut merged = existing.as_object().cloned().unwrap_or_default();
    for source in [Value::Object(progress.clone()), merge_plan] {
        if let Some(object) = source.as_object() {
            for (key, value) in object {
                merged.insert(key.clone(), value.clone());
            }
        }
    }
    merged.insert("meta".into(), merged_meta);
    merged.insert(
        "savedAt".into(),
        request.get("nowIso").cloned().unwrap_or(Value::Null),
    );
    progress_map.insert(content_id.clone(), Value::Object(merged.clone()));
    let progress_json = Value::Object(progress_map.clone()).to_string();
    library.insert("progress".into(), Value::Object(progress_map));
    library.insert(
        "continueWatching".into(),
        crate::library_state::build_continue_watching_from_progress_json(&progress_json)
            .and_then(|value| serde_json::from_str(&value).ok())
            .unwrap_or_else(|| json!([])),
    );
    if let Some(dismissed) = library
        .get_mut("dismissedContinueWatching")
        .and_then(Value::as_object_mut)
    {
        dismissed.remove(&content_id);
    }
    let duration = merged
        .get("duration")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let external_progress = (duration > 0.0).then(|| {
        json!({
            "contentId": content_id,
            "contentType": meta.get("type").and_then(Value::as_str).unwrap_or("movie"),
            "videoId": merged.get("lastVideoId").and_then(Value::as_str).unwrap_or(&content_id),
            "positionSeconds": merged.get("timeOffset").cloned().unwrap_or_else(|| json!(0)),
            "durationSeconds": duration,
            "lastWatched": request.get("nowMs").cloned().unwrap_or_else(|| json!(0)),
            "season": merged.get("lastEpisodeSeason"),
            "episode": merged.get("lastEpisodeNumber"),
        })
    });
    Some(json!({"library": library, "entry": merged, "contentId": content_id, "externalProgress": external_progress}).to_string())
}
