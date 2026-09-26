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
use fluxa_effects::{SessionHandle, Storage};
use fluxa_renderer::egui_wgpu_backend::{EguiWgpuBackend, ScreenDescriptor};
use fluxa_renderer::platform::{GraphicsBackend, backends_for};
use fluxa_renderer::svg_icons::{ICON_SIZE, ICONS, LOGO_SIZE, LOGOS, rasterize_svg};
pub use fluxa_renderer::ui::{GamepadButton, Key};
use fluxa_renderer::ui::{PointerButton, UiAction, UiEvent, UiNode, UiNodeKind, UiTree};
use fluxa_ui::{
    AnimatedTexture, ArtworkPriority, CalendarModel, DetailModel, DiscoverModel, HomeAssets, HomeCard, HomeHero,
    HomeLayout, HomeModel, LibraryModel, LibraryTab, SettingsModel, UiFormFactorJson, Viewport,
    draw_calendar, draw_detail, PlayerModel, draw_discover, draw_home, draw_library, draw_player,
    draw_settings,
};
use serde::Serialize;
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

mod player;
mod poster_data;
mod presence;
mod profiles;
mod projection;
mod trailer;
pub use profiles::ImagePicker;

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
    rendered_layout: Option<(String, HomeLayout)>,
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
    active_scroll: Option<HomeScrollTarget>,
    scroll_velocity: f32,
    scroll_animation_at: Instant,
    route: String,
    screen_scroll_offsets: HashMap<String, f32>,
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
    video: Option<Box<dyn VideoBackend>>,
    fullscreen_toggle: bool,
    profiles: Option<fluxa_ui::ProfilesModel>,
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
    if state.route == "detail" && !state.detail.title.is_empty() {
        return presence::Presence::Viewing {
            title: state.detail.title.clone(),
            poster: state
                .detail
                .poster_url
                .clone()
                .filter(|url| url.starts_with("http")),
        };
    }
    let label = match state.route.as_str() {
        "library" => "Browsing the library",
        "discover" => "Discovering",
        "calendar" => "Checking the calendar",
        "settings" => "In settings",
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

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum NativeAction {
    CoreCommand {
        command: Value,
    },
    Navigate {
        destination: String,
    },
    Detail {
        id: String,
        item_type: String,
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

const MAX_ARTWORK_SIDE: u32 = 1536;
// Android uses the same initial-document prefetch contract as desktop. This
// is still bounded, but avoids evicting visible cards while the remaining
// shelves finish decoding.
const MAX_ARTWORK_TEXTURES: usize = 128;

const NODE_HOME: u64 = 10;
const NODE_LIBRARY: u64 = 11;
const NODE_DISCOVER: u64 = 12;
const NODE_CALENDAR: u64 = 13;
const NODE_PROFILE: u64 = 14;
const NODE_PLAY: u64 = 20;
const NODE_MORE_INFO: u64 = 21;
const NODE_CARD_BASE: u64 = 100;

struct ArtworkLoader {
    fetcher: ArtworkFetcher,
    textures: HashMap<String, egui::TextureHandle>,
    animations: HashMap<String, ArtworkAnimation>,
    animation_checked: HashSet<String>,
    active_animations: HashSet<String>,
    animation_slots: HashSet<String>,
    latest_keys: HashMap<String, String>,
    texture_last_used: HashMap<String, u64>,
    transparent_pixel_ratio: HashMap<String, f32>,
    tones: HashMap<String, [[u8; 3]; 2]>,
    access_counter: u64,
    disabled: bool,
}

struct ArtworkAnimation {
    frames: Vec<fluxa_artwork::AnimatedAtlasFrame>,
    frame_index: usize,
    started_at: Option<Instant>,
    next_frame_at: Instant,
}

impl ArtworkLoader {
    fn new(cache_dir: Option<PathBuf>) -> Self {
        Self {
            fetcher: ArtworkFetcher::new(cache_dir, 4),
            textures: HashMap::new(),
            animations: HashMap::new(),
            animation_checked: HashSet::new(),
            active_animations: HashSet::new(),
            animation_slots: HashSet::new(),
            latest_keys: HashMap::new(),
            texture_last_used: HashMap::new(),
            transparent_pixel_ratio: HashMap::new(),
            tones: HashMap::new(),
            access_counter: 0,
            disabled: std::env::var_os("FLUXA_NATIVE_DISABLE_ARTWORK").is_some(),
        }
    }

    fn poll(&mut self, context: &egui::Context) {
        for prepared in self.fetcher.poll(2) {
            let url = prepared.source_url;
            let target_size = prepared.target_size;
            let animation_requested = prepared.animation_requested;
            let animation_atlas = prepared.animation_atlas;
            let image_pixels = animation_atlas
                .as_ref()
                .map(|atlas| &atlas.image)
                .unwrap_or(&prepared.image);
            let base_key = fluxa_artwork::request_key(&url, target_size);
            let key = if animation_requested {
                format!("{base_key}#animated-atlas")
            } else {
                base_key
            };
            // Artwork is premultiplied before upload. Passing it through the
            // straight-alpha constructor applies alpha a second time and
            // creates the dark/bright fringe visible around transparent logos.
            let max_side = context.input(|input| input.max_texture_side) as u32;
            if image_pixels.width() > max_side || image_pixels.height() > max_side {
                host_log(format!(
                    "Artwork skipped: {}x{} exceeds {max_side}",
                    image_pixels.width(),
                    image_pixels.height()
                ));
                continue;
            }
            let image = egui::ColorImage::from_rgba_premultiplied(
                [
                    image_pixels.width() as usize,
                    image_pixels.height() as usize,
                ],
                image_pixels.as_raw(),
            );
            let transparent_ratio = prepared.transparent_ratio;
            let tones = prepared.tones;
            let texture = context.load_texture(
                format!("fluxa-native-artwork-{key}"),
                image,
                egui::TextureOptions::LINEAR,
            );
            if !animation_requested {
                self.latest_keys.insert(url, key.clone());
            }
            if animation_requested {
                self.animation_checked.insert(key.clone());
            }
            self.access_counter = self.access_counter.wrapping_add(1);
            self.texture_last_used
                .insert(key.clone(), self.access_counter);
            self.transparent_pixel_ratio
                .insert(key.clone(), transparent_ratio);
            self.tones.insert(key.clone(), tones);
            self.textures.insert(key.clone(), texture);
            if let Some(atlas) = animation_atlas.filter(|atlas| atlas.frames.len() > 1) {
                self.animations.insert(
                    key,
                    ArtworkAnimation {
                        frames: atlas.frames,
                        frame_index: 0,
                        started_at: None,
                        next_frame_at: Instant::now(),
                    },
                );
            }
            while self.textures.len() > MAX_ARTWORK_TEXTURES {
                let Some(oldest) = self
                    .texture_last_used
                    .iter()
                    .min_by_key(|(_, last_used)| *last_used)
                    .map(|(key, _)| key.clone())
                else {
                    break;
                };
                self.textures.remove(&oldest);
                self.texture_last_used.remove(&oldest);
                self.transparent_pixel_ratio.remove(&oldest);
                self.tones.remove(&oldest);
                self.animations.remove(&oldest);
                self.animation_checked.remove(&oldest);
            }
            context.request_repaint();
        }
        let now = Instant::now();
        let active = self.active_animations.iter().cloned().collect::<Vec<_>>();
        let mut next_frame_in = None;
        for key in active {
            let Some(animation) = self.animations.get_mut(&key) else {
                continue;
            };
            if let Some(started_at) = animation.started_at
                && now >= animation.next_frame_at
            {
                let (frame_index, next_frame_at) =
                    fluxa_artwork::animation_frame_at(&animation.frames, started_at, now)
                        .unwrap_or((animation.frame_index, now + Duration::from_millis(100)));
                if frame_index != animation.frame_index {
                    animation.frame_index = frame_index;
                    context.request_repaint();
                }
                animation.next_frame_at = next_frame_at;
            }
            let wait = animation.next_frame_at.saturating_duration_since(now);
            next_frame_in = Some(next_frame_in.map_or(wait, |current: Duration| current.min(wait)));
        }
        if let Some(wait) = next_frame_in {
            context.request_repaint_after(wait.max(Duration::from_millis(1)));
        }
    }

    fn next_animation_frame(&self) -> Option<Instant> {
        self.active_animations
            .iter()
            .filter_map(|key| self.animations.get(key))
            .map(|animation| animation.next_frame_at)
            .min()
    }

    fn begin_frame(&mut self) {
        self.active_animations.clear();
        self.animation_slots.clear();
    }

    fn animated_texture_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkFetchPriority,
    ) -> Option<AnimatedTexture> {
        if self.disabled {
            return None;
        }
        let target_size = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_ARTWORK_SIDE),
            target_size[1].min(MAX_ARTWORK_SIDE),
        ]);
        let source_url = fluxa_artwork::normalize_url(url?)?;
        let base_key = fluxa_artwork::request_key(&source_url, target_size);
        let key = format!("{base_key}#animated-atlas");
        if self.animation_checked.contains(&key) && !self.animations.contains_key(&key) {
            return None;
        }
        if !self.animation_slots.contains(&key) {
            if self
                .fetcher
                .animated_request_is_backing_off(&source_url, target_size)
            {
                return None;
            }
            self.animation_slots.insert(key.clone());
        }
        self.access_counter = self.access_counter.wrapping_add(1);
        if let Some(texture) = self.textures.get(&key) {
            self.texture_last_used
                .insert(key.clone(), self.access_counter);
        }
        if self.animations.contains_key(&key) {
            let now = Instant::now();
            let animation = self.animations.get_mut(&key)?;
            animation.started_at.get_or_insert(now);
            let (frame_index, next_frame_at) =
                fluxa_artwork::animation_frame_at(&animation.frames, animation.started_at?, now)?;
            animation.frame_index = frame_index;
            animation.next_frame_at = next_frame_at;
            let frame = &animation.frames[frame_index];
            let texture = self.textures.get(&key)?;
            self.active_animations.insert(key.clone());
            return Some(AnimatedTexture {
                texture: texture.id(),
                uv: egui::Rect::from_min_max(
                    egui::pos2(frame.uv[0], frame.uv[1]),
                    egui::pos2(frame.uv[2], frame.uv[3]),
                ),
                image_size: frame.image_size,
            });
        }
        if self.animation_checked.contains(&key) {
            return None;
        }
        let _ = self
            .fetcher
            .request_animated(Some(&source_url), target_size, priority);
        None
    }

    fn prefetch_animated_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkFetchPriority,
    ) {
        if self.disabled {
            return;
        }
        let target_size = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_ARTWORK_SIDE),
            target_size[1].min(MAX_ARTWORK_SIDE),
        ]);
        let Some(source_url) = url.and_then(fluxa_artwork::normalize_url) else {
            return;
        };
        let key = format!(
            "{}#animated-atlas",
            fluxa_artwork::request_key(&source_url, target_size)
        );
        if self.textures.contains_key(&key)
            || self
                .fetcher
                .animated_request_is_backing_off(&source_url, target_size)
        {
            return;
        }
        let _ = self
            .fetcher
            .prefetch_animated(Some(&source_url), target_size, priority);
    }

    fn texture(&mut self, url: Option<&str>) -> Option<egui::TextureId> {
        self.texture_for_priority(url, [MAX_ARTWORK_SIDE; 2], ArtworkFetchPriority::Visible)
    }

    fn texture_for(&mut self, url: Option<&str>, target_size: [u32; 2]) -> Option<egui::TextureId> {
        self.texture_for_priority(url, target_size, ArtworkFetchPriority::Visible)
    }

    fn texture_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkFetchPriority,
    ) -> Option<egui::TextureId> {
        if self.disabled {
            return None;
        }
        let target_size = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_ARTWORK_SIDE),
            target_size[1].min(MAX_ARTWORK_SIDE),
        ]);
        let url = fluxa_artwork::normalize_url(url?)?;
        let key = fluxa_artwork::request_key(&url, target_size);
        self.access_counter = self.access_counter.wrapping_add(1);
        if let Some(texture) = self.textures.get(&key) {
            self.texture_last_used.insert(key, self.access_counter);
            return Some(texture.id());
        }
        let _ = self.fetcher.request(Some(&url), target_size, priority);
        None
    }

    fn size(&self, url: Option<&str>) -> Option<[u32; 2]> {
        let url = fluxa_artwork::normalize_url(url?)?;
        self.latest_keys
            .get(&url)
            .and_then(|key| self.textures.get(key))
            .map(|texture| texture.size().map(|side| side as u32))
    }

    fn tones(&self, url: Option<&str>) -> Option<[[u8; 3]; 2]> {
        let url = fluxa_artwork::normalize_url(url?)?;
        self.latest_keys
            .get(&url)
            .and_then(|key| self.tones.get(key))
            .copied()
    }

    fn cached_texture(&self, url: Option<&str>) -> Option<egui::TextureId> {
        let url = fluxa_artwork::normalize_url(url?)?;
        self.latest_keys
            .get(&url)
            .and_then(|key| self.textures.get(key))
            .map(egui::TextureHandle::id)
    }

    fn is_real_logo(&self, url: Option<&str>) -> bool {
        let Some(url) = url.and_then(fluxa_artwork::normalize_url) else {
            return false;
        };
        self.latest_keys
            .get(&url)
            .and_then(|key| self.transparent_pixel_ratio.get(key))
            .copied()
            .map(|ratio| ratio >= 0.85)
            .unwrap_or(true)
    }
}

struct Gpu {
    instance: wgpu::Instance,
    _surface: NativeSurface,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    clear_color: wgpu::Color,
    egui_context: egui::Context,
    egui_renderer: EguiWgpuBackend,
    icons: SvgIconRegistry,
    background_texture: egui::TextureHandle,
    brand_mark: Option<egui::TextureHandle>,
    ambient_glow: egui::TextureHandle,
    artwork: ArtworkLoader,
    started_at: Instant,
    density: f32,
    adapter_name: String,
}

struct FrameOutput {
    layout: HomeLayout,
    cursor: egui::CursorIcon,
    wants_keyboard: bool,
    repaint_delay: Duration,
}

struct HostAssets<'a> {
    background: egui::TextureId,
    brand_mark: Option<egui::TextureId>,
    ambient_glow: egui::TextureId,
    accent: Option<egui::Color32>,
    artwork: &'a mut ArtworkLoader,
    icons: &'a SvgIconRegistry,
    profile_name: Option<&'a str>,
    profile_avatar_url: Option<&'a str>,
    custom_background: Option<&'a str>,
}

impl HomeAssets for HostAssets<'_> {
    fn background(&self) -> egui::TextureId {
        self.background
    }
    fn active_profile_name(&self) -> Option<&str> {
        self.profile_name
    }
    fn brand_mark(&self) -> Option<egui::TextureId> {
        self.brand_mark
    }
    fn ambient_glow(&self) -> Option<egui::TextureId> {
        Some(self.ambient_glow)
    }
    fn accent_color(&self) -> Option<egui::Color32> {
        self.accent
    }
    fn custom_background_url(&self) -> Option<&str> {
        self.custom_background
    }
    fn active_profile_avatar_url(&self) -> Option<&str> {
        self.profile_avatar_url
    }
    fn texture(&mut self, url: Option<&str>) -> Option<egui::TextureId> {
        self.artwork.texture(url)
    }
    fn texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) -> Option<egui::TextureId> {
        self.artwork.texture_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        )
    }
    fn animated_texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) -> Option<AnimatedTexture> {
        self.artwork.animated_texture_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        )
    }
    fn prefetch_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) {
        let _ = self.artwork.texture_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        );
    }
    fn prefetch_animated_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) {
        self.artwork.prefetch_animated_for_priority(
            url,
            target_size,
            match priority {
                ArtworkPriority::Hero => ArtworkFetchPriority::Hero,
                ArtworkPriority::Visible => ArtworkFetchPriority::Visible,
                ArtworkPriority::Prefetch => ArtworkFetchPriority::Prefetch,
            },
        );
    }
    fn artwork_tones(&self, url: Option<&str>) -> Option<[[u8; 3]; 2]> {
        self.artwork.tones(url)
    }
    fn texture_size(&self, url: Option<&str>) -> Option<[u32; 2]> {
        self.artwork.size(url)
    }
    fn cached_texture(&self, url: Option<&str>) -> Option<egui::TextureId> {
        self.artwork.cached_texture(url)
    }
    fn icon(&self, name: &str) -> Option<egui::TextureId> {
        self.icons.texture(name)
    }
    fn logo(&self, name: &str) -> Option<(egui::TextureId, egui::Vec2)> {
        let texture = self.icons.textures.get(name)?;
        let [width, height] = texture.size();
        Some((texture.id(), egui::vec2(width as f32 / height as f32, 1.0)))
    }
}

struct SvgIconRegistry {
    textures: HashMap<&'static str, egui::TextureHandle>,
}

impl SvgIconRegistry {
    fn new(context: &egui::Context) -> Self {
        let mut textures = HashMap::new();
        let sized = ICONS
            .iter()
            .map(|(name, svg)| (name, svg, ICON_SIZE))
            .chain(LOGOS.iter().map(|(name, svg)| (name, svg, LOGO_SIZE)));
        for (name, svg, size) in sized {
            host_log(format!("Preparing SVG icon: {name}"));
            let Ok(image) = rasterize_svg(svg.as_bytes(), size) else {
                host_log(format!("failed to rasterize shared SVG icon {name}"));
                continue;
            };
            host_log(format!("Rasterized SVG icon: {name}"));
            let image = egui::ColorImage::from_rgba_premultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            );
            let texture = context.load_texture(
                format!("fluxa-shared-svg-icon-{name}"),
                image,
                egui::TextureOptions::LINEAR,
            );
            textures.insert(*name, texture);
            host_log(format!("Uploaded SVG icon: {name}"));
        }
        Self { textures }
    }

    fn texture(&self, name: &str) -> Option<egui::TextureId> {
        self.textures.get(name).map(egui::TextureHandle::id)
    }
}

impl Gpu {
    async fn create(
        surface: NativeSurface,
        size: [u32; 2],
        density: f32,
        artwork_cache_dir: Option<PathBuf>,
        opener: Option<DeviceOpener>,
    ) -> Result<Self, String> {
        host_log(format!(
            "GPU init start: surface={}x{}, density={density:.2}",
            size[0], size[1]
        ));
        let mut errors = Vec::new();
        for &backend in surface.backends {
            host_log(format!("Trying {} backend", backend.label()));
            match Self::create_for_backend(
                &surface,
                size,
                density,
                artwork_cache_dir.clone(),
                backends_for(backend),
                opener.clone(),
            )
            .await
            {
                Ok(gpu) => {
                    host_log(format!(
                        "GPU ready: {} ({})",
                        gpu.adapter_name,
                        backend.label()
                    ));
                    return Ok(gpu);
                }
                Err(error) => {
                    host_log(format!("{} backend failed: {error}", backend.label()));
                    errors.push(format!("{}: {error}", backend.label()));
                }
            }
        }
        Err(format!("no GPU backend succeeded ({})", errors.join("; ")))
    }

    async fn create_for_backend(
        native_surface: &NativeSurface,
        size: [u32; 2],
        density: f32,
        artwork_cache_dir: Option<PathBuf>,
        backends: wgpu::Backends,
        opener: Option<DeviceOpener>,
    ) -> Result<Self, String> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = backends;
        let instance = wgpu::Instance::new(descriptor);
        let surface = unsafe {
            instance
                .create_surface_unsafe((native_surface.target)()?)
                .map_err(|error| error.to_string())?
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|error| error.to_string())?;
        let info = adapter.get_info();
        let adapter_name = info.name.clone();
        host_log(format!(
            "Adapter selected: {:?} / {}",
            info.backend, info.name
        ));
        let required_limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
        } else {
            wgpu::Limits::default()
        };
        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("fluxa-host-device"),
            required_features: wgpu::Features::empty(),
            required_limits,
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        };
        let (device, queue) = match opener {
            Some(opener) => opener(&adapter, &device_descriptor)?,
            None => adapter
                .request_device(&device_descriptor)
                .await
                .map_err(|error| error.to_string())?,
        };
        host_log("Device requested successfully");
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| capabilities.formats.first().copied())
            .ok_or_else(|| "surface has no compatible format".to_owned())?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size[0].max(1),
            height: size[1].max(1),
            present_mode: if capabilities.present_modes.contains(&wgpu::PresentMode::Mailbox) {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::Fifo
            },
            alpha_mode: capabilities
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        host_log(format!(
            "Surface configured: {:?}, {}x{}, {:?} of {:?}",
            format, config.width, config.height, config.present_mode, capabilities.present_modes
        ));
        let clear = fluxa_renderer::theme::theme("fluxa-dark")
            .and_then(|theme| theme.color("background"))
            .map(|color| wgpu::Color {
                r: f64::from(color.r),
                g: f64::from(color.g),
                b: f64::from(color.b),
                a: f64::from(color.a),
            })
            .unwrap_or(wgpu::Color {
                r: 0.0235,
                g: 0.0235,
                b: 0.0235,
                a: 1.0,
            });
        let egui_context = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "archivo".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../../apps/android/app/src/main/res/font/archivo.ttf"
            ))
            .into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .splice(0..0, ["archivo".to_owned()]);
        fonts
            .families
            .entry(egui::FontFamily::Name("archivo".into()))
            .or_default()
            .splice(0..0, ["archivo".to_owned()]);
        egui_context.set_fonts(fonts);
        let background_image = {
            let image = fluxa_renderer::ambient_background();
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_raw(),
            )
        };
        let background_texture = egui_context.load_texture(
            "fluxa-native-background",
            background_image,
            egui::TextureOptions::LINEAR,
        );
        let brand_mark = image::load_from_memory(BRAND_MARK_BYTES).ok().map(|image| {
            let image = image.to_rgba8();
            egui_context.load_texture(
                "fluxa-brand-mark",
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                ),
                egui::TextureOptions::LINEAR,
            )
        });
        let ambient_glow = {
            let image = fluxa_renderer::ambient_glow();
            egui_context.load_texture(
                "fluxa-ambient-glow",
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                ),
                egui::TextureOptions::LINEAR,
            )
        };
        let egui_renderer = EguiWgpuBackend::new(&device, format);
        host_log("egui renderer created");
        let icons = SvgIconRegistry::new(&egui_context);
        fluxa_ui::set_rating_logos(
            &egui_context,
            ["imdb", "mdblist"]
                .into_iter()
                .filter_map(|name| {
                    let texture = icons.textures.get(name)?;
                    let [width, height] = texture.size();
                    Some((name, texture.id(), egui::vec2(width as f32 / height as f32, 1.0)))
                })
                .collect(),
        );
        host_log(format!(
            "Shared SVG icons uploaded: {}",
            icons.textures.len()
        ));
        Ok(Self {
            instance: instance,
            _surface: native_surface.clone(),
            surface,
            device,
            queue,
            config,
            clear_color: clear,
            egui_context,
            egui_renderer,
            icons,
            background_texture,
            brand_mark,
            ambient_glow,
            artwork: ArtworkLoader::new(artwork_cache_dir),
            started_at: Instant::now(),
            density,
            adapter_name,
        })
    }

    fn resize(&mut self, size: [u32; 2]) {
        self.config.width = size[0].max(1);
        self.config.height = size[1].max(1);
        self.surface.configure(&self.device, &self.config);
    }

    fn render(
        &mut self,
        route: &str,
        home: &HomeModel,
        library: &LibraryModel,
        library_tab: LibraryTab,
        discover: &DiscoverModel,
        calendar: &CalendarModel,
        detail: &DetailModel,
        settings: &SettingsModel,
        mut profiles: Option<&mut fluxa_ui::ProfilesModel>,
        custom_background: Option<&str>,
        player: Option<&PlayerModel>,
        focused: Option<u64>,
        safe_bottom: f32,
        scroll_y: f32,
        events: Vec<egui::Event>,
        modifiers: egui::Modifiers,
        pre_present: Option<&PrePresent>,
        timer: &mut FrameTimer,
    ) -> Result<FrameOutput, String> {
        self.artwork.poll(&self.egui_context);
        self.artwork.begin_frame();
        timer.mark("artwork");
        let screen_size = [self.config.width, self.config.height];
        let logical_size = [
            (screen_size[0] as f32 / self.density).round().max(1.0) as u32,
            (screen_size[1] as f32 / self.density).round().max(1.0) as u32,
        ];
        let raw_input = egui::RawInput {
            max_texture_side: Some(self.device.limits().max_texture_dimension_2d as usize),
            viewports: std::iter::once((
                egui::ViewportId::ROOT,
                egui::ViewportInfo {
                    native_pixels_per_point: Some(self.density),
                    ..Default::default()
                },
            ))
            .collect(),
            screen_rect: Some(EguiRect::from_min_size(
                Pos2::ZERO,
                Vec2::new(logical_size[0] as f32, logical_size[1] as f32),
            )),
            time: Some(self.started_at.elapsed().as_secs_f64()),
            events,
            modifiers,
            focused: true,
            ..Default::default()
        };
        fluxa_ui::set_poster_overlays(&self.egui_context, settings.poster_overlays());
        fluxa_ui::set_poster_personal(&self.egui_context, library.personal.clone());
        let mut rendered_layout = HomeLayout::default();
        let output = self.egui_context.run_ui(raw_input, |ui| {
            let mut assets = HostAssets {
                background: self.background_texture.id(),
                brand_mark: self.brand_mark.as_ref().map(egui::TextureHandle::id),
                ambient_glow: self.ambient_glow.id(),
                accent: home.accent,
                artwork: &mut self.artwork,
                icons: &self.icons,
                profile_name: home.profile_name.as_deref(),
                profile_avatar_url: home.profile_avatar_url.as_deref(),
                custom_background,
            };
            let viewport = Viewport::new(logical_size[0], logical_size[1], home.form_factor.into())
                .with_safe_bottom(safe_bottom)
                .with_scroll_y(scroll_y);
            if let Some(profiles) = profiles.as_deref_mut() {
                rendered_layout = fluxa_ui::draw_profiles(ui.ctx(), viewport, profiles, &mut assets);
            } else if let Some(player) = player {
                rendered_layout = draw_player(ui.ctx(), viewport, player, &mut assets, focused);
            } else if route == "library" {
                rendered_layout = draw_library(
                    ui.ctx(),
                    viewport,
                    library,
                    library_tab,
                    &mut assets,
                    focused,
                );
            } else if route == "discover" {
                rendered_layout = draw_discover(ui.ctx(), viewport, discover, &mut assets, focused);
            } else if route == "calendar" {
                rendered_layout = draw_calendar(ui.ctx(), viewport, calendar, &mut assets, focused);
            } else if route == "detail" {
                rendered_layout = draw_detail(ui.ctx(), viewport, detail, &mut assets, focused);
            } else if route == "settings" {
                rendered_layout = draw_settings(ui.ctx(), viewport, settings, &assets, focused);
            } else {
                rendered_layout = draw_home(ui.ctx(), viewport, home, &mut assets, focused);
            }
        });
        timer.mark("draw");
        let paint_jobs = self
            .egui_context
            .tessellate(output.shapes, output.pixels_per_point);
        self.egui_renderer
            .apply_texture_deltas(&self.device, &self.queue, &output.textures_delta);
        timer.mark("upload");
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: screen_size,
            pixels_per_point: output.pixels_per_point,
        };
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout => return Err("surface timeout".to_owned()),
            wgpu::CurrentSurfaceTexture::Occluded => return Err("surface occluded".to_owned()),
            wgpu::CurrentSurfaceTexture::Outdated => return Err("surface outdated".to_owned()),
            wgpu::CurrentSurfaceTexture::Lost => return Err("surface lost".to_owned()),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("surface validation error".to_owned());
            }
        };
        timer.mark("acquire");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("fluxa-android-renderer-frame"),
            });
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fluxa-android-egui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.egui_renderer
                .render(&mut pass.forget_lifetime(), &paint_jobs, &screen_descriptor);
        }
        self.queue.submit([encoder.finish()]);
        if let Some(pre_present) = pre_present {
            pre_present();
        }
        frame.present();
        timer.mark("present");
        self.egui_renderer
            .free_texture_deltas(&output.textures_delta);
        Ok(FrameOutput {
            layout: rendered_layout,
            cursor: output.platform_output.cursor_icon,
            wants_keyboard: self.egui_context.egui_wants_keyboard_input(),
            repaint_delay: output
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .map_or(Duration::ZERO, |viewport| viewport.repaint_delay),
        })
    }
}

const BRAND_MARK_BYTES: &[u8] = include_bytes!("../../../apps/desktop/public/fluxa.png");


fn rebuild_home_ui(ui: &mut UiTree, [width, height]: [u32; 2], home: &HomeModel, safe_bottom: f32) {
    if ui.node(NODE_PLAY).is_some() {
        return;
    }
    *ui = UiTree::default();
    let root = ui.root();
    if let Some(node) = ui.node_mut(root) {
        node.bounds = fluxa_renderer::Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        };
    }
    let layout = fluxa_ui::home_layout(
        Viewport::new(width, height, home.form_factor.into()).with_safe_bottom(safe_bottom),
        home,
    );
    for (id, rect) in layout.focusable {
        let label = match id {
            NODE_HOME => "Home",
            NODE_LIBRARY => "Library",
            NODE_DISCOVER => "Discover",
            NODE_CALENDAR => "Calendar",
            NODE_PROFILE => "Profile",
            NODE_PLAY => "Play",
            NODE_MORE_INFO => "More info",
            _ => home
                .card_at((id - NODE_CARD_BASE) as usize)
                .map(|card| card.title.as_str())
                .unwrap_or("Title"),
        };
        let _ = ui.add(
            root,
            UiNode::new(
                id,
                UiNodeKind::Button,
                fluxa_renderer::Rect {
                    x: rect.left(),
                    y: rect.top(),
                    width: rect.width(),
                    height: rect.height(),
                },
            )
            .focusable()
            .label(label),
        );
    }
    // Compose opens Home with the primary action focused only when a real
    // billboard exists. A loading/empty state must not draw a focus ring over
    // an unrelated row placeholder.
    let _ = ui.set_focus(if home.form_factor == UiFormFactorJson::Mobile {
        None
    } else if home.item_id.is_some() {
        Some(NODE_PLAY)
    } else {
        Some(NODE_HOME)
    });
}

fn logical_surface_size(state: &RendererState) -> [u32; 2] {
    [
        (state.size[0] as f32 / state.scale()).round().max(1.0) as u32,
        (state.size[1] as f32 / state.scale()).round().max(1.0) as u32,
    ]
}

fn rebuild_current_ui(state: &mut RendererState) {
    if state
        .ui
        .node(state.ui.root())
        .is_some_and(|root| !root.children.is_empty())
    {
        return;
    }
    let size = logical_surface_size(state);
    let cached_layout = state
        .rendered_layout
        .as_ref()
        .filter(|(route, _)| *route == active_route(state))
        .map(|(_, layout)| layout.clone());
    if let Some(layout) = cached_layout {
        // This path only runs when an input event has invalidated the
        // accessibility tree; the normal frame path borrows the layout.
        rebuild_ui_from_layout(state, &layout, size);
        return;
    }
    if state.route != "home" || state.player.is_some() || state.profiles.is_some() {
        return;
    }
    rebuild_home_ui(&mut state.ui, size, &state.home, state.safe_bottom);
}

fn label_for_node(state: &RendererState, node: u64) -> String {
    match node {
        fluxa_ui::NODE_HOME => "Home".to_owned(),
        fluxa_ui::NODE_LIBRARY => "Library".to_owned(),
        fluxa_ui::NODE_DISCOVER => "Discover".to_owned(),
        fluxa_ui::NODE_CALENDAR => "Calendar".to_owned(),
        fluxa_ui::NODE_PROFILE => "Profile and settings".to_owned(),
        fluxa_ui::NODE_LIBRARY_SEARCH => {
            fluxa_ui::localized("library.filter_placeholder", &state.library.language)
        }
        fluxa_ui::NODE_SETTINGS_ADDON_URL => fluxa_ui::localized("settings.addon_url", "en"),
        fluxa_ui::NODE_SETTINGS_SEARCH => fluxa_ui::localized("settings.search_placeholder", "en"),
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL => {
            fluxa_ui::localized("settings.plugin_url", "en")
        }
        id if fluxa_ui::poster_field(id).is_some() => fluxa_ui::poster_field(id)
            .map(|index| fluxa_ui::localized(fluxa_ui::POSTER_FIELDS[index].label, "en"))
            .unwrap_or_default(),
        fluxa_ui::NODE_SETTINGS_ADDON_INSTALL => {
            fluxa_ui::localized("settings.addon_install", "en")
        }
        fluxa_ui::NODE_SETTINGS_ADDON_REFRESH => {
            fluxa_ui::localized("settings.addon_refresh", "en")
        }
        fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL => {
            fluxa_ui::localized("settings.plugin_add", "en")
        }
        id if (fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE
            ..fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + 4)
            .contains(&id) =>
        {
            fluxa_ui::localized("settings.plugin_remove", "en")
        }
        id if (fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE
            ..fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE + 4)
            .contains(&id) =>
        {
            fluxa_ui::localized("settings.plugin_refresh", "en")
        }
        id if (fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE
            ..fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE + 4)
            .contains(&id) =>
        {
            let index = (id - fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE) as usize;
            let enabled = state
                .settings
                .plugins
                .get("scrapers")
                .and_then(Value::as_array)
                .and_then(|items| items.get(index))
                .and_then(|scraper| scraper.get("enabled"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            fluxa_ui::localized(
                if enabled {
                    "settings.plugin_enabled"
                } else {
                    "settings.plugin_disabled"
                },
                "en",
            )
        }
        fluxa_ui::NODE_LIBRARY_SORT => fluxa_ui::localized(
            match state.library_sort.as_str() {
                "title" => "library.sort_title",
                "rating" => "library.sort_rating",
                _ => "library.sort_recent",
            },
            &state.library.language,
        ),
        fluxa_ui::NODE_CALENDAR_PREV => "Previous month".to_owned(),
        fluxa_ui::NODE_CALENDAR_NEXT => "Next month".to_owned(),
        fluxa_ui::NODE_DETAIL_BACK | fluxa_ui::NODE_SETTINGS_BACK => "Back".to_owned(),
        fluxa_ui::NODE_SETTINGS_SWITCH_PROFILE => "Switch profiles".to_owned(),
        fluxa_ui::NODE_DETAIL_PLAY => "Play".to_owned(),
        fluxa_ui::NODE_DETAIL_WATCHLIST => {
            if state.detail.in_watchlist {
                "Remove from watchlist".to_owned()
            } else {
                "Add to watchlist".to_owned()
            }
        }
        id if (fluxa_ui::NODE_LIBRARY_TAB_BASE
            ..fluxa_ui::NODE_LIBRARY_TAB_BASE + LibraryTab::ALL.len() as u64)
            .contains(&id) =>
        {
            LibraryTab::ALL[(id - fluxa_ui::NODE_LIBRARY_TAB_BASE) as usize]
                .label()
                .to_owned()
        }
        id if id == fluxa_ui::NODE_DISCOVER_TYPE_BASE => {
            if id == fluxa_ui::NODE_DISCOVER_TYPE_BASE {
                "Movies".to_owned()
            } else {
                "Series".to_owned()
            }
        }
        id if id == fluxa_ui::NODE_DISCOVER_CATALOG_BASE => state
            .discover
            .catalogs
            .iter()
            .filter(|catalog| catalog.content_type == state.discover.content_type)
            .nth((id - fluxa_ui::NODE_DISCOVER_CATALOG_BASE) as usize)
            .map(|catalog| catalog.label.clone())
            .unwrap_or_else(|| "Catalog".to_owned()),
        id if (fluxa_ui::NODE_CALENDAR_DAY_BASE..fluxa_ui::NODE_CALENDAR_DAY_BASE + 32)
            .contains(&id) =>
        {
            format!("Day {}", id - fluxa_ui::NODE_CALENDAR_DAY_BASE)
        }
        fluxa_ui::NODE_CALENDAR_CLOSE_DAY => "Close day".to_owned(),
        id if (fluxa_ui::NODE_CALENDAR_EVENT_BASE..fluxa_ui::NODE_CALENDAR_EVENT_BASE + 10)
            .contains(&id) =>
        {
            state
                .calendar
                .selected_day
                .and_then(|day| {
                    state
                        .calendar
                        .entries_for_day(day)
                        .nth((id - fluxa_ui::NODE_CALENDAR_EVENT_BASE) as usize)
                })
                .map(|entry| entry.card.title.clone())
                .unwrap_or_else(|| "Release".to_owned())
        }
        id if (fluxa_ui::NODE_DETAIL_SIMILAR_BASE..fluxa_ui::NODE_DETAIL_SIMILAR_BASE + 16)
            .contains(&id) =>
        {
            state
                .detail
                .similar
                .get((id - fluxa_ui::NODE_DETAIL_SIMILAR_BASE) as usize)
                .map(|card| card.title.clone())
                .unwrap_or_else(|| "Recommended title".to_owned())
        }
        id if (fluxa_ui::NODE_SETTINGS_ROW_BASE
            ..fluxa_ui::NODE_SETTINGS_ROW_BASE
                + fluxa_ui::SETTINGS_SECTIONS
                    .iter()
                    .map(|section| section.rows.len())
                    .sum::<usize>() as u64)
            .contains(&id) =>
        {
            fluxa_ui::settings_row_by_index((id - fluxa_ui::NODE_SETTINGS_ROW_BASE) as usize)
                .map(|row| row.label.to_owned())
                .unwrap_or_else(|| "Setting".to_owned())
        }
        id if (fluxa_ui::NODE_SETTINGS_SECTION_BASE
            ..fluxa_ui::NODE_SETTINGS_SECTION_BASE + fluxa_ui::SETTINGS_SECTIONS.len() as u64)
            .contains(&id) =>
        {
            fluxa_ui::SETTINGS_SECTIONS
                .get((id - fluxa_ui::NODE_SETTINGS_SECTION_BASE) as usize)
                .map(|section| section.title.to_owned())
                .unwrap_or_else(|| "Settings".to_owned())
        }
        id if id >= NODE_CARD_BASE => {
            let index = (id - NODE_CARD_BASE) as usize;
            let card = match state.route.as_str() {
                "library" => state.library.cards(state.library_tab).get(index),
                "discover" => state.discover.results.get(index),
                _ => state.home.card_at(index),
            };
            card.map(|card| card.title.clone())
                .unwrap_or_else(|| "Media".to_owned())
        }
        _ => "Button".to_owned(),
    }
}

fn rebuild_ui_from_layout(
    state: &mut RendererState,
    layout: &HomeLayout,
    [width, height]: [u32; 2],
) {
    let target_count = layout
        .focusable
        .iter()
        .filter(|(_, bounds)| {
            // Keep off-screen targets in the focus graph. Pointer hit testing
            // still ignores them because their bounds do not contain an
            // on-screen position, while D-pad navigation can select a card
            // and ask the scroll controller to reveal it.
            !bounds.is_negative() && bounds.width() >= 1.0 && bounds.height() >= 1.0
        })
        .count();
    let root = state.ui.root();
    let same_targets = state
        .ui
        .node(root)
        .is_some_and(|root_node| root_node.children.len() == target_count)
        && layout
            .focusable
            .iter()
            .filter(|(_, bounds)| {
                !bounds.is_negative() && bounds.width() >= 1.0 && bounds.height() >= 1.0
            })
            .zip(
                state
                    .ui
                    .node(root)
                    .into_iter()
                    .flat_map(|root_node| root_node.children.iter()),
            )
            .all(|((id, _), child_id)| id == child_id && state.ui.node(*id).is_some());
    if same_targets {
        // Scrolling changes focus-target coordinates continuously, but not
        // the focus graph. Update those bounds in place instead of allocating
        // a Vec and rebuilding the whole UiTree on every scroll frame.
        for (id, bounds) in layout.focusable.iter().filter(|(_, bounds)| {
            !bounds.is_negative() && bounds.width() >= 1.0 && bounds.height() >= 1.0
        }) {
            if let Some(node) = state.ui.node_mut(*id) {
                node.bounds = fluxa_renderer::Rect {
                    x: bounds.left(),
                    y: bounds.top(),
                    width: bounds.width(),
                    height: bounds.height(),
                };
            }
        }
        if let Some(root_node) = state.ui.node_mut(root) {
            root_node.bounds.width = width as f32;
            root_node.bounds.height = height as f32;
        }
        return;
    }

    let targets = layout
        .focusable
        .iter()
        .filter_map(|(id, bounds)| {
            (!bounds.is_negative() && bounds.width() >= 1.0 && bounds.height() >= 1.0)
                .then_some((*id, *bounds))
        })
        .collect::<Vec<_>>();
    let previous_focus = state.ui.focused();
    let retained_focus = targets
        .iter()
        .any(|(id, _)| Some(*id) == previous_focus)
        .then_some(previous_focus)
        .flatten();
    let first_target = targets.first().map(|(id, _)| *id);
    let mut ui = UiTree::default();
    let root = ui.root();
    if let Some(node) = ui.node_mut(root) {
        node.bounds = fluxa_renderer::Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        };
    }
    for (id, bounds) in targets {
        let Some(node_id) = ui.add(
            root,
            UiNode::new(
                id,
                if id == fluxa_ui::NODE_LIBRARY_SEARCH
                    || id == fluxa_ui::NODE_SETTINGS_ADDON_URL
                    || id == fluxa_ui::NODE_SETTINGS_SEARCH
                    || id == fluxa_ui::NODE_SETTINGS_PLUGIN_URL
                    || fluxa_ui::poster_field(id).is_some()
                {
                    UiNodeKind::Input
                } else {
                    UiNodeKind::Button
                },
                fluxa_renderer::Rect {
                    x: bounds.left(),
                    y: bounds.top(),
                    width: bounds.width(),
                    height: bounds.height(),
                },
            )
            .focusable()
            .label(label_for_node(state, id)),
        ) else {
            continue;
        };
        if matches!(
            node_id,
            fluxa_ui::NODE_HOME
                | fluxa_ui::NODE_LIBRARY
                | fluxa_ui::NODE_DISCOVER
                | fluxa_ui::NODE_CALENDAR
                | fluxa_ui::NODE_PROFILE
        ) {
            if let Some(node) = ui.node_mut(node_id) {
                node.order = 10_000;
            }
        }
    }
    let fallback = match state.route.as_str() {
        "library" => state
            .library
            .cards(state.library_tab)
            .first()
            .map(|_| NODE_CARD_BASE)
            .unwrap_or(fluxa_ui::NODE_LIBRARY_TAB_BASE),
        "discover" => state
            .discover
            .results
            .first()
            .map(|_| NODE_CARD_BASE)
            .unwrap_or(fluxa_ui::NODE_DISCOVER_TYPE_BASE),
        "calendar" => {
            if state.calendar.selected_day.is_some() {
                fluxa_ui::NODE_CALENDAR_CLOSE_DAY
            } else {
                fluxa_ui::NODE_CALENDAR_PREV
            }
        }
        "detail" => fluxa_ui::NODE_DETAIL_PLAY,
        "settings" => fluxa_ui::NODE_SETTINGS_BACK,
        _ if state.home.item_id.is_some() => fluxa_ui::NODE_PLAY,
        _ => fluxa_ui::NODE_HOME,
    };
    let fallback = if ui.node(fallback).is_some() {
        fallback
    } else {
        first_target.unwrap_or(fallback)
    };
    ui.set_focus(Some(retained_focus.unwrap_or(fallback)));
    state.ui = ui;
}

fn ensure_focused_visible(state: &mut RendererState) {
    let Some(focused) = state.ui.focused() else {
        return;
    };
    let Some(node) = state.ui.node(focused) else {
        return;
    };
    let is_navigation = matches!(
        focused,
        fluxa_ui::NODE_HOME
            | fluxa_ui::NODE_LIBRARY
            | fluxa_ui::NODE_DISCOVER
            | fluxa_ui::NODE_CALENDAR
            | fluxa_ui::NODE_PROFILE
    );
    if is_navigation {
        return;
    }
    let viewport = Viewport::new(
        (state.size[0] as f32 / state.scale()).round().max(1.0) as u32,
        (state.size[1] as f32 / state.scale()).round().max(1.0) as u32,
        state.home.form_factor.into(),
    )
    .with_safe_bottom(state.safe_bottom);
    let top_inset = 12.0;
    let bottom_inset = 12.0 + state.safe_bottom;
    let visible_top = top_inset;
    let visible_bottom = (viewport.height - bottom_inset).max(visible_top + 1.0);
    let node_top = node.bounds.y;
    let node_bottom = node.bounds.y + node.bounds.height;
    let delta = if node_top < visible_top {
        node_top - visible_top
    } else if node_bottom > visible_bottom {
        node_bottom - visible_bottom
    } else {
        0.0
    };
    if delta.abs() < 0.5 {
        return;
    }
    if state.route == "home" {
        let max_offset = fluxa_ui::home_scroll_max(viewport, &state.home);
        state.home.scroll_offset = (state.home.scroll_offset + delta).clamp(0.0, max_offset);
    } else {
        let route = state.route.clone();
        let max_offset = screen_scroll_max(state, viewport);
        let offset = state.screen_scroll_offsets.entry(route).or_default();
        *offset = (*offset + delta).clamp(0.0, max_offset);
    }
    // The next frame will draw the newly revealed target at its scrolled
    // position and rebuild the same UiTree with the updated geometry.
}

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
    state.projector.settle(projection.revision, state.core_snapshot_revision);
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
    let addon_url = std::mem::take(&mut state.settings.addon_url);
    let plugin_url = std::mem::take(&mut state.settings.plugin_url);
    let search = std::mem::take(&mut state.settings.search);
    let poster_fields = std::mem::take(&mut state.settings.poster_fields);
    state.settings = projection.settings;
    state.settings.active_section = settings_section.min(fluxa_ui::SETTINGS_SECTIONS.len() - 1);
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
    if state.route == "discover"
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
    state.pending_native_actions.push(NativeAction::CoreCommand {
        command: request.clone(),
    });
}

fn sync_home_from_core_snapshot(state: &mut RendererState) {
    request_projection(state);
    if let Some(projection) = state.projector.take() {
        apply_projection(state, projection);
    }
}

fn core_value(method: &str, args: Value) -> Option<Value> {
    let raw = fluxa_core::ffi::core_invoke(method, &args.to_string());
    let envelope = serde_json::from_str::<Value>(&raw).ok()?;
    (envelope.get("ok").and_then(Value::as_bool) == Some(true))
        .then(|| envelope.get("value").cloned())
        .flatten()
}

fn native_action_for_node(
    node: u64,
    home: &HomeModel,
    library: &LibraryModel,
    library_tab: LibraryTab,
    discover: &DiscoverModel,
    calendar: &CalendarModel,
    detail: &DetailModel,
    settings: &SettingsModel,
    route: &str,
    hero: Option<&HomeHero>,
) -> Option<NativeAction> {
    let destination = match node {
        NODE_HOME => Some("home"),
        NODE_LIBRARY => Some("library"),
        NODE_DISCOVER => Some("discover"),
        NODE_CALENDAR => Some("calendar"),
        NODE_PROFILE => Some("settings"),
        _ => None,
    };
    if let Some(destination) = destination {
        return Some(NativeAction::Navigate {
            destination: destination.to_owned(),
        });
    }
    if route == "discover" {
        if node == fluxa_ui::NODE_DISCOVER_TYPE_BASE {
            return Some(NativeAction::DiscoverType {
                content_type: if node == fluxa_ui::NODE_DISCOVER_TYPE_BASE {
                    "movie".to_owned()
                } else {
                    "series".to_owned()
                },
            });
        }
        if node == fluxa_ui::NODE_DISCOVER_CATALOG_BASE {
            let index = (node - fluxa_ui::NODE_DISCOVER_CATALOG_BASE) as usize;
            if let Some(catalog) = discover
                .catalogs
                .iter()
                .filter(|catalog| catalog.content_type == discover.content_type)
                .nth(index)
            {
                return Some(NativeAction::DiscoverCatalog {
                    content_type: discover.content_type.clone(),
                    catalog_key: catalog.key.clone(),
                    extra_name: discover.selected_extra_name.clone(),
                    extra_value: discover.selected_extra_value.clone(),
                    query: discover.query.clone(),
                });
            }
        }
    }
    if route == "calendar" {
        if (fluxa_ui::NODE_CALENDAR_EVENT_BASE..fluxa_ui::NODE_CALENDAR_EVENT_BASE + 10)
            .contains(&node)
        {
            let day = calendar.selected_day?;
            let entry = calendar
                .entries_for_day(day)
                .nth((node - fluxa_ui::NODE_CALENDAR_EVENT_BASE) as usize)?;
            return Some(NativeAction::Detail {
                id: entry.card.id.clone()?,
                item_type: entry.card.item_type.clone()?,
                preview: entry.card.raw.clone(),
            });
        }
        if node == fluxa_ui::NODE_CALENDAR_PREV || node == fluxa_ui::NODE_CALENDAR_NEXT {
            let delta = if node == fluxa_ui::NODE_CALENDAR_PREV {
                -1
            } else {
                1
            };
            let mut year = calendar.year;
            let mut month = calendar.month + delta;
            if month < 1 {
                year -= 1;
                month = 12;
            } else if month > 12 {
                year += 1;
                month = 1;
            }
            return Some(NativeAction::CalendarMonth { year, month });
        }
    }
    if route == "settings" {
        if node == fluxa_ui::NODE_SETTINGS_BACK {
            return Some(NativeAction::Navigate {
                destination: "home".to_owned(),
            });
        }
        if node == fluxa_ui::NODE_SETTINGS_SWITCH_PROFILE {
            return Some(NativeAction::Navigate {
                destination: "profiles".to_owned(),
            });
        }
        if (fluxa_ui::NODE_SETTINGS_SECTION_BASE
            ..fluxa_ui::NODE_SETTINGS_SECTION_BASE + fluxa_ui::SETTINGS_SECTIONS.len() as u64)
            .contains(&node)
        {
            return Some(NativeAction::SettingsSection {
                index: (node - fluxa_ui::NODE_SETTINGS_SECTION_BASE) as usize,
            });
        }
        if (fluxa_ui::NODE_SETTINGS_ROW_BASE
            ..fluxa_ui::NODE_SETTINGS_ROW_BASE
                + fluxa_ui::SETTINGS_SECTIONS
                    .iter()
                    .map(|section| section.rows.len())
                    .sum::<usize>() as u64)
            .contains(&node)
        {
            let index = (node - fluxa_ui::NODE_SETTINGS_ROW_BASE) as usize;
            let setting = fluxa_ui::settings_row_by_index(index)?;
            return Some(NativeAction::SettingsChange {
                key: setting.key.to_owned(),
                value: settings.next_value_for(setting, home.form_factor.into()),
            });
        }
    }
    if route == "detail" {
        if node == fluxa_ui::NODE_DETAIL_BACK {
            return Some(NativeAction::Back);
        }
        if node == fluxa_ui::NODE_DETAIL_WATCHLIST {
            return Some(NativeAction::ToggleWatchlist {
                item: detail.item.clone(),
            });
        }
        if node == fluxa_ui::NODE_DETAIL_PLAY {
            if !detail.id.is_empty() {
                let first = detail
                    .episodes
                    .iter()
                    .find(|episode| episode.season > 0)
                    .or(detail.episodes.first())
                    .map(|episode| episode.id.clone());
                return Some(NativeAction::StartPlayback {
                    item: playback_item(&detail.item, detail.resume.as_ref(), first),
                });
            }
        }
        if node >= fluxa_ui::NODE_DETAIL_EPISODE_BASE {
            let episode = detail
                .episodes
                .get((node - fluxa_ui::NODE_DETAIL_EPISODE_BASE) as usize)?;
            let mut item = detail.item.clone();
            item.as_object_mut()?
                .insert("lastVideoId".to_owned(), episode.id.clone().into());
            return Some(NativeAction::StartPlayback { item });
        }
        if node >= fluxa_ui::NODE_DETAIL_SEASON_BASE {
            return None;
        }
        if (fluxa_ui::NODE_DETAIL_SIMILAR_BASE..fluxa_ui::NODE_DETAIL_CAST_BASE).contains(&node) {
            let card = detail
                .similar
                .get((node - fluxa_ui::NODE_DETAIL_SIMILAR_BASE) as usize)?;
            return Some(NativeAction::Detail {
                id: card.id.as_ref()?.clone(),
                item_type: card.item_type.as_ref()?.clone(),
                preview: card.raw.clone(),
            });
        }
    }
    let (id, item_type, preview) = if route == "library" && node >= NODE_CARD_BASE {
        let card = library
            .cards(library_tab)
            .get((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &card.raw)
    } else if route == "discover" && node >= NODE_CARD_BASE {
        let card = discover.results.get((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &card.raw)
    } else if node == NODE_PLAY || node == NODE_MORE_INFO {
        if let Some(hero) = hero
            && let (Some(id), Some(item_type)) = (hero.item_id.as_ref(), hero.item_type.as_ref())
        {
            if node == NODE_MORE_INFO {
                return Some(NativeAction::Detail {
                    id: id.clone(),
                    item_type: item_type.clone(),
                    preview: hero.raw.clone(),
                });
            }
            let series = matches!(item_type.as_str(), "series" | "tv" | "show");
            return Some(NativeAction::StartPlayback {
                item: playback_item(&hero.raw, home.resume_for(id), series.then(|| format!("{id}:1:1"))),
            });
        }
        (home.item_id.as_ref()?, home.item_type.as_ref()?, &Value::Null)
    } else if node >= NODE_CARD_BASE {
        let card = home.card_at((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &card.raw)
    } else {
        return None;
    };
    if node == NODE_PLAY {
        Some(NativeAction::Play {
            id: id.clone(),
            item_type: item_type.clone(),
        })
    } else {
        Some(NativeAction::Detail {
            id: id.clone(),
            item_type: item_type.clone(),
            preview: preview.clone(),
        })
    }
}

fn playback_item(item: &Value, resume: Option<&HomeCard>, first_video: Option<String>) -> Value {
    let mut item = item.clone();
    let Some(fields) = item.as_object_mut() else {
        return item;
    };
    if let Some(resume) = resume.and_then(|card| card.raw.as_object()) {
        for (key, value) in resume {
            if key.starts_with("last") || matches!(key.as_str(), "timeOffset" | "duration") {
                fields.insert(key.clone(), value.clone());
            }
        }
        if !fields.contains_key("lastVideoId")
            && let Some(video) = resume.get("videoId")
        {
            fields.insert("lastVideoId".to_owned(), video.clone());
        }
    } else if let Some(video) = first_video {
        fields.insert("lastVideoId".to_owned(), video.into());
    }
    item
}

fn refresh_library_view(state: &mut RendererState) {
    state.library.query = state.library_query.clone();
    state.library.sort_by = state.library_sort.clone();
    if let Some(snapshot) = state.core_snapshot.as_ref() {
        let library = snapshot
            .get("library")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if let Some(plan) = core_value(
            "libraryViewPlan",
            json!({
                "watchlist": library.get("watchlist"),
                "watching": library.get("continueWatching"),
                "completed": library.get("completed"),
                "dropped": library.get("dropped"),
                "favorites": library.get("liked"),
                "progress": library.get("progress"),
                "tab": state.library_tab.core_tab_key(),
                "query": state.library_query,
                "sortBy": state.library_sort,
            }),
        ) {
            state.library.apply_core_plan(&plan, state.library_tab);
        }
    }
}

fn settings_action_json(node: u64, settings: &SettingsModel) -> Option<Value> {
    if node == fluxa_ui::NODE_SETTINGS_ADDON_INSTALL {
        let url = settings.addon_url.trim();
        return (!url.is_empty()).then(
            || json!({"type":"addonInstallRequested", "transportUrl":url, "forceRefresh":false}),
        );
    }
    if node == fluxa_ui::NODE_SETTINGS_ADDON_REFRESH {
        return Some(
            json!({"type":"addonsRefreshRequested", "profile":settings.profile, "forceRefresh":true}),
        );
    }
    for (field, value) in fluxa_ui::POSTER_FIELDS.iter().zip(&settings.poster_fields) {
        if node == field.save {
            return Some(json!({"type":"settingsChanged", "key":field.key, "value":value.trim()}));
        }
        if node == field.clear {
            return Some(json!({"type":"settingsChanged", "key":field.key, "value":""}));
        }
    }
    if node == fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL {
        let url = settings.plugin_url.trim();
        return (!url.is_empty())
            .then(|| json!({"type":"pluginRepositoryAddRequested", "manifestUrl":url}));
    }
    let repositories = settings
        .plugins
        .get("repositories")
        .and_then(Value::as_array);
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + 4)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE) as usize;
        let url = repositories?.get(index)?.get("manifestUrl")?.as_str()?;
        return Some(json!({"type":"pluginRepositoryRemoveRequested", "manifestUrl":url}));
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE + 4)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE) as usize;
        let url = repositories?.get(index)?.get("manifestUrl")?.as_str()?;
        return Some(json!({"type":"pluginRepositoryAddRequested", "manifestUrl":url}));
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE + 4)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE) as usize;
        let scraper = settings.plugins.get("scrapers")?.as_array()?.get(index)?;
        let id = scraper.get("id")?.as_str()?;
        let enabled = !scraper
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        return Some(json!({"type":"pluginScraperToggled", "scraperId":id, "enabled":enabled}));
    }
    None
}

fn remember_actions(state: &mut RendererState, actions: Vec<UiAction>) {
    if actions.is_empty() {
        return;
    }
    for action in &actions {
        if let UiAction::TextInput { node, value } = action {
            if state.route == "library" && *node == fluxa_ui::NODE_LIBRARY_SEARCH {
                state.library_query.push_str(value);
                refresh_library_view(state);
            } else if state.route == "discover" && *node == fluxa_ui::NODE_DISCOVER_SEARCH {
                state.discover.query.push_str(value);
                request_discover_search(state);
                state.ui = UiTree::default();
            } else if state.route == "settings" && *node == fluxa_ui::NODE_SETTINGS_SEARCH {
                state.settings.search.push_str(value);
                state.ui = UiTree::default();
            } else if state.route == "settings" && *node == fluxa_ui::NODE_SETTINGS_ADDON_URL {
                state.settings.addon_url.push_str(value);
            } else if state.route == "settings" && *node == fluxa_ui::NODE_SETTINGS_PLUGIN_URL {
                state.settings.plugin_url.push_str(value);
            } else if state.route == "settings"
                && let Some(index) = fluxa_ui::poster_field(*node)
            {
                state.settings.poster_fields[index].push_str(value);
            }
            continue;
        }
        let node = match action {
            UiAction::Activated(node) | UiAction::PointerReleased(node) => Some(*node),
            _ => None,
        };
        if state.player.is_some() {
            if let Some(node) = node {
                player::activate(state, node);
            } else if matches!(action, UiAction::Back) {
                player::close(state);
            }
            continue;
        }
        if let Some(node) = node {
            if state.route == "library" && node == fluxa_ui::NODE_LIBRARY_SORT {
                state.library_sort = match state.library_sort.as_str() {
                    "recent" => "title",
                    "title" => "rating",
                    _ => "recent",
                }
                .to_owned();
                refresh_library_view(state);
                continue;
            }
            if state.route == "calendar" {
                if node == fluxa_ui::NODE_CALENDAR_CLOSE_DAY {
                    state.calendar.selected_day = None;
                    state.ui = UiTree::default();
                    continue;
                }
                if (fluxa_ui::NODE_CALENDAR_DAY_BASE..fluxa_ui::NODE_CALENDAR_DAY_BASE + 32)
                    .contains(&node)
                {
                    state.calendar.selected_day =
                        Some((node - fluxa_ui::NODE_CALENDAR_DAY_BASE) as u32);
                    state.ui = UiTree::default();
                    continue;
                }
            }
            if state.route == "settings"
                && (fluxa_ui::NODE_SETTINGS_SECTION_BASE
                    ..fluxa_ui::NODE_SETTINGS_SECTION_BASE
                        + fluxa_ui::SETTINGS_SECTIONS.len() as u64)
                    .contains(&node)
            {
                state.settings.active_section =
                    (node - fluxa_ui::NODE_SETTINGS_SECTION_BASE) as usize;
                state.settings.search.clear();
                state.ui = UiTree::default();
                continue;
            }
            if state.route == "settings"
                && let Some(action_json) = settings_action_json(node, &state.settings)
            {
                if node == fluxa_ui::NODE_SETTINGS_ADDON_INSTALL {
                    state.settings.addon_url.clear();
                } else if node == fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL {
                    state.settings.plugin_url.clear();
                }
                state
                    .pending_native_actions
                    .push(NativeAction::CoreCommand {
                        command: action_json,
                    });
                state.ui = UiTree::default();
                continue;
            }
            if state.route == "library"
                && (fluxa_ui::NODE_LIBRARY_TAB_BASE
                    ..fluxa_ui::NODE_LIBRARY_TAB_BASE + LibraryTab::ALL.len() as u64)
                    .contains(&node)
            {
                state.library_tab =
                    LibraryTab::ALL[(node - fluxa_ui::NODE_LIBRARY_TAB_BASE) as usize];
                refresh_library_view(state);
                state.ui = UiTree::default();
                continue;
            }
            if let Some(native_action) = native_action_for_node(
                node,
                &state.home,
                &state.library,
                state.library_tab,
                &state.discover,
                &state.calendar,
                &state.detail,
                &state.settings,
                &state.route,
                state
                    .gpu
                    .as_ref()
                    .and_then(|gpu| fluxa_ui::active_home_hero(&gpu.egui_context, &state.home)),
            ) {
                if let NativeAction::Detail { preview, .. } = &native_action {
                    state.backdrop_prefetch = ["background", "poster"]
                        .into_iter()
                        .find_map(|key| preview.get(key).and_then(Value::as_str))
                        .map(ToOwned::to_owned);
                }
                state.pending_native_actions.push(native_action);
            }
        }
        if matches!(action, UiAction::Back) {
            if state.route == "calendar" && state.calendar.selected_day.is_some() {
                state.calendar.selected_day = None;
                state.ui = UiTree::default();
            } else {
                state.pending_native_actions.push(NativeAction::Back);
            }
        }
    }
    state.last_actions.extend(actions);
    // Keep the bridge bounded while the real scene/action dispatcher is being
    // connected. The queue is diagnostic for now, not the source of truth.
    const MAX_RETAINED_ACTIONS: usize = 32;
    if state.last_actions.len() > MAX_RETAINED_ACTIONS {
        let drain_count = state.last_actions.len() - MAX_RETAINED_ACTIONS;
        state.last_actions.drain(..drain_count);
    }
    const MAX_PENDING_NATIVE_ACTIONS: usize = 16;
    if state.pending_native_actions.len() > MAX_PENDING_NATIVE_ACTIONS {
        let drain_count = state.pending_native_actions.len() - MAX_PENDING_NATIVE_ACTIONS;
        state.pending_native_actions.drain(..drain_count);
    }
}

fn take_native_actions(state: &mut RendererState) -> String {
    if state.pending_native_actions.is_empty() {
        return "[]".to_owned();
    }
    serde_json::to_string(&std::mem::take(&mut state.pending_native_actions))
        .unwrap_or_else(|_| "[]".to_owned())
}

fn advance_home_inertia(state: &mut RendererState) {
    let now = Instant::now();
    let elapsed = now
        .duration_since(state.scroll_animation_at)
        .as_secs_f32()
        .clamp(0.0, 0.05);
    state.scroll_animation_at = now;
    if state.touch_start.is_some() || elapsed == 0.0 || state.scroll_velocity.abs() < 14.0 {
        if state.scroll_velocity.abs() < 14.0 {
            state.scroll_velocity = 0.0;
            state.active_scroll = None;
        }
        return;
    }
    let viewport = Viewport::new(
        (state.size[0] as f32 / state.scale()).round().max(1.0) as u32,
        (state.size[1] as f32 / state.scale()).round().max(1.0) as u32,
        state.home.form_factor.into(),
    )
    .with_safe_bottom(state.safe_bottom);
    let movement = state.scroll_velocity * elapsed;
    match state.active_scroll {
        Some(HomeScrollTarget::Vertical) => {
            let max_offset = fluxa_ui::home_scroll_max(viewport, &state.home);
            let next = (state.home.scroll_offset + movement).clamp(0.0, max_offset);
            if (next - state.home.scroll_offset).abs() < 0.01 {
                state.scroll_velocity = 0.0;
            } else {
                state.home.scroll_offset = next;
            }
        }
        Some(HomeScrollTarget::Horizontal(row_index)) => {
            if state.home.row_scroll_offsets.len() <= row_index {
                state.home.row_scroll_offsets.resize(row_index + 1, 0.0);
            }
            let max_offset = fluxa_ui::home_row_scroll_max(viewport, &state.home, row_index);
            let offset = &mut state.home.row_scroll_offsets[row_index];
            let next = (*offset + movement).clamp(0.0, max_offset);
            if (next - *offset).abs() < 0.01 {
                state.scroll_velocity = 0.0;
            } else {
                *offset = next;
            }
            request_home_row_load_more(state, row_index, viewport);
        }
        Some(HomeScrollTarget::ScreenVertical) => {
            let max_offset = screen_scroll_max(state, viewport);
            let offset = state
                .screen_scroll_offsets
                .entry(state.route.clone())
                .or_default();
            let next = (*offset + movement).clamp(0.0, max_offset);
            if (next - *offset).abs() < 0.01 {
                state.scroll_velocity = 0.0;
            } else {
                *offset = next;
            }
        }
        None => state.scroll_velocity = 0.0,
    }
    // Exponential friction produces a predictable inertial tail independent
    // of the render loop's exact frame cadence.
    // Android's native scroller carries substantially more momentum than a
    // web-style smooth-scroll tail. Keep a long, quick decay so a firm flick
    // can traverse multiple viewports before coming to rest.
    state.scroll_velocity *= (-1.6 * elapsed).exp();
}

fn request_home_row_load_more(
    state: &mut RendererState,
    content_row_index: usize,
    viewport: Viewport,
) {
    if state.route != "home" {
        return;
    }
    let has_continue_row = !state.home.cards.is_empty();
    let Some(row_index) = content_row_index.checked_sub(usize::from(has_continue_row)) else {
        return;
    };
    let Some(row) = state.home.rows.get(row_index) else {
        return;
    };
    let Some(row_id) = row.id.as_deref().map(str::trim).filter(|id| !id.is_empty()) else {
        return;
    };
    let card_count = row.cards.len();
    if !row.can_load_more
        || card_count == 0
        || state.load_more_requested_counts.get(row_id) == Some(&card_count)
    {
        return;
    }
    let scroll_index = row_index + usize::from(has_continue_row);
    let max_offset = fluxa_ui::home_row_scroll_max(viewport, &state.home, scroll_index);
    let visible_width = (viewport.width * 0.9).max(1.0);
    let card_pitch = (max_offset + visible_width) / card_count as f32;
    let threshold = (visible_width * 2.0).max(card_pitch * 6.0).min(max_offset);
    let offset = state
        .home
        .row_scroll_offsets
        .get(scroll_index)
        .copied()
        .unwrap_or(0.0);
    if offset < (max_offset - threshold).max(0.0) {
        return;
    }
    state
        .load_more_requested_counts
        .insert(row_id.to_owned(), card_count);
    if let Some(category) = row.catalog_page.as_ref() {
        let content_type = category
            .get("contentType")
            .or_else(|| category.get("type"))
            .and_then(Value::as_str)
            .filter(|value| matches!(*value, "movie" | "series"))
            .unwrap_or("movie");
        let transport_url = category
            .get("addonTransportUrl")
            .or_else(|| category.get("transportUrl"))
            .cloned()
            .unwrap_or(Value::Null);
        let catalog_id = category
            .get("catalogId")
            .or_else(|| category.get("id"))
            .cloned()
            .unwrap_or(Value::Null);
        let skip =
            json!(category.get("skip").and_then(Value::as_i64).unwrap_or(0) + card_count as i64);
        let remote_sources = category
            .get("remoteSources")
            .or_else(|| category.get("remoteSource"))
            .cloned()
            .unwrap_or(Value::Null);
        let command = json!({
            "type": "catalogPageRequested",
            "categoryId": row_id,
            "transportUrl": transport_url,
            "contentType": content_type,
            "catalogId": catalog_id,
            "skip": skip,
            "genre": category.get("addonGenre").or_else(|| category.get("genre")),
            "remoteSource": remote_sources,
            "profile": Value::Null,
        });
        state
            .pending_native_actions
            .push(NativeAction::CoreCommand { command });
    } else {
        state.pending_native_actions.push(NativeAction::LoadMore {
            row_id: row_id.to_owned(),
        });
    }
}

fn screen_scroll_max(state: &RendererState, viewport: Viewport) -> f32 {
    match state.route.as_str() {
        "library" => fluxa_ui::library_scroll_max(viewport, &state.library, state.library_tab),
        "discover" => fluxa_ui::discover_scroll_max(viewport, &state.discover),
        "calendar" => fluxa_ui::calendar_scroll_max(viewport, &state.calendar),
        "detail" => fluxa_ui::detail_scroll_max(viewport, &state.detail),
        "settings" => fluxa_ui::settings_scroll_max(viewport, &state.settings),
        _ => 0.0,
    }
}

fn update_screen_scroll(state: &mut RendererState, delta: f32, viewport: Viewport) {
    let max_offset = screen_scroll_max(state, viewport);
    let offset = state
        .screen_scroll_offsets
        .entry(state.route.clone())
        .or_default();
    *offset = (*offset + delta).clamp(0.0, max_offset);
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
            active_scroll: None,
            scroll_velocity: 0.0,
            scroll_animation_at: Instant::now(),
            route: "home".to_owned(),
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
            video: None,
            fullscreen_toggle: false,
            profiles: None,
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
            if state.route == "home" {
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
            if state.route == "home" {
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

    pub fn focused_node(&self) -> Option<u64> {
        self.with_state(|state| state.ui.focused()).flatten()
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
        let profile = session.active_profile();
        session.dispatch(json!({
            "type": "homeLoadRequested",
            "profile": profile,
            "language": profile_language(&profile),
            "force": true,
        }))?;
        session.dispatch(discover_command(&profile, "movie", "", "", "", true))?;
        session.dispatch(json!({"type": "libraryHydrateRequested", "profileId": profile.get("id")}))?;
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

fn logical_viewport(state: &RendererState) -> Viewport {
    Viewport::new(
        (state.size[0] as f32 / state.scale()).round().max(1.0) as u32,
        (state.size[1] as f32 / state.scale()).round().max(1.0) as u32,
        state.home.form_factor.into(),
    )
    .with_safe_bottom(state.safe_bottom)
}

fn pointer_event(state: &mut RendererState, phase: PointerPhase, position: [f32; 2]) {
    if phase == PointerPhase::Down {
        state.keyboard_focus_visible = false;
        let now = Instant::now();
        state.touch_start = Some(position);
        state.touch_last = Some(position);
        state.touch_last_at = Some(now);
        state.touch_velocity_samples.clear();
        state.touch_velocity_samples.push_back((now, position));
        state.touch_scrolled = false;
        state.active_scroll = None;
        state.scroll_velocity = 0.0;
        state.scroll_animation_at = now;
    } else if phase == PointerPhase::Move {
        let now = Instant::now();
        if let (Some(start), Some(last)) = (state.touch_start, state.touch_last) {
            let total_x = start[0] - position[0];
            let total_y = start[1] - position[1];
            if total_x.hypot(total_y) > 8.0 {
                state.touch_scrolled = true;
            }
            if state.active_scroll.is_none() && state.touch_scrolled {
                let viewport = logical_viewport(state);
                if state.route == "home" {
                    if total_x.abs() > total_y.abs() * 1.15 {
                        state.active_scroll =
                            fluxa_ui::home_row_at_y(viewport, &state.home, start[1])
                                .map(HomeScrollTarget::Horizontal);
                    }
                    if state.active_scroll.is_none() {
                        state.active_scroll = Some(HomeScrollTarget::Vertical);
                    }
                } else {
                    state.active_scroll = Some(HomeScrollTarget::ScreenVertical);
                }
            }
            let sample_seconds = state
                .touch_last_at
                .map(|last_at| now.duration_since(last_at).as_secs_f32())
                .unwrap_or(1.0 / 60.0)
                .clamp(1.0 / 240.0, 0.08);
            let delta = match state.active_scroll {
                Some(HomeScrollTarget::Vertical) => last[1] - position[1],
                Some(HomeScrollTarget::Horizontal(_)) => last[0] - position[0],
                Some(HomeScrollTarget::ScreenVertical) => last[1] - position[1],
                None => 0.0,
            };
            if delta.abs() > 0.25 {
                let viewport = logical_viewport(state);
                match state.active_scroll {
                    Some(HomeScrollTarget::Vertical) => {
                        let max_offset = fluxa_ui::home_scroll_max(viewport, &state.home);
                        state.home.scroll_offset =
                            (state.home.scroll_offset + delta).clamp(0.0, max_offset);
                    }
                    Some(HomeScrollTarget::Horizontal(row_index)) => {
                        if state.home.row_scroll_offsets.len() <= row_index {
                            state.home.row_scroll_offsets.resize(row_index + 1, 0.0);
                        }
                        let max_offset =
                            fluxa_ui::home_row_scroll_max(viewport, &state.home, row_index);
                        let offset = &mut state.home.row_scroll_offsets[row_index];
                        *offset = (*offset + delta).clamp(0.0, max_offset);
                    }
                    Some(HomeScrollTarget::ScreenVertical) => {
                        update_screen_scroll(state, delta, viewport);
                    }
                    None => {}
                }
                if let Some(HomeScrollTarget::Horizontal(row_index)) = state.active_scroll {
                    request_home_row_load_more(state, row_index, viewport);
                }
                state.touch_velocity_samples.push_back((now, position));
                while state
                    .touch_velocity_samples
                    .front()
                    .is_some_and(|(sample_at, _)| {
                        now.duration_since(*sample_at).as_secs_f32() > 0.12
                    })
                    && state.touch_velocity_samples.len() > 2
                {
                    state.touch_velocity_samples.pop_front();
                }
                let velocity = state
                    .touch_velocity_samples
                    .front()
                    .zip(state.touch_velocity_samples.back())
                    .map(|((first_at, first), (last_at, last))| {
                        let seconds = last_at.duration_since(*first_at).as_secs_f32();
                        if seconds > 0.001 {
                            match state.active_scroll {
                                Some(HomeScrollTarget::Horizontal(_)) => {
                                    (first[0] - last[0]) / seconds
                                }
                                Some(
                                    HomeScrollTarget::Vertical | HomeScrollTarget::ScreenVertical,
                                ) => (first[1] - last[1]) / seconds,
                                None => 0.0,
                            }
                        } else {
                            delta / sample_seconds
                        }
                    })
                    .unwrap_or(delta / sample_seconds);
                // Keep the recent-window velocity (Compose VelocityTracker
                // style) so a firm fling remains materially faster than a
                // slow drag even when the last MotionEvent is noisy.
                // Recent-window pointer velocity underestimates the release
                // velocity on touchscreens because MOVE events are sampled
                // sparsely. Calibrate toward Android OverScroller's spline
                // distance while retaining a hard bound for noisy events.
                state.scroll_velocity = (velocity * 2.5).clamp(-14_000.0, 14_000.0);
                state.scroll_animation_at = now;
            }
        }
        state.touch_last = Some(position);
        state.touch_last_at = Some(now);
    }
    if phase == PointerPhase::Up {
        let was_scroll = state.touch_scrolled;
        state.touch_start = None;
        state.touch_last = None;
        state.touch_last_at = None;
        state.touch_scrolled = false;
        state.touch_velocity_samples.clear();
        if was_scroll {
            return;
        }
    }
    let event = match phase {
        PointerPhase::Move => UiEvent::PointerMove { position },
        PointerPhase::Down => UiEvent::PointerDown {
            position,
            button: PointerButton::Primary,
        },
        PointerPhase::Up => UiEvent::PointerUp {
            position,
            button: PointerButton::Primary,
        },
    };
    let actions = state.ui.dispatch(event);
    remember_actions(state, actions);
}

fn key_down(state: &mut RendererState, input: KeyInput) {
    if state.player.is_some()
        && matches!(player::key(state, input), player::KeyOutcome::Handled)
    {
        return;
    }
    state.keyboard_focus_visible = true;
    rebuild_current_ui(state);
    if matches!(input, KeyInput::Backspace) {
        match state.ui.focused() {
            Some(fluxa_ui::NODE_LIBRARY_SEARCH) if state.route == "library" => {
                state.library_query.pop();
                refresh_library_view(state);
                return;
            }
            Some(fluxa_ui::NODE_SETTINGS_SEARCH) if state.route == "settings" => {
                state.settings.search.pop();
                state.ui = UiTree::default();
                return;
            }
            Some(fluxa_ui::NODE_SETTINGS_ADDON_URL) if state.route == "settings" => {
                state.settings.addon_url.pop();
                return;
            }
            Some(fluxa_ui::NODE_SETTINGS_PLUGIN_URL) if state.route == "settings" => {
                state.settings.plugin_url.pop();
                return;
            }
            Some(node) if state.route == "settings" && fluxa_ui::poster_field(node).is_some() => {
                if let Some(index) = fluxa_ui::poster_field(node) {
                    state.settings.poster_fields[index].pop();
                }
                return;
            }
            _ => {}
        }
    }
    let actions = match input {
        KeyInput::Gamepad(button) => state.ui.dispatch(UiEvent::GamepadButton {
            button,
            pressed: true,
        }),
        KeyInput::Key(key) => state.ui.dispatch(UiEvent::KeyDown(key)),
        KeyInput::Backspace => return,
    };
    remember_actions(state, actions);
    ensure_focused_visible(state);
}

fn active_route(state: &RendererState) -> String {
    if state.profiles.is_some() {
        "profiles".to_owned()
    } else if state.player.is_some() {
        "player".to_owned()
    } else {
        state.route.clone()
    }
}

fn next_redraw(state: &mut RendererState) -> Option<Instant> {
    let now = Instant::now();
    let busy = state.gpu.is_none()
        || state.player.is_some()
        || state.touch_start.is_some()
        || state.scroll_velocity != 0.0
        || state.pack_job.is_some()
        || !state.pending_native_actions.is_empty()
        || state.pending_resize.is_some();
    if busy {
        return Some(now);
    }
    let revision = state.session.as_ref().map(SessionHandle::revision);
    if revision.is_some() && revision != state.session_revision {
        return Some(now);
    }
    if state.session.as_ref().is_some_and(SessionHandle::has_queued_dispatches) {
        return Some(now + Duration::from_millis(4));
    }
    let effects = state
        .session
        .as_ref()
        .is_some_and(|session| session.has_outstanding_effects())
        .then(|| now + Duration::from_millis(50));
    let artwork = state
        .gpu
        .as_ref()
        .is_some_and(|gpu| gpu.artwork.fetcher.has_pending())
        .then(|| now + Duration::from_millis(50));
    let animation = state
        .gpu
        .as_ref()
        .and_then(|gpu| gpu.artwork.next_animation_frame());
    let projecting = state
        .projector
        .pending()
        .then(|| now + Duration::from_millis(8));
    let trailer = trailer::next_redraw(state, now);
    let artwork = [artwork, projecting, animation, effects, trailer].into_iter().flatten().min();
    match (state.redraw_at, artwork) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

struct FrameTimer {
    started: Instant,
    last: Instant,
    phases: Vec<(&'static str, Duration)>,
}

impl FrameTimer {
    fn start() -> Self {
        let now = Instant::now();
        Self { started: now, last: now, phases: Vec::new() }
    }

    fn mark(&mut self, phase: &'static str) {
        let now = Instant::now();
        self.phases.push((phase, now - self.last));
        self.last = now;
    }

    fn report(self, label: &str) {
        let total = self.started.elapsed();
        if total < SLOW_FRAME {
            return;
        }
        let phases = self
            .phases
            .iter()
            .filter(|(_, spent)| *spent >= Duration::from_millis(2))
            .map(|(phase, spent)| format!("{phase}={}ms", spent.as_millis()))
            .collect::<Vec<_>>()
            .join(" ");
        host_log(format!("Slow {label}: {}ms {phases}", total.as_millis()));
    }
}

const SLOW_FRAME: Duration = Duration::from_millis(24);

fn render_frame(state: &mut RendererState) {
    let mut timer = FrameTimer::start();
    route_actions_to_session(state);
    timer.mark("actions");
    pull_session_snapshot(state);
    timer.mark("snapshot");
    profiles::poll(state);
    player::pump(state);
    player::upload_frame(state);
    timer.mark("player");
    if state.gpu.is_none() {
        let count = GPU_WAIT_LOGS.fetch_add(1, Ordering::Relaxed);
        if count % 120 == 0 {
            host_log(format!(
                "Waiting for GPU initialization ({}x{})",
                state.size[0], state.size[1]
            ));
        }
    }
    if let Some(size) = state.pending_resize.take()
        && let Some(gpu) = state.gpu.as_mut()
    {
        gpu.resize(size);
    }
    sync_home_from_core_snapshot(state);
    trailer::tick(state);
    poster_data::tick(state);
    timer.mark("sync");
    if state.route == "home"
        || matches!(state.active_scroll, Some(HomeScrollTarget::ScreenVertical))
    {
        advance_home_inertia(state);
    }
    rebuild_current_ui(state);
    timer.mark("rebuild");
    let route = active_route(state);
    let player_model = state.player.as_ref().map(|player| fluxa_ui::PlayerModel {
        upscaling: player::upscaling(&state.settings).to_owned(),
        ..player.model()
    });
    let library_tab = state.library_tab;
    let focused = state
        .keyboard_focus_visible
        .then(|| state.ui.focused())
        .flatten();
    let safe_bottom = state.safe_bottom;
    let logical_size = logical_surface_size(state);
    if route != "home" && route != "player" {
        let viewport = Viewport::new(
            logical_size[0],
            logical_size[1],
            state.home.form_factor.into(),
        )
        .with_safe_bottom(safe_bottom);
        let max_offset = screen_scroll_max(state, viewport);
        if let Some(offset) = state.screen_scroll_offsets.get_mut(&route) {
            *offset = offset.clamp(0.0, max_offset);
        }
    }
    let scroll_y = state
        .screen_scroll_offsets
        .get(&route)
        .copied()
        .unwrap_or(0.0);
    let scale = state.scale();
    if let (Some(url), Some(gpu)) = (state.backdrop_prefetch.take(), state.gpu.as_mut()) {
        let width = (gpu.config.width as f32 / scale).round().max(1.0);
        gpu.artwork.texture_for_priority(
            Some(&url),
            fluxa_ui::backdrop_target_size(width, scale),
            ArtworkFetchPriority::Hero,
        );
    }
    presence::update(current_presence(state));
    timer.mark("prepare");
    let render_result = {
        let RendererState {
            gpu,
            home,
            library,
            discover,
            calendar,
            detail,
            settings,
            profiles,
            picker_background,
            egui_events,
            modifiers,
            pre_present,
            ..
        } = state;
        let events = std::mem::take(egui_events);
        let modifiers = *modifiers;
        gpu.as_mut().map(|gpu| {
            gpu.density = scale;
            gpu.render(
                &route,
                home,
                library,
                library_tab,
                discover,
                calendar,
                detail,
                settings,
                profiles.as_mut(),
                picker_background.as_deref(),
                player_model.as_ref(),
                focused,
                safe_bottom,
                scroll_y,
                events,
                modifiers,
                pre_present.as_ref(),
                &mut timer,
            )
        })
    };
    if let Some(render_result) = render_result {
        match render_result {
            Ok(frame) => {
                state.cursor = frame.cursor;
                state.wants_keyboard = frame.wants_keyboard;
                state.redraw_at = Instant::now().checked_add(frame.repaint_delay);
                let mut layout = frame.layout;
                if let Some(request) = layout.profiles.take() {
                    profiles::handle(state, request);
                }
                if let Some(position) = layout.seek_to {
                    player::command(state, VideoCommand::SeekTo(position));
                }
                player::hover_seek(state, layout.seek_hover);
                apply_pointer_results(state, &route, &layout);
                if route == "discover" {
                    for request in &layout.load_more {
                        state
                            .pending_native_actions
                            .push(NativeAction::CoreCommand {
                                command: request.clone(),
                            });
                    }
                }
                if route == "discover"
                    && let Some((key, value)) = layout.filter_change.as_ref()
                {
                    match key.as_str() {
                        "discover:contentType" => {
                            state.discover.content_type = value.clone();
                            state
                                .pending_native_actions
                                .push(NativeAction::DiscoverType {
                                    content_type: value.clone(),
                                });
                        }
                        "discover:catalog" => {
                            state.discover.selected_catalog_key = value.clone();
                            let action = NativeAction::DiscoverFilters {
                                content_type: state.discover.content_type.clone(),
                                catalog_key: value.clone(),
                                extra_name: state.discover.selected_extra_name.clone(),
                                extra_value: state.discover.selected_extra_value.clone(),
                                query: state.discover.query.clone(),
                            };
                            state.pending_native_actions.push(action);
                        }
                        "discover:extra" => {
                            state.discover.selected_extra_value = value.clone();
                            let action = NativeAction::DiscoverFilters {
                                content_type: state.discover.content_type.clone(),
                                catalog_key: state.discover.selected_catalog_key.clone(),
                                extra_name: state.discover.selected_extra_name.clone(),
                                extra_value: value.clone(),
                                query: state.discover.query.clone(),
                            };
                            state.pending_native_actions.push(action);
                        }
                        _ => {}
                    }
                }
                rebuild_ui_from_layout(state, &layout, logical_size);
                state.rendered_layout = Some((route, layout));
            }
            Err(error) => host_log(format!("Frame skipped: {error}")),
        }
    }
    timer.mark("layout");
    timer.report(&format!("frame on {}", state.route));
}

fn apply_pointer_results(state: &mut RendererState, route: &str, layout: &HomeLayout) {
    if let (Some(node), Some(value)) = (layout.text_input_node, layout.text_input.as_ref()) {
        set_text_value(state, node, value);
    }
    if let Some((key, value)) = layout.filter_change.as_ref()
        && key == "librarySort"
        && route == "library"
    {
        state.library_sort = value.clone();
        refresh_library_view(state);
    }
    if let Some((key, value)) = layout.setting_change.as_ref() {
        state
            .pending_native_actions
            .push(NativeAction::SettingsChange {
                key: key.clone(),
                value: value.clone(),
            });
    }
    if let Some(node) = layout.activated {
        rebuild_current_ui(state);
        remember_actions(state, vec![UiAction::Activated(node)]);
    }
}

fn set_text_value(state: &mut RendererState, node: u64, value: &str) {
    match node {
        fluxa_ui::NODE_LIBRARY_SEARCH if state.library_query != value => {
            state.library_query = value.to_owned();
            refresh_library_view(state);
        }
        fluxa_ui::NODE_DISCOVER_SEARCH if state.discover.query != value => {
            state.discover.query = value.to_owned();
            request_discover_search(state);
        }
        fluxa_ui::NODE_SETTINGS_ADDON_URL => state.settings.addon_url = value.to_owned(),
        fluxa_ui::NODE_SETTINGS_SEARCH => state.settings.search = value.to_owned(),
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL => state.settings.plugin_url = value.to_owned(),
        node => {
            if let Some(index) = fluxa_ui::poster_field(node) {
                state.settings.poster_fields[index] = value.to_owned();
            }
        }
    }
}

fn request_discover_search(state: &mut RendererState) {
    let query = state.discover.query.trim();
    if query.chars().count() == 1 {
        return;
    }
    let command = json!({
        "type": "searchRequested",
        "query": query,
        "language": state.discover.language,
    });
    state
        .pending_native_actions
        .push(NativeAction::CoreCommand { command });
}

fn profile_language(profile: &Value) -> String {
    profile
        .get("language")
        .and_then(Value::as_str)
        .filter(|language| !language.is_empty())
        .unwrap_or("en")
        .to_owned()
}

fn pull_session_snapshot(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let revision = session.revision();
    if state.session_revision == Some(revision) {
        return;
    }
    state.session_revision = Some(revision);
    state.core_snapshot = Some(session.snapshot());
    state.core_snapshot_revision = state.core_snapshot_revision.wrapping_add(1);
    sync_home_from_core_snapshot(state);
}

fn discover_command(
    profile: &Value,
    content_type: &str,
    catalog_key: &str,
    extra_name: &str,
    extra_value: &str,
    load_catalog_filters: bool,
) -> Value {
    let mut extra = serde_json::Map::new();
    if !extra_name.is_empty() && !extra_value.is_empty() {
        extra.insert(extra_name.to_owned(), json!(extra_value));
    }
    json!({
        "type": "discoverRequested",
        "loadCatalogFilters": load_catalog_filters,
        "contentType": content_type,
        "filters": {
            "catalogKey": (!catalog_key.is_empty()).then_some(catalog_key),
            "extra": extra,
        },
        "profile": profile,
        "language": profile_language(profile),
    })
}

fn navigation(route: &str) -> Value {
    json!({"type": "navigationRequested", "route": route, "params": {}})
}

fn session_commands(action: &NativeAction, profile: &Value) -> Option<Vec<Value>> {
    let commands = match action {
        NativeAction::CoreCommand { command } => vec![command.clone()],
        NativeAction::LoadMore { .. } => Vec::new(),
        NativeAction::Back => vec![navigation("home")],
        NativeAction::Navigate { destination } => match destination.as_str() {
            "home" => vec![
                navigation("home"),
                json!({
                    "type": "refreshContinueWatchingRequested",
                    "profile": profile,
                    "language": profile_language(profile),
                    "source": "navigation",
                }),
            ],
            "library" => vec![
                navigation("library"),
                json!({"type": "libraryHydrateRequested", "profileId": profile.get("id")}),
            ],
            "discover" => vec![
                navigation("discover"),
                discover_command(profile, "movie", "", "", "", true),
            ],
            "calendar" => {
                let (year, month) = current_year_month();
                vec![
                    navigation("calendar"),
                    json!({"type": "calendarMonthRequested", "profile": profile, "year": year, "month": month, "plannedItems": []}),
                ]
            }
            "profile" | "settings" => vec![navigation("settings")],
            _ => return None,
        },
        NativeAction::DiscoverType { content_type } => {
            vec![discover_command(
                profile,
                content_type,
                "",
                "",
                "",
                true,
            )]
        }
        NativeAction::DiscoverCatalog {
            content_type,
            catalog_key,
            extra_name,
            extra_value,
            ..
        }
        | NativeAction::DiscoverFilters {
            content_type,
            catalog_key,
            extra_name,
            extra_value,
            ..
        } => vec![discover_command(
            profile,
            content_type,
            catalog_key,
            extra_name,
            extra_value,
            false,
        )],
        NativeAction::CalendarMonth { year, month } => vec![
            json!({"type": "calendarMonthRequested", "profile": profile, "year": year, "month": month, "plannedItems": []}),
        ],
        NativeAction::Detail { id, item_type, .. } | NativeAction::Play { id, item_type } => vec![
            navigation("detail"),
            json!({
                "type": "detailLoadRequested",
                "id": id,
                "contentType": item_type,
                "language": profile_language(profile),
                "profile": profile,
                "preview": match action {
                    NativeAction::Detail { preview, .. } => preview.clone(),
                    _ => Value::Null,
                },
            }),
        ],
        NativeAction::ToggleWatchlist { item } => {
            vec![json!({"type": "toggleWatchlistRequested", "item": item, "profile": profile})]
        }
        NativeAction::SettingsChange { key, value } => {
            vec![json!({"type": "settingsChanged", "key": key, "value": value})]
        }
        NativeAction::StartPlayback { item } => {
            vec![player::direct_playback_command(item, profile)]
        }
        NativeAction::SettingsSection { .. } => return None,
    };
    Some(commands)
}

fn route_actions_to_session(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    if state.pending_native_actions.is_empty() {
        return;
    }
    let profile = session.active_profile();
    let mut unhandled = Vec::new();
    let mut open_profiles = false;
    for action in std::mem::take(&mut state.pending_native_actions) {
        if matches!(&action, NativeAction::Navigate { destination } if destination == "profiles") {
            open_profiles = true;
            continue;
        }
        if let NativeAction::StartPlayback { item } = &action {
            state.player = Some(player::PlayerSession::new(item.clone()));
            state.ui = UiTree::default();
        }
        if matches!(&action, NativeAction::Navigate { destination } if destination == "discover")
            && !state.discover.catalogs.is_empty()
        {
            if let Err(error) = session.dispatch(navigation("discover")) {
                host_log(format!("core dispatch failed: {error}"));
            }
            continue;
        }
        match session_commands(&action, &profile) {
            Some(commands) => {
                for command in commands {
                    if let Err(error) = session.dispatch(command) {
                        host_log(format!("core dispatch failed: {error}"));
                    }
                }
            }
            None => unhandled.push(action),
        }
    }
    state.pending_native_actions = unhandled;
    if open_profiles {
        state.ui = UiTree::default();
        profiles::open(state);
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
                destination: "settings".to_owned(),
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
    fn back_closes_player_instead_of_navigating() {
        let host = FluxaHost::new(1.0, None);
        host.with_state(|state| state.player = Some(player::PlayerSession::new(json!({}))));
        host.key_down(KeyInput::Key(Key::Back));
        assert!(!host.is_playing());
        host.with_state(|state| assert!(state.pending_native_actions.is_empty()));
    }
}
