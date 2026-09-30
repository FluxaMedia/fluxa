use super::*;

fn entry(video: &str, season: i64, episode: i64, position: f64, duration: f64, at: i64) -> Value {
    json!({
        "content_id": "tt1",
        "content_type": "series",
        "video_id": video,
        "season": season,
        "episode": episode,
        "position": position,
        "duration": duration,
        "last_watched": at,
    })
}

fn videos() -> Value {
    json!([
        { "id": "tt1:2:9", "season": 2, "episode": 9, "name": "Three Ghosts", "released": "2020-01-01T00:00:00Z" },
        { "id": "tt1:2:10", "season": 2, "episode": 10, "released": "2020-01-08T00:00:00Z" },
        { "id": "tt1:2:11", "season": 2, "episode": 11, "released": "2020-01-15T00:00:00Z" },
    ])
}

fn run(progress: Value, now: &str) -> Vec<Value> {
    let now_ms = chrono::DateTime::parse_from_rfc3339(now)
        .unwrap()
        .timestamp_millis();
    let args = json!({
        "watchProgress": progress,
        "metaById": { "tt1": { "name": "Show", "type": "series", "videos": videos() } },
        "nowMs": now_ms,
    });
    serde_json::from_str(&continue_watching_json(&args.to_string()).unwrap()).unwrap()
}

#[test]
fn a_half_watched_episode_stays_a_resume_row() {
    let items = run(
        json!([entry(
            "tt1:2:9",
            2,
            9,
            600_000.0,
            2_545_000.0,
            1_600_000_000_000
        )]),
        "2021-01-01T00:00:00Z",
    );
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["lastVideoId"], "tt1:2:9");
    assert_eq!(items[0]["lastEpisodeName"], "Three Ghosts");
    assert_eq!(items[0]["continueWatchingBadge"], Value::Null);
}

#[test]
fn a_finished_episode_becomes_the_next_one_and_stays_there() {
    let progress = json!([entry(
        "tt1:2:9",
        2,
        9,
        2_500_000.0,
        2_545_000.0,
        1_600_000_000_000
    )]);
    let items = run(progress.clone(), "2021-01-01T00:00:00Z");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["lastEpisodeNumber"], 10);
    assert_eq!(items[0]["continueWatchingBadge"], "upNext");

    let again = run(progress, "2021-01-02T00:00:00Z");
    assert_eq!(again[0]["lastEpisodeNumber"], 10);
}

#[test]
fn resuming_a_later_episode_suppresses_the_next_up_row() {
    let items = run(
        json!([
            entry("tt1:2:9", 2, 9, 2_500_000.0, 2_545_000.0, 1_600_000_000_000),
            entry("tt1:2:10", 2, 10, 300_000.0, 2_545_000.0, 1_600_000_100_000),
        ]),
        "2021-01-01T00:00:00Z",
    );
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["lastEpisodeNumber"], 10);
    assert_eq!(items[0]["continueWatchingBadge"], Value::Null);
}

#[test]
fn a_series_with_no_episode_left_drops_out() {
    let items = run(
        json!([entry(
            "tt1:2:11",
            2,
            11,
            2_500_000.0,
            2_545_000.0,
            1_600_000_000_000
        )]),
        "2021-01-01T00:00:00Z",
    );
    assert!(items.is_empty());
}

#[test]
fn an_unaired_next_episode_still_surfaces_by_default() {
    let items = run(
        json!([entry(
            "tt1:2:9",
            2,
            9,
            2_500_000.0,
            2_545_000.0,
            1_577_836_800_000
        )]),
        "2020-01-05T00:00:00Z",
    );
    assert_eq!(items[0]["lastEpisodeNumber"], 10);
}

#[test]
fn the_remaining_minute_matches_a_millisecond_precise_countdown() {
    let items = run(
        json!([entry(
            "tt1:2:9",
            2,
            9,
            1_400.0,
            1_321_000.0,
            1_600_000_000_000
        )]),
        "2021-01-01T00:00:00Z",
    );
    let time_offset = items[0]["timeOffset"].as_i64().unwrap();
    let duration = items[0]["duration"].as_i64().unwrap();
    assert_eq!((duration - time_offset) / 60, 21);
}

#[test]
fn rows_imported_in_the_same_batch_keep_a_stable_order() {
    let now_ms = chrono::DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z")
        .unwrap()
        .timestamp_millis();
    let watched_at = 1_785_017_916_000i64;
    let progress = json!([
        { "content_id": "ttB", "content_type": "series", "video_id": "ttB:1:1", "season": 1, "episode": 1,
          "position": 1000, "duration": 1000, "last_watched": watched_at },
        { "content_id": "ttA", "content_type": "series", "video_id": "ttA:1:1", "season": 1, "episode": 1,
          "position": 1000, "duration": 1000, "last_watched": watched_at },
    ]);
    let episodes = |id: &str| {
        json!([
            { "id": format!("{id}:1:1"), "season": 1, "episode": 1, "released": "2020-01-01T00:00:00Z" },
            { "id": format!("{id}:1:2"), "season": 1, "episode": 2, "released": "2020-01-08T00:00:00Z" },
        ])
    };
    let args = json!({
        "watchProgress": progress,
        "metaById": {
            "ttA": { "name": "A", "type": "series", "videos": episodes("ttA") },
            "ttB": { "name": "B", "type": "series", "videos": episodes("ttB") },
        },
        "nowMs": now_ms,
    });

    for _ in 0..8 {
        let items: Vec<Value> =
            serde_json::from_str(&continue_watching_json(&args.to_string()).unwrap()).unwrap();
        let ids: Vec<&str> = items
            .iter()
            .filter_map(|item| item["id"].as_str())
            .collect();
        assert_eq!(ids, ["ttA", "ttB"]);
    }
}

#[test]
fn a_barely_started_episode_is_not_stored_as_progress() {
    let items = run(
        json!([entry(
            "tt1:2:9",
            2,
            9,
            500.0,
            2_545_000.0,
            1_600_000_000_000
        )]),
        "2021-01-01T00:00:00Z",
    );
    assert!(items.is_empty());
}
