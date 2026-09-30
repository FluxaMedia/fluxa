use super::*;

pub(super) fn painter_bar(
    painter: &egui::Painter,
    track: Rect,
    thickness: f32,
    color: Color32,
    width: f32,
) {
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(track.left(), track.center().y - thickness * 0.5),
            Vec2::new(width, thickness),
        ),
        thickness * 0.5,
        color,
    );
}

pub(super) fn desktop_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    let margin = chrome.margin();
    let header_y = rect.top() + 30.0;
    let close_center = Pos2::new(margin + 20.0, header_y);
    chrome.button(ui, layout, close_center, 42.0, "close", NODE_PLAYER_CLOSE);
    if !player.has_video() || !player.controls_visible {
        if player.has_video() {
            chrome.surface(ui, layout, rect);
        }
        return;
    }
    scrims(chrome.painter, rect);
    let title_x = close_center.x + 31.0;
    chrome.text(
        Pos2::new(title_x, header_y),
        Align2::LEFT_CENTER,
        &player.title,
        19.0,
        rect.width() - title_x - margin,
        255,
    );
    let track = Rect::from_min_size(
        Pos2::new(margin, rect.bottom() - 93.0 - chrome.viewport.safe_bottom),
        Vec2::new((rect.width() - margin * 2.0).max(80.0), 16.0),
    );
    chrome.surface(
        ui,
        layout,
        Rect::from_min_max(
            Pos2::new(rect.left(), header_y + 29.0),
            Pos2::new(rect.right(), track.top() - 8.0),
        ),
    );
    let position = chrome.seek_bar(ui, layout, track, 3.0);
    let y = rect.bottom() - 48.0 - chrome.viewport.safe_bottom;
    let play = if player.paused { "play" } else { "pause" };
    chrome.button(
        ui,
        layout,
        Pos2::new(margin + 23.0, y),
        48.0,
        play,
        NODE_PLAYER_TOGGLE,
    );
    chrome.button(
        ui,
        layout,
        Pos2::new(margin + 84.0, y),
        42.0,
        "back",
        NODE_PLAYER_REWIND,
    );
    chrome.button(
        ui,
        layout,
        Pos2::new(margin + 132.0, y),
        42.0,
        "forward",
        NODE_PLAYER_FORWARD,
    );
    let right = rect.right() - margin - 20.0;
    let volume_x = right - 52.0;
    let upscaling = Rect::from_center_size(Pos2::new(volume_x - 138.0, y), Vec2::new(150.0, 34.0));
    chrome.button(
        ui,
        layout,
        Pos2::new(upscaling.left() - 28.0, y),
        38.0,
        "cast",
        NODE_PLAYER_CAST,
    );
    chrome.button(
        ui,
        layout,
        Pos2::new(upscaling.left() - 74.0, y),
        38.0,
        "submit",
        NODE_PLAYER_SUBMIT,
    );
    chrome.text(
        Pos2::new(margin + 158.0, y),
        Align2::LEFT_CENTER,
        &chrome.times(position),
        13.0,
        (upscaling.left() - margin - 250.0).max(0.0),
        218,
    );
    chrome.button(
        ui,
        layout,
        Pos2::new(right, y),
        42.0,
        "fullscreen",
        NODE_PLAYER_FULLSCREEN,
    );
    let mute = if player.muted { "mute" } else { "volume" };
    chrome.button(
        ui,
        layout,
        Pos2::new(volume_x, y),
        42.0,
        mute,
        NODE_PLAYER_MUTE,
    );
    let volume = if player.muted {
        localized("player.muted", &player.language)
    } else {
        format!("{}%", player.volume.round() as i32)
    };
    chrome.text(
        Pos2::new(volume_x - 18.0, y),
        Align2::RIGHT_CENTER,
        &volume,
        12.0,
        60.0,
        170,
    );
    chrome.upscaling(ui, layout, upscaling, 13.0);
}

pub(super) fn mobile_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    let margin = chrome.margin();
    let header_y = rect.top() + 38.0;
    let close_center = Pos2::new(margin + 20.0, header_y);
    if !player.has_video() {
        chrome.button(ui, layout, close_center, 42.0, "close", NODE_PLAYER_CLOSE);
        return;
    }
    chrome.surface(ui, layout, rect);
    if !player.controls_visible {
        return;
    }
    scrims(chrome.painter, rect);
    chrome.button(ui, layout, close_center, 42.0, "close", NODE_PLAYER_CLOSE);
    let upscaling = Rect::from_min_size(
        Pos2::new(rect.right() - margin - 132.0, header_y - 16.0),
        Vec2::new(132.0, 32.0),
    );
    chrome.upscaling(ui, layout, upscaling, 12.0);
    chrome.button(
        ui,
        layout,
        Pos2::new(upscaling.left() - 24.0, header_y),
        40.0,
        "cast",
        NODE_PLAYER_CAST,
    );
    chrome.button(
        ui,
        layout,
        Pos2::new(upscaling.left() - 68.0, header_y),
        40.0,
        "submit",
        NODE_PLAYER_SUBMIT,
    );
    let title_x = close_center.x + 31.0;
    chrome.text(
        Pos2::new(title_x, header_y),
        Align2::LEFT_CENTER,
        &player.title,
        17.0,
        upscaling.left() - title_x - 92.0,
        255,
    );
    let center = rect.center();
    let gap = (rect.width() * 0.26).min(150.0);
    let play = if player.paused { "play" } else { "pause" };
    chrome.button(
        ui,
        layout,
        center - Vec2::new(gap, 0.0),
        56.0,
        "back",
        NODE_PLAYER_REWIND,
    );
    chrome.button(ui, layout, center, 76.0, play, NODE_PLAYER_TOGGLE);
    chrome.button(
        ui,
        layout,
        center + Vec2::new(gap, 0.0),
        56.0,
        "forward",
        NODE_PLAYER_FORWARD,
    );
    let track = Rect::from_min_size(
        Pos2::new(margin, rect.bottom() - 44.0 - chrome.viewport.safe_bottom),
        Vec2::new((rect.width() - margin * 2.0).max(80.0), 20.0),
    );
    let position = chrome.seek_bar(ui, layout, track, 3.0);
    let label_y = track.top() - 8.0;
    chrome.text(
        Pos2::new(track.left(), label_y),
        Align2::LEFT_BOTTOM,
        &format_time(position),
        13.0,
        120.0,
        218,
    );
    chrome.text(
        Pos2::new(track.right(), label_y),
        Align2::RIGHT_BOTTOM,
        &format_time(player.duration.max(0.0)),
        13.0,
        120.0,
        218,
    );
}

pub(super) fn tv_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    if !player.has_video() || !player.controls_visible {
        return;
    }
    scrims(chrome.painter, rect);
    let margin = (rect.width() * 0.05).max(48.0);
    let y = rect.bottom() - 64.0 - chrome.viewport.safe_bottom;
    let track = Rect::from_min_size(
        Pos2::new(margin, y - 72.0),
        Vec2::new(rect.width() - margin * 2.0, 20.0),
    );
    let position = chrome.seek_bar(ui, layout, track, 4.0);
    let width = track.width() * 0.7;
    chrome.text(
        Pos2::new(margin, track.top() - 46.0),
        Align2::LEFT_BOTTOM,
        &player.title,
        28.0,
        width,
        255,
    );
    if let Some(episode) = player.episode_title.as_deref() {
        chrome.text(
            Pos2::new(margin, track.top() - 16.0),
            Align2::LEFT_BOTTOM,
            episode,
            17.0,
            width,
            200,
        );
    }
    chrome.text(
        Pos2::new(track.right(), track.top() - 16.0),
        Align2::RIGHT_BOTTOM,
        &chrome.times(position),
        17.0,
        track.width() * 0.3,
        218,
    );
    let play = if player.paused { "play" } else { "pause" };
    chrome.button(
        ui,
        layout,
        Pos2::new(margin + 28.0, y),
        56.0,
        play,
        NODE_PLAYER_TOGGLE,
    );
    chrome.button(
        ui,
        layout,
        Pos2::new(margin + 96.0, y),
        52.0,
        "back",
        NODE_PLAYER_REWIND,
    );
    chrome.button(
        ui,
        layout,
        Pos2::new(margin + 160.0, y),
        52.0,
        "forward",
        NODE_PLAYER_FORWARD,
    );
    let upscaling =
        Rect::from_min_size(Pos2::new(margin + 204.0, y - 22.0), Vec2::new(190.0, 44.0));
    chrome.upscaling(ui, layout, upscaling, 16.0);
    chrome.button(
        ui,
        layout,
        Pos2::new(upscaling.right() + 40.0, y),
        52.0,
        "cast",
        NODE_PLAYER_CAST,
    );
    chrome.button(
        ui,
        layout,
        Pos2::new(upscaling.right() + 104.0, y),
        52.0,
        "submit",
        NODE_PLAYER_SUBMIT,
    );
}

pub(super) fn draw_focus_ring(context: &egui::Context, layout: &HomeLayout, focused: Option<u64>) {
    if let Some((_, rect)) = layout.focusable.iter().find(|(id, _)| Some(*id) == focused) {
        context
            .layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                Id::new("fluxa-player-focus"),
            ))
            .rect_stroke(
                rect.expand(3.0),
                rect.height().min(rect.width()) * 0.5,
                egui::Stroke::new(2.0, Color32::WHITE),
                egui::StrokeKind::Outside,
            );
    }
}

pub(super) fn control(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    icon: &str,
    prominent: bool,
) -> egui::Response {
    let response = ui.interact(rect, Id::new(("fluxa-player-button", icon)), Sense::click());
    if prominent || response.hovered() || response.is_pointer_button_down_on() {
        crate::components::glass(
            painter,
            rect,
            rect.width() * 0.5,
            if prominent {
                Color32::from_white_alpha(30)
            } else {
                Color32::from_black_alpha(75)
            },
        );
    }
    let color = if response.hovered() {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(235)
    };
    let c = rect.center();
    let s = (rect.width() / 48.0).max(1.0);
    let stroke = egui::Stroke::new(2.0, color);
    match icon {
        "play" => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    c + s * Vec2::new(-5.0, -8.0),
                    c + s * Vec2::new(8.0, 0.0),
                    c + s * Vec2::new(-5.0, 8.0),
                ],
                color,
                egui::Stroke::NONE,
            ));
        }
        "pause" => {
            for offset in [-4.0, 4.0] {
                painter.rect_filled(
                    Rect::from_center_size(
                        c + s * Vec2::new(offset, 0.0),
                        s * Vec2::new(4.0, 17.0),
                    ),
                    1.0,
                    color,
                );
            }
        }
        "close" => {
            painter.line_segment(
                [c + s * Vec2::new(-6.0, -6.0), c + s * Vec2::new(6.0, 6.0)],
                stroke,
            );
            painter.line_segment(
                [c + s * Vec2::new(6.0, -6.0), c + s * Vec2::new(-6.0, 6.0)],
                stroke,
            );
        }
        "back" | "forward" => {
            let sign = if icon == "back" { -1.0 } else { 1.0 };
            let center = c + s * Vec2::new(-4.0 * sign, -2.0);
            painter.add(egui::Shape::line(
                (0..=20)
                    .map(|step| {
                        let angle = 0.25 + (5.2 - 0.25) * step as f32 / 20.0;
                        center + Vec2::new(angle.cos() * 8.0 * s, angle.sin() * 8.0 * s)
                    })
                    .collect(),
                egui::Stroke::new(1.8 * s, color),
            ));
            painter.text(
                c + s * Vec2::new(0.0, 7.0),
                Align2::CENTER_CENTER,
                "10",
                FontId::proportional(9.0 * s),
                color,
            );
        }
        "volume" | "mute" => {
            painter.rect_filled(
                Rect::from_min_max(c + s * Vec2::new(-9.0, -4.0), c + s * Vec2::new(-5.0, 4.0)),
                0.5,
                color,
            );
            painter.add(egui::Shape::convex_polygon(
                vec![
                    c + s * Vec2::new(-5.0, -5.0),
                    c + s * Vec2::new(2.0, -10.0),
                    c + s * Vec2::new(2.0, 10.0),
                    c + s * Vec2::new(-5.0, 5.0),
                ],
                color,
                egui::Stroke::NONE,
            ));
            if icon == "mute" {
                let thin = egui::Stroke::new(1.8, color);
                painter.line_segment(
                    [c + s * Vec2::new(5.0, -5.0), c + s * Vec2::new(11.0, 5.0)],
                    thin,
                );
                painter.line_segment(
                    [c + s * Vec2::new(11.0, -5.0), c + s * Vec2::new(5.0, 5.0)],
                    thin,
                );
            } else {
                painter.add(egui::Shape::line(
                    (0..=10)
                        .map(|step| {
                            let angle = -0.75 + 1.5 * step as f32 / 10.0;
                            c + s * Vec2::new(angle.cos() * 10.0, angle.sin() * 10.0)
                        })
                        .collect(),
                    egui::Stroke::new(1.5, color),
                ));
            }
        }
        "cast" => {
            let frame =
                Rect::from_min_max(c + s * Vec2::new(-9.0, -7.0), c + s * Vec2::new(9.0, 7.0));
            for (a, b) in [
                (frame.left_top(), frame.right_top()),
                (frame.right_top(), frame.right_bottom()),
                (
                    frame.right_bottom(),
                    frame.left_bottom() + s * Vec2::new(10.0, 0.0),
                ),
                (frame.left_top(), frame.left_top() + s * Vec2::new(0.0, 3.0)),
            ] {
                painter.line_segment([a, b], stroke);
            }
            for radius in [4.0, 8.0] {
                painter.add(egui::Shape::line(
                    (0..=8)
                        .map(|step| {
                            let angle = -std::f32::consts::FRAC_PI_2 * step as f32 / 8.0;
                            c + s * Vec2::new(-9.0, 7.0)
                                + s * Vec2::new(angle.cos() * radius, angle.sin() * radius)
                        })
                        .collect(),
                    stroke,
                ));
            }
            painter.circle_filled(c + s * Vec2::new(-8.0, 6.0), 1.2 * s, color);
        }
        "submit" => {
            painter.line_segment(
                [c + s * Vec2::new(0.0, 7.0), c + s * Vec2::new(0.0, -7.0)],
                stroke,
            );
            painter.line_segment(
                [c + s * Vec2::new(-6.0, -1.0), c + s * Vec2::new(0.0, -7.0)],
                stroke,
            );
            painter.line_segment(
                [c + s * Vec2::new(6.0, -1.0), c + s * Vec2::new(0.0, -7.0)],
                stroke,
            );
            painter.line_segment(
                [c + s * Vec2::new(-7.0, 9.0), c + s * Vec2::new(7.0, 9.0)],
                stroke,
            );
        }
        "fullscreen" => {
            for (a, b) in [
                ((-8.0, -3.0), (-8.0, -8.0)),
                ((-8.0, -8.0), (-3.0, -8.0)),
                ((8.0, -3.0), (8.0, -8.0)),
                ((8.0, -8.0), (3.0, -8.0)),
                ((-8.0, 3.0), (-8.0, 8.0)),
                ((-8.0, 8.0), (-3.0, 8.0)),
                ((8.0, 3.0), (8.0, 8.0)),
                ((8.0, 8.0), (3.0, 8.0)),
            ] {
                painter.line_segment(
                    [c + s * Vec2::new(a.0, a.1), c + s * Vec2::new(b.0, b.1)],
                    stroke,
                );
            }
        }
        _ => {}
    }
    response
}
