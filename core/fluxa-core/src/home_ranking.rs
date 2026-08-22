mod billboard;
mod bootstrap;
mod folders;
mod helpers;
mod ranking;

pub(crate) use billboard::{
    billboard_candidate_score_json, billboard_editorial_match_score_json,
    billboard_has_backdrop_json, billboard_identity_key_json, billboard_normalized_title,
    billboard_visual_score_json, build_billboard_pool_json, normalize_home_catalog_items_json,
};
pub(crate) use bootstrap::home_hero_plan_json;
pub(crate) use folders::{
    build_home_collection_shelves_json, folder_page_state_json, folder_source_page_plan_json,
    merge_folder_sources_json,
};
pub(crate) use ranking::{
    curate_home_items_json, home_overlap_ratio_json, home_personalization_score_json,
    home_prioritize_rows_json, optimize_home_rows_json,
};
#[cfg(test)]
mod tests {
    use super::folders::resolve_folder_catalog_sources;
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn billboard_policy_scores_match_the_shared_rules() {
        let meta = json!({
            "id": "tt1",
            "type": "series",
            "rank": 1,
            "imdbRating": "8.5",
            "reason": "EDITORIAL_SPOTLIGHT",
            "poster": "https://image.example/poster.jpg",
            "background": "https://image.example/background.jpg",
            "logo": "https://image.example/logo.png",
            "description": "Description",
            "releaseInfo": "2025",
        });
        let args = json!({ "meta": meta, "daysSinceRelease": 10 });

        assert_eq!(
            billboard_candidate_score_json(&args.to_string()),
            Some(2127)
        );
        assert_eq!(billboard_visual_score_json(&args.to_string()), Some(470));
        assert_eq!(
            billboard_editorial_match_score_json(
                &json!({ "meta": args["meta"], "minYear": 2020 }).to_string()
            ),
            Some(738)
        );
        assert_eq!(
            billboard_identity_key_json(&args.to_string()),
            Some("series:tt1".to_string())
        );
        assert_eq!(billboard_normalized_title("Çığ Şöw"), "cig sow");
    }

    #[test]
    fn home_collection_shelves_filter_hidden_collections_and_resolve_catalog_sources() {
        let profile = json!({
            "libraryCollections": [
                {
                    "id": "col1",
                    "title": "My Collection",
                    "showOnHome": true,
                    "pinToTop": true,
                    "folders": [
                        {
                            "id": "f1",
                            "title": "Action",
                            "coverImageUrl": "https://img.example/cover.jpg",
                            "focusGifUrl": "https://img.example/focus.gif",
                            "focusGifEnabled": false,
                            "catalogSources": [{ "catalogId": "top", "type": "movie" }],
                        }
                    ],
                },
                {
                    "id": "col2",
                    "title": "Not Shown",
                    "showOnHome": false,
                    "folders": [{ "id": "f2", "title": "Hidden", "catalogId": "top" }],
                },
            ],
        });
        let addons = json!([
            {
                "transportUrl": "https://addon.example/manifest.json",
                "manifest": { "id": "addon.example", "catalogs": [{ "id": "top", "type": "movie" }] },
            }
        ]);

        let result = build_home_collection_shelves_json(&profile.to_string(), &addons.to_string())
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .expect("shelves");

        assert!(result["regularShelves"].as_array().unwrap().is_empty());
        let pinned = result["pinnedShelves"].as_array().unwrap();
        assert_eq!(pinned.len(), 1);
        assert_eq!(pinned[0]["id"], "col1");
        assert_eq!(pinned[0]["items"][0]["id"], "f1");
        assert_eq!(
            pinned[0]["items"][0]["poster"],
            "https://img.example/cover.jpg"
        );
        assert_eq!(
            pinned[0]["items"][0]["focusGifUrl"],
            "https://img.example/focus.gif"
        );

        let hidden = result["hiddenFolderCategories"].as_array().unwrap();
        assert_eq!(hidden.len(), 1);
        assert_eq!(hidden[0]["id"], "f1");
        assert_eq!(
            hidden[0]["catalogSources"][0]["transportUrl"],
            "https://addon.example/manifest.json"
        );
    }

    #[test]
    fn modern_nuvio_sources_take_precedence_over_legacy_catalog_sources() {
        let folder = json!({
            "sources": [{
                "provider": "addon",
                "addonId": "modern.addon",
                "type": "series",
                "catalogId": "modern",
                "genre": "Drama",
            }],
            "catalogSources": [{
                "addonId": "legacy.addon",
                "type": "movie",
                "catalogId": "legacy",
            }],
        });
        let addons = json!([
            {
                "transportUrl": "https://modern.example/manifest.json",
                "manifest": { "id": "modern.addon", "catalogs": [{ "id": "modern", "type": "series" }] },
            },
            {
                "transportUrl": "https://legacy.example/manifest.json",
                "manifest": { "id": "legacy.addon", "catalogs": [{ "id": "legacy", "type": "movie" }] },
            },
        ]);

        let resolved = resolve_folder_catalog_sources(
            folder.as_object().expect("folder"),
            &addons.to_string(),
        );

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0]["catalogId"], "modern");
        assert_eq!(resolved[0]["type"], "series");
        assert_eq!(resolved[0]["genre"], "Drama");
        assert_eq!(
            resolved[0]["transportUrl"],
            "https://modern.example/manifest.json"
        );
    }

    #[test]
    fn modern_nuvio_remote_sources_are_preserved() {
        let folder = json!({
            "sources": [{
                "provider": "trakt",
                "traktListId": 123,
                "mediaType": "TV",
                "sortBy": "rank",
                "sortHow": "asc",
            }],
            "catalogSources": [{ "catalogId": "legacy", "type": "movie" }],
        });

        let resolved = resolve_folder_catalog_sources(folder.as_object().expect("folder"), "[]");

        assert_eq!(resolved, vec![folder["sources"][0].clone()]);
    }

    #[test]
    fn empty_modern_sources_fall_back_to_legacy_catalog_sources() {
        let folder = json!({
            "sources": [],
            "catalogSources": [{
                "addonId": "addon.example",
                "catalogId": "top",
                "type": "movie",
            }],
        });
        let addons = json!([{
            "transportUrl": "https://addon.example/manifest.json",
            "manifest": { "id": "addon.example", "catalogs": [{ "id": "top", "type": "movie" }] },
        }]);

        let resolved = resolve_folder_catalog_sources(
            folder.as_object().expect("folder"),
            &addons.to_string(),
        );

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0]["catalogId"], "top");
    }

    #[test]
    fn hero_plan_requests_logo_for_catalog_item_missing_one() {
        let plan: Value = serde_json::from_str(
            &home_hero_plan_json(
                &json!({
                    "categories": [{
                        "type": "movie",
                        "items": [{
                            "id": "tmdb:1",
                            "type": "movie",
                            "background": "https://image.example/bg.jpg",
                        }],
                    }],
                    "prefs": {},
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(plan["logoTargets"][0]["id"], "tmdb:1");
        assert_eq!(plan["billboard"]["logo"], Value::Null);
    }

    #[test]
    fn hero_plan_merges_fetched_logo_and_skips_target_once_resolved() {
        let plan: Value = serde_json::from_str(
            &home_hero_plan_json(
                &json!({
                    "categories": [{
                        "type": "movie",
                        "items": [{
                            "id": "tmdb:1",
                            "type": "movie",
                            "background": "https://image.example/bg.jpg",
                        }],
                    }],
                    "prefs": { "tmdbApiKey": "KEY" },
                    "fetchedLogos": { "tmdb:1": "https://image.example/logo.png" },
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(plan["billboard"]["logo"], "https://image.example/logo.png");
        assert_eq!(plan["logoTargets"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn hero_plan_requests_logo_for_addon_item_without_tmdb_api_key() {
        let plan: Value = serde_json::from_str(
            &home_hero_plan_json(
                &json!({
                    "categories": [{
                        "type": "movie",
                        "items": [{
                            "id": "tt1",
                            "type": "movie",
                            "background": "https://image.example/bg.jpg",
                        }],
                    }],
                    "prefs": {},
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(plan["logoTargets"][0]["id"], "tt1");
    }
}
