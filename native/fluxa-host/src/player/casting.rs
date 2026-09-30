use super::{Panel, PlayerSession};
use crate::RendererState;
use crate::cast::Row;

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
