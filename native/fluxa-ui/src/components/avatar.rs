use super::*;
use crate::Space;

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
    metrics: UiMetrics,
) {
    avatar_sized(painter, assets, center, radius, radius, _name, url, metrics);
}

pub(crate) fn avatar_sized(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    center: crate::gfx::Pos2,
    radius: f32,
    texture_radius: f32,
    _name: &str,
    url: Option<&str>,
    _metrics: UiMetrics,
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

pub(crate) fn avatar_grid(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    pack_id: &str,
    avatars: &[crate::ProfileAvatar],
    selected: Option<&str>,
    metrics: UiMetrics,
) -> Option<usize> {
    let tile = metrics.profile_avatar_tile;
    let gap = Space::S14.px(&metrics);
    let label = Space::S20.px(&metrics) + Space::S2.px(&metrics);
    let row_gap = Space::S8.px(&metrics);
    let mut chosen = None;
    crate::kit::grid(avatars.len())
        .cell(Vec2::new(tile, tile + label))
        .gap_px(gap)
        .row_gap_px(row_gap)
        .start()
        .show(ui, |ui, index| {
            let item = &avatars[index];
            let rect = ui.max_rect();
            let response = ui.interact(
                rect,
                Id::new(("fluxa-avatar", pack_id, index)),
                Sense::click(),
            );
            let center = Pos2::new(rect.center().x, rect.top() + tile * 0.5);
            avatar(
                ui.painter(),
                assets,
                center,
                tile * 0.5,
                &item.name,
                Some(&item.url),
                metrics,
            );
            let picked = selected == Some(item.url.as_str());
            if picked || response.hovered() {
                ui.painter().circle_stroke(
                    center,
                    tile * 0.5 + 2.0,
                    crate::gfx::Stroke::new(stroke::emphasis(), ink(if picked { 255 } else { 90 })),
                );
            }
            draw_text(
                ui.painter(),
                Pos2::new(center.x, rect.bottom() - 8.0),
                Align2::CENTER_CENTER,
                &item.name,
                FontId::proportional(metrics.text.label),
                metrics.text_secondary,
                Some(tile + gap - 4.0),
            );
            if response.clicked() {
                chosen = Some(index);
            }
        });
    chosen
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PackAction {
    Refresh,
    Remove,
}

pub(crate) fn pack_row(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    compact: bool,
    pack: &crate::ProfileAvatarPack,
    busy: bool,
    tips: (&str, &str),
    metrics: UiMetrics,
) -> Option<PackAction> {
    let mut action = None;
    let height = Space::S48.px(&metrics) + Space::S12.px(&metrics) * 2.0;
    let row = crate::kit::view()
        .row()
        .height(height)
        .padding(Space::S12)
        .gap(Space::S8);
    let row = if compact {
        row
    } else {
        row.class("avatar-pack")
    };
    row.show(ui, |ui| {
        let previews = if compact {
            0
        } else {
            pack.avatars.len().min(6)
        };
        let preview_width = if previews > 0 {
            24.0 + previews as f32 * 32.0
        } else {
            0.0
        };
        let (rect, _) = ui.allocate_exact_size(Vec2::new(preview_width, 48.0), Sense::hover());
        for (index, item) in pack.avatars.iter().take(previews).enumerate() {
            let center = Pos2::new(rect.left() + 24.0 + index as f32 * 32.0, rect.center().y);
            ui.painter()
                .circle_filled(center, 24.5, crate::surface_color());
            avatar(
                ui.painter(),
                assets,
                center,
                23.0,
                &item.name,
                Some(&item.url),
                metrics,
            );
        }
        let button = 32.0;
        let buttons = button * 2.0 + Space::S8.px(&metrics);
        let label_width = (ui.available_width() - buttons).max(0.0);
        crate::kit::view()
            .row()
            .max_width(label_width)
            .height(button)
            .show(ui, |ui| {
                ui.add(
                    crate::gfx::Label::new(
                        RichText::new(format!("{} · {}", pack.title, pack.avatars.len()))
                            .size(metrics.text.body)
                            .color(metrics.text_primary),
                    )
                    .truncate(),
                );
            });
        let refresh = icon_button(
            ui,
            assets.icon("Refresh"),
            button,
            crate::stylesheet::color("text.subtle"),
            false,
            false,
            !busy,
        );
        if refresh.on_hover_text(tips.1).clicked() {
            action = Some(PackAction::Refresh);
        }
        let remove = icon_button(
            ui,
            assets.icon("Delete"),
            button,
            danger_color(),
            false,
            false,
            true,
        );
        if remove.on_hover_text(tips.0).clicked() {
            action = Some(PackAction::Remove);
        }
    });
    action
}
