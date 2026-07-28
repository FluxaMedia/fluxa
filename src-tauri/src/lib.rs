use fluxa_core::FluxaCore;
mod airplay;
mod artwork;
mod cast;
mod cast_proxy;
mod chromecast;
mod custom_fonts;
mod discord_presence;
mod downloads;
mod libvlc_render;
#[cfg(target_os = "linux")]
mod linux_player_surface;
#[cfg(target_os = "linux")]
mod linux_vulkan;
#[cfg(target_os = "linux")]
mod linux_wayland_subsurface;
#[cfg(target_os = "macos")]
mod macos_player_surface;
#[cfg(target_os = "macos")]
mod macos_vulkan;
mod mpv_render;
mod net_guard;
mod oauth;
mod playback_engine;
mod player;
mod plugin_runtime;
mod poster_cache;
mod roku;
mod sleep_inhibitor;
mod storage;
mod trailer_proxy;
#[cfg(target_os = "windows")]
mod windows_d3d11;
#[cfg(target_os = "windows")]
mod windows_egl;
#[cfg(target_os = "windows")]
mod windows_player_surface;
#[cfg(target_os = "windows")]
mod windows_vulkan;

use airplay::*;
use cast::*;
use cast_proxy::*;
use chromecast::*;
use custom_fonts::*;
use discord_presence::*;
use downloads::*;
use oauth::*;
use player::*;
use poster_cache::*;
use roku::*;
use storage::*;

use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;

#[derive(serde::Serialize)]
struct HttpTextResponse {
    status_code: u16,
    body: String,
}

pub struct DesktopState {
    pub engine_handle: Mutex<Option<u64>>,
    pub data_dir: Mutex<Option<PathBuf>>,
    /// Serializes storage migration and key creation. SQLite coordinates processes,
    /// but the legacy encryption key is a separate file.
    pub storage_lock: Mutex<()>,
    pub download_dir: Mutex<Option<PathBuf>>,
    pub player_renderer: Mutex<Option<mpv_render::MpvRenderer>>,
    pub player_renderer_vlc: Mutex<Option<libvlc_render::LibvlcPlayer>>,
    pub active_player_engine: Mutex<playback_engine::PlayerEngine>,
    #[cfg(target_os = "linux")]
    pub native_player_surface: Mutex<Option<linux_player_surface::NativePlayerSurface>>,
    #[cfg(target_os = "windows")]
    pub native_player_surface: Mutex<Option<windows_player_surface::NativePlayerSurface>>,
    #[cfg(target_os = "macos")]
    pub native_player_surface: Mutex<Option<macos_player_surface::NativePlayerSurface>>,
    pub chapters_json: Mutex<Option<String>>,
    pub skip_segments_json: Mutex<Option<String>>,
    pub next_ep_subtitle: Mutex<String>,
    pub next_ep_threshold_percent: Mutex<f64>,
    pub auto_play_next_episode: Mutex<bool>,
    pub auto_play_countdown_secs: Mutex<u32>,
    pub auto_skip_segments: Mutex<bool>,
    pub use_chapter_skip: Mutex<bool>,
    pub eof_next_fired: Mutex<bool>,
    pub episodes_json: Mutex<Option<String>>,
    pub thumb_url: Mutex<Option<String>>,
    pub seek_thumbnail_enabled: Mutex<bool>,
    pub thumbnail_renderer: Mutex<Option<mpv_render::MpvRenderer>>,
    pub thumbnail_loaded_url: Mutex<Option<String>>,
    pub anime4k_enabled: Mutex<bool>,
    pub pending_hide: AtomicBool,
    #[cfg(target_os = "windows")]
    pub main_window_size: std::sync::atomic::AtomicU64,
    pub downloads: downloads::DownloadsState,
    pub torrent_server_base_url: Mutex<Option<String>>,
    pub torrent_stream_link: Mutex<Option<String>>,
    pub torrent_stream_file_id: Mutex<Option<usize>>,
    pub torrent_generation: Mutex<Option<u64>>,
    pub close_flush_done: AtomicBool,
    pub sleep_inhibitor: Mutex<sleep_inhibitor::SleepInhibitor>,
}

#[derive(Clone, serde::Serialize)]
struct OAuthCodePayload {
    code: String,
    state: Option<String>,
}

struct PendingOAuthCallbacks {
    state: Mutex<OAuthCallbackState>,
}

struct OAuthCallbackState {
    callbacks: HashMap<String, OAuthCodePayload>,
    consumed: HashSet<String>,
}

impl Default for PendingOAuthCallbacks {
    fn default() -> Self {
        Self {
            state: Mutex::new(OAuthCallbackState {
                callbacks: HashMap::new(),
                consumed: HashSet::new(),
            }),
        }
    }
}

fn queue_oauth_callback(
    app: &tauri::AppHandle,
    service: &str,
    code: String,
    state: Option<String>,
) {
    let event = match service {
        "trakt" => "trakt-oauth-code",
        "anilist" => "anilist-oauth-code",
        "simkl" => "simkl-oauth-code",
        _ => return,
    };
    let payload = OAuthCodePayload { code, state };
    let callback_id = format!("{service}:{}", payload.code);
    if let Ok(mut state) = app.state::<PendingOAuthCallbacks>().state.lock() {
        if state.consumed.contains(&callback_id) {
            return;
        }
        state.callbacks.insert(service.to_string(), payload.clone());
    }
    let _ = app.emit(event, payload);
}

#[tauri::command]
fn take_oauth_callback(
    service: String,
    callbacks: State<PendingOAuthCallbacks>,
) -> Result<Option<OAuthCodePayload>, String> {
    match service.as_str() {
        "trakt" | "anilist" | "simkl" => {
            let mut state = callbacks
                .state
                .lock()
                .map_err(|_| "OAuth callback state is unavailable".to_string())?;
            let payload = state.callbacks.remove(&service);
            if let Some(payload) = &payload {
                state.consumed.insert(format!("{service}:{}", payload.code));
            }
            Ok(payload)
        }
        _ => Err("unsupported OAuth service".to_string()),
    }
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            engine_handle: Mutex::new(None),
            data_dir: Mutex::new(None),
            storage_lock: Mutex::new(()),
            download_dir: Mutex::new(None),
            player_renderer: Mutex::new(None),
            player_renderer_vlc: Mutex::new(None),
            active_player_engine: Mutex::new(playback_engine::PlayerEngine::Mpv),
            #[cfg(target_os = "linux")]
            native_player_surface: Mutex::new(None),
            #[cfg(target_os = "windows")]
            native_player_surface: Mutex::new(None),
            #[cfg(target_os = "macos")]
            native_player_surface: Mutex::new(None),
            chapters_json: Mutex::new(None),
            skip_segments_json: Mutex::new(None),
            next_ep_subtitle: Mutex::new(String::new()),
            next_ep_threshold_percent: Mutex::new(85.0),
            auto_play_next_episode: Mutex::new(true),
            auto_play_countdown_secs: Mutex::new(7),
            auto_skip_segments: Mutex::new(false),
            use_chapter_skip: Mutex::new(true),
            eof_next_fired: Mutex::new(false),
            episodes_json: Mutex::new(None),
            thumb_url: Mutex::new(None),
            seek_thumbnail_enabled: Mutex::new(false),
            thumbnail_renderer: Mutex::new(None),
            thumbnail_loaded_url: Mutex::new(None),
            anime4k_enabled: Mutex::new(false),
            pending_hide: AtomicBool::new(false),
            #[cfg(target_os = "windows")]
            main_window_size: std::sync::atomic::AtomicU64::new(0),
            downloads: downloads::DownloadsState::default(),
            torrent_server_base_url: Mutex::new(None),
            torrent_stream_link: Mutex::new(None),
            torrent_stream_file_id: Mutex::new(None),
            torrent_generation: Mutex::new(None),
            close_flush_done: AtomicBool::new(false),
            sleep_inhibitor: Mutex::new(sleep_inhibitor::SleepInhibitor::default()),
        }
    }
}

static DIAGNOSTIC_MODE: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

#[tauri::command]
fn debug_log(msg: String) {
    if std::env::var_os("FLUXA_DEBUG_LOGS").is_some() {
        println!("[perf] {msg}");
    }
    if msg.starts_with("subtitles:") {
        log::warn!("[app] {msg}");
    } else { log::debug!("[app] {msg}"); }
}

#[tauri::command]
fn app_close_flush_done(app: tauri::AppHandle, state: State<DesktopState>) {
    if state
        .close_flush_done
        .swap(true, std::sync::atomic::Ordering::SeqCst)
    {
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }
}

#[tauri::command]
fn set_diagnostic_mode(enabled: bool) {
    let was = DIAGNOSTIC_MODE.swap(enabled, std::sync::atomic::Ordering::Relaxed);
    if enabled && !was {
        log::warn!(
            "diagnostic mode enabled (fluxa v{}, {} {})",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    } else if !enabled && was {
        log::warn!("diagnostic mode disabled");
    }
}

#[tauri::command]
fn export_diagnostic_log(app: tauri::AppHandle, destination: String) -> Result<(), String> {
    let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    let newest = fs::read_dir(&log_dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "log"))
        .max_by_key(|entry| {
            entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        })
        .ok_or_else(|| "no log file found".to_string())?;
    fs::copy(newest.path(), &destination).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn engine_init(state: State<DesktopState>, initial_json: String) -> u64 {
    let handle = FluxaCore::create_headless_engine(&initial_json);
    *state.engine_handle.lock().unwrap() = Some(handle);
    handle
}

#[tauri::command]
fn engine_dispatch(state: State<DesktopState>, action_json: String) -> Option<String> {
    let handle = { *state.engine_handle.lock().unwrap() }?;
    FluxaCore::headless_engine_dispatch_json(handle, &action_json)
}

#[tauri::command]
fn engine_complete_effect(state: State<DesktopState>, result_json: String) -> Option<String> {
    let handle = { *state.engine_handle.lock().unwrap() }?;
    FluxaCore::headless_engine_complete_effect_json(handle, &result_json)
}

#[tauri::command]
fn engine_snapshot(state: State<DesktopState>) -> Option<String> {
    let handle = { *state.engine_handle.lock().unwrap() }?;
    FluxaCore::headless_engine_snapshot_json(handle)
}

#[tauri::command]
async fn http_fetch_text(url: String) -> Result<HttpTextResponse, String> {
    let response = net_guard::vetted_client(&url, std::time::Duration::from_secs(10))
        .await?
        .get(&url)
        .header("User-Agent", "Fluxa/1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status_code = response.status().as_u16();
    let body = response.text().await.map_err(|e| e.to_string())?;
    Ok(HttpTextResponse { status_code, body })
}

#[tauri::command]
async fn http_execute_text(
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: Option<Value>,
) -> Result<HttpTextResponse, String> {
    let client = net_guard::vetted_client(&url, std::time::Duration::from_secs(10)).await?;
    let method = reqwest::Method::from_bytes(method.as_bytes()).map_err(|e| e.to_string())?;
    let mut request = client
        .request(method, &url)
        .header("User-Agent", "Fluxa/1.0");
    for (name, value) in headers {
        request = request.header(name, value);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.map_err(|e| e.to_string())?;
    let status_code = response.status().as_u16();
    let body = response.text().await.map_err(|e| e.to_string())?;
    Ok(HttpTextResponse { status_code, body })
}

#[tauri::command]
async fn run_plugin_scraper(
    code: String,
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        plugin_runtime::execute_scraper(code, tmdb_id, media_type, season, episode)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn core_invoke(method: String, args_json: String) -> String {
    tauri::async_runtime::spawn_blocking(move || fluxa_core::ffi::core_invoke(&method, &args_json))
        .await
        .unwrap_or_default()
}

fn start_torrent_stream_inner(
    data_dir: std::path::PathBuf,
    stream_json: String,
    title: Option<String>,
    preferences: Option<Value>,
    existing_base_url: Option<String>,
) -> Result<(String, String, String, Option<u64>, Option<usize>), String> {
    let (base_url, generation) = if let Some(base_url) = existing_base_url {
        apply_torrent_preferences(&base_url, preferences.as_ref());
        (base_url, None)
    } else {
        let cache_dir = data_dir.join("torrent-cache");
        let server_json =
            fluxa_streaming_engine::start_torrent_server(&cache_dir.to_string_lossy(), 0, "")
                .ok_or_else(|| "failed to start torrent server".to_string())?;
        let server: Value = serde_json::from_str(&server_json)
            .map_err(|e| format!("invalid torrent server response: {e}"))?;
        let base_url = server
            .get("url")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| "torrent server did not return url".to_string())?;
        let generation = server.get("generation").and_then(Value::as_u64);
        apply_torrent_preferences(&base_url, preferences.as_ref());
        (base_url, generation)
    };

    let stream: Value =
        serde_json::from_str(&stream_json).map_err(|e| format!("invalid stream json: {e}"))?;
    let playback_json = FluxaCore::stream_playback_info_json(&stream_json)
        .ok_or_else(|| "stream playback info could not be resolved".to_string())?;
    let playback: Value =
        serde_json::from_str(&playback_json).map_err(|e| format!("invalid playback info: {e}"))?;
    let link = playback
        .get("playableUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| "torrent stream has no playable link".to_string())?;
    let requested_file_idx = stream
        .get("fileIdx")
        .and_then(Value::as_i64)
        .map(|v| v as i32);
    let preferred_filename = stream
        .get("behaviorHints")
        .and_then(|hints| hints.get("filename"))
        .and_then(Value::as_str)
        .or_else(|| stream.get("filename").and_then(Value::as_str));
    let sources = stream
        .get("sources")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();

    let runtime_request = json!({
        "link": link,
        "title": title
            .or_else(|| stream.get("title").and_then(Value::as_str).map(str::to_string))
            .or_else(|| stream.get("name").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "Fluxa stream".to_string()),
        "requestedFileIdx": requested_file_idx,
        "preferredFilename": preferred_filename,
        "sources": sources,
        "fileStats": [],
        "rejectedIndex": Value::Null,
        "baseUrl": base_url,
        "play": true,
        "stat": false
    });
    let runtime_json = FluxaCore::torrent_runtime_info_json(&runtime_request.to_string())
        .ok_or_else(|| "torrent runtime info could not be resolved".to_string())?;
    let runtime: Value = serde_json::from_str(&runtime_json)
        .map_err(|e| format!("invalid torrent runtime response: {e}"))?;
    let stream_url = runtime
        .get("streamUrl")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "torrent runtime did not return streamUrl".to_string())?;
    let stats_link = runtime
        .get("normalizedLink")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| link.to_string());
    let selected_file_id = runtime
        .get("selectedFileIdx")
        .and_then(Value::as_i64)
        .map(|v| v as usize);
    start_torrent_add(&base_url, &stats_link, selected_file_id);
    Ok((
        stream_url,
        base_url,
        stats_link,
        generation,
        selected_file_id,
    ))
}

#[tauri::command]
fn stream_magnet_link(stream_json: String) -> Option<String> {
    FluxaCore::stream_magnet_link_json(&stream_json)
}

async fn ensure_healthy_torrent_base_url(
    state: &State<'_, DesktopState>,
    data_dir: &std::path::Path,
) -> Option<String> {
    let base_url = state.torrent_server_base_url.lock().unwrap().clone()?;
    let healthy = tauri::async_runtime::spawn_blocking({
        let base_url = base_url.clone();
        move || torrent_server_healthy(&base_url)
    })
    .await
    .unwrap_or(false);
    if healthy {
        return Some(base_url);
    }
    let previous_generation = state.torrent_generation.lock().unwrap().take();
    *state.torrent_server_base_url.lock().unwrap() = None;
    *state.torrent_stream_link.lock().unwrap() = None;
    *state.torrent_stream_file_id.lock().unwrap() = None;
    let cleanup_dir = data_dir.to_path_buf();
    let _ = tauri::async_runtime::spawn_blocking(move || {
        let stopped = fluxa_streaming_engine::stop_torrent_server(previous_generation);
        let _ = fs::remove_dir_all(cleanup_dir.join("torrent-cache"));
        stopped
    })
    .await;
    None
}

#[tauri::command]
async fn start_torrent_stream(
    state: State<'_, DesktopState>,
    stream_json: String,
    title: Option<String>,
    preferences: Option<Value>,
) -> Result<String, String> {
    let data_dir = state
        .data_dir
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "app data dir is not ready".to_string())?;
    let info_hash = serde_json::from_str::<Value>(&stream_json)
        .ok()
        .and_then(|stream| {
            stream
                .get("infoHash")
                .and_then(Value::as_str)
                .map(str::to_ascii_lowercase)
        });
    let existing_link = state.torrent_stream_link.lock().unwrap().clone();
    let existing_base_url = ensure_healthy_torrent_base_url(&state, &data_dir).await;
    let reuse_existing_server = existing_base_url.is_some();
    let same_torrent = info_hash.as_ref().is_some_and(|hash| {
        existing_link
            .as_ref()
            .is_some_and(|link| link.to_ascii_lowercase().contains(hash))
    });
    if reuse_existing_server && !same_torrent {
        if let (Some(base_url), Some(old_link)) = (existing_base_url.clone(), existing_link) {
            remove_torrent(&base_url, &old_link);
        }
        *state.torrent_stream_link.lock().unwrap() = None;
        *state.torrent_stream_file_id.lock().unwrap() = None;
    }
    let (stream_url, base_url, link, generation, file_id) =
        tauri::async_runtime::spawn_blocking(move || {
            start_torrent_stream_inner(data_dir, stream_json, title, preferences, existing_base_url)
        })
        .await
        .map_err(|e| e.to_string())??;
    *state.torrent_server_base_url.lock().unwrap() = Some(base_url);
    *state.torrent_stream_link.lock().unwrap() = Some(link);
    *state.torrent_stream_file_id.lock().unwrap() = file_id;
    if let Some(generation) = generation {
        *state.torrent_generation.lock().unwrap() = Some(generation);
    }
    Ok(stream_url)
}

pub(crate) async fn resolve_torrent_download_url(
    state: &State<'_, DesktopState>,
    stream_json: String,
) -> Result<String, String> {
    let data_dir = state
        .data_dir
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "app data dir is not ready".to_string())?;
    let existing_base_url = ensure_healthy_torrent_base_url(state, &data_dir).await;
    let (stream_url, base_url, _link, generation, _file_id) =
        tauri::async_runtime::spawn_blocking(move || {
            start_torrent_stream_inner(data_dir, stream_json, None, None, existing_base_url)
        })
        .await
        .map_err(|e| e.to_string())??;
    *state.torrent_server_base_url.lock().unwrap() = Some(base_url);
    if let Some(generation) = generation {
        *state.torrent_generation.lock().unwrap() = Some(generation);
    }
    Ok(stream_url)
}

#[tauri::command]
async fn stop_torrent_stream(state: State<'_, DesktopState>) -> Result<bool, String> {
    let was_playing = state.torrent_stream_link.lock().unwrap().take().is_some();
    *state.torrent_stream_file_id.lock().unwrap() = None;
    Ok(was_playing)
}

#[tauri::command]
async fn register_trailer_proxy_url(
    proxy_state: State<'_, trailer_proxy::TrailerProxyState>,
    url: String,
) -> Result<String, String> {
    trailer_proxy::register(&proxy_state, url).await
}

#[tauri::command]
async fn player_torrent_stats(state: State<'_, DesktopState>) -> Result<Option<Value>, String> {
    let base_url = state.torrent_server_base_url.lock().unwrap().clone();
    let link = state.torrent_stream_link.lock().unwrap().clone();
    let file_id = *state.torrent_stream_file_id.lock().unwrap();
    let (Some(base_url), Some(link)) = (base_url, link) else {
        return Ok(None);
    };
    let url = format!("{}/torrents", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&serde_json::json!({ "action": "get", "link": link, "file_id": file_id }))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let json_val = response.json::<Value>().await.map_err(|e| e.to_string())?;
    Ok(Some(json_val))
}

pub(crate) fn torrent_engine_request(
    url: &str,
    body: Option<&str>,
    read_timeout: std::time::Duration,
) -> Option<String> {
    use std::net::ToSocketAddrs;
    let rest = url.strip_prefix("http://")?;
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let (host, port) = authority
        .split_once(':')
        .and_then(|(host, port)| port.parse::<u16>().ok().map(|port| (host, port)))
        .unwrap_or((authority, 80));
    let path = format!("/{path}");
    let addr = (host, port).to_socket_addrs().ok()?.next()?;
    let mut stream =
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)).ok()?;
    stream.set_read_timeout(Some(read_timeout)).ok()?;
    let request = match body {
        Some(body) => format!(
            "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
        None => format!("GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n"),
    };
    std::io::Write::write_all(&mut stream, request.as_bytes()).ok()?;
    let mut response = Vec::new();
    let _ = std::io::Read::read_to_end(&mut stream, &mut response);
    Some(String::from_utf8_lossy(&response).into_owned())
}

fn torrent_server_healthy(base_url: &str) -> bool {
    let url = format!("{}/health", base_url.trim_end_matches('/'));
    torrent_engine_request(&url, None, std::time::Duration::from_secs(2))
        .is_some_and(|response| response.starts_with("HTTP/1.1 200"))
}

fn start_torrent_add(base_url: &str, link: &str, file_id: Option<usize>) {
    let url = format!("{}/torrents", base_url.trim_end_matches('/'));
    let body = json!({ "action": "add", "link": link, "file_id": file_id }).to_string();
    std::thread::spawn(move || {
        torrent_engine_request(&url, Some(&body), std::time::Duration::from_secs(60));
    });
}

fn remove_torrent(base_url: &str, link: &str) {
    let url = format!("{}/torrents", base_url.trim_end_matches('/'));
    let body = json!({ "action": "rem", "link": link }).to_string();
    std::thread::spawn(move || {
        torrent_engine_request(&url, Some(&body), std::time::Duration::from_secs(15));
    });
}

fn apply_torrent_preferences(base_url: &str, preferences: Option<&Value>) {
    let preset = preferences
        .and_then(|p| p.get("torrentSpeedPreset"))
        .and_then(Value::as_str)
        .unwrap_or("default");
    let preload_size = match preset {
        "fast" => 32,
        "ultra_fast" => 64,
        _ => 16,
    };
    let url = format!("{}/settings", base_url.trim_end_matches('/'));
    let body = json!({ "PreloadSize": preload_size }).to_string();
    std::thread::spawn(move || {
        torrent_engine_request(&url, Some(&body), std::time::Duration::from_secs(5));
    });
}

#[tauri::command]
fn get_data_dir(state: State<DesktopState>) -> Option<String> {
    state
        .data_dir
        .lock()
        .unwrap()
        .as_ref()
        .map(|d| d.to_string_lossy().to_string())
}

#[tauri::command]
fn player_hdr_supported(app: tauri::AppHandle) -> bool {
    #[cfg(target_os = "windows")]
    {
        return windows_player_surface::hdr_display_supported(&app);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        false
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _sentry_guard = sentry::init(sentry::ClientOptions {
        dsn: if cfg!(debug_assertions) {
            None
        } else {
            "https://7fe8e82cf7ea0eed65175d3d43afb1c0@o4511704565678080.ingest.de.sentry.io/4511706871693392"
                .parse()
                .ok()
        },
        release: sentry::release_name!(),
        ..Default::default()
    });

    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("MPV_LIBMPV_RENDER_BACKEND", "gpu-next") };
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_libmpv::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(main_window) = app.get_webview_window("main") {
                let _ = main_window.unminimize();
                let _ = main_window.show();
                let _ = main_window.set_focus();
            }

            for arg in &args {
                if !arg.starts_with("fluxa://") {
                    continue;
                }
                let url = match tauri::Url::parse(arg) {
                    Ok(url) => url,
                    Err(_) => continue,
                };
                let code = url
                    .query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.into_owned());
                let state = url
                    .query_pairs()
                    .find(|(key, _)| key == "state")
                    .map(|(_, value)| value.into_owned());
                if let Some(code) = code {
                    let service = if arg.contains("/trakt") {
                        "trakt"
                    } else if arg.contains("/anilist") {
                        "anilist"
                    } else if arg.contains("/simkl") {
                        "simkl"
                    } else {
                        continue;
                    };
                    queue_oauth_callback(app, service, code, state);
                }
            }
        }))
        .plugin({
            if std::env::var_os("FLUXA_DEBUG_LOGS").is_some() {
                DIAGNOSTIC_MODE.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            let librqbit_log_level = if std::env::var_os("FLUXA_TORRENT_DEBUG").is_some() {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Off
            };

            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .filter(|metadata| {
                    metadata.level() <= log::Level::Warn
                        || DIAGNOSTIC_MODE.load(std::sync::atomic::Ordering::Relaxed)
                })
                .max_file_size(20 * 1024 * 1024)
                .level_for("librqbit", librqbit_log_level)
                .level_for("librqbit_dht", librqbit_log_level)
                .level_for("librqbit_tracker_comms", librqbit_log_level)
                .level_for("librqbit_upnp", librqbit_log_level)
                .level_for("librqbit_core", librqbit_log_level)
                .level_for("librqbit_peer_protocol", librqbit_log_level)
                .level_for("tracing::span", log::LevelFilter::Off)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                ])
                .build()
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .manage(DesktopState::default())
        .manage(PendingOAuthCallbacks::default())
        .manage(discord_presence::DiscordPresenceState::default())
        .manage(cast::CastState::default())
        .manage(chromecast::ChromecastState::default())
        .manage(airplay::AirplayState::default())
        .manage(roku::RokuState::default())
        .manage(cast_proxy::CastProxyState::default())
        .manage(trailer_proxy::TrailerProxyState::default())
        .setup(|app| {
            #[cfg(any(target_os = "linux", all(debug_assertions, target_os = "windows")))]
            app.deep_link().register_all()?;

            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir")
                .join("fluxa");

            let state = app.state::<DesktopState>();
            *state.data_dir.lock().unwrap() = Some(data_dir.clone());
            let _ = fs::create_dir_all(&data_dir);

            let torrent_cache_dir = data_dir.join("torrent-cache");
            std::thread::spawn(move || {
                let _ = fs::remove_dir_all(&torrent_cache_dir);
            });

            if let Ok(cache_dir) = app.path().app_cache_dir() {
                std::thread::spawn(move || {
                    if let Ok(entries) = fs::read_dir(&cache_dir) {
                        for entry in entries.flatten() {
                            let name = entry.file_name();
                            let name = name.to_string_lossy();
                            if name.starts_with("gc_")
                                && (name.ends_with(".mp4")
                                    || name.ends_with(".webm")
                                    || name.ends_with(".gif")
                                    || name.ends_with(".webp"))
                            {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                });
            }

            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    let s = url.as_str();
                    let code = url
                        .query_pairs()
                        .find(|(k, _)| k == "code")
                        .map(|(_, v)| v.into_owned());
                    let state = url
                        .query_pairs()
                        .find(|(k, _)| k == "state")
                        .map(|(_, v)| v.into_owned());
                    if let Some(code) = code {
                        let service = if s.contains("/trakt") {
                            "trakt"
                        } else if s.contains("/anilist") {
                            "anilist"
                        } else if s.contains("/simkl") {
                            "simkl"
                        } else {
                            continue;
                        };
                        queue_oauth_callback(&handle, service, code, state);
                    } else {
                        let _ = handle.emit("deep-link-opened", json!({ "url": s }));
                    }
                }
            });

            if let Some(main_window) = app.get_webview_window("main") {
                let close_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let state = close_handle.state::<DesktopState>();
                        if state
                            .close_flush_done
                            .load(std::sync::atomic::Ordering::SeqCst)
                        {
                            return;
                        }
                        api.prevent_close();
                        let handle = close_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = handle.emit("native-app-close-requested", ());
                            tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
                            let state = handle.state::<DesktopState>();
                            if !state
                                .close_flush_done
                                .swap(true, std::sync::atomic::Ordering::SeqCst)
                            {
                                if let Some(window) = handle.get_webview_window("main") {
                                    let _ = window.close();
                                }
                            }
                        });
                    }
                });
            }

            #[cfg(target_os = "windows")]
            if let Some(main_window) = app.get_webview_window("main") {
                let store_size = |window: &tauri::WebviewWindow| {
                    if let Ok(size) = window.inner_size() {
                        let state = window.state::<DesktopState>();
                        let packed =
                            ((size.width.max(2) as u64) << 32) | (size.height.max(2) as u64);
                        state
                            .main_window_size
                            .store(packed, std::sync::atomic::Ordering::Release);
                    }
                };
                store_size(&main_window);
                let window_for_event = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Resized(_) = event {
                        store_size(&window_for_event);
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_linux,
            debug_log,
            app_close_flush_done,
            set_diagnostic_mode,
            export_diagnostic_log,
            engine_init,
            engine_dispatch,
            engine_complete_effect,
            engine_snapshot,
            http_fetch_text,
            http_execute_text,
            storage_read,
            storage_write,
            storage_delete,
            library_progress_read,
            library_progress_list,
            library_progress_upsert,
            library_progress_delete,
            library_status_set,
            library_status_list,
            library_watched_set,
            library_watched_list,
            library_last_watched_list,
            library_last_watched_upsert,
            library_last_watched_delete,
            library_continue_watching_list,
            library_continue_watching_upsert,
            library_continue_watching_delete,
            core_invoke,
            run_plugin_scraper,
            stream_magnet_link,
            start_torrent_stream,
            stop_torrent_stream,
            register_trailer_proxy_url,
            player_init,
            player_apply_preferences,
            player_set_http_headers,
            player_load,
            player_render_frame,
            player_command,
            player_auto_sync_subtitles,
            player_capture_subtitle_cues,
            player_set_anime4k_enabled,
            player_get_anime4k_enabled,
            player_show_loading,
            player_hide,
            player_set_title,
            player_set_cursor_visible,
            player_set_loading_artwork,
            player_prefetch_artwork,
            enqueue_offline_download,
            player_add_subtitle,
            player_torrent_sibling_subtitles,
            player_title,
            player_status,
            player_set_sleep_inhibition,
            player_destroy,
            player_get_playback_info,
            player_track_options,
            player_set_seek_thumbnail_enabled,
            player_hdr_supported,
            player_get_seek_thumbnail,
            player_screenshot,
            custom_fonts_list,
            custom_fonts_add,
            custom_fonts_remove,
            cache_poster_image,
            player_set_chapters,
            player_clear_chapters,
            player_set_skip_info,
            player_clear_skip_info,
            player_set_episodes,
            player_clear_episodes,
            get_oauth_client_id,
            take_oauth_callback,
            nuvio_request,
            trakt_device_start,
            trakt_device_poll,
            trakt_oauth_exchange,
            trakt_oauth_refresh,
            anilist_oauth_exchange,
            simkl_oauth_exchange,
            get_data_dir,
            set_download_dir,
            list_offline_downloads,
            delete_offline_download,
            pause_offline_download,
            resume_offline_download,
            cancel_offline_download,
            discord_presence_configure,
            discord_presence_update,
            discord_presence_set_viewing,
            discord_presence_set_browsing,
            discord_presence_clear,
            cast_discover_devices,
            cast_resolve_media_url,
            cast_set_media,
            cast_play,
            cast_pause,
            cast_seek,
            cast_set_volume,
            cast_disconnect,
            chromecast_discover_devices,
            chromecast_connect,
            chromecast_play,
            chromecast_pause,
            chromecast_seek,
            chromecast_set_volume,
            chromecast_disconnect,
            airplay_discover_devices,
            airplay_set_media,
            airplay_play,
            airplay_pause,
            airplay_seek,
            airplay_set_volume,
            airplay_disconnect,
            roku_discover_devices,
            roku_set_media,
            roku_play_pause,
            roku_disconnect,
            cast_proxy_serve,
            player_torrent_stats,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Fluxa Desktop")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                fluxa_streaming_engine::stop_torrent_server(None);
            }
        });
}
