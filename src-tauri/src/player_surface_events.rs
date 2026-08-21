use crate::DesktopState;
use crate::mpv_render::PlayerEvent;
use fluxa_core::FluxaCore;
use tauri::{AppHandle, Emitter, Manager};

pub(crate) fn engine_command(app: &AppHandle, command: String) -> Result<(), String> {
    let state = app.state::<DesktopState>();
    match crate::player::with_renderer_retry_mut(&state, 60, |renderer| {
        renderer.user_command(&command)
    }) {
        Ok(Some(())) => Ok(()),
        Ok(None) => Err("player renderer is not initialized".to_string()),
        Err(error) => Err(error),
    }
}

pub(crate) fn engine_command_args(
    app: &AppHandle,
    commands: Vec<Vec<String>>,
) -> Result<(), String> {
    let state = app.state::<DesktopState>();
    for command in commands {
        let args = command.iter().map(String::as_str).collect::<Vec<_>>();
        match crate::player::with_renderer_retry(&state, 600, |renderer| {
            renderer.command_args(&args)
        }) {
            Ok(Some(())) => {}
            Ok(None) => return Err("player renderer is not initialized".to_string()),
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

pub(crate) fn engine_status(
    app: &AppHandle,
) -> Result<crate::mpv_render::PlayerStatus, String> {
    let state = app.state::<DesktopState>();
    match crate::player::with_renderer_retry(&state, 80, |renderer| Ok(renderer.status())) {
        Ok(Some(status)) => Ok(status),
        Ok(None) => Err("player renderer is not initialized".to_string()),
        Err(error) => Err(error),
    }
}

pub(crate) fn engine_track_options(
    app: &AppHandle,
    track_type: String,
) -> Result<Vec<crate::mpv_render::PlayerTrackOption>, String> {
    let state = app.state::<DesktopState>();
    match crate::player::with_renderer_retry(&state, 80, |renderer| {
        Ok(renderer.track_options(&track_type))
    }) {
        Ok(Some(tracks)) => Ok(tracks),
        Ok(None) => Err("player renderer is not initialized".to_string()),
        Err(error) => Err(error),
    }
}

pub(crate) fn engine_add_subtitle(
    app: &AppHandle,
    url: String,
    title: Option<String>,
    language: Option<String>,
) -> Result<(), String> {
    let state = app.state::<DesktopState>();
    match crate::player::with_renderer_retry(&state, 80, |renderer| {
        renderer.add_subtitle(&url, title.as_deref(), language.as_deref())
    }) {
        Ok(Some(())) => Ok(()),
        Ok(None) => Err("player renderer is not initialized".to_string()),
        Err(error) => Err(error),
    }
}

pub(crate) fn check_player_events(app: &AppHandle) {
    let state = app.state::<DesktopState>();
    let (events, eof) = {
        let Ok(mut renderer) = state.player_mpv_client.try_lock() else {
            return;
        };
        let Some(renderer) = renderer.as_mut() else {
            return;
        };
        let events = renderer.poll_events();
        let eof = renderer.query_property("eof-reached").as_deref() == Some("yes");
        (events, eof)
    };
    let event_eof = drain_player_events(app, events);
    if eof || event_eof {
        fire_eof_transition(app);
    } else {
        clear_eof_latch(app);
    }
}

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
