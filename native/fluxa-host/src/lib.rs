use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use web_time::Instant;

pub use egui;
use egui::{Pos2, Rect as EguiRect, Vec2};
use fluxa_artwork::{ArtworkFetcher, Priority as ArtworkFetchPriority};
use fluxa_effects::{SessionHandle, Storage, core_value};
use fluxa_renderer::egui_wgpu_backend::{EguiWgpuBackend, ScreenDescriptor};
use fluxa_renderer::platform::{GraphicsBackend, backends_for};
use fluxa_renderer::svg_icons::{ICON_SIZE, ICONS, LOGO_SIZE, LOGOS, rasterize_svg};
pub use fluxa_renderer::ui::{GamepadButton, Key};
use fluxa_renderer::ui::{PointerButton, UiAction, UiEvent, UiNode, UiNodeKind, UiTree};
use fluxa_ui::{
    AnimatedTexture, ArtworkPriority, CalendarModel, DetailModel, DiscoverModel, FolderModel,
    HomeAssets, HomeCard, HomeHero, HomeLayout, HomeModel, LibraryModel, LibraryTab, PlayerModel,
    SettingsModel, UiFormFactorJson, Viewport, draw_calendar, draw_detail, draw_discover,
    draw_home, draw_library, draw_player, draw_settings,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

type SharedRenderer = Arc<Mutex<RendererState>>;

static LOGGER: OnceLock<fn(&str)> = OnceLock::new();
pub(crate) static EMOJI_RASTERIZER: OnceLock<EmojiRasterizer> = OnceLock::new();

pub type EmojiRasterizer = fn(&str, u32) -> Option<(u32, u32, Vec<u8>)>;

pub fn set_emoji_rasterizer(rasterizer: EmojiRasterizer) {
    let _ = EMOJI_RASTERIZER.set(rasterizer);
}

pub fn set_logger(logger: fn(&str)) {
    let _ = LOGGER.set(logger);
    fluxa_effects::set_logger(logger);
}

fn host_log(message: impl AsRef<str>) {
    match LOGGER.get() {
        Some(logger) => logger(message.as_ref()),
        None => eprintln!("[fluxa-host] {}", message.as_ref()),
    }
}

#[derive(Clone)]
pub struct NativeSurface {
    target: Arc<dyn Fn() -> Result<wgpu::SurfaceTargetUnsafe, String> + Send + Sync>,
    backends: &'static [GraphicsBackend],
}

impl NativeSurface {
    pub unsafe fn new(
        backends: &'static [GraphicsBackend],
        target: impl Fn() -> Result<wgpu::SurfaceTargetUnsafe, String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            target: Arc::new(target),
            backends,
        }
    }
}

mod accounts;
mod actions;
mod artwork;
mod bridge_video;
mod card_menu;
mod cast;
mod collections;
mod events;
mod frame;
mod gpu;
mod input;
mod launcher;
mod layout;
mod player;
mod poster_data;
mod presence;
mod profiles;
mod projection;
mod route;
mod shortcuts;
mod shorts;
mod stream_badges;
mod surface;
mod sync;
mod trailer;

use actions::*;
use artwork::*;
pub use bridge_video::{BridgeVideo, VideoBridge};
use frame::*;
use gpu::*;
use input::*;
use layout::*;
pub use profiles::{ImagePicker, import_legacy};
use route::*;
use sync::*;

pub type PrePresent = Box<dyn Fn() + Send>;

pub use player::{
    DeviceOpener, PlaybackStats, Thumbnail, TrackSelection, VideoBackend, VideoCommand,
    VideoStatus, VideoTrack,
};

static GPU_WAIT_LOGS: AtomicU32 = AtomicU32::new(0);
static HOME_SYNC_LOGS: AtomicU32 = AtomicU32::new(0);

struct RendererState {
    generation: u64,
    size: [u32; 2],
    density: f32,
    artwork_cache_dir: Option<PathBuf>,
    safe_bottom: f32,
    pending_resize: Option<[u32; 2]>,
    gpu: Option<Gpu>,
    ui: UiTree,
    rendered_layout: Option<(Route, HomeLayout)>,
    last_actions: Vec<UiAction>,
    pending_native_actions: Vec<NativeAction>,
    backdrop_prefetch: Option<String>,
    keyboard_focus_visible: bool,
    load_more_requested_counts: HashMap<String, usize>,
    discover_background_skip: Option<i64>,
    touch_start: Option<[f32; 2]>,
    touch_last: Option<[f32; 2]>,
    touch_last_at: Option<Instant>,
    touch_velocity_samples: VecDeque<(Instant, [f32; 2])>,
    touch_scrolled: bool,
    nav_dragging: bool,
    touch_down_at: Option<Instant>,
    touch_menu_opened: bool,
    card_menu: Option<card_menu::CardMenu>,
    menu_serial: u64,
    active_scroll: Option<HomeScrollTarget>,
    scroll_velocity: f32,
    scroll_animation_at: Instant,
    route: Route,
    screen_scroll_offsets: HashMap<Route, f32>,
    home: HomeModel,
    library: LibraryModel,
    library_tab: LibraryTab,
    library_query: String,
    library_sort: String,
    library_type: String,
    library_list: bool,
    library_downloads: bool,
    discover: DiscoverModel,
    folder: FolderModel,
    folder_tab: usize,
    calendar: CalendarModel,
    detail: DetailModel,
    settings: SettingsModel,
    core_snapshot: Option<Arc<Value>>,
    projector: projection::Projector,
    core_snapshot_revision: u64,
    last_snapshot_revision: Option<u64>,
    session: Option<SessionHandle>,
    session_revision: Option<u64>,
    egui_events: Vec<egui::Event>,
    modifiers: egui::Modifiers,
    mouse_position: Option<Pos2>,
    cursor: egui::CursorIcon,
    wants_keyboard: bool,
    redraw_at: Option<Instant>,
    focus_hint: Option<(Route, u64)>,
    player: Option<player::PlayerSession>,
    shuffle: Option<player::Shuffle>,
    video: Option<Box<dyn VideoBackend>>,
    fullscreen_toggle: bool,
    shortcut_recording: Option<String>,
    collections: collections::State,
    stream_badges: stream_badges::State,
    applied_app_icon: Option<String>,
    open_url: Option<String>,
    profiles: Option<fluxa_ui::ProfilesModel>,
    account_auth: Option<accounts::AccountAuth>,
    pack_job: Option<profiles::PackJob>,
    picker_background: Option<String>,
    image_picker: Option<ImagePicker>,
    pre_present: Option<PrePresent>,
    trailers: trailer::Trailers,
    shorts: fluxa_ui::ShortsModel,
    poster_data: poster_data::PosterData,
}

fn current_presence(state: &RendererState) -> presence::Presence {
    if !state.settings.bool_value("discordRichPresenceEnabled") {
        return presence::Presence::Off;
    }
    if let Some(player) = state.player.as_ref() {
        return player.presence();
    }
    if state.route == Route::Detail && !state.detail.title.is_empty() {
        return presence::Presence::Viewing {
            title: state.detail.title.clone(),
            poster: state
                .detail
                .poster_url
                .clone()
                .filter(|url| url.starts_with("http")),
        };
    }
    let label = match state.route {
        Route::Library => "Browsing the library",
        Route::Discover => "Discovering",
        Route::Calendar => "Checking the calendar",
        Route::Shorts => "Watching shorts",
        Route::Settings => "In settings",
        _ => "Browsing",
    };
    presence::Presence::Browsing(label.to_owned())
}

impl RendererState {
    fn scale(&self) -> f32 {
        let fit = if self.home.form_factor == fluxa_ui::UiFormFactorJson::Desktop {
            let width = self.size[0] as f32 / self.density;
            let height = self.size[1] as f32 / self.density;
            (width / 1920.0).min(height / 1080.0).clamp(1.0, 2.0)
        } else {
            1.0
        };
        self.density * fit * self.settings.ui_scale()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerPhase {
    Down,
    Move,
    Up,
}

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Clone, Copy, Debug)]
pub enum KeyInput {
    Key(Key),
    Gamepad(GamepadButton),
    Backspace,
}

#[derive(Clone)]
pub struct FluxaHost(SharedRenderer);

pub fn app_icon_svg(id: &str) -> String {
    fluxa_core::app_icon::app_icon_svg(fluxa_core::app_icon::resolve_app_icon(Some(id)))
}

pub fn app_icon_rgba(id: &str, size: u32) -> Option<image::RgbaImage> {
    fluxa_renderer::svg_icons::rasterize_svg(app_icon_svg(id).as_bytes(), size).ok()
}

impl FluxaHost {
    pub fn new(density: f32, artwork_cache_dir: Option<PathBuf>) -> Self {
        let density = if density.is_finite() && density > 0.0 {
            density
        } else {
            1.0
        };
        Self(Arc::new(Mutex::new(RendererState {
            generation: 0,
            size: [1, 1],
            density,
            artwork_cache_dir,
            safe_bottom: 0.0,
            pending_resize: None,
            gpu: None,
            ui: UiTree::default(),
            rendered_layout: None,
            last_actions: Vec::new(),
            pending_native_actions: Vec::new(),
            backdrop_prefetch: None,
            keyboard_focus_visible: false,
            load_more_requested_counts: HashMap::new(),
            discover_background_skip: None,
            touch_start: None,
            touch_last: None,
            touch_last_at: None,
            touch_velocity_samples: VecDeque::new(),
            touch_scrolled: false,
            nav_dragging: false,
            touch_down_at: None,
            touch_menu_opened: false,
            card_menu: None,
            menu_serial: 0,
            active_scroll: None,
            scroll_velocity: 0.0,
            scroll_animation_at: Instant::now(),
            route: Route::Home,
            screen_scroll_offsets: HashMap::new(),
            home: HomeModel::default(),
            library: LibraryModel::default(),
            library_tab: LibraryTab::Watchlist,
            library_query: String::new(),
            library_sort: "recent".to_owned(),
            library_type: "all".to_owned(),
            library_list: false,
            library_downloads: false,
            discover: DiscoverModel::default(),
            folder: FolderModel::default(),
            folder_tab: 0,
            calendar: CalendarModel::default(),
            detail: DetailModel::default(),
            settings: SettingsModel::default(),
            core_snapshot: None,
            projector: projection::Projector::new(),
            core_snapshot_revision: 0,
            last_snapshot_revision: None,
            session: None,
            session_revision: None,
            egui_events: Vec::new(),
            modifiers: egui::Modifiers::NONE,
            mouse_position: None,
            cursor: egui::CursorIcon::Default,
            wants_keyboard: false,
            redraw_at: Some(Instant::now()),
            focus_hint: None,
            player: None,
            shuffle: None,
            video: None,
            fullscreen_toggle: false,
            shortcut_recording: None,
            collections: Default::default(),
            stream_badges: Default::default(),
            applied_app_icon: None,
            open_url: None,
            profiles: None,
            account_auth: None,
            pack_job: None,
            picker_background: None,
            image_picker: None,
            pre_present: None,
            trailers: trailer::Trailers::default(),
            shorts: Default::default(),
            poster_data: poster_data::PosterData::default(),
        })))
    }

    fn with_state<T>(&self, f: impl FnOnce(&mut RendererState) -> T) -> Option<T> {
        self.0.lock().ok().map(|mut state| f(&mut state))
    }

    pub fn set_safe_bottom_inset(&self, inset_dp: f32) {
        self.with_state(|state| {
            state.safe_bottom = if inset_dp.is_finite() {
                inset_dp.max(0.0)
            } else {
                0.0
            };
        });
    }

    pub fn set_form_factor(&self, form_factor: &str) {
        let form_factor = match form_factor {
            "mobile" => UiFormFactorJson::Mobile,
            "tv" => UiFormFactorJson::Tv,
            "desktop" => UiFormFactorJson::Desktop,
            _ => return,
        };
        self.with_state(|state| {
            state.home.form_factor = form_factor;
            reset_ui(state);
        });
    }

    pub fn set_platform(&self, platform: &str) {
        let Some(platform) = fluxa_ui::UiPlatform::parse(platform) else {
            return;
        };
        self.with_state(|state| {
            state.home.platform = platform;
            reset_ui(state);
        });
    }

    pub fn set_home_state_json(&self, json: &str) {
        let Ok(home) = serde_json::from_str::<HomeModel>(json) else {
            host_log("ignored invalid home state JSON");
            return;
        };
        self.with_state(|state| {
            let scroll_offset = state.home.scroll_offset;
            let row_scroll_offsets = std::mem::take(&mut state.home.row_scroll_offsets);
            let platform = state.home.platform;
            state.home = home;
            state.home.platform = platform;
            state.home.scroll_offset = scroll_offset;
            state.home.row_scroll_offsets = row_scroll_offsets;
            reset_ui(state);
        });
    }

    pub fn set_core_snapshot_json(&self, json: &str) {
        let Ok(snapshot) = serde_json::from_str::<Value>(json) else {
            host_log("ignored invalid shared Core snapshot JSON");
            return;
        };
        self.with_state(|state| {
            state.core_snapshot = Some(Arc::new(snapshot));
            state.core_snapshot_revision = state.core_snapshot_revision.wrapping_add(1);
            sync_home_from_core_snapshot(state);
            reset_ui(state);
        });
    }

    pub fn snapshot_json(&self) -> Option<String> {
        self.with_state(|state| {
            state
                .core_snapshot
                .as_ref()
                .map(|snapshot| snapshot.to_string())
                .unwrap_or_else(|| "{}".to_owned())
        })
    }

    pub fn take_actions_json(&self) -> Option<String> {
        self.with_state(|state| {
            route_actions_to_session(state);
            take_native_actions(state)
        })
    }

    pub fn start_session(&self, data_dir: PathBuf) -> Result<(), String> {
        let session = SessionHandle::open(Storage::open(data_dir)?)?;
        profiles::load_profile(&session)?;
        let pick = profiles::should_pick_on_start(session.storage());
        let background = profiles::picker_settings(session.storage()).background_url;
        self.with_state(|state| {
            state.session = Some(session);
            state.picker_background = background;
            if pick {
                profiles::open(state);
            }
        });
        Ok(())
    }

    pub fn media_command(&self, command: &str, value: f64) {
        self.with_state(|state| {
            state
                .pending_native_actions
                .push(NativeAction::MediaCommand {
                    command: command.to_owned(),
                    value,
                })
        });
    }

    pub fn push_action_json(&self, json: &str) -> Result<(), String> {
        let action: NativeAction = serde_json::from_str(json).map_err(|error| error.to_string())?;
        self.with_state(|state| state.pending_native_actions.push(action));
        Ok(())
    }

    pub fn set_image_picker(&self, picker: ImagePicker) {
        self.with_state(|state| state.image_picker = Some(picker));
    }

    pub fn set_pre_present(&self, hook: PrePresent) {
        self.with_state(|state| state.pre_present = Some(hook));
    }

    pub fn set_video_backend(&self, backend: Box<dyn VideoBackend>) {
        self.with_state(|state| state.video = Some(backend));
    }

    pub fn is_playing(&self) -> bool {
        self.with_state(|state| state.player.is_some())
            .unwrap_or(false)
    }

    pub fn take_fullscreen_toggle(&self) -> bool {
        self.with_state(|state| std::mem::take(&mut state.fullscreen_toggle))
            .unwrap_or(false)
    }

    pub fn take_open_url(&self) -> Option<String> {
        self.with_state(|state| state.open_url.take()).flatten()
    }

    pub fn launcher_feed_json(&self) -> Option<String> {
        self.with_state(|state| {
            let snapshot = match state.session.as_ref() {
                Some(session) => session.snapshot(),
                None => state.core_snapshot.clone()?,
            };
            Some(launcher::feed(&snapshot).to_string())
        })
        .flatten()
    }

    pub fn take_app_icon(&self) -> Option<String> {
        self.with_state(|state| {
            let icon = fluxa_core::app_icon::resolve_app_icon(state.settings.app_icon());
            if state.applied_app_icon.as_deref() == Some(icon.id.as_str()) {
                return None;
            }
            state.applied_app_icon = Some(icon.id.clone());
            Some(icon.id.clone())
        })
        .flatten()
    }
}

fn current_year_month() -> (i32, u32) {
    use chrono::Datelike;
    let today = chrono::Local::now().date_naive();
    (today.year(), today.month())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_icon_setting_lists_every_icon() {
        let row = (0..)
            .map_while(fluxa_ui::settings_row_by_index)
            .find(|row| row.key == "appIcon")
            .unwrap();
        let ids: Vec<_> = fluxa_core::app_icon::app_icons()
            .iter()
            .map(|icon| icon.id.as_str())
            .collect();
        assert_eq!(row.options, ids.as_slice());
    }

    #[test]
    fn opening_detail_navigates_then_loads_in_profile_language() {
        let profile = json!({"id": "p1", "language": "tr"});
        let commands = session_commands(
            &NativeAction::Detail {
                id: "tt1".to_owned(),
                item_type: "movie".to_owned(),
                preview: Value::Null,
            },
            &profile,
        )
        .unwrap();
        assert_eq!(commands[0]["route"], "detail");
        assert_eq!(commands[1]["type"], "detailLoadRequested");
        assert_eq!(commands[1]["language"], "tr");
    }

    #[test]
    fn profile_button_opens_settings() {
        let commands = session_commands(
            &NativeAction::Navigate {
                destination: Route::Settings,
            },
            &Value::Null,
        )
        .unwrap();
        assert_eq!(commands[0]["route"], "settings");
    }

    #[test]
    fn settings_section_stays_with_the_platform() {
        assert!(
            session_commands(&NativeAction::SettingsSection { index: 2 }, &Value::Null).is_none()
        );
    }

    #[test]
    fn detail_play_requests_direct_playback_with_the_full_item() {
        let item = json!({"id": "tt1", "type": "movie", "lastVideoId": "tt1:1:2"});
        let commands = session_commands(
            &NativeAction::StartPlayback { item: item.clone() },
            &json!({"language": "tr"}),
        )
        .unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0]["type"], "directPlaybackRequested");
        assert_eq!(commands[0]["meta"], item);
        assert_eq!(commands[0]["language"], "tr");
    }

    #[test]
    fn legacy_import_runs_once_and_keeps_packs_loadable() {
        let dir = std::env::temp_dir().join(format!("fluxa-legacy-{}", std::process::id()));
        let legacy = json!({
            "profiles": [{"id": "p1", "name": "A", "localAddons": ["https://a/manifest.json"]}],
            "activeProfileId": "p1",
            "pickerSettings": {"avatarPacks": [{"id": "https://pack", "repositoryUrl": "r", "title": "t", "avatars": []}]},
        })
        .to_string();
        assert!(import_legacy(dir.clone(), &legacy).unwrap());
        assert!(!import_legacy(dir.clone(), &legacy).unwrap());
        let storage = fluxa_effects::Storage::open(dir.clone()).unwrap();
        assert_eq!(
            storage.read_json("legacy_addons_p1").unwrap(),
            Some(json!(["https://a/manifest.json"]))
        );
        assert_eq!(
            profiles::picker_settings(&storage).avatar_packs[0].manifest_url,
            "https://pack"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn android_prefs_import_merges_credentials_and_addons() {
        let dir = std::env::temp_dir().join(format!("fluxa-prefs-{}", std::process::id()));
        let legacy = json!({
            "prefs": {
                "profiles_list": json!([{"id": "p1", "email": "a@b", "name": "A"}]).to_string(),
                "local_addons_p1": json!(["https://a/manifest.json"]).to_string(),
                "last_active_profile_id": "p1",
            },
            "credentials": {"p1": json!({"authKey": "k", "tmdbApiKey": null}).to_string()},
        })
        .to_string();
        assert!(import_legacy(dir.clone(), &legacy).unwrap());
        let storage = fluxa_effects::Storage::open(dir.clone()).unwrap();
        let profiles = storage.read_json("profiles").unwrap().unwrap();
        assert_eq!(profiles[0]["authKey"], "k");
        assert_eq!(
            storage.read_json("legacy_addons_p1").unwrap(),
            Some(json!(["https://a/manifest.json"]))
        );
        assert_eq!(
            storage.read_json("active_profile_id").unwrap(),
            Some(json!("p1"))
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn back_closes_player_instead_of_navigating() {
        let host = FluxaHost::new(1.0, None);
        host.with_state(|state| state.player = Some(player::PlayerSession::new(json!({}))));
        host.key_down(KeyInput::Key(Key::Back));
        assert!(!host.is_playing());
        host.with_state(|state| assert!(state.pending_native_actions.is_empty()));
    }
}
