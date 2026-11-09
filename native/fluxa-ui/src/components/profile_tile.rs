use super::*;
use crate::IconSize;
use crate::Space;
use crate::dp::Dp;
use crate::flex::{self, Insets};

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

const AVATAR: usize = 0;
const NAME: usize = 1;
const BADGE: usize = 2;
const ACTIONS: usize = 3;

fn lay_out_tile(
    ui: &mut Ui,
    cell: Vec2,
    radius: f32,
    badge: bool,
    metrics: UiMetrics,
) -> TileSlots {
    let card = ui.max_rect();
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    let band = |id: usize, height: f32| flex::leaf(Vec2::new(cell.x, height)).id(id);
    let gap = |height: f32| flex::leaf(Vec2::new(cell.x, height));
    let mut bands = vec![
        band(AVATAR, radius * 2.0),
        gap(Space::S12.px(&metrics)),
        band(NAME, Space::S24.px(&metrics)),
    ];
    if badge {
        bands.push(band(BADGE, Space::S20.px(&metrics)));
    }
    bands.push(gap(Space::S4.px(&metrics)));
    bands.push(band(ACTIONS, metrics.profile_action_size));
    let column = flex::column(bands);
    let (area, _) = ui.allocate_exact_size(column.outer_size(), Sense::hover());
    let placed = column.solve(area);
    let slot = |id: usize| placed.find(id).unwrap_or(area);
    let avatar = slot(AVATAR);
    let disc = Rect::from_center_size(avatar.center(), Vec2::splat(radius * 2.0));
    let below = Rect::from_min_size(avatar.left_bottom(), Vec2::new(cell.x, Dp::D40.px()));
    TileSlots {
        card,
        avatar: disc,
        name: slot(NAME),
        badge: placed.find(BADGE),
        actions: slot(ACTIONS),
        hit: below.union(disc),
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
    let (title_rect, _) = ui.allocate_exact_size(Vec2::new(width, Dp::D44.px()), Sense::hover());
    let heading = flex::stack(Align2::CENTER_CENTER, [flex::leaf(title.size()).id(0)])
        .solve(title_rect)
        .find(0)
        .unwrap_or(title_rect);
    ui.painter().galley(heading.min, title, text_primary());
    let beside = Rect::from_min_max(
        Pos2::new(heading.right(), title_rect.top()),
        title_rect.right_bottom(),
    );
    let gear_rect = flex::stack(
        Align2::LEFT_CENTER,
        [flex::stack(
            Align2::CENTER_CENTER,
            [flex::leaf(Vec2::splat(metrics.icon_button_size)).id(0)],
        )
        .width(Dp::D36.px() * 2.0)],
    )
    .solve(beside);
    let gear_rect = gear_rect.find(0).unwrap_or(beside);
    let gear = ui
        .interact(gear_rect, Id::new("fluxa-profiles-gear"), Sense::click())
        .on_hover_text(gear_tip);
    if let Some(icon) = assets.icon("Settings") {
        let glyph = flex::anchored(
            gear_rect,
            Align2::CENTER_CENTER,
            Vec2::splat(Dp::D24.px()),
            0.0,
        );
        ui.painter().image(
            icon,
            glyph,
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
    let floor = metrics.control_profile_reveal_floor;
    let grow = floor + (1.0 - floor) * reveal;
    let r = radius
        * grow
        * if response.hovered() && active {
            metrics.control_profile_hover_scale
        } else {
            1.0
        };
    ui.painter().circle(
        avatar_center,
        r,
        ink(fade(if response.hovered() { 26 } else { 10 }, reveal)),
        crate::gfx::Stroke::new(stroke::hairline(), metrics.border.gamma_multiply(reveal)),
    );
    if let Some(icon) = assets.icon("Plus") {
        ui.painter().image(
            icon,
            flex::anchored(
                avatar_rect,
                Align2::CENTER_CENTER,
                Vec2::splat(IconSize::Xl.px(&metrics) * grow),
                0.0,
            ),
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
        radius * metrics.control_profile_hover_scale
    } else {
        radius
    };
    avatar_sized(
        ui.painter(),
        assets,
        avatar_center,
        r,
        radius * metrics.control_profile_hover_scale,
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
        let lock = Dp::D16.px();
        let badge = avatar_center + Vec2::splat(radius * metrics.control_profile_lock_ratio);
        ui.painter().circle(
            badge,
            lock,
            background().gamma_multiply(0.92),
            crate::gfx::Stroke::new(stroke::hairline(), crate::stylesheet::color("raised")),
        );
        if let Some(icon) = assets.icon("Lock") {
            ui.painter().image(
                icon,
                flex::anchored(
                    Rect::from_center_size(badge, Vec2::splat(lock * 2.0)),
                    Align2::CENTER_CENTER,
                    Vec2::splat(IconSize::Sm.px(&metrics)),
                    0.0,
                ),
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
        let slot = slots.badge.unwrap_or(slots.avatar);
        let chip = flex::stack(
            Align2::CENTER_CENTER,
            [
                flex::stack(Align2::CENTER_CENTER, [flex::leaf(galley.size()).id(1)])
                    .padding(Insets::xy(Dp::D6.px(), Dp::D3.px()))
                    .id(0),
            ],
        )
        .solve(slot);
        let badge = chip.find(0).unwrap_or(slot);
        let label_rect = chip.find(1).unwrap_or(badge);
        ui.painter()
            .rect_filled(badge, radius::chip(), crate::stylesheet::color("track"));
        ui.painter().galley(label_rect.min, galley, text_primary());
    }
    let alpha = if card_hovered { 230 } else { 120 };
    let spread = Dp::D16.px();
    let actions: &[(f32, &str, bool)] = if reveal > 0.001 {
        &[(-spread, "Edit", false), (spread, "Delete", true)]
    } else {
        &[]
    };
    let mut chosen = None;
    for &(offset, icon, is_delete) in actions {
        let label = if is_delete { tile.tips.1 } else { tile.tips.0 };
        let rect = Rect::from_center_size(
            Pos2::new(
                avatar_center.x + offset,
                slots.actions.center().y + (1.0 - reveal) * Dp::D10.px(),
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
                flex::anchored(
                    rect,
                    Align2::CENTER_CENTER,
                    Vec2::splat(IconSize::Sm.px(&metrics)),
                    0.0,
                ),
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
