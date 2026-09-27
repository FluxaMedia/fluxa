use egui::{Align2, Color32, FontId, Id, Pos2, Rect, RichText, Sense, TextureId, Vec2};
use fluxa_theme::theme::{active_shared_ui_tokens, active_theme};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::OnceLock,
    time::Duration,
};

mod calendar;
mod components;
mod detail;
mod discover;
mod library;
mod player;
mod poster_overlay;
mod profiles;
mod settings;

pub use calendar::draw_calendar;
pub use poster_overlay::{
    Enrichment, PersonalIndex, PosterOverlays, set_poster_enrichment, set_poster_landscape,
    set_poster_overlays, set_poster_personal, set_rating_logos,
};

pub use detail::{detail_scroll_max, draw_detail};
pub use discover::draw_discover;
pub use library::draw_library;
pub use profiles::{
    PinPurpose, PinPrompt, ProfileAvatar, ProfileAvatarPack, ProfileDraft, ProfileEntry,
    ProfilePickerSettings, ProfilesMode, ProfilesModel, ProfilesRequest, draw_profiles,
};
pub use player::{
    PlayerModel, content_warning_duration, draw_player, format_time, torrent_status_lines,
};
use settings::settings_card_height;
pub use settings::{
    ACCOUNT_PROVIDERS, AccountPrompt, POSTER_FIELDS, SETTINGS_SECTIONS, SettingsModel, SettingsRow, SettingsSection, draw_settings,
    poster_field, settings_model_from_core_snapshot, settings_row_by_index,
};

pub fn localized(key: &str, language: &str) -> String {
    localized_or(key, key, language)
}

pub fn localized_or(key: &str, fallback: &str, language: &str) -> String {
    static ENGLISH: OnceLock<serde_json::Value> = OnceLock::new();
    static TURKISH: OnceLock<serde_json::Value> = OnceLock::new();
    let english = ENGLISH.get_or_init(|| {
        serde_json::from_str(include_str!("../../../shared/i18n/english_us.json"))
            .expect("English translations")
    });
    let selected = if language.starts_with("tr") {
        TURKISH.get_or_init(|| {
            serde_json::from_str(include_str!("../../../shared/i18n/tr_tr.json"))
                .expect("Turkish translations")
        })
    } else {
        english
    };
    selected
        .get(key)
        .or_else(|| english.get(key))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(fallback)
        .to_owned()
}

fn snapshot_language(snapshot: &serde_json::Value) -> String {
    snapshot
        .pointer("/settings/values/language")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("en")
        .to_owned()
}

fn setting_default(key: &str) -> Option<&'static serde_json::Value> {
    static DEFAULTS: OnceLock<HashMap<String, serde_json::Value>> = OnceLock::new();
    DEFAULTS
        .get_or_init(|| {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../core/fluxa-core/settings/settings-manifest.json"
            ))
            .expect("Core settings manifest");
            manifest
                .get("settings")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    Some((
                        entry.get("key")?.as_str()?.to_owned(),
                        entry.get("default")?.clone(),
                    ))
                })
                .collect()
        })
        .get(key)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiFormFactor {
    Mobile,
    #[default]
    Desktop,
    Tv,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeClass {
    Compact,
    Medium,
    Expanded,
}

#[derive(Clone, Copy, Debug)]
pub struct AnimatedTexture {
    pub texture: TextureId,
    /// Normalized atlas coordinates for the currently displayed frame.
    pub uv: Rect,
    pub image_size: [u32; 2],
}

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub form_factor: UiFormFactor,
    /// Bottom safe-drawing inset reported by the platform host, in logical dp.
    pub safe_bottom: f32,
    /// Vertical content offset supplied by the platform input adapter.
    /// Headers and navigation remain fixed while the screen body scrolls.
    pub scroll_y: f32,
}

impl Viewport {
    pub fn new(width: u32, height: u32, form_factor: UiFormFactor) -> Self {
        Self {
            width: width as f32,
            height: height as f32,
            form_factor,
            safe_bottom: 0.0,
            scroll_y: 0.0,
        }
    }

    pub fn with_safe_bottom(mut self, inset: f32) -> Self {
        self.safe_bottom = if inset.is_finite() {
            inset.max(0.0)
        } else {
            0.0
        };
        self
    }

    pub fn with_scroll_y(mut self, offset: f32) -> Self {
        self.scroll_y = if offset.is_finite() {
            offset.max(0.0)
        } else {
            0.0
        };
        self
    }

    pub fn size_class(self) -> SizeClass {
        if self.width < 600.0 {
            SizeClass::Compact
        } else if self.width < 840.0 {
            SizeClass::Medium
        } else {
            SizeClass::Expanded
        }
    }

    pub fn is_compact(self) -> bool {
        self.size_class() == SizeClass::Compact
    }

    pub fn is_tv(self) -> bool {
        self.form_factor == UiFormFactor::Tv
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HomeModel {
    #[serde(rename = "isLoading")]
    pub is_loading: bool,
    pub title: String,
    pub eyebrow: String,
    pub description: String,
    #[serde(rename = "backgroundUrl")]
    pub background_url: Option<String>,
    #[serde(rename = "logoUrl")]
    pub logo_url: Option<String>,
    #[serde(rename = "profileAvatarUrl")]
    pub profile_avatar_url: Option<String>,
    #[serde(rename = "profileName")]
    pub profile_name: Option<String>,
    #[serde(rename = "itemId")]
    pub item_id: Option<String>,
    #[serde(rename = "itemType")]
    pub item_type: Option<String>,
    #[serde(rename = "heroSlides")]
    pub hero_slides: Vec<HomeHero>,
    #[serde(rename = "showHeroSection")]
    pub show_hero_section: bool,
    #[serde(rename = "gifAutoplayEnabled")]
    pub gif_autoplay_enabled: bool,
    #[serde(skip)]
    pub accent: Option<Color32>,
    pub cards: Vec<HomeCard>,
    pub rows: Vec<HomeRow>,
    #[serde(rename = "formFactor")]
    pub form_factor: UiFormFactorJson,
    #[serde(skip)]
    pub scroll_offset: f32,
    #[serde(skip)]
    pub row_scroll_offsets: Vec<f32>,
    #[serde(skip)]
    pub trailer: Option<HeroTrailer>,
    #[serde(skip)]
    pub language: String,
    #[serde(skip)]
    artwork_signature: OnceLock<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct HomeHero {
    pub title: String,
    pub eyebrow: String,
    pub description: String,
    #[serde(rename = "backgroundUrl")]
    pub background_url: Option<String>,
    #[serde(rename = "logoUrl")]
    pub logo_url: Option<String>,
    #[serde(rename = "itemId")]
    pub item_id: Option<String>,
    #[serde(rename = "itemType")]
    pub item_type: Option<String>,
    pub trailers: Vec<String>,
    #[serde(skip)]
    pub raw: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct HeroTrailer {
    pub item_id: String,
    pub texture: Option<egui::TextureId>,
}

impl Default for HomeModel {
    fn default() -> Self {
        Self {
            is_loading: false,
            title: "Recommended".to_owned(),
            eyebrow: "Featured title".to_owned(),
            description: "Your connected catalogs will appear here.".to_owned(),
            cards: vec![
                HomeCard {
                    title: "Bleach".to_owned(),
                    subtitle: "S1:E1".to_owned(),
                    progress: 0.34,
                    ..Default::default()
                },
                HomeCard {
                    title: "Deadpool".to_owned(),
                    subtitle: "22 min left".to_owned(),
                    progress: 0.43,
                    ..Default::default()
                },
            ],
            ..Self::default_empty()
        }
    }
}

impl HomeModel {
    fn default_empty() -> Self {
        Self {
            is_loading: false,
            title: String::new(),
            eyebrow: String::new(),
            description: String::new(),
            background_url: None,
            logo_url: None,
            profile_avatar_url: None,
            profile_name: None,
            item_id: None,
            item_type: None,
            hero_slides: Vec::new(),
            show_hero_section: true,
            gif_autoplay_enabled: true,
            accent: None,
            cards: Vec::new(),
            rows: Vec::new(),
            form_factor: UiFormFactorJson::Desktop,
            scroll_offset: 0.0,
            row_scroll_offsets: Vec::new(),
            trailer: None,
            language: "en".to_owned(),
            artwork_signature: OnceLock::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LibraryTab {
    #[default]
    Watchlist,
    Watching,
    Completed,
    Dropped,
    Liked,
    Airing,
    Rated,
    History,
}

impl LibraryTab {
    pub const ALL: [Self; 8] = [
        Self::Watchlist,
        Self::Watching,
        Self::Completed,
        Self::Dropped,
        Self::Liked,
        Self::Airing,
        Self::Rated,
        Self::History,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Watchlist => "Watchlist",
            Self::Watching => "Watching",
            Self::Completed => "Completed",
            Self::Dropped => "Dropped",
            Self::Liked => "Favorites",
            Self::Airing => "Airing",
            Self::Rated => "Rated",
            Self::History => "History",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::Watchlist => "library.watchlist",
            Self::Watching => "library.watching",
            Self::Completed => "library.completed",
            Self::Dropped => "library.dropped",
            Self::Liked => "library.favorites",
            Self::Airing => "library.smart_airing",
            Self::Rated => "library.smart_rated",
            Self::History => "library.history",
        }
    }

    pub fn core_tab_key(self) -> &'static str {
        match self {
            Self::Watchlist => "watchlist",
            Self::Watching => "watching",
            Self::Completed => "completed",
            Self::Dropped => "dropped",
            Self::Liked => "favorites",
            Self::Airing => "airing",
            Self::Rated => "rated",
            Self::History => "history",
        }
    }

    fn state_key(self) -> &'static str {
        match self {
            Self::Watchlist => "watchlist",
            Self::Watching => "continueWatching",
            Self::Completed => "completed",
            Self::Dropped => "dropped",
            Self::Liked => "liked",
            Self::Airing | Self::Rated | Self::History => "",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LibraryModel {
    pub language: String,
    pub query: String,
    pub sort_by: String,
    pub source: String,
    pub source_options: Vec<(String, String)>,
    pub view_tab: Option<LibraryTab>,
    pub view_cards: Vec<HomeCard>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub watchlist: Vec<HomeCard>,
    pub watching: Vec<HomeCard>,
    pub completed: Vec<HomeCard>,
    pub dropped: Vec<HomeCard>,
    pub liked: Vec<HomeCard>,
    pub personal: std::sync::Arc<PersonalIndex>,
    pub airing: Vec<HomeCard>,
    pub rated: Vec<HomeCard>,
    pub history: Vec<HomeCard>,
}

impl LibraryModel {
    pub fn cards(&self, tab: LibraryTab) -> &[HomeCard] {
        if self.view_tab == Some(tab) {
            return &self.view_cards;
        }
        match tab {
            LibraryTab::Watchlist => &self.watchlist,
            LibraryTab::Watching => &self.watching,
            LibraryTab::Completed => &self.completed,
            LibraryTab::Dropped => &self.dropped,
            LibraryTab::Liked => &self.liked,
            LibraryTab::Airing => &self.airing,
            LibraryTab::Rated => &self.rated,
            LibraryTab::History => &self.history,
        }
    }

    pub fn apply_core_plan(&mut self, plan: &serde_json::Value, active_tab: LibraryTab) {
        let smart = plan.get("smartLists").unwrap_or(&serde_json::Value::Null);
        let cards = |key| {
            smart
                .get(key)
                .and_then(serde_json::Value::as_array)
                .map(|items| items.iter().take(64).map(core_home_card).collect())
                .unwrap_or_default()
        };
        self.airing = cards("airing");
        self.rated = cards("rated");
        self.history = cards("history");
        self.view_cards = plan
            .get("items")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().take(64).map(core_home_card).collect())
            .unwrap_or_default();
        self.view_tab = Some(active_tab);
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiscoverCatalog {
    pub key: String,
    pub label: String,
    pub content_type: String,
    pub extras: Vec<(String, Vec<String>)>,
    pub required_extras: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct DiscoverModel {
    pub language: String,
    pub query: String,
    pub content_type: String,
    pub content_types: Vec<String>,
    pub selected_catalog_key: String,
    pub selected_extra_name: String,
    pub selected_extra_value: String,
    pub catalogs: Vec<DiscoverCatalog>,
    pub results: Vec<HomeCard>,
    pub generation: u64,
    pub next_page: Option<serde_json::Value>,
    pub is_loading: bool,
    pub catalogs_loading: bool,
    pub error: Option<String>,
    pub sections: Vec<DiscoverSection>,
}

#[derive(Clone, Debug, Default)]
pub struct DiscoverSection {
    pub title: String,
    pub start: usize,
    pub len: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CalendarEntry {
    pub date: String,
    pub show: String,
    pub episode: String,
    pub still_url: Option<String>,
    pub card: HomeCard,
}

#[derive(Clone, Debug, Default)]
pub struct CalendarModel {
    pub language: String,
    pub year: i32,
    pub month: i32,
    pub is_loading: bool,
    pub error: Option<String>,
    pub entries: Vec<CalendarEntry>,
    pub selected_day: Option<u32>,
    pub today: Option<String>,
}

impl CalendarModel {
    pub fn entries_for_day(&self, day: u32) -> impl Iterator<Item = &CalendarEntry> {
        let date = format!("{:04}-{:02}-{:02}", self.year, self.month, day);
        self.entries.iter().filter(move |entry| entry.date == date)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UiFormFactorJson {
    Mobile,
    #[default]
    Desktop,
    Tv,
}

impl From<UiFormFactorJson> for UiFormFactor {
    fn from(value: UiFormFactorJson) -> Self {
        match value {
            UiFormFactorJson::Mobile => Self::Mobile,
            UiFormFactorJson::Desktop => Self::Desktop,
            UiFormFactorJson::Tv => Self::Tv,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct HomeRow {
    pub id: Option<String>,
    pub title: String,
    pub cards: Vec<HomeCard>,
    pub kind: HomeRowKind,
    #[serde(rename = "canLoadMore")]
    pub can_load_more: bool,
    /// Source catalog metadata needed to request the next page without
    /// depending on a platform-specific Home implementation.
    #[serde(skip)]
    pub catalog_page: Option<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HomeRowKind {
    Continue,
    #[default]
    Poster,
    Collection,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct HomeCard {
    pub id: Option<String>,
    #[serde(rename = "itemType")]
    pub item_type: Option<String>,
    pub title: String,
    pub subtitle: String,
    pub progress: f32,
    #[serde(rename = "artworkUrl")]
    pub artwork_url: Option<String>,
    /// Collection-folder presentation metadata from Core. Web renders these
    /// shelves with mixed tile sizes rather than forcing every tile into a
    /// poster slot.
    #[serde(skip)]
    pub collection_shape: Option<String>,
    #[serde(skip)]
    pub hide_title: bool,
    #[serde(skip)]
    pub motion_url: Option<String>,
    #[serde(skip)]
    pub motion_enabled: bool,
    /// Original Core item. Continue Watching cards need the resume fields
    /// (`lastVideoId`, episode numbers, offsets, etc.) when they are
    /// activated; reducing them to display-only fields loses the playback
    /// identity and incorrectly sends the user to an empty detail screen.
    #[serde(skip)]
    pub raw: serde_json::Value,
    #[serde(skip)]
    pub row_kind: HomeRowKind,
    #[serde(skip)]
    pub overlay: poster_overlay::PosterFacts,
    #[serde(skip)]
    pub logo_url: Option<String>,
    #[serde(skip)]
    pub backdrop_url: Option<String>,
}

impl HomeCard {
    fn is_landscape(&self) -> bool {
        poster_overlay::landscape() && self.row_kind != HomeRowKind::Collection
    }

    fn poster_art(&self) -> Option<&str> {
        match self.is_landscape() {
            true => self.backdrop_url.as_deref().or(self.artwork_url.as_deref()),
            false => self.artwork_url.as_deref(),
        }
    }
}

impl HomeModel {
    pub fn content_rows(&self) -> Vec<(&str, &[HomeCard])> {
        self.content_rows_with_kind()
            .into_iter()
            .map(|(title, cards, _)| (title, cards))
            .collect()
    }

    pub fn content_rows_with_kind(
        &self,
    ) -> impl Iterator<Item = (&str, &[HomeCard], HomeRowKind)> + '_ {
        let continue_row = std::iter::once((
            "Continue Watching",
            self.cards.as_slice(),
            HomeRowKind::Continue,
        ))
        .filter(|(_, cards, _)| !cards.is_empty());
        let rows = self
            .rows
            .iter()
            .map(|row| (row.title.as_str(), row.cards.as_slice(), row.kind));
        let fallback = std::iter::once((
            "Continue Watching",
            self.cards.as_slice(),
            HomeRowKind::Continue,
        ))
        .filter(|_| self.cards.is_empty() && self.rows.is_empty());
        continue_row.chain(rows).chain(fallback)
    }

    pub fn card_at(&self, index: usize) -> Option<&HomeCard> {
        if let Some(card) = self.cards.get(index) {
            return Some(card);
        }
        let row_index = index.saturating_sub(self.cards.len());
        self.rows
            .iter()
            .flat_map(|row| row.cards.iter())
            .nth(row_index)
    }
}

/// Projects the host-independent Core snapshot into the renderer's shared
/// Home model. Keeping this conversion here means Android, desktop and the
/// future WASM host all paint the same data shape without a Kotlin/TypeScript
/// mapper in front of the renderer.
pub fn home_model_from_core_snapshot(
    snapshot: &serde_json::Value,
    form_factor: UiFormFactorJson,
) -> HomeModel {
    let home = snapshot.get("home").unwrap_or(&serde_json::Value::Null);
    let billboard = home
        .get("billboard")
        .filter(|value| !value.is_null())
        .or_else(|| snapshot.pointer("/billboard/movie"))
        .unwrap_or(&serde_json::Value::Null);
    let mut model = HomeModel {
        is_loading: home
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        title: first_value_string(billboard, &["name", "title"])
            .unwrap_or_else(|| "Recommended".to_owned()),
        eyebrow: hero_meta_line(billboard, &snapshot_language(snapshot)),
        description: first_value_string(billboard, &["description", "overview"])
            .unwrap_or_else(|| "Your connected catalogs will appear here.".to_owned()),
        background_url: first_value_string(
            billboard,
            &[
                "background",
                "backgroundUrl",
                "backdrop",
                "backdropUrl",
                "poster",
                "posterUrl",
            ],
        ),
        logo_url: first_value_string(billboard, &["logo", "logoUrl", "clearLogo"]),
        profile_avatar_url: first_value_string(
            home.get("activeProfile")
                .unwrap_or(&serde_json::Value::Null),
            &["avatarUrl", "avatar"],
        )
        .or_else(|| {
            first_value_string(
                snapshot
                    .pointer("/profile/active")
                    .unwrap_or(&serde_json::Value::Null),
                &["avatarUrl", "avatar"],
            )
        }),
        profile_name: first_value_string(
            home.get("activeProfile")
                .unwrap_or(&serde_json::Value::Null),
            &["displayName", "name"],
        )
        .or_else(|| {
            first_value_string(
                snapshot
                    .pointer("/profile/active")
                    .unwrap_or(&serde_json::Value::Null),
                &["displayName", "name"],
            )
        }),
        item_id: value_string(billboard, "id"),
        item_type: value_string(billboard, "type"),
        show_hero_section: home
            .get("activeProfile")
            .and_then(|profile| profile.get("showHeroSection"))
            .and_then(serde_json::Value::as_bool)
            .or_else(|| {
                snapshot
                    .pointer("/profile/active/showHeroSection")
                    .and_then(serde_json::Value::as_bool)
            })
            .unwrap_or(true),
        gif_autoplay_enabled: snapshot
            .pointer("/settings/values/gifAutoplayEnabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        accent: snapshot
            .pointer("/settings/values/accentColorArgb")
            .and_then(accent_from_value),
        form_factor,
        language: snapshot_language(snapshot),
        ..HomeModel::default_empty()
    };
    let mut slide_keys = Vec::new();
    let mut hero_slides = Vec::new();
    for item in std::iter::once(billboard).chain(
        home.get("heroSlides")
            .or_else(|| home.get("slides"))
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten(),
    ) {
        let hero = core_home_hero(item, &snapshot_language(snapshot));
        let key = hero
            .item_id
            .clone()
            .or_else(|| hero.background_url.clone())
            .unwrap_or_else(|| hero.title.clone());
        if !key.is_empty() && !slide_keys.contains(&key) {
            slide_keys.push(key);
            hero_slides.push(hero);
        }
    }
    model.hero_slides = hero_slides;
    if let Some(items) = [
        home.get("continueWatching"),
        home.get("currentWatchlist"),
        home.get("externalContinueWatching"),
    ]
    .into_iter()
    .flatten()
    .find_map(serde_json::Value::as_array)
    {
        model.cards = items
            .iter()
            .map(|item| core_home_card_for_kind(item, HomeRowKind::Continue))
            .collect();
    }
    if let Some(categories) = home
        .get("categories")
        .or_else(|| home.get("rows"))
        .and_then(serde_json::Value::as_array)
    {
        // `continueWatching` is rendered from its dedicated list above. Some
        // Core snapshots also include that same list as a category; keeping
        // both produces a duplicate shelf immediately before the catalogs.
        if model.cards.is_empty()
            && let Some(category) = categories.iter().find(|category| {
                value_string(category, "id").as_deref() == Some("continue_watching")
                    || category.get("type").and_then(serde_json::Value::as_str)
                        == Some("continue_watching")
            })
            && let Some(items) = category.get("items").and_then(serde_json::Value::as_array)
        {
            model.cards = items
                .iter()
                .map(|item| core_home_card_for_kind(item, HomeRowKind::Continue))
                .collect();
        }
        model.rows = categories
            .iter()
            .enumerate()
            .filter_map(|(index, category)| {
                if value_string(category, "id").as_deref() == Some("continue_watching")
                    || category.get("type").and_then(serde_json::Value::as_str)
                        == Some("continue_watching")
                {
                    return None;
                }
                let cards = category
                    .get("items")
                    .and_then(serde_json::Value::as_array)?;
                if cards.is_empty() {
                    return None;
                }
                let kind = match (
                    value_string(category, "id").as_deref(),
                    category.get("type").and_then(serde_json::Value::as_str),
                ) {
                    (Some("continue_watching" | "upcoming"), _)
                    | (_, Some("continue_watching" | "upcoming")) => HomeRowKind::Continue,
                    (_, Some("collection" | "collection_folder")) => HomeRowKind::Collection,
                    _ => HomeRowKind::Poster,
                };
                Some(HomeRow {
                    id: value_string(category, "id"),
                    catalog_page: Some(category.clone()),
                    title: first_value_string(
                        category,
                        &["homeTitle", "name", "label", "title", "id"],
                    )
                    .filter(|title| !title.trim().is_empty())
                    .map(|title| normalize_home_row_title(&title))
                    .unwrap_or_else(|| format!("Catalog {}", index + 1)),
                    cards: cards
                        .iter()
                        .map(|item| core_home_card_for_kind(item, kind))
                        .collect(),
                    kind,
                    can_load_more: category
                        .get("canLoadMore")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false),
                })
            })
            .collect();
    }
    model
}

fn core_home_hero(item: &serde_json::Value, language: &str) -> HomeHero {
    HomeHero {
        title: first_value_string(item, &["name", "title"]).unwrap_or_default(),
        eyebrow: hero_meta_line(item, language),
        description: first_value_string(item, &["description", "overview"]).unwrap_or_default(),
        background_url: first_value_string(
            item,
            &[
                "background",
                "backgroundUrl",
                "backdrop",
                "backdropUrl",
                "poster",
                "posterUrl",
            ],
        ),
        logo_url: first_value_string(item, &["logo", "logoUrl", "clearLogo"]),
        item_id: value_string(item, "id"),
        item_type: value_string(item, "type"),
        trailers: trailer_urls(item),
        raw: item.clone(),
    }
}

fn trailer_urls(item: &serde_json::Value) -> Vec<String> {
    item.get("trailers")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| first_value_string(item, &["url", "link"]))
                .take(6)
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Clone, Copy, Debug, Default)]
struct HeroCarouselState {
    index: usize,
    generation: u64,
    next_at: f64,
}

fn home_hero_index(context: &egui::Context, home: &HomeModel) -> usize {
    let count = home.hero_slides.len();
    if count <= 1 {
        return 0;
    }
    let signature = home
        .hero_slides
        .iter()
        .map(|slide| {
            slide
                .item_id
                .as_deref()
                .or(slide.background_url.as_deref())
                .unwrap_or(slide.title.as_str())
        })
        .collect::<Vec<_>>();
    let id = Id::new(("fluxa-home-hero-carousel", signature));
    let now = context.input(|input| input.time);
    let mut state = context
        .data_mut(|data| data.get_temp::<HeroCarouselState>(id))
        .unwrap_or_default();
    let holding = home.trailer.as_ref().is_some_and(|trailer| {
        home.hero_slides.get(state.index % count).and_then(|slide| slide.item_id.as_deref())
            == Some(trailer.item_id.as_str())
    });
    if state.next_at <= 0.0 || holding {
        state.next_at = now + 6.5;
    }
    if now >= state.next_at {
        state.index = (state.index + 1) % count;
        state.generation = state.generation.wrapping_add(1);
        state.next_at = now + 6.5;
    }
    context.request_repaint_after(Duration::from_secs_f64((state.next_at - now).max(0.0)));
    state.index %= count;
    context.data_mut(|data| data.insert_temp(id, state));
    state.index
}

/// Returns the hero item currently displayed by the shared Home carousel.
/// Hosts use this to dispatch View Details for the selected slide instead of
/// accidentally dispatching the first Core billboard forever.
pub fn active_home_hero<'a>(context: &egui::Context, home: &'a HomeModel) -> Option<&'a HomeHero> {
    home.hero_slides.get(home_hero_index(context, home))
}

fn normalize_home_row_title(title: &str) -> String {
    let parts = title
        .split(" - ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() >= 2 {
        parts[1..].join(" ")
    } else {
        title.trim().to_owned()
    }
}

/// Projects the Core library state into the same card model used by Home.
/// The renderer owns the tab/layout decision; Core only owns the data and
/// loading/error state.
pub fn library_model_from_core_snapshot(snapshot: &serde_json::Value) -> LibraryModel {
    let library = snapshot.get("library").unwrap_or(&serde_json::Value::Null);
    let language = snapshot_language(snapshot);
    let cards = |tab: LibraryTab| {
        library
            .get(tab.state_key())
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().take(64).map(core_home_card).collect())
            .unwrap_or_default()
    };
    LibraryModel {
        personal: std::sync::Arc::new(poster_overlay::personal_index(library)),
        language: language.clone(),
        query: String::new(),
        sort_by: "recent".to_owned(),
        source: snapshot
            .pointer("/settings/values/integrationLibrarySource")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("local")
            .to_owned(),
        source_options: {
            let profile = snapshot
                .pointer("/profile/active")
                .or_else(|| snapshot.pointer("/home/activeProfile"))
                .unwrap_or(&serde_json::Value::Null);
            let mut options = vec![(
                "local".to_owned(),
                localized("library.source_local", &language),
            )];
            for (source, token_key) in [
                ("nuvio", "nuvioAccessToken"),
                ("trakt", "traktAccessToken"),
                ("simkl", "simklAccessToken"),
                ("anilist", "anilistAccessToken"),
                ("stremio", "stremioAuthKey"),
            ] {
                if profile
                    .get(token_key)
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|token| !token.is_empty())
                {
                    options.push((
                        source.to_owned(),
                        localized(&format!("library.source_{source}"), &language),
                    ));
                }
            }
            options
        },
        view_tab: None,
        view_cards: Vec::new(),
        is_loading: library
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        error: library
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        watchlist: cards(LibraryTab::Watchlist),
        watching: cards(LibraryTab::Watching),
        completed: cards(LibraryTab::Completed),
        dropped: cards(LibraryTab::Dropped),
        liked: cards(LibraryTab::Liked),
        airing: Vec::new(),
        rated: Vec::new(),
        history: Vec::new(),
    }
}

pub fn discover_model_from_core_snapshot(snapshot: &serde_json::Value) -> DiscoverModel {
    let mut model = discover_catalog_model(snapshot);
    let search = snapshot.get("search").unwrap_or(&serde_json::Value::Null);
    let query = value_string(search, "query").unwrap_or_default();
    if !query.trim().is_empty() {
        model.results.clear();
        model.sections.clear();
        for category in search
            .get("categories")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            let items = category
                .get("items")
                .and_then(serde_json::Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            if items.is_empty() {
                continue;
            }
            let name = value_string(category, "name").unwrap_or_default();
            let addon = value_string(category, "addonName").unwrap_or_default();
            let title = if addon.is_empty() || name.contains(&addon) {
                name
            } else if name.is_empty() {
                addon
            } else {
                format!("{name} · {addon}")
            };
            model.sections.push(DiscoverSection {
                title,
                start: model.results.len(),
                len: items.len(),
            });
            model.results.extend(items.iter().map(core_home_card));
        }
        model.is_loading = search
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        model.error = search
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        model.next_page = None;
        model.generation = search
            .get("generation")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            | 1 << 63;
        model.query = query;
    }
    model
}

fn discover_catalog_model(snapshot: &serde_json::Value) -> DiscoverModel {
    let discover = snapshot.get("discover").unwrap_or(&serde_json::Value::Null);
    let filters = discover.get("filters").unwrap_or(&serde_json::Value::Null);
    let paging = discover.get("paging").unwrap_or(&serde_json::Value::Null);
    let next_page = if paging.get("hasMore").and_then(serde_json::Value::as_bool) == Some(true)
        && paging.get("isLoading").and_then(serde_json::Value::as_bool) != Some(true)
        && paging.get("error").is_none_or(serde_json::Value::is_null)
    {
        filters
            .get("transportUrl")
            .and_then(serde_json::Value::as_str)
            .zip(filters.get("catalogId").and_then(serde_json::Value::as_str))
            .map(|(transport_url, catalog_id)| {
                serde_json::json!({
                    "type": "discoverPageRequested",
                    "transportUrl": transport_url,
                    "contentType": discover.get("contentType"),
                    "catalogId": catalog_id,
                    "skip": paging.get("nextSkip"),
                    "genre": filters.pointer("/extra/genre"),
                    "search": filters.pointer("/extra/search"),
                })
            })
    } else {
        None
    };
    let catalogs = discover
        .get("catalogs")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .take(24)
                .filter_map(|item| {
                    let key = value_string(item, "key")?;
                    Some(DiscoverCatalog {
                        label: first_value_string(item, &["label", "name", "id"])
                            .unwrap_or_else(|| "Catalog".to_owned()),
                        key,
                        content_type: value_string(item, "type")
                            .unwrap_or_else(|| "movie".to_owned()),
                        extras: item
                            .get("extras")
                            .and_then(serde_json::Value::as_array)
                            .map(|extras| {
                                extras
                                    .iter()
                                    .filter_map(|extra| {
                                        let name = value_string(extra, "name")?;
                                        let options = extra
                                            .get("options")
                                            .and_then(serde_json::Value::as_array)?
                                            .iter()
                                            .filter_map(serde_json::Value::as_str)
                                            .map(ToOwned::to_owned)
                                            .collect::<Vec<_>>();
                                        Some((name, options))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        required_extras: item
                            .get("extras")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter(|extra| {
                                extra.get("isRequired").and_then(serde_json::Value::as_bool)
                                    == Some(true)
                            })
                            .filter_map(|extra| value_string(extra, "name"))
                            .collect(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    DiscoverModel {
        language: snapshot_language(snapshot),
        content_type: value_string(discover, "contentType").unwrap_or_else(|| "movie".to_owned()),
        content_types: discover
            .get("contentTypes")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        selected_catalog_key: discover
            .pointer("/filters/catalogKey")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        selected_extra_name: discover
            .pointer("/filters/extra")
            .and_then(serde_json::Value::as_object)
            .and_then(|extra| extra.keys().find(|key| key.as_str() != "search"))
            .cloned()
            .unwrap_or_default(),
        selected_extra_value: discover
            .pointer("/filters/extra")
            .and_then(serde_json::Value::as_object)
            .and_then(|extra| {
                extra
                    .iter()
                    .find(|(key, _)| key.as_str() != "search")
                    .and_then(|(_, value)| value.as_str())
            })
            .unwrap_or_default()
            .to_owned(),
        query: discover
            .pointer("/filters/extra/search")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        catalogs,
        results: discover
            .get("results")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().map(core_home_card).collect())
            .unwrap_or_default(),
        generation: discover
            .get("generation")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        next_page,
        is_loading: discover
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        catalogs_loading: discover
            .get("catalogsLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        error: discover
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        sections: Vec::new(),
    }
}

/// Refreshes fast-changing Discover state without re-projecting the entire
/// result array. A page response only appends to the current generation, so
/// convert just the new tail and update the paging flags in place.
pub fn refresh_discover_model_from_core_snapshot(
    snapshot: &serde_json::Value,
    model: &mut DiscoverModel,
) -> bool {
    let discover = snapshot.get("discover").unwrap_or(&serde_json::Value::Null);
    let filters = discover.get("filters").unwrap_or(&serde_json::Value::Null);
    let paging = discover.get("paging").unwrap_or(&serde_json::Value::Null);
    let generation = discover
        .get("generation")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let content_type = value_string(discover, "contentType").unwrap_or_else(|| "movie".to_owned());
    let selected_catalog_key = filters
        .get("catalogKey")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let searching = snapshot
        .pointer("/search/query")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|query| !query.trim().is_empty());
    if searching || !model.query.is_empty() {
        return false;
    }
    let Some(items) = discover
        .get("results")
        .and_then(serde_json::Value::as_array)
    else {
        return false;
    };
    if generation != model.generation
        || content_type != model.content_type
        || selected_catalog_key != model.selected_catalog_key
        || items.len() < model.results.len()
    {
        return false;
    }
    let catalog_count = discover
        .get("catalogs")
        .and_then(serde_json::Value::as_array)
        .map(|catalogs| catalogs.iter().take(24).count())
        .unwrap_or_default();
    if catalog_count != model.catalogs.len() {
        return false;
    }

    model
        .results
        .extend(items[model.results.len()..].iter().map(core_home_card));
    model.next_page = if paging.get("hasMore").and_then(serde_json::Value::as_bool) == Some(true)
        && paging.get("isLoading").and_then(serde_json::Value::as_bool) != Some(true)
        && paging.get("error").is_none_or(serde_json::Value::is_null)
    {
        filters
            .get("transportUrl")
            .and_then(serde_json::Value::as_str)
            .zip(filters.get("catalogId").and_then(serde_json::Value::as_str))
            .map(|(transport_url, catalog_id)| {
                serde_json::json!({
                    "type": "discoverPageRequested",
                    "transportUrl": transport_url,
                    "contentType": discover.get("contentType"),
                    "catalogId": catalog_id,
                    "skip": paging.get("nextSkip"),
                    "genre": filters.pointer("/extra/genre"),
                    "search": filters.pointer("/extra/search"),
                })
            })
    } else {
        None
    };
    model.is_loading = discover
        .get("isLoading")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    model.catalogs_loading = discover
        .get("catalogsLoading")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    model.error = discover
        .get("error")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);
    true
}

/// Convert a batch of Core Discover records into the shared UI card model.
/// Desktop invokes this from a background worker so page-sized projections do
/// not block the render/input thread when a catalog page arrives.
pub fn discover_cards_from_core_values(items: Vec<serde_json::Value>) -> Vec<HomeCard> {
    items.iter().map(core_home_card).collect()
}

pub fn calendar_model_from_core_snapshot(snapshot: &serde_json::Value) -> CalendarModel {
    let calendar = snapshot.get("calendar").unwrap_or(&serde_json::Value::Null);
    let year = calendar
        .get("year")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0) as i32;
    let month = calendar
        .get("month")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0) as i32;
    let mut raw_items = Vec::new();
    for key in ["items", "localItems", "externalItems"] {
        if let Some(items) = calendar.get(key).and_then(serde_json::Value::as_array) {
            raw_items.extend(items.iter().cloned());
        }
    }
    CalendarModel {
        language: snapshot_language(snapshot),
        year,
        month,
        selected_day: None,
        today: Some(today_iso()),
        is_loading: calendar
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        error: calendar
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        entries: raw_items
            .into_iter()
            .take(256)
            .filter_map(|item| {
                let date = first_value_string(
                    &item,
                    &["dateIso", "airDate", "released", "releaseDate", "date"],
                )?
                .chars()
                .take(10)
                .collect();
                let card = core_home_card(&item);
                let show = item
                    .get("meta")
                    .and_then(|meta| first_value_string(meta, &["name", "title"]))
                    .or_else(|| value_string(&item, "title"))
                    .unwrap_or_else(|| card.title.clone());
                let episode = first_value_string(&item, &["subtitle", "episodeTitle"])
                    .filter(|episode| *episode != show)
                    .unwrap_or_default();
                let still_url = first_value_string(
                    &item,
                    &["episodePoster", "background", "backdrop", "poster"],
                )
                .or_else(|| {
                    item.get("meta")
                        .and_then(|meta| first_value_string(meta, &["background", "poster"]))
                })
                .or_else(|| card.artwork_url.clone());
                Some(CalendarEntry {
                    date,
                    show,
                    episode,
                    still_url,
                    card,
                })
            })
            .collect(),
    }
}

impl HomeModel {
    pub fn resume_for(&self, id: &str) -> Option<&HomeCard> {
        self.cards
            .iter()
            .find(|card| card.row_kind == HomeRowKind::Continue && card.id.as_deref() == Some(id))
    }
}

pub fn resume_episode(card: &HomeCard) -> Option<(i64, i64)> {
    let number = |keys: &[&str]| keys.iter().find_map(|key| card.raw.get(*key)?.as_i64());
    Some((
        number(&["lastEpisodeSeason", "season"])?,
        number(&["lastEpisodeNumber", "episode"])?,
    ))
}

pub fn play_label(language: &str, resume: Option<&HomeCard>, episode: Option<(i64, i64)>) -> String {
    let action = localized(if resume.is_some() { "common.continue" } else { "common.play" }, language);
    match resume.and_then(resume_episode).or(episode) {
        Some((season, number)) => format!(
            "{action}  {}",
            localized("format.season_episode_short", language)
                .replacen("%s", &season.to_string(), 1)
                .replacen("%s", &number.to_string(), 1)
        ),
        None => action,
    }
}

pub fn hero_from_meta(item: &serde_json::Value, language: &str) -> HomeHero {
    core_home_hero(item, language)
}

fn core_home_card(item: &serde_json::Value) -> HomeCard {
    core_home_card_for_kind(item, HomeRowKind::Poster)
}

fn core_home_card_for_kind(item: &serde_json::Value, kind: HomeRowKind) -> HomeCard {
    let progress = item
        .get("resumeProgressPercent")
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32 / 100.0)
        .or_else(|| {
            let offset = item.get("timeOffset").and_then(serde_json::Value::as_f64)?;
            let duration = item.get("duration").and_then(serde_json::Value::as_f64)?;
            (duration > 0.0).then_some((offset / duration) as f32)
        })
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    HomeCard {
        id: value_string(item, "id"),
        item_type: value_string(item, "type"),
        title: first_value_string(item, &["name", "title"])
            .unwrap_or_else(|| "Untitled".to_owned()),
        subtitle: first_value_string(item, &["episodeLabel", "lastEpisodeName", "releaseLabel"])
            .or_else(|| value_display(item, "year"))
            .unwrap_or_default(),
        progress,
        artwork_url: if matches!(kind, HomeRowKind::Continue) {
            first_value_string(
                item,
                &[
                    "continueWatchingBackground",
                    "continueWatchingPoster",
                    "lastEpisodeThumbnail",
                    "background",
                    "backgroundUrl",
                    "poster",
                    "posterUrl",
                    "backdrop",
                    "backdropUrl",
                    "artworkUrl",
                ],
            )
        } else {
            first_value_string(
                item,
                &[
                    "poster",
                    "posterUrl",
                    "resolvedPosterUrl",
                    "artworkUrl",
                    "background",
                    "backgroundUrl",
                    "backdrop",
                    "backdropUrl",
                ],
            )
        },
        collection_shape: if matches!(kind, HomeRowKind::Collection) {
            first_value_string(item, &["reason", "shape", "tileShape"])
        } else {
            None
        },
        hide_title: item
            .get("hideTitle")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        motion_url: if matches!(kind, HomeRowKind::Collection) {
            value_string(item, "focusGifUrl")
        } else {
            None
        },
        motion_enabled: item
            .get("focusGifEnabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        overlay: if matches!(kind, HomeRowKind::Collection) {
            Default::default()
        } else {
            poster_overlay::poster_facts(item)
        },
        logo_url: first_value_string(item, &["logo", "logoUrl", "clearLogo"]),
        backdrop_url: first_value_string(
            item,
            &["background", "backgroundUrl", "backdrop", "backdropUrl"],
        ),
        raw: item.clone(),
        row_kind: kind,
    }
}

fn value_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
}

fn value_display(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(|raw| match raw {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    })
}

fn first_value_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| value_string(value, key))
}

fn hero_meta_line(value: &serde_json::Value, language: &str) -> String {
    let series = matches!(
        value_string(value, "type").as_deref(),
        Some("series" | "tv" | "show")
    )
    .then(|| localized("auto.series", language));
    let year = value
        .get("year")
        .and_then(serde_json::Value::as_i64)
        .map(|year| year.to_string())
        .or_else(|| value_string(value, "year"));
    let runtime = value_string(value, "runtime").or_else(|| {
        value
            .get("duration")
            .and_then(serde_json::Value::as_i64)
            .filter(|duration| *duration > 0)
            .map(|duration| format!("{}min", duration / 60))
    });
    let genres = value
        .get("genres")
        .and_then(serde_json::Value::as_array)
        .map(|genres| {
            genres
                .iter()
                .filter_map(serde_json::Value::as_str)
                .filter(|genre| !genre.eq_ignore_ascii_case("tv movie"))
                .take(2)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|genres| !genres.is_empty());
    [series, year, runtime, genres]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

pub trait HomeAssets {
    fn background(&self) -> TextureId;
    fn texture(&mut self, url: Option<&str>) -> Option<TextureId>;
    /// Requests artwork at the raster size this component actually needs.
    /// Hosts may map this to a provider-sized URL and/or a bounded decode;
    /// the fallback keeps small test hosts source-compatible.
    fn texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        _priority: ArtworkPriority,
    ) -> Option<TextureId> {
        let _ = target_size;
        self.texture(url)
    }
    /// Starts loading artwork without requiring its row/card to be painted yet.
    /// Hosts can use this to prepare the next shelf while the current shelf is
    /// on screen. The default keeps simple/test hosts compatible.
    fn prefetch(&mut self, url: Option<&str>) {
        let _ = self.texture(url);
    }
    fn prefetch_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) {
        let _ = self.texture_for(url, target_size, priority);
    }
    fn prefetch_animated_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        _priority: ArtworkPriority,
    ) {
        let _ = (url, target_size);
    }
    fn texture_size(&self, _url: Option<&str>) -> Option<[u32; 2]> {
        None
    }
    fn artwork_tones(&self, _url: Option<&str>) -> Option<[[u8; 3]; 2]> {
        None
    }
    fn animated_texture_for(
        &mut self,
        _url: Option<&str>,
        _target_size: [u32; 2],
        _priority: ArtworkPriority,
    ) -> Option<AnimatedTexture> {
        None
    }
    fn cached_texture(&self, _url: Option<&str>) -> Option<TextureId> {
        None
    }
    /// Returns a texture rasterized from the shared native SVG icon set.
    /// Hosts upload the same SVG bytes; the UI never substitutes glyphs or
    /// hand-drawn approximations when an icon is requested.
    fn icon(&self, _name: &str) -> Option<TextureId> {
        None
    }
    fn logo(&self, _name: &str) -> Option<(TextureId, Vec2)> {
        None
    }
    fn active_profile_name(&self) -> Option<&str> {
        None
    }
    fn brand_mark(&self) -> Option<TextureId> {
        None
    }
    fn ambient_glow(&self) -> Option<TextureId> {
        None
    }
    fn custom_background_url(&self) -> Option<&str> {
        None
    }
    fn active_profile_avatar_url(&self) -> Option<&str> {
        None
    }
    fn accent_color(&self) -> Option<Color32> {
        None
    }
}

fn metrics_for_assets(viewport: Viewport, assets: &impl HomeAssets) -> UiMetrics {
    let mut metrics = UiMetrics::for_viewport(viewport);
    if let Some(accent) = assets.accent_color() {
        metrics.accent = accent;
        let luminance =
            0.2126 * accent.r() as f32 + 0.7152 * accent.g() as f32 + 0.0722 * accent.b() as f32;
        metrics.accent_foreground = if luminance > 140.0 {
            Color32::from_rgb(20, 20, 22)
        } else {
            Color32::WHITE
        };
    }
    metrics
}

#[derive(Clone, Debug, Default)]
struct HeroFade {
    key: String,
    from: Option<(TextureId, [u32; 2])>,
    current: Option<(TextureId, [u32; 2])>,
    started: f64,
}

#[derive(Clone)]
struct LastReadyHeroTexture {
    texture: TextureId,
    size: [u32; 2],
    hero: HomeHero,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArtworkPriority {
    Hero,
    #[default]
    Visible,
    Prefetch,
}

/// Selects a provider-sized source when the URL is a TMDB image URL. Other
/// providers keep their original URL; their loaders still bound the decoded
/// raster before uploading it to the GPU.
pub fn artwork_request_url(url: &str, target_size: [u32; 2]) -> String {
    if !url.contains("image.tmdb.org") {
        return url.to_owned();
    }
    let marker = "/t/p/";
    let Some(marker_start) = url.find(marker) else {
        return url.to_owned();
    };
    let path_start = marker_start + marker.len();
    let Some((_, path)) = url[path_start..].split_once('/') else {
        return url.to_owned();
    };
    let max_side = target_size[0].max(target_size[1]);
    let variant = if max_side <= 400 {
        "w342"
    } else if max_side <= 650 {
        "w500"
    } else if max_side <= 1000 {
        "w780"
    } else if max_side <= 1280 {
        "w1280"
    } else {
        "original"
    };
    format!("{}{variant}/{path}", &url[..path_start])
}

/// Geometry consumed by every shared Rust screen.
///
/// These values are read from `shared/contracts/ui-tokens.json` through the
/// same parser used by the low-level renderer. The small fallbacks are only a
/// malformed-token safety net; they are not a second design system. Keeping
/// the lookup here prevents platform hosts from growing their own fixed UI
/// dimensions while still allowing the renderer to paint when a user ships a
/// partially migrated custom token document.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMetrics {
    pub background: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub navigation: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_foreground: Color32,
    pub focus: Color32,
    pub page_padding: f32,
    pub screen_padding: f32,
    pub content_header_top_mobile: f32,
    pub content_header_top: f32,
    pub detail_header_top_mobile: f32,
    pub detail_header_top: f32,
    pub detail_poster_width_mobile: f32,
    pub detail_poster_width_desktop: f32,
    pub detail_poster_width_tv: f32,
    pub detail_content_gap: f32,
    pub detail_mobile_content_gap: f32,
    pub detail_action_gap: f32,
    pub detail_action_top_mobile_offset: f32,
    pub detail_action_top_offset: f32,
    pub detail_similar_top_mobile_offset: f32,
    pub detail_similar_top_offset: f32,
    pub detail_play_width: f32,
    pub detail_watchlist_width: f32,
    pub detail_back_width: f32,
    pub home_hero_content_max_width_desktop: f32,
    pub home_hero_logo_max_width_desktop: f32,
    pub home_hero_logo_height_desktop: f32,
    pub home_hero_synopsis_height_desktop: f32,
    pub home_hero_synopsis_size_desktop: f32,
    pub library_search_reserved_width: f32,
    pub library_search_min_width: f32,
    pub library_search_max_width: f32,
    pub library_sort_width: f32,
    pub library_sort_min_width: f32,
    pub calendar_panel_width_desktop: f32,
    pub calendar_panel_width_tv: f32,
    pub screen_card_min_width: f32,
    pub discover_card_title_offset: f32,
    pub discover_card_subtitle_offset: f32,
    pub settings_content_offset: f32,
    pub settings_nav_width: f32,
    pub settings_screen_padding_desktop: f32,
    pub settings_content_max_width_desktop: f32,
    pub settings_nav_item_height_desktop: f32,
    pub settings_nav_item_gap_desktop: f32,
    pub settings_section_title_size_desktop: f32,
    pub settings_description_size_desktop: f32,
    pub settings_row_label_size_desktop: f32,
    pub settings_row_value_size_desktop: f32,
    pub settings_extended_top: f32,
    pub settings_extended_line_spacing: f32,
    pub settings_extended_line_spacing_tv: f32,
    pub settings_extended_line_inset: f32,
    pub settings_extended_input_height: f32,
    pub settings_extended_action_height: f32,
    pub settings_extended_action_gap: f32,
    pub settings_extended_addon_action_width: f32,
    pub settings_extended_plugin_action_width: f32,
    pub settings_extended_small_action_width: f32,
    pub settings_extended_small_action_height: f32,
    pub settings_extended_scraper_action_width: f32,
    pub settings_extended_repo_right_inset: f32,
    pub settings_extended_scraper_right_inset: f32,
    pub screen_control_height: f32,
    pub screen_control_radius: f32,
    pub focus_ring_expand: f32,
    pub focus_ring_radius: f32,
    pub focus_ring_width: f32,
    pub card_overlay_height: f32,
    pub card_content_padding: f32,
    pub settings_card_gap: f32,
    pub settings_card_height_mobile: f32,
    pub settings_card_height_desktop: f32,
    pub settings_card_padding: f32,
    pub settings_title_top: f32,
    pub settings_description_top: f32,
    pub settings_row_inset: f32,
    pub settings_row_top: f32,
    pub settings_toggle_right_inset: f32,
    pub settings_toggle_top_inset: f32,
    pub settings_toggle_on_knob_offset: f32,
    pub settings_toggle_off_knob_offset: f32,
    pub settings_row_height: f32,
    pub settings_row_spacing: f32,
    pub settings_toggle_width: f32,
    pub settings_toggle_height: f32,
    pub settings_toggle_knob_radius: f32,
    pub calendar_grid_gap_mobile: f32,
    pub calendar_grid_gap: f32,
    pub calendar_cell_height_mobile: f32,
    pub calendar_cell_height_desktop: f32,
    pub calendar_cell_height_tv: f32,
    pub calendar_cell_radius: f32,
    pub calendar_day_thumb_width: f32,
    pub calendar_day_thumb_height: f32,
    pub library_empty_offset: f32,
    pub discover_empty_offset: f32,
    pub calendar_empty_offset: f32,
    pub calendar_day_padding: f32,
    pub calendar_day_top: f32,
    pub calendar_entry_top: f32,
    pub calendar_entry_line_height: f32,
    pub calendar_thumb_right_inset: f32,
    pub calendar_thumb_top: f32,
    pub detail_error_gap: f32,
    pub detail_stream_gap: f32,
    pub similar_overlay_height: f32,
    pub similar_text_offset: f32,
    pub section_gap: f32,
    pub control_gap: f32,
    pub card_radius: f32,
    pub card_progress_height: f32,
    pub card_title_size: f32,
    pub card_subtitle_size: f32,
    pub horizontal_card_width: f32,
    pub horizontal_card_height: f32,
    pub home_continue_card_width: f32,
    pub home_continue_card_height: f32,
    pub poster_card_width: f32,
    pub poster_card_height: f32,
    pub mobile_billboard_height: f32,
    pub nav_horizontal_padding: f32,
    pub nav_vertical_padding: f32,
    pub nav_item_gap: f32,
    pub nav_icon_size: f32,
    pub nav_item_horizontal_padding: f32,
    pub nav_item_vertical_padding: f32,
    pub nav_label_size: f32,
    pub screen_title_size_mobile: f32,
    pub screen_title_size: f32,
    pub screen_title_size_tv: f32,
    pub screen_body_size: f32,
    pub screen_body_size_tv: f32,
    pub screen_section_title_size: f32,
    pub screen_section_title_size_tv: f32,
    pub screen_card_title_size: f32,
    pub screen_card_title_size_tv: f32,
    pub screen_card_subtitle_size: f32,
    pub catalog_title_size: f32,
    pub horizontal_spacing: f32,
    pub vertical_spacing: f32,
    pub minimum_card_width: f32,
    pub poster_height_ratio: f32,
    pub horizontal_card_height_ratio: f32,
    pub focused_scale: f32,
}

impl UiMetrics {
    pub fn for_viewport(viewport: Viewport) -> Self {
        let tokens = active_shared_ui_tokens().expect("shared Fluxa tokens must be valid");
        let dark = active_theme("fluxa-dark").expect("shared Fluxa theme must be valid");
        let number = |path: &[&str], fallback: f32| {
            let mut value = &tokens.layout;
            for key in path {
                let Some(next) = value.get(*key) else {
                    return fallback;
                };
                value = next;
            }
            value.as_f64().map(|value| value as f32).unwrap_or(fallback)
        };
        let platform = if viewport.is_compact() {
            "mobile"
        } else if viewport.is_tv() {
            "tv"
        } else {
            "desktop"
        };
        let window_class = match viewport.size_class() {
            SizeClass::Compact => "compact",
            SizeClass::Medium => "medium",
            SizeClass::Expanded => "expanded",
        };
        let roomy = !viewport.is_compact() && !viewport.is_tv();
        let pick = |roomy_value: f32, compact_value: f32| {
            if roomy { roomy_value } else { compact_value }
        };
        let common_dp = |key, fallback| number(&["common", "dp", key], fallback);
        let common_sp = |key, fallback| number(&["common", "sp", key], fallback);
        let platform_dp = |key, fallback| number(&["platforms", platform, "dp", key], fallback);
        let window_dp =
            |key, fallback| number(&["windowClasses", window_class, "dp", key], fallback);
        let window_sp =
            |key, fallback| number(&["windowClasses", window_class, "sp", key], fallback);
        let color = |token: &str, fallback: Color32| {
            dark.color(token)
                .map(|value| {
                    Color32::from_rgba_unmultiplied(
                        (value.r * 255.0).round() as u8,
                        (value.g * 255.0).round() as u8,
                        (value.b * 255.0).round() as u8,
                        (value.a * 255.0).round() as u8,
                    )
                })
                .unwrap_or(fallback)
        };
        let mut metrics = Self {
            background: color("background", Color32::from_rgb(7, 7, 9)),
            surface: color("surface", Color32::from_rgb(20, 20, 22)),
            surface_raised: color("surfaceRaised", Color32::from_rgb(28, 28, 31)),
            navigation: color("navigation", Color32::from_rgb(20, 20, 22)),
            text_primary: color("textPrimary", Color32::WHITE),
            text_secondary: color("textSecondary", Color32::from_rgb(184, 184, 190)),
            text_muted: color("textMuted", Color32::from_rgb(138, 138, 145)),
            accent: color("accent", Color32::from_rgb(239, 78, 52)),
            accent_foreground: color("accentForeground", Color32::WHITE),
            focus: color("focus", Color32::WHITE),
            page_padding: match viewport.size_class() {
                _ if viewport.is_tv() => number(
                    &["windowClasses", "expanded", "dp", "pageHorizontalPadding"],
                    48.0,
                ),
                SizeClass::Expanded => dark.spacing.screen_padding,
                _ => window_dp("pageHorizontalPadding", 16.0),
            },
            screen_padding: dark.spacing.screen_padding,
            content_header_top_mobile: common_dp("contentHeaderTopMobile", 86.0),
            content_header_top: common_dp("contentHeaderTop", 110.0),
            detail_header_top_mobile: common_dp("detailHeaderTopMobile", 82.0),
            detail_header_top: common_dp("detailHeaderTop", 116.0),
            detail_poster_width_mobile: common_dp("detailPosterWidthMobile", 132.0),
            detail_poster_width_desktop: common_dp("detailPosterWidthDesktop", 220.0),
            detail_poster_width_tv: common_dp("detailPosterWidthTv", 270.0),
            detail_content_gap: common_dp("detailContentGap", 28.0),
            detail_mobile_content_gap: common_dp("detailMobileContentGap", 22.0),
            detail_action_gap: common_dp("detailActionGap", 8.0),
            detail_action_top_mobile_offset: common_dp("detailActionTopMobileOffset", 182.0),
            detail_action_top_offset: common_dp("detailActionTopOffset", 188.0),
            detail_similar_top_mobile_offset: common_dp("detailSimilarTopMobileOffset", 300.0),
            detail_similar_top_offset: common_dp("detailSimilarTopOffset", 34.0),
            detail_play_width: common_dp("detailPlayWidth", 118.0),
            detail_watchlist_width: common_dp("detailWatchlistWidth", 148.0),
            detail_back_width: common_dp("detailBackWidth", 92.0),
            home_hero_content_max_width_desktop: common_dp("homeHeroContentMaxWidthDesktop", 980.0),
            home_hero_logo_max_width_desktop: common_dp("homeHeroLogoMaxWidthDesktop", 820.0),
            home_hero_logo_height_desktop: common_dp("homeHeroLogoHeightDesktop", 148.0),
            home_hero_synopsis_height_desktop: common_dp("homeHeroSynopsisHeightDesktop", 88.0),
            home_hero_synopsis_size_desktop: common_sp("homeHeroSynopsisSizeDesktop", 16.0),
            library_search_reserved_width: common_dp("librarySearchReservedWidth", 150.0),
            library_search_min_width: common_dp("librarySearchMinWidth", 100.0),
            library_search_max_width: common_dp("librarySearchMaxWidth", 360.0),
            library_sort_width: common_dp("librarySortWidth", 130.0),
            library_sort_min_width: common_dp("librarySortMinWidth", 50.0),
            calendar_panel_width_desktop: common_dp("calendarPanelWidthDesktop", 360.0),
            calendar_panel_width_tv: common_dp("calendarPanelWidthTv", 440.0),
            screen_card_min_width: common_dp("screenCardMinWidth", 120.0),
            discover_card_title_offset: common_dp("discoverCardTitleOffset", 46.0),
            discover_card_subtitle_offset: common_dp("discoverCardSubtitleOffset", 25.0),
            settings_content_offset: pick(
                common_dp("settingsNavWidthDesktop", 268.0)
                    + common_dp("settingsPanelGapDesktop", 32.0),
                common_dp("settingsContentOffset", 220.0),
            ),
            settings_nav_width: pick(
                common_dp("settingsNavWidthDesktop", 268.0),
                common_dp("settingsNavWidth", 196.0),
            ),
            settings_screen_padding_desktop: common_dp("settingsScreenPaddingDesktop", 48.0),
            settings_content_max_width_desktop: common_dp("settingsContentMaxWidthDesktop", 2400.0),
            settings_nav_item_height_desktop: common_dp("settingsNavItemHeightDesktop", 44.0),
            settings_nav_item_gap_desktop: common_dp("settingsNavItemGapDesktop", 6.0),
            settings_section_title_size_desktop: common_sp("settingsSectionTitleSizeDesktop", 24.0),
            settings_description_size_desktop: common_sp("settingsDescriptionSizeDesktop", 14.0),
            settings_row_label_size_desktop: common_sp("settingsRowLabelSizeDesktop", 18.0),
            settings_row_value_size_desktop: common_sp("settingsRowValueSizeDesktop", 15.0),
            settings_extended_top: common_dp("settingsExtendedTop", 76.0),
            settings_extended_line_spacing: common_dp("settingsExtendedLineSpacing", 34.0),
            settings_extended_line_spacing_tv: common_dp("settingsExtendedLineSpacingTv", 42.0),
            settings_extended_line_inset: common_dp("settingsExtendedLineInset", 20.0),
            settings_extended_input_height: common_dp("settingsExtendedInputHeight", 40.0),
            settings_extended_action_height: common_dp("settingsExtendedActionHeight", 38.0),
            settings_extended_action_gap: common_dp("settingsExtendedActionGap", 10.0),
            settings_extended_addon_action_width: common_dp("settingsAddonActionWidth", 148.0),
            settings_extended_plugin_action_width: common_dp("settingsPluginActionWidth", 156.0),
            settings_extended_small_action_width: common_dp("settingsSmallActionWidth", 76.0),
            settings_extended_small_action_height: common_dp("settingsSmallActionHeight", 34.0),
            settings_extended_scraper_action_width: common_dp("settingsScraperActionWidth", 96.0),
            settings_extended_repo_right_inset: common_dp("settingsRepoRightInset", 80.0),
            settings_extended_scraper_right_inset: common_dp("settingsScraperRightInset", 104.0),
            screen_control_height: common_dp("screenControlHeight", 36.0),
            screen_control_radius: common_dp("screenControlRadius", 9.0),
            focus_ring_expand: common_dp("focusRingExpand", 3.0),
            focus_ring_radius: common_dp("focusRingRadius", 10.0),
            focus_ring_width: common_dp("focusRingWidth", 2.5),
            card_overlay_height: common_dp("cardOverlayHeight", 82.0),
            card_content_padding: common_dp("cardContentPadding", 10.0),
            settings_card_gap: common_dp("settingsCardGap", 18.0),
            settings_card_height_mobile: common_dp("settingsCardHeightMobile", 168.0),
            settings_card_height_desktop: common_dp("settingsCardHeightDesktop", 154.0),
            settings_card_padding: pick(
                common_dp("settingsCardPaddingDesktop", 24.0),
                common_dp("settingsCardPadding", 16.0),
            ),
            settings_title_top: pick(
                common_dp("settingsTitleTopDesktop", 18.0),
                common_dp("settingsTitleTop", 14.0),
            ),
            settings_description_top: pick(
                common_dp("settingsDescriptionTopDesktop", 54.0),
                common_dp("settingsDescriptionTop", 39.0),
            ),
            settings_row_inset: pick(
                common_dp("settingsRowInsetDesktop", 24.0),
                common_dp("settingsRowInset", 14.0),
            ),
            settings_row_top: pick(
                common_dp("settingsRowTopDesktop", 88.0),
                common_dp("settingsRowTop", 67.0),
            ),
            settings_toggle_right_inset: pick(
                common_dp("settingsToggleWidthDesktop", 48.0) + 4.0,
                common_dp("settingsToggleRightInset", 44.0),
            ),
            settings_toggle_top_inset: pick(11.0, common_dp("settingsToggleTopInset", 10.0)),
            settings_toggle_on_knob_offset: pick(
                common_dp("settingsToggleOnKnobOffsetDesktop", 18.0),
                common_dp("settingsToggleOnKnobOffset", 14.0),
            ),
            settings_toggle_off_knob_offset: pick(
                common_dp("settingsToggleOffKnobOffsetDesktop", 38.0),
                common_dp("settingsToggleOffKnobOffset", 34.0),
            ),
            settings_row_height: pick(
                common_dp("settingsRowHeightDesktop", 44.0),
                common_dp("settingsRowHeight", 32.0),
            ),
            settings_row_spacing: pick(
                common_dp("settingsRowSpacingDesktop", 56.0),
                common_dp("settingsRowSpacing", 38.0),
            ),
            settings_toggle_width: pick(
                common_dp("settingsToggleWidthDesktop", 48.0),
                common_dp("settingsToggleWidth", 40.0),
            ),
            settings_toggle_height: pick(
                common_dp("settingsToggleHeightDesktop", 26.0),
                common_dp("settingsToggleHeight", 20.0),
            ),
            settings_toggle_knob_radius: pick(
                common_dp("settingsToggleKnobRadiusDesktop", 9.0),
                common_dp("settingsToggleKnobRadius", 7.0),
            ),
            calendar_grid_gap_mobile: common_dp("calendarGridGapMobile", 5.0),
            calendar_grid_gap: common_dp("calendarGridGap", 8.0),
            calendar_cell_height_mobile: common_dp("calendarCellHeightMobile", 76.0),
            calendar_cell_height_desktop: common_dp("calendarCellHeightDesktop", 96.0),
            calendar_cell_height_tv: common_dp("calendarCellHeightTv", 112.0),
            calendar_cell_radius: common_dp("calendarCellRadius", 9.0),
            calendar_day_thumb_width: common_dp("calendarDayThumbWidth", 23.0),
            calendar_day_thumb_height: common_dp("calendarDayThumbHeight", 32.0),
            library_empty_offset: common_dp("libraryEmptyOffset", 55.0),
            discover_empty_offset: common_dp("discoverEmptyOffset", 70.0),
            calendar_empty_offset: common_dp("calendarEmptyOffset", 150.0),
            calendar_day_padding: common_dp("calendarDayPadding", 8.0),
            calendar_day_top: common_dp("calendarDayTop", 7.0),
            calendar_entry_top: common_dp("calendarEntryTop", 31.0),
            calendar_entry_line_height: common_dp("calendarEntryLineHeight", 19.0),
            calendar_thumb_right_inset: common_dp("calendarThumbRightInset", 30.0),
            calendar_thumb_top: common_dp("calendarThumbTop", 7.0),
            detail_error_gap: common_dp("detailErrorGap", 12.0),
            detail_stream_gap: common_dp("detailStreamGap", 18.0),
            similar_overlay_height: common_dp("similarOverlayHeight", 30.0),
            similar_text_offset: common_dp("similarTextOffset", 20.0),
            section_gap: dark.spacing.section_gap,
            control_gap: dark.spacing.control_gap,
            card_radius: dark.shape.card_radius,
            card_progress_height: common_dp("cardProgressBarHeight", 4.0),
            card_title_size: common_sp("cardTitleSize", 12.0),
            card_subtitle_size: common_sp("cardSubtitleSize", 10.0),
            horizontal_card_width: platform_dp("episodeCardWidth", 300.0),
            horizontal_card_height: platform_dp("episodeCardHeight", 180.0),
            // Home's Continue Watching row uses the same responsive size
            // presets as Compose horizontalCardWidth/Height, not the larger
            // episode-card dimensions used by other surfaces.
            home_continue_card_width: window_dp("horizontalCardBase", 196.0),
            home_continue_card_height: window_dp("horizontalCardBase", 196.0)
                * number(&["common", "number", "horizontalCardHeightRatio"], 0.56),
            poster_card_width: platform_dp("posterCardWidth", 160.0),
            poster_card_height: platform_dp("posterCardHeight", 240.0),
            mobile_billboard_height: common_dp("mobileBillboardHeight", 540.0),
            nav_horizontal_padding: window_dp("navigationHorizontalPadding", 32.0),
            nav_vertical_padding: window_dp("navigationVerticalPadding", 12.0),
            nav_item_gap: window_dp("navigationItemSpacing", 8.0),
            nav_icon_size: window_dp("navigationIconSize", 26.0),
            nav_item_horizontal_padding: window_dp("navigationItemHorizontalPadding", 14.0),
            nav_item_vertical_padding: window_dp("navigationItemVerticalPadding", 9.0),
            nav_label_size: window_sp("navigationLabelSize", 14.0),
            screen_title_size_mobile: common_sp("screenTitleSizeMobile", 32.0),
            screen_title_size: common_sp("screenTitleSize", 34.0),
            screen_title_size_tv: common_sp("screenTitleSizeTv", 42.0),
            screen_body_size: common_sp("screenBodySize", 15.0),
            screen_body_size_tv: common_sp("screenBodySizeTv", 18.0),
            screen_section_title_size: common_sp("screenSectionTitleSize", 20.0),
            screen_section_title_size_tv: common_sp("screenSectionTitleSizeTv", 24.0),
            screen_card_title_size: common_sp("screenCardTitleSize", 15.0),
            screen_card_title_size_tv: common_sp("screenCardTitleSizeTv", 17.0),
            screen_card_subtitle_size: common_sp("screenCardSubtitleSize", 12.0),
            catalog_title_size: window_sp("catalogTitleSize", 26.0),
            horizontal_spacing: window_dp("horizontalSpacing", 16.0),
            vertical_spacing: window_dp("verticalSpacing", 20.0),
            minimum_card_width: window_dp("minimumCardWidth", 170.0),
            poster_height_ratio: number(&["common", "number", "posterHeightRatio"], 1.5),
            horizontal_card_height_ratio: number(
                &["common", "number", "horizontalCardHeightRatio"],
                0.56,
            ),
            focused_scale: number(&["common", "number", "cardFocusedScale"], 1.12),
        };
        if poster_overlay::landscape() {
            metrics.poster_card_width = metrics.home_continue_card_width;
            metrics.poster_card_height = metrics.poster_card_width * 0.5625;
        }
        metrics
    }

    pub fn navigation_label_size(self, tv: bool) -> f32 {
        self.nav_label_size + if tv { 3.0 } else { 0.0 }
    }

    pub fn navigation_icon_size(self, tv: bool) -> f32 {
        if tv { 25.0 } else { 19.0 }
    }

    pub fn navigation_item_width(self, label: &str, tv: bool) -> f32 {
        28.0 + self.navigation_icon_size(tv)
            + 8.0
            + estimated_navigation_text_width(label, self.navigation_label_size(tv))
    }

    pub fn navigation_profile_width(self, profile_name: &str, tv: bool) -> f32 {
        (26.0
            + NAV_AVATAR_RADIUS * 2.0
            + 8.0
            + estimated_navigation_text_width(profile_name, self.navigation_label_size(tv)))
        .clamp(72.0, 200.0)
    }

    pub fn navigation_bar_width(self, viewport: Viewport, profile_name: &str) -> f32 {
        let compact = viewport.is_compact();
        let tv = viewport.is_tv();
        let margin = if compact {
            self.page_padding
        } else if tv {
            self.page_padding.max(32.0)
        } else {
            self.page_padding
        };
        let nav_width = ["Home", "Library", "Discover", "Calendar"]
            .into_iter()
            .map(|label| self.navigation_item_width(label, tv))
            .sum::<f32>();
        (if compact {
            (viewport.width - margin * 2.0).max(280.0)
        } else {
            nav_width
                + NAV_ITEM_GAP * 4.0
                + self.navigation_profile_width(profile_name, tv)
        })
        .min((viewport.width - margin * 2.0).max(1.0))
    }
}

const NAV_BAR_TOP: f32 = 14.0;
const NAV_ITEM_HEIGHT: f32 = 40.0;
const NAV_BAR_PADDING: f32 = 6.0;
const NAV_ITEM_GAP: f32 = 2.0;
const NAV_AVATAR_RADIUS: f32 = 13.5;

fn estimated_navigation_text_width(text: &str, size: f32) -> f32 {
    text.chars()
        .map(|character| match character {
            'i' | 'l' | 'I' | '!' | '.' => 0.30,
            'm' | 'w' | 'M' | 'W' => 0.82,
            ' ' => 0.32,
            _ => 0.54,
        })
        .sum::<f32>()
        * size
        + 4.0
}

fn home_row_dimensions(metrics: UiMetrics, kind: HomeRowKind) -> (f32, f32, f32) {
    match kind {
        HomeRowKind::Continue => (
            metrics.home_continue_card_width,
            metrics.home_continue_card_height,
            metrics.home_continue_card_height,
        ),
        HomeRowKind::Poster => (
            metrics.poster_card_width,
            metrics.poster_card_height,
            metrics.poster_card_height
                + metrics.control_gap
                + metrics.screen_card_title_size
                + metrics.screen_card_subtitle_size
                + metrics.control_gap,
        ),
        HomeRowKind::Collection => (
            156.0,
            234.0,
            234.0 + metrics.control_gap + metrics.screen_card_title_size,
        ),
    }
}

fn home_collection_card_dimensions(card: &HomeCard) -> (f32, f32) {
    match card
        .collection_shape
        .as_deref()
        .unwrap_or("poster")
        .to_ascii_lowercase()
        .as_str()
    {
        "wide" | "landscape" => (280.0, 158.0),
        "square" => (150.0, 150.0),
        _ => (156.0, 234.0),
    }
}

fn home_row_body_height(metrics: UiMetrics, cards: &[HomeCard], kind: HomeRowKind) -> f32 {
    if kind != HomeRowKind::Collection {
        return home_row_dimensions(metrics, kind).2;
    }
    let max_tile_height = cards
        .iter()
        .map(home_collection_card_dimensions)
        .map(|(_, height)| height)
        .fold(0.0_f32, f32::max);
    max_tile_height + metrics.control_gap + metrics.screen_card_title_size
}

fn home_metrics(viewport: Viewport) -> UiMetrics {
    let mut metrics = UiMetrics::for_viewport(viewport);
    if viewport.is_compact() {
        metrics.home_continue_card_width *= 1.14;
        metrics.home_continue_card_height *= 1.14;
    } else if !viewport.is_tv() {
        metrics.home_continue_card_width *= 1.10;
        metrics.home_continue_card_height *= 1.10;
    }
    metrics
}

/// Height occupied by a shelf heading and the gap between it and its cards.
/// Keep this in the shared geometry calculation: omitting it made each next
/// shelf start while the previous shelf's cards were still on screen.
fn home_row_heading_height(metrics: UiMetrics, tv: bool) -> f32 {
    let title_size = if tv {
        metrics.catalog_title_size
    } else {
        metrics.catalog_title_size - 4.0
    };
    title_size * 1.25 + metrics.control_gap
}

fn home_row_total_height(metrics: UiMetrics, kind: HomeRowKind, tv: bool) -> f32 {
    let (_, _, body_height) = home_row_dimensions(metrics, kind);
    home_row_heading_height(metrics, tv) + body_height
}

fn home_artwork_signature(home: &HomeModel) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    home.background_url.hash(&mut hasher);
    home.logo_url.hash(&mut hasher);
    for hero in &home.hero_slides {
        hero.background_url.hash(&mut hasher);
        hero.logo_url.hash(&mut hasher);
    }
    for card in &home.cards {
        card.artwork_url.hash(&mut hasher);
        card.motion_url.hash(&mut hasher);
        card.motion_enabled.hash(&mut hasher);
    }
    home.gif_autoplay_enabled.hash(&mut hasher);
    for row in &home.rows {
        row.title.hash(&mut hasher);
        row.kind.hash(&mut hasher);
        for card in &row.cards {
            card.artwork_url.hash(&mut hasher);
            card.motion_url.hash(&mut hasher);
            card.motion_enabled.hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn should_prefetch_home_artwork(context: &egui::Context, home: &HomeModel) -> bool {
    let signature = *home
        .artwork_signature
        .get_or_init(|| home_artwork_signature(home));
    context.data_mut(|data| {
        let id = Id::new("fluxa-home-artwork-prefetch");
        if data.get_temp::<u64>(id) == Some(signature) {
            false
        } else {
            data.insert_temp(id, signature);
            true
        }
    })
}

fn home_hero_height(viewport: Viewport) -> f32 {
    let base_height = if viewport.is_compact() {
        if viewport.width > viewport.height * 1.15 {
            (viewport.width * 5.0 / 12.0).min(440.0)
        } else {
            (viewport.width * 4.0 / 3.0).min(560.0)
        }
    } else {
        (viewport.height * 0.66 + 120.0).clamp(728.0, 984.0)
    };
    base_height * 0.93
}

/// The first shelf starts after the complete hero viewport, not after an
/// estimate of where the hero buttons happen to land. This keeps the hero
/// controls and the shelf heading in separate visual blocks on every size.
fn home_row_start(
    viewport: Viewport,
    metrics: UiMetrics,
    hero_height: f32,
    show_hero: bool,
) -> f32 {
    if !show_hero {
        return metrics.page_padding / 2.0;
    }
    hero_height
        + if viewport.is_compact() {
            metrics.control_gap.max(8.0)
        } else {
            metrics.section_gap.max(metrics.control_gap * 2.0)
        }
}

#[derive(Clone, Debug, Default)]
pub struct HomeLayout {
    pub focusable: Vec<(u64, Rect)>,
    pub activated: Option<u64>,
    pub text_input: Option<String>,
    pub text_input_node: Option<u64>,
    pub setting_change: Option<(String, serde_json::Value)>,
    pub filter_change: Option<(String, String)>,
    /// Core page requests generated when a shelf approaches its loaded end.
    pub load_more: Vec<serde_json::Value>,
    pub seek_to: Option<f64>,
    pub seek_hover: Option<f64>,
    pub profiles: Option<ProfilesRequest>,
}

pub const NODE_HOME: u64 = 10;
pub const NODE_LIBRARY: u64 = 11;
pub const NODE_DISCOVER: u64 = 12;
pub const NODE_CALENDAR: u64 = 13;
pub const NODE_PROFILE: u64 = 14;
pub const NODE_PLAY: u64 = 20;
pub const NODE_MORE_INFO: u64 = 21;
pub const NODE_LIBRARY_TAB_BASE: u64 = 30;
pub const NODE_LIBRARY_SEARCH: u64 = 80;
pub const NODE_PLAYER_CLOSE: u64 = 700;
pub const NODE_PLAYER_TOGGLE: u64 = 701;
pub const NODE_PLAYER_SEEK: u64 = 702;
pub const NODE_PLAYER_REWIND: u64 = 703;
pub const NODE_PLAYER_FORWARD: u64 = 704;
pub const NODE_PLAYER_MUTE: u64 = 705;
pub const NODE_PLAYER_FULLSCREEN: u64 = 706;
pub const NODE_PLAYER_UPSCALING: u64 = 707;
pub const NODE_PLAYER_RECOMMENDATIONS_CLOSE: u64 = 708;
pub const NODE_PLAYER_RECOMMENDATION_PLAY: u64 = 709;
pub const NODE_PLAYER_RECOMMENDATION_DETAILS: u64 = 710;
pub const NODE_PLAYER_CONTROLS: u64 = 711;
pub const NODE_PLAYER_HIDE_CONTROLS: u64 = 712;
pub const NODE_PLAYER_RECOMMENDATION_BASE: u64 = 720;
pub const PLAYER_RECOMMENDATION_LIMIT: usize = 12;
pub const NODE_LIBRARY_SORT: u64 = 81;
pub const NODE_LIBRARY_SOURCE: u64 = 82;
pub const NODE_DISCOVER_TYPE_BASE: u64 = 40;
pub const NODE_DISCOVER_CATALOG_BASE: u64 = 50;
pub const NODE_DISCOVER_EXTRA: u64 = 90;
pub const NODE_DISCOVER_SEARCH: u64 = 91;
pub const NODE_CALENDAR_PREV: u64 = 60;
pub const NODE_CALENDAR_NEXT: u64 = 61;
pub const NODE_CALENDAR_DAY_BASE: u64 = 200;
pub const NODE_CALENDAR_EVENT_BASE: u64 = 600;
pub const NODE_CALENDAR_CLOSE_DAY: u64 = 699;
pub const NODE_CARD_BASE: u64 = 100;
pub const NODE_DETAIL_BACK: u64 = 70;
pub const NODE_DETAIL_PLAY: u64 = 71;
pub const NODE_DETAIL_WATCHLIST: u64 = 72;
pub const NODE_DETAIL_COMPLETED: u64 = 73;
pub const NODE_DETAIL_DROPPED: u64 = 74;
pub const NODE_DETAIL_FAVORITE: u64 = 75;
pub const NODE_DETAIL_SIMILAR_BASE: u64 = 300;
pub const NODE_DETAIL_CAST_BASE: u64 = 1800;
pub const NODE_DETAIL_SEASON_BASE: u64 = 1900;
pub const NODE_DETAIL_EPISODE_BASE: u64 = 2000;
pub const NODE_SETTINGS_BACK: u64 = 400;
pub const NODE_SETTINGS_SWITCH_PROFILE: u64 = 401;
pub const NODE_SETTINGS_ACCOUNT_BASE: u64 = 402;
pub const NODE_SETTINGS_ROW_BASE: u64 = 800;
pub const NODE_SETTINGS_SECTION_BASE: u64 = 450;
pub const NODE_SETTINGS_ADDON_URL: u64 = 470;
pub const NODE_SETTINGS_ADDON_INSTALL: u64 = 471;
pub const NODE_SETTINGS_ADDON_REFRESH: u64 = 472;
pub const NODE_SETTINGS_POSTER_URL: u64 = 473;
pub const NODE_SETTINGS_POSTER_URL_SAVE: u64 = 474;
pub const NODE_SETTINGS_POSTER_URL_CLEAR: u64 = 475;
pub const NODE_SETTINGS_SEARCH: u64 = 476;
pub const NODE_SETTINGS_POSTER_TMDB_KEY: u64 = 482;
pub const NODE_SETTINGS_POSTER_TMDB_KEY_SAVE: u64 = 483;
pub const NODE_SETTINGS_POSTER_TMDB_KEY_CLEAR: u64 = 484;
pub const NODE_SETTINGS_POSTER_MDBLIST_KEY: u64 = 485;
pub const NODE_SETTINGS_POSTER_MDBLIST_KEY_SAVE: u64 = 486;
pub const NODE_SETTINGS_POSTER_MDBLIST_KEY_CLEAR: u64 = 487;
pub const NODE_SETTINGS_PLUGIN_URL: u64 = 480;
pub const NODE_SETTINGS_PLUGIN_INSTALL: u64 = 481;
pub const NODE_SETTINGS_PLUGIN_REPOSITORY_BASE: u64 = 490;
pub const NODE_SETTINGS_PLUGIN_REFRESH_BASE: u64 = 510;
pub const NODE_SETTINGS_PLUGIN_SCRAPER_BASE: u64 = 530;

#[derive(Clone, Debug, Default)]
pub struct DetailModel {
    pub item: serde_json::Value,
    pub id: String,
    pub content_type: String,
    pub is_loading: bool,
    pub is_loading_streams: bool,
    pub title: String,
    pub meta_line: String,
    pub description: String,
    pub poster_url: Option<String>,
    pub background_url: Option<String>,
    pub in_watchlist: bool,
    pub completed: bool,
    pub dropped: bool,
    pub favorite: bool,
    pub language: String,
    pub ratings: Vec<(String, String)>,
    pub genres: Vec<String>,
    pub trailers: Vec<String>,
    pub trailer: Option<egui::TextureId>,
    pub error: Option<String>,
    pub streams_error: Option<String>,
    pub similar: Vec<HomeCard>,
    pub logo_url: Option<String>,
    pub facts: Vec<String>,
    pub episodes: Vec<DetailEpisode>,
    pub cast: Vec<DetailCastMember>,
    pub resume: Option<HomeCard>,
}

#[derive(Clone, Debug, Default)]
pub struct DetailEpisode {
    pub id: String,
    pub season: i64,
    pub number: i64,
    pub title: String,
    pub overview: String,
    pub thumbnail: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct DetailCastMember {
    pub name: String,
    pub role: Option<String>,
    pub photo: Option<String>,
}

impl DetailModel {
    pub fn is_series(&self) -> bool {
        matches!(self.content_type.as_str(), "series" | "tv" | "show") || !self.episodes.is_empty()
    }

    pub fn seasons(&self) -> Vec<i64> {
        let mut seasons = self
            .episodes
            .iter()
            .map(|episode| episode.season)
            .collect::<Vec<_>>();
        seasons.sort_unstable_by_key(|season| if *season == 0 { i64::MAX } else { *season });
        seasons.dedup();
        seasons
    }
}

fn detail_episodes(meta: &serde_json::Value) -> Vec<DetailEpisode> {
    let mut episodes = meta
        .get("videos")
        .and_then(serde_json::Value::as_array)
        .map(|videos| {
            videos
                .iter()
                .filter_map(|video| {
                    let id = value_string(video, "id")?;
                    let number = |key: &str| video.get(key).and_then(serde_json::Value::as_i64);
                    let season = number("season").unwrap_or(1);
                    let episode = number("episode").or_else(|| number("number")).unwrap_or(0);
                    Some(DetailEpisode {
                        id,
                        season,
                        number: episode,
                        title: first_value_string(video, &["title", "name"]).unwrap_or_default(),
                        overview: first_value_string(video, &["overview", "description"])
                            .unwrap_or_default(),
                        thumbnail: first_value_string(video, &["thumbnail", "still", "image"]),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    episodes.sort_by_key(|episode| (episode.season, episode.number));
    episodes
}

fn detail_cast(meta: &serde_json::Value) -> Vec<DetailCastMember> {
    let mut cast = Vec::<DetailCastMember>::new();
    let sources = [
        meta.get("cast"),
        meta.pointer("/app_extras/cast"),
        meta.pointer("/appExtras/cast"),
    ];
    for item in sources
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_array)
        .flatten()
    {
        let member = match item {
            serde_json::Value::String(name) => DetailCastMember {
                name: name.trim().to_owned(),
                ..DetailCastMember::default()
            },
            _ => DetailCastMember {
                name: first_value_string(item, &["name", "fullName", "actor"]).unwrap_or_default(),
                role: first_value_string(item, &["character", "role", "as"]),
                photo: first_value_string(
                    item,
                    &["profilePath", "profile_path", "photo", "profile", "image", "img"],
                )
                .map(|photo| {
                    if photo.starts_with('/') {
                        format!("https://image.tmdb.org/t/p/w185{photo}")
                    } else {
                        photo
                    }
                }),
            },
        };
        if !member.name.is_empty()
            && !cast
                .iter()
                .any(|known| known.name.eq_ignore_ascii_case(&member.name))
        {
            cast.push(member);
        }
    }
    cast.truncate(20);
    cast
}

fn detail_facts(meta: &serde_json::Value, episodes: &[DetailEpisode], language: &str) -> Vec<String> {
    let mut facts = Vec::new();
    if let Some(release) = first_value_string(meta, &["releaseInfo", "year"]).or_else(|| {
        meta.get("year")
            .and_then(serde_json::Value::as_i64)
            .map(|year| year.to_string())
    }) {
        facts.push(release);
    }
    let mut seasons = episodes
        .iter()
        .map(|episode| episode.season)
        .filter(|season| *season > 0)
        .collect::<Vec<_>>();
    seasons.dedup();
    if !seasons.is_empty() {
        facts.push(format!("{} {}", seasons.len(), localized("auto.seasons", language)));
    }
    if let Some(runtime) = value_string(meta, "runtime") {
        facts.push(runtime);
    }
    facts
}

pub fn detail_model_from_core_snapshot(snapshot: &serde_json::Value) -> DetailModel {
    let detail = snapshot.get("detail").unwrap_or(&serde_json::Value::Null);
    let meta = detail.get("meta").unwrap_or(&serde_json::Value::Null);
    let id = value_string(detail, "id").unwrap_or_default();
    let library = snapshot.get("library").unwrap_or(&serde_json::Value::Null);
    let library_has_item = |key: &str| {
        library
            .get(key)
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    first_value_string(item, &["id", "contentId", "metaId"]).as_deref()
                        == Some(id.as_str())
                })
            })
    };
    let mut ratings = Vec::new();
    let mdblist = detail
        .get("mdblistRatings")
        .and_then(serde_json::Value::as_object);
    let rating_sources = [
        ("imdb", "IMDb", 1),
        ("tmdb", "TMDB", 0),
        ("trakt", "Trakt", 0),
        ("tomatoes", "Rotten Tomatoes", 0),
        ("popcorn", "Audience", 0),
        ("metacritic", "Metacritic", 2),
        ("metacriticuser", "Metacritic Users", 1),
        ("letterboxd", "Letterboxd", 1),
        ("myanimelist", "MyAnimeList", 1),
    ];
    if let Some(mdblist) = mdblist {
        for (key, label, precision) in rating_sources {
            if let Some(value) = mdblist.get(key).and_then(serde_json::Value::as_f64) {
                let value = match precision {
                    0 => format!("{}%", value.round() as i64),
                    1 => format!("{value:.1}"),
                    _ => format!("{}", value.round() as i64),
                };
                ratings.push((label.to_owned(), value));
            }
        }
    }
    if ratings.is_empty() {
        if let Some(value) = meta
            .get("imdbRating")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<f64>().ok())
        {
            ratings.push(("IMDb".to_owned(), format!("{value:.1}")));
        }
        if let Some(value) = detail
            .pointer("/omdbRatings/rottenTomatoes")
            .and_then(serde_json::Value::as_str)
        {
            ratings.push(("Rotten Tomatoes".to_owned(), value.to_owned()));
        }
        if let Some(value) = detail
            .pointer("/omdbRatings/metascore")
            .and_then(serde_json::Value::as_str)
        {
            ratings.push(("Metacritic".to_owned(), value.to_owned()));
        }
    }
    let episodes = detail_episodes(meta);
    DetailModel {
        item: meta.clone(),
        id: id.clone(),
        content_type: value_string(detail, "contentType").unwrap_or_else(|| "movie".to_owned()),
        language: snapshot_language(snapshot),
        is_loading: detail
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        is_loading_streams: detail
            .get("isLoadingStreams")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        title: first_value_string(meta, &["name", "title"])
            .unwrap_or_else(|| "Title details".to_owned()),
        meta_line: hero_meta_line(meta, &snapshot_language(snapshot)),
        description: first_value_string(meta, &["description", "overview", "plot"])
            .unwrap_or_else(|| "No description available.".to_owned()),
        poster_url: first_value_string(
            meta,
            &["poster", "posterUrl", "resolvedPosterUrl", "background"],
        ),
        background_url: first_value_string(
            meta,
            &[
                "backdrop",
                "backdropUrl",
                "background",
                "backgroundUrl",
                "fanart",
            ],
        ),
        in_watchlist: detail
            .get("isInWatchlist")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_else(|| library_has_item("watchlist")),
        completed: library_has_item("completed"),
        dropped: library_has_item("dropped"),
        favorite: library_has_item("liked"),
        ratings,
        genres: meta
            .get("genres")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        item.as_str()
                            .map(ToOwned::to_owned)
                            .or_else(|| first_value_string(item, &["name", "title"]))
                    })
                    .take(6)
                    .collect()
            })
            .unwrap_or_default(),
        trailers: trailer_urls(detail),
        trailer: None,
        error: detail
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        streams_error: detail
            .get("streamsError")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        similar: detail
            .get("similarItems")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().take(16).map(core_home_card).collect())
            .unwrap_or_default(),
        logo_url: detail
            .pointer("/fanartArtwork/hdLogo")
            .and_then(serde_json::Value::as_str)
            .filter(|url| !url.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| value_string(meta, "logo")),
        facts: detail_facts(meta, &episodes, &snapshot_language(snapshot)),
        cast: detail_cast(meta),
        resume: None,
        episodes,
    }
}

/// consume the same responsive rectangles as the painter.
pub fn home_layout(viewport: Viewport, home: &HomeModel) -> HomeLayout {
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let show_hero = home.show_hero_section && home.item_id.is_some();
    let metrics = home_metrics(viewport);
    let margin = if compact {
        metrics.page_padding
    } else if tv {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let mut layout = HomeLayout::default();
    if !compact {
        layout
            .focusable
            .extend(navigation_focus_rects(viewport, metrics));
    }
    let hero_height = home_hero_height(viewport);
    if show_hero {
        let play_y = hero_height - if compact { 92.0 } else { 194.0 } - home.scroll_offset;
        let size = Vec2::new(
            if compact {
                108.0
            } else {
                (metrics.horizontal_card_width * 0.46).max(160.0)
            },
            if compact { 42.0 } else { 50.0 },
        );
        let play = Rect::from_min_size(
            Pos2::new(
                if compact {
                    (viewport.width - size.x * 2.0 - 12.0) * 0.5
                } else {
                    margin
                },
                play_y,
            ),
            size,
        );
        layout.focusable.push((NODE_PLAY, play));
        layout
            .focusable
            .push((NODE_MORE_INFO, play.translate(Vec2::new(size.x + 12.0, 0.0))));
    }
    let row_start = home_row_start(viewport, metrics, hero_height, show_hero);
    let mut flat = 0;
    let mut row_y = row_start;
    for (row_index, (_, cards, kind)) in home
        .content_rows_with_kind()
        .into_iter()
        .enumerate()
        .take(64)
    {
        let (card_width, card_height, body_height) = home_row_dimensions(metrics, kind);
        let row_height = home_row_heading_height(metrics, viewport.is_tv()) + body_height;
        let visible_y = row_y - home.scroll_offset;
        if visible_y + row_height < 0.0 || visible_y > viewport.height {
            row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
            flat += cards.len();
            continue;
        }
        let row_scroll = home
            .row_scroll_offsets
            .get(row_index)
            .copied()
            .unwrap_or(0.0);
        let mut card_x = margin - row_scroll;
        for card in cards {
            let (item_width, item_height) = if kind == HomeRowKind::Collection {
                home_collection_card_dimensions(card)
            } else {
                (card_width, card_height)
            };
            layout.focusable.push((
                NODE_CARD_BASE + flat as u64,
                Rect::from_min_size(
                    Pos2::new(
                        card_x,
                        visible_y + home_row_heading_height(metrics, viewport.is_tv()),
                    ),
                    Vec2::new(item_width, item_height),
                ),
            ));
            card_x += item_width + metrics.horizontal_spacing;
            flat += 1;
        }
        row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
    }
    if compact {
        layout
            .focusable
            .extend(navigation_focus_rects(viewport, metrics));
    }
    layout
}

/// The shelf under the given touch point, if it is over a row's cards rather
/// than the hero or the space between shelves.
pub fn home_row_at_y(viewport: Viewport, home: &HomeModel, y: f32) -> Option<usize> {
    let metrics = home_metrics(viewport);
    let show_hero = home.show_hero_section && home.item_id.is_some();
    let hero_height = home_hero_height(viewport);
    let mut row_y = home_row_start(viewport, metrics, hero_height, show_hero);
    let content_y = y + home.scroll_offset;
    for (index, (_, cards, kind)) in home.content_rows_with_kind().into_iter().enumerate() {
        let body_height = home_row_body_height(metrics, cards, kind);
        let heading_height = home_row_heading_height(metrics, viewport.is_tv());
        if content_y >= row_y + heading_height && content_y <= row_y + heading_height + body_height
        {
            return Some(index);
        }
        row_y += heading_height + body_height + metrics.section_gap + metrics.vertical_spacing;
        if cards.is_empty() {
            continue;
        }
    }
    None
}

/// Maximum horizontal drag offset for a Home shelf.
pub fn home_row_scroll_max(viewport: Viewport, home: &HomeModel, row_index: usize) -> f32 {
    let Some((_, cards, kind)) = home.content_rows_with_kind().nth(row_index) else {
        return 0.0;
    };
    let metrics = home_metrics(viewport);
    home_row_scroll_max_for_cards(viewport, metrics, cards, kind)
}

fn home_row_scroll_max_for_cards(
    viewport: Viewport,
    metrics: UiMetrics,
    cards: &[HomeCard],
    kind: HomeRowKind,
) -> f32 {
    let (card_width, _, _) = home_row_dimensions(metrics, kind);
    let margin = if viewport.is_compact() {
        metrics.page_padding
    } else if viewport.is_tv() {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let content_width = if kind == HomeRowKind::Collection {
        cards
            .iter()
            .map(home_collection_card_dimensions)
            .map(|(width, _)| width + metrics.horizontal_spacing)
            .sum::<f32>()
            - metrics.horizontal_spacing
    } else {
        cards.len() as f32 * (card_width + metrics.horizontal_spacing) - metrics.horizontal_spacing
    };
    (content_width - (viewport.width - margin * 2.0)).max(0.0)
}

/// Maximum vertical offset for Home's touch/scroll adapter.
pub fn home_scroll_max(viewport: Viewport, home: &HomeModel) -> f32 {
    let compact = viewport.is_compact();
    let show_hero = home.show_hero_section && home.item_id.is_some();
    let metrics = home_metrics(viewport);
    let hero_height = home_hero_height(viewport);
    let mut content_height = home_row_start(viewport, metrics, hero_height, show_hero);
    for (_, cards, kind) in home.content_rows_with_kind().into_iter().take(64) {
        if cards.is_empty() {
            continue;
        }
        let row_height = home_row_total_height(metrics, kind, viewport.is_tv());
        content_height += row_height + metrics.section_gap + metrics.vertical_spacing;
    }
    let bottom_navigation_reserve = if compact {
        64.0 + viewport.safe_bottom
    } else {
        0.0
    };
    (content_height - viewport.height + bottom_navigation_reserve + metrics.page_padding).max(0.0)
}

pub fn draw_home(
    context: &egui::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    draw_home_with_options(context, viewport, home, assets, focused, true)
}

pub fn draw_home_content(
    context: &egui::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    draw_home_with_options(context, viewport, home, assets, focused, false)
}

/// Draws the navigation shared by every native screen. The active route is an
/// index into Home/Library/Discover/Calendar and the returned node id is
/// consumed by the platform event adapters.
pub fn draw_navigation_bar(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    assets: &impl HomeAssets,
) -> Option<u64> {
    draw_navigation_bar_with_profile(
        context,
        viewport,
        active_route,
        assets,
        assets.active_profile_avatar_url(),
        assets.active_profile_name().unwrap_or("Profile"),
    )
}

fn draw_navigation_bar_with_profile(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    assets: &impl HomeAssets,
    profile_avatar_url: Option<&str>,
    profile_name: &str,
) -> Option<u64> {
    let metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    if compact {
        return draw_mobile_navigation_bar(
            context,
            viewport,
            active_route,
            metrics,
            assets,
            profile_avatar_url,
        );
    }
    let label_size = metrics.navigation_label_size(tv);
    let icon_size = metrics.navigation_icon_size(tv);
    let profile_width = metrics.navigation_profile_width(profile_name, tv);
    let bar_width = metrics.navigation_bar_width(viewport, profile_name) + NAV_BAR_PADDING * 2.0;
    let bar_x = (viewport.width - bar_width).max(12.0) * 0.5;
    let font = FontId::proportional(label_size);
    let mut activated = None;
    egui::Area::new(Id::new("fluxa-shared-top-bar"))
        .fixed_pos(Pos2::new(bar_x, NAV_BAR_TOP))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            egui::Frame::NONE
                .fill(Color32::from_black_alpha(190))
                .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(22)))
                .corner_radius(NAV_ITEM_HEIGHT * 0.5 + NAV_BAR_PADDING)
                .inner_margin(egui::Margin::same(NAV_BAR_PADDING as i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = NAV_ITEM_GAP;
                        for (index, label) in ["Home", "Library", "Discover", "Calendar"]
                            .into_iter()
                            .enumerate()
                        {
                            let active = index == active_route;
                            let width = metrics.navigation_item_width(label, tv);
                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(width, NAV_ITEM_HEIGHT),
                                Sense::click(),
                            );
                            let fill = if active {
                                Color32::from_white_alpha(28)
                            } else if response.hovered() {
                                Color32::from_white_alpha(12)
                            } else {
                                Color32::TRANSPARENT
                            };
                            ui.painter().rect_filled(rect, NAV_ITEM_HEIGHT * 0.5, fill);
                            let color = if active || response.hovered() {
                                Color32::WHITE
                            } else {
                                Color32::from_white_alpha(160)
                            };
                            let galley = ui.painter().layout_no_wrap(label.to_owned(), font.clone(), color);
                            let content = icon_size + 8.0 + galley.size().x;
                            let left = rect.center().x - content * 0.5;
                            if let Some(icon_id) = assets.icon(label) {
                                ui.painter().image(
                                    icon_id,
                                    Rect::from_min_size(
                                        Pos2::new(left, rect.center().y - icon_size * 0.5),
                                        Vec2::splat(icon_size),
                                    ),
                                    full_uv(),
                                    color,
                                );
                            }
                            ui.painter().galley(
                                Pos2::new(
                                    left + icon_size + 8.0,
                                    rect.center().y - galley.size().y * 0.5,
                                ),
                                galley,
                                color,
                            );
                            if response.clicked() {
                                activated = Some(NODE_HOME + index as u64);
                            }
                        }
                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(profile_width, NAV_ITEM_HEIGHT),
                            Sense::click(),
                        );
                        let profile_active = active_route == 4;
                        if profile_active || response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                NAV_ITEM_HEIGHT * 0.5,
                                Color32::from_white_alpha(if profile_active { 28 } else { 12 }),
                            );
                        }
                        let center = rect.left_center() + Vec2::new(8.0 + NAV_AVATAR_RADIUS, 0.0);
                        if let Some(texture) = assets.cached_texture(profile_avatar_url) {
                            paint_circle_texture(ui.painter(), texture, center, NAV_AVATAR_RADIUS);
                        } else {
                            ui.painter().circle_filled(
                                center,
                                NAV_AVATAR_RADIUS,
                                Color32::from_white_alpha(30),
                            );
                            if let Some(icon) = assets.icon("Account") {
                                ui.painter().image(
                                    icon,
                                    Rect::from_center_size(center, Vec2::splat(16.0)),
                                    full_uv(),
                                    Color32::from_white_alpha(215),
                                );
                            }
                        }
                        let label_left = center.x + NAV_AVATAR_RADIUS + 8.0;
                        ui.painter().text(
                            Pos2::new(label_left, rect.center().y),
                            Align2::LEFT_CENTER,
                            truncate_to_width(
                                ui.painter(),
                                profile_name,
                                &font,
                                rect.right() - 12.0 - label_left,
                            ),
                            font.clone(),
                            if profile_active || response.hovered() {
                                Color32::WHITE
                            } else {
                                Color32::from_white_alpha(200)
                            },
                        );
                        if response.clicked() {
                            activated = Some(NODE_PROFILE);
                        }
                    });
                });
        });
    activated
}

/// Compact Android uses the same bottom navigation as Compose. The native
/// renderer used to draw the desktop top bar on every form factor, which made
/// the compact build look like a cropped desktop screen.
fn draw_mobile_navigation_bar(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    metrics: UiMetrics,
    assets: &impl HomeAssets,
    profile_avatar_url: Option<&str>,
) -> Option<u64> {
    let height = 64.0;
    let slot_width = ((viewport.width - 16.0) / 5.0).max(1.0);
    // Match Compose's safeDrawing bottom inset. In three-button mode this is
    // taller than the gesture inset, so do not guess a fixed 24dp here.
    let y = (viewport.height - viewport.safe_bottom - height).max(0.0);
    let mut activated = None;
    egui::Area::new(Id::new("fluxa-shared-bottom-bar"))
        .fixed_pos(Pos2::new(0.0, y))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            ui.set_width(viewport.width);
            ui.set_height(height);
            egui::Frame::NONE
                .fill(Color32::from_rgb(17, 17, 17))
                .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(18)))
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(viewport.width - 16.0, height - 8.0));
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for (index, (label, icon, node)) in [
                            ("Home", "Home", NODE_HOME),
                            ("Library", "Library", NODE_LIBRARY),
                            ("Discover", "Discover", NODE_DISCOVER),
                            ("Calendar", "Calendar", NODE_CALENDAR),
                            ("Profile", "Account", NODE_PROFILE),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            let active = index == active_route || (index == 4 && active_route == 4);
                            let response = ui.add_sized(
                                [slot_width, height - 16.0],
                                egui::Button::new(
                                    RichText::new("").size(metrics.nav_icon_size.max(25.0)),
                                )
                                .fill(Color32::TRANSPARENT)
                                .corner_radius(18.0),
                            );
                            let avatar_id = (index == 4)
                                .then(|| assets.cached_texture(profile_avatar_url))
                                .flatten();
                            if let Some(avatar_id) = avatar_id {
                                paint_circle_texture(
                                    ui.painter(),
                                    avatar_id,
                                    response.rect.center() - Vec2::new(0.0, 8.0),
                                    13.0,
                                );
                                ui.painter().circle_stroke(
                                    response.rect.center() - Vec2::new(0.0, 8.0),
                                    13.0,
                                    egui::Stroke::new(1.0, Color32::from_white_alpha(130)),
                                );
                            } else if let Some(icon_id) = assets.icon(icon) {
                                let icon_rect = Rect::from_center_size(
                                    response.rect.center() - Vec2::new(0.0, 8.0),
                                    Vec2::splat(25.0),
                                );
                                ui.painter().image(
                                    icon_id,
                                    icon_rect,
                                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                    if active {
                                        Color32::WHITE
                                    } else {
                                        Color32::from_white_alpha(155)
                                    },
                                );
                            }
                            let label = if index == 4 { "Profile" } else { label };
                            ui.painter().text(
                                Pos2::new(response.rect.center().x, response.rect.bottom() - 5.0),
                                Align2::CENTER_BOTTOM,
                                label,
                                FontId::proportional(9.0),
                                if active {
                                    Color32::WHITE
                                } else {
                                    Color32::from_white_alpha(155)
                                },
                            );
                            if response.clicked() {
                                activated = Some(node);
                            }
                        }
                    });
                });
        });
    activated
}

pub fn navigation_focus_rects(viewport: Viewport, metrics: UiMetrics) -> Vec<(u64, Rect)> {
    if viewport.is_compact() {
        let height = 64.0;
        let y = (viewport.height - viewport.safe_bottom - height).max(0.0);
        let slot = ((viewport.width - 16.0) / 5.0).max(1.0);
        [
            NODE_HOME,
            NODE_LIBRARY,
            NODE_DISCOVER,
            NODE_CALENDAR,
            NODE_PROFILE,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, node)| {
            (
                node,
                Rect::from_min_size(
                    Pos2::new(8.0 + slot * index as f32, y + 4.0),
                    Vec2::new(slot, height - 8.0),
                ),
            )
        })
        .collect()
    } else {
        let tv = viewport.is_tv();
        let widths = ["Home", "Library", "Discover", "Calendar"]
            .map(|label| metrics.navigation_item_width(label, tv));
        let bar_width = metrics.navigation_bar_width(viewport, "Profile") + NAV_BAR_PADDING * 2.0;
        let x = (viewport.width - bar_width).max(12.0) * 0.5 + NAV_BAR_PADDING;
        let y = NAV_BAR_TOP + NAV_BAR_PADDING;
        let mut left = x;
        let mut rects = Vec::with_capacity(5);
        for (index, node) in [NODE_HOME, NODE_LIBRARY, NODE_DISCOVER, NODE_CALENDAR]
            .into_iter()
            .enumerate()
        {
            let item_width = widths[index];
            rects.push((
                node,
                Rect::from_min_size(Pos2::new(left, y), Vec2::new(item_width, NAV_ITEM_HEIGHT)),
            ));
            left += item_width + NAV_ITEM_GAP;
        }
        let profile_width = metrics.navigation_profile_width("Profile", tv);
        rects.push((
            NODE_PROFILE,
            Rect::from_min_size(Pos2::new(left, y), Vec2::new(profile_width, NAV_ITEM_HEIGHT)),
        ));
        rects
    }
}

fn is_navigation_node(node: u64) -> bool {
    matches!(
        node,
        NODE_HOME | NODE_LIBRARY | NODE_DISCOVER | NODE_CALENDAR | NODE_PROFILE
    )
}

fn screen_margin(viewport: Viewport, metrics: UiMetrics) -> f32 {
    if viewport.is_compact() {
        metrics.page_padding
    } else if viewport.is_tv() {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    }
}

pub(crate) const LIBRARY_CARD_LIMIT: usize = 240;

fn discover_grid_geometry(
    available_width: f32,
    preferred_card_width: f32,
    gap: f32,
) -> (usize, f32) {
    let available_width = available_width.max(1.0);
    let gap = gap.max(0.0);
    let preferred_card_width = preferred_card_width.min(available_width).max(1.0);
    let columns = ((available_width + gap) / (preferred_card_width + gap))
        .floor()
        .max(1.0) as usize;
    let card_width =
        (available_width - gap * columns.saturating_sub(1) as f32 - 1.0) / columns as f32;
    (columns, card_width.max(1.0))
}

fn mobile_scroll_reserve(viewport: Viewport) -> f32 {
    if viewport.is_compact() {
        64.0 + viewport.safe_bottom
    } else {
        0.0
    }
}

pub(crate) struct PageLayout {
    pub margin: f32,
    pub width: f32,
    pub top: f32,
    pub title_height: f32,
    pub search: Rect,
    pub filters_top: f32,
    pub second_row_top: Option<f32>,
    pub content_top: f32,
}

impl PageLayout {
    pub fn new(viewport: Viewport, metrics: UiMetrics, second_row: bool) -> Self {
        let compact = viewport.is_compact();
        let margin = screen_margin(viewport, metrics);
        let width = (viewport.width - margin * 2.0).max(1.0);
        let top = if compact {
            metrics.content_header_top_mobile
        } else {
            metrics.content_header_top
        };
        let title_size = if compact {
            metrics.screen_title_size_mobile
        } else if viewport.is_tv() {
            metrics.screen_title_size_tv
        } else {
            metrics.screen_title_size
        };
        let control = metrics.screen_control_height;
        let search_height = control + 4.0;
        let (title_height, search, filters_top) = if compact {
            let title_height = title_size * 1.25;
            let search_top = top + title_height + metrics.control_gap;
            (
                title_height,
                Rect::from_min_size(
                    Pos2::new(margin, search_top),
                    Vec2::new(width, search_height),
                ),
                search_top + search_height + metrics.control_gap,
            )
        } else {
            let title_height = (title_size * 1.25).max(search_height);
            let search_width = (width * 0.36).clamp(220.0, 420.0);
            (
                title_height,
                Rect::from_min_size(
                    Pos2::new(
                        margin + width - search_width,
                        top + (title_height - search_height) * 0.5,
                    ),
                    Vec2::new(search_width, search_height),
                ),
                top + title_height + metrics.section_gap,
            )
        };
        let second_row_top = second_row.then_some(filters_top + control + metrics.control_gap);
        let content_top =
            second_row_top.unwrap_or(filters_top) + control + metrics.section_gap * 1.5;
        Self {
            margin,
            width,
            top,
            title_height,
            search,
            filters_top,
            second_row_top,
            content_top,
        }
    }
}

pub(crate) struct PosterGrid {
    pub columns: usize,
    pub card_width: f32,
    pub poster_height: f32,
    pub card_height: f32,
    pub gap: f32,
    pub row_gap: f32,
}

impl PosterGrid {
    pub fn new(width: f32, metrics: UiMetrics) -> Self {
        let gap = metrics.horizontal_spacing;
        let (columns, card_width) = discover_grid_geometry(width, metrics.poster_card_width, gap);
        let poster_height =
            metrics.poster_card_height * (card_width / metrics.poster_card_width.max(1.0));
        let card_height = poster_height
            + metrics.control_gap * 2.0
            + metrics.screen_card_title_size
            + metrics.screen_card_subtitle_size;
        Self {
            columns,
            card_width,
            poster_height,
            card_height,
            gap,
            row_gap: metrics.vertical_spacing * 1.4,
        }
    }

    pub fn height(&self, count: usize) -> f32 {
        let rows = count.div_ceil(self.columns);
        rows as f32 * self.card_height + rows.saturating_sub(1) as f32 * self.row_gap
    }

    pub fn cell(&self, origin: Pos2, index: usize) -> Rect {
        Rect::from_min_size(
            origin
                + Vec2::new(
                    (index % self.columns) as f32 * (self.card_width + self.gap),
                    (index / self.columns) as f32 * (self.card_height + self.row_gap),
                ),
            Vec2::new(self.card_width, self.card_height),
        )
    }
}

pub(crate) fn library_needs_second_row(viewport: Viewport, metrics: UiMetrics) -> bool {
    viewport.is_compact() || viewport.width - screen_margin(viewport, metrics) * 2.0 < 1240.0
}

pub fn library_scroll_max(viewport: Viewport, library: &LibraryModel, tab: LibraryTab) -> f32 {
    let metrics = UiMetrics::for_viewport(viewport);
    let page = PageLayout::new(
        viewport,
        metrics,
        library_needs_second_row(viewport, metrics),
    );
    let grid = PosterGrid::new(page.width, metrics);
    let content = grid.height(library.cards(tab).len().min(LIBRARY_CARD_LIMIT));
    (page.content_top + content + metrics.section_gap
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}

pub fn discover_scroll_max(viewport: Viewport, discover: &DiscoverModel) -> f32 {
    if discover.sections.is_empty() {
        return discover_scroll_max_for_result_count(viewport, discover.results.len());
    }
    let metrics = UiMetrics::for_viewport(viewport);
    let page = PageLayout::new(viewport, metrics, false);
    let grid = PosterGrid::new(page.width, metrics);
    let height = discover_blocks(&grid, metrics, discover)
        .last()
        .map_or(0.0, |block| block.top + block.height);
    (page.content_top + height + metrics.section_gap
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}

pub(crate) struct DiscoverBlock<'a> {
    pub title: Option<&'a str>,
    pub start: usize,
    pub len: usize,
    pub top: f32,
    pub header: f32,
    pub height: f32,
}

pub(crate) fn discover_blocks<'a>(
    grid: &PosterGrid,
    metrics: UiMetrics,
    discover: &'a DiscoverModel,
) -> Vec<DiscoverBlock<'a>> {
    if discover.sections.is_empty() {
        return vec![DiscoverBlock {
            title: None,
            start: 0,
            len: discover.results.len(),
            top: 0.0,
            header: 0.0,
            height: grid.height(discover.results.len()),
        }];
    }
    let header = metrics.screen_section_title_size + metrics.control_gap * 2.0;
    let mut top = 0.0;
    discover
        .sections
        .iter()
        .map(|section| {
            let height = header + grid.card_height;
            let block = DiscoverBlock {
                title: Some(section.title.as_str()),
                start: section.start,
                len: section.len,
                top,
                header,
                height,
            };
            top += height + metrics.section_gap;
            block
        })
        .collect()
}

pub fn discover_scroll_max_for_result_count(viewport: Viewport, result_count: usize) -> f32 {
    let metrics = UiMetrics::for_viewport(viewport);
    let page = PageLayout::new(viewport, metrics, false);
    let grid = PosterGrid::new(page.width, metrics);
    (page.content_top + grid.height(result_count) + metrics.section_gap
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}

pub fn calendar_scroll_max(viewport: Viewport, calendar: &CalendarModel) -> f32 {
    if calendar.year <= 0 || !(1..=12).contains(&calendar.month) {
        return 0.0;
    }
    let metrics = UiMetrics::for_viewport(viewport);
    let grid = calendar::CalendarGrid::new(viewport, metrics, calendar);
    let panel_height = calendar
        .selected_day
        .filter(|_| viewport.is_compact())
        .map(|day| {
            150.0
                + calendar.entries_for_day(day).count().min(10) as f32
                    * (metrics.horizontal_card_height * 0.34 + metrics.control_gap)
        })
        .unwrap_or(0.0);
    (grid.bottom() + panel_height - (viewport.height - mobile_scroll_reserve(viewport))).max(0.0)
}

pub fn settings_scroll_max(viewport: Viewport, settings: &SettingsModel) -> f32 {
    let metrics = UiMetrics::for_viewport(viewport);
    let desktop = !viewport.is_compact() && !viewport.is_tv();
    let top = if viewport.is_compact() {
        metrics.detail_header_top_mobile
    } else if desktop {
        metrics.settings_screen_padding_desktop
    } else {
        metrics.content_header_top
    };
    let section_index = settings.active_section.min(SETTINGS_SECTIONS.len() - 1);
    let card_height = settings_card_height(
        viewport,
        metrics,
        &SETTINGS_SECTIONS[section_index],
        settings,
    );
    let title_height = if desktop {
        0.0
    } else if viewport.is_compact() {
        metrics.screen_title_size_mobile
    } else if viewport.is_tv() {
        metrics.screen_title_size_tv
    } else {
        metrics.screen_title_size
    };
    let header_bottom = if desktop {
        top
    } else {
        top + title_height
            + metrics.control_gap
            + metrics.screen_body_size
            + metrics.section_gap
            + metrics.screen_control_height
    };
    let content_top = if viewport.is_compact() {
        header_bottom
            + metrics.section_gap
            + metrics.screen_control_height * 2.0
            + metrics.control_gap
            + metrics.section_gap
    } else {
        top
    };
    (content_top + card_height + metrics.page_padding
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}


fn resolve_screen_scroll(
    context: &egui::Context,
    viewport: Viewport,
    id: Id,
    max_offset: f32,
) -> f32 {
    if viewport.form_factor == UiFormFactor::Mobile {
        return viewport.scroll_y.clamp(0.0, max_offset);
    }
    // A dropdown menu owns wheel input while open. Without this guard both
    // its ScrollArea and the screen's custom document scroll consume the same
    // wheel delta, moving posters behind the popup while the options scroll.
    if egui::Popup::is_any_open(context) {
        return context.data_mut(|data| {
            data.get_temp::<f32>(id)
                .unwrap_or(viewport.scroll_y)
                .clamp(0.0, max_offset)
        });
    }
    let wheel_delta = context.input(|input| {
        // Mouse wheels report line-sized deltas while the Web renderer moves
        // by CSS pixels. Scale the native event once at the shared boundary.
        input.smooth_scroll_delta.y * 1.65
    });
    let offset = context.data_mut(|data| {
        let previous = data.get_temp::<f32>(id).unwrap_or(viewport.scroll_y);
        let offset = (previous - wheel_delta).clamp(0.0, max_offset);
        data.insert_temp(id, offset);
        offset
    });
    let offset = apply_desktop_drag_scroll(
        context,
        id.with("desktop-drag"),
        Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height)),
        DesktopScrollAxis::Vertical,
        offset,
        max_offset,
    );
    context.data_mut(|data| data.insert_temp(id, offset));
    offset
}

#[derive(Clone, Copy, Debug, Default)]
struct DesktopDragState {
    active: bool,
    axis: DesktopScrollAxis,
    last_pos: Pos2,
    velocity: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DesktopScrollAxis {
    #[default]
    Undecided,
    Vertical,
    Horizontal,
}

/// Applies the same direct-manipulation gesture users expect from a touch
/// surface to desktop-rendered content. The gesture deliberately lives in
/// the shared UI crate so every host can use the same scroll physics.
fn apply_desktop_drag_scroll(
    context: &egui::Context,
    id: Id,
    hit_rect: Rect,
    axis: DesktopScrollAxis,
    current: f32,
    max_offset: f32,
) -> f32 {
    if max_offset <= 0.0 {
        return 0.0;
    }

    let (pressed, released, down, interact_pos) = context.input(|input| {
        (
            input.pointer.primary_pressed(),
            input.pointer.primary_released(),
            input.pointer.primary_down(),
            input.pointer.interact_pos(),
        )
    });

    let mut state = context
        .data_mut(|data| data.get_temp::<DesktopDragState>(id))
        .unwrap_or_default();
    let mut offset = current.clamp(0.0, max_offset);

    if pressed {
        if let Some(pos) = interact_pos.filter(|pos| hit_rect.contains(*pos)) {
            state.active = true;
            state.axis = DesktopScrollAxis::Undecided;
            state.last_pos = pos;
            state.velocity = 0.0;
        }
    }

    if state.active && down {
        if let Some(pos) = interact_pos {
            let movement = pos - state.last_pos;
            state.last_pos = pos;
            if movement.length_sq() > 0.01 {
                if state.axis == DesktopScrollAxis::Undecided {
                    state.axis = if movement.y.abs() >= movement.x.abs() {
                        DesktopScrollAxis::Vertical
                    } else {
                        DesktopScrollAxis::Horizontal
                    };
                }
                if state.axis == axis {
                    let axis_delta = if axis == DesktopScrollAxis::Vertical {
                        movement.y
                    } else {
                        movement.x
                    };
                    offset = (offset - axis_delta).clamp(0.0, max_offset);
                    state.velocity = -axis_delta;
                    if (offset <= 0.0 && state.velocity < 0.0)
                        || (offset >= max_offset && state.velocity > 0.0)
                    {
                        state.velocity = 0.0;
                    }
                    context.request_repaint_after(Duration::from_millis(16));
                }
            }
        }
    } else if state.active && released {
        state.active = false;
        if state.velocity.abs() > 0.25 {
            context.request_repaint_after(Duration::from_millis(16));
        }
    } else if !state.active && state.axis == axis && state.velocity.abs() > 0.25 {
        offset = (offset + state.velocity).clamp(0.0, max_offset);
        if offset <= 0.0 || offset >= max_offset {
            state.velocity = 0.0;
        } else {
            state.velocity *= 0.90;
            context.request_repaint_after(Duration::from_millis(16));
        }
    } else if !down && !pressed {
        state.velocity = 0.0;
    }

    context.data_mut(|data| data.insert_temp(id, state));
    offset
}

fn resolve_desktop_horizontal_scroll(
    context: &egui::Context,
    id: Id,
    row_clip: Rect,
    current: f32,
    max_offset: f32,
) -> f32 {
    if max_offset <= 0.0 {
        return 0.0;
    }
    let wheel_delta = context.input(|input| {
        if input
            .pointer
            .hover_pos()
            .is_some_and(|pos| row_clip.contains(pos))
        {
            input.smooth_scroll_delta.x * 1.65
        } else {
            0.0
        }
    });
    let offset = context.data_mut(|data| {
        let previous = data.get_temp::<f32>(id).unwrap_or(current);
        let offset = (previous - wheel_delta).clamp(0.0, max_offset);
        data.insert_temp(id, offset);
        offset
    });
    let offset = apply_desktop_drag_scroll(
        context,
        id.with("desktop-drag"),
        row_clip,
        DesktopScrollAxis::Horizontal,
        offset,
        max_offset,
    );
    context.data_mut(|data| data.insert_temp(id, offset));
    offset
}

pub fn backdrop_target_size(width: f32, pixels_per_point: f32) -> [u32; 2] {
    artwork_target_size(Vec2::new(width, width * 9.0 / 16.0), pixels_per_point)
}

fn artwork_target_size(size: Vec2, pixels_per_point: f32) -> [u32; 2] {
    let scale = if pixels_per_point.is_finite() {
        pixels_per_point.max(0.25)
    } else {
        1.0
    };
    fluxa_artwork::bounded_target([
        (size.x.max(1.0) * scale).ceil() as u32,
        (size.y.max(1.0) * scale).ceil() as u32,
    ])
}

fn draw_home_with_options(
    context: &egui::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
    draw_top_bar: bool,
) -> HomeLayout {
    let mut metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    if !compact && !tv {
        metrics.home_continue_card_width *= 1.10;
        metrics.home_continue_card_height *= 1.10;
    }
    let scroll_offset = if viewport.form_factor == UiFormFactor::Desktop {
        resolve_screen_scroll(
            context,
            viewport,
            Id::new("fluxa-screen-scroll-home"),
            home_scroll_max(viewport, home),
        )
    } else {
        home.scroll_offset
    };
    let active_hero_index = home_hero_index(context, home);
    let fallback_hero = HomeHero {
        title: home.title.clone(),
        eyebrow: home.eyebrow.clone(),
        description: home.description.clone(),
        background_url: home.background_url.clone(),
        logo_url: home.logo_url.clone(),
        item_id: home.item_id.clone(),
        item_type: home.item_type.clone(),
        trailers: Vec::new(),
        raw: serde_json::Value::Null,
    };
    let mut hero = home
        .hero_slides
        .get(active_hero_index)
        .unwrap_or(&fallback_hero)
        .clone();
    let show_hero = home.show_hero_section && hero.item_id.is_some();
    let hero_height = home_hero_height(viewport);
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let hero_rect = Rect::from_min_size(
        Pos2::new(0.0, -scroll_offset),
        Vec2::new(viewport.width, hero_height),
    );
    // The content geometry ends at `hero_rect`, but the artwork must continue
    // underneath the gap before the first shelf. This is what gives the web
    // and Compose heroes their soft fade instead of a hard image-to-black
    // edge.
    let hero_fade_height = if compact { 96.0 } else { 176.0 };
    let hero_visual_rect = Rect::from_min_size(
        hero_rect.min,
        Vec2::new(viewport.width, hero_height + hero_fade_height),
    );
    let mut hero_slide_offset = 0.0;
    let mut hero_opacity = 1.0;
    let mut hero_image_rect = hero_visual_rect;
    let prefetch_home_artwork = should_prefetch_home_artwork(context, home);
    if prefetch_home_artwork {
        let full_target = backdrop_target_size(viewport.width, context.pixels_per_point());
        assets.prefetch_for(
            hero.background_url.as_deref(),
            full_target,
            ArtworkPriority::Prefetch,
        );
        for slide in &home.hero_slides {
            assets.prefetch_for(
                slide.background_url.as_deref(),
                full_target,
                ArtworkPriority::Prefetch,
            );
        }
    }
    if show_hero {
        let full_target = backdrop_target_size(viewport.width, context.pixels_per_point());
        let full_texture = assets.texture_for(
            hero.background_url.as_deref(),
            full_target,
            ArtworkPriority::Hero,
        );
        let cached_texture = full_texture.or_else(|| {
            // A different target may already be decoded (for example after a
            // resize). Reuse it while the exact hero-sized raster is being
            // prepared instead of flashing to a black rectangle.
            assets.cached_texture(hero.background_url.as_deref())
        });
        let last_ready_id = Id::new("fluxa-last-ready-hero-texture");
        let last_ready = context.data(|data| data.get_temp::<LastReadyHeroTexture>(last_ready_id));
        let (hero_texture, hero_size) = if let Some(texture) = cached_texture {
            let size = assets
                .texture_size(hero.background_url.as_deref())
                .unwrap_or(full_target);
            context.data_mut(|data| {
                data.insert_temp(
                    last_ready_id,
                    LastReadyHeroTexture {
                        texture,
                        size,
                        hero: hero.clone(),
                    },
                );
            });
            (Some(texture), size)
        } else if let Some(last_ready) = last_ready {
            // Keep the previous *complete* hero during a carousel transition
            // or an artwork decode gap. Pairing its texture with the new
            // slide's title/logo made backgrounded windows resume with mixed
            // content until the next image finished loading.
            hero = last_ready.hero;
            hero_slide_offset = 0.0;
            hero_image_rect = hero_visual_rect;
            (Some(last_ready.texture), last_ready.size)
        } else {
            (None, full_target)
        };
        let fade_id = Id::new("fluxa-home-hero-fade");
        let now = context.input(|input| input.time);
        let key = hero
            .item_id
            .clone()
            .or_else(|| hero.background_url.clone())
            .unwrap_or_else(|| hero.title.clone());
        let mut fade = context
            .data(|data| data.get_temp::<HeroFade>(fade_id))
            .unwrap_or_default();
        let current = hero_texture.map(|texture| (texture, hero_size));
        if fade.key != key {
            fade.from = if fade.key.is_empty() { None } else { fade.current };
            fade.key = key;
            fade.started = now;
        }
        fade.current = current;
        context.data_mut(|data| data.insert_temp(fade_id, fade.clone()));
        let linear = ((now - fade.started) / 0.8).clamp(0.0, 1.0) as f32;
        let t = 1.0 - (1.0 - linear).powi(3);
        if linear < 1.0 {
            context.request_repaint();
        }
        if fade.from.is_some() {
            hero_slide_offset = 28.0 * (1.0 - t);
            hero_opacity = t;
        }
        if let Some((texture, size)) = fade.from.filter(|_| linear < 1.0) {
            painter.image(
                texture,
                hero_visual_rect,
                cover_uv(size, hero_visual_rect),
                Color32::from_white_alpha(210),
            );
        }
        if let Some(hero_texture) = hero_texture {
            let zoom = if fade.from.is_some() { 1.0 + 0.035 * (1.0 - t) } else { 1.0 };
            hero_image_rect = Rect::from_center_size(
                hero_visual_rect.center(),
                hero_visual_rect.size() * zoom,
            );
            let uv = cover_uv(hero_size, hero_image_rect);
            let alpha = if fade.from.is_some() { t } else { 1.0 };
            painter.with_clip_rect(hero_visual_rect).image(
                hero_texture,
                hero_image_rect,
                uv,
                Color32::from_white_alpha((210.0 * alpha) as u8),
            );
        }
        if let Some(texture) = home
            .trailer
            .as_ref()
            .filter(|trailer| hero.item_id.as_deref() == Some(trailer.item_id.as_str()))
            .and_then(|trailer| trailer.texture)
        {
            paint_trailer(context, &painter, texture, hero_visual_rect, hero.item_id.as_deref(), 210);
        }
        paint_hero_scrim(&painter, hero_visual_rect, compact, metrics.background);
    }
    let margin = if compact {
        16.0
    } else if tv {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let mut activated = None;
    let mut hero_actions: Option<(Rect, Rect)> = None;
    if draw_top_bar {
        let profile_avatar_url = home
            .profile_avatar_url
            .clone()
            .or_else(|| assets.active_profile_avatar_url().map(ToOwned::to_owned));
        let profile_name = home
            .profile_name
            .clone()
            .or_else(|| assets.active_profile_name().map(ToOwned::to_owned))
            .unwrap_or_else(|| "Profile".to_owned());
        if let Some(avatar_url) = profile_avatar_url.as_deref() {
            let _ = assets.texture_for(Some(avatar_url), [96, 96], ArtworkPriority::Prefetch);
        }
        activated = draw_navigation_bar_with_profile(
            context,
            viewport,
            0,
            assets,
            profile_avatar_url.as_deref(),
            &profile_name,
        );
    }
    let has_home_content = show_hero || !home.cards.is_empty() || !home.rows.is_empty();
    if !has_home_content {
        let layout = HomeLayout {
            focusable: navigation_focus_rects(viewport, metrics),
            activated,
            text_input: None,
            text_input_node: None,
            setting_change: None,
            filter_change: None,
            load_more: Vec::new(),
            ..HomeLayout::default()
        };
        if home.is_loading {
            draw_loading_screen(context, &painter, screen, assets);
        } else {
            painter.text(
                screen.center(),
                Align2::CENTER_CENTER,
                "Home",
                FontId::proportional(metrics.screen_title_size_mobile),
                metrics.text_primary,
            );
        }
        if let Some((_, rect)) = layout
            .focusable
            .iter()
            .find(|(id, _)| Some(*id) == focused && !is_navigation_node(*id))
        {
            painter.rect_stroke(
                rect.expand(metrics.focus_ring_expand),
                metrics.focus_ring_radius,
                egui::Stroke::new(metrics.focus_ring_width, Color32::WHITE),
                egui::StrokeKind::Outside,
            );
        }
        return layout;
    }
    let hero_width = if compact {
        viewport.width - margin * 2.0
    } else {
        (viewport.width * 0.58).min(if tv {
            980.0
        } else {
            metrics.home_hero_content_max_width_desktop
        })
    };
    if show_hero {
        let bottom_padding = if compact { 44.0 } else { 48.0 };
        let has_logo = hero
            .logo_url
            .as_deref()
            .is_some_and(|url| !url.trim().is_empty());
        let title_height = if has_logo {
            if compact {
                72.0
            } else if tv {
                112.0
            } else {
                metrics.home_hero_logo_height_desktop
            }
        } else if compact {
            metrics.catalog_title_size * 2.25
        } else {
            metrics.catalog_title_size * 1.9
        };
        let metadata_height = if home.eyebrow.is_empty() {
            0.0
        } else if compact {
            18.0
        } else {
            22.0
        };
        let synopsis_size = if compact {
            metrics.card_title_size + 1.0
        } else if tv {
            metrics.card_title_size + 6.0
        } else {
            metrics.home_hero_synopsis_size_desktop
        };
        let synopsis_width = if compact {
            hero_width
        } else {
            (hero_width * 0.55).clamp(420.0, 720.0).min(hero_width)
        };
        let synopsis_color = Color32::from_white_alpha(240);
        let synopsis_galley = if hero.description.is_empty() {
            None
        } else {
            let mut job = egui::text::LayoutJob::simple(
                hero.description.clone(),
                egui::FontId::proportional(synopsis_size),
                synopsis_color,
                synopsis_width,
            );
            job.wrap.max_rows = if compact { 2 } else { 3 };
            job.wrap.overflow_character = Some('…');
            Some(context.fonts_mut(|fonts| fonts.layout_job(job)))
        };
        let synopsis_height = synopsis_galley
            .as_ref()
            .map_or(0.0, |galley| galley.size().y);
        let play_height = if compact { 42.0 } else { 44.0 };
        let synopsis_button_gap = if compact { 8.0 } else { 22.0 };
        let block_height = title_height
            + metadata_height
            + synopsis_height
            + play_height
            + if compact {
                26.0
            } else {
                20.0 + synopsis_button_gap
            };
        let content_top = (hero_height - bottom_padding - block_height).max(if compact {
            16.0
        } else {
            metrics.nav_vertical_padding * 2.0
        }) - scroll_offset
            + hero_slide_offset;
        egui::Area::new(Id::new("fluxa-shared-hero"))
            .fixed_pos(Pos2::new(margin, content_top))
            .show(context, |ui| {
                // The Compose hero owns a fixed 3:4/12:5 slide. Clip every
                // label/control to that slide so long metadata cannot paint
                // over the next shelf.
                ui.set_clip_rect(ui.clip_rect().intersect(hero_rect).intersect(screen));
                ui.multiply_opacity(hero_opacity);
                ui.set_min_width(hero_width);
                ui.set_max_width(hero_width);
                // Keep the widget column left-anchored and calculate every
                // hero element against the same rect. Relying on Label's
                // halign here is subtly wrong: a Label can allocate only its
                // galley width, so fallback titles and metadata end up with
                // a different x than an explicitly allocated logo.
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                    // Keep transparent title marks away from the clip edge.
                    // Some provider logos have visible pixels right at the
                    // source bitmap boundary.
                    let logo_inset = if compact { 6.0 } else { 8.0 };
                    let max_logo_size = Vec2::new(
                        if compact {
                            220.0
                        } else if tv {
                            380.0
                        } else {
                            metrics.home_hero_logo_max_width_desktop.min(460.0)
                        },
                        (title_height - logo_inset * 2.0).max(1.0),
                    );
                    if home.hero_slides.len() > 1 {
                        let next = &home.hero_slides[(active_hero_index + 1) % home.hero_slides.len()];
                        assets.texture_for(
                            next.logo_url.as_deref(),
                            artwork_target_size(max_logo_size, context.pixels_per_point()),
                            ArtworkPriority::Hero,
                        );
                    }
                    let title_font_size = if compact {
                        metrics.catalog_title_size + 2.0
                    } else if tv {
                        metrics.catalog_title_size * 2.0
                    } else {
                        metrics.catalog_title_size * 1.65
                    };
                    let title_rect = ui
                        .allocate_exact_size(Vec2::new(hero_width, title_height), Sense::hover())
                        .0;
                    let title_anchor = if compact {
                        egui::Align2::CENTER_TOP
                    } else {
                        egui::Align2::LEFT_TOP
                    };
                    if has_logo {
                        if let Some((texture, size)) = components::title_logo(
                            context.pixels_per_point(),
                            hero.logo_url.as_deref(),
                            max_logo_size,
                            ArtworkPriority::Hero,
                            assets,
                        ) {
                            let logo_x = if compact {
                                title_rect.center().x - size.x * 0.5
                            } else {
                                title_rect.left()
                            };
                            let logo_rect = Rect::from_min_size(
                                Pos2::new(logo_x, title_rect.center().y - size.y * 0.5),
                                size,
                            );
                            ui.painter()
                                .image(texture, logo_rect, full_uv(), Color32::WHITE);
                        }
                    } else {
                        let title = truncate_text(&hero.title, if compact { 52 } else { 72 });
                        ui.painter().text(
                            title_rect.min,
                            title_anchor,
                            title,
                            egui::FontId::proportional(title_font_size),
                            Color32::WHITE,
                        );
                    }
                    ui.add_space(if compact { 6.0 } else { 16.0 });
                    if !hero.eyebrow.is_empty() {
                        let metadata_rect = ui
                            .allocate_exact_size(
                                Vec2::new(hero_width, metadata_height),
                                Sense::hover(),
                            )
                            .0;
                        ui.painter().text(
                            if compact {
                                metadata_rect.center_top()
                            } else {
                                metadata_rect.left_top()
                            },
                            if compact {
                                egui::Align2::CENTER_TOP
                            } else {
                                egui::Align2::LEFT_TOP
                            },
                            truncate_text(&hero.eyebrow, if compact { 64 } else { 90 }),
                            egui::FontId::proportional(if tv {
                                metrics.nav_label_size + 4.0
                            } else {
                                metrics.nav_label_size
                            }),
                            Color32::from_white_alpha(185),
                        );
                    }
                    ui.add_space(if compact { 6.0 } else { 4.0 });
                    if let Some(galley) = synopsis_galley.as_ref() {
                        let synopsis_rect = ui
                            .allocate_exact_size(
                                Vec2::new(synopsis_width, synopsis_height),
                                Sense::hover(),
                            )
                            .0;
                        let synopsis_x = if compact {
                            synopsis_rect.center().x - galley.size().x * 0.5
                        } else {
                            synopsis_rect.left()
                        };
                        ui.painter().galley(
                            Pos2::new(synopsis_x, synopsis_rect.top()),
                            galley.clone(),
                            synopsis_color,
                        );
                    }
                    ui.add_space(if compact { 6.0 } else { synopsis_button_gap });
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        let resume = hero.item_id.as_deref().and_then(|id| home.resume_for(id));
                        let series = matches!(hero.item_type.as_deref(), Some("series" | "tv" | "show"));
                        let label = play_label(&home.language, resume, series.then_some((1, 1)));
                        let details_label = localized("home.view_details", &home.language);
                        let text_size = metrics.nav_label_size + 2.0;
                        let measure = |ui: &egui::Ui, text: &str| {
                            ui.painter()
                                .layout_no_wrap(text.to_owned(), egui::FontId::proportional(text_size), Color32::WHITE)
                                .size()
                                .x
                        };
                        let play_width = measure(ui, &label) + 70.0;
                        let details_width = measure(ui, &details_label) + metrics.control_gap * 2.0;
                        if compact {
                            ui.add_space(((ui.available_width() - play_width - details_width - 12.0) * 0.5).max(0.0));
                        }
                        let play = components::play_button(
                            ui,
                            assets,
                            &label,
                            Some(play_width),
                            play_height,
                            text_size,
                            resume.map(|card| card.progress).filter(|progress| *progress > 0.0),
                        );
                        let details = components::button(
                            ui,
                            &details_label,
                            details_width,
                            play_height,
                            components::ButtonKind::Secondary,
                            metrics,
                        );
                        hero_actions = Some((play.rect, details.rect));
                        if play.clicked() {
                            activated = Some(NODE_PLAY);
                        }
                        if details.clicked() {
                            activated = Some(NODE_MORE_INFO);
                        }
                    });
                });
            });
    }

    let row_start = home_row_start(viewport, metrics, hero_height, show_hero);
    let mut layout = HomeLayout::default();
    let mut flat_index = 0usize;
    let mut row_y = row_start;
    for (row_index, (title, cards, kind)) in home
        .content_rows_with_kind()
        .into_iter()
        .enumerate()
        .take(64)
    {
        let (card_width, card_height, _) = home_row_dimensions(metrics, kind);
        let body_height = home_row_body_height(metrics, cards, kind);
        let row_scroll_max = home_row_scroll_max_for_cards(viewport, metrics, cards, kind);
        let row_height = home_row_heading_height(metrics, tv) + body_height;
        let visible_y = row_y - scroll_offset;
        let row_is_visible = visible_y + row_height >= 0.0 && visible_y <= viewport.height;

        // Prefetch the complete home document, not just the visible row. This
        // is still lazy from the renderer's perspective: requests are async,
        // bounded by the host fetcher, decoded off-thread, and uploaded only
        // as completed images arrive. It prevents the web/Compose behaviour
        // difference where scrolling is required to start loading artwork.
        if prefetch_home_artwork {
            for card in cards {
                assets.prefetch_for(
                    card.poster_art(),
                    artwork_target_size(
                        Vec2::new(
                            if kind == HomeRowKind::Collection {
                                home_collection_card_dimensions(card).0
                            } else {
                                card_width
                            },
                            if kind == HomeRowKind::Collection {
                                home_collection_card_dimensions(card).1
                            } else {
                                card_height
                            },
                        ),
                        context.pixels_per_point(),
                    ),
                    ArtworkPriority::Prefetch,
                );
                if home.gif_autoplay_enabled
                    && kind == HomeRowKind::Collection
                    && card.motion_enabled
                    && !row_is_visible
                    && let Some(url) = card.motion_url.as_deref()
                {
                    let (width, height) = home_collection_card_dimensions(card);
                    assets.prefetch_animated_for(
                        Some(url),
                        fluxa_artwork::animation_target_size(artwork_target_size(
                            Vec2::new(width, height),
                            context.pixels_per_point(),
                        )),
                        ArtworkPriority::Prefetch,
                    );
                }
            }
        }
        if visible_y + row_height < 0.0 || visible_y > viewport.height {
            row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
            flat_index += cards.len();
            continue;
        }
        let heading_height = home_row_heading_height(metrics, tv);
        egui::Area::new(Id::new(format!("fluxa-shared-row-heading-{row_index}")))
            .fixed_pos(Pos2::new(margin, visible_y))
            // Catalog shelves are part of a scrolling document, not floating
            // dialogs. egui's default constraint pulls an off-screen shelf up
            // to fit the viewport, which makes it paint over the prior shelf.
            .constrain(false)
            .show(context, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(screen));
                ui.label(
                    RichText::new(title)
                        .size(if tv {
                            metrics.catalog_title_size
                        } else {
                            metrics.catalog_title_size - 4.0
                        })
                        .strong(),
                );
                ui.add_space(metrics.control_gap);
            });
        let row_clip = Rect::from_min_max(
            Pos2::new(margin, visible_y + heading_height),
            Pos2::new(
                (viewport.width - margin).max(margin),
                (visible_y + heading_height + body_height).min(viewport.height),
            ),
        );
        let row_scroll_offset = if viewport.form_factor == UiFormFactor::Desktop {
            resolve_desktop_horizontal_scroll(
                context,
                Id::new(("fluxa-home-row-scroll", row_index)),
                row_clip,
                home.row_scroll_offsets
                    .get(row_index)
                    .copied()
                    .unwrap_or(0.0),
                row_scroll_max,
            )
        } else {
            home.row_scroll_offsets
                .get(row_index)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, row_scroll_max)
        };
        if viewport.form_factor == UiFormFactor::Desktop
            && let Some(row) = home
                .rows
                .get(row_index.saturating_sub(usize::from(!home.cards.is_empty())))
            && row.can_load_more
            && !cards.is_empty()
            && let Some(row_id) = row.id.as_deref().filter(|value| !value.is_empty())
        {
            let max_offset = row_scroll_max;
            let card_pitch = (card_width + metrics.horizontal_spacing).max(1.0);
            let prefetch = (viewport.width * 2.0).max(card_pitch * 6.0);
            if row_scroll_offset >= (max_offset - prefetch).max(0.0) {
                let request_id = Id::new(("fluxa-home-load-more", row_id));
                let already_requested = context.data_mut(|data| {
                    let previous = data.get_temp::<usize>(request_id).unwrap_or(0);
                    if previous == cards.len() {
                        true
                    } else {
                        data.insert_temp(request_id, cards.len());
                        false
                    }
                });
                if !already_requested && let Some(category) = row.catalog_page.as_ref() {
                    let content_type = category
                        .get("contentType")
                        .or_else(|| category.get("type"))
                        .and_then(serde_json::Value::as_str)
                        .filter(|value| matches!(*value, "movie" | "series"))
                        .unwrap_or("movie");
                    layout.load_more.push(serde_json::json!({
                        "type": "catalogPageRequested",
                        "categoryId": row_id,
                        "transportUrl": category.get("addonTransportUrl").or_else(|| category.get("transportUrl")),
                        "contentType": content_type,
                        "catalogId": category.get("catalogId").or_else(|| category.get("id")),
                        "skip": category.get("skip").and_then(serde_json::Value::as_i64).unwrap_or(0) + cards.len() as i64,
                        "genre": category.get("addonGenre").or_else(|| category.get("genre")),
                        "remoteSource": category.get("remoteSources").or_else(|| category.get("remoteSource")),
                    }));
                }
            }
        }
        egui::Area::new(Id::new(format!("fluxa-shared-row-cards-{row_index}")))
            .fixed_pos(Pos2::new(
                margin - row_scroll_offset,
                visible_y + heading_height,
            ))
            .constrain(false)
            .show(context, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(row_clip));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = metrics.horizontal_spacing;
                    for (column_index, card) in cards.iter().enumerate() {
                        let is_poster = !matches!(kind, HomeRowKind::Continue);
                        let (item_width, item_height) = if kind == HomeRowKind::Collection {
                            home_collection_card_dimensions(card)
                        } else {
                            (card_width, card_height)
                        };
                        let slot_height = if is_poster { body_height } else { card_height };
                        let (widget_id, slot_rect) =
                            ui.allocate_space(Vec2::new(item_width, slot_height));
                        let rect =
                            Rect::from_min_size(slot_rect.min, Vec2::new(item_width, item_height));
                        let node_id = NODE_CARD_BASE + flat_index as u64;
                        layout.focusable.push((node_id, rect));
                        let card_visible = rect.intersects(screen) && rect.intersects(row_clip);
                        let response = card_visible
                            .then(|| ui.interact(slot_rect, widget_id, egui::Sense::click()));
                        if response.as_ref().is_some_and(|response| response.clicked()) {
                            activated = Some(node_id);
                        }
                        if card_visible {
                            if is_poster {
                                components::poster_card(
                                    ui.painter(),
                                    rect,
                                    card,
                                    column_index,
                                    viewport,
                                    metrics,
                                    assets,
                                    card.motion_enabled
                                        && card.motion_url.is_some()
                                        && ((home.gif_autoplay_enabled
                                            && screen.intersect(row_clip).contains_rect(rect))
                                            || focused == Some(node_id)
                                            || response
                                                .as_ref()
                                                .is_some_and(|response| response.hovered())),
                                );
                            } else {
                                components::continue_card(
                                    ui.painter(),
                                    rect,
                                    card,
                                    column_index,
                                    viewport,
                                    metrics,
                                    assets,
                                );
                            }
                        }
                        flat_index += 1;
                    }
                });
            });
        row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
    }
    layout.activated = activated;
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    if let Some((play, details)) = hero_actions {
        layout.focusable.push((NODE_PLAY, play));
        layout.focusable.push((NODE_MORE_INFO, details));
    } else if show_hero {
        let play_y = hero_height - if compact { 92.0 } else { 194.0 } - scroll_offset;
        layout.focusable.push((
            NODE_PLAY,
            Rect::from_min_size(
                Pos2::new(
                    if compact {
                        (viewport.width - 108.0) * 0.5
                    } else {
                        margin
                    },
                    play_y,
                ),
                Vec2::new(
                    if compact {
                        108.0
                    } else {
                        (metrics.horizontal_card_width * 0.46).max(160.0)
                    },
                    if compact { 42.0 } else { 50.0 },
                ),
            ),
        ));
    }
    if let Some((_, rect)) = layout
        .focusable
        .iter()
        .find(|(id, _)| Some(*id) == focused && !is_navigation_node(*id))
    {
        painter.rect_stroke(
            rect.expand(metrics.focus_ring_expand),
            metrics.focus_ring_radius,
            egui::Stroke::new(metrics.focus_ring_width, Color32::WHITE),
            egui::StrokeKind::Outside,
        );
    }
    layout
}

fn full_uv() -> Rect {
    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0))
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let mut output = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        output.pop();
        output.push('…');
    }
    output
}

fn today_iso() -> String {
    let days = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

fn truncate_to_width(painter: &egui::Painter, text: &str, font: &FontId, max_width: f32) -> String {
    if text.is_empty() || max_width <= 0.0 {
        return String::new();
    }
    let full_width = painter
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
        .size()
        .x;
    if full_width <= max_width {
        return text.to_owned();
    }
    let ellipsis = "…";
    let boundaries = text
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .collect::<Vec<_>>();
    let mut lower = 0usize;
    let mut upper = boundaries.len();
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        let prefix = &text[..boundaries[middle]];
        let candidate = format!("{prefix}{ellipsis}");
        let width = painter
            .layout_no_wrap(candidate, font.clone(), Color32::WHITE)
            .size()
            .x;
        if width <= max_width {
            lower = middle + 1;
        } else {
            upper = middle;
        }
    }
    let prefix_len = lower.saturating_sub(1);
    if prefix_len == 0 {
        ellipsis.to_owned()
    } else {
        format!("{}{ellipsis}", &text[..boundaries[prefix_len]])
    }
}

fn accent_from_value(value: &serde_json::Value) -> Option<Color32> {
    if let Some(text) = value.as_str() {
        return settings::parse_settings_color(text);
    }
    let argb = value.as_i64()? as u32;
    Some(Color32::from_rgb((argb >> 16) as u8, (argb >> 8) as u8, argb as u8))
}

pub(crate) fn paint_ambient(painter: &egui::Painter, screen: Rect, assets: &impl HomeAssets) {
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    painter.image(assets.background(), screen, uv, Color32::WHITE);
    let Some(accent) = assets.accent_color() else {
        return;
    };
    let [r, g, b, _] = accent.to_array();
    let chroma = r.max(g).max(b) - r.min(g).min(b);
    if chroma < 24 {
        return;
    }
    if let Some(glow) = assets.ambient_glow() {
        painter.image(glow, screen, uv, Color32::from_rgba_unmultiplied(r, g, b, 70));
    }
}

fn draw_loading_screen(
    context: &egui::Context,
    painter: &egui::Painter,
    screen: Rect,
    assets: &mut impl HomeAssets,
) {
    profiles::paint_custom_background(context, painter, screen, assets);
    let time = context.input(|input| input.time) as f32;
    let unit = (screen.width().min(screen.height()) / 900.0).clamp(0.7, 1.6);
    let breathe = 0.5 - 0.5 * (time * std::f32::consts::TAU / 1.9).cos();
    let alpha = (0.72 + 0.28 * breathe) * 255.0;
    let tint = Color32::from_white_alpha(alpha as u8);
    let mark_size = 64.0 * unit;
    let gap = 14.0 * unit;
    let galley = painter.layout_no_wrap(
        "fluxa".to_owned(),
        FontId::proportional(52.0 * unit),
        tint,
    );
    let brand_width = mark_size + gap + galley.size().x;
    let center = screen.center() - Vec2::new(0.0, 24.0 * unit);
    let left = center.x - brand_width * 0.5;
    if let Some(mark) = assets.brand_mark() {
        painter.image(
            mark,
            Rect::from_center_size(
                Pos2::new(left + mark_size * 0.5, center.y),
                Vec2::splat(mark_size),
            ),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            tint,
        );
    }
    let text_pos = Pos2::new(left + mark_size + gap, center.y - galley.size().y * 0.5);
    painter.galley(text_pos, galley, tint);
    let radius = 12.0 * unit;
    let spinner_center = center + Vec2::new(0.0, mark_size * 0.5 + 44.0 * unit);
    let stroke = 3.0 * unit;
    painter.circle_stroke(
        spinner_center,
        radius,
        egui::Stroke::new(stroke, Color32::from_white_alpha(50)),
    );
    let start = time * std::f32::consts::TAU / 0.8;
    let arc: Vec<Pos2> = (0..=24)
        .map(|step| {
            let angle = start + step as f32 / 24.0 * std::f32::consts::FRAC_PI_2;
            spinner_center + Vec2::angled(angle) * radius
        })
        .collect();
    painter.add(egui::Shape::line(
        arc,
        egui::Stroke::new(stroke, Color32::from_white_alpha(220)),
    ));
    context.request_repaint();
}

fn paint_hero_scrim(painter: &egui::Painter, rect: Rect, compact: bool, background: Color32) {
    if !compact {
        paint_horizontal_gradient(
            painter,
            Rect::from_min_max(
                rect.left_top(),
                Pos2::new(rect.left() + rect.width() * 0.72, rect.bottom()),
            ),
            Color32::from_black_alpha(235),
            Color32::TRANSPARENT,
        );
    }
    let fade_from = if compact { 0.3 } else { 0.5 };
    paint_vertical_gradient(
        painter,
        Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + rect.height() * fade_from),
            rect.right_bottom(),
        ),
        Color32::TRANSPARENT,
        background,
    );
}

fn paint_vertical_gradient(painter: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    paint_gradient(painter, rect, [top, top, bottom, bottom]);
}

fn paint_horizontal_gradient(painter: &egui::Painter, rect: Rect, left: Color32, right: Color32) {
    paint_gradient(painter, rect, [left, right, right, left]);
}

fn paint_gradient(painter: &egui::Painter, rect: Rect, colors: [Color32; 4]) {
    let mut mesh = egui::epaint::Mesh::default();
    for (position, color) in [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
    ]
    .into_iter()
    .zip(colors)
    {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: position,
            uv: Pos2::ZERO,
            color,
        });
    }
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);
    painter.add(egui::Shape::mesh(mesh));
}

fn paint_circle_texture(painter: &egui::Painter, texture: TextureId, center: Pos2, radius: f32) {
    let segments = 40u32;
    let mut mesh = egui::epaint::Mesh::with_texture(texture);
    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: Pos2::new(0.5, 0.5),
        color: Color32::WHITE,
    });
    for segment in 0..segments {
        let angle = std::f32::consts::TAU * segment as f32 / segments as f32;
        let (sin, cos) = angle.sin_cos();
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center + Vec2::new(cos, sin) * radius,
            uv: Pos2::new(0.5 + cos * 0.5, 0.5 + sin * 0.5),
            color: Color32::WHITE,
        });
    }
    for segment in 0..segments {
        let current = segment + 1;
        let next = ((segment + 1) % segments) + 1;
        mesh.indices.extend([0, current, next]);
    }
    painter.add(egui::Shape::mesh(mesh));
}

pub(crate) fn paint_trailer(
    context: &egui::Context,
    painter: &egui::Painter,
    texture: egui::TextureId,
    rect: Rect,
    key: Option<&str>,
    alpha: u8,
) {
    let fade = context.animate_bool_with_time(Id::new(("fluxa-trailer-fade", key, texture)), true, 0.6);
    painter.with_clip_rect(rect).image(
        texture,
        rect,
        cover_uv([1920, 1080], rect),
        Color32::from_white_alpha((f32::from(alpha) * fade) as u8),
    );
}

fn cover_uv(size: [u32; 2], destination: Rect) -> Rect {
    let image_aspect = size[0] as f32 / size[1].max(1) as f32;
    let destination_aspect = destination.width() / destination.height().max(1.0);
    if image_aspect > destination_aspect {
        let visible_width = destination_aspect / image_aspect;
        let crop = (1.0 - visible_width) * 0.5;
        Rect::from_min_max(Pos2::new(crop, 0.0), Pos2::new(1.0 - crop, 1.0))
    } else {
        let visible_height = image_aspect / destination_aspect.max(f32::EPSILON);
        let top = (1.0 - visible_height) * 0.5;
        Rect::from_min_max(Pos2::new(0.0, top), Pos2::new(1.0, top + visible_height))
    }
}

#[cfg(test)]
mod tests;

pub(crate) fn rating_logo(source: &str, score: &str) -> Option<&'static str> {
    let percent = score
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .ok();
    Some(match source {
        "IMDb" => "imdb",
        "TMDB" => "tmdb",
        "Trakt" => "trakt",
        "Letterboxd" => "letterboxd",
        "MyAnimeList" => "mal",
        "MDBList" => "mdblist",
        "Metacritic" | "Metacritic Users" => "metacritic",
        "Rotten Tomatoes" => match percent {
            Some(value) if value >= 60.0 => "rt-tomato-fresh",
            Some(_) => "rt-tomato-rotten",
            None => "rt-tomato-empty",
        },
        "Audience" => match percent {
            Some(value) if value < 60.0 => "rt-popcorn-spilled",
            _ => "rt-popcorn-full",
        },
        _ => return None,
    })
}
