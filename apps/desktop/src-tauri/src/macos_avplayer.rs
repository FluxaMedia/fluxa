use crate::mpv_render::{PlayerStatus, PlayerTrackOption};
use crate::player_surface::{Artwork, PlayerSurface};
use std::ffi::{CStr, CString, c_char, c_void};
use tauri::{AppHandle, Manager};

#[link(name = "FluxaDesktopPlayer", kind = "static")]
unsafe extern "C" {
    fn fluxa_desktop_avplayer_create(parent: *mut c_void, width: f64, height: f64) -> *mut c_void;
    fn fluxa_desktop_avplayer_destroy(handle: *mut c_void);
    fn fluxa_desktop_avplayer_load(
        handle: *mut c_void,
        url: *const c_char,
        title: *const c_char,
        start: f64,
    ) -> *mut c_char;
    fn fluxa_desktop_avplayer_command(handle: *mut c_void, command: *const c_char) -> *mut c_char;
    fn fluxa_desktop_avplayer_add_subtitle(handle: *mut c_void, url: *const c_char) -> *mut c_char;
    fn fluxa_desktop_avplayer_hide(handle: *mut c_void);
    fn fluxa_desktop_avplayer_position(handle: *mut c_void) -> f64;
    fn fluxa_desktop_avplayer_duration(handle: *mut c_void) -> f64;
    fn fluxa_desktop_avplayer_phase(handle: *mut c_void) -> i32;
    fn fluxa_desktop_avplayer_tracks_json(handle: *mut c_void) -> *mut c_char;
    fn fluxa_desktop_avplayer_free_string(value: *mut c_char);
}

#[derive(Clone)]
pub struct NativePlayerSurface {
    handle: usize,
    app: AppHandle,
}

unsafe impl Send for NativePlayerSurface {}
unsafe impl Sync for NativePlayerSurface {}

impl NativePlayerSurface {
    fn handle(&self) -> *mut c_void {
        self.handle as *mut c_void
    }

    fn call_string(&self, value: *mut c_char) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }
        let message = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() };
        unsafe { fluxa_desktop_avplayer_free_string(value) };
        Err(message)
    }

    fn call_command(&self, command: &str) -> Result<(), String> {
        let command =
            CString::new(command).map_err(|_| "AVPlayer command contains NUL".to_string())?;
        self.call_string(unsafe { fluxa_desktop_avplayer_command(self.handle(), command.as_ptr()) })
    }
}

impl PlayerSurface for NativePlayerSurface {
    fn backend_name(&self) -> &'static str {
        "avplayer"
    }

    fn load(
        &self,
        url: String,
        start_at: Option<u64>,
        _total_duration: Option<u64>,
    ) -> Result<(), String> {
        let url = CString::new(url).map_err(|_| "AVPlayer URL contains NUL".to_string())?;
        let title = CString::new("Fluxa").expect("static title");
        let result = self.call_string(unsafe {
            fluxa_desktop_avplayer_load(
                self.handle(),
                url.as_ptr(),
                title.as_ptr(),
                start_at.unwrap_or(0) as f64,
            )
        });
        if result.is_ok() {
            let _ = self.app.emit("native-player-show", ());
        }
        result
    }

    fn hide(&self) {
        unsafe { fluxa_desktop_avplayer_hide(self.handle()) };
        let _ = self.app.emit("native-player-hide", ());
    }

    fn shutdown(&self) -> Result<(), String> {
        unsafe { fluxa_desktop_avplayer_destroy(self.handle()) };
        Ok(())
    }

    fn show_loading(&self, _title: String, _episode_title: Option<String>) {}
    fn set_title(&self, _title: String, _episode_title: Option<String>) {}
    fn set_artwork(
        &self,
        _title: String,
        _episode_title: Option<String>,
        _background: Artwork,
        _logo: Artwork,
    ) {
    }
    fn set_cursor_visible(&self, _visible: bool) {}

    fn command(&self, command: String) -> Result<(), String> {
        self.call_command(&command)
    }
    fn command_args(&self, _commands: Vec<Vec<String>>) -> Result<(), String> {
        Ok(())
    }

    fn status(&self) -> Result<PlayerStatus, String> {
        let phase = unsafe { fluxa_desktop_avplayer_phase(self.handle()) };
        let position = unsafe { fluxa_desktop_avplayer_position(self.handle()) };
        let duration = unsafe { fluxa_desktop_avplayer_duration(self.handle()) };
        let loaded = phase != 0;
        let paused = phase != 2;
        Ok(PlayerStatus {
            loaded,
            path: None,
            media_title: None,
            time_pos: loaded.then(|| position.to_string()),
            duration: (duration > 0).then(|| duration.to_string()),
            percent_pos: (duration > 0)
                .then(|| (position / duration * 100.0).clamp(0.0, 100.0).to_string()),
            pause: loaded.then(|| if paused { "yes".into() } else { "no".into() }),
            mute: Some("no".into()),
            volume: Some("100".into()),
            core_idle: Some(if loaded { "no" } else { "yes" }.into()),
            eof_reached: Some(if phase == 4 { "yes" } else { "no" }.into()),
            vo_configured: loaded.then(|| "yes".into()),
            video_codec: None,
            video_format: None,
            width: None,
            height: None,
            cache_speed: None,
            demuxer_cache_duration: None,
            hwdec_current: Some("VideoToolbox".into()),
            fps: None,
            frame_drop_count: None,
            decoder_frame_drop_count: None,
            avsync: None,
            video_bitrate: None,
            audio_bitrate: None,
            audio_codec: None,
            audio_samplerate: None,
            audio_channels: None,
            audio_output_mode: Some("AVFoundation".into()),
            color_primaries: None,
            color_matrix: None,
            color_gamma: None,
            video_out_primaries: None,
            video_out_matrix: None,
            video_out_gamma: None,
            sig_peak: None,
            hdr_active: false,
            container_fps: None,
            display_fps: None,
            mistimed_frame_count: None,
            vo_delayed_frame_count: None,
            paused_for_cache: None,
            cache_buffering_state: None,
            seeking: None,
            file_format: None,
            frames_rendered: 0,
            first_frame_presented: phase == 2 || phase == 3 || phase == 4,
            has_video_track: loaded,
            track_list_ready: loaded,
            resuming: false,
        })
    }

    fn track_options(&self, track_type: String) -> Result<Vec<PlayerTrackOption>, String> {
        let value = unsafe { fluxa_desktop_avplayer_tracks_json(self.handle()) };
        if value.is_null() {
            return Ok(Vec::new());
        }
        let json = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() };
        unsafe { fluxa_desktop_avplayer_free_string(value) };
        let tracks = serde_json::from_str::<Vec<PlayerTrackOption>>(&json).unwrap_or_default();
        Ok(tracks
            .into_iter()
            .filter(|track| {
                if track_type == "audio" {
                    !track.external && track.id.starts_with("audio.")
                } else {
                    track.external || track.id.starts_with("subtitle.")
                }
            })
            .collect())
    }

    fn add_subtitle(
        &self,
        url: String,
        _title: Option<String>,
        _language: Option<String>,
    ) -> Result<(), String> {
        let url = CString::new(url).map_err(|_| "subtitle URL contains NUL".to_string())?;
        self.call_string(unsafe {
            fluxa_desktop_avplayer_add_subtitle(self.handle(), url.as_ptr())
        })
    }
}

pub fn install(app_handle: AppHandle) -> Result<NativePlayerSurface, String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let parent = match window.window_handle().map_err(|e| e.to_string())?.as_ref() {
        RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr(),
        _ => return Err("macOS AppKit window handle is unavailable".to_string()),
    };
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let handle = unsafe {
        fluxa_desktop_avplayer_create(
            parent,
            size.width as f64 / scale,
            size.height as f64 / scale,
        )
    };
    if handle.is_null() {
        return Err("FluxaPlayerKit AVPlayer surface could not be created".to_string());
    }
    Ok(NativePlayerSurface {
        handle: handle as usize,
        app: app_handle,
    })
}
