use fluxa_core::FluxaCore;
#[cfg(target_os = "linux")]
mod linux_player_surface;
#[cfg(target_os = "windows")]
mod windows_player_surface;
#[cfg(target_os = "macos")]
mod macos_player_surface;
mod mpv_render;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;

// ── App state ─────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct HttpTextResponse {
    status_code: u16,
    body: String,
}

pub struct DesktopState {
    engine_handle: Mutex<Option<u64>>,
    data_dir: Mutex<Option<PathBuf>>,
    player_renderer: Mutex<Option<mpv_render::MpvRenderer>>,
    #[cfg(target_os = "linux")]
    native_player_surface: Mutex<Option<linux_player_surface::NativePlayerSurface>>,
    #[cfg(target_os = "windows")]
    native_player_surface: Mutex<Option<windows_player_surface::NativePlayerSurface>>,
    #[cfg(target_os = "macos")]
    native_player_surface: Mutex<Option<macos_player_surface::NativePlayerSurface>>,
    pub chapters_json: Mutex<Option<String>>,
    pub skip_segments_json: Mutex<Option<String>>,
    pub next_ep_subtitle: Mutex<String>,
    pub next_ep_threshold_percent: Mutex<f64>,
    pub auto_play_next_episode: Mutex<bool>,
    pub eof_next_fired: Mutex<bool>,
    pub episodes_json: Mutex<Option<String>>,
    pub thumb_url: Mutex<Option<String>>,
    pub seek_thumbnail_enabled: Mutex<bool>,
    // Set to true when player_hide is called so the GTK timer can detect a hide
    // request that arrived while prepare_and_load was blocking the GTK main thread
    // (e.g. during the one-time OpenGL context creation on first play).
    pub pending_hide: AtomicBool,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            engine_handle: Mutex::new(None),
            data_dir: Mutex::new(None),
            player_renderer: Mutex::new(None),
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
            eof_next_fired: Mutex::new(false),
            episodes_json: Mutex::new(None),
            thumb_url: Mutex::new(None),
            seek_thumbnail_enabled: Mutex::new(false),
            pending_hide: AtomicBool::new(false),
        }
    }
}

// ── Tauri commands ─────────────────────────────────────────────────────────────

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
fn storage_read(state: State<DesktopState>, key: String) -> Option<String> {
    let dir = state.data_dir.lock().unwrap().clone()?;
    let path = dir.join(format!("{}.json", sanitize_key(&key)));
    fs::read_to_string(path).ok()
}

#[tauri::command]
fn storage_write(state: State<DesktopState>, key: String, value: String) -> bool {
    let dir = match state.data_dir.lock().unwrap().clone() {
        Some(d) => d,
        None => return false,
    };
    if fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let path = dir.join(format!("{}.json", sanitize_key(&key)));
    write_file_atomic(&path, value.as_bytes()).is_ok()
}

#[tauri::command]
fn storage_delete(state: State<DesktopState>, key: String) -> bool {
    let dir = match state.data_dir.lock().unwrap().clone() {
        Some(d) => d,
        None => return false,
    };
    let path = dir.join(format!("{}.json", sanitize_key(&key)));
    fs::remove_file(path).is_ok()
}

#[tauri::command]
async fn http_fetch_text(url: String) -> Result<HttpTextResponse, String> {
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|error| error.to_string())?
        .get(&url)
        .header("User-Agent", "Fluxa/1.0")
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status_code = response.status().as_u16();
    let body = response.text().await.map_err(|error| error.to_string())?;
    Ok(HttpTextResponse { status_code, body })
}

#[tauri::command]
fn core_normalize_manifest_url(raw_url: String) -> String {
    FluxaCore::normalize_manifest_url(&raw_url)
}

#[tauri::command]
fn core_manifest_fetch_plan(raw_url: String) -> Option<String> {
    FluxaCore::manifest_fetch_plan_json(&raw_url)
}

#[tauri::command]
fn core_parse_manifest(body: String, transport_url: String) -> Option<String> {
    FluxaCore::parse_manifest_json(&body, &transport_url, "Unknown Addon")
}

#[tauri::command]
fn core_resolve_manifest_assets(descriptor_json: String) -> Option<String> {
    FluxaCore::resolve_manifest_assets_json(&descriptor_json)
}

#[tauri::command]
fn core_merge_live_manifest(
    descriptor_json: String,
    live_json: Option<String>,
    unknown_name: Option<String>,
) -> Option<String> {
    FluxaCore::merge_live_manifest_json(
        &descriptor_json,
        live_json.as_deref(),
        unknown_name.as_deref().unwrap_or("Unknown Addon"),
    )
}

#[tauri::command]
fn core_build_resource_url(
    transport_url: String,
    resource: String,
    content_type: String,
    id: String,
    extra_json: Option<String>,
) -> String {
    FluxaCore::build_resource_url(
        &transport_url,
        &resource,
        &content_type,
        &id,
        extra_json.as_deref(),
    )
}

#[tauri::command]
fn core_supports_resource(
    manifest_json: String,
    resource_name: String,
    content_type: Option<String>,
    id: Option<String>,
) -> bool {
    FluxaCore::supports_resource(
        &manifest_json,
        &resource_name,
        content_type.as_deref(),
        id.as_deref(),
    )
}

#[tauri::command]
fn core_catalog_supports_extra(catalog_json: String, extra_name: String) -> bool {
    FluxaCore::catalog_supports_extra(&catalog_json, &extra_name)
}

#[tauri::command]
fn core_catalog_requires_extra(catalog_json: String, extra_name: String) -> bool {
    FluxaCore::catalog_requires_extra(&catalog_json, &extra_name)
}

#[tauri::command]
fn core_catalog_has_required_extra_except(
    catalog_json: String,
    allowed_names_json: String,
) -> bool {
    FluxaCore::catalog_has_required_extra_except(&catalog_json, &allowed_names_json)
}

#[tauri::command]
fn core_parse_addon_resource_result(
    resource: String,
    url: String,
    status_code: i32,
    body: Option<String>,
) -> String {
    FluxaCore::parse_addon_resource_result_json(&resource, &url, status_code, body.as_deref())
}

#[tauri::command]
fn core_addon_resource_request_plan(request_json: String) -> Option<String> {
    FluxaCore::addon_resource_request_plan_json(&request_json)
}

#[tauri::command]
fn core_resource_fetch_plan(request_json: String) -> Option<String> {
    FluxaCore::resource_fetch_plan_json(&request_json)
}

#[tauri::command]
fn core_resource_parse_plan(request_json: String) -> Option<String> {
    FluxaCore::resource_parse_plan_json(&request_json)
}

#[tauri::command]
fn core_playback_prepare_plan(request_json: String) -> Option<String> {
    FluxaCore::playback_prepare_plan_json(&request_json)
}

#[tauri::command]
fn core_library_local_state_plan(request_json: String) -> Option<String> {
    FluxaCore::library_local_state_plan_json(&request_json)
}

#[tauri::command]
fn core_preferences_schema() -> String {
    FluxaCore::preferences_schema_json()
}

#[tauri::command]
fn core_apply_preference_update(request_json: String) -> Option<String> {
    FluxaCore::apply_preference_update_json(&request_json)
}

#[tauri::command]
fn core_addon_collection_mutation_plan(request_json: String) -> Option<String> {
    FluxaCore::addon_collection_mutation_plan_json(&request_json)
}

#[tauri::command]
fn core_detail_episode_plan(request_json: String) -> Option<String> {
    FluxaCore::detail_episode_plan_json(&request_json)
}

#[tauri::command]
fn core_normalize_addon_subtitles(subtitles_json: String, resource_url: String) -> String {
    FluxaCore::normalize_addon_subtitles_json(&subtitles_json, &resource_url)
}

#[tauri::command]
fn core_stream_playback_info(stream_json: String) -> Option<String> {
    FluxaCore::stream_playback_info_json(&stream_json)
}

#[tauri::command]
fn core_torrent_runtime_info(request_json: String) -> Option<String> {
    FluxaCore::torrent_runtime_info_json(&request_json)
}

#[tauri::command]
fn core_search_result_grouping(request_json: String) -> Option<String> {
    FluxaCore::search_result_grouping_json(&request_json)
}

#[tauri::command]
fn core_build_metadata_feed_options(addons_json: String) -> Option<String> {
    FluxaCore::build_metadata_feed_options_json(&addons_json)
}

#[tauri::command]
fn core_discover_catalog_options(addons_json: String, selected_type: String) -> Option<String> {
    FluxaCore::discover_catalog_options_json(&addons_json, &selected_type)
}

#[tauri::command]
fn core_discover_sort_plan(request_json: String) -> Option<String> {
    FluxaCore::discover_sort_plan_json(&request_json)
}

#[tauri::command]
fn core_library_sort_plan(request_json: String) -> Option<String> {
    FluxaCore::library_sort_plan_json(&request_json)
}

#[tauri::command]
fn core_watchlist_toggle_plan(request_json: String) -> Option<String> {
    FluxaCore::watchlist_toggle_plan_json(&request_json)
}

#[tauri::command]
fn core_playback_progress_merge_plan(request_json: String) -> Option<String> {
    FluxaCore::playback_progress_merge_plan_json(&request_json)
}

#[tauri::command]
fn core_library_continue_watching_items(items_json: String) -> Option<String> {
    FluxaCore::library_continue_watching_items_json(&items_json)
}

#[tauri::command]
fn core_detail_series_lookup_id(raw_id: String) -> String {
    FluxaCore::detail_series_lookup_id(&raw_id)
}

#[tauri::command]
fn core_detail_season_load_plan(request_json: String) -> Option<String> {
    FluxaCore::detail_season_load_plan_json(&request_json)
}

#[tauri::command]
fn core_player_backend_selection(request_json: String) -> Option<String> {
    FluxaCore::player_backend_selection_json(&request_json)
}

#[tauri::command]
fn core_player_buffer_targets(request_json: String) -> Option<String> {
    FluxaCore::player_buffer_targets_json(&request_json)
}

#[tauri::command]
fn core_offline_download_plan(request_json: String) -> Option<String> {
    FluxaCore::offline_download_plan_json(&request_json)
}

#[tauri::command]
fn core_playback_intro_lookup_content_id(id: String) -> String {
    FluxaCore::playback_intro_lookup_content_id(&id)
}

#[tauri::command]
fn core_player_source_sidebar_plan(request_json: String) -> Option<String> {
    FluxaCore::player_source_sidebar_plan_json(&request_json)
}

#[tauri::command]
fn core_player_retry_policy(request_json: String) -> Option<String> {
    FluxaCore::player_retry_policy_json(&request_json)
}

#[tauri::command]
fn core_effective_metadata_feed_selection(
    selected_keys_json: String,
    available_keys_json: String,
) -> Option<String> {
    FluxaCore::effective_metadata_feed_selection_json(&selected_keys_json, &available_keys_json)
}

#[tauri::command]
fn core_toggle_metadata_feed_limited(
    selected_keys_json: String,
    available_keys_json: String,
    key: String,
    max_enabled: i32,
) -> Option<String> {
    FluxaCore::toggle_metadata_feed_limited_json(
        &selected_keys_json,
        &available_keys_json,
        &key,
        max_enabled,
    )
}

#[tauri::command]
fn core_find_preferred_subtitle_index(
    tracks_json: String,
    last_subtitle_language: Option<String>,
    preferred_subtitle_language: Option<String>,
    secondary_subtitle_language: Option<String>,
) -> i32 {
    FluxaCore::find_preferred_subtitle_index(
        &tracks_json,
        last_subtitle_language.as_deref(),
        preferred_subtitle_language.as_deref(),
        secondary_subtitle_language.as_deref(),
    )
}

#[tauri::command]
fn start_torrent_stream(
    state: State<DesktopState>,
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
    let cache_dir = data_dir.join("torrent-cache");
    let server_json = fluxa_streaming_engine::start_torrent_server(&cache_dir.to_string_lossy(), 0)
        .ok_or_else(|| "failed to start torrent server".to_string())?;
    let server: Value = serde_json::from_str(&server_json)
        .map_err(|error| format!("invalid torrent server response: {error}"))?;
    let base_url = server
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| "torrent server did not return url".to_string())?;
    apply_torrent_preferences(base_url, preferences.as_ref());

    let stream: Value = serde_json::from_str(&stream_json)
        .map_err(|error| format!("invalid stream json: {error}"))?;
    let playback_json = FluxaCore::stream_playback_info_json(&stream_json)
        .ok_or_else(|| "stream playback info could not be resolved".to_string())?;
    let playback: Value = serde_json::from_str(&playback_json)
        .map_err(|error| format!("invalid playback info: {error}"))?;
    let link = playback
        .get("playableUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| "torrent stream has no playable link".to_string())?;
    let requested_file_idx = stream
        .get("fileIdx")
        .and_then(Value::as_i64)
        .map(|value| value as i32);
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
        .map_err(|error| format!("invalid torrent runtime response: {error}"))?;
    runtime
        .get("streamUrl")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "torrent runtime did not return streamUrl".to_string())
}

#[tauri::command]
async fn stop_torrent_stream() -> bool {
    tauri::async_runtime::spawn_blocking(fluxa_streaming_engine::stop_torrent_server)
        .await
        .unwrap_or(false)
}

fn apply_torrent_preferences(base_url: &str, preferences: Option<&Value>) {
    let preset = preferences
        .and_then(|prefs| prefs.get("torrentSpeedPreset"))
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
        let Some(rest) = url.strip_prefix("http://") else {
            return;
        };
        let (authority, path) = rest.split_once('/').unwrap_or((rest, "settings"));
        let (host, port) = authority
            .split_once(':')
            .and_then(|(host, port)| port.parse::<u16>().ok().map(|port| (host, port)))
            .unwrap_or((authority, 80));
        let path = format!("/{path}");
        let Ok(mut stream) = std::net::TcpStream::connect((host, port)) else {
            return;
        };
        let request = format!(
            "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = std::io::Write::write_all(&mut stream, request.as_bytes());
    });
}

#[tauri::command]
fn player_init(app: AppHandle, state: State<DesktopState>) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    let _ = ensure_native_player_surface(&app, &state);

    let mut renderer = state.player_renderer.lock().unwrap();
    if renderer.is_none() {
        *renderer = Some(mpv_render::MpvRenderer::new()?);
    }
    Ok(())
}

#[tauri::command]
fn player_load(
    app: AppHandle,
    state: State<DesktopState>,
    url: String,
    start_at: Option<u64>,
    total_duration: Option<u64>,
) -> Result<(), String> {
    *state.thumb_url.lock().unwrap() = Some(url.clone());

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        if let Some(surface) = ensure_native_player_surface(&app, &state) {
            return surface.load(url, start_at, total_duration);
        }
    }

    let _ = app;
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
fn player_set_seek_thumbnail_enabled(state: State<DesktopState>, enabled: bool) {
    *state.seek_thumbnail_enabled.lock().unwrap() = enabled;
}

#[tauri::command]
fn player_apply_preferences(
    state: State<DesktopState>,
    preferences: serde_json::Value,
) -> Result<(), String> {
    let options = mpv_options_from_preferences(&preferences);
    if options.is_empty() {
        return Ok(());
    }
    state
        .player_renderer
        .lock()
        .unwrap()
        .as_ref()
        .ok_or_else(|| "player renderer is not initialized".to_string())?
        .apply_options(&options)
}

#[tauri::command]
fn player_set_title(state: State<DesktopState>, title: String, episode_title: Option<String>) {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
        surface.set_title(title, episode_title);
    }
}

#[tauri::command]
async fn player_set_loading_artwork(
    state: State<'_, DesktopState>,
    title: String,
    episode_title: Option<String>,
    background_url: Option<String>,
    logo_url: Option<String>,
) -> Result<(), String> {
    // NOTE: Surface is read AFTER decode so this command can be fired concurrently
    // with player_show_loading (which installs the surface). Decode takes at least a
    // few ms; surface install is ~1ms, so the surface is always ready by then.
    // On a decoded-cache hit (near-instant), we do a short async wait to let the
    // concurrent player_show_loading finish installing the surface before we send.
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    let (background_scaled, logo_scaled) = {
        // Check the pre-decoded cache first (populated by player_prefetch_artwork).
        // On a cache hit we skip both network fetch and image decode entirely.
        let bg_cached = background_url.as_deref()
            .and_then(|u| artwork_bg_decoded().lock().ok()?.get(&normalize_url(u)).cloned());
        let logo_cached = logo_url.as_deref()
            .and_then(|u| artwork_logo_decoded().lock().ok()?.get(&normalize_url(u)).cloned());

        // Fast path: every requested URL is already decoded. logo_url=None counts as
        // "ready" — only treat it as a miss when a URL was provided but not yet decoded.
        let bg_ready = bg_cached.is_some() || background_url.is_none();
        let logo_ready = logo_cached.is_some() || logo_url.is_none();
        if bg_ready && logo_ready {
            (bg_cached, logo_cached)
        } else {
            // Only fetch URLs whose decoded form isn't cached yet.
            let bg_fetch = if bg_cached.is_none() { background_url.clone() } else { None };
            let logo_fetch = if logo_cached.is_none() { logo_url.clone() } else { None };
            let bg_handle = tauri::async_runtime::spawn(fetch_player_artwork_bytes_owned(bg_fetch));
            let logo_handle = tauri::async_runtime::spawn(fetch_player_artwork_bytes_owned(logo_fetch));
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
        // Read surface after decode. If it's still None the concurrent player_show_loading
        // hasn't installed it yet (only possible on a decoded-cache hit which completes
        // faster than the IPC round-trip for show_loading). Wait up to ~60ms for it.
        let mut surface = state.native_player_surface.lock().unwrap().clone();
        if surface.is_none() {
            for _ in 0..6 {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                surface = state.native_player_surface.lock().unwrap().clone();
                if surface.is_some() { break; }
            }
        }
        if let Some(surface) = surface {
            surface.set_artwork(title, episode_title, background_scaled, logo_scaled);
        }
    }
    Ok(())
}

async fn fetch_player_artwork_bytes_owned(url: Option<String>) -> Option<Vec<u8>> {
    fetch_player_artwork_bytes(url.as_deref()).await
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn scale_artwork_cover(
    bytes: Vec<u8>,
    target_w: u32,
    target_h: u32,
) -> Option<(Vec<u8>, i32, i32)> {
    let img = image::load_from_memory(&bytes).ok()?;
    let filled = img.resize_to_fill(target_w, target_h, image::imageops::FilterType::Triangle);
    let rgba = filled.to_rgba8();
    Some((rgba.into_raw(), target_w as i32, target_h as i32))
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn scale_artwork_fit(bytes: Vec<u8>, max_w: u32, max_h: u32) -> Option<(Vec<u8>, i32, i32)> {
    let img = image::load_from_memory(&bytes).ok()?;
    let resized = img.resize(max_w, max_h, image::imageops::FilterType::Triangle);
    let (rw, rh) = (resized.width(), resized.height());
    let rgba = resized.to_rgba8();
    Some((rgba.into_raw(), rw as i32, rh as i32))
}

#[tauri::command]
async fn enqueue_offline_download(
    state: State<'_, DesktopState>,
    request_json: String,
) -> Result<Option<String>, String> {
    let plan_json = FluxaCore::offline_download_plan_json(&request_json)
        .ok_or_else(|| "offline download plan could not be created".to_string())?;
    let plan: Value = serde_json::from_str(&plan_json)
        .map_err(|error| format!("invalid offline download plan: {error}"))?;
    if plan.get("supported").and_then(Value::as_bool) != Some(true) {
        return Ok(Some(plan_json));
    }
    let data_dir = state
        .data_dir
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "app data dir is not ready".to_string())?;
    let offline_dir = data_dir.join("offline");
    fs::create_dir_all(&offline_dir).map_err(|error| error.to_string())?;
    let playback_url = plan
        .get("playbackUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| "offline plan has no playback url".to_string())?;
    let video_file_name = plan
        .get("videoFileName")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "offline plan has no video file name".to_string())?;
    let safe_video_file_name = sanitize_file_name(&video_file_name);
    let target_path = offline_dir.join(&safe_video_file_name);
    let temp_path = offline_dir.join(format!("{safe_video_file_name}.part"));
    let mut response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60 * 60))
        .build()
        .map_err(|error| error.to_string())?
        .get(playback_url)
        .header("User-Agent", "Fluxa/1.0")
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("download failed: HTTP {}", response.status()));
    }
    let mut file = fs::File::create(&temp_path).map_err(|error| error.to_string())?;
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        file.write_all(&chunk).map_err(|error| error.to_string())?;
    }
    file.flush().map_err(|error| error.to_string())?;
    drop(file);
    if target_path.exists() {
        fs::remove_file(&target_path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temp_path, &target_path).map_err(|error| error.to_string())?;
    let mut completed = plan;
    completed["status"] = json!("downloaded");
    completed["path"] = json!(target_path.to_string_lossy().to_string());
    completed["videoFileName"] = json!(safe_video_file_name);
    serde_json::to_string(&completed)
        .map(Some)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn player_add_subtitle(
    state: State<DesktopState>,
    url: String,
    title: Option<String>,
    language: Option<String>,
) -> Result<(), String> {
    state
        .player_renderer
        .lock()
        .unwrap()
        .as_ref()
        .ok_or_else(|| "player renderer is not initialized".to_string())?
        .add_subtitle(&url, title.as_deref(), language.as_deref())
}

#[tauri::command]
fn player_render_frame(
    state: State<DesktopState>,
    width: i32,
    height: i32,
) -> Result<mpv_render::PlayerFrame, String> {
    let mut renderer = state.player_renderer.lock().unwrap();
    renderer
        .as_mut()
        .ok_or_else(|| "player renderer is not initialized".to_string())?
        .render_frame(width, height)
}

#[tauri::command]
fn player_command(state: State<DesktopState>, command: String) -> Result<(), String> {
    if command == "stop" {
        *state.eof_next_fired.lock().unwrap() = true;
    }
    // Use try_lock so a stuck or slow renderer (e.g. opening a network stream) never
    // blocks a Tauri thread-pool thread indefinitely — that would starve other commands.
    let renderer = state.player_renderer.try_lock()
        .map_err(|_| "player renderer busy".to_string())?;
    renderer
        .as_ref()
        .ok_or_else(|| "player renderer is not initialized".to_string())?
        .command_string(&command)
}

#[tauri::command]
fn player_show_loading(
    app: AppHandle,
    state: State<DesktopState>,
    title: String,
    episode_title: Option<String>,
) {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        if let Some(surface) = ensure_native_player_surface(&app, &state) {
            surface.show_loading(title, episode_title);
        }
    }
}

#[tauri::command]
fn player_hide(state: State<DesktopState>) {
    // Set flag BEFORE sending the Hide command so the GTK timer can detect a hide
    // request that arrived while prepare_and_load was blocking (first-play GL init).
    state.pending_hide.store(true, Ordering::Release);
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
        surface.hide();
    }

    let _ = state;
}

#[tauri::command]
fn player_title(state: State<DesktopState>) -> Option<String> {
    state
        .player_renderer
        .lock()
        .unwrap()
        .as_ref()
        .and_then(mpv_render::MpvRenderer::title)
}

fn mpv_options_from_preferences(preferences: &serde_json::Value) -> Vec<(String, String)> {
    let mut options = Vec::new();
    let get = |key: &str| preferences.get(key).and_then(|value| value.as_str());

    if let Some(speed) = get("playbackSpeed").and_then(|value| value.parse::<f64>().ok()) {
        if (0.25..=4.0).contains(&speed) {
            options.push(("speed".to_string(), format!("{speed:.2}")));
        }
    }
    let buffer_request = json!({
        "cacheSizeMb": get("playerBufferCacheMb").and_then(|value| value.parse::<i64>().ok()),
        "forwardBufferSeconds": get("playerForwardBufferSeconds").and_then(|value| value.parse::<i64>().ok()),
        "backBufferSeconds": get("playerBackBufferSeconds").and_then(|value| value.parse::<i64>().ok()),
        "isTorrent": preferences.get("isTorrentPlayback").and_then(Value::as_bool).unwrap_or(false)
    });
    if let Some(targets_json) = FluxaCore::player_buffer_targets_json(&buffer_request.to_string()) {
        if let Ok(targets) = serde_json::from_str::<Value>(&targets_json) {
            if let Some(cache_bytes) = targets.get("cacheSizeBytes").and_then(Value::as_i64) {
                options.push(("demuxer-max-bytes".to_string(), cache_bytes.to_string()));
            }
            if let Some(forward_ms) = targets.get("forwardBufferMs").and_then(Value::as_i64) {
                let seconds = (forward_ms / 1000).max(1).to_string();
                options.push(("cache-secs".to_string(), seconds.clone()));
                options.push(("demuxer-readahead-secs".to_string(), seconds));
            }
        }
    }
    if preferences
        .get("forceSoftwareAudio")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        options.push(("ad".to_string(), "lavc".to_string()));
    }
    if let Some(size) = get("subtitleSize").and_then(|value| value.parse::<f64>().ok()) {
        options.push((
            "sub-scale".to_string(),
            format!("{:.2}", (size / 100.0).clamp(0.5, 2.0)),
        ));
    }
    let sub_text_opacity = get("subtitleTextOpacity")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    if let Some(color) =
        get("subtitleColor").and_then(|v| css_hex_with_alpha_to_mpv_color(v, sub_text_opacity))
    {
        options.push(("sub-color".to_string(), color));
    }
    let sub_border_opacity = get("subtitleOutlineOpacity")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    if let Some(color) = get("subtitleOutlineColor")
        .and_then(|v| css_hex_with_alpha_to_mpv_color(v, sub_border_opacity))
    {
        options.push(("sub-border-color".to_string(), color));
    }
    let sub_bg_opacity = get("subtitleBackgroundOpacity")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);
    if let Some(color) = get("subtitleBackgroundColor")
        .and_then(|v| css_hex_with_alpha_to_mpv_color(v, sub_bg_opacity))
    {
        options.push(("sub-back-color".to_string(), color));
    }
    if preferences
        .get("autoEnableSubtitles")
        .and_then(|value| value.as_bool())
        == Some(false)
    {
        options.push(("sid".to_string(), "no".to_string()));
    }
    if preferences
        .get("subtitleShadow")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        options.push(("sub-shadow-offset".to_string(), "3".to_string()));
        options.push(("sub-shadow-color".to_string(), "#80000000".to_string()));
    } else {
        options.push(("sub-shadow-offset".to_string(), "0".to_string()));
    }
    let audio_languages =
        language_list(&[get("preferredAudioLanguage"), get("secondaryAudioLanguage")]);
    if !audio_languages.is_empty() {
        options.push(("alang".to_string(), audio_languages));
    }
    let subtitle_languages = language_list(&[
        get("preferredSubtitleLanguage"),
        get("secondarySubtitleLanguage"),
    ]);
    if !subtitle_languages.is_empty() {
        options.push(("slang".to_string(), subtitle_languages));
    }
    if let Some(custom) = get("mpvCustomOptions") {
        for line in custom.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((name, value)) = line.split_once('=') {
                let name = name.trim();
                let value = value.trim();
                if is_safe_mpv_option_name(name) && !value.is_empty() {
                    options.push((name.to_string(), value.to_string()));
                }
            }
        }
    }

    if let Some(mode) = get("audioDecoderMode") {
        let hwdec = match mode {
            "hw_prefer" => "auto-safe",
            "hw_only" => "auto",
            "sw_only" => "no",
            _ => "",
        };
        if !hwdec.is_empty() {
            options.push(("hwdec".to_string(), hwdec.to_string()));
        }
    }

    if preferences
        .get("showFpsCounter")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        options.push(("osd-level".to_string(), "3".to_string()));
    }

    options
}

fn css_hex_with_alpha_to_mpv_color(value: &str, opacity: f64) -> Option<String> {
    let hex = value.trim().strip_prefix('#')?;
    if hex.len() == 6 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        Some(format!("#{hex}{alpha:02X}"))
    } else {
        None
    }
}

fn css_hex_to_mpv_color(value: &str) -> Option<String> {
    let hex = value.trim().strip_prefix('#')?;
    if hex.len() == 6 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Some(format!("#{hex}"))
    } else {
        None
    }
}

fn is_safe_mpv_option_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '/')
}

fn language_list(values: &[Option<&str>]) -> String {
    values
        .iter()
        .filter_map(|value| value.map(str::trim))
        .filter(|value| !value.is_empty() && *value != "none")
        .filter(|value| {
            value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[tauri::command]
fn player_status(state: State<DesktopState>) -> Result<mpv_render::PlayerStatus, String> {
    state
        .player_renderer
        .try_lock()
        .map_err(|_| "player renderer busy".to_string())?
        .as_ref()
        .ok_or_else(|| "player renderer is not initialized".to_string())
        .map(mpv_render::MpvRenderer::status)
}

#[tauri::command]
fn player_get_playback_info(state: State<DesktopState>) -> serde_json::Value {
    serde_json::json!({
        "skipSegmentsJson": state.skip_segments_json.lock().unwrap().clone(),
        "chaptersJson": state.chapters_json.lock().unwrap().clone(),
        "episodesJson": state.episodes_json.lock().unwrap().clone(),
        "nextEpSubtitle": state.next_ep_subtitle.lock().unwrap().clone(),
        "nextEpThresholdPercent": *state.next_ep_threshold_percent.lock().unwrap(),
        "autoPlayNextEpisode": *state.auto_play_next_episode.lock().unwrap(),
    })
}

#[tauri::command]
fn player_track_options(state: State<DesktopState>, track_type: String) -> Vec<mpv_render::PlayerTrackOption> {
    state
        .player_renderer
        .try_lock()
        .ok()
        .and_then(|g| g.as_ref().map(|r| r.track_options(&track_type)))
        .unwrap_or_default()
}

#[tauri::command]
fn player_destroy(state: State<DesktopState>) -> bool {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    if let Some(surface) = state.native_player_surface.lock().unwrap().as_ref() {
        surface.hide();
        return state.player_renderer.lock().unwrap().is_some();
    }

    state.player_renderer.lock().unwrap().take().is_some()
}

#[tauri::command]
fn player_set_chapters(state: State<DesktopState>, chapters_json: String) {
    *state.chapters_json.lock().unwrap() =
        if chapters_json.trim().is_empty() || chapters_json == "[]" {
            None
        } else {
            Some(chapters_json)
        };
}

#[tauri::command]
fn player_clear_chapters(state: State<DesktopState>) {
    *state.chapters_json.lock().unwrap() = None;
}

#[tauri::command]
fn player_set_skip_info(
    state: State<DesktopState>,
    segments_json: String,
    next_ep_subtitle: Option<String>,
    next_ep_threshold_percent: Option<f64>,
    auto_play_next_episode: Option<bool>,
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
}

#[tauri::command]
fn player_clear_skip_info(state: State<DesktopState>) {
    *state.skip_segments_json.lock().unwrap() = None;
    *state.next_ep_subtitle.lock().unwrap() = String::new();
    *state.eof_next_fired.lock().unwrap() = false;
}

#[tauri::command]
fn player_set_episodes(state: State<DesktopState>, episodes_json: String) {
    *state.episodes_json.lock().unwrap() =
        if episodes_json.trim() == "[]" || episodes_json.trim().is_empty() {
            None
        } else {
            Some(episodes_json)
        };
}

#[tauri::command]
fn player_clear_episodes(state: State<DesktopState>) {
    *state.episodes_json.lock().unwrap() = None;
}

#[tauri::command]
fn core_capabilities() -> String {
    FluxaCore::core_capabilities_json(true)
}

// ── OAuth compile-time credentials ────────────────────────────────────────────
// Values are injected at build time via environment variables.
// Set them in .env (local dev) or GitHub Actions secrets (CI releases).

const TRAKT_CLIENT_ID: &str = env!("FLUXA_TRAKT_CLIENT_ID");
const TRAKT_CLIENT_SECRET: &str = env!("FLUXA_TRAKT_CLIENT_SECRET");
const MAL_CLIENT_ID: &str = env!("FLUXA_MAL_CLIENT_ID");
const SIMKL_CLIENT_ID: &str = env!("FLUXA_SIMKL_CLIENT_ID");
const SIMKL_CLIENT_SECRET: &str = env!("FLUXA_SIMKL_CLIENT_SECRET");

#[tauri::command]
fn get_oauth_client_id(service: &str) -> &'static str {
    match service {
        "trakt" => TRAKT_CLIENT_ID,
        "mal" => MAL_CLIENT_ID,
        "simkl" => SIMKL_CLIENT_ID,
        _ => "",
    }
}

// ── Trakt OAuth ───────────────────────────────────────────────────────────────

#[tauri::command]
fn trakt_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("Fluxa Desktop/1.0")
        .default_headers({
            let mut h = reqwest::header::HeaderMap::new();
            h.insert("Content-Type", "application/json".parse().unwrap());
            h.insert("trakt-api-version", "2".parse().unwrap());
            h.insert("trakt-api-key", TRAKT_CLIENT_ID.parse().unwrap());
            h
        })
        .build()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn trakt_device_start() -> Result<String, String> {
    let res = trakt_client()?
        .post("https://api.trakt.tv/oauth/device/code")
        .json(&json!({ "client_id": TRAKT_CLIENT_ID }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("Trakt device code request failed: HTTP {status}"));
    }
    Ok(text)
}

#[tauri::command]
async fn trakt_device_poll(device_code: String) -> Result<String, String> {
    let res = trakt_client()?
        .post("https://api.trakt.tv/oauth/device/token")
        .json(&json!({ "code": device_code, "client_id": TRAKT_CLIENT_ID }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    match res.status().as_u16() {
        200 => Ok(res.text().await.map_err(|e| e.to_string())?),
        400 | 429 => Ok("pending".to_string()),
        _ => Err("expired".to_string()),
    }
}

#[tauri::command]
async fn trakt_oauth_exchange(code: String) -> Result<String, String> {
    let res = trakt_client()?
        .post("https://api.trakt.tv/oauth/token")
        .json(&serde_json::json!({
            "code": code,
            "client_id": TRAKT_CLIENT_ID,
            "client_secret": TRAKT_CLIENT_SECRET,
            "redirect_uri": "fluxa://oauth/trakt",
            "grant_type": "authorization_code",
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("Trakt token exchange failed: HTTP {status}: {text}"));
    }
    Ok(text)
}

// ── MAL / SIMKL OAuth ─────────────────────────────────────────────────────────

fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char);
            }
            b => {
                encoded.push('%');
                encoded.push(
                    char::from_digit((b >> 4) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
                encoded.push(
                    char::from_digit((b & 0xf) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
            }
        }
    }
    encoded
}

#[tauri::command]
async fn mal_oauth_exchange(code: String, code_verifier: String) -> Result<String, String> {
    let client_id = MAL_CLIENT_ID;
    let redirect_uri = "fluxa://oauth/mal";
    let body = format!(
        "client_id={}&grant_type=authorization_code&code={}&redirect_uri={}&code_verifier={}",
        url_encode(&client_id),
        url_encode(&code),
        url_encode(redirect_uri),
        url_encode(&code_verifier),
    );
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?
        .post("https://myanimelist.net/v1/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("MAL token exchange failed: HTTP {status}: {text}"));
    }
    Ok(text)
}

#[tauri::command]
async fn simkl_oauth_exchange(code: String) -> Result<String, String> {
    let body = serde_json::json!({
        "code": code,
        "client_id": SIMKL_CLIENT_ID,
        "client_secret": SIMKL_CLIENT_SECRET,
        "redirect_uri": "fluxa://oauth/simkl",
        "grant_type": "authorization_code",
    });
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?
        .post("https://api.simkl.com/oauth/token")
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let text = response.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("SIMKL token exchange failed: HTTP {status}: {text}"));
    }
    Ok(text)
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
fn list_offline_downloads(state: State<DesktopState>) -> Vec<Value> {
    let data_dir = match state.data_dir.lock().unwrap().clone() {
        Some(d) => d,
        None => return vec![],
    };
    let offline_dir = data_dir.join("offline");
    if !offline_dir.exists() {
        return vec![];
    }
    let entries = match fs::read_dir(&offline_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    let mut items: Vec<Value> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("part") {
                return None;
            }
            let metadata = fs::metadata(&path).ok()?;
            let size = metadata.len();
            let name = path.file_name()?.to_string_lossy().to_string();
            Some(json!({
                "id": name.clone(),
                "videoFileName": name,
                "path": path.to_string_lossy().to_string(),
                "sizeBytes": size,
                "status": "downloaded",
            }))
        })
        .collect();
    items.sort_by(|a, b| {
        a["videoFileName"]
            .as_str()
            .unwrap_or("")
            .cmp(b["videoFileName"].as_str().unwrap_or(""))
    });
    items
}

#[tauri::command]
fn delete_offline_download(state: State<DesktopState>, file_name: String) -> Result<(), String> {
    let data_dir = state
        .data_dir
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "app data dir not ready".to_string())?;
    let safe_name = sanitize_file_name(&file_name);
    let path = data_dir.join("offline").join(&safe_name);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Core: content identity extras ────────────────────────────────────────────

#[tauri::command]
fn core_parse_video_id(id: String) -> String {
    FluxaCore::parse_video_id_json(&id)
}

#[tauri::command]
fn core_build_trakt_ids(video_id: String) -> Option<String> {
    FluxaCore::build_trakt_ids_json(&video_id)
}

// ── Core: calendar extras ─────────────────────────────────────────────────────

#[tauri::command]
fn core_calendar_items_from_meta(meta_json: String, month_prefix: String) -> Option<String> {
    FluxaCore::calendar_items_from_meta_json(&meta_json, &month_prefix)
}

#[tauri::command]
fn core_calendar_item_matches_month(item_json: String, month_prefix: String) -> bool {
    FluxaCore::calendar_item_matches_month_json(&item_json, &month_prefix)
}

// ── Core: Trakt high-level ────────────────────────────────────────────────────

#[tauri::command]
fn core_trakt_playback_items_to_library(items_json: String) -> Option<String> {
    FluxaCore::trakt_playback_items_to_library_json(&items_json)
}

#[tauri::command]
fn core_trakt_watchlist_to_items(movies_json: String, shows_json: String) -> Option<String> {
    FluxaCore::trakt_watchlist_to_items_json(&movies_json, &shows_json)
}

#[tauri::command]
fn core_trakt_watched_to_ids(movies_json: String, shows_json: String) -> Option<String> {
    FluxaCore::trakt_watched_to_ids_json(&movies_json, &shows_json)
}

#[tauri::command]
fn core_merge_external_watchlist(local_json: String, external_json: String) -> String {
    FluxaCore::merge_external_watchlist_json(&local_json, &external_json)
}

#[tauri::command]
fn core_merge_external_watched(local_json: String, external_json: String) -> String {
    FluxaCore::merge_external_watched_json(&local_json, &external_json)
}

#[tauri::command]
fn core_merge_continue_watching_lists(
    local_json: String,
    external_json: String,
    progress_json: String,
) -> Option<String> {
    FluxaCore::merge_continue_watching_lists_json(&local_json, &external_json, &progress_json)
}

#[tauri::command]
fn core_remember_last_watched_episodes(lib_json: String, watched_ids_json: String) -> String {
    FluxaCore::remember_last_watched_episodes_json(&lib_json, &watched_ids_json)
}

// ── Core: Simkl ───────────────────────────────────────────────────────────────

#[tauri::command]
fn core_simkl_watching_to_items(shows_json: String, movies_json: String) -> Option<String> {
    FluxaCore::simkl_watching_to_items_json(&shows_json, &movies_json)
}

#[tauri::command]
fn core_simkl_watchlist_to_items(shows_json: String, movies_json: String) -> Option<String> {
    FluxaCore::simkl_watchlist_to_items_json(&shows_json, &movies_json)
}

#[tauri::command]
fn core_simkl_watched_to_ids(shows_json: String, movies_json: String) -> Option<String> {
    FluxaCore::simkl_watched_to_ids_json(&shows_json, &movies_json)
}

// ── Core: library_state extras ────────────────────────────────────────────────

#[tauri::command]
fn core_normalize_library_document(json: String) -> String {
    FluxaCore::normalize_library_document_json(&json)
}

#[tauri::command]
fn core_is_up_next_continue_watching_item(item_json: String) -> bool {
    FluxaCore::is_up_next_continue_watching_item_json(&item_json)
}

#[tauri::command]
fn core_build_continue_watching_from_progress(progress_json: String) -> Option<String> {
    FluxaCore::build_continue_watching_from_progress_json(&progress_json)
}

#[tauri::command]
fn core_compute_continue_watching_badges(
    candidates_json: String,
    videos_by_series_json: String,
    last_watched_json: String,
    now_ms: i64,
) -> Option<String> {
    FluxaCore::compute_continue_watching_badges_json(
        &candidates_json,
        &videos_by_series_json,
        &last_watched_json,
        now_ms,
    )
}

// ── Core: TMDB ────────────────────────────────────────────────────────────────

#[tauri::command]
fn core_tmdb_content_type(content_type: String) -> String {
    FluxaCore::tmdb_content_type(&content_type).to_string()
}

#[tauri::command]
fn core_tmdb_language(language: String) -> String {
    FluxaCore::tmdb_language(&language)
}

#[tauri::command]
fn core_tmdb_image_url(path: Option<String>, size: String) -> Option<String> {
    FluxaCore::tmdb_image_url(path.as_deref(), &size)
}

#[tauri::command]
fn core_tmdb_meta_to_meta(item_json: String, requested_type: String, language: String) -> Option<String> {
    FluxaCore::tmdb_meta_to_meta_json(&item_json, &requested_type, &language)
}

#[tauri::command]
fn core_tmdb_video_to_trailer(video_json: String) -> Option<String> {
    FluxaCore::tmdb_video_to_trailer_json(&video_json)
}

#[tauri::command]
fn core_tmdb_bulk_metas(items_json: String, requested_type: String, language: String) -> Option<String> {
    FluxaCore::tmdb_bulk_metas_to_metas_json(&items_json, &requested_type, &language)
}

#[tauri::command]
fn core_tmdb_bulk_videos_to_trailers(items_json: String) -> Option<String> {
    FluxaCore::tmdb_bulk_videos_to_trailers_json(&items_json)
}

#[tauri::command]
fn core_tmdb_resolve_id_hint(content_id: String) -> (String, bool) {
    FluxaCore::tmdb_resolve_id_hint(&content_id)
}

// ── Core: intro segments ──────────────────────────────────────────────────────

#[tauri::command]
fn core_parse_intro_db_segments(data_json: String) -> Option<String> {
    FluxaCore::parse_intro_db_segments_json(&data_json)
}

#[tauri::command]
fn core_parse_aniskip_results(results_json: String) -> Option<String> {
    FluxaCore::parse_aniskip_results_json(&results_json)
}

#[tauri::command]
fn core_unique_intro_segments(segments_a_json: String, segments_b_json: String) -> Option<String> {
    FluxaCore::unique_intro_segments_json(&segments_a_json, &segments_b_json)
}

#[tauri::command]
fn core_merge_intro_segments(sources_json: String) -> Option<String> {
    FluxaCore::merge_intro_segments_json(&sources_json)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn ensure_native_player_surface(
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
fn ensure_native_player_surface(
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
fn ensure_native_player_surface(
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

fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn sanitize_file_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('.').trim_matches('_');
    if trimmed.is_empty() {
        "download.mp4".to_string()
    } else {
        trimmed.chars().take(180).collect()
    }
}

fn write_file_atomic(path: &PathBuf, bytes: &[u8]) -> Result<(), String> {
    let tmp_path = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp_path).map_err(|error| error.to_string())?;
        file.write_all(bytes).map_err(|error| error.to_string())?;
        file.flush().map_err(|error| error.to_string())?;
    }
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&tmp_path, path).map_err(|error| error.to_string())
}

// In-memory cache for artwork image bytes. Avoids re-downloading on repeated opens
// of the same show. Capped at 40 entries (~20 shows × 2 images).
static ARTWORK_CACHE: OnceLock<Mutex<HashMap<String, Vec<u8>>>> = OnceLock::new();
fn artwork_cache() -> &'static Mutex<HashMap<String, Vec<u8>>> {
    ARTWORK_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// Shared HTTP client for artwork fetches — reuses TLS connections and avoids the
// overhead of creating a new client (fresh TLS handshake) on every artwork download.
static ARTWORK_HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
fn artwork_http_client() -> &'static reqwest::Client {
    ARTWORK_HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("artwork HTTP client")
    })
}

// Pre-decoded, pre-scaled RGBA pixels — populated by player_prefetch_artwork so that
// player_set_loading_artwork can skip both network and decode when the user clicks play.
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
static ARTWORK_BG_DECODED: OnceLock<Mutex<HashMap<String, (Vec<u8>, i32, i32)>>> = OnceLock::new();
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn artwork_bg_decoded() -> &'static Mutex<HashMap<String, (Vec<u8>, i32, i32)>> {
    ARTWORK_BG_DECODED.get_or_init(|| Mutex::new(HashMap::new()))
}
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
static ARTWORK_LOGO_DECODED: OnceLock<Mutex<HashMap<String, (Vec<u8>, i32, i32)>>> = OnceLock::new();
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn artwork_logo_decoded() -> &'static Mutex<HashMap<String, (Vec<u8>, i32, i32)>> {
    ARTWORK_LOGO_DECODED.get_or_init(|| Mutex::new(HashMap::new()))
}

fn normalize_url(url: &str) -> String {
    if let Some(rest) = url.trim().strip_prefix("//") {
        format!("https:{rest}")
    } else {
        url.trim().to_string()
    }
}

async fn fetch_player_artwork_bytes(url: Option<&str>) -> Option<Vec<u8>> {
    let url = url?.trim();
    if url.is_empty() {
        return None;
    }
    let normalized = if let Some(protocol_relative) = url.strip_prefix("//") {
        format!("https:{protocol_relative}")
    } else {
        url.to_string()
    };

    // Cache hit — skip network entirely
    if let Ok(cache) = artwork_cache().lock() {
        if let Some(cached) = cache.get(&normalized) {
            return Some(cached.clone());
        }
    }

    let response = artwork_http_client().get(&normalized).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    if bytes.len() > 8 * 1024 * 1024 {
        return None;
    }
    let vec = bytes.to_vec();

    // Populate cache, evicting one entry when full
    if let Ok(mut cache) = artwork_cache().lock() {
        if cache.len() >= 40 {
            if let Some(key) = cache.keys().next().cloned() {
                cache.remove(&key);
            }
        }
        cache.insert(normalized, vec.clone());
    }

    Some(vec)
}

/// Pre-warms the artwork caches for the given URLs without updating the player surface.
/// Fetches, decodes, and scales both images so that player_set_loading_artwork can show
/// them instantly (cache lookup only) when the user clicks play.
#[tauri::command]
async fn player_prefetch_artwork(background_url: Option<String>, logo_url: Option<String>) {
    let bg_key = background_url.as_deref().map(normalize_url);
    let logo_key = logo_url.as_deref().map(normalize_url);
    let bg = tauri::async_runtime::spawn(fetch_player_artwork_bytes_owned(background_url));
    let logo = tauri::async_runtime::spawn(fetch_player_artwork_bytes_owned(logo_url));
    let bg_bytes = bg.await.unwrap_or(None);
    let logo_bytes = logo.await.unwrap_or(None);

    // Decode and scale on a blocking thread, then store the result so
    // player_set_loading_artwork can skip both network and decode entirely.
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

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        std::env::remove_var("WAYLAND_DISPLAY");
        std::env::remove_var("MOZ_ENABLE_WAYLAND");
        std::env::set_var("GDK_BACKEND", "x11");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_libmpv::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            for arg in &args {
                if !arg.starts_with("fluxa://") { continue; }
                let code = arg.split('?').nth(1)
                    .and_then(|q| q.split('&').find(|p| p.starts_with("code=")))
                    .map(|p| p.trim_start_matches("code=").to_string());
                if let Some(code) = code {
                    let evt = if arg.contains("/trakt") { "trakt-oauth-code" }
                        else if arg.contains("/mal") { "mal-oauth-code" }
                        else if arg.contains("/simkl") { "simkl-oauth-code" }
                        else { continue };
                    let _ = app.emit(evt, code);
                }
            }
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_deep_link::init())
        .manage(DesktopState::default())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir")
                .join("fluxa");

            let state = app.state::<DesktopState>();
            *state.data_dir.lock().unwrap() = Some(data_dir.clone());
            let _ = fs::create_dir_all(&data_dir);

            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    let s = url.as_str();
                    let code = url
                        .query_pairs()
                        .find(|(k, _)| k == "code")
                        .map(|(_, v)| v.into_owned());
                    if let Some(code) = code {
                        let evt = if s.contains("/trakt") {
                            "trakt-oauth-code"
                        } else if s.contains("/mal") {
                            "mal-oauth-code"
                        } else if s.contains("/simkl") {
                            "simkl-oauth-code"
                        } else {
                            continue;
                        };
                        let _ = handle.emit(evt, code);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            engine_init,
            engine_dispatch,
            engine_complete_effect,
            engine_snapshot,
            http_fetch_text,
            storage_read,
            storage_write,
            storage_delete,
            core_normalize_manifest_url,
            core_manifest_fetch_plan,
            core_parse_manifest,
            core_resolve_manifest_assets,
            core_merge_live_manifest,
            core_build_resource_url,
            core_supports_resource,
            core_catalog_supports_extra,
            core_catalog_requires_extra,
            core_catalog_has_required_extra_except,
            core_parse_addon_resource_result,
            core_addon_resource_request_plan,
            core_resource_fetch_plan,
            core_resource_parse_plan,
            core_playback_prepare_plan,
            core_library_local_state_plan,
            core_preferences_schema,
            core_apply_preference_update,
            core_addon_collection_mutation_plan,
            core_detail_episode_plan,
            core_normalize_addon_subtitles,
            core_stream_playback_info,
            core_torrent_runtime_info,
            core_search_result_grouping,
            core_build_metadata_feed_options,
            core_discover_catalog_options,
            core_discover_sort_plan,
            core_library_sort_plan,
            core_watchlist_toggle_plan,
            core_playback_progress_merge_plan,
            core_library_continue_watching_items,
            core_detail_series_lookup_id,
            core_detail_season_load_plan,
            core_player_backend_selection,
            core_player_buffer_targets,
            core_offline_download_plan,
            core_playback_intro_lookup_content_id,
            core_player_source_sidebar_plan,
            core_player_retry_policy,
            core_effective_metadata_feed_selection,
            core_toggle_metadata_feed_limited,
            core_find_preferred_subtitle_index,
            start_torrent_stream,
            stop_torrent_stream,
            player_init,
            player_apply_preferences,
            player_load,
            player_render_frame,
            player_command,
            player_show_loading,
            player_hide,
            player_set_title,
            player_set_loading_artwork,
            player_prefetch_artwork,
            enqueue_offline_download,
            player_add_subtitle,
            player_title,
            player_status,
            player_destroy,
            player_get_playback_info,
            player_track_options,
            player_set_seek_thumbnail_enabled,
            player_set_chapters,
            player_clear_chapters,
            player_set_skip_info,
            player_clear_skip_info,
            player_set_episodes,
            player_clear_episodes,
            core_capabilities,
            get_oauth_client_id,
            trakt_device_start,
            trakt_device_poll,
            trakt_oauth_exchange,
            mal_oauth_exchange,
            simkl_oauth_exchange,
            get_data_dir,
            list_offline_downloads,
            delete_offline_download,
            core_parse_video_id,
            core_build_trakt_ids,
            core_calendar_items_from_meta,
            core_calendar_item_matches_month,
            core_trakt_playback_items_to_library,
            core_trakt_watchlist_to_items,
            core_trakt_watched_to_ids,
            core_merge_external_watchlist,
            core_merge_external_watched,
            core_merge_continue_watching_lists,
            core_remember_last_watched_episodes,
            core_simkl_watching_to_items,
            core_simkl_watchlist_to_items,
            core_simkl_watched_to_ids,
            core_normalize_library_document,
            core_is_up_next_continue_watching_item,
            core_build_continue_watching_from_progress,
            core_compute_continue_watching_badges,
            core_tmdb_content_type,
            core_tmdb_language,
            core_tmdb_image_url,
            core_tmdb_meta_to_meta,
            core_tmdb_video_to_trailer,
            core_tmdb_bulk_metas,
            core_tmdb_bulk_videos_to_trailers,
            core_tmdb_resolve_id_hint,
            core_parse_intro_db_segments,
            core_parse_aniskip_results,
            core_unique_intro_segments,
            core_merge_intro_segments,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Fluxa Desktop");
}
