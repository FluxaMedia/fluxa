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
    pub title_chars: usize,
    pub metadata_height: f32,
    pub metadata_size: f32,
    pub metadata_chars: usize,
    pub title_gap: f32,
    pub metadata_gap: f32,
    pub synopsis_size: f32,
    pub synopsis_rows: usize,
    pub button_gap: f32,
    pub button_height: f32,
    pub stack_extra: f32,
}

impl HeroStyle {
    pub fn new(viewport: Viewport, metrics: UiMetrics, margin: f32) -> Self {
        if viewport.is_compact() {
            return Self {
                centered: true,
                shows_synopsis: false,
                width: viewport.width - margin * 2.0,
                fade_height: 96.0,
                bottom_padding: 44.0,
                min_top: 16.0,
                logo_height: 72.0,
                text_title_height: metrics.catalog_title_size * 2.25,
                logo_inset: 6.0,
                logo_max_width: 220.0,
                title_size: metrics.catalog_title_size + 2.0,
                title_chars: 52,
                metadata_height: 18.0,
                metadata_size: metrics.nav_label_size,
                metadata_chars: 64,
                title_gap: 6.0,
                metadata_gap: 6.0,
                synopsis_size: metrics.card_title_size + 1.0,
                synopsis_rows: 2,
                button_gap: 6.0,
                button_height: 42.0,
                stack_extra: 26.0,
            };
        }
        let tv = viewport.is_tv();
        let width_cap = if tv {
            980.0
        } else {
            metrics.home_hero_content_max_width_desktop
        };
        Self {
            centered: false,
            shows_synopsis: true,
            width: (viewport.width * 0.58).min(width_cap),
            fade_height: 176.0,
            bottom_padding: 48.0,
            min_top: metrics.nav_vertical_padding * 2.0,
            logo_height: if tv {
                112.0
            } else {
                metrics.home_hero_logo_height_desktop
            },
            text_title_height: metrics.catalog_title_size * 1.9,
            logo_inset: 8.0,
            logo_max_width: if tv {
                380.0
            } else {
                metrics.home_hero_logo_max_width_desktop.min(460.0)
            },
            title_size: if tv {
                metrics.catalog_title_size * 2.0
            } else {
                metrics.catalog_title_size * 1.65
            },
            title_chars: 72,
            metadata_height: 22.0,
            metadata_size: if tv {
                metrics.nav_label_size + 4.0
            } else {
                metrics.nav_label_size
            },
            metadata_chars: 90,
            title_gap: 16.0,
            metadata_gap: 4.0,
            synopsis_size: if tv {
                metrics.card_title_size + 6.0
            } else {
                metrics.home_hero_synopsis_size_desktop
            },
            synopsis_rows: 3,
            button_gap: 22.0,
            button_height: 44.0,
            stack_extra: 20.0 + 22.0,
        }
    }

    pub fn fallback_play(&self, metrics: UiMetrics, width: f32, margin: f32) -> Rect {
        if self.centered {
            Rect::from_min_size(
                Pos2::new((width - 108.0) * 0.5, -92.0),
                Vec2::new(108.0, 42.0),
            )
        } else {
            Rect::from_min_size(
                Pos2::new(margin, -194.0),
                Vec2::new((metrics.horizontal_card_width * 0.46).max(160.0), 50.0),
            )
        }
    }

    pub fn synopsis_width(&self) -> f32 {
        if self.centered {
            self.width
        } else {
            (self.width * 0.55).clamp(420.0, 720.0).min(self.width)
        }
    }
}
