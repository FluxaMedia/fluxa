use super::continue_watching::format_episode_line_json;
use serde_json::{Value, json};

/// Selects the best artwork URL for a continue-watching card.
/// `artwork_preference` is "poster", "background", or "episode" (default).
/// `is_horizontal` controls whether the card layout is wide/horizontal.
pub(crate) fn select_continue_watching_artwork_json(
    item_json: &str,
    artwork_preference: &str,
    is_horizontal: bool,
) -> Option<String> {
    let item: Value = serde_json::from_str(item_json).ok()?;
    let str_field = |key: &str| -> Option<String> {
        item.get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
    };

    let poster = str_field("poster");
    let background = str_field("background");
    let logo = str_field("logo");
    let thumbnail = str_field("lastEpisodeThumbnail");
    let cw_poster = str_field("continueWatchingPoster");
    let cw_background = str_field("continueWatchingBackground");

    let is_real_backdrop = background.as_deref().is_some_and(|bg| {
        (poster.as_deref() != Some(bg)) && !bg.to_lowercase().contains("/poster/")
    });
    let existing_backdrop = if is_real_backdrop {
        background.clone()
    } else {
        None
    };

    if !is_horizontal {
        thumbnail
            .or(cw_poster)
            .or(poster)
            .or(cw_background)
            .or(background)
    } else {
        let content_type = item.get("type").and_then(Value::as_str).unwrap_or("");
        let is_series = matches!(content_type, "series" | "tv" | "anime");
        let _ = is_series;
        match artwork_preference {
            "poster" => poster.or(cw_background).or(existing_backdrop),
            "background" => existing_backdrop.or(cw_background).or(poster),
            _ => thumbnail
                .or(cw_background)
                .or(existing_backdrop)
                .or(background)
                .or(logo)
                .or(poster),
        }
    }
}

/// Batched form of select_continue_watching_artwork_json + format_episode_line_json for
/// a whole Continue Watching row at once — each card used to call both over IPC
/// individually, which meant one IPC round trip per card on every Home load.
pub(crate) fn continue_watching_card_fields_json(
    items_json: &str,
    artwork_preference: &str,
    is_horizontal: bool,
) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let fields: Vec<Value> = items
        .iter()
        .map(|item| {
            let id = item
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let artwork = select_continue_watching_artwork_json(
                &item.to_string(),
                artwork_preference,
                is_horizontal,
            );
            let episode_line = format_episode_line_json(
                item.get("lastEpisodeName").and_then(Value::as_str),
                item.get("lastEpisodeSeason").and_then(Value::as_i64),
                item.get("lastEpisodeNumber").and_then(Value::as_i64),
                item.get("lastVideoId").and_then(Value::as_str),
            );
            json!({ "id": id, "artwork": artwork, "episodeLine": episode_line })
        })
        .collect();
    serde_json::to_string(&fields).ok()
}

/// Decides which entries of a bool map (e.g. watched) actually changed and need
/// persisting — before/after are id -> value maps.
pub(crate) fn watched_map_diff_json(before_json: &str, after_json: &str) -> Option<String> {
    let before: Value = serde_json::from_str(before_json).ok()?;
    let after: Value = serde_json::from_str(after_json).ok()?;
    let before = before.as_object()?;
    let after = after.as_object()?;

    let changed: Vec<Value> = after
        .iter()
        .filter(|(id, value)| before.get(*id) != Some(*value))
        .map(|(id, value)| json!({ "id": id, "value": value }))
        .collect();
    serde_json::to_string(&changed).ok()
}

/// Full upsert+delete diff for an id -> value map (e.g. playback progress).
pub(crate) fn value_map_diff_json(before_json: &str, after_json: &str) -> Option<String> {
    let before: Value = serde_json::from_str(before_json).ok()?;
    let after: Value = serde_json::from_str(after_json).ok()?;
    let before = before.as_object()?;
    let after = after.as_object()?;

    let upserts: Vec<Value> = after
        .iter()
        .filter(|(id, value)| before.get(*id) != Some(*value))
        .map(|(id, value)| json!({ "id": id, "value": value }))
        .collect();
    let deletes: Vec<&String> = before
        .keys()
        .filter(|id| !after.contains_key(*id))
        .collect();
    serde_json::to_string(&json!({ "upserts": upserts, "deletes": deletes })).ok()
}

/// Full upsert+delete diff for an id-keyed item list (e.g. continue watching rows).
pub(crate) fn item_list_diff_json(before_json: &str, after_json: &str) -> Option<String> {
    let before: Vec<Value> = serde_json::from_str(before_json).ok()?;
    let after: Vec<Value> = serde_json::from_str(after_json).ok()?;

    let mut before_by_id: std::collections::HashMap<String, &Value> =
        std::collections::HashMap::new();
    for item in &before {
        if let Some(id) = item.get("id").and_then(Value::as_str) {
            before_by_id.insert(id.to_string(), item);
        }
    }

    let mut after_ids = std::collections::HashSet::new();
    let mut upserts: Vec<Value> = Vec::new();
    for item in &after {
        let Some(id) = item.get("id").and_then(Value::as_str) else {
            continue;
        };
        after_ids.insert(id.to_string());
        if before_by_id.get(id) != Some(&item) {
            upserts.push(item.clone());
        }
    }
    let deletes: Vec<&String> = before_by_id
        .keys()
        .filter(|id| !after_ids.contains(*id))
        .collect();
    serde_json::to_string(&json!({ "upserts": upserts, "deletes": deletes })).ok()
}

/// New entries of an id-keyed item list that weren't present before (e.g. status lists
/// like watchlist/completed/dropped, which are append-only from the merge's perspective).
pub(crate) fn item_list_new_entries_json(before_json: &str, after_json: &str) -> Option<String> {
    let before: Vec<Value> = serde_json::from_str(before_json).ok()?;
    let after: Vec<Value> = serde_json::from_str(after_json).ok()?;

    let before_ids: std::collections::HashSet<String> = before
        .iter()
        .filter_map(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
        })
        .collect();
    let new_entries: Vec<&Value> = after
        .iter()
        .filter(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| !before_ids.contains(id))
        })
        .collect();
    serde_json::to_string(&new_entries).ok()
}
