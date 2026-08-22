mod addon_priority;
mod collections;
mod delta_state;
mod export_push;
mod helpers;
mod profiles;
mod progress_sync;
mod reconciliation;

pub(crate) use addon_priority::{addon_state_json, sort_addons_by_priority_json};
pub(crate) use collections::map_collections_json;
pub(crate) use delta_state::{
    apply_delta_sync_json, apply_progress_sync_json, delta_sync_request_plan_json,
    progress_sync_request_plan_json,
};
pub(crate) use export_push::{
    collection_request_json, export_push_plan_json, library_item_request_json,
    playback_progress_request_json, watched_items_request_json,
};
pub(crate) use profiles::build_local_profiles_json;
pub(crate) use progress_sync::{
    import_merge_plan_json, library_to_watchlist_json, progress_meta_needs_json,
    resolve_continue_watching_json,
};
pub(crate) use reconciliation::{addon_reconciliation_plan_json, library_mutation_plan_json};
#[cfg(test)]
mod tests {
    use super::helpers::iso_from_ms;
    use super::*;
    use serde_json::{Value, json};

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
        assert_eq!(result[0]["nuvioProfileUpdatedAt"], json!("2026-08-18T00:00:00Z"));
    }
}
