use super::*;

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
