mod airplay;
mod artwork;
mod cast;
mod cast_proxy;
mod chromecast;
mod fcast;
mod core_commands;
mod custom_fonts;
mod diagnostics;
mod discord_presence;
mod downloads;
mod external_player;
mod libvlc_render;
#[cfg(target_os = "linux")]
mod linux_player_surface;
#[cfg(target_os = "linux")]
mod linux_vulkan;
#[cfg(target_os = "linux")]
mod linux_wayland_subsurface;
mod local_media;
#[cfg(target_os = "macos")]
mod macos_player_surface;
#[cfg(target_os = "macos")]
mod macos_vulkan;
mod mpv_render;
mod net_guard;
mod oauth;
mod oauth_callbacks;
mod playback_engine;
mod player_surface_events;
mod player;
mod plugin_executor;
mod poster_cache;
mod roku;
mod sleep_inhibitor;
mod storage;
mod stream_proxy;
mod torrent_stream;
mod torrent_transport;
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
use fcast::*;
use core_commands::*;
use custom_fonts::*;
use discord_presence::*;
use downloads::*;
use external_player::*;
use local_media::*;
use oauth::*;
use oauth_callbacks::{PendingOAuthCallbacks, queue_oauth_callback, take_oauth_callback};
use player::*;
use poster_cache::*;
use roku::*;
use storage::*;
pub(crate) use torrent_stream::resolve_torrent_download_url;
use torrent_stream::{
    player_torrent_stats, player_torrent_telemetry, start_torrent_stream, stop_torrent_stream,
    stream_magnet_link,
};

use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64};
use tauri::{Emitter, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;

pub struct PlayerOverlayState {
    pub chapters_json: Option<String>,
    pub skip_segments_json: Option<String>,
    pub next_ep_subtitle: String,
    pub next_ep_threshold_percent: f64,
    pub auto_play_next_episode: bool,
    pub auto_play_countdown_secs: u32,
    pub auto_skip_segments: bool,
    pub use_chapter_skip: bool,
    pub eof_next_fired: bool,
    pub episodes_json: Option<String>,
    pub anime4k_enabled: bool,
}

impl Default for PlayerOverlayState {
    fn default() -> Self {
        Self {
            chapters_json: None,
            skip_segments_json: None,
            next_ep_subtitle: String::new(),
            next_ep_threshold_percent: 85.0,
            auto_play_next_episode: true,
            auto_play_countdown_secs: 7,
            auto_skip_segments: false,
            use_chapter_skip: true,
            eof_next_fired: false,
            episodes_json: None,
            anime4k_enabled: false,
        }
    }
}

#[derive(Default)]
pub struct ThumbnailRuntimeState {
    pub enabled: bool,
    pub url: Option<String>,
    pub renderer: Option<mpv_render::MpvThumbnailRenderer>,
    pub loaded_url: Option<String>,
}

impl PlayerOverlayState {
    pub fn take_eof_next(&mut self) -> bool {
        if self.eof_next_fired {
            return false;
        }
        self.eof_next_fired = true;
        true
    }
}

#[derive(Default)]
pub struct TorrentRuntimeState {
    pub server_base_url: Option<String>,
    pub stream_link: Option<String>,
    pub stream_file_id: Option<usize>,
    pub generation: Option<u64>,
    pub telemetry_generation: u64,
}

pub struct DesktopState {
    pub engine_handle: Mutex<Option<u64>>,
    pub data_dir: Mutex<Option<PathBuf>>,
    /// Serializes storage migration and key creation. SQLite coordinates processes,
    /// but the legacy encryption key is a separate file.
    pub storage_lock: Mutex<()>,
    pub download_dir: Mutex<Option<PathBuf>>,
    pub player_mpv_client: Mutex<Option<mpv_render::MpvClientHandle>>,
    pub player_render_state: Mutex<Option<mpv_render::MpvRenderState>>,
    pub player_renderer_vlc: Mutex<Option<libvlc_render::LibvlcPlayer>>,
    pub active_player_engine: Mutex<playback_engine::PlayerEngine>,
    #[cfg(target_os = "linux")]
    pub native_player_surface: Mutex<Option<linux_player_surface::NativePlayerSurface>>,
    #[cfg(target_os = "windows")]
    pub native_player_surface: Mutex<Option<windows_player_surface::NativePlayerSurface>>,
    #[cfg(target_os = "macos")]
    pub native_player_surface: Mutex<Option<macos_player_surface::NativePlayerSurface>>,
    pub player_overlay: Mutex<PlayerOverlayState>,
    pub thumbnail: Mutex<ThumbnailRuntimeState>,
    pub pending_hide: AtomicBool,
    pub player_telemetry_running: AtomicBool,
    pub player_stats_enabled: AtomicBool,
    pub player_position_interval_ms: AtomicU64,
    #[cfg(target_os = "windows")]
    pub main_window_size: std::sync::atomic::AtomicU64,
    pub downloads: downloads::DownloadsState,
    pub pending_stream_headers: Mutex<Vec<(String, String)>>,
    pub torrent: Mutex<TorrentRuntimeState>,
    pub close_flush_done: AtomicBool,
    pub sleep_inhibitor: Mutex<sleep_inhibitor::SleepInhibitor>,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            engine_handle: Mutex::new(None),
            data_dir: Mutex::new(None),
            storage_lock: Mutex::new(()),
            download_dir: Mutex::new(None),
            player_mpv_client: Mutex::new(None),
            player_render_state: Mutex::new(None),
            player_renderer_vlc: Mutex::new(None),
            active_player_engine: Mutex::new(playback_engine::PlayerEngine::Mpv),
            #[cfg(target_os = "linux")]
            native_player_surface: Mutex::new(None),
            #[cfg(target_os = "windows")]
            native_player_surface: Mutex::new(None),
            #[cfg(target_os = "macos")]
            native_player_surface: Mutex::new(None),
            player_overlay: Mutex::new(PlayerOverlayState::default()),
            thumbnail: Mutex::new(ThumbnailRuntimeState::default()),
            pending_hide: AtomicBool::new(false),
            player_telemetry_running: AtomicBool::new(false),
            player_stats_enabled: AtomicBool::new(false),
            player_position_interval_ms: AtomicU64::new(750),
            #[cfg(target_os = "windows")]
            main_window_size: std::sync::atomic::AtomicU64::new(0),
            downloads: downloads::DownloadsState::default(),
            pending_stream_headers: Mutex::new(Vec::new()),
            torrent: Mutex::new(TorrentRuntimeState::default()),
            close_flush_done: AtomicBool::new(false),
            sleep_inhibitor: Mutex::new(sleep_inhibitor::SleepInhibitor::default()),
        }
    }
}

pub(crate) static DIAGNOSTIC_MODE: AtomicBool = AtomicBool::new(false);
static SENTRY_GUARD: Mutex<Option<sentry::ClientInitGuard>> = Mutex::new(None);
const COMPANION_PORT: u16 = 19876;

fn start_companion_server() {
    tauri::async_runtime::spawn(async {
        match fluxa_streaming_engine::companion_server::serve(COMPANION_PORT).await {
            Ok(()) => log::info!("companion server stopped"),
            Err(error) => log::warn!("companion server unavailable on 127.0.0.1:{COMPANION_PORT}: {error}"),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::PlayerOverlayState;

    #[test]
    fn eof_next_is_consumed_once_per_playback() {
        let mut overlay = PlayerOverlayState::default();
        assert!(overlay.take_eof_next());
        assert!(!overlay.take_eof_next());
        overlay.eof_next_fired = false;
        assert!(overlay.take_eof_next());
    }
}

pub(crate) fn sentry_dsn() -> Option<sentry::types::Dsn> {
    "https://7fe8e82cf7ea0eed65175d3d43afb1c0@o4511704565678080.ingest.de.sentry.io/4511706871693392"
        .parse()
        .ok()
}

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
    } else {
        log::debug!("[app] {msg}");
    }
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
        if !cfg!(debug_assertions) {
            let mut guard = SENTRY_GUARD.lock().unwrap();
            if guard.is_none() {
                let mut options = sentry::ClientOptions::default();
                options.dsn = sentry_dsn();
                options.release = sentry::release_name!();
                *guard = Some(sentry::init(options));
            }
        }
    } else if !enabled && was {
        log::warn!("diagnostic mode disabled");
        SENTRY_GUARD.lock().unwrap().take();
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
async fn register_trailer_proxy_url(
    proxy_state: State<'_, trailer_proxy::TrailerProxyState>,
    url: String,
) -> Result<String, String> {
    trailer_proxy::register(&proxy_state, url).await
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

#[tauri::command]
fn in_app_updates_supported() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("APPIMAGE").is_some()
    }
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
                .level_for("h2", log::LevelFilter::Warn)
                .level_for("hyper", log::LevelFilter::Warn)
                .level_for("hyper_util", log::LevelFilter::Warn)
                .level_for("reqwest", log::LevelFilter::Warn)
                .level_for("rustls", log::LevelFilter::Warn)
                .targets({
                    let mut targets = vec![tauri_plugin_log::Target::new(
                        tauri_plugin_log::TargetKind::LogDir { file_name: None },
                    )];
                    if cfg!(debug_assertions) {
                        targets.push(tauri_plugin_log::Target::new(
                            tauri_plugin_log::TargetKind::Stdout,
                        ));
                    }
                    targets
                })
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
        .manage(fcast::FcastState::default())
        .manage(airplay::AirplayState::default())
        .manage(roku::RokuState::default())
        .manage(cast_proxy::CastProxyState::default())
        .manage(trailer_proxy::TrailerProxyState::default())
        .manage(stream_proxy::StreamProxyState::default())
        .setup(|app| {
            diagnostics::set_app_handle(app.handle().clone());
            start_companion_server();

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
            storage::initialize_storage(&data_dir).map_err(std::io::Error::other)?;

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
            library_progress_upsert_many,
            library_snapshot,
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
            local_media_scan,
            run_plugin_scraper,
            stream_magnet_link,
            start_torrent_stream,
            stop_torrent_stream,
            register_trailer_proxy_url,
            player_last_stream_error,
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
            player_set_status_interval,
            player_set_stats_enabled,
            player_set_sleep_inhibition,
            player_destroy,
            player_get_playback_info,
            player_track_options,
            player_set_seek_thumbnail_enabled,
            player_hdr_supported,
            external_player_options,
            external_player_launch,
            external_player_status,
            external_player_stop,
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
            in_app_updates_supported,
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
            fcast_discover_devices,
            fcast_connect,
            fcast_play,
            fcast_pause,
            fcast_seek,
            fcast_set_volume,
            fcast_set_speed,
            fcast_stop,
            fcast_disconnect,
            roku_discover_devices,
            roku_set_media,
            roku_play_pause,
            roku_disconnect,
            cast_proxy_serve,
            player_torrent_stats,
            player_torrent_telemetry,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Fluxa Desktop")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let state = app_handle.state::<DesktopState>();
                #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
                if let Some(surface) = state.native_player_surface.lock().unwrap().take() {
                    #[cfg(target_os = "macos")]
                    let _ = surface.shutdown();
                    #[cfg(not(target_os = "macos"))]
                    surface.hide();
                }
                fluxa_streaming_engine::stop_torrent_server(None);
            }
        });
}
