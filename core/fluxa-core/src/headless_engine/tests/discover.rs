use super::super::*;
use serde_json::{Value, json};

#[test]
fn addon_search_discover_and_catalog_backbone_are_effect_driven() {
    let handle = create_headless_engine(r#"{"profile":{"activeProfileId":"p1"}}"#);

    let addon: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"addonInstallRequested","transportUrl":"https://addon.example/manifest.json","forceRefresh":true}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(addon["effects"][0]["type"], "fetchAddonManifest");
    assert_eq!(
        addon["effects"][0]["payload"]["transportUrl"],
        "https://addon.example/manifest.json"
    );

    let completed_addon: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": addon["effects"][0]["id"].as_str().unwrap(),
                "status": "ok",
                "value": {
                    "id": "addon.example",
                    "transportUrl": "https://addon.example/manifest.json",
                    "name": "Addon"
                }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        completed_addon["state"]["addons"]["installed"][0]["name"],
        "Addon"
    );

    let resource: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"addonResourceRequested","transportUrl":"https://addon.example/manifest.json","resource":"stream","contentType":"movie","id":"tt1","extra":{"search":"keep order"}}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(resource["effects"][0]["type"], "fetchAddonResource");
    assert_eq!(resource["effects"][0]["payload"]["resource"], "stream");
    assert_eq!(
        resource["effects"][0]["payload"]["extra"]["search"],
        "keep order"
    );

    let search: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"searchRequested","query":"matrix","language":"en"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(search["effects"][0]["type"], "runSearch");
    assert_eq!(search["effects"][0]["payload"]["profileId"], "p1");

    let discover: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"discoverRequested","contentType":"movie","filters":{"genre":"sci-fi"},"language":"en"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(discover["effects"][0]["type"], "runDiscover");
    assert_eq!(
        discover["effects"][0]["payload"]["filters"]["genre"],
        "sci-fi"
    );

    let page: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"catalogPageRequested","categoryId":"cat","transportUrl":"https://addon.example/manifest.json","contentType":"movie","catalogId":"top","skip":-10,"genre":null,"search":null}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(page["effects"][0]["type"], "fetchCatalogPage");
    assert_eq!(page["effects"][0]["payload"]["skip"], 0);
    assert!(destroy_headless_engine(handle));
}

#[test]
fn concurrent_catalog_filters_request_does_not_drop_discover_results() {
    let handle = create_headless_engine("{}");
    let discover: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"discoverRequested","contentType":"movie","filters":{"catalogKey":"top"}}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let effect_id = discover["effects"][0]["id"].as_str().unwrap().to_string();

    headless_engine_dispatch_json(
            handle,
            r#"{"type":"discoverCatalogFiltersRequested","contentType":"movie","selectedCatalogKey":"top"}"#,
        )
        .unwrap();

    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": { "results": [{"id": "tt1", "type": "movie", "name": "A Movie"}] }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(completed["state"]["discover"]["results"][0]["id"], "tt1");
    assert!(destroy_headless_engine(handle));
}

#[test]
fn discover_pages_continue_until_the_catalog_returns_no_new_items() {
    let handle = create_headless_engine(
        r#"{"discover":{"catalogs":[{"key":"top","transportUrl":"https://addon.example/manifest.json","id":"top","type":"movie"}]}}"#,
    );
    let discover: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"discoverRequested","contentType":"movie","filters":{"catalogKey":"top","extra":{"genre":"action"}}}"#,
            )
            .unwrap(),
        )
        .unwrap();
    let first_page: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": discover["effects"][0]["id"].as_str().unwrap(),
                "status": "ok",
                "value": { "results": [{ "id": "tt1" }] }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        first_page["state"]["discover"]["results"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(first_page["effects"].as_array().unwrap().is_empty());
    assert_eq!(first_page["state"]["discover"]["paging"]["nextSkip"], 1);
    assert_eq!(first_page["state"]["discover"]["paging"]["hasMore"], true);
    let page_request = |skip| {
        json!({
            "type": "discoverPageRequested",
            "transportUrl": "https://addon.example/manifest.json",
            "contentType": "movie",
            "catalogId": "top",
            "skip": skip,
            "genre": "action",
        })
    };
    let second_request: Value = serde_json::from_str(
        &headless_engine_dispatch_json(handle, &page_request(1).to_string()).unwrap(),
    )
    .unwrap();
    assert_eq!(second_request["effects"][0]["payload"]["skip"], 1);
    let duplicate: Value = serde_json::from_str(
        &headless_engine_dispatch_json(handle, &page_request(1).to_string()).unwrap(),
    )
    .unwrap();
    assert!(duplicate["effects"].as_array().unwrap().is_empty());
    let second_page: Value = serde_json::from_str(&headless_engine_complete_effect_json(
        handle,
        &json!({"effectId": second_request["effects"][0]["id"], "status":"ok", "value":{"items":[{"id":"tt2"}]}}).to_string(),
    ).unwrap()).unwrap();
    assert_eq!(
        second_page["state"]["discover"]["results"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(second_page["state"]["discover"]["paging"]["nextSkip"], 2);
    assert_eq!(
        second_page["state"]["discover"]["resultSources"]["movie:tt2"]["catalogId"],
        "top"
    );
    let third_request: Value = serde_json::from_str(
        &headless_engine_dispatch_json(handle, &page_request(2).to_string()).unwrap(),
    )
    .unwrap();
    let third_page: Value = serde_json::from_str(&headless_engine_complete_effect_json(
        handle,
        &json!({"effectId": third_request["effects"][0]["id"], "status":"ok", "value":{"items":[]}}).to_string(),
    ).unwrap()).unwrap();
    assert_eq!(
        third_page["state"]["discover"]["results"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(third_page["state"]["discover"]["paging"]["hasMore"], false);
    let finished: Value = serde_json::from_str(
        &headless_engine_dispatch_json(handle, &page_request(2).to_string()).unwrap(),
    )
    .unwrap();
    assert!(finished["effects"].as_array().unwrap().is_empty());
    assert!(destroy_headless_engine(handle));
}

#[test]
fn discover_effect_carries_the_selected_catalog_io_details_for_every_host() {
    let addons = json!([{
        "transportUrl": "https://addon.example/manifest.json",
        "manifest": {
            "id": "com.example.catalog",
            "resources": ["catalog"],
            "catalogs": [{"type": "movie", "id": "popular", "name": "Popular"}]
        }
    }]);
    let handle = create_headless_engine(
        &json!({
            "addons": {"installed": addons.clone()},
            "profile": {"active": {"id": "p1"}, "activeProfileId": "p1"}
        })
        .to_string(),
    );
    let filters = serde_json::from_str::<Value>(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"discoverCatalogFiltersRequested","contentType":"movie"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert!(filters["effects"][0]["payload"]["profile"].is_object());
    for core_owned_field in ["contentType", "selectedCatalogKey", "catalogs", "addons"] {
        assert!(
            filters["effects"][0]["payload"]
                .get(core_owned_field)
                .is_none()
        );
    }
    let completed = serde_json::from_str::<Value>(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": filters["effects"][0]["id"],
                "status": "ok",
                "value": {"addons": addons}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        completed["state"]["discover"]["catalogs"][0]["id"],
        "popular"
    );
    let catalog_key = completed["state"]["discover"]["catalogs"][0]["key"]
        .as_str()
        .unwrap();
    let discover = serde_json::from_str::<Value>(
        &headless_engine_dispatch_json(
            handle,
            &json!({
                "type": "discoverRequested",
                "contentType": "movie",
                "filters": {
                    "catalogKey": catalog_key,
                    "extra": {"genre": "Action", "search": "aliens"}
                }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    let payload = &discover["effects"][0]["payload"];
    assert_eq!(payload["filters"]["catalogKey"], catalog_key);
    assert_eq!(
        payload["filters"]["transportUrl"],
        "https://addon.example/manifest.json"
    );
    assert_eq!(payload["filters"]["catalogId"], "popular");
    assert_eq!(payload["filters"]["catalogType"], "movie");
    assert_eq!(payload["filters"]["extra"]["genre"], "Action");
    assert_eq!(payload["filters"]["extra"]["search"], "aliens");
    assert!(destroy_headless_engine(handle));
}

#[test]
fn one_discover_request_loads_catalogs_then_runs_the_selected_catalog() {
    let addons = json!([{
        "transportUrl": "https://addon.example/manifest.json",
        "manifest": {
            "id": "com.example.catalog",
            "resources": ["catalog"],
            "catalogs": [{"type": "movie", "id": "popular", "name": "Popular"}]
        }
    }]);
    let handle =
        create_headless_engine(&json!({"addons": {"installed": addons.clone()}}).to_string());
    let requested = serde_json::from_str::<Value>(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"discoverRequested","loadCatalogFilters":true,"contentType":"movie","filters":{"catalogKey":null,"extra":{"genre":"Action","search":"aliens"}}}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        requested["effects"][0]["type"],
        "readDiscoverCatalogFilters"
    );

    let completed = serde_json::from_str::<Value>(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": requested["effects"][0]["id"],
                "status": "ok",
                "value": {"addons": addons}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(completed["effects"][0]["type"], "runDiscover");
    assert_eq!(
        completed["effects"][0]["payload"]["filters"]["extra"]["genre"],
        "Action"
    );
    assert_eq!(
        completed["effects"][0]["payload"]["filters"]["extra"]["search"],
        "aliens"
    );
    assert_eq!(
        completed["effects"][0]["payload"]["filters"]["catalogId"],
        "popular"
    );
    assert!(destroy_headless_engine(handle));
}

#[test]
fn empty_discover_content_type_defaults_to_movie_and_loads_catalogs() {
    let addons = json!([{
        "transportUrl": "https://addon.example/manifest.json",
        "manifest": {
            "id": "com.example.catalog",
            "resources": ["catalog"],
            "catalogs": [{"type": "movie", "id": "popular", "name": "Popular"}]
        }
    }]);
    let handle = create_headless_engine(r#"{}"#);
    let requested: Value = serde_json::from_str(&headless_engine_dispatch_json(
        handle,
        r#"{"type":"discoverRequested","loadCatalogFilters":true,"contentType":"","filters":{"catalogKey":null,"extra":{}}}"#,
    ).unwrap()).unwrap();
    assert_eq!(requested["state"]["discover"]["contentType"], "movie");
    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": requested["effects"][0]["id"],
                "status": "ok",
                "value": {"addons": addons}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        completed["state"]["discover"]["catalogs"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(completed["effects"][0]["type"], "runDiscover");
    assert!(destroy_headless_engine(handle));
}

#[test]
fn discover_filter_options_and_genres_are_derived_by_core_from_addon_descriptors() {
    let addons = json!([{
        "transportUrl": "https://addon.example/manifest.json",
        "manifest": {
            "id": "com.example.catalog",
            "resources": ["catalog"],
            "catalogs": [
                {
                    "type": "movie",
                    "id": "required-genre",
                    "name": "Required genre",
                    "extra": [{"name": "genre", "isRequired": true, "options": ["Action", "Drama"]}]
                },
                {
                    "type": "movie",
                    "id": "optional-genre",
                    "name": "Optional genre",
                    "extra": [{"name": "genre", "options": ["Comedy"]}]
                }
            ]
        }
    }]);
    let handle = create_headless_engine("{}");
    let requested = serde_json::from_str::<Value>(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"discoverCatalogFiltersRequested","contentType":"movie","selectedCatalogKey":"discover:https_addon.example_manifest.json:movie:optional-genre"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let completed = serde_json::from_str::<Value>(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": requested["effects"][0]["id"],
                "status": "ok",
                "value": {"addons": addons}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        completed["state"]["discover"]["catalogs"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        completed["state"]["discover"]["genres"][0]["id"],
        Value::Null
    );
    assert_eq!(completed["state"]["discover"]["genres"][1]["id"], "Comedy");
    let required_key = "discover:https_addon.example_manifest.json:movie:required-genre";
    let required_request = serde_json::from_str::<Value>(
        &headless_engine_dispatch_json(
            handle,
            &json!({
                "type": "discoverRequested",
                "contentType": "movie",
                "filters": {"catalogKey": required_key, "extra": {}}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        required_request["state"]["discover"]["filters"]["extra"]["genre"],
        "Action"
    );
    assert!(destroy_headless_engine(handle));
}
