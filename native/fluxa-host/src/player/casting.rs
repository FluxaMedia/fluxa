use super::{Panel, PlayerSession};
use crate::RendererState;
use crate::cast::{Device, Row};

pub(super) fn open(player: &mut PlayerSession) {
    player.cast.scan();
    player.panel = Some(Panel::Cast);
}

pub(super) fn activate_row(state: &mut RendererState, index: usize) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let Some(row) = player.cast.rows().get(index).copied() else {
        return;
    };
    let Row::Device(device) = row else {
        player.cast.apply(row);
        return;
    };
    let (Some(device), Some(url)) = (player.cast.device(device), player.loaded_url.clone()) else {
        return;
    };
    let title = player.title();
    let position = player.status.position;
    let paused = player.status.paused;
    player.cast.start(device, &url, &title, position);
    if !paused {
        super::command(state, super::VideoCommand::TogglePause);
    }
}

pub(crate) fn cast_to(state: &mut RendererState, device: Device, url: &str, title: &str) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let paused = player.status.paused;
    player.cast.start(device, url, title, 0.0);
    if !paused {
        super::command(state, super::VideoCommand::TogglePause);
    }
    crate::sync::show_notice(state, "sources.play_on_tv_started");
}
