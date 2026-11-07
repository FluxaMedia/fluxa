use super::*;

pub(crate) fn title_logo(
    ppp: f32,
    url: Option<&str>,
    bounds: Vec2,
    priority: ArtworkPriority,
    assets: &mut impl HomeAssets,
) -> Option<(egui::TextureId, Vec2)> {
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
    center: egui::Pos2,
    radius: f32,
    _name: &str,
    url: Option<&str>,
    metrics: UiMetrics,
) {
    avatar_sized(painter, assets, center, radius, radius, _name, url, metrics);
}

pub(crate) fn avatar_sized(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    center: egui::Pos2,
    radius: f32,
    texture_radius: f32,
    _name: &str,
    url: Option<&str>,
    _metrics: UiMetrics,
) {
    if dom_avatar(painter, assets, center, radius, url) {
        return;
    }
    let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
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
    painter.circle_filled(center, radius, ink(tone::raised()));
}

pub(crate) fn animated_avatar(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    center: Pos2,
    radius: f32,
    texture_radius: f32,
    url: Option<&str>,
) {
    let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
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
    painter.circle_filled(center, radius, ink(tone::raised()));
    rounded_image(painter, animated.texture, rect, radius, uv, Color32::WHITE);
}

pub(crate) fn avatar_tile(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    radius: f32,
    name: &str,
    url: Option<&str>,
    metrics: UiMetrics,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(radius * 2.0), Sense::hover());
    avatar(
        ui.painter(),
        assets,
        rect.center(),
        radius,
        name,
        url,
        metrics,
    );
}

pub(crate) fn avatar_button(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    radius: f32,
    name: &str,
    url: Option<&str>,
    hover_label: &str,
    metrics: UiMetrics,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(radius * 2.0), Sense::click());
    avatar(
        ui.painter(),
        assets,
        rect.center(),
        radius,
        name,
        url,
        metrics,
    );
    if response.hovered() {
        ui.painter()
            .circle_filled(rect.center(), radius, shade(Shade::Faint));
        draw_text(
            ui.painter(),
            rect.center(),
            Align2::CENTER_CENTER,
            hover_label,
            FontId::proportional(metrics.text.label),
            text_primary(),
            None,
        );
    }
    response
}
