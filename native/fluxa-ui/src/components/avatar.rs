use super::*;

pub(crate) fn title_logo(
    ppp: f32,
    url: Option<&str>,
    bounds: Vec2,
    priority: ArtworkPriority,
    assets: &mut impl HomeAssets,
) -> Option<(crate::gfx::TextureId, Vec2)> {
    let url = url.filter(|url| !url.trim().is_empty());
    let texture = assets.texture_for(url, artwork_target_size(bounds, ppp), priority)?;
    let [width, height] = assets.texture_size(url)?;
    let aspect = width as f32 / height.max(1) as f32;
    let size = if bounds.y * aspect > bounds.x {
        Vec2::new(bounds.x, bounds.x / aspect)
    } else {
        Vec2::new(bounds.y * aspect, bounds.y)
    };
    Some((texture, size))
}

pub(crate) fn avatar(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    center: crate::gfx::Pos2,
    radius: f32,
    _name: &str,
    url: Option<&str>,
) {
    avatar_sized(painter, assets, center, radius, radius, _name, url);
}

pub(crate) fn avatar_sized(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    center: crate::gfx::Pos2,
    radius: f32,
    texture_radius: f32,
    _name: &str,
    url: Option<&str>,
) {
    if dom_avatar(painter, assets, center, radius, url) {
        return;
    }
    let rect = crate::gfx::square_around(center, radius * 2.0);
    let target = artwork_target_size(
        Vec2::splat(texture_radius * 2.0),
        painter.ctx().pixels_per_point(),
    );
    if rounded_artwork(
        painter,
        rect,
        radius,
        url,
        target,
        ArtworkPriority::Visible,
        Color32::WHITE,
        assets,
    ) {
        animated_avatar(painter, assets, center, radius, texture_radius, url);
        return;
    }
    if url.is_some() {
        loading_circle(painter, center, radius);
        return;
    }
    painter.circle_filled(center, radius, crate::stylesheet::color("raised"));
}

pub(crate) fn animated_avatar(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    center: Pos2,
    radius: f32,
    texture_radius: f32,
    url: Option<&str>,
) {
    let rect = crate::gfx::square_around(center, radius * 2.0);
    let target = artwork_target_size(
        Vec2::splat(texture_radius * 2.0),
        painter.ctx().pixels_per_point(),
    );
    let Some(animated) = assets.animated_texture_for(
        url,
        fluxa_artwork::animation_target_size(target),
        ArtworkPriority::Visible,
    ) else {
        return;
    };
    let crop = cover_uv(animated.image_size, rect);
    let tile = animated.uv;
    let uv = Rect::from_min_max(
        Pos2::new(
            tile.min.x + crop.min.x * tile.width(),
            tile.min.y + crop.min.y * tile.height(),
        ),
        Pos2::new(
            tile.min.x + crop.max.x * tile.width(),
            tile.min.y + crop.max.y * tile.height(),
        ),
    );
    painter.circle_filled(center, radius, crate::stylesheet::color("raised"));
    rounded_image(painter, animated.texture, rect, radius, uv, Color32::WHITE);
}
