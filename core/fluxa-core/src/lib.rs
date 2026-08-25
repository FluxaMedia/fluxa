// Non-native consumers intentionally compile partial API surfaces: desktop uses
// Rust/Tauri calls plus core_invoke, WASM exposes a small JS bridge, and the
// streaming engine uses only stream policy helpers. The Android/native build is
// the exhaustive JNI surface, so keep dead-code checking strict there and avoid
// warning noise for the partial compatibility builds.
#![cfg_attr(not(feature = "native"), allow(dead_code))]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )
)]

#[cfg(feature = "uniffi-bindings")]
uniffi::setup_scaffolding!();

#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod action_contract;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod addon_protocol;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod addon_resource;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod addon_store;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod addon_uptime;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod anime_detection;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod app_state;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod calendar_plan;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod cast_protocol;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod constants;
mod content_identity;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod content_warnings;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub mod core_api;
#[cfg(not(any(feature = "full-api", not(feature = "streaming-shared"))))]
mod core_api;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub mod core_contract;
mod core_error;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod data_policy;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod desktop_playback;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod device_resource;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod discovery_plan;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod dolby_vision_plan;
#[cfg(feature = "dv-codec")]
mod dolby_vision_rpu;
#[cfg(feature = "dv-codec")]
mod dolby_vision_sample;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod external_sync;
mod fluxa_sync;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod headless_adapter_plan;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod headless_engine;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod home_ranking;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod integration_settings;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod intro_segments;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod library_persistence;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod library_state;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod local_media;
pub mod log_sink;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod mdblist_plan;
pub mod media_demux;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod continue_watching;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod nuvio_sync;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod nuvio_pin;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod oauth_plan;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod offline_download;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod platform_plan;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod player_flow;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod player_policy;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod player_scrobble;
#[cfg(feature = "plugin-js-engine")]
pub mod plugin_runtime;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod plugins;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod profile_avatar_pack;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod profile_contract;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod profile_prefs;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod publicmetadb_plan;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod repository_flow;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod search_plan;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod stream_badges;
mod stream_policy;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod subtitle_sync;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod tmdb_plan;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod trailer_subtitles;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod watchlist_plan;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub mod watch_together;

#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub mod env;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub mod ffi;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub mod runtime;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub mod types;

#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub mod bindings;

pub use core_api::FluxaCore;

// Re-exports internal parsing functions for the `fuzz/` crate only. These stay
// pub(crate) for real consumers — this exists purely so libFuzzer can call
// straight into them without going through ffi::core_invoke's catch_unwind,
// which would otherwise swallow the exact panics fuzzing is trying to find.
#[cfg(all(
    feature = "fuzzing",
    any(feature = "full-api", not(feature = "streaming-shared"))
))]
pub mod fuzz_targets {
    pub use crate::addon_protocol::parse_manifest;
    pub use crate::content_identity::{
        contains_compact_episode, contains_spaced_episode, parse_episode_locator,
        percent_decode_component,
    };
    #[cfg(feature = "dv-codec")]
    pub use crate::dolby_vision_sample::{fuzz_process_sample, process_dv_sample_json};
    pub use crate::headless_engine::{
        create_headless_engine, destroy_headless_engine, headless_engine_complete_effect_json,
        headless_engine_dispatch_json,
    };
}

#[cfg(all(test, any(feature = "full-api", not(feature = "streaming-shared"))))]
mod tests {
    use crate::addon_protocol::{
        catalog_has_required_extra_except, catalog_requires_extra, catalog_supports_extra,
    };
    use crate::content_identity::stream_request_ids;
    use crate::home_ranking::optimize_home_rows_json;
    use crate::stream_policy::{
        stream_playback_info_json, stream_request_headers_json, stream_request_referer,
        torrent_runtime_info_json, torrent_status_info_json,
    };
    use serde_json::{Value, json};

    #[test]
    fn stream_request_ids_keep_requested_tmdb_episode_before_canonical_fallback() {
        assert_eq!(
            stream_request_ids(
                "series",
                "tmdb:12345:1:2",
                Some("tmdb:12345"),
                Some("tmdb:12345"),
                Some("tt9999999"),
            ),
            vec!["tmdb:12345:1:2", "tt9999999:1:2"]
        );
    }

    #[test]
    fn stream_request_ids_keep_custom_episode_id_before_canonical_fallback() {
        assert_eq!(
            stream_request_ids(
                "series",
                "kitsu:777:1:2",
                Some("tt9999999"),
                Some("kitsu:777"),
                Some("tt9999999"),
            ),
            vec!["kitsu:777:1:2", "tt9999999:1:2"]
        );
    }

    #[test]
    fn stream_request_ids_prefer_canonical_for_tmdb_movies() {
        assert_eq!(
            stream_request_ids("movie", "tmdb:12345", None, None, Some("tt9999999")),
            vec!["tt9999999", "tmdb:12345"]
        );
    }

    #[test]
    fn stream_playback_info_reads_stremio_stream_fields_without_rewriting_result() {
        let info = stream_playback_info_json(
            r#"{
                "name":"Source",
                "url":"https://cdn.example/Breaking%20Bad.mkv",
                "behaviorHints":{
                    "videoHash":"abc123",
                    "videoSize":42,
                    "filename":"Custom File.mkv"
                }
            }"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("stream playback info");

        assert_eq!(
            info.get("playableUrl").and_then(Value::as_str),
            Some("https://cdn.example/Breaking%20Bad.mkv")
        );
        assert_eq!(
            info.get("effectiveVideoHash").and_then(Value::as_str),
            Some("abc123")
        );
        assert_eq!(
            info.get("effectiveVideoSize").and_then(Value::as_i64),
            Some(42)
        );
        assert_eq!(
            info.get("effectiveFilename").and_then(Value::as_str),
            Some("Custom File.mkv")
        );
        assert_eq!(
            info.get("subtitleExtraArgs").and_then(Value::as_str),
            Some("videoHash=abc123&videoSize=42&filename=Custom+File.mkv")
        );
        assert_eq!(
            info.get("isLikelyPlayerCompatible")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn stream_playback_info_builds_torrent_url_from_info_hash() {
        let info = stream_playback_info_json(r#"{"infoHash":"abcdef","fileIdx":3}"#)
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .expect("stream playback info");

        assert_eq!(
            info.get("playableUrl").and_then(Value::as_str),
            Some("stremio://torrent/abcdef/3")
        );
        assert_eq!(
            info.get("isTorrentPlaybackUrl").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            info.get("isLikelyPlayerCompatible")
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn torrent_runtime_info_normalizes_link_and_resolves_file_index() {
        // fileIdx provided by addon → use it directly, no episode matching
        let info = torrent_runtime_info_json(
            r#"{
                "link":"stremio://torrent/ABCDEF1234567890ABCDEF1234567890ABCDEF12/4",
                "title":"tt123:1:2",
                "requestedFileIdx":1,
                "preferredFilename":null,
                "sources":["tracker:udp://tracker.example:1337/announce","tracker:udp://tracker.example:1337/announce"],
                "fileStats":[
                    {"id":1,"path":"Show.S01E02.mkv","length":100},
                    {"id":2,"path":"Show.S01E03.mkv","length":300}
                ],
                "rejectedIndex":null,
                "baseUrl":"http://127.0.0.1:8090",
                "play":true,
                "stat":false
            }"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("torrent runtime info");

        let normalized = info
            .get("normalizedLink")
            .and_then(Value::as_str)
            .expect("normalizedLink");
        assert!(
            normalized.starts_with("magnet:?xt=urn:btih:abcdef1234567890abcdef1234567890abcdef12"),
            "unexpected magnet prefix: {normalized}"
        );
        // Addon-provided tracker survives dedupe (appears once).
        assert_eq!(
            normalized.matches("tracker.example%3A1337").count(),
            1,
            "addon tracker should appear exactly once: {normalized}"
        );
        // Fallback trackers are always appended for peer discovery.
        assert!(
            normalized.contains("opentrackr.org"),
            "fallback tracker missing: {normalized}"
        );
        assert_eq!(info.get("selectedFileIdx").and_then(Value::as_i64), Some(1));
        assert_eq!(
            info.get("selectedReason").and_then(Value::as_str),
            Some("requested")
        );

        // No fileIdx but an episode-shaped title → the matching episode file
        // wins even when a bigger video is present
        let episode = torrent_runtime_info_json(
            r#"{
                "link":"stremio://torrent/ABCDEF1234567890ABCDEF1234567890ABCDEF12",
                "title":"tt123:1:2",
                "requestedFileIdx":null,
                "preferredFilename":null,
                "sources":[],
                "fileStats":[
                    {"id":1,"path":"Show.S01E02.mkv","length":100},
                    {"id":2,"path":"Show.S01E03.mkv","length":300}
                ],
                "rejectedIndex":null,
                "baseUrl":"http://127.0.0.1:8090",
                "play":true,
                "stat":false
            }"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("torrent episode info");

        assert_eq!(
            episode.get("selectedFileIdx").and_then(Value::as_i64),
            Some(1)
        );
        assert_eq!(
            episode.get("selectedReason").and_then(Value::as_str),
            Some("episode")
        );

        // No fileIdx and no episode in the title → falls back to largest video
        let fallback = torrent_runtime_info_json(
            r#"{
                "link":"stremio://torrent/ABCDEF1234567890ABCDEF1234567890ABCDEF12",
                "title":"tt123",
                "requestedFileIdx":null,
                "preferredFilename":null,
                "sources":[],
                "fileStats":[
                    {"id":1,"path":"Show.S01E02.mkv","length":100},
                    {"id":2,"path":"Show.S01E03.mkv","length":300}
                ],
                "rejectedIndex":null,
                "baseUrl":"http://127.0.0.1:8090",
                "play":true,
                "stat":false
            }"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("torrent fallback info");

        assert_eq!(
            fallback.get("selectedFileIdx").and_then(Value::as_i64),
            Some(2)
        );
        assert_eq!(
            fallback.get("selectedReason").and_then(Value::as_str),
            Some("largest-video")
        );
    }

    #[test]
    fn torrent_status_info_reports_progress_and_playability() {
        let info = torrent_status_info_json(
            r#"{
                "stat":1,
                "progress":4.0,
                "loaded_size":262144,
                "preload_size":524288
            }"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("torrent status info");

        assert_eq!(info.get("bufferProgress").and_then(Value::as_i64), Some(50));
        assert_eq!(
            info.get("isPlayableEnough").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            info.get("statusKey").and_then(Value::as_str),
            Some("player.torrent_status.preloading")
        );
    }

    #[test]
    fn stream_request_headers_keep_only_explicit_clean_headers() {
        let headers = stream_request_headers_json(r#"{"X-Test":"ok","":"ignored","Blank":""}"#)
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .expect("headers");

        assert_eq!(headers.get("X-Test").and_then(Value::as_str), Some("ok"));
        assert!(headers.get("").is_none());
        assert!(headers.get("Blank").is_none());
        assert_eq!(
            stream_request_referer("https://vidmoly.me/video.mp4"),
            Some("https://vidmoly.me/".to_string())
        );
        assert_eq!(stream_request_referer("magnet:?xt=urn:btih:abc"), None);
    }

    #[test]
    fn catalog_extra_helpers_match_manifest_extra_shapes() {
        let modern =
            r#"{"type":"movie","id":"modern","extra":[{"name":"search","isRequired":true}]}"#;
        let legacy = r#"{"type":"movie","id":"legacy","extraSupported":["genre"]}"#;

        assert!(catalog_supports_extra(modern, "search"));
        assert!(catalog_requires_extra(modern, "search"));
        assert!(catalog_supports_extra(legacy, "genre"));
        assert!(!catalog_requires_extra(legacy, "genre"));
        assert!(!catalog_has_required_extra_except(modern, r#"["search"]"#));
        assert!(catalog_has_required_extra_except(modern, "[]"));
    }

    #[test]
    fn home_rows_keep_pinned_before_native_ranked_rows() {
        let request = json!({
            "categories": [
                {"id":"popular","name":"Popular","semanticName":"Popular","type":"movie","items":[
                    {"id":"p1","name":"P1","type":"movie","poster":null},
                    {"id":"p2","name":"P2","type":"movie","poster":null},
                    {"id":"p3","name":"P3","type":"movie","poster":null},
                    {"id":"p4","name":"P4","type":"movie","poster":null},
                    {"id":"p5","name":"P5","type":"movie","poster":null},
                    {"id":"p6","name":"P6","type":"movie","poster":null}
                ]},
                {"id":"continue_watching","name":"Continue Watching","semanticName":"Continue Watching","type":"movie","items":[
                    {"id":"cw1","name":"CW1","type":"movie","poster":null}
                ]},
                {"id":"trending","name":"Trending Now","semanticName":"Trending Now","type":"movie","items":[
                    {"id":"t1","name":"T1","type":"movie","poster":null},
                    {"id":"t2","name":"T2","type":"movie","poster":null},
                    {"id":"t3","name":"T3","type":"movie","poster":null},
                    {"id":"t4","name":"T4","type":"movie","poster":null},
                    {"id":"t5","name":"T5","type":"movie","poster":null},
                    {"id":"t6","name":"T6","type":"movie","poster":null}
                ]}
            ],
            "preferredOrderLabels": ["Trending Now", "Popular"],
            "preferredGenres": {},
            "preferredTypes": {},
            "priorityLabels": {
                "trendingNow": "Trending Now",
                "popularForYou": "Popular For You",
                "mostWatched": "Most Watched"
            }
        });
        let rows = optimize_home_rows_json(&request.to_string())
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .expect("rows");
        let rows = rows.as_array().expect("row array");

        assert_eq!(
            rows[0].get("id").and_then(Value::as_str),
            Some("continue_watching")
        );
        assert_eq!(rows[1].get("id").and_then(Value::as_str), Some("trending"));
    }
}
