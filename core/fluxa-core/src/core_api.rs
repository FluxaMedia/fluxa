use crate::stream_policy;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
use crate::{
    cast_protocol, desktop_playback, headless_engine, library_persistence, oauth_plan,
    offline_download, player_policy, subtitle_sync,
};

pub struct FluxaCore;

// desktop calls these directly rather than through ffi::core_invoke, so each
// one needs its own panic guard rather than inheriting core_invoke's.
fn guard<T>(default: T, f: impl FnOnce() -> T) -> T {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(default)
}

impl FluxaCore {
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn oauth_request_plan_json(request_json: &str) -> Option<String> {
        guard(None, || oauth_plan::oauth_request_plan_json(request_json))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn oauth_response_outcome(service: &str, operation: &str, status: u16) -> &'static str {
        guard("error", || {
            oauth_plan::oauth_response_outcome(service, operation, status)
        })
    }
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn create_headless_engine(initial_json: &str) -> u64 {
        guard(0, || headless_engine::create_headless_engine(initial_json))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn headless_engine_snapshot_json(handle: u64) -> Option<String> {
        guard(None, || {
            headless_engine::headless_engine_snapshot_json(handle)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn headless_engine_dispatch_json(handle: u64, action_json: &str) -> Option<String> {
        guard(None, || {
            headless_engine::headless_engine_dispatch_json(handle, action_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn headless_engine_complete_effect_json(handle: u64, result_json: &str) -> Option<String> {
        guard(None, || {
            headless_engine::headless_engine_complete_effect_json(handle, result_json)
        })
    }

    pub fn stream_playback_info_json(stream_json: &str) -> Option<String> {
        guard(None, || {
            stream_policy::stream_playback_info_json(stream_json)
        })
    }

    pub fn torrent_runtime_info_json(request_json: &str) -> Option<String> {
        guard(None, || {
            stream_policy::torrent_runtime_info_json(request_json)
        })
    }

    pub fn stream_magnet_link_json(stream_json: &str) -> Option<String> {
        guard(None, || stream_policy::stream_magnet_link_json(stream_json))
    }

    pub fn torrent_sibling_subtitle_matches_json(request_json: &str) -> Option<String> {
        guard(None, || {
            stream_policy::torrent_sibling_subtitle_matches_json(request_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn player_buffer_targets_json(request_json: &str) -> Option<String> {
        guard(None, || {
            player_policy::player_buffer_targets_json(request_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn should_play_next_episode(has_next_episode: bool, auto_play: bool) -> bool {
        guard(false, || {
            desktop_playback::should_play_next_episode(has_next_episode, auto_play)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn chapter_skip_segments_json(chapters_json: &str) -> String {
        guard("[]".to_string(), || {
            desktop_playback::chapter_skip_segments_json(chapters_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn library_progress_entries_json(document_json: &str) -> String {
        guard("[]".to_string(), || {
            library_persistence::progress_entries_json(document_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn library_items_json(document_json: &str) -> String {
        guard("[]".to_string(), || {
            library_persistence::library_items_json(document_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn library_watched_video_ids_json(document_json: &str) -> String {
        guard("[]".to_string(), || {
            library_persistence::watched_video_ids_json(document_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn library_last_watched_entries_json(document_json: &str) -> String {
        guard("[]".to_string(), || {
            library_persistence::last_watched_entries_json(document_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn library_continue_watching_entries_json(document_json: &str) -> String {
        guard("[]".to_string(), || {
            library_persistence::continue_watching_entries_json(document_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn subtitle_sync_estimate_json(request_json: &str) -> Option<String> {
        guard(None, || {
            subtitle_sync::estimate_subtitle_delay_json(request_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn subtitle_cues_around_time_json(request_json: &str) -> Option<String> {
        guard(None, || {
            subtitle_sync::subtitle_cues_around_time_json(request_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn subtitle_sync_capture_json(request_json: &str) -> Option<String> {
        guard(None, || subtitle_sync::subtitle_sync_capture_json(request_json))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn subtitle_sync_apply_json(request_json: &str) -> Option<String> {
        guard(None, || subtitle_sync::subtitle_sync_apply_json(request_json))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn subtitle_cue_list_json(request_json: &str) -> Option<String> {
        guard(None, || subtitle_sync::subtitle_cue_list_json(request_json))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn offline_download_plan_json(request_json: &str) -> Option<String> {
        guard(None, || {
            offline_download::offline_download_plan_json(request_json)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn validate_stream_url(url: &str) -> bool {
        guard(false, || cast_protocol::validate_stream_url(url))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn dlna_parse_device_description(xml: &str, base_url: &str) -> Option<String> {
        guard(None, || {
            cast_protocol::dlna_parse_device_description_json(xml, base_url)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn dlna_soap_action_body(urn: &str, action: &str, args: &str) -> String {
        guard(String::new(), || {
            cast_protocol::soap_action_body(urn, action, args)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn dlna_set_av_transport_args(
        media_url: &str,
        title: &str,
        subtitle_url: Option<&str>,
    ) -> Option<String> {
        guard(None, || {
            cast_protocol::dlna_set_av_transport_args(media_url, title, subtitle_url)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn dlna_seek_args(position_secs: f64) -> String {
        guard(String::new(), || {
            cast_protocol::dlna_seek_args(position_secs)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn dlna_set_volume_args(level: f64) -> String {
        guard(String::new(), || cast_protocol::dlna_set_volume_args(level))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn dlna_resolve_loopback_url(stream_url: &str, lan_ip: &str) -> String {
        guard(stream_url.to_string(), || {
            cast_protocol::resolve_loopback_url(stream_url, lan_ip)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn chromecast_guess_content_type(media_url: &str) -> String {
        guard("video/mp4".to_string(), || {
            cast_protocol::guess_cast_content_type(media_url).to_string()
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn chromecast_encode_message(
        source_id: &str,
        destination_id: &str,
        namespace: &str,
        payload_utf8: &str,
    ) -> Vec<u8> {
        guard(Vec::new(), || {
            cast_protocol::encode_cast_message(source_id, destination_id, namespace, payload_utf8)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn chromecast_decode_message(buf: &[u8]) -> Option<(String, String)> {
        guard(None, || {
            cast_protocol::decode_cast_message(buf).map(|m| (m.namespace, m.payload_utf8))
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_PLAY: u8 = cast_protocol::FCAST_OP_PLAY;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_PAUSE: u8 = cast_protocol::FCAST_OP_PAUSE;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_RESUME: u8 = cast_protocol::FCAST_OP_RESUME;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_STOP: u8 = cast_protocol::FCAST_OP_STOP;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_SEEK: u8 = cast_protocol::FCAST_OP_SEEK;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_PLAYBACK_UPDATE: u8 = cast_protocol::FCAST_OP_PLAYBACK_UPDATE;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_VOLUME_UPDATE: u8 = cast_protocol::FCAST_OP_VOLUME_UPDATE;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_SET_VOLUME: u8 = cast_protocol::FCAST_OP_SET_VOLUME;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_PLAYBACK_ERROR: u8 = cast_protocol::FCAST_OP_PLAYBACK_ERROR;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_SET_SPEED: u8 = cast_protocol::FCAST_OP_SET_SPEED;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_VERSION: u8 = cast_protocol::FCAST_OP_VERSION;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_PING: u8 = cast_protocol::FCAST_OP_PING;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_OP_PONG: u8 = cast_protocol::FCAST_OP_PONG;
    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub const FCAST_MAX_MESSAGE_BYTES: usize = cast_protocol::FCAST_MAX_MESSAGE_BYTES;

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn fcast_encode_message(opcode: u8, body_json: &str) -> Option<Vec<u8>> {
        guard(None, || cast_protocol::fcast_encode_message(opcode, body_json))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn fcast_decode_message(buf: &[u8]) -> Option<(u8, String)> {
        guard(None, || cast_protocol::fcast_decode_message(buf))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn fcast_play_body(media_url: &str, resume_position_secs: f64) -> Option<String> {
        guard(None, || {
            cast_protocol::fcast_play_body(media_url, resume_position_secs)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn fcast_seek_body(position_secs: f64) -> String {
        guard(String::new(), || cast_protocol::fcast_seek_body(position_secs))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn fcast_set_volume_body(level: f64) -> String {
        guard(String::new(), || cast_protocol::fcast_set_volume_body(level))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn fcast_set_speed_body(speed: f64) -> String {
        guard(String::new(), || cast_protocol::fcast_set_speed_body(speed))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn fcast_version_body(version: u32) -> String {
        guard(String::new(), || cast_protocol::fcast_version_body(version))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn fcast_playback_update(body_json: &str) -> Option<(u8, f64, f64, f64)> {
        guard(None, || cast_protocol::fcast_playback_update(body_json))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn fcast_error_message(body_json: &str) -> Option<String> {
        guard(None, || cast_protocol::fcast_error_message(body_json))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn roku_device_name(xml: &str) -> Option<String> {
        guard(None, || cast_protocol::roku_device_name(xml))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn roku_launch_url(
        host: &str,
        media_url: &str,
        subtitle_url: Option<&str>,
    ) -> Option<String> {
        guard(None, || {
            cast_protocol::roku_launch_url(host, media_url, subtitle_url)
        })
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn airplay_volume_db(level: f64) -> f64 {
        guard(-30.0, || cast_protocol::airplay_volume_db(level))
    }

    #[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
    pub fn airplay_play_body(media_url: &str) -> Option<String> {
        guard(None, || cast_protocol::airplay_play_body(media_url))
    }
}

#[cfg(test)]
mod tests {
    use super::guard;

    #[test]
    fn guard_recovers_from_a_panic_instead_of_propagating_it() {
        let result = guard(42, || std::panic::panic_any("boom"));
        assert_eq!(result, 42);
    }
}
