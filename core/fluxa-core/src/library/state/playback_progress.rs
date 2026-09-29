use super::continue_watching::build_continue_watching_from_progress_json;
use super::helpers::text;
use serde_json::{Value, json};

fn library_item_from_meta(meta: &Value, state: Value, last_watched: Option<&str>) -> Value {
    let mut item = json!({
        "_id": text(meta, "id").unwrap_or(""),
        "name": text(meta, "name").unwrap_or(""),
        "type": text(meta, "type").unwrap_or(""),
        "poster": meta.get("poster").cloned().unwrap_or(Value::Null),
        "background": meta.get("background").cloned().unwrap_or(Value::Null),
        "logo": meta.get("logo").cloned().unwrap_or(Value::Null),
        "state": state
    });
    if let Some(last_watched) = last_watched
        && let Some(fields) = item.as_object_mut()
    {
        fields.insert(
            "lastWatched".to_string(),
            Value::String(last_watched.to_string()),
        );
    }
    item
}

pub(crate) fn playback_progress_item_json(
    meta_json: &str,
    time_offset: i64,
    duration: i64,
    now_utc: &str,
) -> Option<String> {
    let meta: Value = serde_json::from_str(meta_json).ok()?;
    let item = library_item_from_meta(
        &meta,
        json!({
            "lastWatched": now_utc,
            "timeOffset": time_offset,
            "duration": duration
        }),
        None,
    );
    serde_json::to_string(&item).ok()
}

pub(crate) fn clear_playback_progress_item_json(meta_json: &str) -> Option<String> {
    let meta: Value = serde_json::from_str(meta_json).ok()?;
    let item = library_item_from_meta(
        &meta,
        json!({
            "lastWatched": Value::Null,
            "timeOffset": 0,
            "duration": 0,
            "videoId": Value::Null,
            "timesWatched": 0,
            "flaggedWatched": 0
        }),
        None,
    );
    serde_json::to_string(&item).ok()
}
pub(crate) fn clear_playback_progress_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let mut library = args.get("library")?.clone();
    let meta = args.get("meta")?;
    let id = text(meta, "id")?.to_string();
    let preserve_last_watched = args
        .get("preserveLastWatched")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let drop_continue_watching = args
        .get("dropContinueWatching")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let now_iso = args.get("nowIso").and_then(Value::as_str).unwrap_or("");
    let document = library.as_object_mut()?;
    let progress = document
        .entry("progress")
        .or_insert_with(|| json!({}))
        .as_object_mut()?;
    progress.remove(&id);
    let progress_json = serde_json::to_string(progress).ok()?;
    document.insert(
        "continueWatching".to_string(),
        serde_json::from_str(&build_continue_watching_from_progress_json(&progress_json)?).ok()?,
    );
    let mut removed_external = false;
    let mut dropped_external = Value::Null;
    if let Some(external) = document
        .entry("externalContinueWatching")
        .or_insert_with(|| json!([]))
        .as_array_mut()
    {
        let before = external.len();
        dropped_external = external
            .iter()
            .find(|item| text(item, "id") == Some(&id))
            .cloned()
            .unwrap_or(Value::Null);
        external.retain(|item| text(item, "id") != Some(&id));
        removed_external = external.len() != before;
    }
    if drop_continue_watching {
        document
            .entry("dismissedContinueWatching")
            .or_insert_with(|| json!({}))
            .as_object_mut()?
            .insert(id.clone(), Value::String(now_iso.to_string()));
    }
    let mut last_watched_entry = Value::Null;
    if preserve_last_watched
        && meta
            .get("lastVideoId")
            .is_some_and(|value| !value.is_null())
    {
        last_watched_entry = json!({
            "meta": {
                "id": id,
                "type": meta.get("type").cloned().unwrap_or_else(|| json!("series")),
                "name": meta.get("name").cloned().unwrap_or(Value::Null),
                "poster": meta.get("poster").cloned().unwrap_or(Value::Null),
                "background": meta.get("background").cloned().unwrap_or(Value::Null),
            },
            "lastVideoId": meta.get("lastVideoId").cloned().unwrap_or(Value::Null),
            "lastEpisodeSeason": meta.get("lastEpisodeSeason").cloned().unwrap_or(Value::Null),
            "lastEpisodeNumber": meta.get("lastEpisodeNumber").cloned().unwrap_or(Value::Null),
            "lastEpisodeName": meta.get("lastEpisodeName").cloned().unwrap_or(Value::Null),
            "lastEpisodeThumbnail": meta.get("lastEpisodeThumbnail").cloned().unwrap_or(Value::Null),
            "savedAt": now_iso,
        });
        document
            .entry("lastWatchedEpisodes")
            .or_insert_with(|| json!({}))
            .as_object_mut()?
            .insert(id.clone(), last_watched_entry.clone());
    } else if !preserve_last_watched {
        document
            .entry("lastWatchedEpisodes")
            .or_insert_with(|| json!({}))
            .as_object_mut()?
            .remove(&id);
    }
    serde_json::to_string(&json!({
        "library": library,
        "contentId": id,
        "lastWatchedEntry": last_watched_entry,
        "removedExternalContinueWatching": removed_external,
        "droppedExternalContinueWatching": dropped_external,
    }))
    .ok()
}

pub(crate) fn watched_state_items_json(
    meta_json: &str,
    episodes_json: &str,
    watched: bool,
    watched_at: Option<&str>,
) -> Option<String> {
    let meta: Value = serde_json::from_str(meta_json).ok()?;
    let episodes: Vec<Value> = serde_json::from_str(episodes_json).unwrap_or_default();
    let watched_value = if watched { 1 } else { 0 };
    let watched_at_value = watched_at
        .map(|value| Value::String(value.to_string()))
        .unwrap_or(Value::Null);
    let items = if text(&meta, "type") == Some("series") && !episodes.is_empty() {
        episodes
            .iter()
            .map(|episode| {
                json!({
                    "_id": text(episode, "id").unwrap_or(""),
                    "name": text(episode, "name").or_else(|| text(&meta, "name")).unwrap_or(""),
                    "type": "series",
                    "poster": episode.get("thumbnail").cloned().unwrap_or(Value::Null),
                    "background": meta.get("background").cloned().unwrap_or(Value::Null),
                    "logo": meta.get("logo").cloned().unwrap_or(Value::Null),
                    "state": {
                        "lastWatched": watched_at_value,
                        "timeOffset": 0,
                        "duration": 0,
                        "videoId": text(episode, "id").unwrap_or(""),
                        "timesWatched": watched_value,
                        "flaggedWatched": watched_value
                    },
                    "lastWatched": watched_at_value
                })
            })
            .collect::<Vec<_>>()
    } else {
        vec![library_item_from_meta(
            &meta,
            json!({
                "lastWatched": watched_at_value,
                "timeOffset": 0,
                "duration": 0,
                "videoId": Value::Null,
                "timesWatched": watched_value,
                "flaggedWatched": watched_value
            }),
            watched_at,
        )]
    };
    serde_json::to_string(&items).ok()
}
