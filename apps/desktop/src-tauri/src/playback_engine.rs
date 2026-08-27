use crate::mpv_render::{MpvClientHandle, PlayerEvent, PlayerStatus, PlayerTrackOption};

pub trait PlaybackEngine: Send {
    fn load(&mut self, url: &str, start_at: Option<u64>) -> Result<(), String>;
    fn user_command(&mut self, command: &str) -> Result<(), String> {
        self.command_string(command)
    }
    fn command_string(&self, command: &str) -> Result<(), String>;
    fn command_args(&self, args: &[&str]) -> Result<(), String>;
    fn apply_options(&self, options: &[(String, String)]) -> Result<(), String>;
    fn set_http_headers(&self, headers: &[(String, String)]) -> Result<(), String>;
    fn add_subtitle(
        &self,
        url: &str,
        title: Option<&str>,
        language: Option<&str>,
    ) -> Result<(), String>;
    fn query_property(&self, name: &str) -> Option<String>;
    fn status(&self) -> PlayerStatus;
    fn track_options(&self, track_type: &str) -> Vec<PlayerTrackOption>;
    fn title(&self) -> Option<String>;
    fn poll_events(&mut self) -> Vec<PlayerEvent>;
}

impl PlaybackEngine for MpvClientHandle {
    fn load(&mut self, url: &str, start_at: Option<u64>) -> Result<(), String> {
        MpvClientHandle::load(self, url, start_at)
    }
    fn command_string(&self, command: &str) -> Result<(), String> {
        MpvClientHandle::command_string(self, command)
    }
    fn user_command(&mut self, command: &str) -> Result<(), String> {
        MpvClientHandle::user_command(self, command)
    }
    fn command_args(&self, args: &[&str]) -> Result<(), String> {
        MpvClientHandle::command_args(self, args)
    }
    fn apply_options(&self, options: &[(String, String)]) -> Result<(), String> {
        MpvClientHandle::apply_options(self, options)
    }
    fn set_http_headers(&self, headers: &[(String, String)]) -> Result<(), String> {
        MpvClientHandle::set_http_headers(self, headers)
    }
    fn add_subtitle(
        &self,
        url: &str,
        title: Option<&str>,
        language: Option<&str>,
    ) -> Result<(), String> {
        MpvClientHandle::add_subtitle(self, url, title, language)
    }
    fn query_property(&self, name: &str) -> Option<String> {
        MpvClientHandle::query_property(self, name)
    }
    fn status(&self) -> PlayerStatus {
        MpvClientHandle::status(self)
    }
    fn track_options(&self, track_type: &str) -> Vec<PlayerTrackOption> {
        MpvClientHandle::track_options(self, track_type)
    }
    fn title(&self) -> Option<String> {
        MpvClientHandle::title(self)
    }
    fn poll_events(&mut self) -> Vec<PlayerEvent> {
        MpvClientHandle::poll_events(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayerEngine {
    Mpv,
    Vlc,
    AvPlayer,
}

pub fn read_player_engine(app: &tauri::AppHandle) -> PlayerEngine {
    use tauri::Manager;
    let state = app.state::<crate::DesktopState>();
    match crate::storage::read_pref_field(state, "playerEngine").as_deref() {
        Some("libvlc") => PlayerEngine::Vlc,
        Some("avplayer") if cfg!(target_os = "macos") => PlayerEngine::AvPlayer,
        _ => PlayerEngine::Mpv,
    }
}
