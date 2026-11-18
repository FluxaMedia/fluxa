use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets};
use crate::{IconSize, Space};

pub(crate) fn toast(
    context: &crate::gfx::Context,
    viewport: Viewport,
    top_center: Pos2,
    text: &str,
    level: Option<f32>,
    opacity: f32,
) {
    let metrics = UiMetrics::for_viewport(viewport);
    let (size, pad, bar) = (
        metrics.toast_text_size,
        metrics.toast_padding,
        metrics.toast_bar_width,
    );
    let painter = context.layer_painter(crate::gfx::LayerId::new(
        crate::gfx::Order::Tooltip,
        Id::new("fluxa-player-toast"),
    ));
    let font = FontId::proportional(size);
    let galley =
        painter.layout_no_wrap(text.to_owned(), font, metrics.ink((255.0 * opacity) as u8));
    let text_width = galley.size().x;
    let gap = Space::S14.px(&metrics);
    let mut cells = vec![flex::slot(text_width).id(0)];
    if level.is_some() {
        cells.push(flex::slot(bar).id(1));
    }
    let cells = flex::row(cells)
        .gap(gap)
        .align(Align::Stretch)
        .padding(Insets::xy(pad, 0.0));
    let width = cells.outer_size().x;
    let height = size * metrics.control_toast_height_ratio;
    let rect = flex::anchored(
        Rect::from_x_y_ranges(
            (top_center.x - width)..=(top_center.x + width),
            top_center.y..=(top_center.y + height),
        ),
        Align2::CENTER_TOP,
        Vec2::new(width, height),
        0.0,
    );
    painter.rect_filled(
        rect,
        metrics.dialog_radius,
        metrics.surface.gamma_multiply(245.0 / 255.0 * opacity),
    );
    painter.rect_stroke(
        rect,
        metrics.dialog_radius,
        crate::gfx::Stroke::new(stroke::hairline(), metrics.ink((20.0 * opacity) as u8)),
        crate::gfx::StrokeKind::Inside,
    );
    let placed = cells.solve(rect);
    let label = placed.find(0).unwrap_or(rect);
    let meter = placed.find(1);
    let text_rect = flex::anchored(label, Align2::LEFT_CENTER, galley.size(), 0.0);
    painter.galley(text_rect.min, galley, metrics.text_primary);
    if let (Some(level), Some(meter)) = (level, meter) {
        let track = flex::anchored(
            meter,
            Align2::CENTER_CENTER,
            Vec2::new(bar, Dp::D4.px()),
            0.0,
        );
        painter.rect_filled(track, radius::BAR, metrics.ink((60.0 * opacity) as u8));
        let filled = flex::fraction(track, level);
        painter.rect_filled(filled, radius::BAR, metrics.ink((255.0 * opacity) as u8));
    }
}

/// Shared image path for UI components: request the appropriately sized
/// cached texture, calculate cover-crop UVs from its decoded dimensions, and
/// paint it consistently. Screens should compose this instead of creating an
/// independent image/loading path.
pub(crate) fn artwork_image(
    painter: &Painter,
    rect: Rect,
    url: Option<&str>,
    target_size: [u32; 2],
    priority: ArtworkPriority,
    tint: Color32,
    assets: &mut impl HomeAssets,
) -> bool {
    rounded_artwork(painter, rect, 0.0, url, target_size, priority, tint, assets)
}

pub(super) fn near_screen(painter: &Painter, rect: Rect) -> bool {
    let screen = painter.ctx().content_rect();
    screen.expand2(screen.size()).intersects(rect)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn loading_fill(painter: &Painter) -> Color32 {
    let time = painter.ctx().input(|input| input.time()) as f32;
    painter
        .ctx()
        .request_repaint_after(crate::motion::HALF_FRAME_RATE);
    let wave = (time * std::f32::consts::TAU / 1.4).sin() * 0.5 + 0.5;
    ink(tone::faint() + (wave * f32::from(tone::faint())) as u8)
}

pub(crate) fn loading_rect(painter: &Painter, rect: Rect, radius: f32) {
    painter.rect_filled(rect, radius, loading_fill(painter));
}

pub(crate) fn loading_circle(painter: &Painter, center: Pos2, radius: f32) {
    painter.circle_filled(center, radius, loading_fill(painter));
}

pub(crate) fn rounded_artwork(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    url: Option<&str>,
    target_size: [u32; 2],
    priority: ArtworkPriority,
    tint: Color32,
    assets: &mut impl HomeAssets,
) -> bool {
    if !painter.clip_rect().intersects(rect) {
        if near_screen(painter, rect) {
            assets.prefetch_for(url, target_size, ArtworkPriority::Prefetch);
        }
        return false;
    }
    let Some(texture) = assets.texture_for(url, target_size, priority) else {
        if url.is_some() {
            if assets.artwork_failed(url, target_size) {
                if let Some(url) = url.filter(|_| assets.dom_images()) {
                    dom_image(painter, rect, radius, url);
                    return true;
                }
                painter.rect_filled(rect, radius, crate::stylesheet::color("well"));
            } else {
                loading_rect(painter, rect, radius);
            }
        }
        return false;
    };
    let uv = assets
        .texture_size(url)
        .map(|size| cover_uv(size, rect))
        .unwrap_or_else(full_uv);
    if radius > 0.0 {
        rounded_image(painter, texture, rect, radius, uv, tint);
    } else {
        texture_image(painter, texture, rect, uv, tint);
    }
    true
}
