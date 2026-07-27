use crate::mpv_render::{MpvRenderer, PlayerEvent, PlayerStatus, PlayerTrackOption};

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

impl PlaybackEngine for MpvRenderer {
    fn load(&mut self, url: &str, start_at: Option<u64>) -> Result<(), String> {
        MpvRenderer::load(self, url, start_at)
    }
    fn command_string(&self, command: &str) -> Result<(), String> {
        MpvRenderer::command_string(self, command)
    }
    fn user_command(&mut self, command: &str) -> Result<(), String> {
        MpvRenderer::user_command(self, command)
    }
    fn command_args(&self, args: &[&str]) -> Result<(), String> {
        MpvRenderer::command_args(self, args)
    }
    fn apply_options(&self, options: &[(String, String)]) -> Result<(), String> {
        MpvRenderer::apply_options(self, options)
    }
    fn set_http_headers(&self, headers: &[(String, String)]) -> Result<(), String> {
        MpvRenderer::set_http_headers(self, headers)
    }
    fn add_subtitle(
        &self,
        url: &str,
        title: Option<&str>,
        language: Option<&str>,
    ) -> Result<(), String> {
        MpvRenderer::add_subtitle(self, url, title, language)
    }
    fn query_property(&self, name: &str) -> Option<String> {
        MpvRenderer::query_property(self, name)
    }
    fn status(&self) -> PlayerStatus {
        MpvRenderer::status(self)
    }
    fn track_options(&self, track_type: &str) -> Vec<PlayerTrackOption> {
        MpvRenderer::track_options(self, track_type)
    }
    fn title(&self) -> Option<String> {
        MpvRenderer::title(self)
    }
    fn poll_events(&mut self) -> Vec<PlayerEvent> {
        MpvRenderer::poll_events(self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayerEngine {
    Mpv,
    Vlc,
}

pub fn read_player_engine(app: &tauri::AppHandle) -> PlayerEngine {
    use tauri::Manager;
    let state = app.state::<crate::DesktopState>();
    match crate::storage::read_pref_field(state, "playerEngine").as_deref() {
        Some("libvlc") => PlayerEngine::Vlc,
        _ => PlayerEngine::Mpv,
    }
}
