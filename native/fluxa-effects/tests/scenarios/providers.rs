use super::support::{Reply, Scenario};
use super::{simkl, trakt};
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

fn trakt_library_profile() -> serde_json::Value {
    json!({"id": "p1", "name": "Me", "traktAccessToken": "tok", "integrationLibrarySource": "trakt"})
}

fn movie() -> serde_json::Value {
    json!({"id": "tt1", "type": "movie", "name": "Movie One"})
}

#[test]
fn watchlist_toggle_adds_then_removes_on_trakt() {
    let scenario = Scenario::start();
    let trakt = trakt::serve(&scenario.world);
    let app = scenario.app_with_profile(trakt_library_profile());

    app.dispatch(json!({"type": "toggleWatchlistRequested", "item": movie(), "profile": null}));
    let added = scenario.world.calls(trakt::HOST, "POST", "/sync/watchlist");
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].json()["movies"][0]["ids"]["imdb"], "tt1");
    assert_eq!(trakt.watchlist.lock().unwrap().len(), 1);

    app.dispatch(json!({"type": "toggleWatchlistRequested", "item": movie(), "profile": null}));
    assert_eq!(scenario.world.calls(trakt::HOST, "POST", "/sync/watchlist/remove").len(), 1);
    assert!(trakt.watchlist.lock().unwrap().is_empty());
    assert!(scenario.world.unmatched().is_empty(), "{:?}", scenario.world.unmatched());
}

#[test]
fn marking_watched_posts_trakt_history() {
    let scenario = Scenario::start();
    trakt::serve(&scenario.world);
    let app = scenario.app_with_profile(trakt_library_profile());

    app.dispatch(json!({
        "type": "markWatchedRequested", "seriesId": "tt1", "videoIds": ["tt1"],
        "watched": true, "meta": movie(), "episodes": [], "profile": null
    }));

    let calls = scenario.world.calls(trakt::HOST, "POST", "/sync/history");
    assert_eq!(calls.len(), 1, "{:?}", scenario.world.unmatched());
    assert_eq!(calls[0].json()["movies"][0]["ids"]["imdb"], "tt1");
}

fn simkl_profile() -> serde_json::Value {
    json!({"id": "p1", "name": "Me", "simklAccessToken": "stok", "integrationLibrarySource": "simkl"})
}

#[test]
fn simkl_watchlist_toggle_lands_in_plan_to_watch() {
    let scenario = Scenario::start();
    let simkl = simkl::serve(&scenario.world);
    let app = scenario.app_with_profile(simkl_profile());

    app.dispatch(json!({"type": "libraryHydrateRequested"}));
    app.dispatch(json!({"type": "toggleWatchlistRequested", "item": movie(), "profile": null}));

    let calls = scenario.world.calls(simkl::HOST, "POST", "/sync/add-to-list");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].json()["movies"][0]["ids"]["imdb"], "tt1");
    assert_eq!(calls[0].json()["movies"][0]["to"], "plantowatch");
    assert_eq!(calls[0].header("authorization"), Some("Bearer stok"));
    assert_eq!(calls[0].header("simkl-api-key"), Some("test-simkl"));
    assert_eq!(simkl.plantowatch.lock().unwrap().len(), 1);
    assert!(scenario.world.unmatched().is_empty(), "{:?}", scenario.world.unmatched());
}

#[test]
fn simkl_mark_watched_and_scrobble_hit_their_endpoints() {
    let scenario = Scenario::start();
    simkl::serve(&scenario.world);
    let app = scenario.app_with_profile(simkl_profile());

    app.dispatch(json!({
        "type": "markWatchedRequested", "seriesId": "tt1", "videoIds": ["tt1"],
        "watched": true, "meta": movie(), "episodes": [], "profile": null
    }));
    app.dispatch(json!({
        "type": "scrobbleRequested", "token": "stok", "metaType": "movie",
        "itemId": "tt1", "progress": 10.0, "actionName": "start", "profile": null
    }));

    let history = scenario.world.calls(simkl::HOST, "POST", "/sync/history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].json()["movies"][0]["ids"]["imdb"], "tt1");
    let scrobble = scenario.world.calls(simkl::HOST, "POST", "/scrobble/start");
    assert_eq!(scrobble.len(), 1);
    assert_eq!(scrobble[0].json()["progress"], 10.0);
}

#[test]
fn rate_limited_library_read_is_retried_until_it_succeeds() {
    let scenario = Scenario::start();
    trakt::serve(&scenario.world);
    let hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = hits.clone();
    scenario.world.on(trakt::HOST, "GET", "/sync/watchlist/movies", move |_| {
        if counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
            Reply::status(429)
        } else {
            Reply::json(200, json!([]))
        }
    });
    let app = scenario.app_with_profile(trakt_library_profile());

    app.dispatch(json!({"type": "libraryHydrateRequested"}));

    assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(app.at("/library/error"), serde_json::Value::Null);
}

#[test]
fn revoked_trakt_token_fails_the_library_read_without_retrying() {
    let scenario = Scenario::start();
    trakt::serve(&scenario.world);
    scenario.world.respond(trakt::HOST, "GET", "/sync/watchlist/movies", 401, json!({}));
    let app = scenario.app_with_profile(trakt_library_profile());

    app.dispatch(json!({"type": "libraryHydrateRequested"}));

    assert_eq!(scenario.world.calls(trakt::HOST, "GET", "/sync/watchlist/movies").len(), 1);
    assert_ne!(app.at("/library/error"), serde_json::Value::Null);
}
