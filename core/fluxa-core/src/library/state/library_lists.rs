use super::continue_watching::is_up_next_item;
use super::helpers::{number, text};
use serde_json::{Value, json};

pub(crate) fn library_continue_watching_items_json(items_json: &str) -> Option<String> {
    let mut items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    items.retain(|item| {
        let state = item.get("state").unwrap_or(&Value::Null);
        let removed = item
            .get("removed")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        !removed
            && !state.is_null()
            && number(state, "timeOffset").unwrap_or(0) > 0
            && number(state, "flaggedWatched").unwrap_or(0) == 0
    });
    items.sort_by(|a, b| {
        let a = a
            .get("state")
            .and_then(|state| text(state, "lastWatched"))
            .unwrap_or("");
        let b = b
            .get("state")
            .and_then(|state| text(state, "lastWatched"))
            .unwrap_or("");
        b.cmp(a)
    });
    let metas = items
        .into_iter()
        .map(|item| {
            let state = item.get("state").unwrap_or(&Value::Null);
            json!({
                "id": text(&item, "_id").unwrap_or(""),
                "name": text(&item, "name").unwrap_or(""),
                "type": text(&item, "type").unwrap_or(""),
                "poster": item.get("poster").cloned().unwrap_or(Value::Null),
                "background": item.get("background").cloned().unwrap_or(Value::Null),
                "logo": item.get("logo").cloned().unwrap_or(Value::Null),
                "description": Value::Null,
                "timeOffset": number(state, "timeOffset"),
                "duration": number(state, "duration"),
                "lastVideoId": text(state, "videoId"),
                "reason": "stremio"
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&metas).ok()
}

pub(crate) fn library_watchlist_items_json(items_json: &str) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let entries: Vec<Value> = items
        .iter()
        .filter(|item| {
            !item
                .get("removed")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .filter_map(|item| {
            let id = text(item, "_id").filter(|s| !s.is_empty())?.to_string();
            let updated_at_ms = text(item, "_mtime")
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.timestamp_millis())?;
            Some(json!({
                "id": id,
                "name": text(item, "name").unwrap_or(""),
                "type": text(item, "type").unwrap_or(""),
                "poster": item.get("poster").cloned().unwrap_or(Value::Null),
                "background": item.get("background").cloned().unwrap_or(Value::Null),
                "updatedAtMs": updated_at_ms
            }))
        })
        .collect();
    serde_json::to_string(&entries).ok()
}

pub(crate) fn filter_home_continue_watching_json(
    items_json: &str,
    trakt_watched_json: &str,
) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let trakt: Value = serde_json::from_str(trakt_watched_json).unwrap_or(Value::Null);

    let movie_keys: std::collections::HashSet<&str> = trakt
        .get("movieKeys")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let episode_keys: std::collections::HashSet<&str> = trakt
        .get("episodeKeys")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    let filtered: Vec<&Value> = items
        .iter()
        .filter(|item| {
            let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
            let last_video_id = item
                .get("lastVideoId")
                .and_then(Value::as_str)
                .unwrap_or("");
            let time_offset = item.get("timeOffset").and_then(Value::as_i64).unwrap_or(0);
            let duration = item.get("duration").and_then(Value::as_i64).unwrap_or(0);
            let is_series = matches!(item_type, "series" | "tv" | "anime");
            let is_up_next =
                is_series && !last_video_id.is_empty() && time_offset <= 0 && duration <= 0;
            let has_progress = time_offset > 0 && duration > 0;
            if !is_up_next && !has_progress {
                return false;
            }
            let watched_keys = crate::catalog::identity::content_watched_keys_value(item);
            if item_type == "movie"
                && !movie_keys.is_empty()
                && watched_keys.iter().any(|k| movie_keys.contains(k.as_str()))
            {
                return false;
            }
            if is_series
                && !episode_keys.is_empty()
                && !last_video_id.is_empty()
                && let Some((_, season, episode)) =
                    crate::catalog::identity::parse_episode_locator(last_video_id)
                && watched_keys.iter().any(|k| {
                    let candidate = format!("{k}:{season}:{episode}");
                    episode_keys.contains(candidate.as_str())
                })
            {
                return false;
            }
            true
        })
        .collect::<Vec<_>>();

    let mut ranked = filtered;
    ranked.sort_by_key(|item| {
        std::cmp::Reverse(
            item.get("lastWatchedAt")
                .and_then(Value::as_i64)
                .unwrap_or(0),
        )
    });

    serde_json::to_string(&ranked).ok()
}

pub(crate) fn watched_video_ids_json(items_json: &str, imdb_id: &str) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;
    let ids = items
        .iter()
        .filter(|item| {
            text(item, "_id").is_some_and(|id| id.starts_with(imdb_id))
                && item
                    .get("state")
                    .and_then(|state| number(state, "flaggedWatched"))
                    == Some(1)
        })
        .filter_map(|item| text(item, "_id").map(str::to_string))
        .collect::<Vec<_>>();
    serde_json::to_string(&ids).ok()
}

pub(crate) fn normalize_library_document_json(json: &str) -> String {
    let lib = serde_json::from_str(json).unwrap_or(Value::Null);
    serde_json::to_string(&normalize_library_document(&lib)).unwrap_or_else(|_| "{}".to_string())
}

pub(crate) fn normalize_library_document(lib: &Value) -> Value {
    let mut lib = lib.as_object().cloned().unwrap_or_default();
    lib.insert("schemaVersion".to_string(), json!(2));
    if !lib.get("watchlist").map(Value::is_array).unwrap_or(false) {
        lib.insert("watchlist".to_string(), json!([]));
    }
    if !lib.get("liked").map(Value::is_array).unwrap_or(false) {
        lib.insert("liked".to_string(), json!([]));
    }
    if !lib.get("history").map(Value::is_array).unwrap_or(false) {
        lib.insert("history".to_string(), json!([]));
    }
    if !lib
        .get("continueWatching")
        .map(Value::is_array)
        .unwrap_or(false)
    {
        lib.insert("continueWatching".to_string(), json!([]));
    }
    if !lib
        .get("progress")
        .map(|v| v.is_object() && !v.is_array())
        .unwrap_or(false)
    {
        lib.insert("progress".to_string(), json!({}));
    }
    if !lib
        .get("watched")
        .map(|v| v.is_object() && !v.is_array())
        .unwrap_or(false)
    {
        lib.insert("watched".to_string(), json!({}));
    }
    if !lib.get("dropped").map(Value::is_array).unwrap_or(false) {
        lib.insert("dropped".to_string(), json!([]));
    }
    if !lib.get("completed").map(Value::is_array).unwrap_or(false) {
        lib.insert("completed".to_string(), json!([]));
    }
    Value::Object(lib)
}

pub(crate) fn normalize_library_read_result_json(json: &str) -> String {
    let mut result = serde_json::from_str::<Value>(json)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    for (field, aliases) in [
        ("watchlist", &["watchlist", "libraryItems"][..]),
        ("continueWatching", &["continueWatching"][..]),
        ("liked", &["liked", "favorites"][..]),
        ("dropped", &["dropped"][..]),
        ("completed", &["completed"][..]),
    ] {
        let value = aliases
            .iter()
            .find_map(|alias| result.get(*alias).filter(|value| value.is_array()))
            .cloned()
            .unwrap_or_else(|| json!([]));
        result.insert(field.to_string(), value);
    }

    let watched = result
        .get("watched")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut canonical_watched = serde_json::Map::new();
    for (key, value) in watched {
        match value {
            Value::Bool(watched) => {
                canonical_watched.insert(key, Value::Bool(watched));
            }
            Value::Array(ids) => {
                for id in ids
                    .into_iter()
                    .filter_map(|id| id.as_str().map(str::to_owned))
                {
                    canonical_watched.insert(id, Value::Bool(true));
                }
            }
            _ => {}
        }
    }
    result.insert("watched".to_string(), Value::Object(canonical_watched));
    serde_json::to_string(&Value::Object(result)).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_library_document_supplies_shared_empty_views() {
        let value: Value = serde_json::from_str(&normalize_library_document_json("{}"))
            .expect("normalized library document");
        assert_eq!(value["watchlist"], json!([]));
        assert_eq!(value["liked"], json!([]));
        assert_eq!(value["continueWatching"], json!([]));
        assert_eq!(value["progress"], json!({}));
        assert_eq!(value["schemaVersion"], 2);
    }

    #[test]
    fn library_read_result_normalizes_android_and_desktop_watched_shapes() {
        let android: Value = serde_json::from_str(&normalize_library_read_result_json(
            r#"{"watchlist":[],"continueWatching":[],"favorites":[{"id":"f"}],"watched":{"tt1":["tt1:1:2","tt1:1:3"]}}"#,
        ))
        .unwrap();
        let desktop: Value = serde_json::from_str(&normalize_library_read_result_json(
            r#"{"watchlist":[],"continueWatching":[],"liked":[{"id":"f"}],"watched":{"tt1:1:2":true,"tt1:1:3":true}}"#,
        ))
        .unwrap();

        assert_eq!(android["liked"], desktop["liked"]);
        assert_eq!(android["watched"], desktop["watched"]);
        assert_eq!(android["completed"], json!([]));
        assert_eq!(android["dropped"], json!([]));
    }
}

pub(crate) fn is_up_next_continue_watching_item_json(item_json: &str) -> bool {
    let item: Value = serde_json::from_str(item_json).unwrap_or(Value::Null);
    if let Some(content_type) = item.get("type").and_then(Value::as_str)
        && !matches!(content_type, "series" | "tv" | "anime")
    {
        return false;
    }
    is_up_next_item(&item)
}
