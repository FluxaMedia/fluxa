mod complete;
mod intro_subtitles;
mod playback;
mod scrobble;
mod state;
mod stream_load;

use super::HeadlessEngine;

pub(super) use complete::complete;
pub(super) use intro_subtitles::{
    dispatch_intro_imdb_id, dispatch_intro_segments, dispatch_subtitle_load,
};
pub(super) use playback::{complete_direct_playback, dispatch_resolve_playback};
pub(super) use scrobble::dispatch_scrobble;
pub(super) use state::PlayerState;
pub(super) use stream_load::{
    dispatch_load_streams, dispatch_next_episode_prefetch, dispatch_streams_failed,
    dispatch_streams_loaded,
};

pub(super) fn set_buffering(engine: &mut HeadlessEngine, buffering: bool) {
    engine.state.player.is_buffering = buffering;
}

pub(super) fn set_stream_index(engine: &mut HeadlessEngine, stream_index: i64) {
    engine.state.player.current_stream_index = stream_index;
}

pub(super) fn set_position(engine: &mut HeadlessEngine, position_ms: i64) {
    engine.state.player.last_position_ms = position_ms;
}
