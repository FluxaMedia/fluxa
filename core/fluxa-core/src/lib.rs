#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )
)]

mod accounts;
pub mod addons;
mod catalog;
mod headless_engine;
mod home;
mod library;
pub mod player;
mod profile;
mod services;
mod settings;

pub mod ffi;
pub mod runtime;
pub use runtime::{app_icon, env, log_sink};
pub mod types;

pub mod bindings;

pub use headless_engine::{Engine, PageUpdate, Update};
pub use home::ranking::home_hero_plan;

// Re-exports internal parsing functions for the `fuzz/` crate only. These stay
// pub(crate) for real consumers — this exists purely so libFuzzer can call
// straight into them without going through ffi::core_invoke's catch_unwind,
// which would otherwise swallow the exact panics fuzzing is trying to find.
#[cfg(feature = "fuzzing")]
pub mod fuzz_targets {
    pub use crate::addons::sources::protocol::parse_manifest;
    pub use crate::catalog::identity::{
        contains_compact_episode, contains_spaced_episode, parse_episode_locator,
        percent_decode_component
    };
    pub use crate::headless_engine::{
        create_headless_engine, destroy_headless_engine, headless_engine_complete_effect_json,
        headless_engine_dispatch_json
    };
}

#[cfg(test)]
mod tests {
    use crate::addons::sources::protocol::{
        catalog_has_required_extra_except, catalog_requires_extra, catalog_supports_extra
    };
    use crate::catalog::identity::stream_request_ids;
    use crate::player::streams::stream_policy::{
        stream_playback_info_json, 
        torrent_runtime_info_json 
    };
    use serde_json::Value;

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

}
