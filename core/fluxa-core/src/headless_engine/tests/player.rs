use super::super::*;
use serde_json::{Value, json};

#[test]
fn player_load_streams_uses_effect_completion_without_reordering_streams() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"playerLoadStreamsRequested","contentType":"movie","id":"tt1","currentVideoId":"tt1","initialStreamIndex":1}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(requested["effects"][0]["type"], "loadStreams");

    let effect_id = requested["effects"][0]["id"].as_str().unwrap();
    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": [
                    { "title": "A", "playableUrl": "http://a" },
                    { "title": "B", "playableUrl": "http://b" }
                ]
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(completed["state"]["player"]["currentStreamIndex"], 1);
    assert_eq!(completed["state"]["player"]["currentUrl"], "http://b");
    assert_eq!(
        completed["state"]["player"]["currentStreams"][0]["title"],
        "A"
    );
    assert!(destroy_headless_engine(handle));
}

#[test]
fn player_load_streams_saves_outgoing_episode_progress_before_switching() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            &json!({
                "type": "playerLoadStreamsRequested",
                "contentType": "series",
                "id": "tt1",
                "currentVideoId": "tt1:1:5",
                "initialVideoId": "tt1:1:6",
                "outgoingProgress": {
                    "timeOffset": 1200000,
                    "duration": 1300000,
                    "lastEpisodeSeason": 1,
                    "lastEpisodeNumber": 5
                }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    let effects = requested["effects"].as_array().unwrap();
    assert!(effects.iter().any(|e| e["type"] == "writePlaybackProgress"
        && e["payload"]["progress"]["lastVideoId"] == "tt1:1:5"
        && e["payload"]["progress"]["lastEpisodeNumber"] == 5));
    assert!(effects.iter().any(|e| e["type"] == "loadStreams"));
    assert!(destroy_headless_engine(handle));
}

#[test]
fn player_resolve_playback_emits_torrent_or_direct_platform_effects() {
    let handle = create_headless_engine("{}");
    let torrent: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"playerResolvePlaybackRequested","url":"stremio://torrent/abc","stream":{"title":"T"},"currentVideoId":"tt1","title":"Movie"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(torrent["effects"][0]["type"], "startTorrentStream");
    let effect_id = torrent["effects"][0]["id"].as_str().unwrap();

    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": { "url": "http://127.0.0.1:8090/stream" }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        completed["state"]["player"]["resolvedUrl"],
        "http://127.0.0.1:8090/stream"
    );

    let direct: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"playerResolvePlaybackRequested","url":"https://video.example/file.mp4","title":"Movie"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(
        direct["state"]["player"]["resolvedUrl"],
        "https://video.example/file.mp4"
    );
    assert_eq!(direct["effects"][0]["type"], "stopTorrent");
    assert!(destroy_headless_engine(handle));
}

#[test]
fn next_episode_card_shown_prefetches_streams_and_load_streams_consumes_cache() {
    let handle = create_headless_engine("{}");

    // 1. Next episode card shown for episode tt1:1:2
    let prefetch_requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"playerNextEpisodeCardShown","contentType":"series","seriesId":"tt1","nextVideoId":"tt1:1:2","title":"Show","language":"en"}"#,
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        prefetch_requested["effects"][0]["type"],
        "prefetchNextEpisodeStreams"
    );
    assert_eq!(
        prefetch_requested["effects"][0]["payload"]["nextVideoId"],
        "tt1:1:2"
    );
    assert_eq!(
        prefetch_requested["state"]["player"]["prefetchingNextVideoId"],
        "tt1:1:2"
    );

    // Duplicate card-shown dispatch must not change prefetching state.
    let duplicate: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"playerNextEpisodeCardShown","contentType":"series","seriesId":"tt1","nextVideoId":"tt1:1:2"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    // Guard works: nothing in player changed, so it's correctly absent from this patch
    // entirely (no new prefetch effect was queued either).
    assert!(duplicate["state"]["player"].is_null());

    // 2. Platform completes the prefetch with streams for tt1:1:2
    let effect_id = prefetch_requested["effects"][0]["id"].as_str().unwrap();
    let prefetch_done: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": {
                    "streams": [
                        { "title": "S", "playableUrl": "http://ep2" }
                    ]
                }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        prefetch_done["state"]["player"]["prefetchedNextEpisode"]["videoId"],
        "tt1:1:2"
    );
    assert_eq!(
        prefetch_done["state"]["player"]["prefetchedNextEpisode"]["streams"][0]["title"],
        "S"
    );
    assert!(prefetch_done["state"]["player"]["prefetchingNextVideoId"].is_null());

    // 3. User navigates to ep2 — load streams without passing initial_streams.
    //    Core must inject the prefetched streams and use_initial_streams = true.
    let load: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"playerLoadStreamsRequested","contentType":"series","id":"tt1","currentVideoId":"tt1:1:2"}"#,
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(load["effects"][0]["type"], "loadStreams");
    // useInitialStreams = true means the platform skips the network fetch
    assert_eq!(load["effects"][0]["payload"]["useInitialStreams"], true);
    // Cache must be consumed (cleared) after use
    assert!(load["state"]["player"]["prefetchedNextEpisode"].is_null());

    assert!(destroy_headless_engine(handle));
}
