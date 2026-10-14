use super::*;
use crate::nodes::{
    NODE_PLAYER_CLOSE, NODE_PLAYER_EXPAND, NODE_PLAYER_TOGGLE,
};
use crate::paint::paint_vertical_gradient;
use crate::player::{PlayerModel, control};
use crate::{HomeLayout, Viewport};

pub(crate) fn mini_player(
    ui: &mut Ui,
    painter: &Painter,
    tile: Rect,
    player: &PlayerModel,
    layout: &mut HomeLayout,
    expand: u64,
) {
    painter.rect_filled(tile.expand(1.0), crate::style::radius::control(), crate::ink(crate::style::tone::RAISED));
    match player.video {
        Some(texture) => rounded_image(painter, texture, tile, 7.0, crate::paint::full_uv(), Color32::WHITE),
        None => {
            painter.rect_filled(tile, crate::style::radius::control(), Color32::BLACK);
        }
    }
    let hit = ui.interact(tile, Id::new(("fluxa-mini-player", expand)), Sense::click());
    let hovered = hit.hovered() || ui.rect_contains_pointer(tile);
    let shown = crate::motion::toggle(
        ui.ctx(),
        Id::new("fluxa-player-mini-controls"),
        hovered,
        crate::motion::normal(),
    );
    if shown <= 0.0 {
        if hit.clicked() {
            layout.activated = Some(expand);
        }
        return;
    }
    let strip = Rect::from_min_max(Pos2::new(tile.left(), tile.bottom() - 46.0), tile.max);
    paint_vertical_gradient(
        painter,
        strip,
        Color32::TRANSPARENT,
        Color32::from_black_alpha((215.0 * shown) as u8),
    );
    let y = strip.center().y + 4.0;
    for (offset, icon, node) in [
        (-40.0, if player.paused { "play" } else { "pause" }, NODE_PLAYER_TOGGLE),
        (0.0, "fullscreen", expand),
        (40.0, "close", NODE_PLAYER_CLOSE),
    ] {
        let button = control(
            ui,
            painter,
            Rect::from_center_size(Pos2::new(tile.center().x + offset, y), Vec2::splat(32.0)),
            icon,
            false,
        );
        focus_action(layout, node, &button);
    }
    if player.duration > 0.0 {
        let ratio = (player.position / player.duration).clamp(0.0, 1.0) as f32;
        let bar = Rect::from_min_max(Pos2::new(tile.left(), tile.bottom() - 3.0), tile.max);
        painter.rect_filled(bar, 0.0, Color32::from_white_alpha(60));
        painter.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new(bar.width() * ratio, bar.height())),
            0.0,
            Color32::WHITE,
        );
    }
}

pub fn draw_mini_player_overlay(
    context: &egui::Context,
    viewport: Viewport,
    player: &PlayerModel,
    layout: &mut HomeLayout,
) {
    let width = (viewport.width * 0.22).clamp(280.0, 400.0);
    let size = Vec2::new(width, width * 9.0 / 16.0);
    let margin = 24.0;
    let tile = Rect::from_min_size(
        Pos2::new(
            viewport.width - margin - size.x,
            viewport.height - margin - size.y - viewport.safe_bottom,
        ),
        size,
    );
    egui::Area::new(Id::new("fluxa-mini-player-area"))
        .order(egui::Order::Foreground)
        .fixed_pos(Pos2::ZERO)
        .interactable(true)
        .show(context, |ui| {
            let painter = ui.painter().clone();
            mini_player(ui, &painter, tile, player, layout, NODE_PLAYER_EXPAND);
        });
}
