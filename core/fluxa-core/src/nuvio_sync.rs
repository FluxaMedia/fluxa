mod addon_priority;
mod collections;
mod delta_state;
mod export_push;
mod helpers;
mod home_layout;
mod plugin_content;
mod profiles;
mod progress_sync;
mod reconciliation;
mod write_requests;

pub(crate) use addon_priority::{
    addon_snapshot_plan_json, addon_state_json, sort_addons_by_priority_json,
};
pub(crate) use collections::map_collections_json;
pub(crate) use delta_state::{
    apply_delta_sync_json, apply_progress_sync_json, delta_sync_request_plan_json,
    progress_sync_request_plan_json,
};
pub(crate) use export_push::{
    collection_request_json, export_push_plan_json, library_item_request_json,
    playback_progress_request_json, watched_items_request_json,
};
pub(crate) use helpers::canonical_content_type;
pub(crate) use home_layout::home_layout_json;
pub(crate) use plugin_content::{candidate_content_types, plugin_content_id, plugin_content_type};
pub(crate) use profiles::{build_local_profiles_json, effective_profile_scopes_json};
pub(crate) use progress_sync::{
    import_merge_plan_json, library_to_watchlist_json, progress_meta_needs_json,
    progress_presentation_json, provider_library_snapshot_json, resolve_continue_watching_json,
};
pub(crate) use reconciliation::{addon_reconciliation_plan_json, library_mutation_plan_json};
pub(crate) use write_requests::write_requests_json;
#[cfg(test)]
mod tests {
    use super::helpers::{canonical_content_type, iso_from_ms};
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn canonical_content_type_maps_nuvio_aliases_to_server_types() {
        for value in ["series", " show ", "TV", "anime"] {
            assert_eq!(canonical_content_type(value), "series");
        }
        assert_eq!(canonical_content_type("movie"), "movie");
        assert_eq!(canonical_content_type(""), "movie");
    }

    #[test]
    fn addon_snapshot_fetches_only_new_manifests_in_server_order() {
        let rows = json!([
            {"url": "https://b/manifest.json", "enabled": false, "sort_order": 1},
            {"url": "https://a/manifest.json", "enabled": true, "sort_order": 0}
        ]);
        let cached = json!({"rows": [], "addons": [{"transportUrl": "https://b/manifest.json", "manifest": {"id": "b"}}]});
        let plan: Value = serde_json::from_str(
            &addon_snapshot_plan_json(&json!({"rows": rows, "snapshot": cached}).to_string())
                .unwrap(),
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
            serde_json::from_str::<Value>(
                &effective_profile_scopes_json(&input.to_string()).unwrap(),
            )
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

    fn merge(args: Value) -> Value {
        serde_json::from_str(&import_merge_plan_json(&args.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn watched_episode_removes_its_progress_entry() {
        let result = merge(json!({
            "progress": {},
            "watched": {},
            "library": [],
            "addonMetas": {},
            "watchProgress": [{
                "content_id": "tt1", "content_type": "series", "video_id": "tt1:1:2",
                "position": 500_000, "duration": 1_000_000,
                "season": 1, "episode": 2, "last_watched": 1_700_000_000_000i64,
            }],
            "watchHistory": [
                { "content_id": "tt1", "content_type": "series", "season": 1, "episode": 3, "watched_at": 1_700_000_100_000i64 },
            ],
        }));
        assert!(result["progress"]["tt1"].is_object());

        let result = merge(json!({
            "progress": {},
            "watched": { "tt1:1:2": true },
            "library": [],
            "addonMetas": {},
            "watchProgress": [],
            "watchHistory": [],
        }));
        assert_eq!(result["watched"]["tt1:1:2"], json!(true));
    }

    #[test]
    fn active_remote_progress_clears_conflicting_watched_flags() {
        let result = merge(json!({
            "progress": {},
            "watched": { "tt1:1:2": true, "tt9": true },
            "library": [],
            "addonMetas": {},
            "watchProgress": [{
                "content_id": "tt1", "content_type": "series", "video_id": "vid1",
                "position": 500_000, "duration": 1_000_000,
                "season": 1, "episode": 2, "last_watched": 1_700_000_000_000i64,
            }],
            "watchHistory": [],
        }));
        assert!(result["watched"].get("tt1:1:2").is_none());
        assert_eq!(result["watched"]["tt9"], json!(true));
    }

    #[test]
    fn resolved_up_next_saved_at_ignores_history_watched_at() {
        let result = merge(json!({
            "progress": {},
            "watched": {},
            "library": [],
            "addonMetas": {},
            "watchProgress": [{
                "content_id": "tt1", "content_type": "series", "video_id": "tt1:2:1",
                "position": 0, "duration": 1_000_000,
                "season": 2, "episode": 1, "last_watched": 1_700_000_000_000i64,
            }],
            "watchHistory": [
                { "content_id": "tt1", "content_type": "series", "season": 1, "episode": 9, "watched_at": 1_700_000_500_000i64 },
            ],
        }));
        let entry = &result["progress"]["tt1"];
        assert_eq!(entry["continueWatchingBadge"], json!("upNext"));
        assert_eq!(entry["savedAt"], json!(iso_from_ms(1_700_000_000_000)));
    }

    #[test]
    fn resolved_nuvio_progress_targets_the_following_episode() {
        let result = merge(json!({
            "progress": {},
            "watched": {},
            "library": [],
            "addonMetas": {
                "tt0760437": {
                    "videos": [
                        { "id": "tt0760437:1:2", "season": 1, "episode": 2, "title": "Washington B.C." },
                        { "id": "tt0760437:1:3", "season": 1, "episode": 3, "title": "The Krakken" }
                    ]
                }
            },
            "watchProgress": [{
                "content_id": "tt0760437", "content_type": "series", "video_id": "tt0760437:1:2",
                "position": 1_000, "duration": 1_000,
                "season": 1, "episode": 2, "last_watched": 1_700_000_000_000i64
            }],
            "watchHistory": []
        }));
        let entry = &result["progress"]["tt0760437"];
        assert_eq!(entry["lastVideoId"], json!("tt0760437:1:3"));
        assert_eq!(entry["lastEpisodeNumber"], json!(3));
        assert_eq!(entry["lastEpisodeName"], json!("The Krakken"));
        assert_eq!(entry["continueWatchingBadge"], json!("upNext"));
    }

    #[test]
    fn live_continue_watching_sync_rolls_a_finished_episode_to_the_next_one() {
        let resolved: Value = serde_json::from_str(
            &resolve_continue_watching_json(
                &json!({
                    "progress": [{
                        "content_id": "tt0760437", "content_type": "series", "video_id": "tt0760437:1:2",
                        "position": 1_000, "duration": 1_000,
                        "season": 1, "episode": 2, "last_watched": 1_700_000_000_000i64
                    }],
                    "addonMetas": {
                        "tt0760437": {
                            "videos": [
                                { "id": "tt0760437:1:2", "season": 1, "episode": 2, "title": "Washington B.C." },
                                { "id": "tt0760437:1:3", "season": 1, "episode": 3, "title": "The Krakken" }
                            ]
                        }
                    }
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        let entry = &resolved[0];
        assert_eq!(entry["video_id"], json!("tt0760437:1:3"));
        assert_eq!(entry["season"], json!(1));
        assert_eq!(entry["episode"], json!(3));
        assert_eq!(entry["position"], json!(0));
        assert_eq!(entry["duration"], json!(0));
    }

    #[test]
    fn live_continue_watching_sync_rolls_a_finished_season_finale_into_the_next_season() {
        let resolved: Value = serde_json::from_str(
            &resolve_continue_watching_json(
                &json!({
                    "progress": [{
                        "content_id": "tt1", "content_type": "series", "video_id": "tt1:1:10",
                        "position": 1_500_000, "duration": 1_500_000,
                        "season": 1, "episode": 10, "last_watched": 1_700_000_000_000i64
                    }],
                    "addonMetas": {
                        "tt1": {
                            "videos": [
                                { "id": "tt1:1:10", "season": 1, "episode": 10, "title": "Season 1 Finale" },
                                { "id": "tt1:2:1", "season": 2, "episode": 1, "title": "Season 2 Premiere" }
                            ]
                        }
                    }
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        let entry = &resolved[0];
        assert_eq!(entry["video_id"], json!("tt1:2:1"));
        assert_eq!(entry["season"], json!(2));
        assert_eq!(entry["episode"], json!(1));
    }

    #[test]
    fn live_continue_watching_sync_leaves_genuine_in_progress_rows_untouched() {
        let resolved: Value = serde_json::from_str(
            &resolve_continue_watching_json(
                &json!({
                    "progress": [{
                        "content_id": "tt6741278", "content_type": "series", "video_id": "tt6741278:1:2",
                        "position": 2_188_000, "duration": 2_667_000,
                        "season": 1, "episode": 2, "last_watched": 1_786_309_465_762i64
                    }],
                    "addonMetas": {}
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        let entry = &resolved[0];
        assert_eq!(entry["video_id"], json!("tt6741278:1:2"));
        assert_eq!(entry["position"], json!(2_188_000));
    }

    #[test]
    fn live_continue_watching_sync_drops_a_finished_series_finale_with_no_next_episode() {
        let resolved: Value = serde_json::from_str(
            &resolve_continue_watching_json(
                &json!({
                    "progress": [{
                        "content_id": "tt9", "content_type": "series", "video_id": "tt9:1:1",
                        "position": 1_000, "duration": 1_000,
                        "season": 1, "episode": 1, "last_watched": 1
                    }],
                    "addonMetas": {
                        "tt9": { "videos": [{ "id": "tt9:1:1", "season": 1, "episode": 1 }] }
                    }
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(resolved.as_array().unwrap().is_empty());
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

    #[test]
    fn missing_history_keeps_local_watched_untouched() {
        let result = merge(json!({
            "progress": {},
            "watched": { "vid1": true },
            "library": [],
            "addonMetas": {},
            "watchProgress": [{
                "content_id": "tt1", "content_type": "movie", "video_id": "vid1",
                "position": 500_000, "duration": 1_000_000, "last_watched": 1_700_000_000_000i64,
            }],
            "watchHistory": null,
        }));
        assert_eq!(result["watched"]["vid1"], json!(true));
        assert!(result["progress"].get("tt1").is_none());
    }

    #[test]
    fn mid_progress_entry_is_not_marked_up_next() {
        let result = merge(json!({
            "progress": {},
            "watched": {},
            "library": [{ "content_id": "tt1", "name": "Show", "poster": "p.jpg" }],
            "addonMetas": {},
            "watchProgress": [{
                "content_id": "tt1", "content_type": "movie", "video_id": "vid1",
                "position": 600_000, "duration": 1_200_000, "last_watched": 1_700_000_000_000i64,
            }],
            "watchHistory": [],
        }));
        let entry = &result["progress"]["tt1"];
        assert!(entry.get("continueWatchingBadge").is_none());
        assert_eq!(entry["timeOffset"], json!(600));
        assert_eq!(entry["meta"]["name"], json!("Show"));
        assert_eq!(entry["meta"]["poster"], json!("p.jpg"));
    }

    #[test]
    fn request_payloads_preserve_episode_progress_and_catalog_sources() {
        let progress: Value = serde_json::from_str(&playback_progress_request_json(&json!({"meta":{"id":"tt1","type":"series"},"videoId":"tt1:2:3","position":400,"duration":1000,"watchedAt":42}).to_string()).unwrap()).unwrap();
        assert_eq!(progress["progress_key"], "tt1_s2e3");
        let collection: Value = serde_json::from_str(&collection_request_json(&json!({"id":"c","title":"C","folders":[{"id":"f","title":"F","catalogSources":[{"addonId":"a","catalogId":"top","type":"movie"}]}]}).to_string()).unwrap()).unwrap();
        assert_eq!(collection["folders"][0]["sources"][0]["provider"], "addon");
    }

    #[test]
    fn addon_reconciliation_preserves_core_addon_state_rules() {
        let plan: Value = serde_json::from_str(&addon_reconciliation_plan_json(&json!({
            "current": [{"id":"old","url":"https://old"},{"id":"keep","url":"https://keep"}],
            "desired": [{"url":"https://keep","enabled":false},{"url":"https://new"}],
            "userId":"user", "profileId":2,
        }).to_string()).unwrap()).unwrap();
        assert_eq!(plan["deleteIds"], json!(["old"]));
        assert_eq!(plan["updates"][0]["payload"]["enabled"], false);
        assert_eq!(plan["creates"][0]["profile_id"], 2);
    }

    #[test]
    fn imported_profile_uses_its_nuvio_avatar_catalog_entry() {
        let result: Value = serde_json::from_str(&build_local_profiles_json(&json!({
            "sessionProfile": {"id":"local","nuvioUserId":"user","nuvioEmail":"user@example.com","nuvioAccessToken":"token"},
            "nuvioProfiles": [{"profile_index":1,"name":"Primary","avatar_id":"avatar-1","avatar_url":null,"pin_enabled":true,"pin_locked_until":null,"updated_at":"2026-08-18T00:00:00Z"}],
            "avatarCatalog": [{"id":"avatar-1","storage_path":"profiles/avatar-1.png"}],
            "existingProfiles": [{"id":"local","nuvioUserId":"user","nuvioProfileIndex":1}],
        }).to_string()).unwrap()).unwrap();
        assert_eq!(
            result[0]["avatarUrl"],
            json!("https://api.nuvio.tv/storage/v1/object/public/avatars/profiles/avatar-1.png")
        );
        assert_eq!(result[0]["nuvioPinEnabled"], json!(true));
        assert_eq!(
            result[0]["nuvioProfileUpdatedAt"],
            json!("2026-08-18T00:00:00Z")
        );
    }
}
