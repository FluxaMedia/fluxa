mod backend_selection;
mod buffer_targets;
mod dolby_vision;
mod next_episode;
mod playback_close;
mod retry_and_ordering;
mod source_sidebar;
mod torrent_fallback;

pub(crate) use self::dolby_vision::*;
pub(crate) use backend_selection::player_backend_selection_json;
pub(crate) use buffer_targets::player_buffer_targets_json;
pub(crate) use next_episode::{can_prefetch_next_episode_json, select_next_episode_stream_json};
pub(crate) use playback_close::{playback_close_plan_json, playback_preferences_plan_json};
pub(crate) use retry_and_ordering::{
    next_retry_source_plan_json, order_streams_plan_json, player_retry_policy_json,
    stream_shell_plan_json,
};
pub use source_sidebar::player_source_sidebar_plan_json;
pub(crate) use torrent_fallback::torrent_fallback_file_policy_json;
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn backend_selection_defaults_to_exoplayer() {
        let result: Value = serde_json::from_str(
            &player_backend_selection_json(r#"{"stream":{"url":"http://example.com/video.mp4"}}"#)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(result["backend"], "exoplayer");
    }

    #[test]
    fn backend_selection_respects_mpv_user_preference() {
        let result: Value = serde_json::from_str(
            &player_backend_selection_json(
                r#"{"stream":{"url":"http://example.com/video.mp4"},"preferredPlayer":"mpv"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["backend"], "mpv");
        assert_eq!(result["reason"], "user_preference");
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
            &player_retry_policy_json(
                r#"{"errorCode":"timeout","retryCount":1,"isTorrent":false}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["shouldRetry"], true);
        assert!(result["delayMs"].as_i64().unwrap() > 0);
    }

    fn plan(json: &str) -> Value {
        serde_json::from_str(&dv_proxy_plan_json(json).unwrap()).unwrap()
    }

    #[test]
    fn dv_proxy_off_mode_returns_none() {
        let p = plan(
            r#"{"stream":{"name":"4K DV HDR","dvProfile":7},"url":"https://cdn.example/movie.mkv","fallbackMode":"off"}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "user_disabled");
    }

    #[test]
    fn dv_proxy_hls_url_defers_to_manifest_rewrite() {
        let p = plan(
            r#"{"stream":{"name":"4K DV","dvProfile":7},"url":"https://cdn.example/index.m3u8","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "manifest_handled");
    }

    #[test]
    fn dv_proxy_dash_url_defers_to_manifest_rewrite() {
        let p = plan(
            r#"{"stream":{"name":"4K DV","dvProfile":7},"url":"https://cdn.example/stream.mpd","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "manifest_handled");
    }

    #[test]
    fn dv_proxy_non_dv_stream_returns_none() {
        let p = plan(
            r#"{"stream":{"name":"1080p HDR AVC"},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "not_dv");
    }

    #[test]
    fn dv_proxy_hw_dv_decoder_skips_proxy() {
        let p = plan(
            r#"{"stream":{"name":"4K DV","dvProfile":7},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":true}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "hw_dv_decoder");
    }

    #[test]
    fn dv_proxy_p5_no_dv_decoder_returns_none() {
        let p = plan(
            r#"{"stream":{"dvProfile":5},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "no_hdr_base_layer");
        assert_eq!(p["profile"], "P5");
    }

    #[test]
    fn dv_proxy_p4_no_dv_decoder_returns_none() {
        let p = plan(
            r#"{"stream":{"dvProfile":4},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "no_hdr_base_layer");
        assert_eq!(p["profile"], "P4");
    }

    #[test]
    fn dv_proxy_p10_compat_0_returns_none() {
        let p = plan(
            r#"{"stream":{"dvProfile":10,"dvCompatId":0},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "p10_compat_id_no_hdr_base");
    }

    #[test]
    fn dv_proxy_p10_compat_2_returns_none() {
        let p = plan(
            r#"{"stream":{"dvProfile":10,"dvCompatId":2},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "none");
    }

    #[test]
    fn dv_proxy_unknown_profile_returns_none() {
        // DV detected but no profile info → safe default is none.
        let p = plan(
            r#"{"stream":{"name":"Dolby Vision"},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "unknown_profile_no_safe_fallback");
    }

    #[test]
    fn dv_proxy_p7_mkv_auto_gives_dvcc_strip_medium_safety() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":false,"deviceHasDvDisplay":false}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["profile"], "P7");
        assert_eq!(p["compatibility"], "HDR10");
        assert_eq!(p["safety"], "medium");
    }

    #[test]
    fn dv_proxy_p8_1_gives_dvcc_strip_low_safety() {
        let p = plan(
            r#"{"stream":{"dvProfile":8,"dvCompatId":1},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["profile"], "P8.1");
        assert_eq!(p["compatibility"], "HDR10");
        assert_eq!(p["safety"], "low");
    }

    #[test]
    fn dv_proxy_p8_4_fallback_is_hlg_not_hdr10() {
        let p = plan(
            r#"{"stream":{"dvProfile":8,"dvCompatId":4},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["profile"], "P8.4");
        assert_eq!(p["compatibility"], "HLG");
        assert_ne!(p["compatibility"], "HDR10");
    }

    #[test]
    fn dv_proxy_p8_unknown_compat_strips_with_assumed_hdr10() {
        // "DV P8" in name → P8Unknown → strip, medium safety, HDR10_assumed
        let p = plan(
            r#"{"stream":{"name":"DV P8"},"url":"https://debrid.example/file.mkv","fallbackMode":"hdr10","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["profile"], "P8");
        assert_eq!(p["compatibility"], "HDR10_assumed");
        assert_eq!(p["safety"], "medium");
    }

    #[test]
    fn dv_proxy_p10_compat_1_gives_dvcc_strip() {
        let p = plan(
            r#"{"stream":{"dvProfile":10,"dvCompatId":1},"url":"https://cdn.example/movie.mkv","fallbackMode":"auto","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["profile"], "P10_compat1");
        assert_eq!(p["compatibility"], "HDR10");
    }

    #[test]
    fn dv_proxy_p7_raw_hevc_dv8_mode_gives_rpu_convert() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/stream.hevc","fallbackMode":"dv8","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "rpu_convert");
        assert_eq!(p["rpuMode"], 2);
        assert_eq!(p["profile"], "P7");
    }

    #[test]
    fn dv_proxy_p7_raw_hevc_auto_dv_display_gives_rpu_convert() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/stream.hevc","fallbackMode":"auto","deviceHasDvDecoder":false,"deviceHasDvDisplay":true}"#,
        );
        assert_eq!(p["action"], "rpu_convert");
    }

    #[test]
    fn dv_proxy_structured_caps_p8_only_still_allows_p7_convert() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/stream.hevc","fallbackMode":"convert_dv81","deviceHasDvDecoder":false,"deviceCapabilities":{"profile7":false,"profile8":true}}"#,
        );
        assert_eq!(p["action"], "rpu_convert");
        assert_eq!(p["profile"], "P7");
    }

    #[test]
    fn dv_proxy_runtime_verified_p8_counts_even_when_not_advertised() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/stream.hevc","fallbackMode":"convert_dv81","deviceHasDvDecoder":false,"deviceCapabilities":{"profile7":{"advertised":false,"runtimeVerified":false},"profile8":{"advertised":false,"runtimeVerified":true}}}"#,
        );
        assert_eq!(p["action"], "rpu_convert");
    }

    #[test]
    fn dv_proxy_advertised_but_unverified_p8_still_counts() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/stream.hevc","fallbackMode":"convert_dv81","deviceHasDvDecoder":false,"deviceCapabilities":{"profile8":{"advertised":true,"runtimeVerified":false}}}"#,
        );
        assert_eq!(p["action"], "rpu_convert");
    }

    #[test]
    fn dv_proxy_structured_caps_no_p7_no_p8_rejects_convert() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/stream.hevc","fallbackMode":"convert_dv81","deviceHasDvDecoder":true,"deviceCapabilities":{"profile5":true,"profile7":false,"profile8":false}}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
    }

    #[test]
    fn dv_proxy_rpu_convert_rejected_for_mkv_falls_back_to_dvcc_strip() {
        // dv8 mode + MKV without a DV decoder → falls back to dvcc_strip because
        // rpu_convert needs a DV decoder in the convert_dv81 path, and dv8 mode
        // is annexb-only (rejects non-raw-HEVC containers).
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/movie.mkv","fallbackMode":"dv8","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["reason"], "rpu_convert_rejected_not_annexb");
    }

    #[test]
    fn dv_proxy_rpu_convert_rejected_for_mp4_falls_back_to_dvcc_strip() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/movie.mp4","fallbackMode":"dv8","deviceHasDvDecoder":false}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["reason"], "rpu_convert_rejected_not_annexb");
    }

    #[test]
    fn dv_detection_dolby_vision_p8_text_gives_action() {
        // "P8" token → P8Unknown → dvcc_strip
        let p = plan(
            r#"{"stream":{"name":"Dolby Vision P8"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_ne!(p["action"], "none");
        assert_eq!(p["profile"], "P8");
    }

    #[test]
    fn dv_detection_dovi_without_profile_gives_none() {
        // DV detected ("dovi") but no profile info → unknown → none.
        let p = plan(
            r#"{"stream":{"name":"4K DoVi 5.1"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "unknown_profile_no_safe_fallback");
    }

    #[test]
    fn dv_detection_standalone_dv_without_profile_gives_none() {
        // "[DV]" detected but no profile info → none.
        let p = plan(
            r#"{"stream":{"name":"[4K] [DV] [HDR10+]"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "unknown_profile_no_safe_fallback");
    }

    #[test]
    fn dv_detection_dvhe_fourcc_in_name_gives_profile_p7() {
        let p = plan(
            r#"{"stream":{"name":"dvhe.07.06 BDRemux"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_ne!(p["action"], "none");
        assert_eq!(p["profile"], "P7");
    }

    #[test]
    fn dv_detection_dvhe_08_01_in_name_gives_p8_unknown_not_p8_1() {
        // "01" is codec-string level, not compat id — must not guess P8.1.
        let p = plan(
            r#"{"stream":{"name":"dvhe.08.01 Remux"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["profile"], "P8");
        assert_eq!(p["compatibility"], "HDR10_assumed");
    }

    #[test]
    fn dv_detection_explicit_dv_profile_and_compat_id_still_gives_p8_1() {
        // Compat id from an explicit field (not the codec string) is still trusted.
        let p = plan(
            r#"{"stream":{"name":"dvhe.08.01 Remux","dvProfile":8,"dvCompatId":1},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["profile"], "P8.1");
        assert_eq!(p["safety"], "low");
    }

    #[test]
    fn dv_detection_no_false_positive_from_dvd() {
        let p = plan(
            r#"{"stream":{"name":"DVD Rip 1080p"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "not_dv");
    }

    #[test]
    fn dv_detection_no_false_positive_from_hdvd() {
        let p = plan(
            r#"{"stream":{"name":"HDVD Edition"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["action"], "none");
    }

    #[test]
    fn dv_detection_explicit_boolean_flag_with_profile() {
        let p = plan(
            r#"{"stream":{"dv":true,"dvProfile":8,"dvCompatId":1,"name":"4K HDR"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_ne!(p["action"], "none");
        assert_eq!(p["profile"], "P8.1");
    }

    #[test]
    fn dv_detection_filename_without_profile_gives_none() {
        // DV keyword in filename but no profile → safe default is none.
        let p = plan(
            r#"{"stream":{"name":"4K HDR","effectiveFilename":"Movie.2023.UHD.DV.HEVC.mkv"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "unknown_profile_no_safe_fallback");
    }

    #[test]
    fn dv_detection_dvhe_codec_in_filename_gives_profile() {
        let p = plan(
            r#"{"stream":{"effectiveFilename":"Movie.dvhe.07.06.mkv"},"url":"https://cdn.example/f.mkv","fallbackMode":"auto"}"#,
        );
        assert_ne!(p["action"], "none");
        assert_eq!(p["profile"], "P7");
    }

    // These tests mirror real Stremio addon stream objects, covering the full plan output.

    #[test]
    fn sample_p5_dvonly_no_fallback() {
        // P5 is HEVC single-layer with no HDR base. Stripping DVCC would expose
        // a DV-only bitstream to an HDR10 decoder → broken colour. Never rewrite.
        let p = plan(
            r#"{
            "stream": {
                "name": "AETHER | 4K | Dolby Vision | DD+ Atmos",
                "description": "📺 4K | 🎬 dvhe.05.06 | 🔊 DD+ Atmos",
                "dvProfile": 5
            },
            "url": "https://debrid.example/movie.mkv",
            "fallbackMode": "auto",
            "deviceHasDvDecoder": false
        }"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "no_hdr_base_layer");
        assert_eq!(p["profile"], "P5");
        let limitations = p["limitations"].as_array().unwrap();
        assert!(
            limitations
                .iter()
                .any(|l| l.as_str().unwrap().contains("p4_p5"))
        );
    }

    #[test]
    fn sample_p7_dual_layer_hdr10_fallback() {
        // P7 BL+EL: stripping DVCC reveals the HDR10 base layer. Medium risk —
        // RPU NALs remain in-stream but HEVC decoders ignore them.
        let p = plan(
            r#"{
            "stream": {
                "name": "FLUX | 4K | dvhe.07.06 | Atmos",
                "description": "HDR10 + Dolby Vision P7 BL+EL remux",
                "dvProfile": 7
            },
            "url": "https://realdebrid.com/dl/movie2024.mkv",
            "fallbackMode": "auto",
            "deviceHasDvDecoder": false,
            "deviceHasDvDisplay": false
        }"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["profile"], "P7");
        assert_eq!(p["compatibility"], "HDR10");
        assert_eq!(p["safety"], "medium");
        let limitations = p["limitations"].as_array().unwrap();
        assert!(
            limitations
                .iter()
                .any(|l| l.as_str().unwrap().contains("does_not_convert_bitstream"))
        );
    }

    #[test]
    fn sample_p8_1_single_layer_low_risk_fallback() {
        // P8.1 has an HDR10-compatible base layer encoded into the single HEVC stream.
        // Stripping DVCC gives clean HDR10 output. Lowest-risk rewrite.
        let p = plan(
            r#"{
            "stream": {
                "name": "HDMUX | 4K | dvhe.08.01 | TrueHD Atmos",
                "dvProfile": 8,
                "dvCompatId": 1
            },
            "url": "https://debrid.example/Movie.2023.2160p.DV.HEVC.mkv",
            "fallbackMode": "auto",
            "deviceHasDvDecoder": false
        }"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["profile"], "P8.1");
        assert_eq!(p["compatibility"], "HDR10");
        assert_eq!(p["safety"], "low");
    }

    #[test]
    fn sample_p8_4_hlg_base_not_hdr10() {
        // P8.4 has an HLG base layer, not HDR10. Rewriting it as HDR10 would
        // produce incorrect colour. The compatibility field must reflect HLG.
        let p = plan(
            r#"{
            "stream": {
                "name": "BBC iPlayer | 4K | Dolby Vision HLG | AAC",
                "dvProfile": 8,
                "dvCompatId": 4
            },
            "url": "https://cdn.example/show_ep01.mkv",
            "fallbackMode": "auto",
            "deviceHasDvDecoder": false
        }"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
        assert_eq!(p["profile"], "P8.4");
        assert_eq!(p["compatibility"], "HLG");
        assert_ne!(
            p["compatibility"], "HDR10",
            "P8.4 has HLG base, must not be labelled HDR10"
        );
        assert_eq!(p["safety"], "medium");
    }

    #[test]
    fn sample_unknown_profile_from_addon_with_only_dv_keyword() {
        // Many addons only set a "Dolby Vision" label without specifying the
        // profile. Without profile info the only safe action is none.
        let p = plan(
            r#"{
            "stream": {
                "name": "4K | Dolby Vision | DD+ Atmos",
                "description": "UHD Remux"
            },
            "url": "https://debrid.example/movie.mkv",
            "fallbackMode": "auto",
            "deviceHasDvDecoder": false
        }"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "unknown_profile_no_safe_fallback");
        let limitations = p["limitations"].as_array().unwrap();
        assert!(
            limitations
                .iter()
                .any(|l| l.as_str().unwrap().contains("set_dvProfile_field"))
        );
    }

    #[test]
    fn sample_p7_rpu_convert_on_raw_hevc_dv8_mode() {
        // Raw Annex-B HEVC + P7 + dv8 mode → live RPU conversion. The only
        // case where rpu_convert is emitted instead of dvcc_strip.
        let p = plan(
            r#"{
            "stream": {
                "name": "RAW HEVC | 4K | dvhe.07.06",
                "dvProfile": 7
            },
            "url": "https://cdn.example/stream.hevc",
            "fallbackMode": "dv8",
            "deviceHasDvDecoder": false
        }"#,
        );
        assert_eq!(p["action"], "rpu_convert");
        assert_eq!(p["profile"], "P7");
        assert_eq!(p["compatibility"], "DV8");
        assert_eq!(p["rpuMode"], 2);
    }

    #[test]
    fn convert_dv81_p7_mkv_decoder_no_display_returns_rpu_convert() {
        // Decoder present, no DV display: MKV now supported via EBML RPU rewriter.
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/movie.mkv","fallbackMode":"convert_dv81","deviceHasDvDecoder":true,"deviceHasDvDisplay":false}"#,
        );
        assert_eq!(p["action"], "rpu_convert");
        assert_eq!(p["reason"], "p7_rpu_convert_to_dv81");
    }

    #[test]
    fn convert_dv81_p7_mp4_decoder_no_display_returns_rpu_convert() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/movie.mp4","fallbackMode":"convert_dv81","deviceHasDvDecoder":true,"deviceHasDvDisplay":false}"#,
        );
        assert_eq!(p["action"], "rpu_convert");
        assert_eq!(p["reason"], "p7_rpu_convert_to_dv81");
    }

    #[test]
    fn convert_dv81_p7_raw_hevc_decoder_no_display_returns_rpu_convert() {
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/movie.hevc","fallbackMode":"convert_dv81","deviceHasDvDecoder":true,"deviceHasDvDisplay":false}"#,
        );
        assert_eq!(p["action"], "rpu_convert");
        assert_eq!(p["reason"], "p7_rpu_convert_to_dv81");
    }

    #[test]
    fn convert_dv81_decoder_and_display_returns_native_passthrough() {
        // Full DV device → native, no proxy needed.
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/movie.mp4","fallbackMode":"convert_dv81","deviceHasDvDecoder":true,"deviceHasDvDisplay":true}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "hw_dv_decoder");
    }

    #[test]
    fn convert_dv81_no_decoder_falls_back_to_dvcc_strip() {
        // No DV decoder → same as Auto: strip to HDR10.
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/movie.mp4","fallbackMode":"convert_dv81","deviceHasDvDecoder":false,"deviceHasDvDisplay":false}"#,
        );
        assert_eq!(p["action"], "dvcc_strip");
    }

    #[test]
    fn convert_dv81_hls_p7_decoder_returns_hls_rpu_convert() {
        // P7 HLS with a DV decoder available: segment-level RPU rewrite
        // (fluxa-streaming-engine's OkHttp interceptor), not the plain
        // manifest passthrough.
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/index.m3u8","fallbackMode":"convert_dv81","deviceHasDvDecoder":true,"deviceHasDvDisplay":false}"#,
        );
        assert_eq!(p["action"], "hls_rpu_convert");
        assert_eq!(p["reason"], "p7_hls_segment_rpu_convert");
    }

    #[test]
    fn convert_dv81_hls_p7_no_decoder_deferred_to_manifest_rewrite() {
        // Without a DV decoder there's nothing to convert into, so HLS
        // still just defers to manifest passthrough.
        let p = plan(
            r#"{"stream":{"dvProfile":7},"url":"https://cdn.example/index.m3u8","fallbackMode":"convert_dv81","deviceHasDvDecoder":false,"deviceHasDvDisplay":false}"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "manifest_handled");
    }

    #[test]
    fn sample_hls_stream_always_deferred_to_manifest_rewrite() {
        // HLS streams are handled by the OkHttp interceptor regardless of profile.
        // The proxy must never be activated for .m3u8 URLs.
        let p = plan(
            r#"{
            "stream": {
                "name": "Apple TV+ | 4K | dvhe.08.01",
                "dvProfile": 8,
                "dvCompatId": 1
            },
            "url": "https://cdn.example/master.m3u8",
            "fallbackMode": "auto",
            "deviceHasDvDecoder": false
        }"#,
        );
        assert_eq!(p["action"], "none");
        assert_eq!(p["reason"], "manifest_handled");
    }
}
