use serde_json::{Map, Value, json};

fn key(item: &Value) -> Option<String> {
    let id = item.get("content_id")?.as_str()?.trim();
    if id.is_empty() {
        return None;
    }
    item.get("progress_key")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .or_else(|| {
            Some(
                match (
                    item.get("season").and_then(Value::as_i64),
                    item.get("episode").and_then(Value::as_i64),
                ) {
                    (Some(season), Some(episode)) => format!("{id}_s{season}e{episode}"),
                    _ => id.to_string(),
                },
            )
        })
}

fn resource_key(resource: &str, item: &Value) -> Option<String> {
    match resource {
        "progress" => key(item),
        "library" => {
            let id = item.get("content_id")?.as_str()?.trim();
            let content_type = item.get("content_type")?.as_str()?.trim();
            (!id.is_empty() && !content_type.is_empty())
                .then(|| format!("{}:{id}", content_type.to_ascii_lowercase()))
        }
        "history" => {
            let id = item.get("content_id")?.as_str()?.trim();
            if id.is_empty() {
                return None;
            }
            Some(
                match (
                    item.get("season").and_then(Value::as_i64),
                    item.get("episode").and_then(Value::as_i64),
                ) {
                    (Some(season), Some(episode)) => format!("{id}_s{season}e{episode}"),
                    _ => id.to_string(),
                },
            )
        }
        _ => None,
    }
}

fn newer(left: &Value, right: &Value) -> bool {
    let left_at = left
        .get("last_watched")
        .and_then(Value::as_i64)
        .unwrap_or(i64::MIN);
    let right_at = right
        .get("last_watched")
        .and_then(Value::as_i64)
        .unwrap_or(i64::MIN);
    left_at > right_at
        || (left_at == right_at
            && left.get("position").and_then(Value::as_i64).unwrap_or(0)
                > right.get("position").and_then(Value::as_i64).unwrap_or(0))
}

fn deduplicate(items: Vec<Value>) -> Vec<Value> {
    let mut by_key = Map::new();
    for item in items {
        let Some(item_key) = key(&item) else {
            continue;
        };
        if by_key
            .get(&item_key)
            .is_none_or(|previous| newer(&item, previous))
        {
            by_key.insert(item_key, item);
        }
    }
    let mut values: Vec<Value> = by_key.into_values().collect();
    values.sort_by(|left, right| {
        right
            .get("last_watched")
            .and_then(Value::as_i64)
            .cmp(&left.get("last_watched").and_then(Value::as_i64))
    });
    values
}

fn projection(items: &[Value]) -> Vec<Value> {
    let mut latest = Map::new();
    for item in items {
        let id = item
            .get("content_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        let content_type = item
            .get("content_type")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if id.is_empty()
            || content_type.is_empty()
            || item.get("duration").and_then(Value::as_i64).unwrap_or(0) <= 0
            || item.get("position").and_then(Value::as_i64).unwrap_or(-1) < 0
        {
            continue;
        }
        let content_key = format!("{}:{id}", content_type.to_ascii_lowercase());
        if latest
            .get(&content_key)
            .is_none_or(|previous| newer(item, previous))
        {
            latest.insert(content_key, item.clone());
        }
    }
    let mut values: Vec<Value> = latest.into_values().collect();
    values.sort_by(|left, right| {
        right
            .get("last_watched")
            .and_then(Value::as_i64)
            .cmp(&left.get("last_watched").and_then(Value::as_i64))
    });
    values
}

pub(crate) fn progress_sync_request_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let state = args.get("state").and_then(Value::as_object);
    let initialized = state
        .and_then(|value| value.get("initialized"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let cursor = state
        .and_then(|value| value.get("cursor"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0);
    serde_json::to_string(
        &json!({ "mode": if initialized { "delta" } else { "bootstrap" }, "cursor": cursor }),
    )
    .ok()
}

pub(crate) fn delta_sync_request_plan_json(args_json: &str) -> Option<String> {
    progress_sync_request_plan_json(args_json)
}

pub(crate) fn apply_delta_sync_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let resource = args.get("resource").and_then(Value::as_str)?;
    let state = args.get("state").and_then(Value::as_object);
    let initialized = state
        .and_then(|value| value.get("initialized"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut cursor = state
        .and_then(|value| value.get("cursor"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0);
    let items = if initialized {
        state
            .and_then(|value| value.get("items"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    } else {
        cursor = args
            .get("snapshotCursor")
            .and_then(Value::as_i64)
            .unwrap_or(cursor)
            .max(0);
        args.get("snapshot")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let mut by_key: Map<String, Value> = items
        .into_iter()
        .filter_map(|item| resource_key(resource, &item).map(|item_key| (item_key, item)))
        .collect();
    let mut events = args
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    events.sort_by_key(|event| event.get("event_id").and_then(Value::as_i64).unwrap_or(0));
    for event in events {
        let event_id = event.get("event_id").and_then(Value::as_i64).unwrap_or(0);
        if event_id <= cursor {
            continue;
        }
        if let Some(event_key) = resource_key(resource, &event) {
            match event
                .get("operation")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase()
                .as_str()
            {
                "delete" => {
                    by_key.remove(&event_key);
                }
                "upsert" => {
                    by_key.insert(event_key, event);
                }
                _ => {}
            }
        }
        cursor = event_id;
    }
    let mut items: Vec<Value> = by_key.into_values().collect();
    items.sort_by(|left, right| {
        right
            .get("last_watched")
            .or_else(|| right.get("watched_at"))
            .or_else(|| right.get("added_at"))
            .and_then(Value::as_i64)
            .cmp(
                &left
                    .get("last_watched")
                    .or_else(|| left.get("watched_at"))
                    .or_else(|| left.get("added_at"))
                    .and_then(Value::as_i64),
            )
    });
    serde_json::to_string(&json!({ "initialized": true, "cursor": cursor, "items": items })).ok()
}

pub(crate) fn apply_progress_sync_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let state = args.get("state").and_then(Value::as_object);
    let initialized = state
        .and_then(|value| value.get("initialized"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut cursor = state
        .and_then(|value| value.get("cursor"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0);
    let items = if initialized {
        state
            .and_then(|value| value.get("items"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    } else {
        cursor = args
            .get("snapshotCursor")
            .and_then(Value::as_i64)
            .unwrap_or(cursor)
            .max(0);
        args.get("snapshot")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let mut by_key: Map<String, Value> = deduplicate(items)
        .into_iter()
        .filter_map(|item| key(&item).map(|item_key| (item_key, item)))
        .collect();
    let mut events = args
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    events.sort_by_key(|event| event.get("event_id").and_then(Value::as_i64).unwrap_or(0));
    for event in events {
        let event_id = event.get("event_id").and_then(Value::as_i64).unwrap_or(0);
        if event_id <= cursor {
            continue;
        }
        if let Some(event_key) = key(&event) {
            match event
                .get("operation")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase()
                .as_str()
            {
                "delete" => {
                    by_key.remove(&event_key);
                }
                "upsert" => {
                    by_key.insert(event_key, event);
                }
                _ => {}
            }
        }
        cursor = event_id;
    }
    let items = deduplicate(by_key.into_values().collect());
    serde_json::to_string(&json!({ "initialized": true, "cursor": cursor, "items": items, "continueWatching": projection(&items) })).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn bootstrap_applies_ordered_events_and_projects_one_latest_episode_per_content() {
        let result = apply_progress_sync_json(&json!({
            "state": { "initialized": false, "cursor": 0, "items": [] },
            "snapshotCursor": 8,
            "snapshot": [
                { "content_id": "tt1", "content_type": "series", "progress_key": "tt1_s1e1", "season": 1, "episode": 1, "position": 200, "duration": 1000, "last_watched": 10 },
                { "content_id": "tt1", "content_type": "series", "progress_key": "tt1_s1e2", "season": 1, "episode": 2, "position": 300, "duration": 1000, "last_watched": 20 }
            ],
            "events": [
                { "event_id": 9, "operation": "upsert", "content_id": "tt1", "content_type": "series", "progress_key": "tt1_s1e3", "season": 1, "episode": 3, "position": 400, "duration": 1000, "last_watched": 30 },
                { "event_id": 10, "operation": "delete", "content_id": "tt1", "content_type": "series", "progress_key": "tt1_s1e1" }
            ]
        }).to_string()).unwrap();
        let value: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value["cursor"], 10);
        assert_eq!(value["items"].as_array().unwrap().len(), 2);
        assert_eq!(value["continueWatching"].as_array().unwrap().len(), 1);
        assert_eq!(value["continueWatching"][0]["episode"], 3);
    }

    #[test]
    fn initialized_state_uses_delta_without_replacing_the_snapshot() {
        let request =
            progress_sync_request_plan_json(r#"{"state":{"initialized":true,"cursor":42}}"#)
                .unwrap();
        let request: Value = serde_json::from_str(&request).unwrap();
        assert_eq!(request["mode"], "delta");
        assert_eq!(request["cursor"], 42);
        let result = apply_progress_sync_json(&json!({
            "state": { "initialized": true, "cursor": 42, "items": [{ "content_id": "tt2", "content_type": "movie", "position": 1, "duration": 10, "last_watched": 2 }] },
            "snapshot": [{ "content_id": "ignored", "content_type": "movie", "position": 1, "duration": 10 }],
            "events": []
        }).to_string()).unwrap();
        let value: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value["items"][0]["content_id"], "tt2");
    }

    #[test]
    fn generic_delta_state_uses_resource_identity_for_library_and_history() {
        let library = apply_delta_sync_json(&json!({
            "resource": "library",
            "state": { "initialized": false, "cursor": 0 },
            "snapshotCursor": 3,
            "snapshot": [{ "content_id": "tt1", "content_type": "movie", "name": "Old" }],
            "events": [{ "event_id": 4, "operation": "upsert", "content_id": "tt1", "content_type": "movie", "name": "New" }]
        }).to_string()).unwrap();
        let library: Value = serde_json::from_str(&library).unwrap();
        assert_eq!(library["items"][0]["name"], "New");

        let history = apply_delta_sync_json(&json!({
            "resource": "history",
            "state": { "initialized": true, "cursor": 4, "items": [{ "content_id": "tt1", "season": 1, "episode": 1 }] },
            "events": [{ "event_id": 5, "operation": "delete", "content_id": "tt1", "season": 1, "episode": 1 }]
        }).to_string()).unwrap();
        let history: Value = serde_json::from_str(&history).unwrap();
        assert!(history["items"].as_array().unwrap().is_empty());
    }
}
