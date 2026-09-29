use super::support::{App, Reply, Scenario, World};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

const HOST: &str = "api.nuvio.tv";

struct FakeNuvio {
    library: Arc<Mutex<Vec<Value>>>,
    progress: Arc<Mutex<Vec<Value>>>,
}

fn rpc(name: &str) -> String {
    format!("/rest/v1/rpc/{name}")
}

fn serve(world: &World) -> FakeNuvio {
    let library = Arc::new(Mutex::new(Vec::<Value>::new()));
    let progress = Arc::new(Mutex::new(Vec::<Value>::new()));
    let rows = library.clone();
    world.on(HOST, "POST", &rpc("sync_pull_library"), move |_| {
        Reply::json(200, json!(rows.lock().unwrap().clone()))
    });
    let rows = progress.clone();
    world.on(HOST, "POST", &rpc("sync_pull_watch_progress"), move |_| {
        Reply::json(200, json!(rows.lock().unwrap().clone()))
    });
    for name in ["library", "watch_progress", "watched_items"] {
        world.respond(
            HOST,
            "POST",
            &rpc(&format!("sync_get_{name}_delta_cursor")),
            200,
            json!(0),
        );
        world.respond(
            HOST,
            "POST",
            &rpc(&format!("sync_pull_{name}_delta")),
            200,
            json!([]),
        );
    }
    world.respond(
        HOST,
        "POST",
        &rpc("sync_pull_watched_items"),
        200,
        json!([]),
    );
    let rows = library.clone();
    world.on(
        HOST,
        "POST",
        &rpc("sync_push_library_items"),
        move |request| {
            for item in request.json()["p_items"].as_array().into_iter().flatten() {
                rows.lock().unwrap().push(item.clone());
            }
            Reply::json(200, json!(null))
        },
    );
    world.respond(
        HOST,
        "POST",
        &rpc("sync_push_watch_progress"),
        200,
        json!(null),
    );
    world.respond(
        HOST,
        "POST",
        &rpc("sync_push_watched_items"),
        200,
        json!(null),
    );
    world.respond(
        HOST,
        "POST",
        &rpc("sync_delete_watched_items"),
        200,
        json!(null),
    );
    FakeNuvio { library, progress }
}

fn profile(expires_at: i64) -> Value {
    json!({
        "id": "p1", "name": "Me", "nuvioAccessToken": "access-1", "nuvioRefreshToken": "refresh-1",
        "nuvioTokenExpiresAt": expires_at, "nuvioUserId": "user-1", "nuvioProfileIndex": 1,
        "integrationLibrarySource": "nuvio"
    })
}

fn fresh_profile() -> Value {
    profile(4_000_000_000)
}

fn library_row(id: &str, name: &str) -> Value {
    json!({"content_id": id, "content_type": "movie", "name": name, "poster": "https://cdn.example/p.jpg"})
}

fn hydrate(app: &App) {
    app.dispatch(json!({"type": "libraryHydrateRequested"}));
}

#[test]
fn library_is_pulled_from_the_account() {
    let scenario = Scenario::start();
    let nuvio = serve(&scenario.world);
    nuvio
        .library
        .lock()
        .unwrap()
        .push(library_row("tt1", "Movie One"));
    let app = scenario.app_with_profile(fresh_profile());

    hydrate(&app);

    assert_eq!(app.at("/library/error"), Value::Null);
    let watchlist = app.at("/library/watchlist");
    assert_eq!(watchlist.as_array().map(Vec::len), Some(1), "{watchlist}");
    assert_eq!(watchlist[0]["id"], "tt1");
    let pull = &scenario
        .world
        .calls(HOST, "POST", &rpc("sync_pull_library"))[0];
    assert_eq!(pull.json()["p_profile_id"], 1);
    assert_eq!(pull.header("apikey"), Some("test-key"));
    assert_eq!(pull.header("authorization"), Some("Bearer access-1"));
}

#[test]
fn second_hydrate_within_the_check_window_does_not_hit_the_server() {
    let scenario = Scenario::start();
    serve(&scenario.world);
    let app = scenario.app_with_profile(fresh_profile());

    hydrate(&app);
    hydrate(&app);

    assert_eq!(
        scenario
            .world
            .calls(HOST, "POST", &rpc("sync_pull_library"))
            .len(),
        1
    );
}

#[test]
fn watchlist_toggle_is_pushed_to_the_account() {
    let scenario = Scenario::start();
    let nuvio = serve(&scenario.world);
    let app = scenario.app_with_profile(fresh_profile());

    app.dispatch(json!({
        "type": "toggleWatchlistRequested",
        "item": {"id": "tt9", "type": "movie", "name": "Nine"}, "profile": null
    }));

    let pushes = scenario
        .world
        .calls(HOST, "POST", &rpc("sync_push_library_items"));
    assert_eq!(pushes.len(), 1, "{:?}", scenario.world.unmatched());
    assert_eq!(pushes[0].json()["p_items"][0]["content_id"], "tt9");
    assert_eq!(nuvio.library.lock().unwrap().len(), 1);
}

#[test]
fn saved_progress_is_pushed_to_the_account() {
    let scenario = Scenario::start();
    serve(&scenario.world);
    let app = scenario.app_with_profile(fresh_profile());

    app.dispatch(json!({
        "type": "savePlaybackProgressRequested", "profile": null,
        "meta": {"id": "tt1", "type": "movie", "name": "Movie One"},
        "timeOffset": 120, "duration": 600, "lastVideoId": "tt1"
    }));

    let pushes = scenario
        .world
        .calls(HOST, "POST", &rpc("sync_push_watch_progress"));
    assert_eq!(pushes.len(), 1, "{:?}", scenario.world.unmatched());
    assert_eq!(pushes[0].json()["p_entries"][0]["content_id"], "tt1");
}

#[test]
fn mark_watched_is_pushed_to_the_account() {
    let scenario = Scenario::start();
    serve(&scenario.world);
    let app = scenario.app_with_profile(fresh_profile());

    app.dispatch(json!({
        "type": "markWatchedRequested", "seriesId": "tt1", "videoIds": ["tt1"], "watched": true,
        "meta": {"id": "tt1", "type": "movie", "name": "Movie One"}, "profile": null
    }));

    assert_eq!(
        scenario
            .world
            .calls(HOST, "POST", &rpc("sync_push_watched_items"))
            .len(),
        1,
        "{:?}",
        scenario.world.unmatched()
    );
}

#[test]
fn expired_token_is_refreshed_and_stored_before_syncing() {
    let scenario = Scenario::start();
    serve(&scenario.world);
    scenario.world.respond(
        HOST,
        "POST",
        "/auth/v1/token",
        200,
        json!({
            "access_token": "access-2", "refresh_token": "refresh-2", "expires_in": 3600
        }),
    );
    let app = scenario.app_with_profile(profile(1));

    hydrate(&app);

    let refresh = &scenario.world.calls(HOST, "POST", "/auth/v1/token")[0];
    assert_eq!(
        refresh.query_param("grant_type").as_deref(),
        Some("refresh_token")
    );
    assert_eq!(refresh.json()["refresh_token"], "refresh-1");
    let pull = &scenario
        .world
        .calls(HOST, "POST", &rpc("sync_pull_library"))[0];
    assert_eq!(pull.header("authorization"), Some("Bearer access-2"));
}

#[test]
fn failed_refresh_surfaces_an_error_instead_of_an_empty_library() {
    let scenario = Scenario::start();
    serve(&scenario.world);
    scenario.world.respond(
        HOST,
        "POST",
        "/auth/v1/token",
        400,
        json!({"error": "invalid_grant"}),
    );
    let app = scenario.app_with_profile(profile(1));

    hydrate(&app);

    assert_ne!(app.at("/library/error"), Value::Null);
    assert!(
        scenario
            .world
            .calls(HOST, "POST", &rpc("sync_pull_library"))
            .is_empty()
    );
}

#[test]
fn server_failure_after_a_good_sync_keeps_the_cached_library() {
    let scenario = Scenario::start();
    let nuvio = serve(&scenario.world);
    nuvio
        .library
        .lock()
        .unwrap()
        .push(library_row("tt1", "Movie One"));
    let app = scenario.app_with_profile(fresh_profile());
    hydrate(&app);
    scenario
        .world
        .respond(HOST, "POST", &rpc("sync_pull_library"), 500, json!({}));

    let app = app.restart();
    hydrate(&app);

    assert_eq!(
        app.at("/library/watchlist").as_array().map(Vec::len),
        Some(1)
    );
}
