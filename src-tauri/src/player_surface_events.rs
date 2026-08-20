use crate::DesktopState;
use crate::mpv_render::PlayerEvent;
use fluxa_core::FluxaCore;
use tauri::{AppHandle, Emitter, Manager};

pub(crate) fn drain_player_events(app: &AppHandle, events: Vec<PlayerEvent>) -> bool {
    let mut eof_seen = false;
    for event in events {
        match event {
            PlayerEvent::PauseChanged(paused) => {
                let _ = app.emit("native-player-pause-changed", paused);
            }
            PlayerEvent::EndFile { eof, error } => {
                log::info!("player surface: mpv END_FILE event eof={eof} error={error:?}");
                if let Some(message) = error {
                    log::error!("player surface: stream failed to play: {message}");
                    let _ = app.emit("native-player-error", message);
                } else if eof {
                    eof_seen = true;
                }
            }
        }
    }
    eof_seen
}

pub(crate) fn clear_eof_latch(app: &AppHandle) {
    let state = app.state::<DesktopState>();
    let mut overlay = state.player_overlay.lock().unwrap();
    if overlay.eof_next_fired {
        overlay.eof_next_fired = false;
    }
}

pub(crate) fn fire_eof_transition(app: &AppHandle) {
    let state = app.state::<DesktopState>();
    let mut overlay = state.player_overlay.lock().unwrap();
    if !overlay.take_eof_next() {
        return;
    }
    let next_sub = overlay.next_ep_subtitle.clone();
    let auto_play = overlay.auto_play_next_episode;
    drop(overlay);
    if FluxaCore::should_play_next_episode(!next_sub.is_empty(), auto_play) {
        log::info!("player surface: eof reached, auto-playing next episode");
        let _ = app.emit("native-player-next-episode", ());
    } else {
        log::info!("player surface: eof reached, closing player");
        let _ = app.emit("native-player-close-requested", ());
    }
}
