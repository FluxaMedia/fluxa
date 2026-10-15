use super::support::{Reply, Scenario};
use super::{simkl, trakt};
use serde_json::json;

fn trakt_profile() -> serde_json::Value {
    json!({"id": "p1", "name": "Me", "traktAccessToken": "tok"})
}

#[test]
fn trakt_scrobble_pause_reaches_the_trakt_api() {
    let scenario = Scenario::start();
    scenario
        .world
        .on("api.trakt.tv", "POST", "/scrobble/pause", |_| {
            Reply::json(201, json!({"action": "pause"}))
        });
    let app = scenario.app_with_profile(trakt_profile());

    app.dispatch(json!({
        "type": "scrobbleRequested", "token": "tok", "metaType": "movie",
        "itemId": "tt1", "progress": 42.5, "actionName": "pause", "profile": null
    }));

    let calls = scenario
        .world
        .calls("api.trakt.tv", "POST", "/scrobble/pause");
    assert_eq!(
        calls.len(),
        1,
        "unmatched: {:?}",
        scenario.world.unmatched()
    );
    assert_eq!(calls[0].header("authorization"), Some("Bearer tok"));
    assert_eq!(calls[0].header("trakt-api-key"), Some("test-trakt"));
    assert_eq!(calls[0].json()["progress"], 42.5);
    assert_eq!(calls[0].json()["movie"]["ids"]["imdb"], "tt1");
}

#[test]
fn trakt_scrobble_is_not_retried_when_the_server_fails() {
    let scenario = Scenario::start();
    scenario
        .world
        .respond("api.trakt.tv", "POST", "/scrobble/start", 500, json!({}));
    let app = scenario.app_with_profile(trakt_profile());

    app.dispatch(json!({
        "type": "scrobbleRequested", "token": "tok", "metaType": "movie",
        "itemId": "tt1", "progress": 1.0, "actionName": "start", "profile": null
    }));

    assert_eq!(
        scenario
            .world
            .calls("api.trakt.tv", "POST", "/scrobble/start")
            .len(),
        1
    );
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
    assert_eq!(
        scenario
            .world
            .calls(trakt::HOST, "POST", "/sync/watchlist/remove")
            .len(),
        1
    );
    assert!(trakt.watchlist.lock().unwrap().is_empty());
    assert!(
        scenario.world.unmatched().is_empty(),
        "{:?}",
        scenario.world.unmatched()
    );
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

    let calls = scenario
        .world
        .calls(simkl::HOST, "POST", "/sync/add-to-list");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].json()["movies"][0]["ids"]["imdb"], "tt1");
    assert_eq!(calls[0].json()["movies"][0]["to"], "plantowatch");
    assert_eq!(calls[0].header("authorization"), Some("Bearer stok"));
    assert_eq!(calls[0].header("simkl-api-key"), Some("test-simkl"));
    assert_eq!(simkl.plantowatch.lock().unwrap().len(), 1);
    assert!(
        scenario.world.unmatched().is_empty(),
        "{:?}",
        scenario.world.unmatched()
    );
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

fn stop(app: &super::support::App, meta_type: &str, item_id: &str) {
    app.dispatch(json!({
        "type": "scrobbleRequested", "token": "stok", "metaType": meta_type,
        "itemId": item_id, "progress": 95.0, "actionName": "stop", "profile": null
    }));
}

fn allow_rewatch(call: &super::support::Recorded) -> Option<String> {
    call.query_param("allow_rewatch")
}

#[test]
fn scrobble_stop_counts_as_rewatch_only_after_the_rewatch_action() {
    let scenario = Scenario::start();
    simkl::serve(&scenario.world);
    let app = scenario.app_with_profile(simkl_profile());

    stop(&app, "movie", "tt1");
    fluxa_effects::set_simkl_rewatch_intent(app.session.storage(), "p1", "tt1");
    stop(&app, "movie", "tt1");
    stop(&app, "movie", "tt1");

    let stops = scenario.world.calls(simkl::HOST, "POST", "/scrobble/stop");
    assert_eq!(stops.len(), 3);
    assert_eq!(allow_rewatch(&stops[0]), None);
    assert_eq!(allow_rewatch(&stops[1]).as_deref(), Some("yes"));
    assert_eq!(allow_rewatch(&stops[2]), None);
}

#[test]
fn rewatch_intent_on_a_series_covers_its_episodes_only() {
    let scenario = Scenario::start();
    simkl::serve(&scenario.world);
    let app = scenario.app_with_profile(simkl_profile());

    fluxa_effects::set_simkl_rewatch_intent(app.session.storage(), "p1", "tt2");
    stop(&app, "series", "tt22:1:1");
    stop(&app, "series", "tt2:1:3");

    let stops = scenario.world.calls(simkl::HOST, "POST", "/scrobble/stop");
    assert_eq!(allow_rewatch(&stops[0]), None);
    assert_eq!(allow_rewatch(&stops[1]).as_deref(), Some("yes"));
}

#[test]
fn rate_limited_library_read_is_retried_until_it_succeeds() {
    let scenario = Scenario::start();
    trakt::serve(&scenario.world);
    let hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = hits.clone();
    scenario
        .world
        .on(trakt::HOST, "GET", "/sync/watchlist/movies", move |_| {
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
    scenario
        .world
        .respond(trakt::HOST, "GET", "/sync/watchlist/movies", 401, json!({}));
    let app = scenario.app_with_profile(trakt_library_profile());

    app.dispatch(json!({"type": "libraryHydrateRequested"}));

    assert_eq!(
        scenario
            .world
            .calls(trakt::HOST, "GET", "/sync/watchlist/movies")
            .len(),
        1
    );
    assert_ne!(app.at("/library/error"), serde_json::Value::Null);
}

#[test]
fn detail_secondary_lists_trakt_related_titles_for_a_trakt_profile() {
    let scenario = Scenario::start();
    scenario
        .world
        .on("api.trakt.tv", "GET", "/movies/tt1/related", |_| {
            Reply::json(
                200,
                json!([{"title": "Tenet", "year": 2020, "ids": {"imdb": "tt6723592"},
                        "images": {"poster": ["media.trakt.tv/tenet.jpg"]}}]),
            )
        });
    let app = scenario.app_with_profile(json!({
        "id": "p1", "name": "Me", "traktAccessToken": "tok", "integrationLibrarySource": "trakt"
    }));
    let profile = app.at("/profile/active");

    app.dispatch(json!({
        "type": "detailSecondaryRequested", "contentType": "movie", "id": "tt1",
        "language": "en", "profile": profile
    }));

    let similar = app.at("/detail/similarItems");
    assert_eq!(
        similar[0]["id"],
        "tt6723592",
        "{:?}",
        scenario.world.unmatched()
    );
    assert_eq!(similar[0]["poster"], "https://media.trakt.tv/tenet.jpg");
}
