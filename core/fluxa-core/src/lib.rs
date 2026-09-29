#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )
)]

mod action_contract;
mod addon_protocol;
mod addon_resource;
mod addon_store;
mod addon_uptime;
mod anime_detection;
pub mod app_icon;
mod app_state;
mod calendar_plan;
mod cast_protocol;
mod checksum_policy;
mod constants;
mod content_identity;
mod content_warnings;
mod continue_watching;
pub mod core_api;
pub mod core_contract;
mod core_error;
mod data_policy;
mod desktop_playback;
mod device_auth;
mod device_resource;
mod discord_presence;
mod discovery_plan;
pub mod dolby_vision_plan;
mod external_sync;
mod fluxa_sync;
mod headless_adapter_plan;
mod headless_engine;
mod home_ranking;
mod integration_settings;
mod intro_segments;
mod library_persistence;
mod library_state;
mod local_media;
pub mod log_sink;
mod mdblist_plan;
mod nuvio_pin;
mod nuvio_sync;
mod oauth_plan;
mod offline_download;
mod platform_plan;
mod player_flow;
mod player_policy;
mod player_scrobble;
pub mod plugin_network_policy;
mod plugins;
mod profile_avatar_pack;
mod profile_contract;
mod profile_prefs;
mod provider_library;
mod publicmetadb_plan;
mod recommendation_policy;
mod release_date;
mod repository_flow;
mod runtime_label;
mod search_plan;
mod settings_contract;
mod stream_badges;
mod stream_policy;
mod subtitle_sync;
mod tmdb_plan;
mod trailer_subtitles;
mod version_policy;
pub mod watch_together;
mod watchlist_plan;

pub mod env;
pub mod ffi;
pub mod runtime;
pub mod types;

pub mod bindings;

pub use core_api::FluxaCore;
pub use headless_engine::{Engine, PageUpdate, Update};
pub use home_ranking::home_hero_plan;

// Re-exports internal parsing functions for the `fuzz/` crate only. These stay
// pub(crate) for real consumers — this exists purely so libFuzzer can call
// straight into them without going through ffi::core_invoke's catch_unwind,
// which would otherwise swallow the exact panics fuzzing is trying to find.
#[cfg(feature = "fuzzing")]
pub mod fuzz_targets {
    pub use crate::addon_protocol::parse_manifest;
    pub use crate::content_identity::{
        contains_compact_episode, contains_spaced_episode, parse_episode_locator,
        percent_decode_component,
    };
    pub use crate::headless_engine::{
        create_headless_engine, destroy_headless_engine, headless_engine_complete_effect_json,
        headless_engine_dispatch_json,
    };
}

#[cfg(test)]
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
