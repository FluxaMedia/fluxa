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

const TMDB: &str = "api.themoviedb.org";

fn metadatakit_app(scenario: &Scenario) -> super::support::App {
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));
    app.dispatch(json!({"type": "settingsChanged", "key": "tmdbApiKey", "value": "k123"}));
    app.dispatch(json!({"type": "settingsChanged", "key": "metadatakitEnabled", "value": true}));
    app
}

#[test]
fn metadatakit_catalogs_fill_home_like_any_addon() {
    let scenario = Scenario::start();
    for (path, name) in [
        ("/3/trending/movie/week", "Kit Movie"),
        ("/3/trending/tv/week", "Kit Show"),
    ] {
        scenario.world.respond(
            TMDB,
            "GET",
            path,
            200,
            json!({"page": 1, "total_pages": 1, "results": [
                {"id": 7, "title": name, "name": name, "poster_path": "/p.jpg",
                 "release_date": "2024-05-01", "first_air_date": "2024-05-01"}
            ]}),
        );
    }
    let app = metadatakit_app(&scenario);

    app.dispatch(json!({"type": "homeLoadRequested", "force": true}));

    let names: Vec<String> = app
        .at("/home/categories")
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .flat_map(|category| category["items"].as_array().cloned().unwrap_or_default())
        .filter_map(|item| item["name"].as_str().map(str::to_owned))
        .collect();
    assert!(names.contains(&"Kit Movie".to_owned()), "{names:?}");
    assert!(names.contains(&"Kit Show".to_owned()), "{names:?}");
}

#[test]
fn metadatakit_serves_title_details_from_tmdb() {
    let scenario = Scenario::start();
    scenario.world.respond(
        TMDB,
        "GET",
        "/3/movie/603",
        200,
        json!({
            "id": 603, "title": "The Matrix", "original_title": "The Matrix",
            "original_language": "en", "overview": "A hacker learns the truth.",
            "release_date": "1999-03-30", "runtime": 136,
            "external_ids": {"imdb_id": "tt0133093"},
            "release_dates": {"results": [
                {"iso_3166_1": "US", "release_dates": [{"certification": "R"}]}
            ]}
        }),
    );
    let app = metadatakit_app(&scenario);

    app.dispatch(json!({
        "type": "detailLoadRequested", "contentType": "movie", "id": "tmdb:603",
        "language": "en", "profile": null
    }));

    assert_eq!(app.at("/detail/meta/name"), "The Matrix");
    assert!(
        scenario
            .world
            .calls(TMDB, "GET", "/3/movie/603")
            .iter()
            .any(|call| call
                .query_param("append_to_response")
                .is_some_and(|append| append.contains("release_dates"))),
    );
}
