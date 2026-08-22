use serde_json::{Value, json};

pub(super) fn hash_payload(value: &Value) -> String {
    let text = value.to_string();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    format!("{hash:016x}")
}

pub(super) fn document(entity: &str, key: &str, payload: Value) -> Value {
    json!({ "entity_type": entity, "key": key, "payload": payload })
}

pub(super) fn entries(value: Option<&Value>) -> Vec<(String, Value)> {
    value
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .map(|(key, item)| (key.clone(), item.clone()))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn sparse_against(value: &Value, defaults: &Value) -> Value {
    let (Some(fields), Some(base)) = (value.as_object(), defaults.as_object()) else {
        return value.clone();
    };
    let mut diff = serde_json::Map::new();
    for (key, current) in fields {
        if base.get(key) != Some(current) {
            diff.insert(key.clone(), current.clone());
        }
    }
    Value::Object(diff)
}

pub(super) fn merged_with(sparse: &Value, defaults: &Value) -> Value {
    let Some(base) = defaults.as_object() else {
        return sparse.clone();
    };
    let mut merged = base.clone();
    if let Some(fields) = sparse.as_object() {
        for (key, value) in fields {
            merged.insert(key.clone(), value.clone());
        }
    }
    Value::Object(merged)
}

fn push_single(documents: &mut Vec<Value>, args: &Value, field: &str, entity: &str, key: &str) {
    if let Some(value) = args.get(field).filter(|value| !value.is_null()) {
        let payload = if field == "addons" { compact_addons(value) } else { value.clone() };
        documents.push(document(entity, key, payload));
    }
}

fn compact_addons(value: &Value) -> Value {
    Value::Array(value.as_array().cloned().unwrap_or_default().into_iter().filter_map(|addon| {
        let object = addon.as_object()?;
        let url = object.get("transportUrl").or_else(|| object.get("url")).and_then(Value::as_str)?;
        Some(json!({
            "url": url,
            "name": object.get("name").or_else(|| object.get("manifest").and_then(|manifest| manifest.get("name"))).cloned().unwrap_or_else(|| json!(url)),
            "enabled": object.get("enabled").cloned().unwrap_or_else(|| json!(true)),
            "sortOrder": object.get("sortOrder").cloned().unwrap_or_else(|| json!(0)),
        }))
    }).collect())
}

fn compact_library_item(value: &Value) -> Value {
    let Some(object) = value.as_object() else { return value.clone(); };
    let allowed = [
        "id", "type", "name", "poster", "posterShape", "background", "description",
        "releaseInfo", "imdbRating", "genres", "addonBaseUrl", "addedAt",
    ];
    let mut compact = serde_json::Map::new();
    for key in allowed {
        if let Some(value) = object.get(key).filter(|value| !value.is_null()) {
            compact.insert(key.into(), value.clone());
        }
    }
    Value::Object(compact)
}

fn compact_progress(key: &str, value: &Value) -> Value {
    let mut compact = serde_json::Map::new();
    let source = value.as_object();
    let meta = source.and_then(|object| object.get("meta")).and_then(Value::as_object);

    compact.insert(
        "contentId".into(),
        source
            .and_then(|object| object.get("contentId"))
            .or_else(|| meta.and_then(|object| object.get("id")))
            .cloned()
            .unwrap_or_else(|| json!(key)),
    );
    compact.insert(
        "contentType".into(),
        source
            .and_then(|object| object.get("contentType"))
            .or_else(|| meta.and_then(|object| object.get("type")))
            .cloned()
            .unwrap_or_else(|| json!("movie")),
    );

    for (target, aliases) in [
        ("videoId", &["videoId", "lastVideoId"][..]),
        ("season", &["season", "lastEpisodeSeason"][..]),
        ("episode", &["episode", "lastEpisodeNumber"][..]),
        ("position", &["position", "timeOffset"][..]),
        ("duration", &["duration"][..]),
        ("lastWatched", &["lastWatched", "savedAt"][..]),
        ("progressKey", &["progressKey"][..]),
        ("lastAudioLanguage", &["lastAudioLanguage"][..]),
        ("lastSubtitleLanguage", &["lastSubtitleLanguage"][..]),
        ("lastStreamIndex", &["lastStreamIndex"][..]),
    ] {
        if let Some(found) = aliases
            .iter()
            .find_map(|alias| source.and_then(|object| object.get(*alias)))
            .filter(|item| !item.is_null())
        {
            compact.insert(target.into(), found.clone());
        }
    }
    if !compact.contains_key("progressKey") {
        compact.insert("progressKey".into(), json!(key));
    }
    Value::Object(compact)
}

fn compact_history(value: &Value) -> Value {
    let object = value.as_object();
    let mut compact = serde_json::Map::new();
    compact.insert(
        "contentType".into(),
        object
            .and_then(|entry| entry.get("contentType"))
            .or_else(|| object.and_then(|entry| entry.get("type")))
            .cloned()
            .unwrap_or_else(|| json!("movie")),
    );
    for (target, aliases) in [
        ("videoId", &["videoId", "lastVideoId", "id"][..]),
        ("season", &["season", "lastEpisodeSeason"][..]),
        ("episode", &["episode", "lastEpisodeNumber"][..]),
        ("lastWatched", &["lastWatched", "savedAt"][..]),
    ] {
        if let Some(found) = aliases
            .iter()
            .find_map(|alias| object.and_then(|entry| entry.get(*alias)))
            .filter(|item| !item.is_null())
        {
            compact.insert(target.into(), found.clone());
        }
    }
    if compact.is_empty() {
        compact.insert("watched".into(), json!(true));
    }
    Value::Object(compact)
}

pub(crate) fn documents_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let mut documents: Vec<Value> = Vec::new();

    for (key, value) in entries(args.get("progress")) {
        documents.push(document("watch_progress", &key, compact_progress(&key, &value)));
    }

    for (key, value) in entries(args.get("watched")) {
        if value.as_bool() == Some(true) {
            documents.push(document(
                "watched_history",
                &format!("video:{key}"),
                json!({ "watched": true, "contentType": "movie", "videoId": key }),
            ));
        }
    }

    for (key, value) in entries(args.get("lastWatched")) {
        documents.push(document("watched_history", &format!("series:{key}"), compact_history(&value)));
    }

    for (status, items) in entries(args.get("library")) {
        for item in items.as_array().cloned().unwrap_or_default() {
            let Some(id) = item.get("id").and_then(Value::as_str) else {
                continue;
            };
            documents.push(document(
                "library",
                id,
                json!({ "status": status, "item": compact_library_item(&item) }),
            ));
        }
    }

    for collection in args
        .get("collections")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(id) = collection.get("id").and_then(Value::as_str) else {
            continue;
        };
        documents.push(document("collections", id, collection.clone()));
    }

    push_single(&mut documents, &args, "addons", "addons", "local");
    push_single(&mut documents, &args, "plugins", "plugins", "local");
    push_single(&mut documents, &args, "profile", "settings", "profile");

    if let Some(settings) = args.get("settings").filter(|value| !value.is_null()) {
        let defaults = args.get("settingsDefaults").cloned().unwrap_or(Value::Null);
        let sparse = sparse_against(settings, &defaults);
        if sparse.as_object().is_none_or(|diff| !diff.is_empty()) {
            documents.push(document("settings", "app", sparse));
        }
    }

    serde_json::to_string(&json!({ "documents": documents })).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn payload_for(result: &str, entity: &str, key: &str) -> Value {
        let parsed: Value = serde_json::from_str(result).expect("valid json");
        parsed
            .get("documents")
            .and_then(Value::as_array)
            .expect("documents")
            .iter()
            .find(|document| {
                document.get("entity_type").and_then(Value::as_str) == Some(entity)
                    && document.get("key").and_then(Value::as_str) == Some(key)
            })
            .and_then(|document| document.get("payload"))
            .cloned()
            .unwrap_or(Value::Null)
    }

    #[test]
    fn settings_matching_defaults_are_not_stored() {
        let result = documents_json(
            &json!({
                "settings": { "autoplay": true, "theme": "dark", "volume": 40 },
                "settingsDefaults": { "autoplay": true, "theme": "dark", "volume": 100 },
            })
            .to_string(),
        )
        .expect("plan");

        assert_eq!(payload_for(&result, "settings", "app"), json!({ "volume": 40 }));
    }

    #[test]
    fn settings_back_at_their_defaults_stop_being_a_document() {
        let result = documents_json(
            &json!({
                "settings": { "autoplay": true, "volume": 100 },
                "settingsDefaults": { "autoplay": true, "volume": 100 },
            })
            .to_string(),
        )
        .expect("plan");
        let parsed: Value = serde_json::from_str(&result).expect("valid json");

        assert!(
            parsed
                .get("documents")
                .and_then(Value::as_array)
                .expect("documents")
                .is_empty()
        );
    }

    #[test]
    fn settings_survive_a_default_that_changes_later() {
        let stored = json!({ "volume": 40 });
        let effective = merged_with(&stored, &json!({ "autoplay": false, "volume": 100 }));

        assert_eq!(effective.get("autoplay"), Some(&json!(false)));
        assert_eq!(effective.get("volume"), Some(&json!(40)));
    }

    #[test]
    fn progress_documents_only_contain_compact_resume_data() {
        let result = documents_json(
            &json!({
                "progress": {
                    "tt3823824": {
                        "meta": {"id":"tt3823824", "type":"series", "name":"Example", "poster":"https://example/poster.jpg"},
                        "timeOffset": 1234,
                        "duration": 3600,
                        "lastVideoId": "tt3823824:1:4",
                        "lastEpisodeSeason": 1,
                        "lastEpisodeNumber": 4,
                        "lastStreamUrl": "https://private.example/temporary",
                        "lastStream": {"url":"https://private.example/temporary"}
                    }
                }
            }).to_string(),
        ).expect("plan");
        let payload = payload_for(&result, "watch_progress", "tt3823824");
        assert_eq!(payload["contentId"], "tt3823824");
        assert_eq!(payload["videoId"], "tt3823824:1:4");
        assert!(payload.get("meta").is_none());
        assert!(payload.get("lastStreamUrl").is_none());
        assert!(payload.get("lastStream").is_none());
    }

    #[test]
    fn library_documents_keep_metadata_for_bulk_library_display() {
        let result = documents_json(
            &json!({
                "library": {
                    "watchlist": [{
                        "id":"tt123",
                        "type":"movie",
                        "name":"Example",
                        "poster":"https://example/poster.jpg",
                        "background":"https://example/background.jpg",
                        "streamUrl":"https://private.example/stream"
                    }]
                }
            }).to_string(),
        ).expect("plan");
        let payload = payload_for(&result, "library", "tt123");
        assert_eq!(payload["item"]["id"], "tt123");
        assert_eq!(payload["item"]["poster"], "https://example/poster.jpg");
        assert!(payload["item"].get("streamUrl").is_none());
    }

    #[test]
    fn history_documents_keep_only_episode_identity_and_time() {
        let result = documents_json(
            &json!({
                "lastWatched": {
                    "tt123": {
                        "contentType":"series",
                        "lastVideoId":"tt123:1:4",
                        "lastEpisodeSeason":1,
                        "lastEpisodeNumber":4,
                        "lastEpisodeName":"Example",
                        "poster":"https://example/poster.jpg"
                    }
                }
            }).to_string(),
        ).expect("plan");
        let payload = payload_for(&result, "watched_history", "series:tt123");
        assert_eq!(payload["contentType"], "series");
        assert_eq!(payload["videoId"], "tt123:1:4");
        assert!(payload.get("lastEpisodeName").is_none());
        assert!(payload.get("poster").is_none());
    }
}
