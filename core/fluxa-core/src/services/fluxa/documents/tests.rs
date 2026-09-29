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

    assert_eq!(
        payload_for(&result, "settings", "app"),
        json!({ "volume": 40 })
    );
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
        })
        .to_string(),
    )
    .expect("plan");
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
        })
        .to_string(),
    )
    .expect("plan");
    let payload = payload_for(&result, "watched_history", "series:tt123");
    assert_eq!(payload["contentType"], "series");
    assert_eq!(payload["videoId"], "tt123:1:4");
    assert!(payload.get("lastEpisodeName").is_none());
    assert!(payload.get("poster").is_none());
}
