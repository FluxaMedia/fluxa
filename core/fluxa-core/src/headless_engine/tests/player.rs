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
fn continue_watching_reuses_saved_stream_without_direct_addon_discovery() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
        &headless_engine_dispatch_json(
            handle,
            &json!({
                "type": "continueWatchingPlaybackRequested",
                "item": {
                    "id": "tt-show",
                    "type": "series",
                    "name": "Show",
                    "lastVideoId": "tt-show:2:4",
                    "lastStreamUrl": "stremio://torrent/abc/7",
                    "lastStreamTitle": "Saved torrent",
                    "lastStream": {"infoHash": "abc", "fileIdx": 7, "title": "Saved torrent"}
                }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    let effect = &requested["effects"][0];
    assert_eq!(effect["type"], "loadStreams");
    assert_eq!(effect["payload"]["useInitialStreams"], true);
    assert_eq!(effect["payload"]["id"], "tt-show:2:4");
    assert_eq!(effect["payload"]["initialStreams"][0]["infoHash"], "abc");
    assert_eq!(effect["payload"]["initialStreams"][0]["fileIdx"], 7);

    let effect_id = effect["id"].as_str().unwrap();
    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": effect_id,
                "status": "ok",
                "value": effect["payload"]["initialStreams"].clone()
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        completed["state"]["player"]["currentUrl"],
        "stremio://torrent/abc/7"
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

fn dispatch(handle: u64, action: Value) -> Value {
    serde_json::from_str(&headless_engine_dispatch_json(handle, &action.to_string()).unwrap())
        .unwrap()
}

#[test]
fn clicked_cue_sets_subtitle_delay_from_captured_time() {
    let handle = create_headless_engine("{}");
    let captured = dispatch(
        handle,
        json!({
            "type": "subtitleSyncCaptured",
            "subtitleText": "00:00:10,000 --> 00:00:11,000\nHi\n\n00:00:20,000 --> 00:00:21,000\nBye",
            "currentTime": 11.6
        }),
    );
    let sync = &captured["state"]["player"]["subtitleSync"];
    assert_eq!(sync["capturedTime"], 11.6);
    assert_eq!(sync["cues"].as_array().map(Vec::len), Some(2));

    let applied = dispatch(
        handle,
        json!({ "type": "subtitleSyncCueSelected", "cueStart": 10.0 }),
    );
    let sync = &applied["state"]["player"]["subtitleSync"];
    assert_eq!(sync["delaySeconds"], 1.6);
    assert_eq!(sync["cues"].as_array().map(Vec::len), Some(0));
}

#[test]
fn cue_selection_without_capture_keeps_delay() {
    let handle = create_headless_engine("{}");
    let result = dispatch(
        handle,
        json!({ "type": "subtitleSyncCueSelected", "cueStart": 10.0 }),
    );
    assert_eq!(
        result["state"]["player"]["subtitleSync"]["delaySeconds"],
        0.0
    );
}

#[test]
fn estimate_records_delay_and_confidence() {
    let handle = create_headless_engine("{}");
    let result = dispatch(
        handle,
        json!({
            "type": "subtitleSyncEstimated",
            "subtitleText": "00:00:10,000 --> 00:00:10,800\na\n\n00:00:20,000 --> 00:00:20,800\nb\n\n00:00:30,000 --> 00:00:30,800\nc",
            "speechIntervals": [
                {"start": 12.0, "end": 12.8},
                {"start": 22.0, "end": 22.8},
                {"start": 32.0, "end": 32.8}
            ]
        }),
    );
    let sync = &result["state"]["player"]["subtitleSync"];
    assert!((sync["delaySeconds"].as_f64().unwrap() - 2.0).abs() <= 0.1);
    assert!(sync["confidence"].as_f64().unwrap() > 0.0);
}

fn srt(times: &[(f64, f64)]) -> String {
    let stamp = |t: f64| {
        let ms = (t * 1000.0).round() as i64;
        format!(
            "{:02}:{:02}:{:02},{:03}",
            ms / 3_600_000,
            ms / 60_000 % 60,
            ms / 1000 % 60,
            ms % 1000
        )
    };
    times
        .iter()
        .enumerate()
        .map(|(i, (s, e))| format!("{}\n{} --> {}\nline {i}\n", i + 1, stamp(*s), stamp(*e)))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn embedded_track_fixes_offset_and_framerate_drift() {
    let embedded: Vec<(f64, f64)> = (0..40)
        .map(|i| {
            (
                20.0 + i as f64 * 37.0 + (i % 3) as f64,
                22.5 + i as f64 * 37.0 + (i % 3) as f64,
            )
        })
        .collect();
    let external: Vec<(f64, f64)> = embedded
        .iter()
        .map(|(s, e)| ((s - 3.0) / 1.0427, (e - 3.0) / 1.0427))
        .collect();
    let handle = create_headless_engine("{}");
    let result = dispatch(
        handle,
        json!({
            "type": "subtitleSyncEstimated",
            "subtitleText": srt(&external),
            "referenceText": srt(&embedded)
        }),
    );
    let sync = &result["state"]["player"]["subtitleSync"];
    assert!((sync["scale"].as_f64().unwrap() - 1.0427).abs() < 0.001);
    let first = sync["retimedCues"][0]["start"].as_f64().unwrap();
    assert!((first - embedded[0].0).abs() < 0.3);
}

#[test]
fn unrelated_reference_leaves_delay_untouched() {
    let handle = create_headless_engine("{}");
    let result = dispatch(
        handle,
        json!({
            "type": "subtitleSyncEstimated",
            "subtitleText": srt(&[(10.0, 11.0), (30.0, 31.0), (55.0, 56.0)]),
            "speechIntervals": [{"start": 400.0, "end": 401.0}]
        }),
    );
    assert_eq!(result["state"], json!({}));
}

#[test]
fn audio_envelope_aligns_subtitles_when_embedded_track_is_unusable() {
    let frame = 0.05;
    let cues: Vec<(f64, f64)> = (0..30)
        .map(|i| {
            let start = 20.0 + i as f64 * 11.0 + ((i * 7) % 5) as f64 * 1.3;
            (start, start + 1.5 + (i % 3) as f64 * 0.5)
        })
        .collect();
    let mut energies = vec![0.002; 8000];
    for (start, end) in &cues {
        for i in ((start + 2.5) / frame) as usize..((end + 2.5) / frame) as usize {
            energies[i] = 0.1;
        }
    }
    let handle = create_headless_engine("{}");
    let result = dispatch(
        handle,
        json!({
            "type": "subtitleSyncAudioAnalyzed",
            "subtitleText": srt(&cues),
            "frameSeconds": frame,
            "energies": energies
        }),
    );
    let sync = &result["state"]["player"]["subtitleSync"];
    assert!((sync["delaySeconds"].as_f64().unwrap() - 2.5).abs() <= 0.1);
}
