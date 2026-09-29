use super::simkl;
use super::support::Scenario;
use serde_json::{Value, json};

fn month(app: &super::support::App, profile: Value) -> Value {
    app.dispatch(
        json!({"type": "calendarMonthRequested", "profile": profile, "year": 2026, "month": 9}),
    );
    app.at("/calendar/externalItems")
}

#[test]
fn trakt_calendar_lists_the_shows_and_movies_airing_that_month() {
    let scenario = Scenario::start();
    scenario.world.respond(
        "api.trakt.tv",
        "GET",
        "/calendars/my/shows/2026-09-01/30",
        200,
        json!([{
            "first_aired": "2026-09-10T01:00:00.000Z",
            "episode": {"season": 2, "number": 4, "title": "Fourth"},
            "show": {"title": "Show One", "ids": {"imdb": "tt111", "trakt": 1}}
        }]),
    );
    scenario.world.respond(
        "api.trakt.tv",
        "GET",
        "/calendars/my/movies/2026-09-01/30",
        200,
        json!([{
            "released": "2026-09-20",
            "movie": {"title": "Movie One", "ids": {"imdb": "tt222", "trakt": 2}}
        }]),
    );
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me", "traktAccessToken": "tok", "integrationLibrarySource": "trakt"}));

    let items = month(&app, app.at("/profile/active"));

    let ids: Vec<&str> = items
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["contentId"].as_str())
        .collect();
    assert!(ids.contains(&"tt111"), "{items}");
    assert!(ids.contains(&"tt222"), "{items}");
    let episode = items
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["contentId"] == "tt111")
        .unwrap();
    assert_eq!(episode["seasonNumber"], 2);
    assert_eq!(episode["episodeNumber"], 4);
    assert!(
        scenario.world.unmatched().is_empty(),
        "{:?}",
        scenario.world.unmatched()
    );
}

#[test]
fn trakt_calendar_is_cached_for_a_second_open() {
    let scenario = Scenario::start();
    scenario.world.respond(
        "api.trakt.tv",
        "GET",
        "/calendars/my/shows/2026-09-01/30",
        200,
        json!([]),
    );
    scenario.world.respond(
        "api.trakt.tv",
        "GET",
        "/calendars/my/movies/2026-09-01/30",
        200,
        json!([]),
    );
    let app = scenario.app_with_profile(json!({"id": "p1", "name": "Me", "traktAccessToken": "tok", "integrationLibrarySource": "trakt"}));

    month(&app, app.at("/profile/active"));
    month(&app, app.at("/profile/active"));

    assert_eq!(
        scenario
            .world
            .calls("api.trakt.tv", "GET", "/calendars/my/shows/2026-09-01/30")
            .len(),
        1
    );
}

#[test]
fn simkl_calendar_only_shows_titles_on_the_users_lists() {
    let scenario = Scenario::start();
    let fake = simkl::serve(&scenario.world);
    fake.plantowatch.lock().unwrap().push(json!({
        "status": "plantowatch",
        "movie": {"title": "Listed Movie", "year": 2026, "ids": {"simkl": 11, "imdb": "tt11"}}
    }));
    scenario.world.respond(
        "data.simkl.in",
        "GET",
        "/calendar/v2/2026/9/tv.json",
        200,
        json!({"metadata": {}, "calendar": []}),
    );
    scenario.world.respond(
        "data.simkl.in",
        "GET",
        "/calendar/v2/2026/9/anime.json",
        200,
        json!({"metadata": {}, "calendar": []}),
    );
    scenario.world.respond("data.simkl.in", "GET", "/calendar/v2/2026/9/movie_release.json", 200, json!({
        "metadata": {
            "11": {"title": "Listed Movie", "poster": "ab/cd", "ids": {"simkl": 11, "imdb": "tt11"}},
            "12": {"title": "Unlisted Movie", "poster": "ef/gh", "ids": {"simkl": 12, "imdb": "tt12"}}
        },
        "calendar": [
            {"simkl_id": 11, "date": "2026-09-18T00:00:00Z"},
            {"simkl_id": 12, "date": "2026-09-19T00:00:00Z"}
        ]
    }));
    let profile = json!({"id": "p1", "name": "Me", "simklAccessToken": "stok", "integrationLibrarySource": "simkl"});
    let app = scenario.app_with_profile(profile.clone());
    app.dispatch(json!({"type": "libraryHydrateRequested"}));

    let items = month(&app, profile);

    let ids: Vec<&str> = items
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["contentId"].as_str())
        .collect();
    assert_eq!(ids, vec!["tt11"], "{items}");
}

#[test]
fn mdblist_calendar_maps_episodes_and_movies() {
    let scenario = Scenario::start();
    scenario.world.respond("api.mdblist.com", "GET", "/calendar/events", 200, json!({"events": [
        {"type": "episode", "id": "e1", "title": "Show", "episode_title": "Pilot", "show_tmdb": 500,
         "season_number": 1, "episode_number": 1, "start": "2026-09-05", "is_season_finale": false},
        {"type": "movie", "id": "m1", "title": "Film", "tmdb": 600, "start": "2026-09-25"}
    ]}));
    let profile = json!({"id": "p1", "name": "Me", "mdblistAccessToken": "mtok", "integrationLibrarySource": "mdblist"});
    let app = scenario.app_with_profile(profile.clone());

    let items = month(&app, profile);

    let ids: Vec<&str> = items
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["contentId"].as_str())
        .collect();
    assert!(
        ids.contains(&"tmdb:500") && ids.contains(&"tmdb:600"),
        "{items}"
    );
    let call = &scenario
        .world
        .calls("api.mdblist.com", "GET", "/calendar/events")[0];
    assert_eq!(call.query_param("start").as_deref(), Some("2026-09-01"));
    assert_eq!(call.query_param("end").as_deref(), Some("2026-09-30"));
}

#[test]
fn anilist_calendar_keeps_only_episodes_airing_in_the_month() {
    let scenario = Scenario::start();
    let in_month = 1_789_000_000i64;
    let outside = in_month + 90 * 86_400;
    scenario.world.on("graphql.anilist.co", "POST", "/", move |request| {
        let query = request.json()["query"].as_str().unwrap_or("").to_owned();
        if query.contains("RELEASING") {
            super::support::Reply::json(200, json!({"data": {"Page": {"media": [{
                "id": 7, "title": {"romaji": "Nana", "english": "Seven"}, "coverImage": {"large": "https://cdn.example/7.jpg"},
                "airingSchedule": {"nodes": [{"airingAt": in_month, "episode": 3}, {"airingAt": outside, "episode": 4}]}
            }]}}}))
        } else {
            super::support::Reply::json(200, json!({"data": {"Page": {"media": []}}}))
        }
    });
    let profile = json!({"id": "p1", "name": "Me", "anilistAccessToken": "h.eyJzdWIiOiI0MiJ9.s", "integrationLibrarySource": "anilist"});
    let app = scenario.app_with_profile(profile.clone());

    let items = month(&app, profile);

    assert_eq!(items.as_array().map(Vec::len), Some(1), "{items}");
    assert_eq!(items[0]["episodeNumber"], 3);
    assert_eq!(items[0]["contentId"], "anilist:7");
}
