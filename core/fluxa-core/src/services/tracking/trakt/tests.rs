use super::*;
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
fn trakt_collection_body_uses_provider_resource_for_content_type() {
    let movie =
        trakt_collection_body_json(r#"{"idsJson":"{\"imdb\":\"tt1\"}","contentType":"movie"}"#)
            .and_then(|body| serde_json::from_str::<Value>(&body).ok())
            .unwrap();
    assert!(movie.get("movies").is_some());
    assert!(movie.get("shows").is_none());

    let series = trakt_collection_body_json(r#"{"idsJson":"{\"tmdb\":42}","contentType":"show"}"#)
        .and_then(|body| serde_json::from_str::<Value>(&body).ok())
        .unwrap();
    assert!(series.get("shows").is_some());
    assert!(series.get("movies").is_none());
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
fn items_without_imdb_or_tmdb_keep_their_tvdb_id() {
    let source = json!({"ids": {"tvdb": 81189, "trakt": 1}});
    assert_eq!(trakt_id_from_source(&source).as_deref(), Some("tvdb:81189"));
}

#[test]
fn slug_ids_resolve_on_write() {
    let ids: Value =
        serde_json::from_str(&trakt_ids_from_content_id_json("slug:breaking-bad").unwrap())
            .unwrap();
    assert_eq!(ids["slug"], "breaking-bad");
}

#[test]
fn multi_season_addon_episodes_map_to_trakt_absolute_numbering() {
    let addon: Vec<Value> = (1..=2)
        .flat_map(|season| {
            (1..=2).map(move |n| json!({"id": format!("tt9:{season}:{n}"), "season": season, "episode": n, "title": format!("Ep {season}{n}")}))
        })
        .collect();
    let trakt = json!([{"number": 1, "episodes": [
        {"season": 1, "number": 1, "title": "Ep 11"},
        {"season": 1, "number": 2, "title": "Ep 12"},
        {"season": 1, "number": 3, "title": "Ep 21"},
        {"season": 1, "number": 4, "title": "Ep 22"}
    ]}]);
    let mapped = trakt_remap_video_ids_json(
        &json!({"videoIds": ["tt9:2:1", "tt9:2:2"], "addonEpisodes": addon, "traktSeasons": trakt})
            .to_string(),
    )
    .unwrap();
    assert_eq!(mapped, r#"["tt9:1:3","tt9:1:4"]"#);
}

#[test]
fn matching_season_structure_keeps_video_ids() {
    let addon = json!([{"id": "tt9:1:1", "season": 1, "episode": 1, "title": "A"}]);
    let trakt = json!([{"number": 1, "episodes": [{"season": 1, "number": 1, "title": "A"}]}]);
    let mapped = trakt_remap_video_ids_json(
        &json!({"videoIds": ["tt9:1:1"], "addonEpisodes": addon, "traktSeasons": trakt})
            .to_string(),
    )
    .unwrap();
    assert_eq!(mapped, r#"["tt9:1:1"]"#);
}

#[test]
fn trakt_absolute_numbers_map_back_to_addon_seasons() {
    let addon: Vec<Value> = (1..=2)
        .flat_map(|season| {
            (1..=2).map(move |n| json!({"id": format!("tt9:{season}:{n}"), "season": season, "episode": n, "title": format!("Ep {season}{n}")}))
        })
        .collect();
    let trakt = json!([{"number": 1, "episodes": [
        {"season": 1, "number": 1, "title": "Ep 11"},
        {"season": 1, "number": 2, "title": "Ep 12"},
        {"season": 1, "number": 3, "title": "Ep 21"},
        {"season": 1, "number": 4, "title": "Ep 22"}
    ]}]);
    let mapped = trakt_remap_video_ids_json(
        &json!({"direction": "toAddon", "videoIds": ["tt9:1:3", "tt9:1:1"], "addonEpisodes": addon, "traktSeasons": trakt})
            .to_string(),
    )
    .unwrap();
    assert_eq!(mapped, r#"["tt9:2:1","tt9:1:1"]"#);
}

#[test]
fn unmatched_lengths_leave_ids_alone_when_titles_are_generic() {
    let addon = json!([
        {"id": "tt9:1:1", "season": 1, "episode": 1, "title": "Episode 1"},
        {"id": "tt9:1:2", "season": 1, "episode": 2, "title": "Episode 2"}
    ]);
    let trakt = json!([{"number": 1, "episodes": [
        {"season": 1, "number": 1, "title": "Episode 1"},
        {"season": 1, "number": 2, "title": "Episode 2"},
        {"season": 1, "number": 3, "title": "Episode 3"}
    ]}]);
    let mapped = trakt_remap_video_ids_json(
        &json!({"videoIds": ["tt9:1:2"], "addonEpisodes": addon, "traktSeasons": trakt})
            .to_string(),
    )
    .unwrap();
    assert_eq!(mapped, r#"["tt9:1:2"]"#);
}
