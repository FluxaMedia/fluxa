use super::*;

#[tauri::command]
pub async fn player_init(app: AppHandle, state: State<'_, DesktopState>) -> Result<(), String> {
    log::info!("player_init: start");
    state.pending_hide.store(false, Ordering::Release);

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        let app_clone = app.clone();
        let native_ready = tauri::async_runtime::spawn_blocking(move || {
            let state = app_clone.state::<DesktopState>();
            ensure_native_player_surface(&app_clone, &state).is_some()
        })
        .await
        .map_err(|e| e.to_string())?;
        if native_ready {
            log::info!("player_init: native surface ready");
        } else {
            log::warn!(
                "player_init: native player surface unavailable, using software video rendering"
            );
        }
    }

    if playback_engine::read_player_engine(&app) != PlayerEngine::AvPlayer {
        let app_for_headless = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let state = app_for_headless.state::<DesktopState>();
            if state.player_mpv_client.lock().unwrap().is_none() {
                match mpv_render::MpvClientHandle::new_with_scripts(
                    crate::player::mpv_script_paths(&app_for_headless),
                ) {
                    Ok((client, render)) => {
                        *state.player_render_state.lock().unwrap() = Some(render);
                        *state.player_mpv_client.lock().unwrap() = Some(client);
                    }
                    Err(error) => {
                        log::error!("player_init: MpvClientHandle::new failed: {error}");
                        crate::diagnostics::report(
                            &app_for_headless,
                            format!("MpvClientHandle::new failed: {error}"),
                            sentry::Level::Error,
                        );
                        return Err(error);
                    }
                }
            }
            Ok(())
        })
        .await
        .map_err(|e| e.to_string())??;
    }

    log::info!("player_init: ok");
    start_telemetry_publisher(app, &state);
    Ok(())
}

fn start_telemetry_publisher(app: AppHandle, state: &DesktopState) {
    if state.player_telemetry_running.swap(true, Ordering::AcqRel) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let mut last_static = None;
        let mut last_position_sent = std::time::Instant::now() - Duration::from_secs(1);
        loop {
            let state = app.state::<DesktopState>();
            if state.pending_hide.load(Ordering::Acquire) {
                state
                    .player_telemetry_running
                    .store(false, Ordering::Release);
                break;
            }
            let position_due = last_position_sent.elapsed()
                >= Duration::from_millis(state.player_position_interval_ms.load(Ordering::Acquire));
            let active_engine = *state.active_player_engine.lock().unwrap();
            let stats_enabled = state.player_stats_enabled.load(Ordering::Acquire);
            let (static_status, position_status) = match active_engine {
                PlayerEngine::Mpv => {
                    let renderer = state.player_mpv_client.lock().unwrap();
                    renderer
                        .as_ref()
                        .map(|renderer| {
                            let static_status = Some(renderer.cached_static_status());
                            let position_status = position_due.then(|| {
                                if stats_enabled {
                                    renderer.stats_position_status()
                                } else {
                                    renderer.fast_position_status()
                                }
                            });
                            (static_status, position_status)
                        })
                        .unwrap_or((None, None))
                }
                PlayerEngine::Vlc => state
                    .player_renderer_vlc
                    .try_lock()
                    .ok()
                    .and_then(|renderer| {
                        renderer.as_ref().map(|renderer| {
                            let status = renderer.status();
                            (
                                Some(status.static_status()),
                                position_due.then(|| status.position_status()),
                            )
                        })
                    })
                    .unwrap_or((None, None)),
                PlayerEngine::AvPlayer => state
                    .native_player_surface
                    .lock()
                    .unwrap()
                    .as_ref()
                    .and_then(|surface| surface.status().ok())
                    .map(|status| {
                        (
                            Some(status.static_status()),
                            position_due.then(|| status.position_status()),
                        )
                    })
                    .unwrap_or((None, None)),
            };
            if let Some(static_status) = static_status {
                if last_static.as_ref() != Some(&static_status) {
                    let _ = app.emit("player-static-status", static_status.clone());
                    last_static = Some(static_status);
                }
                if let Some(position_status) = position_status {
                    let _ = app.emit("player-position-status", position_status);
                    last_position_sent = std::time::Instant::now();
                }
            } else if let Some(position_status) = position_status {
                let _ = app.emit("player-position-status", position_status);
                last_position_sent = std::time::Instant::now();
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    });
}

#[tauri::command]
pub async fn player_load(
    app: AppHandle,
    state: State<'_, DesktopState>,
    stream_proxy_state: State<'_, crate::stream_proxy::StreamProxyState>,
    url: String,
    start_at: Option<u64>,
    total_duration: Option<u64>,
) -> Result<(), String> {
    log::info!("player_load: url={url} start_at={start_at:?} total_duration={total_duration:?}");

    let pending_headers = std::mem::take(&mut *state.pending_stream_headers.lock().unwrap());
    let engine = playback_engine::read_player_engine(&app);
    *state.active_player_engine.lock().unwrap() = engine;

    #[cfg(target_os = "macos")]
    let mut avplayer_adapter_applied = false;

    #[cfg(target_os = "macos")]
    if engine == PlayerEngine::AvPlayer {
        let headers = pending_headers.iter().cloned().collect::<std::collections::HashMap<_, _>>();
        let needs_adapter = !headers.is_empty() || avplayer_needs_remux(&url);
        if needs_adapter && (url.starts_with("http://") || url.starts_with("https://")) {
            if let Some(previous_id) = state.avplayer_local_stream_id.lock().unwrap().take() {
                let _ = fluxa_streaming_engine::stop_local_stream_server(&previous_id);
            }
            let headers_json = serde_json::to_string(&headers).unwrap_or_else(|_| "{}".to_string());
            let local = fluxa_streaming_engine::start_local_stream_server(&url, &headers_json, 0)
                .ok_or_else(|| "could not start AVPlayer local stream adapter".to_string())?;
            let payload: serde_json::Value = serde_json::from_str(&local)
                .map_err(|error| format!("invalid AVPlayer local stream response: {error}"))?;
            let id = payload.get("id").and_then(serde_json::Value::as_str)
                .ok_or_else(|| "AVPlayer local stream response has no id".to_string())?;
            let base_url = payload.get("url").and_then(serde_json::Value::as_str)
                .ok_or_else(|| "AVPlayer local stream response has no url".to_string())?;
            *state.avplayer_local_stream_id.lock().unwrap() = Some(id.to_string());
            url = if avplayer_needs_remux(&url) {
                format!("{base_url}/remux")
            } else {
                base_url.to_string()
            };
            avplayer_adapter_applied = true;
            log::info!("player_load: AVPlayer adapter url={url}");
        }
    }

    #[cfg(target_os = "macos")]
    let adapter_applied = avplayer_adapter_applied;
    #[cfg(not(target_os = "macos"))]
    let adapter_applied = false;
    let url = if !adapter_applied && !pending_headers.is_empty()
        && (url.starts_with("http://") || url.starts_with("https://"))
    {
        match crate::stream_proxy::register(&stream_proxy_state, url.clone(), pending_headers).await
        {
            Ok(proxied_url) => proxied_url,
            Err(error) => {
                log::warn!(
                    "player_load: stream_proxy registration failed, playing raw url: {error}"
                );
                url
            }
        }
    } else {
        url
    };

    state.thumbnail.lock().unwrap().url = Some(url.clone());

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    if engine == PlayerEngine::Vlc {
        {
            let mut renderer = state.player_renderer_vlc.lock().unwrap();
            if renderer.is_none() {
                *renderer = Some(libvlc_render::LibvlcPlayer::new()?);
            }
        }
        let surface = ensure_native_player_surface(&app, &state)
            .ok_or_else(|| "native player surface is unavailable for libVLC".to_string())?;
        return surface.load(url, start_at, total_duration);
    }
    if engine == PlayerEngine::Vlc {
        let mut renderer = state.player_renderer_vlc.lock().unwrap();
        if renderer.is_none() {
            *renderer = Some(libvlc_render::LibvlcPlayer::new()?);
        }
        let pending_options = state.pending_player_options.lock().unwrap().clone();
        if let Some(player) = renderer.as_ref() {
            if !pending_options.is_empty() {
                player.apply_options(&pending_options)?;
            }
        }
        return renderer
            .as_mut()
            .ok_or_else(|| "libvlc player is not initialized".to_string())?
            .load(&url, start_at);
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(surface) = ensure_native_player_surface(&app, &state) {
            match surface.load(url.clone(), start_at, total_duration) {
                Ok(()) => return Ok(()),
                Err(error) => {
                    log::warn!(
                        "player_load: Windows native player surface failed, using software video rendering: {error}"
                    );
                    *state.native_player_surface.lock().unwrap() = None;
                }
            }
        }
        log::warn!(
            "player_load: Windows native player surface unavailable, using software video rendering"
        );
        if let Ok(mut renderer) = state.player_render_state.lock() {
            if let Some(renderer) = renderer.as_mut() {
                renderer.reset_render_context();
            }
        }
        let _ = app.emit("native-player-show", ());
        let _ = app.emit(
            "native-player-software-rendering",
            "Windows native player surface is unavailable; using software video rendering",
        );
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(surface) = ensure_native_player_surface(&app, &state) {
            return surface.load(url, start_at, total_duration);
        }
        log::warn!(
            "player_load: macOS native player surface unavailable, using software video rendering"
        );
        if let Ok(mut renderer) = state.player_render_state.lock() {
            if let Some(renderer) = renderer.as_mut() {
                renderer.reset_render_context();
            }
        }
        let _ = app.emit("native-player-show", ());
        let _ = app.emit(
            "native-player-software-rendering",
            "macOS native player surface is unavailable; using software video rendering",
        );
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(surface) = ensure_native_player_surface(&app, &state) {
            return surface.load(url, start_at, total_duration);
        }
        log::warn!(
            "player_load: no native player surface available, using the Linux software renderer"
        );
    }

    if state.player_mpv_client.lock().unwrap().is_none() {
        let (client, render) =
            mpv_render::MpvClientHandle::new_with_scripts(crate::player::mpv_script_paths(&app))?;
        *state.player_render_state.lock().unwrap() = Some(render);
        *state.player_mpv_client.lock().unwrap() = Some(client);
    }
    let mut renderer = state.player_mpv_client.lock().unwrap();
    renderer
        .as_mut()
        .ok_or_else(|| "player renderer is not initialized".to_string())?
        .load(&url, start_at)
}

#[cfg(target_os = "macos")]
fn avplayer_needs_remux(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url).to_ascii_lowercase();
    path.ends_with(".mkv") || path.ends_with(".matroska")
}

#[tauri::command]
pub fn player_set_http_headers(
    state: State<DesktopState>,
    headers: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let header_list = headers.into_iter().collect::<Vec<_>>();
    *state.pending_stream_headers.lock().unwrap() = header_list.clone();
    match with_renderer_retry(&state, 80, |renderer| {
        renderer.set_http_headers(&header_list)
    }) {
        Ok(Some(())) => Ok(()),
        Ok(None) | Err(_) => Ok(()),
    }
}

#[tauri::command]
pub fn player_last_stream_error(
    stream_proxy_state: State<crate::stream_proxy::StreamProxyState>,
) -> Option<String> {
    crate::stream_proxy::take_last_failure(&stream_proxy_state).map(|(_, detail)| detail)
}

#[tauri::command]
pub fn player_apply_preferences(
    app: AppHandle,
    state: State<DesktopState>,
    preferences: serde_json::Value,
) -> Result<(), String> {
    if let Some(v) = preferences.get("useChapterSkip").and_then(|v| v.as_bool()) {
        state.player_overlay.lock().unwrap().use_chapter_skip = v;
    }
    let previous_options = state.pending_player_options.lock().unwrap().clone();
    let (options, anime4k_resolved) = mpv_options_from_preferences(Some(&app), &preferences);
    let audio_options = |items: &[(String, String)]| {
        items
            .iter()
            .filter(|(name, _)| name == "audio-spdif" || name == "audio-channels" || name == "af")
            .cloned()
            .collect::<Vec<_>>()
    };
    let audio_policy_changed = audio_options(&previous_options) != audio_options(&options);
    *state.pending_player_options.lock().unwrap() = options.clone();
    if anime4k_should_apply(&preferences) && !anime4k_resolved {
        log::warn!("Anime4K was requested by preferences but its shader could not be resolved");
    }
    let options_applied = if options.is_empty() {
        true
    } else {
        match with_renderer_retry(&state, 80, |renderer| renderer.apply_options(&options)) {
            Ok(Some(())) => true,
            Ok(None) => {
                log::warn!("Preferences could not be applied: player renderer not ready");
                false
            }
            Err(err) => {
                log::warn!("Preferences could not be applied: {err}");
                false
            }
        }
    };
    if options_applied
        && audio_policy_changed
        && *state.active_player_engine.lock().unwrap() == PlayerEngine::Vlc
    {
        // LibVLC consumes these audio options when the media object is
        // created. Rebuild only for an audio-policy change and preserve the
        // current position/play state; unrelated preference changes stay hot.
        let _ = with_renderer_retry_mut(&state, 80, |renderer| {
            let status = renderer.status();
            let Some(url) = renderer.query_property("path") else {
                return Ok(());
            };
            if !status.loaded {
                return Ok(());
            }
            let position_ms = renderer
                .query_property("time-pos")
                .and_then(|value| value.parse::<f64>().ok())
                .filter(|value| value.is_finite() && *value >= 0.0)
                .map(|value| (value * 1000.0).round() as u64);
            let was_paused = status.pause.as_deref() == Some("yes");
            renderer.load(&url, position_ms)?;
            if was_paused {
                renderer.command_string("set pause yes")?;
            }
            Ok(())
        });
    }
    let anime4k_enabled = anime4k_resolved && options_applied;
    state.player_overlay.lock().unwrap().anime4k_enabled = anime4k_enabled;
    let _ = app.emit(
        "player-anime4k-state",
        serde_json::json!({ "enabled": anime4k_enabled }),
    );
    Ok(())
}
