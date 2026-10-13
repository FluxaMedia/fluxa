use super::*;

pub fn metrics_for_assets(viewport: Viewport, assets: &impl HomeAssets) -> UiMetrics {
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
pub(crate) struct HeroFade {
    pub(super) key: String,
    pub(super) from: Option<(TextureId, [u32; 2])>,
    pub(super) current: Option<(TextureId, [u32; 2])>,
    pub(super) started: f64,
}

#[derive(Clone)]
pub(crate) struct LastReadyHeroTexture {
    pub(super) texture: TextureId,
    pub(super) size: [u32; 2],
    pub(super) hero: HomeHero,
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

pub(crate) fn home_row_dimensions(metrics: UiMetrics, kind: HomeRowKind) -> (f32, f32, f32) {
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
            metrics.collection_poster_width,
            metrics.collection_poster_height,
            metrics.collection_poster_height + metrics.control_gap + metrics.screen_card_title_size,
        ),
    }
}

pub(crate) fn home_collection_card_dimensions(metrics: UiMetrics, card: &HomeCard) -> (f32, f32) {
    let (width, height) = match card
        .collection_shape
        .as_deref()
        .unwrap_or("poster")
        .to_ascii_lowercase()
        .as_str()
    {
        "wide" | "landscape" => (
            metrics.collection_wide_width,
            metrics.collection_wide_height,
        ),
        "square" => (
            metrics.collection_square_size,
            metrics.collection_square_size,
        ),
        _ => (
            metrics.collection_poster_width,
            metrics.collection_poster_height,
        ),
    };
    (width, height)
}

pub(crate) fn home_card_dimensions(
    metrics: UiMetrics,
    card: &HomeCard,
    kind: HomeRowKind,
) -> (f32, f32) {
    if kind == HomeRowKind::Collection {
        return home_collection_card_dimensions(metrics, card);
    }
    let (width, height, _) = home_row_dimensions(metrics, kind);
    if kind != HomeRowKind::Poster || card.is_landscape() {
        return (width, height);
    }
    match card.poster_shape.as_deref() {
        Some("landscape") => (height * 16.0 / 9.0, height),
        Some("square") => (height, height),
        _ => (width, height),
    }
}

pub(crate) fn home_card_lead(title: &str, kind: HomeRowKind, index: usize, height: f32) -> f32 {
    poster_overlay::lead_width(height, poster_overlay::rank_for(title, kind, index))
}

pub(crate) fn home_row_body_height(
    metrics: UiMetrics,
    cards: &[HomeCard],
    kind: HomeRowKind,
) -> f32 {
    if kind != HomeRowKind::Collection {
        return home_row_dimensions(metrics, kind).2;
    }
    let max_tile_height = cards
        .iter()
        .map(|card| home_collection_card_dimensions(metrics, card))
        .map(|(_, height)| height)
        .fold(0.0_f32, f32::max);
    max_tile_height + metrics.control_gap + metrics.screen_card_title_size
}

pub(crate) fn home_metrics(viewport: Viewport) -> UiMetrics {
    UiMetrics::for_viewport(viewport)
}

/// Height occupied by a shelf heading and the gap between it and its cards.
/// Keep this in the shared geometry calculation: omitting it made each next
/// shelf start while the previous shelf's cards were still on screen.
pub(crate) fn home_row_heading_height(metrics: UiMetrics) -> f32 {
    metrics.row_heading_size * 1.25 + metrics.control_gap
}

pub(crate) fn home_artwork_signature(home: &HomeModel) -> u64 {
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

pub(crate) fn should_prefetch_home_artwork(context: &egui::Context, home: &HomeModel) -> bool {
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

pub(crate) fn home_hero_height(viewport: Viewport) -> f32 {
    let base_height = if viewport.is_compact() {
        if viewport.width > viewport.height * 1.15 {
            (viewport.width * 5.0 / 12.0).min(440.0)
        } else {
            (viewport.width * 4.0 / 3.0).min(560.0)
        }
    } else {
        (viewport.height * 0.66 + 120.0).clamp(728.0, 984.0)
    };
    if viewport.is_tv() {
        return viewport.height * 0.70;
    }
    base_height * 0.93
}

pub(crate) fn home_shows_hero(home: &HomeModel) -> bool {
    home.show_hero_section
        && (home.item_id.is_some() || !home.cards.is_empty() || !home.rows.is_empty())
}

pub(crate) fn home_hero_pinned(viewport: Viewport, home: &HomeModel) -> bool {
    home.follow_focus_hero && home.focus_driven && home_shows_hero(home) && !viewport.is_compact()
}

pub(crate) fn home_hero_height_for(viewport: Viewport, home: &HomeModel) -> f32 {
    if home_hero_pinned(viewport, home) {
        if viewport.is_tv() {
            return viewport.height * 0.70;
        }
        (viewport.height * 0.55).max(300.0)
    } else {
        home_hero_height(viewport)
    }
}

/// Where the focused shelf snaps while the hero is pinned: the screen y of
/// its heading and the heading height.
pub fn home_pinned_rows(viewport: Viewport, home: &HomeModel) -> Option<(f32, f32)> {
    home_hero_pinned(viewport, home).then(|| {
        let metrics = home_metrics(viewport);
        let hero_height = home_hero_height_for(viewport, home);
        (
            home_row_start(viewport, metrics, hero_height, true),
            home_row_heading_height(metrics),
        )
    })
}

/// The first shelf starts after the complete hero viewport, not after an
/// estimate of where the hero buttons happen to land. This keeps the hero
/// controls and the shelf heading in separate visual blocks on every size.
pub(crate) struct HomeFlow {
    pub hero_top: f32,
    pub rows_top: f32,
}

pub(crate) fn home_flow(viewport: Viewport, metrics: UiMetrics, home: &HomeModel) -> HomeFlow {
    let hero_height = home_hero_height_for(viewport, home);
    let lead = if home_hero_pinned(viewport, home) {
        0.0
    } else {
        home.top_inset
    };
    let mut column = crate::flow::Column::from(0.0);
    column.place(lead);
    let hero_top = column.place(home_row_start(
        viewport,
        metrics,
        hero_height,
        home_shows_hero(home),
    ));
    HomeFlow {
        hero_top,
        rows_top: column.end(),
    }
}

pub(crate) fn home_nav_viewport(viewport: Viewport, home: &HomeModel) -> Viewport {
    if viewport.is_compact() {
        return viewport;
    }
    let scrolled = if home_hero_pinned(viewport, home) {
        0.0
    } else {
        home.scroll_offset
    };
    viewport.with_nav_lead(home.top_inset - scrolled)
}

pub(crate) struct Shelf<'a> {
    pub index: usize,
    pub top: f32,
    pub heading: f32,
    pub body: f32,
    pub first_card: usize,
    pub title: &'a str,
    pub cards: &'a [HomeCard],
    pub kind: HomeRowKind,
}

impl Shelf<'_> {
    pub(crate) fn height(&self) -> f32 {
        self.heading + self.body
    }
}

pub(crate) fn shelf_gap(metrics: UiMetrics) -> f32 {
    metrics.section_gap + metrics.vertical_spacing
}

pub(crate) fn home_shelves<'a>(
    metrics: UiMetrics,
    home: &'a HomeModel,
    top: f32,
) -> impl Iterator<Item = Shelf<'a>> + 'a {
    let heading = home_row_heading_height(metrics);
    let mut column = crate::flow::Column::from(top);
    let mut first_card = 0;
    home.content_rows_with_kind()
        .enumerate()
        .map(move |(index, (title, cards, kind))| {
            let body = home_row_body_height(metrics, cards, kind);
            let shelf = Shelf {
                index,
                top: column.place(heading + body + shelf_gap(metrics)),
                heading,
                body,
                first_card,
                title,
                cards,
                kind,
            };
            first_card += cards.len();
            shelf
        })
}

pub(crate) fn home_row_start(
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
