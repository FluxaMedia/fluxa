mod complete;
mod intro_subtitles;
mod playback;
mod scrobble;
mod state;
mod stream_load;
pub(crate) mod trailer;
#[cfg(feature = "js-engine")]
mod youtube_cipher;

use crate::headless_engine::HeadlessEngine;

pub(crate) use complete::complete;
pub(crate) use intro_subtitles::{
    dispatch_intro_imdb_id, dispatch_intro_segments, dispatch_subtitle_load,
};
pub(crate) use playback::{
    complete_direct_playback, dispatch_resolve_playback, reset_for_direct_playback,
};
pub(crate) use scrobble::dispatch_scrobble;
pub(crate) use state::PlayerState;
pub(crate) use stream_load::{
    dispatch_continue_watching_playback, dispatch_load_streams, dispatch_next_episode_prefetch,
    dispatch_streams_failed, dispatch_streams_loaded,
};

pub(crate) fn dispatch_reset_for_episode(engine: &mut HeadlessEngine, video_id: String) {
    engine.state.player.current_video_id = serde_json::Value::String(video_id);
    engine.state.player.current_stream_index = 0;
    engine.state.player.last_position_ms = 0;
    engine.state.player.is_buffering = true;
    engine.state.player.is_video_rendered = false;
    engine.state.player.playback_ended = false;
    engine.state.player.has_started_playing = false;
}

pub(crate) fn dispatch_telemetry(
    engine: &mut HeadlessEngine,
    position_ms: i64,
    stream_index: i64,
    buffering: bool,
    playback_ended: bool,
    started: bool,
    rendered: bool,
) {
    engine.state.player.last_position_ms = position_ms.max(0);
    engine.state.player.current_stream_index = stream_index.max(0);
    engine.state.player.is_buffering = buffering;
    engine.state.player.playback_ended = playback_ended;
    engine.state.player.has_started_playing = started;
    engine.state.player.is_video_rendered = rendered;
}

pub(crate) fn set_buffering(engine: &mut HeadlessEngine, buffering: bool) {
    engine.state.player.is_buffering = buffering;
}

pub(crate) fn set_stream_index(engine: &mut HeadlessEngine, stream_index: i64) {
    engine.state.player.current_stream_index = stream_index;
}

pub(crate) fn set_position(engine: &mut HeadlessEngine, position_ms: i64) {
    engine.state.player.last_position_ms = position_ms;
}
