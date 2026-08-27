use crate::DesktopState;
use crate::artwork::{
    artwork_bg_decoded, artwork_logo_decoded, fetch_player_artwork_bytes_owned, normalize_url,
    scale_artwork_cover, scale_artwork_fit,
};
use crate::custom_fonts;
use crate::libvlc_render;
use crate::mpv_render;
use crate::playback_engine::{self, PlaybackEngine, PlayerEngine};
#[cfg(target_os = "macos")]
use crate::render_backend::{RenderBackend, read_render_backend};
use fluxa_core::FluxaCore;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager, State};

#[cfg(target_os = "linux")]
use crate::linux_player_surface;
#[cfg(target_os = "macos")]
use crate::macos_avplayer;
#[cfg(target_os = "macos")]
use crate::macos_player_surface;
#[cfg(target_os = "windows")]
use crate::windows_player_surface;

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn cache_native_player_surface<S, F>(
    state: &DesktopState,
    install: F,
) -> Option<std::sync::Arc<dyn crate::player_surface::PlayerSurface>>
where
    S: crate::player_surface::PlayerSurface + 'static,
    F: FnOnce() -> Result<S, String>,
{
    if let Some(surface) = state.native_player_surface.lock().unwrap().clone() {
        return Some(surface);
    }
    match install() {
        Ok(surface) => {
            let surface: std::sync::Arc<dyn crate::player_surface::PlayerSurface> =
                std::sync::Arc::new(surface);
            *state.native_player_surface.lock().unwrap() = Some(surface.clone());
            Some(surface)
        }
        Err(error) => {
            log::warn!("native player surface was not installed: {error}");
            None
        }
    }
}

pub(crate) fn reset_playback_state(state: &DesktopState) {
    state
        .pending_hide
        .store(false, std::sync::atomic::Ordering::Release);
    let mut overlay = state.player_overlay.lock().unwrap();
    overlay.eof_next_fired = false;
    overlay.chapters_json = None;
}

pub(crate) fn mpv_script_paths(app: &AppHandle) -> Vec<PathBuf> {
    let state = app.state::<DesktopState>();
    let Some(data_dir) = state.data_dir.lock().unwrap().clone() else {
        return Vec::new();
    };
    let scripts_dir = data_dir.join("mpv").join("scripts");
    let _ = std::fs::create_dir_all(&scripts_dir);
    let Ok(entries) = std::fs::read_dir(&scripts_dir) else {
        return Vec::new();
    };
    let mut scripts = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("lua"))
        })
        .collect::<Vec<_>>();
    scripts.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    scripts
}

pub(crate) fn mpv_shader_paths(app: &AppHandle) -> Vec<PathBuf> {
    let state = app.state::<DesktopState>();
    let Some(data_dir) = state.data_dir.lock().unwrap().clone() else {
        return Vec::new();
    };
    let shaders_dir = data_dir.join("mpv").join("shaders");
    let _ = std::fs::create_dir_all(&shaders_dir);
    let Ok(entries) = std::fs::read_dir(&shaders_dir) else {
        return Vec::new();
    };
    let mut shaders = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("glsl"))
        })
        .collect::<Vec<_>>();
    shaders.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    shaders
}

pub(crate) fn load_libvlc_for_surface<F>(
    state: &DesktopState,
    url: &str,
    start_at: Option<u64>,
    attach: F,
) -> Result<(), String>
where
    F: FnOnce(&mut libvlc_render::LibvlcPlayer) -> Result<(), String>,
{
    let mut players = state.player_renderer_vlc.lock().unwrap();
    if players.is_none() {
        *players = Some(libvlc_render::LibvlcPlayer::new()?);
    }
    let player = players
        .as_mut()
        .ok_or_else(|| "libVLC player is unavailable".to_string())?;
    let pending_options = state.pending_player_options.lock().unwrap().clone();
    if !pending_options.is_empty() {
        player.apply_options(&pending_options)?;
    }
    attach(player)?;
    player.load(url, start_at)
}

pub(crate) fn load_mpv_engine(
    client: &mut mpv_render::MpvClientHandle,
    url: &str,
    start_at: Option<u64>,
    hdr: bool,
) -> Result<(), String> {
    if hdr {
        let _ = client.set_option("target-trc", "linear");
        let _ = client.set_option("target-prim", "bt.709");
    }
    client.load(url, start_at)
}

#[cfg(target_os = "linux")]
pub fn ensure_native_player_surface(
    app_handle: &AppHandle,
    state: &DesktopState,
) -> Option<std::sync::Arc<dyn crate::player_surface::PlayerSurface>> {
    cache_native_player_surface(state, || linux_player_surface::install(app_handle.clone()))
}

#[cfg(target_os = "windows")]
pub fn ensure_native_player_surface(
    app_handle: &AppHandle,
    state: &DesktopState,
) -> Option<std::sync::Arc<dyn crate::player_surface::PlayerSurface>> {
    cache_native_player_surface(state, || {
        windows_player_surface::install(app_handle.clone())
    })
}

#[cfg(target_os = "macos")]
pub fn ensure_native_player_surface(
    app_handle: &AppHandle,
    state: &DesktopState,
) -> Option<std::sync::Arc<dyn crate::player_surface::PlayerSurface>> {
    if playback_engine::read_player_engine(app_handle) == PlayerEngine::AvPlayer {
        if let Some(surface) = state.native_player_surface.lock().unwrap().clone() {
            if surface.backend_name() == "avplayer" {
                return Some(surface);
            }
            let old = state.native_player_surface.lock().unwrap().take();
            if let Some(old) = old {
                let _ = old.shutdown();
            }
        }
        return match macos_avplayer::install(app_handle.clone()) {
            Ok(surface) => {
                let surface: std::sync::Arc<dyn crate::player_surface::PlayerSurface> =
                    std::sync::Arc::new(surface);
                *state.native_player_surface.lock().unwrap() = Some(surface.clone());
                Some(surface)
            }
            Err(error) => {
                log::warn!("FluxaPlayerKit AVPlayer surface was not installed: {error}");
                None
            }
        };
    }
    let requested = read_render_backend(app_handle);
    if let Some(surface) = state.native_player_surface.lock().unwrap().clone() {
        if surface.backend_name() == requested.name() {
            return Some(surface);
        }
        log::info!(
            "macOS render backend changed from {} to {}; rebuilding native surface",
            surface.backend_name(),
            requested.name()
        );
        let old = state.native_player_surface.lock().unwrap().take();
        if let Some(old) = old {
            if let Err(error) = old.shutdown() {
                log::warn!("failed to stop previous macOS native surface: {error}");
            }
        }
    }
    let installed = macos_player_surface::install(app_handle.clone());
    match installed {
        Ok(surface) => {
            let surface: std::sync::Arc<dyn crate::player_surface::PlayerSurface> =
                std::sync::Arc::new(surface);
            *state.native_player_surface.lock().unwrap() = Some(surface.clone());
            Some(surface)
        }
        Err(error) => {
            log::warn!("native Vulkan player surface was not installed: {error}");
            None
        }
    }
}

mod controls;
mod lifecycle;
mod player_preferences;
mod presentation;

pub use controls::{
    player_add_subtitle, player_command, player_get_anime4k_enabled, player_render_frame,
    player_set_anime4k_enabled, player_set_sleep_inhibition, player_torrent_sibling_subtitles,
};
pub use lifecycle::{
    player_apply_preferences, player_init, player_last_stream_error, player_load,
    player_set_http_headers,
};
use player_preferences::*;
pub use presentation::{
    player_prefetch_artwork, player_set_cursor_visible, player_set_loading_artwork,
    player_set_title,
};

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

fn torrent_sibling_subtitles(state: &DesktopState) -> Vec<(String, String, Option<String>)> {
    let torrent = state.torrent.lock().unwrap();
    let base_url = torrent.server_base_url.clone();
    let link = torrent.stream_link.clone();
    let selected_id = torrent.stream_file_id;
    drop(torrent);
    let (Some(base_url), Some(link), Some(selected_id)) = (base_url, link, selected_id) else {
        return Vec::new();
    };
    let status_url = format!("{}/torrents", base_url.trim_end_matches('/'));
    let body =
        serde_json::json!({ "action": "get", "link": link, "file_id": selected_id }).to_string();
    let Some(response) = crate::torrent_transport::request(
        &status_url,
        Some(&body),
        std::time::Duration::from_secs(5),
    ) else {
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
    let request = serde_json::json!({ "selectedPath": selected_path, "files": files }).to_string();
    let Some(matches_json) = FluxaCore::torrent_sibling_subtitle_matches_json(&request) else {
        return Vec::new();
    };
    let Ok(matches) = serde_json::from_str::<Vec<serde_json::Value>>(&matches_json) else {
        return Vec::new();
    };

    let mut subtitles = Vec::new();
    for entry in matches {
        let Some(id) = entry.get("id").and_then(serde_json::Value::as_u64) else {
            continue;
        };
        let Some(path) = entry.get("path").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let url = format!(
            "{}/stream/fname?link={}&index={}&role=subtitle&play&title={}",
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
        let language = entry
            .get("language")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
        subtitles.push((url, title, language));
    }
    log::warn!(
        "torrent subtitles: selected={selected_path:?}, sibling_count={}",
        subtitles.len()
    );
    subtitles
}
pub(crate) fn with_renderer_retry<T, F>(
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
                if let Ok(guard) = state.player_mpv_client.try_lock() {
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
            PlayerEngine::AvPlayer => {
                return Err("AVPlayer uses its native surface directly".to_string());
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("player renderer busy".to_string())
}

pub(crate) fn with_renderer_retry_mut<T, F>(
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
                if let Ok(mut guard) = state.player_mpv_client.try_lock() {
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
            PlayerEngine::AvPlayer => {
                return Err("AVPlayer uses its native surface directly".to_string());
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err("player renderer busy".to_string())
}

mod player_state;
mod subtitles;

pub use player_state::{
    player_clear_chapters, player_clear_episodes, player_clear_skip_info, player_destroy,
    player_get_playback_info, player_get_seek_thumbnail, player_hide, player_screenshot,
    player_set_chapters, player_set_episodes, player_set_seek_thumbnail_enabled,
    player_set_skip_info, player_set_stats_enabled, player_set_status_interval,
    player_show_loading, player_status, player_title, player_track_options,
};
pub(crate) use subtitles::{player_auto_sync_subtitles, player_capture_subtitle_cues};
