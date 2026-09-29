mod anime4k;
mod audio_channels;
mod audio_tracks;
mod backend_selection;
mod buffer_targets;
mod dolby_vision;
mod next_episode;
mod playback_close;
mod retry_and_ordering;
mod shuffle;
mod source_sidebar;
mod torrent_fallback;

pub(crate) use self::dolby_vision::*;
pub(crate) use anime4k::anime4k_shader_chain_json;
pub(crate) use audio_channels::audio_pcm_channel_count_json;
pub(crate) use audio_tracks::select_audio_track_json;
pub(crate) use backend_selection::player_backend_selection_json;
pub(crate) use buffer_targets::player_buffer_targets_json;
pub(crate) use next_episode::{can_prefetch_next_episode_json, select_next_episode_stream_json};
pub(crate) use playback_close::{playback_close_plan_json, playback_preferences_plan_json};
pub(crate) use retry_and_ordering::{
    next_retry_source_plan_json, order_streams_plan_json, player_retry_policy_json,
    stream_shell_plan_json,
};
pub(crate) use shuffle::shuffle_episode_pick_json;
pub use source_sidebar::player_source_sidebar_plan_json;
pub(crate) use torrent_fallback::torrent_fallback_file_policy_json;

pub(crate) fn stream_subtitles_result_json(stream_json: &str) -> Option<String> {
    let stream = serde_json::from_str::<serde_json::Value>(stream_json).ok()?;
    let subtitles = stream
        .get("subtitles")
        .filter(|value| value.is_array())
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    Some(serde_json::json!({ "subtitles": subtitles }).to_string())
}

#[cfg(test)]
mod tests;
