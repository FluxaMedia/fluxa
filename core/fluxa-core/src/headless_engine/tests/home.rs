use super::super::*;
use serde_json::{Value, json};

#[test]
fn home_load_is_owned_by_core_and_resolved_through_platform_effect() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"homeLoadRequested","profile":{"id":"p1","showHeroSection":false},"language":"tr","force":true}"#,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(requested["state"]["home"]["isLoading"], true);
    assert_eq!(requested["effects"][0]["type"], "readHomeBootstrap");
    assert_eq!(requested["effects"][0]["payload"]["profileId"], "p1");
    assert_eq!(requested["effects"][0]["payload"]["language"], "tr");
    assert_eq!(requested["effects"][0]["payload"]["force"], true);
    assert_eq!(requested["state"]["profile"]["active"]["id"], "p1");
    assert_eq!(
        requested["state"]["home"]["activeProfile"]["showHeroSection"],
        false
    );

    let effect_id = requested["effects"][0]["id"].as_str().unwrap();
    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": {
                    "categories": [{ "id": "featured" }],
                    "continueWatching": [{ "id": "tt1" }],
                    "metadataFeeds": [{ "key": "cinemeta" }],
                    "billboard": { "id": "tt2" }
                }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(completed["state"]["home"]["isLoading"], false);
    assert_eq!(
        completed["state"]["home"]["categories"][0]["id"],
        "featured"
    );
    assert_eq!(
        completed["state"]["home"]["continueWatching"][0]["id"],
        "tt1"
    );
    assert_eq!(completed["state"]["home"]["billboard"]["id"], "tt2");
    assert!(destroy_headless_engine(handle));
}

#[test]
fn catalog_page_results_append_to_the_matching_home_row() {
    let handle = create_headless_engine("{}");
    let initial: Value = serde_json::from_str(
        &headless_engine_dispatch_json(handle, r#"{"type":"homeLoadRequested"}"#).unwrap(),
    )
    .unwrap();
    headless_engine_complete_effect_json(
        handle,
        &json!({
            "effectId": initial["effects"][0]["id"],
            "status": "ok",
            "value": {"categories": [{"id":"popular","type":"movie","items":[{"id":"tt1","type":"movie"}],"canLoadMore":true}]}
        })
        .to_string(),
    )
    .unwrap();

    let page: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"catalogPageRequested","categoryId":"popular","contentType":"movie","catalogId":"top","skip":1}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": page["effects"][0]["id"],
                "status": "ok",
                "value": {"items": [{"id":"tt1","type":"movie"},{"id":"tt2","type":"movie"}]}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        completed["state"]["home"]["categories"][0]["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        completed["state"]["home"]["categories"][0]["items"][1]["id"],
        "tt2"
    );
    assert!(destroy_headless_engine(handle));
}

#[test]
fn forced_home_refresh_keeps_stale_categories_visible() {
    let handle = create_headless_engine("{}");
    let initial: Value = serde_json::from_str(
        &headless_engine_dispatch_json(handle, r#"{"type":"homeLoadRequested"}"#).unwrap(),
    )
    .unwrap();
    let cached: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": initial["effects"][0]["id"],
                "status": "ok",
                "value": { "stale": true, "categories": [{ "id": "cached" }] }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(cached["state"]["home"]["isStale"], true);
    assert_eq!(cached["state"]["home"]["categories"][0]["id"], "cached");

    let refresh: Value = serde_json::from_str(
        &headless_engine_dispatch_json(handle, r#"{"type":"homeLoadRequested","force":true}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(refresh["state"]["home"]["isLoading"], true);
    assert_eq!(refresh["state"]["home"]["isStale"], false);
    assert_eq!(refresh["state"]["home"]["categories"][0]["id"], "cached");
    assert!(destroy_headless_engine(handle));
}

#[test]
fn home_load_delivers_continue_watching_in_its_single_bootstrap_effect() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"homeLoadRequested","profile":{"id":"p1"},"language":"tr"}"#,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(requested["effects"].as_array().unwrap().len(), 1);
    assert_eq!(requested["effects"][0]["type"], "readHomeBootstrap");
    assert_eq!(requested["effects"][0]["payload"]["profileId"], "p1");
    assert_eq!(requested["effects"][0]["payload"]["language"], "tr");

    let bootstrap_id = requested["effects"][0]["id"].as_str().unwrap();
    let bootstrap_completed: Value = serde_json::from_str(
            &headless_engine_complete_effect_json(
                handle,
                &json!({
                    "effectId": bootstrap_id,
                    "status": "ok",
                    "value": { "continueWatching": [{ "id": "tt1", "continueWatchingBadge": "newEpisode" }] }
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(bootstrap_completed["state"]["home"]["isLoading"], false);
    assert_eq!(
        bootstrap_completed["state"]["home"]["continueWatching"][0]["continueWatchingBadge"],
        "newEpisode"
    );
    assert_eq!(
        bootstrap_completed["state"]["home"]["continueWatching"][0]["continueWatchingBadge"],
        "newEpisode"
    );
    assert!(destroy_headless_engine(handle));
}

#[test]
fn a_cached_home_is_shown_and_then_refetched() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"homeLoadRequested","profile":{"id":"p1"},"language":"tr"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(requested["effects"][0]["payload"]["force"], false);

    let effect_id = requested["effects"][0]["id"].as_str().unwrap();
    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": { "stale": true, "continueWatching": [{ "id": "tt1" }] }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        completed["state"]["home"]["continueWatching"][0]["id"],
        "tt1"
    );
    assert_eq!(completed["effects"][0]["type"], "readHomeBootstrap");
    assert_eq!(completed["effects"][0]["payload"]["force"], true);
    assert_eq!(completed["effects"][0]["payload"]["language"], "tr");
}
