use crate::artwork::{
    artwork_bg_decoded, artwork_logo_decoded, fetch_player_artwork_bytes_owned, normalize_url,
    scale_artwork_cover, scale_artwork_fit,
};
use crate::custom_fonts;
use crate::libvlc_render;
use crate::mpv_render;
use crate::playback_engine::{self, PlaybackEngine, PlayerEngine};
use crate::DesktopState;
use fluxa_core::FluxaCore;
use serde_json::{json, Value};
use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager, State};

#[cfg(target_os = "linux")]
use crate::linux_player_surface;
#[cfg(target_os = "macos")]
use crate::macos_player_surface;
#[cfg(target_os = "windows")]
use crate::windows_player_surface;

#[cfg(target_os = "linux")]
pub fn ensure_native_player_surface(
    app_handle: &AppHandle,
    state: &DesktopState,
) -> Option<linux_player_surface::NativePlayerSurface> {
    if let Some(surface) = state.native_player_surface.lock().unwrap().clone() {
        return Some(surface);
    }
    match linux_player_surface::install(app_handle.clone()) {
        Ok(surface) => {
            *state.native_player_surface.lock().unwrap() = Some(surface.clone());
            Some(surface)
        }
        Err(error) => {
            log::warn!("native OpenGL player surface was not installed: {error}");
            None
        }
    }
}

#[cfg(target_os = "windows")]
pub fn ensure_native_player_surface(
    app_handle: &AppHandle,
    state: &DesktopState,
) -> Option<windows_player_surface::NativePlayerSurface> {
    if let Some(surface) = state.native_player_surface.lock().unwrap().clone() {
        return Some(surface);
    }
    match windows_player_surface::install(app_handle.clone()) {
        Ok(surface) => {
            *state.native_player_surface.lock().unwrap() = Some(surface.clone());
            Some(surface)
        }
        Err(error) => {
            log::warn!("native OpenGL player surface was not installed: {error}");
            None
        }
    }
}

#[cfg(target_os = "macos")]
pub fn ensure_native_player_surface(
    app_handle: &AppHandle,
    state: &DesktopState,
) -> Option<macos_player_surface::NativePlayerSurface> {
    if let Some(surface) = state.native_player_surface.lock().unwrap().clone() {
        return Some(surface);
    }
    match macos_player_surface::install(app_handle.clone()) {
        Ok(surface) => {
            *state.native_player_surface.lock().unwrap() = Some(surface.clone());
            Some(surface)
        }
        Err(error) => {
            log::warn!("native OpenGL player surface was not installed: {error}");
            None
        }
    }
}

mod player_preferences;

use player_preferences::*;

fn encode_query_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            use std::fmt::Write as _;
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

const VIDEO_EXTENSIONS: [&str; 9] = [
    ".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v", ".ts",
];

/// Pulls a "s01e02"-style tag out of a lowercased filename, if present.
/// Season-pack subtitle folders/files usually keep this tag even when the
/// rest of the naming (release group, resolution) differs from the video.
fn extract_episode_tag(lower_name: &str) -> Option<String> {
    let bytes = lower_name.as_bytes();
    for start in 0..bytes.len() {
        if bytes[start] != b's' {
            continue;
        }
        let mut i = start + 1;
        let season_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() && i - season_start < 2 {
            i += 1;
        }
        if i == season_start || i >= bytes.len() || bytes[i] != b'e' {
            continue;
        }
        i += 1;
        let episode_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() && i - episode_start < 3 {
            i += 1;
        }
        if i == episode_start {
            continue;
        }
        return Some(lower_name[start..i].to_string());
    }
    None
}

fn torrent_sibling_subtitles(state: &DesktopState) -> Vec<(String, String, Option<String>)> {
    let base_url = state.torrent_server_base_url.lock().unwrap().clone();
    let link = state.torrent_stream_link.lock().unwrap().clone();
    let selected_id = *state.torrent_stream_file_id.lock().unwrap();
    let (Some(base_url), Some(link), Some(selected_id)) = (base_url, link, selected_id) else {
        return Vec::new();
    };
    let status_url = format!("{}/torrents", base_url.trim_end_matches('/'));
    let body =
        serde_json::json!({ "action": "get", "link": link, "file_id": selected_id }).to_string();
    let Some(response) =
        crate::torrent_engine_request(&status_url, Some(&body), std::time::Duration::from_secs(5))
    else {
        log::warn!("torrent subtitles: status request failed");
        return Vec::new();
    };
    let Some((_, response_body)) = response.split_once("\r\n\r\n") else {
        return Vec::new();
    };
    let Ok(status) = serde_json::from_str::<serde_json::Value>(response_body) else {
        log::warn!("torrent subtitles: invalid status response");
        return Vec::new();
    };
    let files = status
        .get("file_stats")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let selected_path = files.iter().find_map(|file| {
        (file.get("id").and_then(serde_json::Value::as_u64) == Some(selected_id as u64))
            .then(|| file.get("path").and_then(serde_json::Value::as_str))
            .flatten()
            .map(str::to_string)
    });
    let Some(selected_path) = selected_path else {
        return Vec::new();
    };
    let selected_name = std::path::Path::new(&selected_path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if selected_name.is_empty() {
        return Vec::new();
    }
    let episode_tag = extract_episode_tag(&selected_name);
    // Single-video torrents (a lone episode/movie release) have no other
    // video file to disambiguate against, so any subtitle file in the
    // torrent must belong to it regardless of naming.
    let is_single_video_torrent = files
        .iter()
        .filter(|file| {
            file.get("path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| {
                    let lower = path.to_ascii_lowercase();
                    VIDEO_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
                })
        })
        .count()
        <= 1;

    let mut subtitles = Vec::new();
    let mut total_subtitle_files = 0usize;
    for file in files {
        let Some(id) = file.get("id").and_then(serde_json::Value::as_u64) else {
            continue;
        };
        let Some(path) = file.get("path").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let lower = path.to_ascii_lowercase();
        let is_subtitle = [".srt", ".ass", ".ssa", ".vtt", ".sub"]
            .iter()
            .any(|extension| lower.ends_with(extension));
        if !is_subtitle {
            continue;
        }
        total_subtitle_files += 1;
        let matches_name = lower.contains(&selected_name);
        let matches_episode_tag = episode_tag
            .as_deref()
            .is_some_and(|tag| lower.contains(tag));
        if !matches_name && !matches_episode_tag && !is_single_video_torrent {
            log::warn!("torrent subtitles: skipped unmatched path={path:?}");
            continue;
        }
        let url = format!(
            "{}/stream/fname?link={}&index={}&play&title={}",
            base_url.trim_end_matches('/'),
            encode_query_component(&link),
            id,
            encode_query_component(path),
        );
        let title = std::path::Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Torrent subtitle")
            .to_string();
        let language = lower
            .split('.')
            .rev()
            .nth(1)
            .filter(|part| part.len() == 3)
            .map(str::to_string);
        subtitles.push((url, title, language));
    }
    log::warn!(
        "torrent subtitles: selected={selected_path:?}, episode_tag={episode_tag:?}, single_video={is_single_video_torrent}, total_subtitle_files={total_subtitle_files}, sibling_count={}",
        subtitles.len()
    );
    subtitles
}
fn with_renderer_retry<T, F>(
    state: &DesktopState,
    attempts: usize,
    f: F,
) -> Result<Option<T>, String>
where
    F: Fn(&dyn PlaybackEngine) -> Result<T, String>,
{
    let engine = *state.active_player_engine.lock().unwrap();
    for _ in 0..attempts {
        match engine {
            PlayerEngine::Mpv => {
                if let Ok(guard) = state.player_renderer.try_lock() {
                    if let Some(renderer) = guard.as_ref() {
                        return f(renderer).map(Some);
                    }
                    return Ok(None);
                }
            }
            PlayerEngine::Vlc => {
                if let Ok(guard) = state.player_renderer_vlc.try_lock() {
                    if let Some(renderer) = guard.as_ref() {
                        return f(renderer).map(Some);
                    }
                    return Ok(None);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("player renderer busy".to_string())
}

fn with_renderer_retry_mut<T, F>(
    state: &DesktopState,
    attempts: usize,
    mut f: F,
) -> Result<Option<T>, String>
where
    F: FnMut(&mut dyn PlaybackEngine) -> Result<T, String>,
{
    let engine = *state.active_player_engine.lock().unwrap();
    for _ in 0..attempts {
        match engine {
            PlayerEngine::Mpv => {
                if let Ok(mut guard) = state.player_renderer.try_lock() {
                    if let Some(renderer) = guard.as_mut() {
                        return f(renderer).map(Some);
                    }
                    return Ok(None);
                }
            }
            PlayerEngine::Vlc => {
                if let Ok(mut guard) = state.player_renderer_vlc.try_lock() {
                    if let Some(renderer) = guard.as_mut() {
                        return f(renderer).map(Some);
                    }
                    return Ok(None);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("player renderer busy".to_string())
}

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

    let app_for_headless = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_headless.state::<DesktopState>();
        let mut renderer = state.player_renderer.lock().unwrap();
        if renderer.is_none() {
            match mpv_render::MpvRenderer::new() {
                Ok(r) => *renderer = Some(r),
                Err(error) => {
                    log::error!("player_init: MpvRenderer::new failed: {error}");
                    sentry::capture_message(
                        &format!("MpvRenderer::new failed: {error}"),
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

    log::info!("player_init: ok");
    Ok(())
}

#[tauri::command]
pub fn player_load(
    app: AppHandle,
    state: State<DesktopState>,
    url: String,
    start_at: Option<u64>,
    total_duration: Option<u64>,
) -> Result<(), String> {
    log::info!("player_load: url={url} start_at={start_at:?} total_duration={total_duration:?}");
    *state.thumb_url.lock().unwrap() = Some(url.clone());

    let engine = playback_engine::read_player_engine(&app);
    *state.active_player_engine.lock().unwrap() = engine;
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
                    log::warn!("player_load: Windows native player surface failed, using software video rendering: {error}");
                    *state.native_player_surface.lock().unwrap() = None;
                }
            }
        }
        log::warn!(
            "player_load: Windows native player surface unavailable, using software video rendering"
        );
        if let Ok(mut renderer) = state.player_renderer.lock() {
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

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        if let Some(surface) = ensure_native_player_surface(&app, &state) {
            return surface.load(url, start_at, total_duration);
        }
        log::warn!(
            "player_load: no native player surface available, falling back to headless renderer"
        );
    }

    let mut renderer = state.player_renderer.lock().unwrap();
    if renderer.is_none() {
        *renderer = Some(mpv_render::MpvRenderer::new()?);
    }
    renderer
        .as_mut()
        .ok_or_else(|| "player renderer is not initialized".to_string())?
        .load(&url, start_at)
}

#[tauri::command]
pub fn player_set_http_headers(
    state: State<DesktopState>,
    headers: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let header_list = headers.into_iter().collect::<Vec<_>>();
    match with_renderer_retry(&state, 80, |renderer| {
        renderer.set_http_headers(&header_list)
    }) {
        Ok(Some(())) => Ok(()),
        Ok(None) | Err(_) => Ok(()),
    }
}

#[tauri::command]
pub fn player_apply_preferences(
    app: AppHandle,
    state: State<DesktopState>,
    preferences: serde_json::Value,
) -> Result<(), String> {
    if let Some(v) = preferences.get("useChapterSkip").and_then(|v| v.as_bool()) {
        *state.use_chapter_skip.lock().unwrap() = v;
    }
    let (options, anime4k_resolved) = mpv_options_from_preferences(Some(&app), &preferences);
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
    let anime4k_enabled = anime4k_resolved && options_applied;
    *state.anime4k_enabled.lock().unwrap() = anime4k_enabled;
    let _ = app.emit(
        "player-anime4k-state",
        serde_json::json!({ "enabled": anime4k_enabled }),
    );
    Ok(())
}

#[tauri::command]
pub fn player_set_cursor_visible(state: State<DesktopState>, visible: bool) {
    #[cfg(target_os = "windows")]
    if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
        surface.set_cursor_visible(visible);
    }
    #[cfg(not(target_os = "windows"))]
    let _ = (state, visible);
}

#[tauri::command]
pub fn player_set_title(
    app: AppHandle,
    state: State<DesktopState>,
    title: String,
    episode_title: Option<String>,
) {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
        surface.set_title(title.clone(), episode_title.clone());
    }
    let _ = app.emit(
        "native-player-title",
        serde_json::json!({ "title": title, "episodeTitle": episode_title }),
    );
}

#[tauri::command]
pub async fn player_set_loading_artwork(
    state: State<'_, DesktopState>,
    title: String,
    episode_title: Option<String>,
    background_url: Option<String>,
    logo_url: Option<String>,
) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    let (background_scaled, logo_scaled) = {
        let bg_cached = background_url.as_deref().and_then(|u| {
            artwork_bg_decoded()
                .lock()
                .ok()?
                .get(&normalize_url(u))
                .cloned()
        });
        let logo_cached = logo_url.as_deref().and_then(|u| {
            artwork_logo_decoded()
                .lock()
                .ok()?
                .get(&normalize_url(u))
                .cloned()
        });

        let bg_ready = bg_cached.is_some() || background_url.is_none();
        let logo_ready = logo_cached.is_some() || logo_url.is_none();
        if bg_ready && logo_ready {
            (bg_cached, logo_cached)
        } else {
            let bg_fetch = if bg_cached.is_none() {
                background_url.clone()
            } else {
                None
            };
            let logo_fetch = if logo_cached.is_none() {
                logo_url.clone()
            } else {
                None
            };
            let bg_handle = tauri::async_runtime::spawn(fetch_player_artwork_bytes_owned(bg_fetch));
            let logo_handle =
                tauri::async_runtime::spawn(fetch_player_artwork_bytes_owned(logo_fetch));
            let background_bytes = bg_handle.await.unwrap_or(None);
            let logo_bytes = logo_handle.await.unwrap_or(None);

            let (bg_decoded, logo_decoded) = tauri::async_runtime::spawn_blocking(move || {
                let bg = background_bytes.and_then(|b| scale_artwork_cover(b, 1280, 720));
                let logo = logo_bytes.and_then(|b| scale_artwork_fit(b, 500, 170));
                (bg, logo)
            })
            .await
            .unwrap_or((None, None));
            (bg_cached.or(bg_decoded), logo_cached.or(logo_decoded))
        }
    };

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        let mut surface = state.native_player_surface.lock().unwrap().clone();
        if surface.is_none() {
            for _ in 0..6 {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                surface = state.native_player_surface.lock().unwrap().clone();
                if surface.is_some() {
                    break;
                }
            }
        }
        if let Some(surface) = surface {
            surface.set_artwork(title, episode_title, background_scaled, logo_scaled);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn player_prefetch_artwork(background_url: Option<String>, logo_url: Option<String>) {
    let bg_key = background_url.as_deref().map(normalize_url);
    let logo_key = logo_url.as_deref().map(normalize_url);
    let bg = tauri::async_runtime::spawn(fetch_player_artwork_bytes_owned(background_url));
    let logo = tauri::async_runtime::spawn(fetch_player_artwork_bytes_owned(logo_url));
    let bg_bytes = bg.await.unwrap_or(None);
    let logo_bytes = logo.await.unwrap_or(None);

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    if bg_bytes.is_some() || logo_bytes.is_some() {
        let _ = tauri::async_runtime::spawn_blocking(move || {
            if let (Some(key), Some(bytes)) = (bg_key, bg_bytes) {
                if let Some(decoded) = scale_artwork_cover(bytes, 1280, 720) {
                    if let Ok(mut cache) = artwork_bg_decoded().lock() {
                        cache.insert(key, decoded);
                    }
                }
            }
            if let (Some(key), Some(bytes)) = (logo_key, logo_bytes) {
                if let Some(decoded) = scale_artwork_fit(bytes, 500, 170) {
                    if let Ok(mut cache) = artwork_logo_decoded().lock() {
                        cache.insert(key, decoded);
                    }
                }
            }
        })
        .await;
    }
}

#[tauri::command]
pub fn player_torrent_sibling_subtitles(state: State<DesktopState>) -> Vec<Value> {
    torrent_sibling_subtitles(&state)
        .into_iter()
        .map(|(url, title, language)| {
            json!({ "url": url, "title": title, "language": language })
        })
        .collect()
}

#[tauri::command]
pub fn player_add_subtitle(
    state: State<DesktopState>,
    url: String,
    title: Option<String>,
    language: Option<String>,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
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
    let mut renderer = state.player_renderer.lock().unwrap();
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
    let commands: Vec<Vec<String>> = if enabled {
        let chain_path = resolve_anime4k_chain(
            Some(&app),
            quality.as_deref().unwrap_or("anime4k_m"),
            mode.as_deref().unwrap_or("a"),
        )
        .ok_or_else(|| {
            log::error!("player_set_anime4k_enabled: shader chain not found for quality={:?} mode={:?}", quality, mode);
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
    #[cfg(target_os = "windows")]
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Mpv {
        if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
            surface.command_args(commands)?;
            *state.anime4k_enabled.lock().unwrap() = enabled;
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
                log::error!("player_set_anime4k_enabled: command {:?} failed: {err}", args);
                return Err(err);
            }
        }
    }
    *state.anime4k_enabled.lock().unwrap() = enabled;
    let _ = app.emit(
        "player-anime4k-state",
        serde_json::json!({ "enabled": enabled }),
    );
    Ok(())
}

#[tauri::command]
pub fn player_get_anime4k_enabled(state: State<DesktopState>) -> bool {
    *state.anime4k_enabled.lock().unwrap()
}

#[tauri::command]
pub fn player_command(
    _app: AppHandle,
    state: State<DesktopState>,
    command: String,
) -> Result<(), String> {
    if command == "stop" {
        *state.eof_next_fired.lock().unwrap() = true;
    }
    #[cfg(target_os = "windows")]
    if *state.active_player_engine.lock().unwrap() == PlayerEngine::Mpv {
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

mod subtitles;

pub(crate) use subtitles::{player_auto_sync_subtitles, player_capture_subtitle_cues};
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
    for _ in 0..40 {
        if renderer.query_property("seeking").as_deref() != Some("yes") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
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
