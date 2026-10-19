use super::*;
use crate::components::title_logo;
use crate::draw_text;

pub const BACKDROP_IN: f32 = 0.9;
pub const OVERLAY_IN: f32 = 0.9;
pub const OVERLAY_OUT: f32 = 0.4;
const TITLE_DELAY: f32 = 0.25;
const TITLE_IN: f32 = 0.5;
const GENRES_DELAY: f32 = 0.1;
const GENRES_IN: f32 = 0.45;
const HOLD: f32 = 10.0;
const GENRES_OUT: f32 = 0.25;
const TITLE_OUT: f32 = 0.35;
const BACKDROP_OUT: f32 = 0.7;
const BLACK_HOLD: f32 = 0.3;
const TITLE_RISE: f32 = 18.0;
const GENRES_RISE: f32 = 12.0;
const MAX_SCALE: f32 = 1.08;
const MAX_PAN_X: f32 = 0.025;
const MAX_PAN_Y: f32 = 0.02;

pub const MOTION_COUNT: usize = 7;

const TITLE_AT: f32 = TITLE_DELAY;
const GENRES_AT: f32 = TITLE_AT + GENRES_DELAY;
const SETTLED: f32 = GENRES_AT + GENRES_IN;
const GENRES_OUT_AT: f32 = SETTLED + HOLD;
const TITLE_OUT_AT: f32 = GENRES_OUT_AT + GENRES_OUT;
const BACKDROP_OUT_AT: f32 = TITLE_OUT_AT + TITLE_OUT;
pub const SLIDE_SECONDS: f32 = BACKDROP_OUT_AT + BACKDROP_OUT + BLACK_HOLD;

#[derive(Clone, Debug, Default)]
pub struct AmbientView {
    pub backdrop: String,
    pub title: String,
    pub logo: Option<String>,
    pub genres: Vec<String>,
    pub elapsed: f32,
    pub motion: usize,
    pub overlay: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timeline {
    pub backdrop: f32,
    pub title: f32,
    pub genres: f32,
}

fn ramp(elapsed: f32, start: f32, length: f32) -> f32 {
    ((elapsed - start) / length).clamp(0.0, 1.0)
}

pub fn timeline(elapsed: f32) -> Timeline {
    let backdrop = ramp(elapsed, 0.0, BACKDROP_IN) - ramp(elapsed, BACKDROP_OUT_AT, BACKDROP_OUT);
    let title = ramp(elapsed, TITLE_AT, TITLE_IN) - ramp(elapsed, TITLE_OUT_AT, TITLE_OUT);
    let genres = ramp(elapsed, GENRES_AT, GENRES_IN) - ramp(elapsed, GENRES_OUT_AT, GENRES_OUT);
    Timeline {
        backdrop: backdrop.clamp(0.0, 1.0),
        title: title.clamp(0.0, 1.0),
        genres: genres.clamp(0.0, 1.0),
    }
}

pub fn next_motion(previous: Option<usize>, roll: u64) -> usize {
    let pool = MOTION_COUNT - usize::from(previous.is_some());
    let pick = (roll % pool as u64) as usize;
    match previous {
        Some(previous) if pick >= previous => pick + 1,
        _ => pick,
    }
}

fn motion_path(index: usize) -> ([f32; 3], [f32; 3]) {
    let rest = [1.0, 0.0, 0.0];
    let full = |x: f32, y: f32| [MAX_SCALE, x, y];
    match index % MOTION_COUNT {
        0 => (rest, full(0.0, 0.0)),
        1 => (rest, full(-MAX_PAN_X, 0.0)),
        2 => (rest, full(MAX_PAN_X, 0.0)),
        3 => (rest, full(0.0, -MAX_PAN_Y)),
        4 => (full(0.0, 0.0), rest),
        5 => (rest, full(-MAX_PAN_X, -MAX_PAN_Y)),
        _ => (full(MAX_PAN_X, MAX_PAN_Y), rest),
    }
}

fn ease(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn ease_out(value: f32) -> f32 {
    1.0 - (1.0 - value).powi(3)
}

pub fn ken_burns(motion: usize, elapsed: f32) -> [f32; 3] {
    let (from, to) = motion_path(motion);
    let progress = ease((elapsed / SLIDE_SECONDS).clamp(0.0, 1.0));
    let mix = |index: usize| from[index] + (to[index] - from[index]) * progress;
    let scale = mix(0);
    let slack = (scale - 1.0) * 0.5;
    [
        scale,
        mix(1).clamp(-slack, slack),
        mix(2).clamp(-slack, slack),
    ]
}

fn crop(uv: Rect, motion: [f32; 3]) -> Rect {
    let [scale, pan_x, pan_y] = motion;
    let size = uv.size() / scale;
    let center = uv.center() + Vec2::new(pan_x * uv.width(), pan_y * uv.height());
    Rect::from_center_size(center, size)
}

pub fn draw_ambient(
    context: &egui::Context,
    viewport: Viewport,
    view: &AmbientView,
    assets: &mut impl HomeAssets,
) {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let layer = egui::LayerId::new(egui::Order::Debug, egui::Id::new("fluxa-ambient"));
    let painter = context.layer_painter(layer);
    let overlay = view.overlay.clamp(0.0, 1.0);
    painter.rect_filled(screen, 0.0, Color32::BLACK.gamma_multiply(overlay));
    let stage = timeline(view.elapsed);
    let texture = assets.cached_texture(Some(&view.backdrop));
    let size = assets.texture_size(Some(&view.backdrop));
    if let (Some(texture), Some(size)) = (texture, size) {
        let uv = crop(
            crate::cover_uv(size, screen),
            ken_burns(view.motion, view.elapsed),
        );
        let fade = ease(stage.backdrop) * overlay;
        painter.image(texture, screen, uv, Color32::WHITE.gamma_multiply(fade));
        let shade = |alpha: u8| Color32::from_black_alpha(alpha).gamma_multiply(fade);
        crate::paint_vertical_gradient(&painter, screen, shade(70), shade(225));
    }
    let margin = (viewport.width * 0.06).max(32.0);
    let genres = view.genres.join("  •  ");
    let genres_size = (viewport.height * 0.034).clamp(14.0, 34.0);
    let title_size = (viewport.height * 0.085).clamp(24.0, 84.0);
    let genres_y = screen.bottom() - viewport.height * 0.1;
    let title_bottom = genres_y - genres_size * 1.6;
    let title_alpha = ease(stage.title) * overlay;
    let title_rise = (1.0 - ease_out(stage.title)) * TITLE_RISE;
    let logo_bounds = Vec2::new(viewport.width * 0.36, viewport.height * 0.2);
    let logo = view.logo.as_deref().and_then(|url| {
        title_logo(
            context.pixels_per_point(),
            Some(url),
            logo_bounds,
            ArtworkPriority::Hero,
            assets,
        )
    });
    if let Some((texture, logo_size)) = logo {
        let rect = Rect::from_min_size(
            Pos2::new(margin, title_bottom - logo_size.y + title_rise),
            logo_size,
        );
        painter.image(
            texture,
            rect,
            crate::full_uv(),
            Color32::WHITE.gamma_multiply(title_alpha),
        );
    } else if crate::components::failed_dom_image(
        &painter,
        assets,
        Rect::from_min_size(
            Pos2::new(margin, title_bottom - logo_bounds.y + title_rise),
            logo_bounds,
        ),
        0.0,
        view.logo.as_deref(),
        artwork_target_size(logo_bounds, context.pixels_per_point()),
        crate::ImageFit::Contain,
        [0.0, 1.0],
    ) {
    } else if !view.title.is_empty() {
        draw_text(
            &painter,
            Pos2::new(margin, title_bottom + title_rise),
            Align2::LEFT_BOTTOM,
            &view.title,
            crate::fonts::bold(title_size),
            Color32::WHITE.gamma_multiply(title_alpha),
            Some(viewport.width - margin * 2.0),
        );
    }
    if !genres.is_empty() {
        let alpha = ease(stage.genres) * overlay;
        let rise = (1.0 - ease_out(stage.genres)) * GENRES_RISE;
        draw_text(
            &painter,
            Pos2::new(margin, genres_y + rise),
            Align2::LEFT_CENTER,
            genres,
            crate::fonts::medium(genres_size),
            Color32::from_white_alpha(220).gamma_multiply(alpha),
            Some(viewport.width - margin * 2.0),
        );
    }
    context.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_arrives_after_the_backdrop_starts() {
        let early = timeline(0.2);
        assert!(early.backdrop > 0.0);
        assert_eq!(early.title, 0.0);
        let later = timeline(1.0);
        assert_eq!(later.title, 1.0);
        assert_eq!(later.genres, 1.0);
    }

    #[test]
    fn everything_is_dark_before_the_next_slide() {
        let end = timeline(SLIDE_SECONDS - 0.01);
        assert_eq!(end.backdrop, 0.0);
        assert_eq!(end.title, 0.0);
        assert_eq!(end.genres, 0.0);
    }

    #[test]
    fn motion_never_repeats() {
        for previous in 0..MOTION_COUNT {
            for roll in 0..50 {
                let next = next_motion(Some(previous), roll);
                assert_ne!(next, previous);
                assert!(next < MOTION_COUNT);
            }
        }
    }

    #[test]
    fn pan_stays_inside_the_zoomed_image() {
        for motion in 0..MOTION_COUNT {
            for step in 0..=20 {
                let [scale, x, y] = ken_burns(motion, SLIDE_SECONDS * step as f32 / 20.0);
                assert!(scale >= 1.0 && scale <= MAX_SCALE);
                assert!(x.abs() <= (scale - 1.0) * 0.5 + f32::EPSILON);
                assert!(y.abs() <= (scale - 1.0) * 0.5 + f32::EPSILON);
            }
        }
    }
}
