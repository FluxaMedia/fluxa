use super::folders::resolve_folder_catalog_sources;
use super::*;
use serde_json::{Value, json};

#[test]
fn home_metadata_feed_plan_shares_order_and_unknown_selection_fallback() {
    let feeds = json!([
        {"key":"a", "label":"A"},
        {"key":"b", "label":"B"},
        {"key":"c", "label":"C"}
    ]);
    let planned: Value = serde_json::from_str(
        &home_metadata_feed_plan_json(
            &json!({"feeds":feeds.clone(), "order":["c", "a"], "selectedKeys":["removed"]})
                .to_string(),
        )
        .expect("home feed plan"),
    )
    .expect("valid home feed plan");
    let keys = planned
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|feed| feed["key"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(keys, vec!["c", "a", "b"]);

    let empty_selection_defaults_to_all: Value = serde_json::from_str(
        &home_metadata_feed_plan_json(
            &json!({"feeds":feeds, "order":[], "selectedKeys":[]}).to_string(),
        )
        .expect("empty selection plan"),
    )
    .expect("valid empty plan");
    assert_eq!(
        empty_selection_defaults_to_all,
        json!([
            {"key":"a", "label":"A"},
            {"key":"b", "label":"B"},
            {"key":"c", "label":"C"}
        ])
    );
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

    let resolved =
        resolve_folder_catalog_sources(folder.as_object().expect("folder"), &addons.to_string());

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

    let resolved =
        resolve_folder_catalog_sources(folder.as_object().expect("folder"), &addons.to_string());

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

#[test]
fn hero_plan_uses_selected_catalog_items_not_collection_tiles_or_fallback_billboard() {
    let collection_tile = json!({
        "id": "trending-folder",
        "type": "catalog_folder",
        "name": "Trending",
        "background": "https://image.example/collection-collage.webp",
    });
    let plan: Value = serde_json::from_str(
        &home_hero_plan_json(
            &json!({
                "categories": [
                    {"id":"my-collection", "type":"collection", "items":[collection_tile.clone()]},
                    {"id":"selected-feed", "type":"movie", "items":[{"id":"tt123", "type":"movie", "background":"https://image.example/catalog-backdrop.jpg"}]},
                ],
                "billboard": collection_tile,
                "prefs": {"heroFeedToggles":["selected-feed"]},
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(plan["billboard"]["id"], "tt123");
    assert!(
        plan["slides"]
            .as_array()
            .unwrap()
            .iter()
            .all(|slide| slide["id"] != "trending-folder")
    );

    let collections_only: Value = serde_json::from_str(
        &home_hero_plan_json(
            &json!({
                "categories": [{"id":"my-collection", "type":"collection", "items":[{"id":"folder", "type":"catalog_folder", "background":"https://image.example/folder.webp"}]}],
                "billboard": {"id":"folder", "type":"catalog_folder", "background":"https://image.example/folder.webp"},
                "prefs": {},
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(collections_only["billboard"].is_null());
    assert!(collections_only["slides"].as_array().unwrap().is_empty());
}

#[test]
fn hero_plan_recovers_when_saved_feed_keys_are_stale() {
    let plan: Value = serde_json::from_str(
        &home_hero_plan_json(
            &json!({
                "categories": [{
                    "id": "current-feed",
                    "type": "movie",
                    "items": [{"id":"tt123", "background":"https://image.example/title.webp"}],
                }],
                "prefs": {"heroFeedToggles": ["removed-addon-feed"]},
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(plan["billboard"]["id"], "tt123");
}
