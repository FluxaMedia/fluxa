use super::*;

#[tauri::command]
pub fn player_screenshot(
    state: State<DesktopState>,
    suggested_name: String,
) -> Result<String, String> {
    let base_dir = state
        .download_dir
        .lock()
        .unwrap()
        .clone()
        .or_else(|| state.data_dir.lock().unwrap().clone())
        .ok_or_else(|| "no writable directory available".to_string())?;
    let screenshots_dir = base_dir.join("Screenshots");
    std::fs::create_dir_all(&screenshots_dir).map_err(|e| e.to_string())?;

    let safe_name: String = suggested_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = screenshots_dir.join(format!("{safe_name}_{timestamp}.png"));
    let path_str = path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    match with_renderer_retry(&state, 60, |renderer| {
        renderer.command_string(&format!("screenshot-to-file \"{path_str}\" video"))
    }) {
        Ok(Some(())) => Ok(path.to_string_lossy().to_string()),
        Ok(None) => Err("player renderer is not initialized".to_string()),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub fn player_show_loading(
    app: AppHandle,
    state: State<DesktopState>,
    title: String,
    episode_title: Option<String>,
) {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
        surface.show_loading(title, episode_title);
        return;
    }
    let _ = app.emit(
        "native-player-title",
        serde_json::json!({ "title": title, "episodeTitle": episode_title }),
    );
}

#[tauri::command]
pub fn player_hide(app: AppHandle, state: State<DesktopState>) {
    state.pending_hide.store(true, Ordering::Release);
    let _ = state.sleep_inhibitor.lock().unwrap().set_enabled(false);
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Vlc {
        if let Ok(guard) = state.player_renderer_vlc.try_lock() {
            if let Some(player) = guard.as_ref() {
                let _ = player.command_string("stop");
            }
        }
    }
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
        surface.hide();
        return;
    }

    let _ = app.emit("native-player-hide", ());
}

#[tauri::command]
pub fn player_title(state: State<DesktopState>) -> Option<String> {
    with_renderer_retry(&state, 20, |renderer| Ok(renderer.title()))
        .ok()
        .flatten()
        .flatten()
}

#[tauri::command]
pub fn player_status(
    app: AppHandle,
    state: State<DesktopState>,
) -> Result<mpv_render::PlayerStatus, String> {
    if let Ok(mut guard) = state.player_renderer_vlc.try_lock() {
        if let Some(renderer) = guard.as_mut() {
            for event in renderer.poll_events() {
                let mpv_render::PlayerEvent::EndFile { eof, error } = event;
                if let Some(message) = error {
                    let _ = app.emit("native-player-error", message);
                } else if eof {
                    let next_sub = state.next_ep_subtitle.lock().unwrap().clone();
                    let auto_play = *state.auto_play_next_episode.lock().unwrap();
                    if FluxaCore::should_play_next_episode(!next_sub.is_empty(), auto_play) {
                        let _ = app.emit("native-player-next-episode", ());
                    } else {
                        let _ = app.emit("native-player-close-requested", ());
                    }
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Mpv {
        if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
            return surface.status();
        }
    }
    match with_renderer_retry(&state, 80, |renderer| Ok(renderer.status())) {
        Ok(Some(status)) => Ok(status),
        Ok(None) => Err("player renderer is not initialized".to_string()),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub fn player_get_playback_info(state: State<DesktopState>) -> serde_json::Value {
    serde_json::json!({
        "skipSegmentsJson": state.skip_segments_json.lock().unwrap().clone(),
        "chaptersJson": state.chapters_json.lock().unwrap().clone(),
        "episodesJson": state.episodes_json.lock().unwrap().clone(),
        "nextEpSubtitle": state.next_ep_subtitle.lock().unwrap().clone(),
        "nextEpThresholdPercent": *state.next_ep_threshold_percent.lock().unwrap(),
        "autoPlayNextEpisode": *state.auto_play_next_episode.lock().unwrap(),
        "autoPlayCountdownSecs": *state.auto_play_countdown_secs.lock().unwrap(),
        "autoSkipSegments": *state.auto_skip_segments.lock().unwrap(),
    })
}

#[tauri::command]
pub fn player_track_options(
    state: State<DesktopState>,
    track_type: String,
) -> Vec<mpv_render::PlayerTrackOption> {
    log::warn!("player_track_options invoked: track_type={track_type:?}");
    #[cfg(target_os = "windows")]
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Mpv {
        if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
            return match surface.track_options(track_type.clone()) {
                Ok(tracks) => {
                    log::warn!(
                        "player_track_options surface result: track_type={track_type:?}, count={}",
                        tracks.len()
                    );
                    tracks
                }
                Err(error) => {
                    log::error!(
                        "player_track_options surface failed: track_type={track_type:?}, error={error}"
                    );
                    Vec::new()
                }
            };
        }
    }
    match with_renderer_retry(&state, 80, |renderer| {
        Ok(renderer.track_options(&track_type))
    }) {
        Ok(Some(tracks)) => {
            log::warn!(
                "player_track_options result: track_type={track_type:?}, count={}",
                tracks.len()
            );
            tracks
        }
        Ok(None) => {
            log::warn!(
                "player_track_options result: track_type={track_type:?}, renderer unavailable"
            );
            Vec::new()
        }
        Err(error) => {
            log::error!("player_track_options failed: track_type={track_type:?}, error={error}");
            Vec::new()
        }
    }
}

#[tauri::command]
pub fn player_destroy(state: State<DesktopState>) -> bool {
    let _ = state.sleep_inhibitor.lock().unwrap().set_enabled(false);
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Vlc {
        return state.player_renderer_vlc.lock().unwrap().take().is_some();
    }
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
        surface.hide();
        return state.player_renderer.lock().unwrap().is_some();
    }

    state.player_renderer.lock().unwrap().take().is_some()
}

#[tauri::command]
pub fn player_set_chapters(state: State<DesktopState>, chapters_json: String) {
    *state.chapters_json.lock().unwrap() =
        if chapters_json.trim().is_empty() || chapters_json == "[]" {
            None
        } else {
            Some(chapters_json)
        };
}

#[tauri::command]
pub fn player_clear_chapters(state: State<DesktopState>) {
    *state.chapters_json.lock().unwrap() = None;
}

#[tauri::command]
pub fn player_set_skip_info(
    app: AppHandle,
    state: State<DesktopState>,
    segments_json: String,
    next_ep_subtitle: Option<String>,
    next_ep_threshold_percent: Option<f64>,
    auto_play_next_episode: Option<bool>,
    auto_play_countdown_secs: Option<u32>,
    auto_skip_segments: Option<bool>,
) {
    *state.skip_segments_json.lock().unwrap() =
        if segments_json.trim().is_empty() || segments_json == "[]" {
            None
        } else {
            Some(segments_json)
        };
    *state.next_ep_subtitle.lock().unwrap() = next_ep_subtitle.unwrap_or_default();
    *state.eof_next_fired.lock().unwrap() = false;
    if let Some(t) = next_ep_threshold_percent {
        *state.next_ep_threshold_percent.lock().unwrap() = t.clamp(1.0, 99.0);
    }
    if let Some(v) = auto_play_next_episode {
        *state.auto_play_next_episode.lock().unwrap() = v;
    }
    if let Some(s) = auto_play_countdown_secs {
        *state.auto_play_countdown_secs.lock().unwrap() = s.max(1);
    }
    if let Some(v) = auto_skip_segments {
        *state.auto_skip_segments.lock().unwrap() = v;
    }
    let _ = app.emit("player-skip-info-updated", ());
}

#[tauri::command]
pub fn player_clear_skip_info(app: AppHandle, state: State<DesktopState>) {
    *state.skip_segments_json.lock().unwrap() = None;
    *state.next_ep_subtitle.lock().unwrap() = String::new();
    *state.eof_next_fired.lock().unwrap() = false;
    let _ = app.emit("player-skip-info-updated", ());
}

#[tauri::command]
pub fn player_set_episodes(state: State<DesktopState>, episodes_json: String) {
    *state.episodes_json.lock().unwrap() =
        if episodes_json.trim() == "[]" || episodes_json.trim().is_empty() {
            None
        } else {
            Some(episodes_json)
        };
}

#[tauri::command]
pub fn player_clear_episodes(state: State<DesktopState>) {
    *state.episodes_json.lock().unwrap() = None;
}

#[tauri::command]
pub fn player_set_seek_thumbnail_enabled(state: State<DesktopState>, enabled: bool) {
    *state.seek_thumbnail_enabled.lock().unwrap() = enabled;
}

#[tauri::command]
pub fn player_get_seek_thumbnail(
    state: State<DesktopState>,
    time_pos: f64,
) -> Result<String, String> {
    use base64::{engine::general_purpose, Engine as _};

    if !*state.seek_thumbnail_enabled.lock().unwrap() {
        return Ok(String::new());
    }
    let url = state
        .thumb_url
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "no url".to_string())?;

    let mut renderer_guard = state.thumbnail_renderer.lock().unwrap();
    let mut loaded_url_guard = state.thumbnail_loaded_url.lock().unwrap();

    if renderer_guard.is_none() {
        *renderer_guard = Some(mpv_render::MpvRenderer::new_thumbnail()?);
    }
    let renderer = renderer_guard.as_mut().unwrap();

    if loaded_url_guard.as_deref() != Some(url.as_str()) {
        renderer.load_thumbnail(&url)?;
        *loaded_url_guard = Some(url.clone());
        for _ in 0..50 {
            if renderer.query_property("duration").is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    renderer.seek_to(time_pos)?;
    let mut still_seeking = true;
    for _ in 0..300 {
        if renderer.query_property("seeking").as_deref() != Some("yes") {
            still_seeking = false;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if still_seeking {
        return Err("thumbnail not ready".to_string());
    }

    let pixels = renderer.render_thumbnail(320, 180)?;
    drop(renderer_guard);
    drop(loaded_url_guard);

    let img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(320, 180, pixels)
        .ok_or_else(|| "frame buffer mismatch".to_string())?;
    let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
    let mut jpeg: Vec<u8> = Vec::new();
    rgb.write_to(
        &mut std::io::Cursor::new(&mut jpeg),
        image::ImageFormat::Jpeg,
    )
    .map_err(|e| e.to_string())?;

    Ok(format!(
        "data:image/jpeg;base64,{}",
        general_purpose::STANDARD.encode(&jpeg)
    ))
}

