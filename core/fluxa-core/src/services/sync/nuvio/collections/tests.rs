use super::map_collections_json;
use serde_json::{Value, json};

#[test]
fn maps_nuvio_collections_to_the_canonical_profile_shape() {
    let result: Value = serde_json::from_str(
        &map_collections_json(
            &json!({
                "collections": [{
                    "id": "c1",
                    "title": "Action",
                    "folders": [{
                        "id": "f1",
                        "title": "Movies",
                        "coverImageUrl": "https://image.example/cover.jpg",
                        "sources": [
                            { "provider": "addon", "addonId": "addon.example", "catalogId": "top", "type": "movie" },
                            { "provider": "trakt", "traktListId": 42, "mediaType": "MOVIE" }
                        ]
                    }]
                }]
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    let collection = &result[0];
    assert_eq!(collection["showOnHome"], true);
    assert_eq!(collection["viewMode"], "TABBED_GRID");
    assert_eq!(collection["showAllTab"], true);
    assert_eq!(collection["focusGlowEnabled"], true);
    let folder = &collection["folders"][0];
    assert_eq!(folder["imageUrl"], "https://image.example/cover.jpg");
    assert_eq!(folder["shape"], "poster");
    assert_eq!(folder["catalogSources"].as_array().unwrap().len(), 1);
    assert_eq!(folder["sources"].as_array().unwrap().len(), 2);
}

#[test]
fn generates_stable_ids_and_titles_for_incomplete_collections() {
    let result: Value = serde_json::from_str(
        &map_collections_json(
            &json!({
                "collections": [{
                    "folders": [{}]
                }]
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(result[0]["id"], "nuvio_collection_0");
    assert_eq!(result[0]["title"], "Collection 1");
    assert_eq!(result[0]["folders"][0]["id"], "nuvio_collection_0_folder_0");
    assert_eq!(result[0]["folders"][0]["title"], "Folder 1");
}

#[test]
fn preserves_profile_scoped_fallback_collection_ids() {
    let result: Value = serde_json::from_str(
        &map_collections_json(
            &json!({
                "profileIndex": 7,
                "collections": [{"title": "My Action List"}]
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(result[0]["id"], "nuvio_7_my_action_list");
}

#[test]
fn falls_back_to_catalog_sources_when_modern_sources_are_empty() {
    let result: Value = serde_json::from_str(
        &map_collections_json(
            &json!({
                "collections": [{
                    "id": "c1",
                    "title": "Action",
                    "folders": [{
                        "id": "f1",
                        "title": "Movies",
                        "sources": [],
                        "catalogSources": [{"catalogId": "top", "type": "movie"}]
                    }]
                }]
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        result[0]["folders"][0]["catalogSources"][0]["catalogId"],
        "top"
    );
}
