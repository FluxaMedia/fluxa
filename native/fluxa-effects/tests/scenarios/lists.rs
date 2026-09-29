use super::support::{Reply, Scenario};
use serde_json::{Value, json};

const ANILIST_TOKEN: &str = "h.eyJzdWIiOiI0MiJ9.s";

fn anilist_profile() -> Value {
    json!({"id": "p1", "name": "Me", "anilistAccessToken": ANILIST_TOKEN, "integrationLibrarySource": "anilist"})
}

fn mdblist_profile() -> Value {
    json!({"id": "p1", "name": "Me", "mdblistAccessToken": "mtok", "integrationLibrarySource": "mdblist"})
}

fn movie() -> Value {
    json!({"id": "tt1", "type": "movie", "name": "Movie One"})
}

fn anilist_lists() -> Value {
    json!({"data": {"MediaListCollection": {"hasNextChunk": false, "lists": [
        {"isCustomList": false, "entries": [
            {"status": "CURRENT", "progress": 3, "updatedAt": 1_700_000_000, "media": {
                "id": 1, "idMal": 10, "format": "TV", "episodes": 12, "seasonYear": 2024,
                "genres": ["Action"], "title": {"english": "Show One", "romaji": "Show Ichi", "native": "x"},
                "coverImage": {"extraLarge": "https://cdn.example/a.jpg", "large": "https://cdn.example/a.jpg"},
                "bannerImage": null}},
            {"status": "PLANNING", "progress": 0, "updatedAt": 1_700_000_100, "media": {
                "id": 2, "idMal": 20, "format": "TV", "episodes": 12, "seasonYear": 2024,
                "genres": [], "title": {"english": "Show Two", "romaji": "Show Ni", "native": "y"},
                "coverImage": {"extraLarge": null, "large": null}, "bannerImage": null}}
        ]}
    ]}}})
}

#[test]
fn anilist_library_is_read_with_the_user_id_from_the_token() {
    let scenario = Scenario::start();
    scenario
        .world
        .on("graphql.anilist.co", "POST", "/", |request| {
            assert_eq!(request.json()["variables"]["userId"], 42);
            assert_eq!(
                request.header("authorization"),
                Some(&format!("Bearer {ANILIST_TOKEN}")[..])
            );
            if request.json()["variables"]["chunk"] == 1 {
                Reply::json(200, anilist_lists())
            } else {
                Reply::json(200, json!({"data": {"MediaListCollection": {"lists": []}}}))
            }
        });
    let app = scenario.app_with_profile(anilist_profile());

    app.dispatch(json!({"type": "libraryHydrateRequested"}));

    assert_eq!(app.at("/library/error"), Value::Null);
    assert_eq!(
        app.at("/library/watchlist").as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        app.at("/library/continueWatching").as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        scenario
            .world
            .calls("graphql.anilist.co", "POST", "/")
            .len(),
        3
    );
    assert!(
        scenario.world.unmatched().is_empty(),
        "{:?}",
        scenario.world.unmatched()
    );
}

#[test]
fn anilist_mark_watched_saves_the_episode_progress() {
    let scenario = Scenario::start();
    scenario
        .world
        .respond("graphql.anilist.co", "POST", "/", 200, anilist_lists());
    let app = scenario.app_with_profile(anilist_profile());

    app.dispatch(json!({
        "type": "markWatchedRequested", "seriesId": "anilist:1", "videoIds": ["anilist:1:1:5"],
        "watched": true, "meta": {"id": "anilist:1", "type": "series", "name": "Show One"},
        "episodes": [{"id": "anilist:1:1:5"}], "profile": null
    }));

    let saves: Vec<_> = scenario
        .world
        .calls("graphql.anilist.co", "POST", "/")
        .into_iter()
        .filter(|call| call.body.contains("SaveMediaListEntry"))
        .collect();
    assert_eq!(
        saves.len(),
        1,
        "{:?}",
        scenario
            .world
            .requests_all()
            .iter()
            .map(|r| r.body.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(saves[0].json()["variables"]["progress"], 5);
}

#[test]
fn mdblist_library_is_assembled_from_its_endpoints() {
    let scenario = Scenario::start();
    let host = "api.mdblist.com";
    scenario.world.respond(host, "GET", "/watchlist/items", 200, json!({
        "movies": [{"id": 1, "title": "Movie One", "imdb_id": "tt1", "mediatype": "movie", "release_year": 2020}],
        "shows": []
    }));
    scenario.world.respond(
        host,
        "GET",
        "/sync/watched",
        200,
        json!({"movies": [], "episodes": []}),
    );
    scenario
        .world
        .respond(host, "GET", "/sync/dropped", 200, json!([]));
    scenario
        .world
        .respond(host, "GET", "/sync/playback", 200, json!([]));
    scenario
        .world
        .respond(host, "GET", "/upnext", 200, json!([]));
    let app = scenario.app_with_profile(mdblist_profile());

    app.dispatch(json!({"type": "libraryHydrateRequested"}));

    assert_eq!(app.at("/library/error"), Value::Null);
    assert_eq!(app.at("/library/watchlist/0/id"), "tt1");
    assert_eq!(
        scenario.world.calls(host, "GET", "/watchlist/items")[0].header("authorization"),
        Some("Bearer mtok")
    );
    assert!(
        scenario.world.unmatched().is_empty(),
        "{:?}",
        scenario.world.unmatched()
    );
}
