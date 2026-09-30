use super::*;
use serde_json::{Value, json};

#[test]
fn addon_snapshot_fetches_only_new_manifests_in_server_order() {
    let rows = json!([
        {"url": "https://b/manifest.json", "enabled": false, "sort_order": 1},
        {"url": "https://a/manifest.json", "enabled": true, "sort_order": 0}
    ]);
    let cached = json!({"rows": [], "addons": [{"transportUrl": "https://b/manifest.json", "manifest": {"id": "b"}}]});
    let plan: Value = serde_json::from_str(
        &addon_snapshot_plan_json(&json!({"rows": rows, "snapshot": cached}).to_string()).unwrap(),
    )
    .unwrap();
    assert_eq!(plan["changed"], true);
    assert_eq!(plan["fetch"], json!(["https://a/manifest.json"]));
    assert_eq!(plan["addons"][0]["enabled"], false);

    let settled = json!({"rows": plan["rows"], "addons": plan["addons"]});
    let manifests = json!({"https://a/manifest.json": {"id": "a"}});
    let plan: Value = serde_json::from_str(
        &addon_snapshot_plan_json(
            &json!({"rows": rows, "snapshot": settled, "manifests": manifests}).to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(plan["changed"], false);
    assert_eq!(plan["fetch"], json!([]));
    assert_eq!(plan["addons"][0]["manifest"]["id"], "a");
}

#[test]
fn effective_profile_scopes_apply_primary_inheritance_for_all_consumers() {
    let profiles = json!([
        {"profile_index": 1, "uses_primary_addons": false, "uses_primary_plugins": false},
        {"profile_index": 2, "uses_primary_addons": true, "uses_primary_plugins": false},
        {"profileIndex": 3, "usesPrimaryAddons": false, "usesPrimaryPlugins": true}
    ]);
    let resolve = |profile_index| {
        let input = json!({"profileIndex": profile_index, "profiles": profiles});
        serde_json::from_str::<Value>(&effective_profile_scopes_json(&input.to_string()).unwrap())
            .unwrap()
    };
    assert_eq!(resolve(1), json!({"addons": 1, "plugins": 1}));
    assert_eq!(resolve(2), json!({"addons": 1, "plugins": 2}));
    assert_eq!(resolve(3), json!({"addons": 3, "plugins": 1}));
    assert_eq!(resolve(4), json!({"addons": 4, "plugins": 4}));
}

#[test]
fn provider_library_snapshot_maps_nuvio_watchlist_and_progress_for_native_consumers() {
    let snapshot: Value = serde_json::from_str(
        &provider_library_snapshot_json(
            &json!({
                "library": [{
                    "content_id": "tt1", "content_type": "series", "name": "Show",
                    "poster": "poster.jpg", "release_info": "2024", "genres": ["Drama"]
                }],
                "progress": [{
                    "content_id": "tt1", "content_type": "series", "video_id": "tt1:1:2",
                    "season": 1, "episode": 2, "position": 120_000, "duration": 600_000,
                    "last_watched": 1_700_000_000_000i64
                }]
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(snapshot["watchlist"][0]["id"], "tt1");
    assert_eq!(snapshot["watchlist"][0]["inWatchlist"], true);
    assert_eq!(snapshot["continueWatching"][0]["name"], "Show");
    assert_eq!(snapshot["continueWatching"][0]["poster"], "poster.jpg");
    assert_eq!(snapshot["continueWatching"][0]["videoId"], "tt1:1:2");
    assert_eq!(snapshot["continueWatching"][0]["timeOffset"], 120);
    assert_eq!(snapshot["continueWatching"][0]["duration"], 600);
}

#[test]
fn series_episodes_collapse_to_the_latest_with_its_thumbnail() {
    let episode = |number: i64, at: i64| {
        json!({
            "content_id": "tt3", "content_type": "series", "video_id": format!("tt3:1:{number}"),
            "season": 1, "episode": number, "position": 60_000, "duration": 600_000,
            "last_watched": at
        })
    };
    let snapshot: Value = serde_json::from_str(
        &provider_library_snapshot_json(
            &json!({
                "library": [],
                "progress": [episode(1, 100), episode(2, 300), episode(3, 200)],
                "metas": {"tt3": {"name": "Show", "videos": [
                    {"id": "tt3:1:2", "season": 1, "episode": 2, "name": "Two", "thumbnail": "e2.jpg"}
                ]}}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(snapshot["continueWatching"].as_array().unwrap().len(), 1);
    assert_eq!(snapshot["continueWatching"][0]["videoId"], "tt3:1:2");
    assert_eq!(
        snapshot["continueWatching"][0]["lastEpisodeThumbnail"],
        "e2.jpg"
    );
    assert_eq!(snapshot["continueWatching"][0]["lastEpisodeName"], "Two");
    assert_eq!(snapshot["continueWatching"][0]["lastEpisodeNumber"], 2);
}

#[test]
fn unwatching_an_episode_deletes_its_season_key() {
    let requests: Value = serde_json::from_str(
        &write_requests_json(
            &json!({"profileIndex": 2, "nowMs": 5, "action": {"kind": "watched", "watched": false, "seriesId": "tt3", "meta": {"type": "series"}, "episodeInfos": [{"season": 1, "episode": 4}]}}).to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(requests[0]["rpc"], "sync_delete_watched_items");
    assert_eq!(
        requests[0]["body"]["p_keys"],
        json!([{"content_id": "tt3", "season": 1, "episode": 4}])
    );
}

#[test]
fn home_rows_follow_nuvio_order_and_drop_disabled_catalogs() {
    let layout: Value = serde_json::from_str(
        &home_layout_json(
            &json!({
                "addons": [{"transportUrl": "https://c/manifest.json", "manifest": {"id": "cinemeta"}}],
                "categories": [
                    {"id": "k1", "type": "movie", "catalogId": "top", "transportUrl": "https://c/manifest.json", "name": "Top"},
                    {"id": "k2", "type": "series", "catalogId": "top", "transportUrl": "https://c/manifest.json", "name": "Top"},
                    {"id": "col-a", "type": "collection", "name": "Studios"},
                    {"id": "other", "type": "movie", "catalogId": "x", "transportUrl": "https://x", "name": "X"}
                ],
                "items": [
                    {"order": 1, "enabled": true, "addon_id": "cinemeta", "type": "movie", "catalog_id": "top", "custom_title": "Popular", "is_collection": false},
                    {"order": 0, "enabled": true, "collection_id": "col-a", "is_collection": true},
                    {"order": 2, "enabled": false, "addon_id": "cinemeta", "type": "series", "catalog_id": "top", "is_collection": false}
                ]
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    let ids: Vec<&str> = layout
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["col-a", "k1", "other"]);
    assert_eq!(layout[1]["name"], "Popular");
}

#[test]
fn watched_history_marks_movies_and_single_episodes() {
    let snapshot: Value = serde_json::from_str(
        &provider_library_snapshot_json(
            &json!({
                "library": [],
                "progress": [],
                "watched": [
                    {"content_id": "tt1", "content_type": "movie"},
                    {"content_id": "tt2", "content_type": "series", "season": 1, "episode": 3}
                ]
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(snapshot["watched"]["tt1"], true);
    assert_eq!(snapshot["watched"]["tt2:1:3"], true);
    assert!(snapshot["watched"].get("tt2").is_none());
}

#[test]
fn progress_outside_the_library_takes_its_title_from_fetched_metas() {
    let snapshot: Value = serde_json::from_str(
        &provider_library_snapshot_json(
            &json!({
                "library": [],
                "progress": [{
                    "content_id": "tt2", "content_type": "movie",
                    "position": 60_000, "duration": 600_000,
                    "last_watched": 1_700_000_000_000i64
                }],
                "metas": {"tt2": {"name": "Film", "poster": "film.jpg"}}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(snapshot["continueWatching"][0]["name"], "Film");
    assert_eq!(snapshot["continueWatching"][0]["poster"], "film.jpg");
}

#[test]
fn progress_metadata_needs_are_unique_per_content() {
    let needs: Value = serde_json::from_str(
        &progress_meta_needs_json(
            &json!({
                "watchProgress": [
                    { "content_id": "tt0760437", "content_type": "series" },
                    { "content_id": "tt0760437", "content_type": "series" },
                    { "content_id": "tt12343534", "content_type": "series" }
                ],
                "library": []
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(needs.as_array().unwrap().len(), 2);
}

#[test]
fn progress_metadata_plan_keeps_series_episode_metadata_but_skips_complete_movie_cards() {
    let needs: Value = serde_json::from_str(
        &progress_meta_needs_json(
            &json!({
                "watchProgress": [
                    { "content_id": "show", "content_type": "series", "progress_key": "show_s1e2", "season": 1, "episode": 2 },
                    { "content_id": "movie", "content_type": "movie", "progress_key": "movie", "position": 20, "duration": 100 }
                ],
                "library": [
                    { "content_id": "show", "content_type": "series", "name": "Show", "poster": "poster" },
                    { "content_id": "movie", "content_type": "movie", "name": "Feature Film", "poster": "poster" }
                ]
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        needs,
        json!([{
            "contentId": "show",
            "contentType": "series",
            "progressKey": "show_s1e2"
        }])
    );
}
