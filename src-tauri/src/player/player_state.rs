use super::*;

#[tauri::command]
pub fn player_set_status_interval(state: State<DesktopState>, interval_ms: u64) {
    state
        .player_position_interval_ms
        .store(interval_ms.clamp(250, 1000), Ordering::Release);
}

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
                let (eof, error) = match event {
                    mpv_render::PlayerEvent::EndFile { eof, error } => (eof, error),
                    mpv_render::PlayerEvent::PauseChanged(paused) => {
                        let _ = app.emit("native-player-pause-changed", paused);
                        continue;
                    }
                };
                if let Some(message) = error {
                    let _ = app.emit("native-player-error", message);
                } else if eof {
                    let mut overlay = state.player_overlay.lock().unwrap();
                    if !overlay.take_eof_next() {
                        continue;
                    }
                    let next_sub = overlay.next_ep_subtitle.clone();
                    let auto_play = overlay.auto_play_next_episode;
                    drop(overlay);
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
    let overlay = state.player_overlay.lock().unwrap();
    serde_json::json!({
        "skipSegmentsJson": overlay.skip_segments_json.clone(),
        "chaptersJson": overlay.chapters_json.clone(),
        "episodesJson": overlay.episodes_json.clone(),
        "nextEpSubtitle": overlay.next_ep_subtitle.clone(),
        "nextEpThresholdPercent": overlay.next_ep_threshold_percent,
        "autoPlayNextEpisode": overlay.auto_play_next_episode,
        "autoPlayCountdownSecs": overlay.auto_play_countdown_secs,
        "autoSkipSegments": overlay.auto_skip_segments,
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
    state.pending_hide.store(true, Ordering::Release);
    let _ = state.sleep_inhibitor.lock().unwrap().set_enabled(false);
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Vlc {
        return state.player_renderer_vlc.lock().unwrap().take().is_some();
    }
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
        surface.hide();
        return state.player_mpv_client.lock().unwrap().is_some();
    }

    let had_render = state.player_render_state.lock().unwrap().take().is_some();
    let had_client = state.player_mpv_client.lock().unwrap().take().is_some();
    had_render || had_client
}

#[tauri::command]
pub fn player_set_chapters(state: State<DesktopState>, chapters_json: String) {
    state.player_overlay.lock().unwrap().chapters_json =
        if chapters_json.trim().is_empty() || chapters_json == "[]" {
            None
        } else {
            Some(chapters_json)
        };
}

#[tauri::command]
pub fn player_clear_chapters(state: State<DesktopState>) {
    state.player_overlay.lock().unwrap().chapters_json = None;
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
    let mut overlay = state.player_overlay.lock().unwrap();
    overlay.skip_segments_json =
        if segments_json.trim().is_empty() || segments_json == "[]" {
            None
        } else {
            Some(segments_json)
        };
    overlay.next_ep_subtitle = next_ep_subtitle.unwrap_or_default();
    overlay.eof_next_fired = false;
    if let Some(t) = next_ep_threshold_percent {
        overlay.next_ep_threshold_percent = t.clamp(1.0, 99.0);
    }
    if let Some(v) = auto_play_next_episode {
        overlay.auto_play_next_episode = v;
    }
    if let Some(s) = auto_play_countdown_secs {
        overlay.auto_play_countdown_secs = s.max(1);
    }
    if let Some(v) = auto_skip_segments {
        overlay.auto_skip_segments = v;
    }
    let _ = app.emit("player-skip-info-updated", ());
}

#[tauri::command]
pub fn player_clear_skip_info(app: AppHandle, state: State<DesktopState>) {
    let mut overlay = state.player_overlay.lock().unwrap();
    overlay.skip_segments_json = None;
    overlay.next_ep_subtitle = String::new();
    overlay.eof_next_fired = false;
    let _ = app.emit("player-skip-info-updated", ());
}

#[tauri::command]
pub fn player_set_episodes(state: State<DesktopState>, episodes_json: String) {
    state.player_overlay.lock().unwrap().episodes_json =
        if episodes_json.trim() == "[]" || episodes_json.trim().is_empty() {
            None
        } else {
            Some(episodes_json)
        };
}

#[tauri::command]
pub fn player_clear_episodes(state: State<DesktopState>) {
    state.player_overlay.lock().unwrap().episodes_json = None;
}

#[tauri::command]
pub fn player_set_seek_thumbnail_enabled(state: State<DesktopState>, enabled: bool) {
    state.thumbnail.lock().unwrap().enabled = enabled;
}

#[tauri::command]
pub fn player_get_seek_thumbnail(
    state: State<DesktopState>,
    time_pos: f64,
) -> Result<String, String> {
    use base64::{engine::general_purpose, Engine as _};

    let mut thumbnail = state.thumbnail.lock().unwrap();
    if !thumbnail.enabled {
        return Ok(String::new());
    }
    let url = thumbnail.url.clone().ok_or_else(|| "no url".to_string())?;

    if thumbnail.renderer.is_none() {
        thumbnail.renderer = Some(mpv_render::MpvThumbnailRenderer::new()?);
    }
    let reload_thumbnail = thumbnail.loaded_url.as_deref() != Some(url.as_str());
    if reload_thumbnail {
        thumbnail.loaded_url = Some(url.clone());
    }
    let renderer = thumbnail.renderer.as_mut().unwrap();

    if reload_thumbnail {
        renderer.load_thumbnail(&url)?;
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
    drop(thumbnail);

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
