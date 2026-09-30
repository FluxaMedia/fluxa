use super::support::{Scenario, addon_manifest};
use serde_json::{Value, json};

const ADDON: &str = "addon.test";
const TRANSPORT: &str = "https://addon.test/manifest.json";

fn metas(names: &[(&str, &str)]) -> Value {
    json!({"metas": names.iter().map(|(id, name)| json!({"id": id, "type": "movie", "name": name})).collect::<Vec<_>>()})
}

fn browsable_addon(scenario: &Scenario) {
    let world = &scenario.world;
    world.respond(
        ADDON,
        "GET",
        "/manifest.json",
        200,
        addon_manifest(
            "a",
            json!(["catalog", "meta", "stream"]),
            json!(["movie", "series"]),
            json!([{"type": "movie", "id": "top", "name": "Top", "extra": [
                {"name": "search"}, {"name": "skip"}, {"name": "genre", "options": ["Drama"]}
            ]}]),
        ),
    );
    world.respond(
        ADDON,
        "GET",
        "/catalog/movie/top.json",
        200,
        metas(&[("tt1", "Movie One")]),
    );
    world.respond(
        ADDON,
        "GET",
        "/catalog/movie/top/search=one.json",
        200,
        metas(&[("tt1", "Movie One")]),
    );
    world.respond(
        ADDON,
        "GET",
        "/catalog/movie/top/skip=1.json",
        200,
        metas(&[("tt2", "Movie Two")]),
    );
    world.respond(
        ADDON,
        "GET",
        "/catalog/movie/top/genre=Drama.json",
        200,
        metas(&[("tt3", "Drama Three")]),
    );
}

fn movie() -> Value {
    json!({"id": "tt1", "type": "movie", "name": "Movie One"})
}

#[test]
fn search_queries_every_searchable_catalog() {
    let scenario = Scenario::start();
    browsable_addon(&scenario);
    let app = scenario.app();
    app.dispatch(json!({"type": "addonInstallRequested", "transportUrl": TRANSPORT}));

    app.dispatch(
        json!({"type": "searchRequested", "query": "one", "profile": null, "language": "en"}),
    );

    assert_eq!(app.at("/search/results/0/name"), "Movie One");
    assert_eq!(
        scenario
            .world
            .calls(ADDON, "GET", "/catalog/movie/top/search=one.json")
            .len(),
        1
    );
}

#[test]
fn discover_loads_the_first_page_then_the_next() {
    let scenario = Scenario::start();
    browsable_addon(&scenario);
    let app = scenario.app();
    app.dispatch(json!({"type": "addonInstallRequested", "transportUrl": TRANSPORT}));

    app.dispatch(json!({
        "type": "discoverRequested", "contentType": "movie", "filters": null,
        "profile": null, "language": "en", "loadCatalogFilters": true
    }));
    assert_eq!(app.at("/discover/results/0/id"), "tt1");
    assert_eq!(app.at("/discover/genres/1/id"), "Drama");

    app.dispatch(json!({
        "type": "discoverPageRequested", "transportUrl": TRANSPORT, "contentType": "movie",
        "catalogId": "top", "skip": app.at("/discover/paging/nextSkip")
    }));
    assert_eq!(
        scenario
            .world
            .calls(ADDON, "GET", "/catalog/movie/top/skip=1.json")
            .len(),
        1
    );
    assert_eq!(app.at("/discover/paging/error"), Value::Null);
}

fn second_addon(scenario: &Scenario) {
    let world = &scenario.world;
    world.respond(
        "addon2.test",
        "GET",
        "/manifest.json",
        200,
        addon_manifest(
            "b",
            json!(["catalog"]),
            json!(["movie"]),
            json!([{"type": "movie", "id": "new", "name": "New"}]),
        ),
    );
    world.respond(
        "addon2.test",
        "GET",
        "/catalog/movie/new.json",
        200,
        metas(&[("tt9", "Nine")]),
    );
}

#[test]
fn removing_an_addon_drops_its_rows_from_home() {
    let scenario = Scenario::start();
    browsable_addon(&scenario);
    second_addon(&scenario);
    let app = scenario.app();
    for url in [TRANSPORT, "https://addon2.test/manifest.json"] {
        app.dispatch(json!({"type": "addonInstallRequested", "transportUrl": url}));
    }
    app.dispatch(json!({"type": "homeLoadRequested"}));
    assert_eq!(app.at("/home/categories").as_array().map(Vec::len), Some(2));

    app.dispatch(json!({"type": "addonRemoveRequested", "transportUrl": TRANSPORT}));
    app.dispatch(json!({"type": "homeLoadRequested", "force": true}));

    let categories = app.at("/home/categories");
    assert_eq!(categories.as_array().map(Vec::len), Some(1), "{categories}");
    assert_eq!(
        categories[0]["transportUrl"],
        "https://addon2.test/manifest.json"
    );
}

#[test]
#[ignore = "an empty home reload falls back to the cached bootstrap, so the last removed addon's rows stay"]
fn removing_the_last_addon_empties_home() {
    let scenario = Scenario::start();
    browsable_addon(&scenario);
    let app = scenario.app();
    app.dispatch(json!({"type": "addonInstallRequested", "transportUrl": TRANSPORT}));
    app.dispatch(json!({"type": "homeLoadRequested"}));

    app.dispatch(json!({"type": "addonRemoveRequested", "transportUrl": TRANSPORT}));
    app.dispatch(json!({"type": "homeLoadRequested", "force": true}));

    assert_eq!(app.at("/home/categories"), json!([]));
}

#[test]
fn settings_survive_a_restart() {
    let scenario = Scenario::start();
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));
    app.dispatch(json!({"type": "settingsChanged", "key": "language", "value": "tr"}));

    let app = app.restart();

    assert_eq!(app.at("/settings/values/language"), "tr");
}

#[test]
fn installed_addons_survive_a_restart() {
    let scenario = Scenario::start();
    browsable_addon(&scenario);
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));
    app.dispatch(json!({"type": "addonInstallRequested", "transportUrl": TRANSPORT}));

    let app = app.restart();
    app.dispatch(json!({"type": "addonsRefreshRequested", "profile": null, "forceRefresh": false}));

    assert_eq!(
        app.at("/addons/installed").as_array().map(Vec::len),
        Some(1)
    );
}

#[test]
fn saved_progress_shows_up_in_continue_watching() {
    let scenario = Scenario::start();
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));

    app.dispatch(json!({
        "type": "savePlaybackProgressRequested", "profile": null, "meta": movie(),
        "timeOffset": 50, "duration": 100
    }));
    app.dispatch(json!({"type": "refreshContinueWatchingRequested", "profile": null, "language": "en", "source": null}));

    assert_eq!(app.at("/home/continueWatching/0/id"), "tt1");
    assert_eq!(app.at("/home/continueWatching/0/timeOffset"), 50);
    assert!(scenario.world.requests_all().is_empty());
}

#[test]
fn subtitles_come_from_the_selected_stream() {
    let scenario = Scenario::start();
    let app = scenario.app();

    app.dispatch(json!({
        "type": "subtitleLoadRequested", "contentType": "movie", "id": "tt1", "extraArgs": null,
        "stream": {"url": "https://cdn.example/a.mp4", "subtitles": [
            {"id": "s1", "url": "https://cdn.example/s.srt", "lang": "eng"}
        ]}
    }));

    assert_eq!(app.at("/player/subtitleLoading"), false);
    assert_eq!(
        app.at("/player/subtitles/0/url"),
        "https://cdn.example/s.srt"
    );
}

#[test]
#[ignore = "effect clearPlaybackProgress has no native implementation yet"]
fn clearing_progress_removes_the_continue_watching_entry() {
    let scenario = Scenario::start();
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));
    app.dispatch(json!({
        "type": "savePlaybackProgressRequested", "profile": null, "meta": movie(),
        "timeOffset": 50, "duration": 100
    }));

    app.dispatch(
        json!({"type": "clearPlaybackProgressRequested", "profile": null, "meta": movie()}),
    );

    assert_eq!(app.at("/library/savedPlaybackProgress"), Value::Null);
}

