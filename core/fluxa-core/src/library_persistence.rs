use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryItem<'a> {
    media_id: &'a str,
    status: &'static str,
    value: &'a Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyedValue<'a> {
    key: &'a str,
    value: &'a Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LibraryViews<'a> {
    progress: Vec<KeyedValue<'a>>,
    library_items: Vec<LibraryItem<'a>>,
    watched_video_ids: Vec<&'a str>,
    last_watched_entries: Vec<KeyedValue<'a>>,
    continue_watching_entries: Vec<KeyedValue<'a>>,
}

pub(crate) fn progress_entries_json(document_json: &str) -> String {
    object_entries(document_json, "progress")
}

fn with_document<F>(document_json: &str, render: F) -> String
where
    F: FnOnce(&Value) -> String,
{
    static CACHE: OnceLock<Mutex<HashMap<(u64, usize), (Arc<str>, Arc<Value>)>>> = OnceLock::new();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    document_json.hash(&mut hasher);
    let key = (hasher.finish(), document_json.len());
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let document = if let Some((source, document)) = cache.get(&key) {
        if source.as_ref() == document_json {
            Arc::clone(document)
        } else {
            let Ok(document) = serde_json::from_str(document_json) else {
                return "[]".to_string();
            };
            let document = Arc::new(document);
            cache.insert(key, (Arc::from(document_json), Arc::clone(&document)));
            document
        }
    } else {
        let Ok(document) = serde_json::from_str(document_json) else {
            return "[]".to_string();
        };
        if cache.len() >= 4 {
            cache.clear();
        }
        let document = Arc::new(document);
        cache.insert(key, (Arc::from(document_json), Arc::clone(&document)));
        document
    };
    drop(cache);
    render(&document)
}

pub(crate) fn library_items_json(document_json: &str) -> String {
    with_document(document_json, |document| {
        let mut entries = Vec::new();
        for status in ["watchlist", "completed", "dropped"] {
            for item in document
                .get(status)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(media_id) = item.get("id").and_then(Value::as_str) {
                    entries.push(LibraryItem {
                        media_id,
                        status,
                        value: item,
                    });
                }
            }
        }
        serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
    })
}

pub(crate) fn watched_video_ids_json(document_json: &str) -> String {
    with_document(document_json, |document| {
        let ids = document
            .get("watched")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter_map(|(video_id, watched)| {
                (watched.as_bool() == Some(true)).then_some(video_id.as_str())
            })
            .collect::<Vec<_>>();
        serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string())
    })
}

pub(crate) fn last_watched_entries_json(document_json: &str) -> String {
    object_entries(document_json, "lastWatchedEpisodes")
}

pub(crate) fn continue_watching_entries_json(document_json: &str) -> String {
    with_document(document_json, |document| {
        let entries = document
            .get("externalContinueWatching")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| {
                Some(KeyedValue {
                    key: item.get("id")?.as_str()?,
                    value: item,
                })
            })
            .collect::<Vec<_>>();
        serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
    })
}

pub(crate) fn document_views_json(document_json: &str) -> String {
    with_document(document_json, |document| {
        let progress = object_entries_value(document, "progress");
        let last_watched_entries = object_entries_value(document, "lastWatchedEpisodes");
        let continue_watching_entries = document
            .get("externalContinueWatching")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| {
                Some(KeyedValue {
                    key: item.get("id")?.as_str()?,
                    value: item,
                })
            })
            .collect();
        let library_items = ["watchlist", "completed", "dropped"]
            .into_iter()
            .flat_map(|status| {
                document
                    .get(status)
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(move |item| {
                        Some(LibraryItem {
                            media_id: item.get("id")?.as_str()?,
                            status,
                            value: item,
                        })
                    })
            })
            .collect();
        let watched_video_ids = document
            .get("watched")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter_map(|(video_id, watched)| {
                (watched.as_bool() == Some(true)).then_some(video_id.as_str())
            })
            .collect();
        serde_json::to_string(&LibraryViews {
            progress,
            library_items,
            watched_video_ids,
            last_watched_entries,
            continue_watching_entries,
        })
        .unwrap_or_else(|_| "{}".to_string())
    })
}

fn object_entries(document_json: &str, field: &str) -> String {
    with_document(document_json, |document| {
        let entries = object_entries_value(document, field);
        serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
    })
}

fn object_entries_value<'a>(document: &'a Value, field: &str) -> Vec<KeyedValue<'a>> {
    document
        .get(field)
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(key, value)| KeyedValue { key, value })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_all_legacy_library_domains() {
        let document = r#"{
            "progress":{"movie:1":{"position":12}},
            "watchlist":[{"id":"movie:1"}],
            "completed":[{"id":"movie:2"}],
            "dropped":[],
            "watched":{"video:1":true,"video:2":false},
            "lastWatchedEpisodes":{"series:1":{"id":"episode:2"}},
            "externalContinueWatching":[{"id":"movie:3"}]
        }"#;
        assert!(progress_entries_json(document).contains("movie:1"));
        assert!(library_items_json(document).contains("completed"));
        assert_eq!(watched_video_ids_json(document), r#"["video:1"]"#);
        assert!(last_watched_entries_json(document).contains("series:1"));
        assert!(continue_watching_entries_json(document).contains("movie:3"));
    }

    #[test]
    fn extracts_all_views_from_one_document_parse() {
        let document = r#"{
            "progress":{"movie:1":{"position":12}},
            "watchlist":[{"id":"movie:1"}],
            "watched":{"video:1":true},
            "lastWatchedEpisodes":{"series:1":{"id":"episode:2"}},
            "externalContinueWatching":[{"id":"movie:3"}]
        }"#;
        let views: Value = serde_json::from_str(&document_views_json(document)).unwrap();
        assert_eq!(views["watchedVideoIds"][0], "video:1");
        assert_eq!(views["libraryItems"][0]["mediaId"], "movie:1");
        assert!(views["progress"][0]["key"] == "movie:1");
    }
}
