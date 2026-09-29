use super::support::{Scenario, addon_manifest};
use serde_json::{Value, json};

fn series_addon() -> Value {
    addon_manifest(
        "addon.test",
        json!(["catalog", "meta", "stream"]),
        json!(["series"]),
        json!([{"type": "series", "id": "top", "name": "Top"}]),
    )
}

fn serve(scenario: &Scenario) {
    let host = "addon.test";
    scenario
        .world
        .respond(host, "GET", "/manifest.json", 200, series_addon());
    scenario.world.respond(
        host,
        "GET",
        "/meta/series/tt5.json",
        200,
        json!({"meta": {
            "id": "tt5", "type": "series", "name": "Show Five",
            "videos": [
                {"id": "tt5:1:1", "season": 1, "episode": 1, "title": "One"},
                {"id": "tt5:1:2", "season": 1, "episode": 2, "title": "Two"},
                {"id": "tt5:2:1", "season": 2, "episode": 1, "title": "Three"}
            ]
        }}),
    );
    scenario.world.respond(
        host,
        "GET",
        "/stream/series/tt5:1:2.json",
        200,
        json!({"streams": [
            {"name": "A", "url": "https://cdn.example/a.mp4"},
            {"name": "B", "url": "https://cdn.example/b.mp4"}
        ]}),
    );
}

fn installed(scenario: &Scenario) -> super::support::App {
    serve(scenario);
    let app = scenario.app();
    app.dispatch(json!({"type": "addonInstallRequested", "transportUrl": "https://addon.test/manifest.json"}));
    app
}

#[test]
fn player_loads_streams_for_the_current_episode() {
    let scenario = Scenario::start();
    let app = installed(&scenario);

    app.dispatch(json!({
        "type": "playerLoadStreamsRequested", "contentType": "series", "id": "tt5:1:2",
        "currentVideoId": "tt5:1:2", "language": "en", "profile": null
    }));

    let state = app.state();
    let streams = state
        .pointer("/player/currentStreams")
        .cloned()
        .unwrap_or(Value::Null);
    assert_eq!(
        streams.as_array().map(Vec::len),
        Some(2),
        "{}",
        state["player"]
    );
    assert_eq!(streams[0]["url"], "https://cdn.example/a.mp4");
    assert_eq!(
        scenario
            .world
            .calls("addon.test", "GET", "/stream/series/tt5:1:2.json")
            .len(),
        1
    );
}

#[test]
fn player_uses_initial_streams_without_asking_the_addon() {
    let scenario = Scenario::start();
    let app = installed(&scenario);

    app.dispatch(json!({
        "type": "playerLoadStreamsRequested", "contentType": "series", "id": "tt5:1:2",
        "currentVideoId": "tt5:1:2", "initialVideoId": "tt5:1:2",
        "initialStreams": [{"name": "Prefetched", "url": "https://cdn.example/p.mp4"}],
        "language": "en", "profile": null
    }));

    assert_eq!(
        app.at("/player/currentStreams/0/url"),
        "https://cdn.example/p.mp4"
    );
    assert!(
        scenario
            .world
            .calls("addon.test", "GET", "/stream/series/tt5:1:2.json")
            .is_empty()
    );
}

#[test]
fn player_with_no_streams_reports_a_failure() {
    let scenario = Scenario::start();
    let app = installed(&scenario);
    scenario.world.respond(
        "addon.test",
        "GET",
        "/stream/series/tt5:1:2.json",
        200,
        json!({"streams": []}),
    );

    app.dispatch(json!({
        "type": "playerLoadStreamsRequested", "contentType": "series", "id": "tt5:1:2",
        "currentVideoId": "tt5:1:2", "language": "en", "profile": null
    }));

    assert_eq!(app.at("/player/playerError"), "no_source");
}

#[test]
#[ignore = "effect fetchSeasonEpisodes has no native implementation yet"]
fn detail_season_selects_that_seasons_episodes() {
    let scenario = Scenario::start();
    let app = installed(&scenario);
    app.dispatch(json!({"type": "detailLoadRequested", "contentType": "series", "id": "tt5", "language": "en", "profile": null}));

    app.dispatch(json!({"type": "detailSeasonRequested", "seriesId": "tt5", "season": 2, "profile": null, "language": "en"}));

    assert_eq!(app.at("/detail/error"), Value::Null);
    assert_eq!(
        app.at("/detail/seasonEpisodes").as_array().map(Vec::len),
        Some(1)
    );
}

#[test]
fn continue_watching_playback_resolves_the_saved_episode() {
    let scenario = Scenario::start();
    let app = installed(&scenario);
    app.dispatch(json!({
        "type": "savePlaybackProgressRequested", "profile": null,
        "meta": {"id": "tt5", "type": "series", "name": "Show Five"},
        "timeOffset": 300, "duration": 2400, "lastVideoId": "tt5:1:2"
    }));

    app.dispatch(json!({"type": "libraryHydrateRequested"}));
    app.dispatch(json!({
        "type": "continueWatchingPlaybackRequested",
        "item": app.at("/library/continueWatching/0"), "language": "en", "profile": null
    }));

    assert_eq!(
        app.at("/player/directPlaybackTarget/streams/0/url"),
        "https://cdn.example/a.mp4"
    );
    assert_eq!(
        app.at("/player/directPlaybackTarget/hasStreamProviders"),
        true
    );
    assert_eq!(
        scenario
            .world
            .calls("addon.test", "GET", "/stream/series/tt5:1:2.json")
            .len(),
        1
    );
}

#[test]
fn activating_a_profile_makes_it_the_active_one() {
    let scenario = Scenario::start();
    let app = scenario.app();

    app.dispatch(json!({"type": "profileActivated", "profile": {"id": "p2", "name": "Kid"}}));

    assert_eq!(app.at("/library/activeProfileId"), "p2");
    assert_eq!(app.at("/profile/active/name"), "Kid");
}
