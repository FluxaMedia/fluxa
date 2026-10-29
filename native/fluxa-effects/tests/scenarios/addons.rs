use super::support::{Scenario, addon_manifest};
use serde_json::{Value, json};

fn movie_addon(id: &str) -> Value {
    addon_manifest(
        id,
        json!(["catalog", "meta", "stream"]),
        json!(["movie", "series"]),
        json!([{"type": "movie", "id": "top", "name": "Top"}]),
    )
}

fn serve_movie_addon(scenario: &Scenario, host: &str) {
    scenario
        .world
        .respond(host, "GET", "/manifest.json", 200, movie_addon(host));
    scenario.world.respond(
        host,
        "GET",
        "/catalog/movie/top.json",
        200,
        json!({"metas": [{"id": "tt1", "type": "movie", "name": "Movie One"}]}),
    );
    scenario.world.respond(
        host,
        "GET",
        "/meta/movie/tt1.json",
        200,
        json!({"meta": {"id": "tt1", "type": "movie", "name": "Movie One"}}),
    );
    scenario.world.respond(
        host,
        "GET",
        "/stream/movie/tt1.json",
        200,
        json!({"streams": [{"name": "HD", "url": "https://cdn.example/a.mp4"}]}),
    );
}

#[test]
fn installed_addon_feeds_home_detail_and_streams() {
    let scenario = Scenario::start();
    serve_movie_addon(&scenario, "addon.test");
    let app = scenario.app();

    app.dispatch(json!({"type": "addonInstallRequested", "transportUrl": "https://addon.test/manifest.json"}));
    assert_eq!(app.at("/addons/installed/0/manifest/id"), "addon.test");

    app.dispatch(json!({"type": "homeLoadRequested"}));
    assert_eq!(app.at("/home/categories/0/items/0/name"), "Movie One");

    app.dispatch(json!({
        "type": "detailLoadRequested", "contentType": "movie", "id": "tt1",
        "language": "en", "profile": null
    }));
    assert_eq!(app.at("/detail/meta/name"), "Movie One");

    app.dispatch(json!({
        "type": "detailStreamsRequested", "contentType": "movie", "requestIds": ["tt1"],
        "detail": app.at("/detail/meta"), "language": "en", "profile": null
    }));
    assert_eq!(app.at("/detail/streams/0/url"), "https://cdn.example/a.mp4");
    assert!(
        scenario.world.unmatched().is_empty(),
        "{:?}",
        scenario.world.unmatched()
    );
}

#[test]
fn missing_manifest_reports_an_error_and_installs_nothing() {
    let scenario = Scenario::start();
    let app = scenario.app();

    app.dispatch(json!({"type": "addonInstallRequested", "transportUrl": "https://addon.test/manifest.json"}));

    assert_eq!(app.at("/addons/installed"), json!([]));
    assert_ne!(app.at("/addons/error"), Value::Null);
}

#[test]
fn one_failing_addon_does_not_hide_the_others_on_home() {
    let scenario = Scenario::start();
    serve_movie_addon(&scenario, "addon.test");
    serve_movie_addon(&scenario, "addon2.test");
    scenario.world.respond(
        "addon2.test",
        "GET",
        "/catalog/movie/top.json",
        500,
        json!({}),
    );
    let app = scenario.app();
    for host in ["addon.test", "addon2.test"] {
        app.dispatch(json!({"type": "addonInstallRequested", "transportUrl": format!("https://{host}/manifest.json")}));
    }

    app.dispatch(json!({"type": "homeLoadRequested"}));

    let categories = app.at("/home/categories");
    assert_eq!(categories.as_array().map(Vec::len), Some(1), "{categories}");
    assert_eq!(
        categories[0]["transportUrl"],
        "https://addon.test/manifest.json"
    );
}

fn listed_ids(app: &super::support::App) -> Vec<Value> {
    app.at("/addons/installed")
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|addon| addon["manifest"]["id"].clone())
        .collect()
}

#[test]
fn kits_are_not_listed_until_enabled() {
    let scenario = Scenario::start();
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));

    assert_eq!(listed_ids(&app), Vec::<Value>::new());

    app.dispatch(json!({"type": "settingsChanged", "key": "streamkitEnabled", "value": true}));
    assert_eq!(listed_ids(&app), vec![json!("fluxa.streamkit")]);

    app.dispatch(json!({"type": "settingsChanged", "key": "metadatakitEnabled", "value": true}));
    assert_eq!(
        listed_ids(&app),
        vec![json!("fluxa.streamkit"), json!("fluxa.metadatakit")]
    );

    app.dispatch(json!({"type": "settingsChanged", "key": "streamkitEnabled", "value": false}));
    app.dispatch(json!({"type": "settingsChanged", "key": "metadatakitEnabled", "value": false}));
    assert_eq!(listed_ids(&app), Vec::<Value>::new());
}
