use super::*;

#[tauri::command]
pub fn player_torrent_sibling_subtitles(state: State<DesktopState>) -> Vec<Value> {
    torrent_sibling_subtitles(&state)
        .into_iter()
        .map(|(url, title, language)| json!({ "url": url, "title": title, "language": language }))
        .collect()
}

#[tauri::command]
pub fn player_add_subtitle(
    state: State<DesktopState>,
    url: String,
    title: Option<String>,
    language: Option<String>,
) -> Result<(), String> {
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::AvPlayer {
        if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
            return surface.add_subtitle(url, title, language);
        }
    }
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Mpv {
        if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
            return surface.add_subtitle(url, title, language);
        }
    }
    match with_renderer_retry(&state, 80, |renderer| {
        renderer.add_subtitle(&url, title.as_deref(), language.as_deref())
    }) {
        Ok(Some(())) => Ok(()),
        Ok(None) => Err("player renderer is not initialized".to_string()),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub fn player_render_frame(
    state: State<DesktopState>,
    width: i32,
    height: i32,
) -> Result<mpv_render::PlayerFrame, String> {
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Vlc {
        return Err("headless frame rendering is not supported by the libvlc engine".to_string());
    }
    let mut renderer = state.player_render_state.lock().unwrap();
    renderer
        .as_mut()
        .ok_or_else(|| "player renderer is not initialized".to_string())?
        .render_frame(width, height)
}

#[tauri::command]
pub fn player_set_anime4k_enabled(
    app: AppHandle,
    state: State<DesktopState>,
    enabled: bool,
    quality: Option<String>,
    mode: Option<String>,
) -> Result<(), String> {
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::AvPlayer {
        state.player_overlay.lock().unwrap().anime4k_enabled = false;
        let _ = app.emit(
            "player-anime4k-state",
            serde_json::json!({ "enabled": false }),
        );
        return Ok(());
    }
    let commands: Vec<Vec<String>> = if enabled {
        let chain_path = resolve_anime4k_chain(
            Some(&app),
            quality.as_deref().unwrap_or("anime4k_m"),
            mode.as_deref().unwrap_or("a"),
        )
        .ok_or_else(|| {
            log::error!(
                "player_set_anime4k_enabled: shader chain not found for quality={:?} mode={:?}",
                quality,
                mode
            );
            "Anime4K shader chain not found".to_string()
        })?;
        vec![
            vec![
                "change-list".to_string(),
                "glsl-shaders".to_string(),
                "set".to_string(),
                chain_path,
            ],
            vec![
                "set".to_string(),
                "scale".to_string(),
                "ewa_lanczossharp".to_string(),
            ],
            vec![
                "set".to_string(),
                "cscale".to_string(),
                "ewa_lanczos".to_string(),
            ],
            vec![
                "set".to_string(),
                "dscale".to_string(),
                "mitchell".to_string(),
            ],
            vec![
                "set".to_string(),
                "correct-downscaling".to_string(),
                "yes".to_string(),
            ],
            vec![
                "set".to_string(),
                "linear-downscaling".to_string(),
                "yes".to_string(),
            ],
        ]
    } else {
        vec![
            vec![
                "change-list".to_string(),
                "glsl-shaders".to_string(),
                "clr".to_string(),
                String::new(),
            ],
            vec![
                "set".to_string(),
                "scale".to_string(),
                "bilinear".to_string(),
            ],
            vec![
                "set".to_string(),
                "cscale".to_string(),
                "bilinear".to_string(),
            ],
            vec![
                "set".to_string(),
                "dscale".to_string(),
                "mitchell".to_string(),
            ],
        ]
    };
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Mpv {
        if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
            surface.command_args(commands)?;
            state.player_overlay.lock().unwrap().anime4k_enabled = enabled;
            let _ = app.emit(
                "player-anime4k-state",
                serde_json::json!({ "enabled": enabled }),
            );
            return Ok(());
        }
    }
    for command in commands {
        let args = command.iter().map(String::as_str).collect::<Vec<_>>();
        match with_renderer_retry(&state, 600, |renderer| renderer.command_args(&args)) {
            Ok(Some(())) => {}
            Ok(None) => return Err("player renderer is not initialized".to_string()),
            Err(err) => {
                log::error!(
                    "player_set_anime4k_enabled: command {:?} failed: {err}",
                    args
                );
                return Err(err);
            }
        }
    }
    state.player_overlay.lock().unwrap().anime4k_enabled = enabled;
    let _ = app.emit(
        "player-anime4k-state",
        serde_json::json!({ "enabled": enabled }),
    );
    Ok(())
}

#[tauri::command]
pub fn player_get_anime4k_enabled(state: State<DesktopState>) -> bool {
    state.player_overlay.lock().unwrap().anime4k_enabled
}

#[tauri::command]
pub fn player_command(
    _app: AppHandle,
    state: State<DesktopState>,
    command: String,
) -> Result<(), String> {
    if command == "stop" {
        state.player_overlay.lock().unwrap().eof_next_fired = true;
    }
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Mpv {
        if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
            return surface.command(command);
        }
    }
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::AvPlayer {
        if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
            return surface.command(command);
        }
    }
    match with_renderer_retry_mut(&state, 60, |renderer| renderer.user_command(&command)) {
        Ok(Some(())) => Ok(()),
        Ok(None) => Err("player renderer is not initialized".to_string()),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub fn player_set_sleep_inhibition(
    state: State<DesktopState>,
    enabled: bool,
) -> Result<(), String> {
    state.sleep_inhibitor.lock().unwrap().set_enabled(enabled)
}
