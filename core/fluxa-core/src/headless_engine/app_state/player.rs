use super::*;

pub fn app_core_set_player_position(handle: u64, position_ms: i64) -> bool {
    update_player(handle, |player| player.last_saved_position = position_ms)
}

pub fn app_core_set_player_buffering(handle: u64, buffering: bool) -> bool {
    update_player(handle, |player| player.is_buffering = buffering)
}

pub fn app_core_set_player_stream_index(handle: u64, stream_index: i64) -> bool {
    update_player(handle, |player| player.current_stream_index = stream_index)
}

pub fn app_core_set_player_playback_ended(handle: u64, ended: bool) -> bool {
    update_player(handle, |player| player.playback_ended = ended)
}

pub fn app_core_set_player_video_rendered(handle: u64, rendered: bool) -> bool {
    update_player(handle, |player| player.is_video_rendered = rendered)
}

pub fn app_core_set_player_started(handle: u64, started: bool) -> bool {
    update_player(handle, |player| player.has_started_playing = started)
}

pub fn app_core_update_player(
    handle: u64,
    position_ms: i64,
    stream_index: i64,
    buffering: bool,
    playback_ended: bool,
    started: bool,
    rendered: bool,
) -> bool {
    let Some(state) = lock_store().get(&handle).cloned() else {
        return false;
    };
    let Some(mut state) = lock_app_state(&state) else {
        return false;
    };
    state.player.last_saved_position = position_ms;
    state.player.current_stream_index = stream_index;
    state.player.is_buffering = buffering;
    state.player.playback_ended = playback_ended;
    state.player.has_started_playing = started;
    state.player.is_video_rendered = rendered;
    true
}

pub(super) fn update_player(handle: u64, update: impl FnOnce(&mut PlayerCoreState)) -> bool {
    let Some(state) = lock_store().get(&handle).cloned() else {
        return false;
    };
    let Some(mut state) = lock_app_state(&state) else {
        return false;
    };
    update(&mut state.player);
    true
}
