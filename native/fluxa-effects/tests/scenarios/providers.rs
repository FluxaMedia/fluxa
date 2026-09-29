use super::support::{Reply, Scenario};
use serde_json::json;

fn trakt_profile() -> serde_json::Value {
    json!({"id": "p1", "name": "Me", "traktAccessToken": "tok"})
}

#[test]
fn trakt_scrobble_pause_reaches_the_trakt_api() {
    let scenario = Scenario::start();
    scenario.world.on("api.trakt.tv", "POST", "/scrobble/pause", |_| {
        Reply::json(201, json!({"action": "pause"}))
    });
    let app = scenario.app_with_profile(trakt_profile());

    app.dispatch(json!({
        "type": "scrobbleRequested", "token": "tok", "metaType": "movie",
        "itemId": "tt1", "progress": 42.5, "actionName": "pause", "profile": null
    }));

    let calls = scenario.world.calls("api.trakt.tv", "POST", "/scrobble/pause");
    assert_eq!(calls.len(), 1, "unmatched: {:?}", scenario.world.unmatched());
    assert_eq!(calls[0].header("authorization"), Some("Bearer tok"));
    assert_eq!(calls[0].header("trakt-api-key"), Some("test-trakt"));
    assert_eq!(calls[0].json()["progress"], 42.5);
    assert_eq!(calls[0].json()["movie"]["ids"]["imdb"], "tt1");
}

#[test]
fn trakt_scrobble_is_not_retried_when_the_server_fails() {
    let scenario = Scenario::start();
    scenario.world.respond("api.trakt.tv", "POST", "/scrobble/start", 500, json!({}));
    let app = scenario.app_with_profile(trakt_profile());

    app.dispatch(json!({
        "type": "scrobbleRequested", "token": "tok", "metaType": "movie",
        "itemId": "tt1", "progress": 1.0, "actionName": "start", "profile": null
    }));

    assert_eq!(scenario.world.calls("api.trakt.tv", "POST", "/scrobble/start").len(), 1);
}

#[test]
fn scrobble_without_a_connected_provider_makes_no_requests() {
    let scenario = Scenario::start();
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));

    app.dispatch(json!({
        "type": "scrobbleRequested", "token": "", "metaType": "movie",
        "itemId": "tt1", "progress": 10.0, "actionName": "start", "profile": null
    }));

    assert!(scenario.world.requests("api.trakt.tv").is_empty());
}
