use super::super::trakt::trakt_playback_item_to_library;
use super::super::*;
use serde_json::{Value, json};

#[test]
fn trakt_ids_support_stremio_episode_ids() {
    assert_eq!(
        trakt_ids_from_content_id_json("tt1234567:1:2")
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .and_then(|ids| ids.get("imdb").and_then(Value::as_str).map(str::to_owned))
            .as_deref(),
        Some("tt1234567")
    );
    assert_eq!(
        trakt_ids_from_content_id_json("tmdb:42:1:2")
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .and_then(|ids| ids.get("tmdb").and_then(Value::as_i64)),
        Some(42)
    );
}

#[test]
fn trakt_token_expiry_stays_in_epoch_seconds() {
    assert_eq!(trakt_token_expires_at(1_700_000_000, 3_600), 1_700_003_300);
}

#[test]
fn trakt_urls_accept_only_supported_routes() {
    assert_eq!(
        trakt_scrobble_url("pause").as_deref(),
        Some("https://api.trakt.tv/scrobble/pause")
    );
    assert_eq!(trakt_scrobble_url("delete"), None);
    assert_eq!(
        trakt_playback_url(Some("series")).as_deref(),
        Some("https://api.trakt.tv/sync/playback/episodes")
    );
    assert_eq!(trakt_playback_url(Some("unknown")), None);
}

#[test]
fn trakt_playback_tmdb_show_keeps_a_resolvable_episode_id() {
    let item = json!({
        "progress": 50.0,
        "paused_at": "2026-07-21T00:00:00.000Z",
        "show": {"title": "Show", "runtime": 45, "ids": {"tmdb": 42}},
        "episode": {"season": 1, "number": 2, "runtime": 45}
    });
    let result = trakt_playback_item_to_library(&item).expect("playback item");
    assert_eq!(result["id"], "tmdb:42");
    assert_eq!(result["lastVideoId"], "tmdb:42:1:2");
}

#[test]
fn trakt_watched_shows_create_continue_watching_items() {
    let watched = json!([{
        "last_watched_at": "2026-07-21T00:00:00.000Z",
        "completed": 4,
        "show": {"title": "Example", "aired_episodes": 8, "ids": {"imdb": "tt42"}},
        "seasons": [{"number": 1, "episodes": [
            {"number": 4, "last_watched_at": "2026-07-21T00:00:00.000Z"}
        ]}]
    }]);
    let items: Value = serde_json::from_str(
        &trakt_watched_shows_to_items_json(&watched.to_string()).expect("items"),
    )
    .unwrap();
    assert_eq!(items[0]["id"], "tt42");
    assert_eq!(items[0]["lastVideoId"], "tt42:1:4");
    assert_eq!(items[0]["timeOffset"], 1);
}

#[test]
fn trakt_up_next_items_use_the_source_next_episode() {
    let response = json!([{
        "show": {"title": "Example", "ids": {"imdb": "tt42"}},
        "progress": {
            "last_watched_at": "2026-07-21T00:00:00.000Z",
            "next_episode": {"season": 1, "number": 3, "title": "Episode Three"}
        }
    }]);
    let items: Value =
        serde_json::from_str(&trakt_up_next_to_items_json(&response.to_string()).expect("items"))
            .unwrap();
    assert_eq!(items[0]["lastVideoId"], "tt42:1:3");
    assert_eq!(items[0]["lastEpisodeName"], "Episode Three");
    assert_eq!(items[0]["continueWatchingBadge"], "upNext");
}

#[test]
fn trakt_watched_shows_without_episode_timestamps_use_latest_episode() {
    let watched = json!([{
        "last_watched_at": "2026-07-21T00:00:00.000Z",
        "completed": 3,
        "show": {"title": "Example", "aired_episodes": 12, "ids": {"imdb": "tt42"}},
        "seasons": [
            {"number": 1, "episodes": [{"number": 3}]},
            {"number": 9, "episodes": [{"number": 3}]}
        ]
    }]);
    let items: Value = serde_json::from_str(
        &trakt_watched_shows_to_items_json(&watched.to_string()).expect("items"),
    )
    .unwrap();
    assert_eq!(items[0]["lastVideoId"], "tt42:9:3");
}

#[test]
fn trakt_watched_shows_use_furthest_episode_over_newer_earlier_rewatch() {
    let watched = json!([{
        "last_watched_at": "2026-07-21T00:00:00.000Z",
        "completed": 3,
        "show": {"title": "Example", "aired_episodes": 12, "ids": {"imdb": "tt42"}},
        "seasons": [
            {"number": 1, "episodes": [{"number": 2, "last_watched_at": "2026-07-21T00:00:00.000Z"}]},
            {"number": 9, "episodes": [{"number": 2, "last_watched_at": "2026-07-01T00:00:00.000Z"}]}
        ]
    }]);
    let items: Value = serde_json::from_str(
        &trakt_watched_shows_to_items_json(&watched.to_string()).expect("items"),
    )
    .unwrap();
    assert_eq!(items[0]["lastVideoId"], "tt42:9:2");
}

#[test]
fn trakt_playback_dedup_keeps_the_furthest_watched_episode() {
    let items = json!([
        {"id": "tt42", "lastEpisodeSeason": 1, "lastEpisodeNumber": 1, "savedAt": "2026-07-22T00:00:00.000Z"},
        {"id": "tt42", "lastEpisodeSeason": 1, "lastEpisodeNumber": 2, "savedAt": "2026-07-21T00:00:00.000Z", "continueWatchingBadge": "upNext"}
    ]);
    let result: Value = serde_json::from_str(
        &trakt_playback_items_dedup_json(&items.to_string()).expect("deduped items"),
    )
    .unwrap();
    assert_eq!(result[0]["lastEpisodeNumber"], 2);
}

#[test]
fn history_request_builds_show_seasons_from_episode_ids() {
    let request = trakt_history_request_json(
            r#"{"id":"tt1234567","name":"Show","type":"series","poster":null}"#,
            r#"[{"id":"tt1234567:1:2","name":null,"season":null,"number":null,"released":null,"thumbnail":null}]"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("history request");

    assert_eq!(
        request
            .get("shows")
            .and_then(Value::as_array)
            .and_then(|shows| shows.first())
            .and_then(|show| show.get("seasons"))
            .and_then(Value::as_array)
            .and_then(|seasons| seasons.first())
            .and_then(|season| season.get("number"))
            .and_then(Value::as_i64),
        Some(1)
    );
    assert!(request.get("movies").is_none());
}

#[test]
fn trakt_oauth_error_code_extracts_structured_error() {
    assert_eq!(
        trakt_oauth_error_code(r#"{"error":"authorization_pending"}"#).as_deref(),
        Some("authorization_pending")
    );
    assert_eq!(trakt_oauth_error_code("{}"), None);
}

#[test]
fn trakt_mark_watched_body_groups_episodes_by_show_and_dedupes() {
    let body = trakt_mark_watched_body_json(
        &json!([
            "tt1234567:1:1",
            "tt1234567:1:2",
            "tt1234567:1:1",
            "tt7654321"
        ])
        .to_string(),
    )
    .and_then(|json| serde_json::from_str::<Value>(&json).ok())
    .expect("body");

    let movies = body["movies"].as_array().unwrap();
    assert_eq!(movies.len(), 1);
    assert_eq!(movies[0]["ids"]["imdb"], "tt7654321");

    let shows = body["shows"].as_array().unwrap();
    assert_eq!(shows.len(), 1);
    assert_eq!(shows[0]["ids"]["imdb"], "tt1234567");
    let seasons = shows[0]["seasons"].as_array().unwrap();
    assert_eq!(seasons.len(), 1);
    assert_eq!(seasons[0]["number"], 1);
    // The duplicate tt1234567:1:1 must not produce a duplicate episode entry.
    let episodes = seasons[0]["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 2);
    assert_eq!(episodes[0]["number"], 1);
    assert_eq!(episodes[1]["number"], 2);
}

#[test]
fn trakt_mark_watched_body_is_none_for_unrecognized_ids() {
    assert_eq!(
        trakt_mark_watched_body_json(&json!(["not-an-id"]).to_string()),
        None
    );
}

#[test]
fn trakt_sync_item_policy_normalizes_identity_and_release_date() {
    let meta = trakt_sync_item_to_meta_json(
        &json!({
            "item": { "show": { "title": "Show", "year": 2025, "ids": { "imdb": "tt1" } } },
            "type": "series",
            "unknownName": "Unknown",
        })
        .to_string(),
    )
    .and_then(|value| serde_json::from_str::<Value>(&value).ok())
    .unwrap();
    assert_eq!(meta["id"], "tt1");
    assert_eq!(meta["released"], "2025-01-01");
}

#[test]
fn trakt_playback_deletion_matches_shared_content_identity() {
    let ids: Value = serde_json::from_str(&trakt_playback_delete_ids_json(&json!({
            "contentId":"tmdb:42",
            "items":[{"id":1,"show":{"ids":{"tmdb":42}}},{"id":2,"movie":{"ids":{"imdb":"tt1"}}}],
        }).to_string()).unwrap()).unwrap();
    assert_eq!(ids, json!([1]));
}

#[test]
fn trakt_activity_diff_flags_only_moved_timestamps() {
    let result: Value = serde_json::from_str(&trakt_activity_diff_json(&json!({
            "previous": { "movies": { "paused_at": "t1", "watchlisted_at": "t1", "watched_at": "t1" }, "episodes": { "paused_at": "t1", "watched_at": "t1" }, "shows": { "watchlisted_at": "t1" } },
            "current": { "movies": { "paused_at": "t1", "watchlisted_at": "t1", "watched_at": "t2" }, "episodes": { "paused_at": "t1", "watched_at": "t1" }, "shows": { "watchlisted_at": "t1" } },
            "hasPlayback": true,
            "hasWatchlistMovies": true,
            "hasWatchlistShows": true,
            "hasWatchedMovies": true,
            "hasWatchedShows": true,
        }).to_string()).unwrap()).unwrap();
    assert_eq!(result["playbackChanged"], false);
    assert_eq!(result["watchlistMoviesChanged"], false);
    assert_eq!(result["watchlistShowsChanged"], false);
    assert_eq!(result["watchedMoviesChanged"], true);
    assert_eq!(result["watchedShowsChanged"], false);
}

#[test]
fn trakt_activity_diff_forces_full_when_nothing_cached_yet() {
    let result: Value = serde_json::from_str(
        &trakt_activity_diff_json(
            &json!({
                "previous": null,
                "current": { "movies": {}, "episodes": {}, "shows": {} },
                "hasPlayback": false,
                "hasWatchlistMovies": false,
                "hasWatchlistShows": false,
                "hasWatchedMovies": false,
                "hasWatchedShows": false,
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result["playbackChanged"], true);
    assert_eq!(result["watchedShowsChanged"], true);
}
