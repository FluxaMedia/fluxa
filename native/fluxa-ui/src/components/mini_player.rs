use super::*;
use crate::nodes::{NODE_PLAYER_CLOSE, NODE_PLAYER_EXPAND, NODE_PLAYER_TOGGLE};
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
) -> Response {
    painter.rect_filled(
        tile.expand(1.0),
        crate::style::radius::control(),
        crate::ink(crate::style::tone::raised()),
    );
    painter.rect_filled(tile, crate::style::radius::control(), Color32::BLACK);
    match player.video {
        Some(texture) => rounded_image(
            painter,
            texture,
            tile,
            7.0,
            crate::paint::full_uv(),
            Color32::WHITE,
        ),
        None => {}
    }
    let hit = ui.interact(
        tile,
        Id::new(("fluxa-mini-player", expand)),
        Sense::click_and_drag(),
    );
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
        return hit;
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
        (
            -40.0,
            if player.paused { "play" } else { "pause" },
            NODE_PLAYER_TOGGLE,
        ),
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
    hit
}

const MINI_RECT: &str = "fluxa-mini-player-rect";

pub(crate) fn over_mini_player(context: &egui::Context, pos: Pos2) -> bool {
    let now = context.cumulative_pass_nr();
    context
        .data(|data| data.get_temp::<(u64, Rect)>(Id::new(MINI_RECT)))
        .is_some_and(|(pass, rect)| now.saturating_sub(pass) <= 2 && rect.contains(pos))
}

#[derive(Clone, Copy)]
struct MiniState {
    corner: usize,
    pos: Pos2,
}

pub fn draw_mini_player_overlay(
    context: &egui::Context,
    viewport: Viewport,
    player: &PlayerModel,
    layout: &mut HomeLayout,
) {
    let width = (viewport.width * 0.22).clamp(280.0, 400.0);
    let size = Vec2::new(width, width * 9.0 / 16.0);
    let (side, top) = (24.0, 80.0);
    let bottom = 24.0 + viewport.safe_bottom;
    let corners = [
        Pos2::new(side, top),
        Pos2::new(viewport.width - side - size.x, top),
        Pos2::new(side, viewport.height - bottom - size.y),
        Pos2::new(
            viewport.width - side - size.x,
            viewport.height - bottom - size.y,
        ),
    ];
    let id = Id::new("fluxa-mini-player-state");
    let mut state = context
        .data(|data| data.get_temp::<MiniState>(id))
        .unwrap_or(MiniState {
            corner: 3,
            pos: corners[3],
        });
    let pass = context.cumulative_pass_nr();
    context.data_mut(|data| {
        data.insert_temp(
            Id::new(MINI_RECT),
            (pass, Rect::from_min_size(state.pos, size)),
        )
    });
    egui::Area::new(Id::new("fluxa-mini-player-area"))
        .order(egui::Order::Foreground)
        .fixed_pos(state.pos)
        .interactable(true)
        .show(context, |ui| {
            ui.set_min_size(size);
            let painter = ui.painter().clone();
            let tile = Rect::from_min_size(state.pos, size);
            let hit = mini_player(ui, &painter, tile, player, layout, NODE_PLAYER_EXPAND);
            if hit.dragged() {
                state.pos += hit.drag_delta();
                let max = Pos2::new(viewport.width - size.x, viewport.height - size.y);
                state.pos = state.pos.clamp(Pos2::ZERO, max);
                state.corner = nearest(&corners, state.pos);
            } else {
                let target = corners[state.corner.min(3)];
                let step = 1.0 - (-ui.ctx().input(|input| input.stable_dt) * 18.0).exp();
                let next = state.pos + (target - state.pos) * step;
                if (target - next).length() < 0.5 {
                    state.pos = target;
                } else {
                    state.pos = next;
                    ui.ctx().request_repaint();
                }
            }
        });
    context.data_mut(|data| data.insert_temp(id, state));
}

fn nearest(corners: &[Pos2; 4], pos: Pos2) -> usize {
    (0..4)
        .min_by(|a, b| {
            let da = (corners[*a] - pos).length_sq();
            let db = (corners[*b] - pos).length_sq();
            da.total_cmp(&db)
        })
        .unwrap_or(3)
}
