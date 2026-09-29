use super::*;
use serde_json::Value;

#[test]
fn backend_selection_defaults_to_mpv() {
    let result: Value = serde_json::from_str(
        &player_backend_selection_json(r#"{"stream":{"url":"http://example.com/video.mp4"}}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(result["backend"], "mpv");
}

#[test]
fn stream_subtitles_result_has_one_shared_empty_and_populated_shape() {
    let populated: Value = serde_json::from_str(
        &stream_subtitles_result_json(
            r#"{"subtitles":[{"url":"https://sub.example/en.vtt","lang":"en"}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(populated["subtitles"][0]["lang"], "en");

    let empty: Value = serde_json::from_str(&stream_subtitles_result_json("{}").unwrap()).unwrap();
    assert_eq!(empty["subtitles"], serde_json::json!([]));
}

#[test]
fn backend_selection_keeps_mpv_for_hdr_and_audio_hints() {
    let result: Value = serde_json::from_str(
        &player_backend_selection_json(
            r#"{"stream":{"url":"http://example.com/video.mp4","hdr":true},"preferredPlayer":"mpv"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result["backend"], "mpv");
}

#[test]
fn stream_shell_plan_preserves_top_level_plugin_headers() {
    let result: Value = serde_json::from_str(
        &stream_shell_plan_json(
            r#"{
                "url":"https://cdn.example/video.m3u8",
                "headers":{"Referer":"https://hdfilmizle.example/","X-Test":"1"},
                "behaviorHints":{"requestHeaders":{"Authorization":"Bearer token"}}
            }"#,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        result["requestHeaders"]["Referer"].as_str(),
        Some("https://hdfilmizle.example/")
    );
    assert_eq!(result["requestHeaders"]["X-Test"].as_str(), Some("1"));
    assert_eq!(
        result["requestHeaders"]["Authorization"].as_str(),
        Some("Bearer token")
    );
}

#[test]
fn torrent_fallback_excludes_rejected_index_and_sorts_by_size() {
    let result: Value = serde_json::from_str(
        &torrent_fallback_file_policy_json(
            r#"{"rejectedIndex":1,"fileStats":[{"id":1,"path":"Big.mkv","length":1000000000},{"id":2,"path":"Small.mkv","length":500000000},{"id":3,"path":"Extras.mkv","length":200000000}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let fallback: Vec<i64> = result["fallbackFileIndexes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    assert!(!fallback.contains(&1), "rejected index must be excluded");
    assert_eq!(fallback[0], 2, "largest remaining file should be first");
}

#[test]
fn buffer_targets_reduces_forward_buffer_for_torrent() {
    let torrent_result: Value = serde_json::from_str(
        &player_buffer_targets_json(
            r#"{"forwardBufferSeconds":120,"backBufferSeconds":30,"isTorrent":true}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let direct_result: Value = serde_json::from_str(
        &player_buffer_targets_json(
            r#"{"forwardBufferSeconds":120,"backBufferSeconds":30,"isTorrent":false}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        torrent_result["forwardBufferMs"].as_i64().unwrap()
            < direct_result["forwardBufferMs"].as_i64().unwrap()
    );
}

#[test]
fn buffer_targets_negative_cache_size_means_unbounded() {
    let result: Value =
        serde_json::from_str(&player_buffer_targets_json(r#"{"cacheSizeMb":-1}"#).unwrap())
            .unwrap();
    assert_eq!(
        result["cacheSizeBytes"].as_i64().unwrap(),
        64_000 * 1_000_000
    );
}

#[test]
fn retry_policy_is_not_retryable_for_no_source() {
    let result: Value = serde_json::from_str(
        &player_retry_policy_json(r#"{"errorCode":"no_source","retryCount":0}"#).unwrap(),
    )
    .unwrap();
    assert_eq!(result["shouldRetry"], false);
}

#[test]
fn retry_policy_retries_connection_errors_with_backoff() {
    let result: Value = serde_json::from_str(
        &player_retry_policy_json(r#"{"errorCode":"timeout","retryCount":1,"isTorrent":false}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(result["shouldRetry"], true);
    assert!(result["delayMs"].as_i64().unwrap() > 0);
}
