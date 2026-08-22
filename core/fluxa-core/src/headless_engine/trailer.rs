mod commands;
mod effects;
mod state;
mod stream_resolution;
mod watch_config;

pub(super) use commands::{complete, dispatch_prewarm, dispatch_resolve};
pub(super) use state::TrailerState;

#[cfg(test)]
mod tests {
    use super::stream_resolution::{
        best_adaptive_pair, first_direct_url, player_script_url, requires_player_script,
        resolve_player_response,
    };
    use super::watch_config::parse_watch_config;
    use serde_json::json;

    #[test]
    fn pairs_highest_resolution_avc1_video_with_highest_bitrate_audio() {
        let formats = json!([
            { "url": "video-360p", "mimeType": "video/mp4; codecs=\"avc1.4d401e\"", "height": 360, "bitrate": 500_000 },
            { "url": "video-1080p", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
            { "url": "video-1080p-vp9", "mimeType": "video/webm; codecs=\"vp9\"", "height": 1080, "bitrate": 4_000_000 },
            { "url": "audio-low", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 64_000 },
            { "url": "audio-high", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 },
        ]);
        let pair = best_adaptive_pair(Some(&formats), None, None);
        assert_eq!(
            pair,
            Some(("video-1080p".to_string(), "audio-high".to_string(), 1080))
        );
    }

    #[test]
    fn mobile_pair_uses_720p_video_and_highest_audio_track() {
        let formats = json!([
            { "url": "video-360p", "mimeType": "video/mp4; codecs=\"avc1.4d401e\"", "height": 360, "bitrate": 500_000 },
            { "url": "video-720p", "mimeType": "video/mp4; codecs=\"avc1.4d401f\"", "height": 720, "bitrate": 1_800_000 },
            { "url": "video-1080p", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
            { "url": "audio-low", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 64_000 },
            { "url": "audio-high", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 }
        ]);
        assert_eq!(
            best_adaptive_pair(Some(&formats), Some(720), None),
            Some(("video-720p".to_string(), "audio-high".to_string(), 720))
        );
    }

    #[test]
    fn large_screen_pair_caps_video_at_1080p_and_keeps_highest_audio_track() {
        let formats = json!([
            { "url": "video-720p", "mimeType": "video/mp4; codecs=\"avc1.4d401f\"", "height": 720, "bitrate": 1_800_000 },
            { "url": "video-1080p", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
            { "url": "video-2160p", "mimeType": "video/mp4; codecs=\"avc1.640033\"", "height": 2160, "bitrate": 12_000_000 },
            { "url": "audio-low", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 64_000 },
            { "url": "audio-high", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 }
        ]);
        assert_eq!(
            best_adaptive_pair(Some(&formats), Some(1080), None),
            Some(("video-1080p".to_string(), "audio-high".to_string(), 1080))
        );
    }

    #[test]
    fn player_response_separates_mobile_and_large_screen_quality_caps() {
        let body = json!({
            "playabilityStatus": { "status": "OK" },
            "streamingData": {
                "adaptiveFormats": [
                    { "url": "video-720p", "mimeType": "video/mp4; codecs=\"avc1.4d401f\"", "height": 720, "bitrate": 1_800_000 },
                    { "url": "video-1080p", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
                    { "url": "video-2160p", "mimeType": "video/mp4; codecs=\"avc1.640033\"", "height": 2160, "bitrate": 12_000_000 },
                    { "url": "audio-low", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 64_000 },
                    { "url": "audio-high", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 }
                ]
            }
        })
        .to_string();
        let response = json!({ "body": body });
        let mobile = resolve_player_response(&response, Some(720), None);
        let large_screen = resolve_player_response(&response, Some(1080), None);

        assert_eq!(mobile["streamUrl"], "video-720p");
        assert_eq!(mobile["audioUrl"], "audio-high");
        assert_eq!(mobile["height"], 720);
        assert_eq!(large_screen["streamUrl"], "video-1080p");
        assert_eq!(large_screen["audioUrl"], "audio-high");
        assert_eq!(large_screen["height"], 1080);
    }

    #[test]
    fn supplied_youtube_videos_select_720p_on_mobile_and_1080p_on_large_screens() {
        for (video_id, formats) in [
            (
                "qgQunxD0qCk",
                json!([
                    { "itag": 136, "url": "qg-video-720", "mimeType": "video/mp4; codecs=\"avc1.4d401f\"", "height": 720, "bitrate": 1_800_000 },
                    { "itag": 137, "url": "qg-video-1080", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
                    { "itag": 140, "url": "qg-audio", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 }
                ]),
            ),
            (
                "WNhH00OIPP0",
                json!([
                    { "itag": 136, "url": "wnh-video-720", "mimeType": "video/mp4; codecs=\"avc1.4d401f\"", "height": 720, "bitrate": 1_800_000 },
                    { "itag": 137, "url": "wnh-video-1080", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
                    { "itag": 140, "url": "wnh-audio", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 }
                ]),
            ),
        ] {
            assert_eq!(
                best_adaptive_pair(Some(&formats), Some(720), None).map(|(_, _, height)| height),
                Some(720),
                "{video_id} mobile selection"
            );
            assert_eq!(
                best_adaptive_pair(Some(&formats), Some(1080), None).map(|(_, _, height)| height),
                Some(1080),
                "{video_id} large-screen selection"
            );
        }
    }

    #[test]
    fn ciphered_tracks_resolve_before_mobile_quality_selection() {
        let body = json!({
            "playabilityStatus": { "status": "OK" },
            "streamingData": {
                "adaptiveFormats": [
                    { "signatureCipher": "url=https%3A%2F%2Fvideo.example%2F720&s=abcdef&sp=sig", "mimeType": "video/mp4; codecs=\"avc1.4d401f\"", "height": 720, "bitrate": 1_800_000 },
                    { "signatureCipher": "url=https%3A%2F%2Fvideo.example%2F1080&s=abcdef&sp=sig", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
                    { "signatureCipher": "url=https%3A%2F%2Faudio.example%2Fhigh&s=abcdef&sp=sig", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 }
                ]
            }
        })
        .to_string();
        let response = json!({ "body": body });
        let player_js = r#"var XY={rv:function(a){a.reverse()}};sig=function(a){a=a.split("");XY.rv(a);return a.join("")}"#;
        let mobile = resolve_player_response(&response, Some(720), Some(player_js));
        assert_eq!(mobile["streamUrl"], "https://video.example/720?sig=fedcba");
        assert_eq!(mobile["audioUrl"], "https://audio.example/high?sig=fedcba");
        assert_eq!(mobile["height"], 720);
    }

    #[test]
    fn cipher_alias_and_n_parameter_are_resolved_before_large_screen_selection() {
        let body = json!({
            "playabilityStatus": { "status": "OK" },
            "streamingData": {
                "adaptiveFormats": [
                    { "cipher": "url=https%3A%2F%2Fvideo.example%2F720%3Fn%3Dabcdef&s=abcdef&sp=sig", "mimeType": "video/mp4; codecs=\"avc1.4d401f\"", "height": 720, "bitrate": 1_800_000 },
                    { "cipher": "url=https%3A%2F%2Fvideo.example%2F1080%3Fn%3Dabcdef&s=abcdef&sp=sig", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
                    { "url": "https://audio.example/high?n=abcdef", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 }
                ]
            }
        })
        .to_string();
        let response = json!({ "body": body });
        let player_js = r#"var XY={rv:function(a){a.reverse()}};sig=function(a){a=a.split("");XY.rv(a);return a.join("")};nfunc=function(a){a=a.split("");XY.rv(a);return a.join("")};var routes={n:nfunc}"#;
        let large_screen = resolve_player_response(&response, Some(1080), Some(player_js));
        assert_eq!(
            large_screen["streamUrl"],
            "https://video.example/1080?n=fedcba&sig=fedcba"
        );
        assert_eq!(
            large_screen["audioUrl"],
            "https://audio.example/high?n=fedcba"
        );
        assert_eq!(large_screen["height"], 1080);
    }

    #[test]
    fn player_script_is_requested_for_cipher_or_n_parameter_formats() {
        let response = json!({
            "body": json!({
                "streamingData": {
                    "adaptiveFormats": [
                        { "cipher": "url=https%3A%2F%2Fvideo.example%2F720&s=abcdef", "mimeType": "video/mp4" },
                        { "url": "https://audio.example/high?n=abcdef", "mimeType": "audio/mp4" }
                    ]
                }
            }).to_string()
        });
        assert!(requires_player_script(&response));
    }

    #[test]
    fn watch_config_uses_the_player_script_url_when_player_response_omits_assets() {
        let config = parse_watch_config(&json!({
            "body": r#"<script>ytcfg.set({"INNERTUBE_API_KEY":"key","VISITOR_DATA":"visitor","jsUrl":"/s/player/current/base.js"});</script>"#
        }));
        assert_eq!(config.api_key, "key");
        assert_eq!(config.visitor_data.as_deref(), Some("visitor"));
        assert_eq!(
            config.player_script_url.as_deref(),
            Some("https://www.youtube.com/s/player/current/base.js")
        );
    }

    #[test]
    fn player_response_assets_script_url_is_normalized_for_platform_http() {
        let response = json!({
            "body": json!({ "assets": { "js": "/s/player/current/base.js" } }).to_string()
        });
        assert_eq!(
            player_script_url(&response).as_deref(),
            Some("https://www.youtube.com/s/player/current/base.js")
        );
    }

    #[test]
    fn no_pair_when_only_vp9_video_is_available() {
        let formats = json!([
            { "url": "video-vp9", "mimeType": "video/webm; codecs=\"vp9\"", "height": 1080, "bitrate": 3_000_000 },
            { "url": "audio", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 },
        ]);
        assert_eq!(best_adaptive_pair(Some(&formats), None, None), None);
    }

    #[test]
    fn no_pair_when_audio_track_is_missing() {
        let formats = json!([
            { "url": "video", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
        ]);
        assert_eq!(best_adaptive_pair(Some(&formats), None, None), None);
    }

    #[test]
    fn no_pair_when_adaptive_formats_are_absent() {
        assert_eq!(best_adaptive_pair(None, None, None), None);
    }

    #[test]
    fn first_direct_url_picks_first_entry_with_a_url() {
        let formats = json!([
            { "itag": 18 },
            { "url": "progressive-url", "itag": 22 },
        ]);
        assert_eq!(
            first_direct_url(Some(&formats), None),
            Some("progressive-url".to_string())
        );
    }

    #[test]
    fn resolve_player_response_includes_audio_url_for_paired_adaptive_streams() {
        let body = json!({
            "playabilityStatus": { "status": "OK" },
            "streamingData": {
                "adaptiveFormats": [
                    { "url": "video-1080p", "mimeType": "video/mp4; codecs=\"avc1.640028\"", "height": 1080, "bitrate": 3_000_000 },
                    { "url": "audio", "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128_000 },
                ]
            }
        }).to_string();
        let response = json!({ "body": body });
        let resolved = resolve_player_response(&response, None, None);
        assert_eq!(resolved["streamUrl"], "video-1080p");
        assert_eq!(resolved["audioUrl"], "audio");
    }

    #[test]
    fn resolve_player_response_falls_back_to_progressive_when_no_adaptive_pair() {
        let body = json!({
            "playabilityStatus": { "status": "OK" },
            "streamingData": {
                "formats": [{ "url": "progressive-360p", "itag": 18 }]
            }
        })
        .to_string();
        let response = json!({ "body": body });
        let resolved = resolve_player_response(&response, None, None);
        assert_eq!(resolved["streamUrl"], "progressive-360p");
        assert!(resolved["audioUrl"].is_null());
    }
}
