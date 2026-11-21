use super::*;

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

pub(crate) fn home_row_dimensions(kind: HomeRowKind) -> (f32, f32, f32) {
    match kind {
        HomeRowKind::Continue => (
            crate::metrics::theme().continue_width,
            crate::metrics::theme().continue_height,
            crate::metrics::theme().continue_height,
        ),
        HomeRowKind::Poster => (
            crate::metrics::theme().poster_width,
            crate::metrics::theme().poster_height,
            crate::metrics::theme().poster_height
                + crate::vars::px("control-gap")
                + crate::vars::px("screen-card-title-size")
                + crate::vars::px("screen-card-subtitle-size")
                + crate::vars::px("control-gap"),
        ),
        HomeRowKind::Collection => (
            crate::vars::px("collection-poster-width"),
            crate::vars::px("collection-poster-height"),
            crate::vars::px("collection-poster-height")
                + crate::vars::px("control-gap")
                + crate::vars::px("screen-card-title-size"),
        ),
    }
}

pub(crate) fn home_collection_card_dimensions(card: &HomeCard) -> (f32, f32) {
    let (width, height) = match card
        .collection_shape
        .as_deref()
        .unwrap_or("poster")
        .to_ascii_lowercase()
        .as_str()
    {
        "wide" | "landscape" => (
            crate::vars::px("collection-wide-width"),
            crate::vars::px("collection-wide-height"),
        ),
        "square" => (
            crate::vars::px("collection-square-size"),
            crate::vars::px("collection-square-size"),
        ),
        _ => (
            crate::vars::px("collection-poster-width"),
            crate::vars::px("collection-poster-height"),
        ),
    };
    (width, height)
}

pub(crate) fn home_card_dimensions(card: &HomeCard, kind: HomeRowKind) -> (f32, f32) {
    if kind == HomeRowKind::Collection {
        return home_collection_card_dimensions(card);
    }
    let (width, height, _) = home_row_dimensions(kind);
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

pub(crate) fn home_row_body_height(cards: &[HomeCard], kind: HomeRowKind) -> f32 {
    if kind != HomeRowKind::Collection {
        return home_row_dimensions(kind).2;
    }
    let max_tile_height = cards
        .iter()
        .map(|card| home_collection_card_dimensions(card))
        .map(|(_, height)| height)
        .fold(0.0_f32, f32::max);
    max_tile_height + crate::vars::px("control-gap") + crate::vars::px("screen-card-title-size")
}

/// Height occupied by a shelf heading and the gap between it and its cards.
/// Keep this in the shared geometry calculation: omitting it made each next
/// shelf start while the previous shelf's cards were still on screen.
pub(crate) fn home_row_heading_height() -> f32 {
    crate::vars::px("row-heading-size") * 1.25 + crate::vars::px("control-gap")
}

pub(crate) fn home_hero_height(viewport: Viewport) -> f32 {
    crate::metrics::enter(viewport);
    let base_height = if viewport.is_compact() {
        if viewport.width > viewport.height * crate::vars::px("screen-hero-landscape-switch") {
            (viewport.width * crate::vars::px("screen-hero-landscape-ratio"))
                .min(crate::vars::px("screen-hero-landscape-max"))
        } else {
            (viewport.width * crate::vars::px("screen-hero-portrait-ratio"))
                .min(crate::vars::px("screen-hero-portrait-max"))
        }
    } else {
        crate::vars::scaled("home-hero-height", viewport.height)
    };
    if viewport.is_tv() {
        return viewport.height * crate::vars::px("screen-hero-tv-height-ratio");
    }
    base_height * crate::vars::px("screen-hero-trim-ratio")
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
        crate::metrics::enter(viewport);
        if viewport.is_tv() {
            return viewport.height * crate::vars::px("screen-hero-tv-height-ratio");
        }
        (viewport.height * crate::vars::px("screen-hero-pinned-height-ratio"))
            .max(crate::vars::px("screen-hero-pinned-min-height"))
    } else {
        home_hero_height(viewport)
    }
}

/// Where the focused shelf snaps while the hero is pinned: the screen y of
/// its heading and the heading height.
pub fn home_pinned_rows(viewport: Viewport, home: &HomeModel) -> Option<(f32, f32)> {
    home_hero_pinned(viewport, home).then(|| {
        crate::metrics::enter(viewport);
        let hero_height = home_hero_height_for(viewport, home);
        (
            home_row_start(viewport, hero_height, true),
            home_row_heading_height(),
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

pub(crate) fn home_flow(viewport: Viewport, home: &HomeModel) -> HomeFlow {
    let hero_height = home_hero_height_for(viewport, home);
    let lead = if home_hero_pinned(viewport, home) {
        0.0
    } else {
        home.top_inset
    } + super::category_bar_space(viewport, home);
    let mut column = crate::flow::Column::from(0.0);
    column.place(lead);
    let hero_top = column.place(home_row_start(viewport, hero_height, home_shows_hero(home)));
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

pub(crate) fn shelf_gap() -> f32 {
    crate::vars::px("section-gap") + crate::vars::px("vertical-spacing")
}

pub(crate) fn home_shelves<'a>(
    home: &'a HomeModel,
    top: f32,
) -> impl Iterator<Item = Shelf<'a>> + 'a {
    let heading = home_row_heading_height();
    let mut column = crate::flow::Column::from(top);
    let mut first_card = 0;
    home.content_rows_with_kind()
        .enumerate()
        .map(move |(index, (title, cards, kind))| {
            let body = home_row_body_height(cards, kind);
            let shelf = Shelf {
                index,
                top: column.place(heading + body + shelf_gap()),
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

pub(crate) fn home_row_start(viewport: Viewport, hero_height: f32, show_hero: bool) -> f32 {
    if !show_hero {
        return crate::vars::px("page-padding") / 2.0;
    }
    hero_height
        + if viewport.is_compact() {
            crate::vars::px("control-gap").max(8.0)
        } else {
            crate::vars::px("section-gap").max(crate::vars::px("control-gap") * 2.0)
        }
}
