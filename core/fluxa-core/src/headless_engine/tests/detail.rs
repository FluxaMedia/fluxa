use super::super::*;
use serde_json::{Value, json};

#[test]
fn detail_load_emits_platform_effects_and_completion_updates_state() {
    let handle = create_headless_engine("{}");
    let result: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"detailLoadRequested","contentType":"movie","id":"tt1","language":"en"}"#,
        )
        .expect("dispatch"),
    )
    .expect("json");

    assert_eq!(result["state"]["detail"]["isLoading"], true);
    assert_eq!(result["effects"][0]["type"], "fetchMetaDetail");
    assert_eq!(result["effects"][1]["type"], "readPlaybackProgress");

    let effect_id = result["effects"][0]["id"].as_str().unwrap();
    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": { "meta": { "id": "tt1", "name": "Movie" } }
            })
            .to_string(),
        )
        .expect("complete"),
    )
    .expect("json");

    assert_eq!(completed["state"]["detail"]["isLoading"], false);
    assert_eq!(completed["state"]["detail"]["meta"]["name"], "Movie");
    assert!(destroy_headless_engine(handle));
}

#[test]
fn concurrent_local_state_request_does_not_drop_meta_detail_completion() {
    let handle = create_headless_engine("{}");
    let load: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"detailLoadRequested","contentType":"series","id":"tt1"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let effect_id = load["effects"][0]["id"].as_str().unwrap().to_string();

    headless_engine_dispatch_json(
        handle,
        r#"{"type":"detailLocalStateRequested","primaryId":"tt1","contentType":"series"}"#,
    )
    .unwrap();

    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": {
                    "meta": {
                        "id": "tt1",
                        "name": "Rick and Morty",
                        "videos": [{"id": "tt1:1:1", "season": 1, "episode": 1}]
                    }
                }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        completed["state"]["detail"]["meta"]["name"],
        "Rick and Morty"
    );
    assert_eq!(
        completed["state"]["detail"]["meta"]["videos"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(destroy_headless_engine(handle));
}

#[test]
fn detail_meta_trailers_are_normalized_in_core_before_tmdb_fallback() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"detailLoadRequested","contentType":"movie","id":"tt1","language":"en"}"#,
        )
        .expect("dispatch"),
    )
    .expect("json");
    let effect_id = requested["effects"][0]["id"].as_str().unwrap();
    let completed: Value = serde_json::from_str(
            &headless_engine_complete_effect_json(
                handle,
                &json!({
                    "effectId": effect_id,
                    "status": "ok",
                    "value": {
                        "meta": {
                            "id": "tt1",
                            "name": "Movie",
                            "trailers": [
                                { "source": "abc123", "type": "Trailer" },
                                { "title": "Featurette", "url": "https://video.example/f.mp4", "type": "Clip" }
                            ]
                        }
                    }
                })
                .to_string(),
            )
            .expect("complete"),
        )
        .expect("json");

    assert_eq!(
        completed["state"]["detail"]["trailers"][0]["url"],
        "https://www.youtube.com/watch?v=abc123"
    );
    assert_eq!(
        completed["state"]["detail"]["trailers"][1]["title"],
        "Featurette"
    );

    assert!(destroy_headless_engine(handle));
}

#[test]
fn detail_meta_link_trailers_become_direct_playback_sources() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"detailLoadRequested","contentType":"series","id":"tt0944947","language":"en"}"#,
            )
            .expect("dispatch"),
        )
        .expect("json");
    let effect_id = requested["effects"][0]["id"].as_str().unwrap();
    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": {
                    "meta": {
                        "id": "tt0944947",
                        "name": "Game of Thrones",
                        "links": [
                            {
                                "trailers": "https://video.fandango.com/trailer.mp4",
                                "provider": "Rotten Tomatoes 1080p"
                            },
                            {
                                "trailers": "https://imdb-video.media-imdb.com/trailer.m3u8",
                                "provider": "IMDb SD"
                            }
                        ]
                    }
                }
            })
            .to_string(),
        )
        .expect("complete"),
    )
    .expect("json");

    assert_eq!(
        completed["state"]["detail"]["trailers"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        completed["state"]["detail"]["trailers"][0]["title"],
        "Rotten Tomatoes 1080p"
    );
    assert_eq!(
        completed["state"]["detail"]["trailers"][1]["url"],
        "https://imdb-video.media-imdb.com/trailer.m3u8"
    );

    assert!(destroy_headless_engine(handle));
}

#[test]
fn detail_selected_addon_changes_visible_streams_without_mutating_raw_streams() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"detailStreamsRequested","contentType":"movie","requestIds":["tt1"],"detail":null,"seasonEpisodes":[],"language":"en"}"#,
            )
            .expect("dispatch"),
        )
        .expect("json");
    let effect_id = requested["effects"][0]["id"].as_str().unwrap();
    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": {
                    "streams": [
                        { "title": "A", "addonName": "One" },
                        { "title": "B", "addonName": "Two" },
                        { "title": "C", "addonName": "One" }
                    ],
                    "availableAddons": ["One", "Two"],
                    "hasStreamProviders": true
                }
            })
            .to_string(),
        )
        .expect("complete"),
    )
    .expect("json");
    assert_eq!(completed["state"]["detail"]["streams"][0]["title"], "A");
    assert_eq!(
        completed["state"]["detail"]["visibleStreams"][1]["title"],
        "B"
    );

    let selected: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"detailSelectedAddonChanged","addon":"one"}"#,
        )
        .expect("dispatch"),
    )
    .expect("json");

    assert_eq!(
        selected["state"]["detail"]["streams"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        selected["state"]["detail"]["visibleStreams"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        selected["state"]["detail"]["visibleStreams"][0]["title"],
        "A"
    );
    assert_eq!(
        selected["state"]["detail"]["visibleStreams"][1]["title"],
        "C"
    );
    assert!(destroy_headless_engine(handle));
}

#[test]
fn stale_detail_effect_completion_does_not_override_newer_state() {
    let handle = create_headless_engine("{}");
    let first: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            r#"{"type":"detailLoadRequested","contentType":"movie","id":"tt1"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let stale_effect_id = first["effects"][0]["id"].as_str().unwrap().to_string();

    headless_engine_dispatch_json(
        handle,
        r#"{"type":"detailLoadRequested","contentType":"movie","id":"tt2"}"#,
    )
    .unwrap();

    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": stale_effect_id,
                "status": "ok",
                "value": { "meta": { "id": "tt1", "name": "Old" } }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    // Stale completion is ignored, so this dispatch's patch doesn't touch detail at all —
    // its absence here is what proves tt2's state wasn't overridden by tt1's late result.
    assert!(completed["state"]["detail"].is_null());
    assert!(destroy_headless_engine(handle));
}
