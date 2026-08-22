use super::super::*;
use serde_json::{Value, json};

#[test]
fn plugin_repository_add_completion_populates_repositories_and_scrapers() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"pluginRepositoryAddRequested","manifestUrl":"https://example.com/manifest.json"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(requested["effects"][0]["type"], "fetchPluginManifest");
    assert_eq!(
        requested["state"]["plugins"]["addingRepositoryUrl"],
        "https://example.com/manifest.json"
    );

    let completed: Value = serde_json::from_str(
            &headless_engine_complete_effect_json(
                handle,
                &json!({
                    "effectId": requested["effects"][0]["id"].as_str().unwrap(),
                    "status": "ok",
                    "value": {
                        "manifestUrl": "https://example.com/manifest.json",
                        "manifest": {
                            "name": "Phisher's Repo",
                            "version": "1.0.0",
                            "scrapers": [
                                {"id": "MoviesDrive", "name": "MoviesDrive", "version": "1.1.1", "filename": "src/providers/moviesdrive.js"}
                            ]
                        }
                    }
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        completed["state"]["plugins"]["addingRepositoryUrl"],
        Value::Null
    );
    assert_eq!(
        completed["state"]["plugins"]["repositories"][0]["name"],
        "Phisher's Repo"
    );
    assert_eq!(
        completed["state"]["plugins"]["scrapers"][0]["repositoryUrl"],
        "https://example.com/manifest.json"
    );

    let removed: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"pluginRepositoryRemoveRequested","manifestUrl":"https://example.com/manifest.json"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(
        removed["state"]["plugins"]["repositories"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        removed["state"]["plugins"]["scrapers"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert!(destroy_headless_engine(handle));
}

#[test]
fn plugin_repository_refetch_preserves_disabled_state_and_settings() {
    let handle = create_headless_engine("{}");
    let manifest_value = json!({
        "manifestUrl": "https://example.com/manifest.json",
        "manifest": {
            "name": "Phisher's Repo",
            "version": "1.0.0",
            "scrapers": [
                {"id": "MoviesDrive", "name": "MoviesDrive", "version": "1.1.1", "filename": "src/providers/moviesdrive.js"}
            ]
        }
    });

    let requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"pluginRepositoryAddRequested","manifestUrl":"https://example.com/manifest.json"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    headless_engine_complete_effect_json(
        handle,
        &json!({
            "effectId": requested["effects"][0]["id"].as_str().unwrap(),
            "status": "ok",
            "value": manifest_value
        })
        .to_string(),
    )
    .unwrap();

    headless_engine_dispatch_json(
        handle,
        r#"{"type":"pluginScraperToggled","scraperId":"MoviesDrive","enabled":false}"#,
    )
    .unwrap();
    headless_engine_dispatch_json(
            handle,
            r#"{"type":"pluginScraperSettingsUpdated","scraperId":"MoviesDrive","settings":{"quality":"1080p"}}"#,
        )
        .unwrap();

    let refetch_requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"pluginRepositoryAddRequested","manifestUrl":"https://example.com/manifest.json"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    let refetched: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": refetch_requested["effects"][0]["id"].as_str().unwrap(),
                "status": "ok",
                "value": manifest_value
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    let scraper = &refetched["state"]["plugins"]["scrapers"][0];
    assert_eq!(scraper["enabled"], false);
    assert_eq!(scraper["settings"]["quality"], "1080p");
    assert!(destroy_headless_engine(handle));
}
