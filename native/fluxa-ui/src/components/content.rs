use super::*;


pub(crate) fn wrapped_text(
    painter: &Painter,
    text: &str,
    size: f32,
    color: Color32,
    width: f32,
    rows: usize,
) -> std::sync::Arc<crate::gfx::Galley> {
    let mut job = crate::gfx::text::LayoutJob::simple(
        text.to_owned(),
        FontId::proportional(size),
        color,
        width,
    );
    job.wrap.max_rows = rows;
    painter.layout_job(job)
}

pub(crate) fn texture_image(
    painter: &Painter,
    texture: crate::gfx::TextureId,
    rect: Rect,
    uv: Rect,
    tint: Color32,
) {
    painter.image(texture, rect, uv, tint);
}

pub(crate) fn rounded_image(
    painter: &Painter,
    texture: crate::gfx::TextureId,
    rect: Rect,
    radius: impl Into<crate::gfx::CornerRadius>,
    uv: Rect,
    tint: Color32,
) {
    painter
        .add(crate::gfx::epaint::RectShape::filled(rect, radius, tint).with_texture(texture, uv));
}


