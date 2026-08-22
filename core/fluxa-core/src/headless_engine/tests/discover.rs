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
fn discover_prefetches_two_pages_in_one_round_trip() {
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
    let effects = first_page["effects"].as_array().unwrap();
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[0]["type"], "fetchDiscoverPage");
    assert_eq!(effects[0]["payload"]["skip"], 20);
    assert_eq!(effects[1]["type"], "fetchDiscoverPage");
    assert_eq!(effects[1]["payload"]["skip"], 40);

    let second_page: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effects[0]["id"].as_str().unwrap(),
                "status": "ok",
                "value": { "items": [{ "id": "tt2" }] }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        second_page["state"]["discover"]["results"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let third_page: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effects[1]["id"].as_str().unwrap(),
                "status": "ok",
                "value": { "items": [{ "id": "tt3" }] }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        third_page["state"]["discover"]["results"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert!(destroy_headless_engine(handle));
}
