use super::super::*;
use serde_json::{Value, json};

#[test]
fn simkl_watching_items_are_kept_for_continue_watching() {
    let items = simkl_watching_to_items_json(
            r#"{"shows":[{"show":{"title":"Example","ids":{"imdb":"tt42"}},"last_watched":"S01E02","next_to_watch":"S01E03","last_watched_at":"2026-07-21T00:00:00.000Z","seasons":[{"number":1,"episodes":[{"number":2}]}]}]}"#,
            "[]",
        )
        .expect("items");
    let replaced: Value = serde_json::from_str(&replace_external_continue_watching_json(
        "[]",
        Some("simkl"),
        &items,
        None,
        None,
        None,
    ))
    .unwrap();
    assert_eq!(replaced[0]["id"], "tt42");
    assert_eq!(replaced[0]["lastVideoId"], "tt42:1:3");
    assert_eq!(replaced[0]["lastEpisodeSeason"], 1);
    assert_eq!(replaced[0]["lastEpisodeNumber"], 3);
    assert_eq!(replaced[0]["continueWatchingBadge"], "upNext");
    assert!(replaced[0].get("timeOffset").is_none());
    assert!(replaced[0].get("duration").is_none());
}

#[test]
fn simkl_request_policy_builds_series_history_and_watchlist_removal() {
    let history = simkl_history_request_json(
        &json!({
            "imdbId": "tt1",
            "isSeries": true,
            "episodesBySeasonNumber": { "2": [3, 4] },
        })
        .to_string(),
    )
    .and_then(|value| serde_json::from_str::<Value>(&value).ok())
    .unwrap();
    assert_eq!(history["shows"][0]["seasons"][0]["number"], 2);
    assert_eq!(
        history["shows"][0]["seasons"][0]["episodes"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    let removal = simkl_watchlist_request_json(
        &json!({ "imdbId": "tt1", "isSeries": false }).to_string(),
        true,
    )
    .and_then(|value| serde_json::from_str::<Value>(&value).ok())
    .unwrap();
    assert!(removal["movies"][0].get("to").is_none());
}

#[test]
fn simkl_resource_sync_plan_marks_removed_from_list_as_force_full() {
    let plan: Value = serde_json::from_str(&simkl_resource_sync_plan_json(&json!({
            "previous": { "tv_shows": { "watching": "t1", "removed_from_list": "r1" } },
            "current": { "tv_shows": { "watching": "t1", "removed_from_list": "r2" } },
            "resources": [{ "key": "showsWatching", "type": "tv_shows", "status": "watching", "hasCached": true }],
        }).to_string()).unwrap()).unwrap();
    assert_eq!(plan[0]["action"], "full");
}

#[test]
fn simkl_resource_sync_plan_uses_date_from_for_partial_delta() {
    let plan: Value = serde_json::from_str(&simkl_resource_sync_plan_json(&json!({
            "previous": { "tv_shows": { "watching": "t1", "removed_from_list": "r1" } },
            "current": { "tv_shows": { "watching": "t2", "removed_from_list": "r1" } },
            "resources": [{ "key": "showsWatching", "type": "tv_shows", "status": "watching", "hasCached": true }],
        }).to_string()).unwrap()).unwrap();
    assert_eq!(plan[0]["action"], "delta");
    assert_eq!(plan[0]["dateFrom"], "t1");
}

#[test]
fn simkl_merge_delta_updates_existing_and_appends_new_items() {
    let merged: Value = serde_json::from_str(&simkl_merge_delta_json(
            &json!([{ "ids": { "simkl": 1 }, "progress": 10 }, { "ids": { "simkl": 2 }, "progress": 20 }]).to_string(),
            &json!([{ "ids": { "simkl": 1 }, "progress": 50 }, { "ids": { "simkl": 3 }, "progress": 5 }]).to_string(),
        ).unwrap()).unwrap();
    assert_eq!(
        merged,
        json!([
            { "ids": { "simkl": 1 }, "progress": 50 },
            { "ids": { "simkl": 2 }, "progress": 20 },
            { "ids": { "simkl": 3 }, "progress": 5 },
        ])
    );
}

#[test]
fn simkl_playback_deletion_matches_shared_content_identity() {
    let ids: Value = serde_json::from_str(&simkl_playback_delete_ids_json(&json!({
            "contentId": "tt4574334",
            "items": [
                {"id": 12345, "show": {"ids": {"simkl": 39687, "imdb": "tt4574334", "tvdb": 305288}}},
                {"id": 99, "movie": {"ids": {"simkl": 1, "imdb": "tt9999999"}}},
            ],
        }).to_string()).unwrap()).unwrap();
    assert_eq!(ids, json!([12345]));
}
