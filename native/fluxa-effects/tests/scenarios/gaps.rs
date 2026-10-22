use super::support::{Reply, Scenario};
use serde_json::json;

#[test]
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
        app.at("/plugins/repositories").as_array().map(Vec::len),
        Some(1)
    );
}

#[test]
fn offline_download_completes_and_survives_a_restart() {
    let scenario = Scenario::start();
    let body = "x".repeat(4096);
    let served = body.clone();
    scenario
        .world
        .on("cdn.example", "GET", "/a.mp4", move |_| Reply {
            status: 200,
            body: served.clone(),
        });
    let app = scenario.app();

    app.dispatch(json!({
        "type": "offlineDownloadRequested",
        "meta": {"id": "tt1", "type": "movie", "name": "Movie One"},
        "stream": {"name": "HD", "url": "https://cdn.example/a.mp4"},
        "videoId": "tt1", "profileId": "p1", "language": "en", "nowMs": 1000
    }));

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.at("/offline/items/0/status") != "completed" {
        assert!(
            std::time::Instant::now() < deadline,
            "download never finished"
        );
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    assert_eq!(app.at("/offline/items/0/bytesDone"), body.len());
    let path = app.at("/offline/items/0/path");
    assert_eq!(
        std::fs::read_to_string(path.as_str().unwrap()).unwrap(),
        body
    );

    let app = app.restart();
    assert_eq!(app.at("/offline/items/0/status"), "completed");
}
