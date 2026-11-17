use super::*;

mod calendar;
mod cards;
mod detail;
mod discover;
mod downloads;
mod folder;
mod home_snapshot;
mod library;
#[derive(Clone, Debug)]
pub struct HoverTip {
    pub rect: Rect,
    pub text: String,
}

pub use calendar::*;
pub(crate) use cards::*;
pub use detail::*;
pub use discover::*;
pub use downloads::*;
pub use folder::*;
pub use home_snapshot::*;
pub use library::*;

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
    pub safe_top: f32,
    pub safe_left: f32,
    pub safe_right: f32,
    /// Vertical content offset supplied by the platform input adapter.
    /// Headers and navigation remain fixed while the screen body scrolls.
    pub scroll_y: f32,
    pub nav_lead: f32,
    pub side_inset: f32,
}

impl Viewport {
    pub fn new(width: u32, height: u32, form_factor: UiFormFactor) -> Self {
        let inset = crate::sidebar::side_inset(width as f32, form_factor);
        Self {
            width: width as f32 - inset,
            side_inset: inset,
            height: height as f32,
            form_factor,
            platform: UiPlatform::default(),
            safe_bottom: Self::min_safe(form_factor).bottom,
            safe_top: Self::min_safe(form_factor).top,
            safe_left: Self::min_safe(form_factor).left,
            safe_right: Self::min_safe(form_factor).right,
            scroll_y: 0.0,
            nav_lead: 0.0,
        }
    }

    pub fn with_nav_lead(mut self, lead: f32) -> Self {
        self.nav_lead = lead.max(0.0);
        self
    }

    fn min_safe(form_factor: UiFormFactor) -> crate::flex::Insets {
        if form_factor == UiFormFactor::Tv {
            crate::flex::Insets::xy(48.0, 27.0)
        } else {
            crate::flex::Insets::default()
        }
    }

    pub fn with_safe_area(mut self, top: f32, left: f32, right: f32) -> Self {
        let floor = Self::min_safe(self.form_factor);
        let clean = |value: f32, floor: f32| {
            if value.is_finite() {
                value.max(floor)
            } else {
                floor
            }
        };
        self.safe_top = clean(top, floor.top);
        self.safe_left = clean(left, floor.left);
        self.safe_right = clean(right, floor.right);
        self
    }

    pub(crate) fn safe_area(&self) -> crate::flex::Insets {
        crate::flex::Insets {
            left: self.safe_left,
            top: self.safe_top,
            right: self.safe_right,
            bottom: self.safe_bottom,
        }
    }

    pub fn with_safe_bottom(mut self, inset: f32) -> Self {
        let floor = Self::min_safe(self.form_factor).bottom;
        self.safe_bottom = if inset.is_finite() {
            inset.max(floor)
        } else {
            floor
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
        if self.is_tv() {
            SizeClass::Expanded
        } else if self.width < 600.0 {
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

    pub fn is_phone(self) -> bool {
        self.form_factor == UiFormFactor::Mobile && self.is_compact()
    }

    pub fn is_tablet(self) -> bool {
        self.form_factor == UiFormFactor::Mobile && !self.is_compact()
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
    #[serde(rename = "pendingFeeds")]
    pub pending_feeds: u32,
    #[serde(rename = "continuePending")]
    pub continue_pending: bool,
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
    #[serde(skip)]
    pub playback_unavailable: Option<String>,
    #[serde(skip)]
    pub notice: Option<(String, f32)>,
    #[serde(skip)]
    pub hover_tip: Option<HoverTip>,
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
    pub cards: Vec<HomeCard>,
    pub rows: Vec<HomeRow>,
    #[serde(rename = "formFactor")]
    pub form_factor: UiFormFactorJson,
    #[serde(skip)]
    pub platform: UiPlatform,
    #[serde(skip)]
    pub scroll_offset: f32,
    #[serde(skip)]
    pub top_inset: f32,
    #[serde(skip)]
    pub hero_poster: bool,
    #[serde(skip)]
    pub category_bar: bool,
    #[serde(skip)]
    pub categories: Vec<crate::HomeCategory>,
    #[serde(skip)]
    pub active_category: crate::HomeCategory,
    #[serde(skip)]
    pub row_scroll_offsets: Vec<f32>,
    #[serde(skip)]
    pub trailer: Option<HeroTrailer>,
    #[serde(skip)]
    pub poster_trailer: Option<HeroTrailer>,
    #[serde(skip)]
    pub follow_focus_hero: bool,
    #[serde(skip)]
    pub focus_driven: bool,
    #[serde(skip)]
    pub expanded_posters: bool,
    #[serde(skip)]
    pub expanded_posters_delay: f64,
    #[serde(skip)]
    pub poster_hover_preview: bool,
    #[serde(skip)]
    pub hover_preview_delay: f64,
    #[serde(skip)]
    pub poster_lift: bool,
    #[serde(skip)]
    pub expanded_node: Option<u64>,
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
    #[serde(skip)]
    pub badges: Vec<crate::SourceBadge>,
}

#[derive(Clone, Debug)]
pub struct HeroTrailer {
    pub item_id: String,
    pub texture: Option<crate::gfx::TextureId>,
    pub finished: bool,
    pub subtitle: Option<String>,
}

impl Default for HomeModel {
    fn default() -> Self {
        Self {
            is_loading: false,
            pending_feeds: 0,
            continue_pending: false,
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
            pending_feeds: 0,
            continue_pending: false,
            title: String::new(),
            eyebrow: String::new(),
            description: String::new(),
            background_url: None,
            logo_url: None,
            profile_avatar_url: None,
            profile_name: None,
            playback_unavailable: None,
            notice: None,
            hover_tip: None,
            item_id: None,
            item_type: None,
            hero_slides: Vec::new(),
            show_hero_section: true,
            gif_autoplay_enabled: true,
            cards: Vec::new(),
            rows: Vec::new(),
            form_factor: UiFormFactorJson::Desktop,
            platform: UiPlatform::default(),
            scroll_offset: 0.0,
            top_inset: 0.0,
            hero_poster: false,
            category_bar: false,
            categories: Vec::new(),
            active_category: crate::HomeCategory::Home,
            row_scroll_offsets: Vec::new(),
            trailer: None,
            poster_trailer: None,
            follow_focus_hero: false,
            focus_driven: false,
            expanded_posters: false,
            expanded_posters_delay: 1.0,
            poster_hover_preview: true,
            hover_preview_delay: 0.5,
            poster_lift: false,
            expanded_node: None,
            language: "en".to_owned(),
            artwork_signature: OnceLock::new(),
        }
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
    pub catalog_page: Option<std::sync::Arc<serde_json::Value>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HomeRowKind {
    Continue,
    #[default]
    Poster,
    Collection,
}

#[derive(Clone, Debug, Default)]
pub enum CardRaw {
    #[default]
    Empty,
    Owned(std::sync::Arc<serde_json::Value>),
    Item {
        row: std::sync::Arc<serde_json::Value>,
        index: usize,
    },
}

impl CardRaw {
    pub fn item(row: &std::sync::Arc<serde_json::Value>, index: usize) -> Self {
        Self::Item {
            row: std::sync::Arc::clone(row),
            index,
        }
    }

    pub fn cloned(&self) -> serde_json::Value {
        (**self).clone()
    }

    pub fn to_mut(&mut self) -> &mut serde_json::Value {
        if !matches!(self, Self::Owned(_)) {
            *self = Self::Owned(std::sync::Arc::new((**self).clone()));
        }
        match self {
            Self::Owned(value) => std::sync::Arc::make_mut(value),
            _ => unreachable!(),
        }
    }
}

impl Serialize for CardRaw {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (**self).serialize(serializer)
    }
}

impl From<serde_json::Value> for CardRaw {
    fn from(value: serde_json::Value) -> Self {
        Self::Owned(std::sync::Arc::new(value))
    }
}

impl std::ops::Deref for CardRaw {
    type Target = serde_json::Value;

    fn deref(&self) -> &serde_json::Value {
        static NULL: serde_json::Value = serde_json::Value::Null;
        match self {
            Self::Empty => &NULL,
            Self::Owned(value) => value,
            Self::Item { row, index } => row
                .get("items")
                .and_then(|items| items.get(*index))
                .unwrap_or(&NULL),
        }
    }
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
    #[serde(skip_deserializing, rename = "collectionShape")]
    pub collection_shape: Option<String>,
    #[serde(skip_deserializing, rename = "hideTitle")]
    pub hide_title: bool,
    #[serde(skip_deserializing, rename = "motionUrl")]
    pub motion_url: Option<String>,
    #[serde(skip_deserializing, rename = "motionEnabled")]
    pub motion_enabled: bool,
    /// Original Core item. Continue Watching cards need the resume fields
    /// (`lastVideoId`, episode numbers, offsets, etc.) when they are
    /// activated; reducing them to display-only fields loses the playback
    /// identity and incorrectly sends the user to an empty detail screen.
    #[serde(skip)]
    pub raw: CardRaw,
    #[serde(skip)]
    pub row_kind: HomeRowKind,
    #[serde(skip)]
    pub overlay: poster_overlay::PosterFacts,
    #[serde(skip)]
    pub logo_url: Option<String>,
    #[serde(skip_deserializing, rename = "backdropUrl")]
    pub backdrop_url: Option<String>,
    #[serde(skip_deserializing)]
    pub meta: String,
    #[serde(skip_deserializing)]
    pub synopsis: String,
    #[serde(skip_deserializing)]
    pub trailers: Vec<String>,
    #[serde(skip)]
    pub poster_shape: Option<String>,
    #[serde(skip_deserializing)]
    pub detail: String,
    #[serde(skip_deserializing)]
    pub episodes: String,
    #[serde(skip)]
    pub upcoming: bool,
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

pub(crate) fn first_episode(meta: &serde_json::Value) -> Option<(i64, i64, String)> {
    let episodes = detail::detail_episodes(meta, &serde_json::Value::Null);
    let first = episodes
        .iter()
        .find(|episode| episode.season > 0)
        .or(episodes.first())?;
    Some((first.season, first.number, first.title.clone()))
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

pub fn library_data(library: &LibraryModel, tab: LibraryTab) -> serde_json::Value {
    let tabs: Vec<_> = LibraryTab::ALL
        .into_iter()
        .map(|candidate| {
            serde_json::json!({
                "id": candidate.core_tab_key(),
                "key": "libraryStatus",
                "label": localized(candidate.translation_key(), &library.language),
                "selected": candidate == tab,
            })
        })
        .collect();
    let language = library.language.as_str();
    let statuses: Vec<_> = LibraryTab::ALL
        .into_iter()
        .filter(|candidate| {
            library
                .statuses
                .iter()
                .any(|status| status == candidate.core_tab_key())
        })
        .map(|candidate| {
            serde_json::json!({
                "value": candidate.core_tab_key(),
                "label": localized(candidate.translation_key(), language),
            })
        })
        .collect();
    let mut types =
        vec![serde_json::json!({"value": "all", "label": localized("auto.all", language)})];
    for (key, label) in [
        ("movie", "auto.movies"),
        ("series", "auto.series"),
        ("anime", "auto.anime"),
    ] {
        if library.types.iter().any(|kind| kind == key) {
            types.push(serde_json::json!({"value": key, "label": localized(label, language)}));
        }
    }
    let sorts: Vec<_> = [
        "tracker",
        "recent",
        "oldest",
        "title",
        "title_desc",
        "rating",
    ]
    .into_iter()
    .filter(|sort| library.sorts.is_empty() || library.sorts.iter().any(|item| item == sort))
    .map(|sort| {
        serde_json::json!({
            "value": sort,
            "label": localized(&format!("library.sort_{sort}"), language),
        })
    })
    .collect();
    let sort = if sorts
        .iter()
        .any(|item| item["value"] == library.sort_by.as_str())
    {
        library.sort_by.clone()
    } else {
        sorts
            .first()
            .and_then(|item| item["value"].as_str())
            .unwrap_or_default()
            .to_owned()
    };
    let kind = if library.content_type.is_empty() {
        "all".to_owned()
    } else {
        library.content_type.clone()
    };
    serde_json::json!({
        "downloadsOpen": library.downloads_open,
        "isEmpty": !library.downloads_open && library.cards(tab).is_empty(),
        "downloads": downloads_data(library),
        "count": library.cards(tab).len(),
        "statusOptions": statuses,
        "typeOptions": types,
        "sortOptions": sorts,
        "type": kind,
        "sort": sort,
        "query": library.query,
        "loading": library.is_loading,
        "error": library.error,
        "listView": library.list_view,
        "tab": tab.core_tab_key(),
        "tabs": tabs,
        "cards": library.cards(tab),
    })
}

pub fn downloads_data(library: &LibraryModel) -> serde_json::Value {
    let language = library.language.as_str();
    let row_json = |row: &DownloadRow| {
        let icon = match row.status {
            DownloadStatus::Downloading | DownloadStatus::Queued => "PauseFilled",
            DownloadStatus::Paused => "PlayFilled",
            _ => "Refresh",
        };
        serde_json::json!({
            "id": row.id,
            "title": row.title,
            "status": crate::components::status_label(row, language),
            "bytes": crate::components::bytes_label(row, language),
            "timing": crate::components::timing_label(row, language),
            "percent": row.percent / 100.0,
            "icon": icon,
            "thumbnail": row.thumbnail,
            "season": row.season,
            "episode": row.episode,
        })
    };
    let groups: Vec<_> = library
        .downloads
        .iter()
        .map(|group| {
            let line = localized("downloads.storage_summary", language)
                .replacen("%s", &group.rows.len().to_string(), 1)
                .replacen("%s", &format_bytes(group.bytes_done()), 1);
            serde_json::json!({
                "id": group.id,
                "title": group.title,
                "poster": group.poster,
                "line": line,
                "progress": group.active_fraction().unwrap_or(0.0),
                "active": group.active() > 0,
            })
        })
        .collect();
    let open = library.open_download_group();
    let summary = &library.downloads_summary;
    serde_json::json!({
        "groups": groups,
        "open": open.is_some(),
        "listing": library.downloads_open && open.is_none(),
        "none": library.downloads_open && library.downloads.is_empty(),
        "openTitle": open.map(|(_, group)| group.title.clone()),
        "rows": open
            .map(|(_, group)| group.rows.iter().map(row_json).collect::<Vec<_>>())
            .unwrap_or_default(),
        "speed": format!("{}/s", format_bytes(summary.speed_bps as u64)),
        "location": summary.location.as_ref().map(|info| {
            localized("downloads.free_of", language)
                .replacen("%s", &format_bytes(info.free_bytes), 1)
                .replacen("%s", &format_bytes(info.total_bytes), 1)
        }),
    })
}

pub fn discover_data(discover: &DiscoverModel) -> serde_json::Value {
    let filters = crate::discover::filter_options(discover);
    let options = |pairs: &[(String, String)]| {
        pairs
            .iter()
            .map(|(value, label)| serde_json::json!({"value": value, "label": label}))
            .collect::<Vec<_>>()
    };
    let types: Vec<_> = discover
        .content_types
        .iter()
        .map(|kind| {
            let mut label = kind.clone();
            if let Some(first) = label.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            serde_json::json!({
                "id": kind,
                "key": "discover:contentType",
                "label": label,
                "selected": *kind == discover.content_type,
            })
        })
        .collect();
    serde_json::json!({
        "query": discover.query,
        "loading": discover.is_loading || discover.catalogs_loading,
        "error": discover.error,
        "contentType": discover.content_type,
        "contentTypes": discover.content_types,
        "types": types,
        "results": discover.results,
        "nextPage": discover.next_page,
        "typeOptions": options(&filters.types),
        "catalogOptions": options(&filters.catalogs),
        "extraOptions": options(&filters.extras),
        "catalog": filters.catalog_key,
        "extra": discover.selected_extra_value,
        "hasExtra": filters.has_extra && !filters.extras.is_empty(),
    })
}

pub fn hero_slides_data(home: &HomeModel) -> serde_json::Value {
    let slides = home.hero_slides.iter().map(|hero| {
        let mut slide = serde_json::to_value(hero).unwrap_or_default();
        let resume = hero.item_id.as_deref().and_then(|id| home.resume_for(id));
        let series = matches!(hero.item_type.as_deref(), Some("series" | "tv" | "show"));
        let label = home.playback_unavailable.clone().unwrap_or_else(|| {
            let first = series
                .then(|| first_episode(&hero.raw))
                .flatten()
                .unwrap_or((1, 1, String::new()));
            play_label(
                &home.language,
                resume,
                series.then_some((first.0, first.1, Some(first.2.as_str()))),
            )
        });
        let ratings: Vec<_> = mdblist_ratings(&hero.raw)
            .into_iter()
            .map(|(label, value)| {
                serde_json::json!({"label": label, "value": value, "logo": crate::rating_logo(&label, &value).unwrap_or("")})
            })
            .collect();
        if let Some(fields) = slide.as_object_mut() {
            fields.insert("playLabel".into(), label.into());
            fields.insert(
                "progress".into(),
                resume.map_or(0.0, |card| card.progress).into(),
            );
            fields.insert("ratings".into(), ratings.into());
        }
        slide
    });
    serde_json::Value::Array(slides.collect())
}

pub fn hero_from_meta(item: &serde_json::Value, language: &str) -> HomeHero {
    core_home_hero(item, language)
}
