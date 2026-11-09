use super::*;

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
                fade_height: metrics.screen_hero_compact_fade,
                bottom_padding: metrics.screen_hero_compact_bottom_padding,
                min_top: metrics.screen_hero_compact_min_top,
                logo_height: metrics.screen_hero_compact_logo_height,
                text_title_height: metrics.catalog_title_size * 2.25,
                logo_inset: metrics.screen_hero_compact_logo_inset,
                logo_max_width: metrics.screen_hero_compact_logo_max_width,
                title_size: metrics.catalog_title_size + 2.0,
                metadata_height: metrics.screen_hero_compact_metadata_height,
                metadata_size: metrics.nav_label_size,
                metadata_chars: 64,
                title_gap: Space::S6,
                metadata_gap: Space::S6,
                synopsis_size: metrics.card_title_size + 1.0,
                synopsis_rows: 2,
                button_gap: Space::S6,
                button_height: metrics.button_height,
            };
        }
        Self {
            centered: false,
            shows_synopsis: true,
            width: (viewport.width * metrics.screen_hero_width_ratio).min(metrics.hero_width_cap),
            fade_height: metrics.screen_hero_desktop_fade,
            bottom_padding: metrics.home_hero_bottom_padding,
            min_top: metrics.nav_vertical_padding * 2.0,
            logo_height: metrics.hero_logo_height,
            text_title_height: metrics.catalog_title_size * 1.9,
            logo_inset: metrics.screen_hero_desktop_logo_inset,
            logo_max_width: metrics.hero_logo_width_cap,
            title_size: metrics.catalog_title_size * metrics.hero_title_scale,
            metadata_height: metrics.screen_hero_desktop_metadata_height,
            metadata_size: metrics.hero_metadata_size,
            metadata_chars: 90,
            title_gap: Space::S16,
            metadata_gap: Space::S4,
            synopsis_size: metrics.hero_synopsis_size,
            synopsis_rows: 3,
            button_gap: Space::S20,
            button_height: metrics.button_height,
        }
    }

    pub fn fallback_play(&self, metrics: UiMetrics, width: f32, margin: f32) -> Rect {
        let (rise, height) = if self.centered {
            (metrics.screen_hero_compact_play_rise, metrics.button_height)
        } else {
            (
                metrics.screen_hero_desktop_play_rise,
                metrics.screen_hero_fallback_play_height,
            )
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
    let wide = (metrics.horizontal_card_width * metrics.screen_hero_play_card_ratio)
        .max(metrics.screen_hero_play_min_width);
    let size = Vec2::new(
        if centered {
            metrics.screen_hero_play_compact_width
        } else {
            wide
        },
        height,
    );
    let lane = Rect::from_min_size(Pos2::new(0.0, top), Vec2::new(width, size.y));
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
