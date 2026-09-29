mod support;

use serde_json::json;
use support::{Reply, Scenario};

fn require_provider_ids() {
    assert!(
        option_env!("FLUXA_TRAKT_CLIENT_ID").is_some(),
        "run through `npm run check:scenarios` so provider client ids are compiled in"
    );
}

#[test]
fn trakt_scrobble_pause_reaches_the_trakt_api() {
    require_provider_ids();
    let scenario = Scenario::start();
    scenario.world.on("api.trakt.tv", "POST", "/scrobble/pause", |_| {
        Reply::json(201, json!({"action": "pause"}))
    });
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me", "traktAccessToken": "tok"}));

    app.dispatch(json!({
        "type": "scrobbleRequested",
        "token": "tok",
        "metaType": "movie",
        "itemId": "tt1",
        "progress": 42.5,
        "actionName": "pause",
        "profile": null
    }));

    let calls = scenario.world.calls("api.trakt.tv", "POST", "/scrobble/pause");
    assert_eq!(calls.len(), 1, "unmatched: {:?}", scenario.world.unmatched());
    assert_eq!(calls[0].header("authorization"), Some("Bearer tok"));
    assert_eq!(calls[0].header("trakt-api-key"), Some("test-trakt"));
    assert_eq!(calls[0].json()["progress"], 42.5);
    assert_eq!(calls[0].json()["movie"]["ids"]["imdb"], "tt1");
}
