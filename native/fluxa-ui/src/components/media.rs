use super::*;

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
