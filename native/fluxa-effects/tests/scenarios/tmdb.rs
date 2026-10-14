use super::support::{Reply, Scenario};
use serde_json::{Value, json};

fn popular(name: &str, id: i64) -> Value {
    json!({"results": [{"id": id, "title": name, "name": name, "poster_path": "/p.jpg", "overview": "x",
        "release_date": "2024-05-01", "first_air_date": "2024-05-01"}]})
}

#[test]
fn tmdb_key_adds_the_builtin_catalogs_to_home() {
    let scenario = Scenario::start();
    scenario.world.respond(
        "api.themoviedb.org",
        "GET",
        "/3/movie/popular",
        200,
        popular("Movie Pop", 11),
    );
    scenario.world.respond(
        "api.themoviedb.org",
        "GET",
        "/3/tv/popular",
        200,
        popular("Show Pop", 22),
    );
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));
    app.dispatch(json!({"type": "settingsChanged", "key": "tmdbApiKey", "value": "k123"}));

    app.dispatch(json!({"type": "homeLoadRequested"}));

    let categories = app.at("/home/categories");
    let names: Vec<String> = categories
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|category| category["items"].as_array().cloned().unwrap_or_default())
        .filter_map(|item| item["name"].as_str().map(str::to_owned))
        .collect();
    assert!(
        names.contains(&"Movie Pop".to_owned()) && names.contains(&"Show Pop".to_owned()),
        "{categories}"
    );
    let call = &scenario
        .world
        .calls("api.themoviedb.org", "GET", "/3/movie/popular")[0];
    assert_eq!(call.query_param("api_key").as_deref(), Some("k123"));
}

#[test]
fn a_failing_tmdb_catalog_does_not_break_the_other() {
    let scenario = Scenario::start();
    scenario.world.respond(
        "api.themoviedb.org",
        "GET",
        "/3/movie/popular",
        401,
        json!({"status_message": "Invalid API key"}),
    );
    scenario
        .world
        .on("api.themoviedb.org", "GET", "/3/tv/popular", |_| {
            Reply::json(200, popular("Show Pop", 22))
        });
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));
    app.dispatch(json!({"type": "settingsChanged", "key": "tmdbApiKey", "value": "bad"}));

    app.dispatch(json!({"type": "homeLoadRequested"}));

    let categories = app.at("/home/categories");
    assert_eq!(categories.as_array().map(Vec::len), Some(1), "{categories}");
}

#[test]
fn detail_secondary_loads_tmdb_similar_titles() {
    let scenario = Scenario::start();
    scenario
        .world
        .on("api.themoviedb.org", "GET", "/3/find/tt1", |_| {
            Reply::json(
                200,
                json!({"movie_results": [{"id": 603}], "tv_results": []}),
            )
        });
    scenario
        .world
        .on("api.themoviedb.org", "GET", "/3/movie/603/recommendations", |_| {
            Reply::json(
                200,
                json!({"results": [{"id": 604, "title": "Reloaded", "release_date": "2003-05-15", "poster_path": "/r.jpg"}]}),
            )
        });
    scenario
        .world
        .on("api.themoviedb.org", "GET", "/3/movie/603/similar", |_| {
            Reply::json(200, json!({"results": []}))
        });
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me"}));
    app.dispatch(json!({"type": "settingsChanged", "key": "tmdbApiKey", "value": "k123"}));

    app.dispatch(json!({
        "type": "detailSecondaryRequested", "contentType": "movie", "id": "tt1",
        "language": "en", "profile": {"id": "p1"}, "similarTitlesSource": "tmdb"
    }));

    let similar = app.at("/detail/similarItems");
    assert_eq!(
        similar[0]["name"],
        "Reloaded",
        "{similar} {:?}",
        scenario.world.unmatched()
    );
}
