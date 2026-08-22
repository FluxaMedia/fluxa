use crate::player_flow::PlayerFlowState;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(in crate::headless_engine) struct PlayerState {
    pub(super) current_video_id: Value,
    pub(super) current_streams: Value,
    pub(super) current_stream_index: i64,
    pub(super) last_position_ms: i64,
    pub(super) current_url: Value,
    pub(super) resolved_url: Value,
    pub(super) zero_speed_ticks: i64,
    pub(super) is_buffering: bool,
    pub(super) is_video_rendered: bool,
    pub(super) player_error: Value,
    pub(super) preferred_binge_group: Value,
    pub(super) pending_stream_load: Value,
    pub(super) prefetching_next_video_id: Value,
    pub(super) prefetched_next_episode: Value,
    pub(super) subtitle_loading: bool,
    pub(super) subtitles: Value,
    pub(super) intro_segments: Value,
    pub(super) intro_imdb_id: Value,
    pub(super) last_scrobble: Value,
    pub(super) direct_playback_target: Value,
    pub(super) stop_torrent_warning: Value,
    pub(super) generation: u64,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            current_video_id: Value::Null,
            current_streams: serde_json::json!([]),
            current_stream_index: 0,
            last_position_ms: 0,
            current_url: Value::Null,
            resolved_url: Value::Null,
            zero_speed_ticks: 0,
            is_buffering: true,
            is_video_rendered: false,
            player_error: Value::Null,
            preferred_binge_group: Value::Null,
            pending_stream_load: Value::Null,
            prefetching_next_video_id: Value::Null,
            prefetched_next_episode: Value::Null,
            subtitle_loading: false,
            subtitles: serde_json::json!([]),
            intro_segments: serde_json::json!([]),
            intro_imdb_id: Value::Null,
            last_scrobble: Value::Null,
            direct_playback_target: Value::Null,
            stop_torrent_warning: Value::Null,
            generation: 0,
        }
    }
}

impl PlayerState {
    pub(super) fn to_flow_state(&self) -> PlayerFlowState {
        PlayerFlowState {
            current_video_id: self.current_video_id.as_str().map(str::to_string),
            current_streams: self.current_streams.as_array().cloned().unwrap_or_default(),
            current_stream_index: self.current_stream_index as i32,
            current_url: self.current_url.as_str().map(str::to_string),
            zero_speed_ticks: self.zero_speed_ticks as i32,
            is_buffering: self.is_buffering,
            is_video_rendered: self.is_video_rendered,
            player_error: self.player_error.as_str().map(str::to_string),
            preferred_binge_group: self.preferred_binge_group.as_str().map(str::to_string),
        }
    }

    // The player_flow sub-engine owns only the playback-selection fields (current
    // video/streams/url/etc). Applying its result wholesale replaces the player
    // namespace, dropping every headless-level extension field (pendingStreamLoad,
    // prefetch cache, subtitles, ...) back to default — this mirrors the previous
    // behavior of overwriting `engine.state["player"]` with the flow's state outright.
    pub(super) fn from_flow_state(flow_state: PlayerFlowState) -> Self {
        Self {
            current_video_id: flow_state
                .current_video_id
                .map(Value::String)
                .unwrap_or(Value::Null),
            current_streams: Value::Array(flow_state.current_streams),
            current_stream_index: flow_state.current_stream_index as i64,
            current_url: flow_state
                .current_url
                .map(Value::String)
                .unwrap_or(Value::Null),
            zero_speed_ticks: flow_state.zero_speed_ticks as i64,
            is_buffering: flow_state.is_buffering,
            is_video_rendered: flow_state.is_video_rendered,
            player_error: flow_state
                .player_error
                .map(Value::String)
                .unwrap_or(Value::Null),
            preferred_binge_group: flow_state
                .preferred_binge_group
                .map(Value::String)
                .unwrap_or(Value::Null),
            ..Self::default()
        }
    }
}
