use super::*;
use crate::Space;

pub(crate) struct ProfileTile<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub avatar_url: Option<&'a str>,
    pub locked: bool,
    pub primary_label: Option<String>,
    pub cell: Vec2,
    pub radius: f32,
    pub manage: bool,
    pub reveal: f32,
    pub tips: (&'a str, &'a str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TileAction {
    Edit,
    Delete,
}

struct TileSlots {
    card: Rect,
    avatar: Rect,
    name: Rect,
    badge: Option<Rect>,
    actions: Rect,
    hit: Rect,
}

fn lay_out_tile(
    ui: &mut Ui,
    cell: Vec2,
    radius: f32,
    badge: bool,
    metrics: UiMetrics,
) -> TileSlots {
    let card = ui.max_rect();
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    let mut slot = |ui: &mut Ui, height: f32| {
        ui.allocate_exact_size(Vec2::new(cell.x, height), Sense::hover())
            .0
    };
    let avatar = slot(ui, radius * 2.0);
    slot(ui, Space::S12.px(&metrics));
    let name = slot(ui, Space::S24.px(&metrics));
    let badge = badge.then(|| slot(ui, Space::S20.px(&metrics)));
    slot(ui, Space::S4.px(&metrics));
    let actions = slot(ui, metrics.profile_action_size);
    let hit = Rect::from_min_size(avatar.left_bottom(), Vec2::new(cell.x, 40.0)).union(
        Rect::from_center_size(avatar.center(), Vec2::splat(radius * 2.0)),
    );
    TileSlots {
        card,
        avatar: Rect::from_center_size(avatar.center(), Vec2::splat(radius * 2.0)),
        name,
        badge,
        actions,
        hit,
    }
}

fn fade(alpha: u8, reveal: f32) -> u8 {
    (f32::from(alpha) * reveal) as u8
}

pub(crate) fn picker_title(
    ui: &mut Ui,
    assets: &impl HomeAssets,
    width: f32,
    text: String,
    gear_tip: &str,
    metrics: UiMetrics,
) -> Response {
    let title = ui.painter().layout_no_wrap(
        text,
        crate::fonts::bold(metrics.profile_title_size),
        text_primary(),
    );
    let (title_rect, _) = ui.allocate_exact_size(Vec2::new(width, 46.0), Sense::hover());
    let title_pos = Pos2::new(
        title_rect.center().x - title.size().x * 0.5,
        title_rect.center().y - title.size().y * 0.5,
    );
    let title_right = title_pos.x + title.size().x;
    ui.painter().galley(title_pos, title, text_primary());
    let gear_rect = Rect::from_center_size(
        Pos2::new(title_right + 38.0, title_rect.center().y),
        Vec2::splat(metrics.icon_button_size),
    );
    let gear = ui
        .interact(gear_rect, Id::new("fluxa-profiles-gear"), Sense::click())
        .on_hover_text(gear_tip);
    if let Some(icon) = assets.icon("Settings") {
        ui.painter().image(
            icon,
            Rect::from_center_size(gear_rect.center(), Vec2::splat(26.0)),
            full_uv(),
            Color32::from_white_alpha(if gear.hovered() { 255 } else { 180 }),
        );
    }
    gear
}

pub(crate) fn add_tile(
    ui: &mut Ui,
    assets: &impl HomeAssets,
    cell: Vec2,
    radius: f32,
    active: bool,
    reveal: f32,
    label: String,
    on_interact: impl FnOnce(&mut Ui, &Response, Rect),
    metrics: UiMetrics,
) {
    let slots = lay_out_tile(ui, cell, radius, false, metrics);
    let avatar_center = slots.avatar.center();
    let avatar_rect = slots.avatar;
    let response = ui.interact(
        slots.hit,
        Id::new("fluxa-profiles-add"),
        if active {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    on_interact(ui, &response, avatar_rect);
    let grow = 0.88 + 0.12 * reveal;
    let r = radius
        * grow
        * if response.hovered() && active {
            1.04
        } else {
            1.0
        };
    ui.painter().circle(
        avatar_center,
        r,
        ink(fade(if response.hovered() { 26 } else { 10 }, reveal)),
        egui::Stroke::new(stroke::hairline(), metrics.border.gamma_multiply(reveal)),
    );
    if let Some(icon) = assets.icon("Plus") {
        ui.painter().image(
            icon,
            Rect::from_center_size(avatar_center, Vec2::splat(36.0 * grow)),
            full_uv(),
            crate::stylesheet::color("glyph").gamma_multiply(reveal),
        );
    }
    draw_text(
        ui.painter(),
        slots.name.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(metrics.text.body),
        metrics.text_muted.gamma_multiply(reveal),
        None,
    );
}

pub(crate) fn profile_tile(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    tile: &ProfileTile,
    on_interact: impl FnOnce(&mut Ui, &Response, Rect),
    metrics: UiMetrics,
) -> Option<TileAction> {
    let ProfileTile {
        cell,
        radius,
        manage,
        reveal,
        ..
    } = *tile;
    let slots = lay_out_tile(ui, cell, radius, tile.primary_label.is_some(), metrics);
    let avatar_center = slots.avatar.center();
    let avatar_rect = slots.avatar;
    let response = ui.interact(
        slots.hit,
        Id::new(("fluxa-profile", tile.id)),
        Sense::click(),
    );
    on_interact(ui, &response, avatar_rect);
    let card_hovered = ui.rect_contains_pointer(slots.card);
    let r = if response.hovered() {
        radius * 1.04
    } else {
        radius
    };
    avatar_sized(
        ui.painter(),
        assets,
        avatar_center,
        r,
        radius * 1.04,
        tile.name,
        tile.avatar_url,
        metrics,
    );
    if !response.hovered() {
        ui.painter().circle_filled(
            avatar_center,
            r,
            Color32::from_black_alpha(30 + fade(70, reveal)),
        );
    }
    if tile.locked {
        let badge = avatar_center + Vec2::splat(radius * 0.72);
        ui.painter().circle(
            badge,
            17.0,
            background().gamma_multiply(0.92),
            egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("raised")),
        );
        if let Some(icon) = assets.icon("Lock") {
            ui.painter().image(
                icon,
                Rect::from_center_size(badge, Vec2::splat(16.0)),
                full_uv(),
                crate::stylesheet::color("text.body"),
            );
        }
    }
    let name_font = FontId::proportional(metrics.text.subtitle);
    let name = truncate_to_width(ui.painter(), tile.name, &name_font, cell.x);
    let name_y = slots.name.center().y;
    draw_text(
        ui.painter(),
        Pos2::new(avatar_center.x, name_y),
        Align2::CENTER_CENTER,
        name,
        name_font,
        ink(if response.hovered() { 255 } else { 215 }),
        None,
    );
    if let Some(label) = tile.primary_label.clone() {
        let galley = ui.painter().layout_no_wrap(
            label,
            FontId::proportional(metrics.text.label),
            metrics.text_secondary,
        );
        let badge = Rect::from_center_size(
            slots.badge.map_or(avatar_center, |slot| slot.center()),
            galley.size() + Vec2::new(14.0, 6.0),
        );
        ui.painter()
            .rect_filled(badge, radius::chip(), crate::stylesheet::color("track"));
        ui.painter()
            .galley(badge.center() - galley.size() * 0.5, galley, text_primary());
    }
    let alpha = if card_hovered { 230 } else { 120 };
    let actions: &[(f32, &str, bool)] = if reveal > 0.001 {
        &[(-16.0, "Edit", false), (16.0, "Delete", true)]
    } else {
        &[]
    };
    let mut chosen = None;
    for &(offset, icon, is_delete) in actions {
        let label = if is_delete { tile.tips.1 } else { tile.tips.0 };
        let rect = Rect::from_center_size(
            Pos2::new(
                avatar_center.x + offset,
                slots.actions.center().y + (1.0 - reveal) * 10.0,
            ),
            Vec2::splat(metrics.profile_action_size),
        );
        let action = ui
            .interact(
                rect,
                Id::new(("fluxa-profile-action", tile.id, is_delete)),
                if manage {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            )
            .on_hover_text(label);
        if action.hovered() && manage {
            ui.painter()
                .rect_filled(rect, radius::control(), crate::stylesheet::color("track"));
        }
        let color = ink(fade(if action.hovered() { 255 } else { alpha }, reveal));
        if let Some(texture) = assets.icon(icon) {
            ui.painter().image(
                texture,
                Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
                full_uv(),
                color,
            );
        }
        if manage && action.clicked() {
            chosen = Some(if is_delete {
                TileAction::Delete
            } else {
                TileAction::Edit
            });
        }
    }
    chosen
}
