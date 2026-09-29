use super::super::*;
use serde_json::Value;

#[test]
fn stremio_episode_entries_become_watched_keys_not_watchlist_items() {
    let items = r#"[
            {"_id":"tt1","name":"A Movie","type":"movie","poster":"p.jpg","state":{"flaggedWatched":1}},
            {"_id":"tt2","name":"A Show","type":"series","state":{"flaggedWatched":0}},
            {"_id":"tt2:1:3","name":"A Show","type":"series","state":{"flaggedWatched":1}},
            {"_id":"tt3","name":"Removed","type":"movie","removed":true,"state":null}
        ]"#;
    let watchlist: Vec<Value> =
        serde_json::from_str(&stremio_watchlist_to_items_json(items).unwrap()).unwrap();
    let ids: Vec<&str> = watchlist
        .iter()
        .filter_map(|i| i.get("id").and_then(Value::as_str))
        .collect();
    assert_eq!(ids, vec!["tt1", "tt2"]);

    let watched: Value =
        serde_json::from_str(&stremio_watched_to_ids_json(items).unwrap()).unwrap();
    assert_eq!(watched.get("tt1"), Some(&Value::Bool(true)));
    assert_eq!(watched.get("tt2:1:3"), Some(&Value::Bool(true)));
    assert_eq!(watched.get("tt2"), None);
}
