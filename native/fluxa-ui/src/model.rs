use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiFormFactor {
    Mobile,
    #[default]
    Desktop,
    Tv,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiPlatform {
    Android,
    Ios,
    Tvos,
    Windows,
    Macos,
    #[default]
    Linux,
    Web,
    Webos,
}

impl UiPlatform {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "android" => Self::Android,
            "ios" => Self::Ios,
            "tvos" => Self::Tvos,
            "windows" => Self::Windows,
            "macos" => Self::Macos,
            "linux" => Self::Linux,
            "web" => Self::Web,
            "webos" => Self::Webos,
            _ => return None,
        })
    }

    pub fn is_apple(self) -> bool {
        matches!(self, Self::Ios | Self::Tvos | Self::Macos)
    }
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
    pub platform: UiPlatform,
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
            platform: UiPlatform::default(),
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

    pub fn with_platform(mut self, platform: UiPlatform) -> Self {
        self.platform = platform;
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
    pub platform: UiPlatform,
    #[serde(skip)]
    pub scroll_offset: f32,
    #[serde(skip)]
    pub row_scroll_offsets: Vec<f32>,
    #[serde(skip)]
    pub trailer: Option<HeroTrailer>,
    #[serde(skip)]
    pub language: String,
    #[serde(skip)]
    pub(crate) artwork_signature: OnceLock<u64>,
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
    pub(crate) fn default_empty() -> Self {
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
            platform: UiPlatform::default(),
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
    OnHold,
    Liked,
}

impl LibraryTab {
    pub const ALL: [Self; 6] = [
        Self::Watchlist,
        Self::Watching,
        Self::Completed,
        Self::Dropped,
        Self::OnHold,
        Self::Liked,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Watchlist => "Watchlist",
            Self::Watching => "Watching",
            Self::Completed => "Completed",
            Self::Dropped => "Dropped",
            Self::OnHold => "On Hold",
            Self::Liked => "Favorites",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::Watchlist => "library.watchlist",
            Self::Watching => "library.watching",
            Self::Completed => "library.completed",
            Self::Dropped => "library.dropped",
            Self::OnHold => "library.on_hold",
            Self::Liked => "library.favorites",
        }
    }

    pub fn core_tab_key(self) -> &'static str {
        match self {
            Self::Watchlist => "watchlist",
            Self::Watching => "watching",
            Self::Completed => "completed",
            Self::Dropped => "dropped",
            Self::OnHold => "hold",
            Self::Liked => "favorites",
        }
    }

    fn state_key(self) -> &'static str {
        match self {
            Self::Watchlist => "watchlist",
            Self::Watching => "continueWatching",
            Self::Completed => "completed",
            Self::Dropped => "dropped",
            Self::OnHold => "onHold",
            Self::Liked => "liked",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LibraryModel {
    pub language: String,
    pub query: String,
    pub sort_by: String,
    pub content_type: String,
    pub list_view: bool,
    pub downloads_open: bool,
    pub types: Vec<String>,
    pub statuses: Vec<String>,
    pub sorts: Vec<String>,
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
    pub on_hold: Vec<HomeCard>,
    pub liked: Vec<HomeCard>,
    pub personal: std::sync::Arc<PersonalIndex>,
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
            LibraryTab::OnHold => &self.on_hold,
            LibraryTab::Liked => &self.liked,
        }
    }

    pub fn apply_core_plan(&mut self, plan: &serde_json::Value, active_tab: LibraryTab) {
        self.view_cards = plan
            .get("items")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().take(64).map(core_home_card).collect())
            .unwrap_or_default();
        let strings = |key| -> Vec<String> {
            plan.get(key)
                .and_then(serde_json::Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                        .collect()
                })
                .unwrap_or_default()
        };
        self.types = strings("types");
        self.statuses = strings("statuses");
        self.sorts = strings("sorts");
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
    #[serde(rename = "typeLabel")]
    pub type_label: Option<String>,
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
    #[serde(skip)]
    pub poster_shape: Option<String>,
}

impl HomeCard {
    pub(crate) fn is_landscape(&self) -> bool {
        poster_overlay::landscape() && self.row_kind != HomeRowKind::Collection
    }

    pub(crate) fn poster_art(&self) -> Option<&str> {
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
        let language = model.language.clone();
        let show_catalog_type = snapshot
            .pointer("/settings/values/showCatalogType")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
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
                    type_label: show_catalog_type
                        .then(|| category.get("type"))
                        .flatten()
                        .and_then(serde_json::Value::as_str)
                        .and_then(|kind| match kind {
                            "movie" => Some("settings.addon_type.movie"),
                            "series" => Some("settings.addon_type.series"),
                            _ => None,
                        })
                        .map(|key| crate::localized(key, &language)),
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

pub(crate) fn core_home_hero(item: &serde_json::Value, language: &str) -> HomeHero {
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

pub(crate) fn trailer_urls(item: &serde_json::Value) -> Vec<String> {
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
pub(crate) struct HeroCarouselState {
    index: usize,
    generation: u64,
    next_at: f64,
}

pub(crate) fn home_hero_index(context: &egui::Context, home: &HomeModel) -> usize {
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
        home.hero_slides
            .get(state.index % count)
            .and_then(|slide| slide.item_id.as_deref())
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

pub(crate) fn normalize_home_row_title(title: &str) -> String {
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
        content_type: "all".to_owned(),
        list_view: false,
        downloads_open: false,
        types: Vec::new(),
        statuses: Vec::new(),
        sorts: Vec::new(),
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
        on_hold: cards(LibraryTab::OnHold),
        liked: cards(LibraryTab::Liked),
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

pub(crate) fn discover_catalog_model(snapshot: &serde_json::Value) -> DiscoverModel {
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

pub fn play_label(
    language: &str,
    resume: Option<&HomeCard>,
    episode: Option<(i64, i64, Option<&str>)>,
) -> String {
    let action = localized(
        if resume.is_some() {
            "common.continue"
        } else {
            "common.play"
        },
        language,
    );
    let resumed = resume.and_then(|card| {
        let (season, number) = resume_episode(card)?;
        Some((
            season,
            number,
            card.raw.get("lastEpisodeName").and_then(|v| v.as_str()),
        ))
    });
    let Some((season, number, name)) = resumed.or(episode) else {
        return action;
    };
    let label = localized("format.play_episode", language)
        .replacen("%s", &action, 1)
        .replacen("%s", &season.to_string(), 1)
        .replacen("%s", &number.to_string(), 1);
    match name.map(str::trim).filter(|name| !name.is_empty()) {
        Some(name) => format!("{label}: {name}"),
        None => label,
    }
}

pub fn hero_from_meta(item: &serde_json::Value, language: &str) -> HomeHero {
    core_home_hero(item, language)
}

pub(crate) fn core_home_card(item: &serde_json::Value) -> HomeCard {
    core_home_card_for_kind(item, HomeRowKind::Poster)
}

pub(crate) fn core_home_card_for_kind(item: &serde_json::Value, kind: HomeRowKind) -> HomeCard {
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
        poster_shape: value_string(item, "posterShape"),
        raw: item.clone(),
        row_kind: kind,
    }
}

pub(crate) fn value_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
}

pub(crate) fn value_display(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(|raw| match raw {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    })
}

pub(crate) fn first_value_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| value_string(value, key))
}

pub(crate) fn hero_meta_line(value: &serde_json::Value, language: &str) -> String {
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
    pub row_scroll_offsets: [f32; 4],
    pub selected_season: Option<i64>,
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

pub(crate) fn detail_episodes(meta: &serde_json::Value) -> Vec<DetailEpisode> {
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

pub(crate) fn detail_cast(meta: &serde_json::Value) -> Vec<DetailCastMember> {
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
                    &[
                        "profilePath",
                        "profile_path",
                        "photo",
                        "profile",
                        "image",
                        "img",
                    ],
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

pub(crate) fn detail_facts(
    meta: &serde_json::Value,
    episodes: &[DetailEpisode],
    language: &str,
) -> Vec<String> {
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
        facts.push(format!(
            "{} {}",
            seasons.len(),
            localized("auto.seasons", language)
        ));
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
        row_scroll_offsets: [0.0; 4],
        selected_season: None,
    }
}
