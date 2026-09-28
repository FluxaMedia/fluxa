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
    AnimatedTexture, ArtworkPriority, CalendarModel, DetailModel, DiscoverModel, HomeAssets,
    HomeCard, HomeHero, HomeLayout, HomeModel, LibraryModel, LibraryTab, PlayerModel,
    SettingsModel, UiFormFactorJson, Viewport, draw_calendar, draw_detail, draw_discover,
    draw_home, draw_library, draw_player, draw_settings,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

type SharedRenderer = Arc<Mutex<RendererState>>;

static LOGGER: OnceLock<fn(&str)> = OnceLock::new();

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
mod frame;
mod gpu;
mod input;
mod layout;
mod player;
mod poster_data;
mod presence;
mod profiles;
mod projection;
mod trailer;

use actions::*;
use artwork::*;
pub use bridge_video::{BridgeVideo, VideoBridge};
use frame::*;
use gpu::*;
use input::*;
use layout::*;
pub use profiles::{ImagePicker, import_legacy};

pub type PrePresent = Box<dyn Fn() + Send>;

pub use player::{DeviceOpener, Thumbnail, VideoBackend, VideoCommand, VideoStatus};

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
    discover: DiscoverModel,
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
    player: Option<player::PlayerSession>,
    shuffle: Option<player::Shuffle>,
    video: Option<Box<dyn VideoBackend>>,
    fullscreen_toggle: bool,
    profiles: Option<fluxa_ui::ProfilesModel>,
    account_auth: Option<accounts::AccountAuth>,
    pack_job: Option<profiles::PackJob>,
    picker_background: Option<String>,
    image_picker: Option<ImagePicker>,
    pre_present: Option<PrePresent>,
    trailers: trailer::Trailers,
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

#[derive(Clone, Copy, Debug)]
enum HomeScrollTarget {
    Vertical,
    Horizontal(usize),
    ScreenVertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Route {
    #[default]
    Home,
    Library,
    Discover,
    Calendar,
    Detail,
    Settings,
    Profiles,
    Player,
}

impl Route {
    fn parse(route: &str) -> Self {
        match route {
            "library" => Self::Library,
            "discover" => Self::Discover,
            "calendar" => Self::Calendar,
            "detail" => Self::Detail,
            "settings" => Self::Settings,
            "profiles" => Self::Profiles,
            "player" => Self::Player,
            _ => Self::Home,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Library => "library",
            Self::Discover => "discover",
            Self::Calendar => "calendar",
            Self::Detail => "detail",
            Self::Settings => "settings",
            Self::Profiles => "profiles",
            Self::Player => "player",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum NativeAction {
    CoreCommand {
        command: Value,
    },
    Navigate {
        destination: Route,
    },
    Detail {
        id: String,
        item_type: String,
        #[serde(default)]
        preview: Value,
    },
    Play {
        id: String,
        item_type: String,
    },
    StartPlayback {
        item: Value,
    },
    ToggleWatchlist {
        item: Value,
    },
    SettingsChange {
        key: String,
        value: Value,
    },
    SettingsSection {
        index: usize,
    },
    AccountToggle {
        provider: String,
    },
    DiscoverType {
        content_type: String,
    },
    DiscoverCatalog {
        content_type: String,
        catalog_key: String,
        extra_name: String,
        extra_value: String,
        query: String,
    },
    DiscoverFilters {
        content_type: String,
        catalog_key: String,
        extra_name: String,
        extra_value: String,
        query: String,
    },
    CalendarMonth {
        year: i32,
        month: i32,
    },
    Back,
    LoadMore {
        row_id: String,
    },
}

const NODE_HOME: u64 = 10;
const NODE_LIBRARY: u64 = 11;
const NODE_DISCOVER: u64 = 12;
const NODE_CALENDAR: u64 = 13;
const NODE_PROFILE: u64 = 14;
const NODE_PLAY: u64 = 20;
const NODE_MORE_INFO: u64 = 21;
const NODE_CARD_BASE: u64 = 100;

fn request_projection(state: &mut RendererState) {
    let Some(snapshot) = state.core_snapshot.clone() else {
        return;
    };
    let revision = state.core_snapshot_revision;
    if state.last_snapshot_revision == Some(revision) {
        return;
    }
    state.last_snapshot_revision = Some(revision);
    state.projector.submit(projection::Request {
        revision,
        snapshot,
        form_factor: state.home.form_factor,
        library_tab: state.library_tab,
        library_query: state.library_query.clone(),
        library_sort: state.library_sort.clone(),
        hero_trailers: state.trailers.inputs(),
    });
}

fn apply_projection(state: &mut RendererState, projection: projection::Projection) {
    state
        .projector
        .settle(projection.revision, state.core_snapshot_revision);
    if projection.route != state.route {
        state.active_scroll = None;
        state.scroll_velocity = 0.0;
    }
    state.route = projection.route;
    let scroll_offset = state.home.scroll_offset;
    let row_scroll_offsets = std::mem::take(&mut state.home.row_scroll_offsets);
    state.home = projection.home;
    state.home.scroll_offset = scroll_offset;
    state.home.row_scroll_offsets = row_scroll_offsets;
    if HOME_SYNC_LOGS.fetch_add(1, Ordering::Relaxed) % 120 == 0 {
        let titles = state
            .home
            .rows
            .iter()
            .take(5)
            .map(|row| row.title.as_str())
            .collect::<Vec<_>>();
        host_log(format!(
            "Home snapshot: hero={}, title={:?}, artwork={}, continue_cards={}, rows={:?}",
            state.home.show_hero_section,
            state.home.title,
            state.home.background_url.is_some(),
            state.home.cards.len(),
            titles,
        ));
    }
    state.library = projection.library;
    let query = std::mem::take(&mut state.discover.query);
    state.discover = projection.discover;
    state.discover.query = query;
    let selected_day = state.calendar.selected_day;
    let previous_month = (state.calendar.year, state.calendar.month);
    state.calendar = projection.calendar;
    if previous_month == (state.calendar.year, state.calendar.month) {
        state.calendar.selected_day = selected_day;
    }
    state.detail = projection.detail;
    state.detail.resume = state.home.resume_for(&state.detail.id).cloned();
    state.trailers.set_targets(projection.trailer_targets);
    let settings_section = state.settings.active_section;
    let section_open = state.settings.section_open;
    let addon_url = std::mem::take(&mut state.settings.addon_url);
    let plugin_url = std::mem::take(&mut state.settings.plugin_url);
    let search = std::mem::take(&mut state.settings.search);
    let poster_fields = std::mem::take(&mut state.settings.poster_fields);
    state.settings = projection.settings;
    state.settings.active_section = settings_section.min(fluxa_ui::SETTINGS_SECTIONS.len() - 1);
    state.settings.section_open = section_open;
    state.settings.addon_url = addon_url;
    state.settings.plugin_url = plugin_url;
    state.settings.search = search;
    for (field, typed) in state.settings.poster_fields.iter_mut().zip(poster_fields) {
        if !typed.is_empty() {
            *field = typed;
        }
    }
    state.ui = UiTree::default();
    request_discover_background_page(state);
}

const DISCOVER_BACKGROUND_LIMIT: usize = 400;

fn request_discover_background_page(state: &mut RendererState) {
    let discover = &state.discover;
    if state.route == Route::Discover
        || discover.is_loading
        || !discover.query.is_empty()
        || discover.results.len() >= DISCOVER_BACKGROUND_LIMIT
    {
        return;
    }
    let Some(request) = discover.next_page.as_ref() else {
        return;
    };
    let skip = request.get("skip").and_then(Value::as_i64);
    if skip.is_none() || skip == state.discover_background_skip {
        return;
    }
    state.discover_background_skip = skip;
    state
        .pending_native_actions
        .push(NativeAction::CoreCommand {
            command: request.clone(),
        });
}

fn sync_home_from_core_snapshot(state: &mut RendererState) {
    request_projection(state);
    if let Some(projection) = state.projector.take() {
        apply_projection(state, projection);
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
            discover: DiscoverModel::default(),
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
            player: None,
            shuffle: None,
            video: None,
            fullscreen_toggle: false,
            profiles: None,
            account_auth: None,
            pack_job: None,
            picker_background: None,
            image_picker: None,
            pre_present: None,
            trailers: trailer::Trailers::default(),
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
            state.ui = UiTree::default();
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
            state.home = home;
            state.home.scroll_offset = scroll_offset;
            state.home.row_scroll_offsets = row_scroll_offsets;
            state.ui = UiTree::default();
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
            state.ui = UiTree::default();
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

    pub fn scroll(&self, delta_y: f32) {
        self.with_state(|state| {
            let viewport = logical_viewport(state);
            if state.route == Route::Home {
                let max = fluxa_ui::home_scroll_max(viewport, &state.home);
                state.home.scroll_offset = (state.home.scroll_offset + delta_y).clamp(0.0, max);
            } else {
                update_screen_scroll(state, delta_y, viewport);
            }
        });
    }

    pub fn mouse_moved(&self, x: f32, y: f32) {
        self.with_state(|state| {
            let position = Pos2::new(x, y) * (state.density / state.scale());
            state.mouse_position = Some(position);
            state.keyboard_focus_visible = false;
            if let Some(player) = state.player.as_mut() {
                player.touch();
            }
            state.egui_events.push(egui::Event::PointerMoved(position));
        });
    }

    pub fn mouse_button(&self, button: MouseButton, pressed: bool, x: f32, y: f32) {
        self.with_state(|state| {
            let position = Pos2::new(x, y) * (state.density / state.scale());
            state.mouse_position = Some(position);
            state.keyboard_focus_visible = false;
            if pressed
                && matches!(button, MouseButton::Secondary)
                && state.card_menu.is_none()
                && card_menu::open_at(state, position)
            {
                return;
            }
            state.egui_events.push(egui::Event::PointerButton {
                pos: position,
                button: match button {
                    MouseButton::Primary => egui::PointerButton::Primary,
                    MouseButton::Secondary => egui::PointerButton::Secondary,
                    MouseButton::Middle => egui::PointerButton::Middle,
                },
                pressed,
                modifiers: state.modifiers,
            });
        });
    }

    pub fn mouse_left(&self) {
        self.with_state(|state| {
            state.mouse_position = None;
            state.egui_events.push(egui::Event::PointerGone);
        });
    }

    pub fn wheel(&self, delta_x: f32, delta_y: f32) {
        self.with_state(|state| {
            let (delta_x, delta_y) = if state.modifiers.shift && delta_x == 0.0 {
                (delta_y, 0.0)
            } else {
                (delta_x, delta_y)
            };
            state.egui_events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(-delta_x, -delta_y),
                phase: egui::TouchPhase::Move,
                modifiers: state.modifiers,
            });
            let viewport = logical_viewport(state);
            if state.route == Route::Home {
                if delta_x != 0.0
                    && let Some(row) = state.mouse_position.and_then(|position| {
                        fluxa_ui::home_row_at_y(viewport, &state.home, position.y)
                    })
                {
                    if state.home.row_scroll_offsets.len() <= row {
                        state.home.row_scroll_offsets.resize(row + 1, 0.0);
                    }
                    let max = fluxa_ui::home_row_scroll_max(viewport, &state.home, row);
                    let offset = &mut state.home.row_scroll_offsets[row];
                    *offset = (*offset + delta_x).clamp(0.0, max);
                    request_home_row_load_more(state, row, viewport);
                }
                let max = fluxa_ui::home_scroll_max(viewport, &state.home);
                state.home.scroll_offset = (state.home.scroll_offset + delta_y).clamp(0.0, max);
            } else if delta_y != 0.0 {
                update_screen_scroll(state, delta_y, viewport);
            }
        });
    }

    pub fn set_modifiers(&self, modifiers: egui::Modifiers) {
        self.with_state(|state| state.modifiers = modifiers);
    }

    pub fn egui_key(&self, key: egui::Key, pressed: bool) {
        self.with_state(|state| {
            state.egui_events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: state.modifiers,
            });
        });
    }

    pub fn egui_text(&self, text: &str) {
        self.with_state(|state| state.egui_events.push(egui::Event::Text(text.to_owned())));
    }

    pub fn egui_event(&self, event: egui::Event) {
        self.with_state(|state| state.egui_events.push(event));
    }

    pub fn wants_keyboard(&self) -> bool {
        self.with_state(|state| state.wants_keyboard)
            .unwrap_or(false)
    }

    pub fn cursor(&self) -> egui::CursorIcon {
        self.with_state(|state| state.cursor).unwrap_or_default()
    }

    pub fn pointer(&self, phase: PointerPhase, x: f32, y: f32) {
        self.with_state(|state| {
            let ratio = state.density / state.scale();
            pointer_event(state, phase, [x * ratio, y * ratio])
        });
    }

    pub fn key_down(&self, input: KeyInput) {
        self.with_state(|state| key_down(state, input));
    }

    pub fn back(&self) -> bool {
        self.with_state(|state| {
            if state.player.is_none() && state.profiles.is_none() && state.route == Route::Home {
                return false;
            }
            key_down(state, KeyInput::Key(Key::Back));
            true
        })
        .unwrap_or(false)
    }

    pub fn focused_node(&self) -> Option<u64> {
        self.with_state(|state| state.ui.focused()).flatten()
    }

    pub fn text_input_focused(&self) -> bool {
        self.with_state(|state| {
            state.ui.focused().and_then(|node| state.ui.node(node)).is_some_and(|node| node.kind == UiNodeKind::Input)
        })
        .unwrap_or(false)
    }

    pub fn focus_node(&self, node: u64) {
        self.with_state(|state| {
            state.keyboard_focus_visible = true;
            rebuild_current_ui(state);
            let actions = state.ui.dispatch(UiEvent::FocusRequest(node));
            remember_actions(state, actions);
            ensure_focused_visible(state);
        });
    }

    pub fn activate_node(&self, node: u64) {
        self.with_state(|state| {
            rebuild_current_ui(state);
            remember_actions(state, vec![UiAction::Activated(node)]);
        });
    }

    pub fn text_input(&self, text: &str) {
        self.with_state(|state| {
            let actions = state.ui.dispatch(UiEvent::TextInput(text.to_owned()));
            remember_actions(state, actions);
        });
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

    pub fn surface_created(&self, surface: NativeSurface, width: u32, height: u32) {
        let size = [width.max(1), height.max(1)];
        host_log(format!("Surface created: {}x{}", size[0], size[1]));
        let Some((generation, density, artwork_cache_dir, opener)) = self.with_state(|state| {
            state.generation = state.generation.saturating_add(1);
            state.size = size;
            state.pending_resize = None;
            state.gpu = None;
            (
                state.generation,
                state.density,
                state.artwork_cache_dir.clone(),
                state.video.as_ref().and_then(|video| video.device_opener()),
            )
        }) else {
            return;
        };
        let shared = self.0.clone();
        let task = async move {
            let result = Gpu::create(surface, size, density, artwork_cache_dir, opener).await;
            let Ok(mut state) = shared.lock() else { return };
            if state.generation != generation {
                return;
            }
            match result {
                Ok(gpu) => state.gpu = Some(gpu),
                Err(error) => host_log(format!("GPU surface init failed: {error}")),
            }
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || pollster::block_on(task));
    }

    pub fn surface_changed(&self, width: u32, height: u32) {
        let size = [width.max(1), height.max(1)];
        self.with_state(|state| {
            state.size = size;
            state.pending_resize = Some(size);
        });
    }

    pub fn surface_destroyed(&self) {
        self.with_state(|state| {
            state.generation = state.generation.saturating_add(1);
            state.pending_resize = None;
            state.gpu = None;
        });
    }

    pub fn render(&self) {
        self.with_state(render_frame);
    }

    /// Returns when the next frame is due; `None` means the host is idle until input arrives.
    pub fn next_redraw(&self) -> Option<Instant> {
        self.with_state(next_redraw).flatten()
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
