use crate::mpv_render::{PlayerStatus, PlayerTrackOption};

pub type Artwork = Option<(Vec<u8>, i32, i32)>;

pub trait PlayerSurface: Send + Sync {
    fn backend_name(&self) -> &'static str;

    fn load(
        &self,
        url: String,
        start_at: Option<u64>,
        total_duration: Option<u64>,
    ) -> Result<(), String>;

    fn hide(&self);

    fn shutdown(&self) -> Result<(), String>;

    fn show_loading(&self, title: String, episode_title: Option<String>);

    fn set_title(&self, title: String, episode_title: Option<String>);

    fn set_artwork(
        &self,
        title: String,
        episode_title: Option<String>,
        background: Artwork,
        logo: Artwork,
    );

    fn set_cursor_visible(&self, visible: bool);

    fn command(&self, command: String) -> Result<(), String>;

    fn command_args(&self, commands: Vec<Vec<String>>) -> Result<(), String>;

    fn status(&self) -> Result<PlayerStatus, String>;

    fn track_options(&self, track_type: String) -> Result<Vec<PlayerTrackOption>, String>;

    fn add_subtitle(
        &self,
        url: String,
        title: Option<String>,
        language: Option<String>,
    ) -> Result<(), String>;
}
