use super::*;
use crate::dp::Dp;

#[derive(Clone, Copy)]
pub(super) struct HeroStyle {
    pub centered: bool,
    pub shows_synopsis: bool,
    pub width: f32,
    pub fade_height: f32,
    pub bottom_padding: f32,
    pub min_top: f32,
    pub logo_height: f32,
    pub text_title_height: f32,
    pub logo_inset: f32,
    pub logo_max_width: f32,
    pub title_size: f32,
    pub metadata_height: f32,
    pub metadata_size: f32,
    pub metadata_chars: usize,
    pub title_gap: Space,
    pub metadata_gap: Space,
    pub synopsis_size: f32,
    pub synopsis_rows: usize,
    pub button_gap: Space,
    pub button_height: f32,
}

impl HeroStyle {
    pub fn new(viewport: Viewport, metrics: UiMetrics, margin: f32) -> Self {
        if viewport.is_compact() {
            return Self {
                centered: true,
                shows_synopsis: false,
                width: viewport.width - margin * 2.0,
                fade_height: Dp::D96.px(),
                bottom_padding: Dp::D44.px(),
                min_top: Dp::D16.px(),
                logo_height: Dp::D72.px(),
                text_title_height: crate::vars::px("catalog-title-size") * 2.25,
                logo_inset: Dp::D6.px(),
                logo_max_width: crate::vars::px("screen-hero-compact-logo-max-width"),
                title_size: crate::vars::px("catalog-title-size") + 2.0,
                metadata_height: Dp::D16.px(),
                metadata_size: crate::vars::px("nav-label-size"),
                metadata_chars: 64,
                title_gap: Space::S6,
                metadata_gap: Space::S6,
                synopsis_size: crate::vars::px("card-title-size") + 1.0,
                synopsis_rows: 2,
                button_gap: Space::S6,
                button_height: crate::vars::px("button-height"),
            };
        }
        let sizes = crate::stylesheet::base_with("hero-sizes", &[]);
        let metadata = crate::stylesheet::base_with("hero-metadata-size", &[]);
        let synopsis = crate::stylesheet::base_with("hero-synopsis-size", &[]);
        Self {
            centered: false,
            shows_synopsis: true,
            width: (viewport.width * crate::vars::px("screen-hero-width-ratio"))
                .min(sizes.width.unwrap_or(980.0)),
            fade_height: crate::vars::px("screen-hero-desktop-fade"),
            bottom_padding: crate::vars::px("home-hero-bottom-padding"),
            min_top: crate::vars::px("nav-vertical-padding") * 2.0,
            logo_height: sizes.height.unwrap_or(148.0),
            text_title_height: crate::vars::px("catalog-title-size") * 1.9,
            logo_inset: Dp::D8.px(),
            logo_max_width: sizes.max_width.unwrap_or(460.0),
            title_size: crate::vars::px("catalog-title-size") * sizes.scale,
            metadata_height: Dp::D20.px(),
            metadata_size: metadata.font_size.unwrap_or(14.0),
            metadata_chars: 90,
            title_gap: Space::S16,
            metadata_gap: Space::S4,
            synopsis_size: synopsis.font_size.unwrap_or(16.0),
            synopsis_rows: 3,
            button_gap: Space::S20,
            button_height: crate::vars::px("button-height"),
        }
    }

    pub fn fallback_play(&self, metrics: UiMetrics, width: f32, margin: f32) -> Rect {
        let (rise, height) = if self.centered {
            (Dp::D96.px(), crate::vars::px("button-height"))
        } else {
            (crate::vars::px("screen-hero-desktop-play-rise"), Dp::D48.px())
        };
        play_rect(self.centered, metrics, width, margin, -rise, height)
    }

    pub fn synopsis_width(&self) -> f32 {
        if self.centered {
            self.width
        } else {
            crate::metrics::active()
                .home_hero_width
                .of(self.width)
                .min(self.width)
        }
    }
}

pub(super) fn play_rect(
    centered: bool,
    metrics: UiMetrics,
    width: f32,
    margin: f32,
    top: f32,
    height: f32,
) -> Rect {
    let wide = (crate::vars::px("horizontal-card-width") * crate::vars::px("screen-hero-play-card-ratio"))
        .max(crate::vars::px("screen-hero-play-min-width"));
    let size = Vec2::new(
        if centered {
            crate::vars::px("screen-hero-play-compact-width")
        } else {
            wide
        },
        height,
    );
    let lane = crate::gfx::rect_at(Pos2::new(0.0, top), Vec2::new(width, size.y));
    let (align, padding) = if centered {
        (Align2::CENTER_TOP, crate::flex::Insets::default())
    } else {
        (
            Align2::LEFT_TOP,
            crate::flex::Insets {
                left: margin,
                ..crate::flex::Insets::default()
            },
        )
    };
    crate::flex::stack(align, [crate::flex::leaf(size).id(0)])
        .padding(padding)
        .solve(lane)
        .find(0)
        .unwrap_or(lane)
}
