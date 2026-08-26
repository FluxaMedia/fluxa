#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
mod external_audio;
mod language;
mod magnet;
mod meta;
mod selection;
mod torrent_files;
mod torrent_runtime;

#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use external_audio::*;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use language::*;
pub(crate) use magnet::*;
pub(crate) use meta::*;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use selection::*;
pub(crate) use torrent_files::*;
pub(crate) use torrent_runtime::*;
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn track_state(request: Value) -> Value {
        serde_json::from_str(&player_track_state_json(&request.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn player_track_state_uses_audio_memory_before_profile_preference() {
        let state = track_state(json!({
            "lastAudioLanguage": "tr",
            "preferredAudioLanguage": "en",
            "originalLanguage": "en"
        }));

        assert_eq!(state["preferredAudioLanguage"], "tr");
    }

    #[test]
    fn player_track_state_uses_japanese_original_for_english_anime_preference() {
        let state = track_state(json!({
            "preferredAudioLanguage": "en",
            "originalLanguage": "ja"
        }));

        assert_eq!(state["preferredAudioLanguage"], "ja");
    }

    #[test]
    fn player_track_state_selects_subtitle_memory_then_secondary() {
        let memory = track_state(json!({
            "availableSubtitles": [
                { "id": "en", "label": "English", "language": "en" },
                { "id": "tr", "label": "Turkish", "language": "tr" }
            ],
            "lastSubtitleLanguage": "tr",
            "preferredSubtitleLanguage": "en"
        }));
        assert_eq!(memory["preferredSubtitleIndex"], 1);
        assert_eq!(memory["preferredSubtitleId"], "tr");
        assert_eq!(memory["subtitlesDisabled"], false);

        let secondary = track_state(json!({
            "availableSubtitles": [
                { "id": "tr", "label": "Turkish", "language": "tr" }
            ],
            "preferredSubtitleLanguage": "en",
            "secondarySubtitleLanguage": "tr"
        }));
        assert_eq!(secondary["preferredSubtitleIndex"], 0);
        assert_eq!(secondary["preferredSubtitleId"], "tr");
    }

    #[test]
    fn player_track_state_disables_subtitles_when_no_preferred_match_exists() {
        let state = track_state(json!({
            "availableSubtitles": [
                { "id": "tr", "label": "Turkish", "language": "tr" }
            ],
            "preferredSubtitleLanguage": "en"
        }));

        assert_eq!(state["preferredSubtitleIndex"], -1);
        assert!(state["preferredSubtitleId"].is_null());
        assert_eq!(state["subtitlesDisabled"], true);
    }

    #[test]
    fn build_magnet_dedupes_addon_tracker_and_appends_fallbacks() {
        let magnet = build_magnet(
            "ABCDEF1234567890ABCDEF1234567890ABCDEF12",
            &["tracker:udp://tracker.example:1337/announce".to_string()],
        );

        assert!(magnet.starts_with("magnet:?xt=urn:btih:abcdef1234567890abcdef1234567890abcdef12"));
        assert_eq!(magnet.matches("tracker.example%3A1337").count(), 1);
        assert!(magnet.contains("opentrackr.org"));
    }

    #[test]
    fn stream_magnet_link_builds_from_info_hash_and_sources() {
        let stream = json!({
            "infoHash": "ABCDEF1234567890ABCDEF1234567890ABCDEF12",
            "sources": ["tracker:udp://tracker.example:1337/announce"],
        });
        let link = stream_magnet_link(&stream).unwrap();
        assert!(link.starts_with("magnet:?xt=urn:btih:abcdef1234567890abcdef1234567890abcdef12"));
    }

    #[test]
    fn torrent_sibling_subtitles_match_by_episode_tag_over_unrelated_files() {
        let files = vec![
            (1, "Show.S01E01.mkv".to_string()),
            (2, "Show.S01E02.mkv".to_string()),
            (3, "Show.S01E02.eng.srt".to_string()),
            (4, "Show.S01E03.eng.srt".to_string()),
            (5, "readme.txt".to_string()),
        ];
        let matches = torrent_sibling_subtitle_matches("Show.S01E02.mkv", &files);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, 3);
        assert_eq!(matches[0].language.as_deref(), Some("eng"));
    }

    #[test]
    fn torrent_sibling_subtitles_single_video_torrent_accepts_any_subtitle() {
        let files = vec![
            (1, "release-group-video.mkv".to_string()),
            (2, "totally-unrelated-name.srt".to_string()),
        ];
        let matches = torrent_sibling_subtitle_matches("release-group-video.mkv", &files);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, 2);
    }

    #[test]
    fn stream_magnet_link_none_for_direct_http_stream() {
        let stream = json!({ "url": "https://cdn.example/video.mkv" });
        assert!(stream_magnet_link(&stream).is_none());
    }

    #[test]
    fn resolve_torrent_file_index_prefers_requested_then_filename_then_largest_video() {
        let stats = vec![
            TorrentFileStat {
                id: 1,
                path: "Show.S01E01.mkv".to_string(),
                length: 100,
            },
            TorrentFileStat {
                id: 2,
                path: "Show.S01E02.mkv".to_string(),
                length: 300,
            },
            TorrentFileStat {
                id: 3,
                path: "sample.txt".to_string(),
                length: 999_999,
            },
        ];

        // Addon-provided fileIdx wins outright, even though it doesn't match any stat.
        assert_eq!(
            resolve_torrent_file_index("title", Some(9), None, &stats),
            (Some(9), Some("requested".to_string()))
        );

        // No requested index, but a preferred filename matches by basename.
        assert_eq!(
            resolve_torrent_file_index("title", None, Some("Show.S01E01.mkv"), &stats),
            (Some(1), Some("filename".to_string()))
        );

        // No requested index or filename match — falls back to the largest *video* file,
        // ignoring the much larger non-video sample.txt.
        assert_eq!(
            resolve_torrent_file_index("title", None, None, &stats),
            (Some(2), Some("largest-video".to_string()))
        );

        assert_eq!(
            resolve_torrent_file_index("title", None, None, &[]),
            (None, None)
        );
    }

}
