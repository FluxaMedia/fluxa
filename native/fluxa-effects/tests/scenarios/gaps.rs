use super::support::Scenario;
use serde_json::json;

#[test]
#[ignore = "effect fetchPluginManifest has no native implementation yet"]
fn plugin_repository_can_be_added_from_a_manifest_url() {
    let scenario = Scenario::start();
    scenario.world.respond("addon.test", "GET", "/plugins/manifest.json", 200, json!({
        "name": "Repo", "version": "1.0.0", "scrapers": [{"id": "s1", "name": "Scraper", "filename": "s1.js"}]
    }));
    let app = scenario.app();

    app.dispatch(json!({"type": "pluginRepositoryAddRequested", "manifestUrl": "https://addon.test/plugins/manifest.json"}));

    assert_eq!(
        scenario
            .world
            .calls("addon.test", "GET", "/plugins/manifest.json")
            .len(),
        1
    );
    assert_eq!(
        app.at("/addons/plugins/repositories")
            .as_array()
            .map(Vec::len),
        Some(1)
    );
}

#[test]
#[ignore = "effect enqueueOfflineDownload has no native implementation yet"]
fn offline_download_is_queued_for_the_chosen_stream() {
    let scenario = Scenario::start();
    let app = scenario.app();

    app.dispatch(json!({
        "type": "offlineDownloadRequested",
        "meta": {"id": "tt1", "type": "movie", "name": "Movie One"},
        "stream": {"name": "HD", "url": "https://cdn.example/a.mp4"},
        "videoId": "tt1", "profileId": "p1", "language": "en"
    }));

    assert_eq!(app.at("/offline/error"), serde_json::Value::Null);
}
