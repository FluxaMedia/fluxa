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

type Action = (&'static str, u64);

impl Chrome<'_> {
    fn actions(&self, mobile_header: bool) -> Vec<Action> {
        let (player, options) = (self.player, &self.player.options);
        let mut actions = Vec::new();
        if options.audio_subtitles {
            actions.push(("AudioSubtitles", NODE_PLAYER_TRACKS));
        }
        if options.speed {
            actions.push(("Gauge", NODE_PLAYER_SPEED));
        }
        if options.episodes && player.has_episodes {
            actions.push(("ListVideo", NODE_PLAYER_EPISODES));
        }
        if options.settings && self.viewport.form_factor == UiFormFactor::Desktop {
            actions.push(("Settings", NODE_PLAYER_SETTINGS));
        }
        if !mobile_header {
            if options.cast && self.viewport.form_factor == UiFormFactor::Tv {
                actions.push(("Cast", NODE_PLAYER_CAST));
            }
            if options.mark_segment {
                actions.push(("Flag", NODE_PLAYER_SUBMIT));
            }
        }
        actions
    }

    fn header_actions(&self, mobile: bool) -> Vec<Action> {
        let mut actions = if mobile {
            self.actions(true)
        } else {
            Vec::new()
        };
        if self.player.options.cast {
            actions.push(("Cast", NODE_PLAYER_CAST));
        }
        actions
    }

    fn volume_slider(&self, ui: &mut egui::Ui, layout: &mut HomeLayout, x: f32, y: f32) -> f32 {
        let player = self.player;
        let full = 88.0;
        let zone = Rect::from_min_max(
            Pos2::new(x - 52.0, y - 24.0),
            Pos2::new(x + full + 56.0, y + 24.0),
        );
        let id = Id::new("fluxa-player-volume");
        let dragging: bool = ui.data(|data| data.get_temp(id)).unwrap_or(false);
        let open = dragging || ui.rect_contains_pointer(zone);
        let amount = ui.ctx().animate_bool_with_time(id.with("open"), open, 0.16);
        let width = full * amount;
        if width < 2.0 {
            return x + full + 8.0;
        }
        let max = if player.boost { 200.0 } else { 100.0 };
        let track = Rect::from_min_size(Pos2::new(x + 4.0, y - 12.0), Vec2::new(width - 8.0, 24.0));
        let response = ui.interact(track, id, Sense::click_and_drag());
        ui.data_mut(|data| data.insert_temp(id, response.dragged()));
        if (response.clicked() || response.dragged())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let ratio = ((pointer.x - track.left()) / track.width()).clamp(0.0, 1.0);
            layout.player_gesture = Some(PlayerGesture::VolumeSet(ratio * max));
        }
        if response.hovered() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll.abs() > 0.5 {
                layout.player_gesture = Some(PlayerGesture::Volume(scroll.signum() * 0.05));
            }
        }
        let alpha = (255.0 * amount) as u8;
        let shown = if player.muted {
            0.0
        } else {
            player.volume as f32
        };
        let ratio = (shown / max).clamp(0.0, 1.0);
        let bar = Rect::from_center_size(track.center(), Vec2::new(track.width(), 4.0));
        self.painter
            .rect_filled(bar, 2.0, Color32::from_white_alpha(alpha / 3));
        let filled = Rect::from_min_size(bar.min, Vec2::new(bar.width() * ratio, 4.0));
        self.painter
            .rect_filled(filled, 2.0, Color32::from_white_alpha(alpha));
        if player.boost {
            let mark = bar.left() + bar.width() * 0.5;
            self.painter.line_segment(
                [
                    Pos2::new(mark, bar.center().y - 5.0),
                    Pos2::new(mark, bar.center().y + 5.0),
                ],
                egui::Stroke::new(1.5, Color32::from_white_alpha(alpha / 2)),
            );
        }
        self.painter.circle_filled(
            Pos2::new(filled.right(), bar.center().y),
            6.0 * amount,
            Color32::WHITE,
        );
        self.painter.text(
            Pos2::new(filled.right(), bar.center().y - 20.0),
            Align2::CENTER_CENTER,
            format!("{}%", shown.round() as i32),
            FontId::proportional(12.0),
            Color32::from_white_alpha(alpha),
        );
        x + full + 8.0
    }

    fn secondary(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.player.options.mark_segment {
            actions.push(("Flag", NODE_PLAYER_SUBMIT));
        }
        actions
    }

    fn row_left(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        mut x: f32,
        y: f32,
        size: f32,
        actions: &[Action],
    ) -> f32 {
        for (icon, node) in actions {
            self.button(ui, layout, Pos2::new(x + size * 0.5, y), size, icon, *node);
            x += size + size * 0.1;
        }
        x
    }

    fn row_right(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        mut x: f32,
        y: f32,
        size: f32,
        actions: &[Action],
    ) -> f32 {
        for (icon, node) in actions.iter().rev() {
            self.button(ui, layout, Pos2::new(x - size * 0.5, y), size, icon, *node);
            x -= size + size * 0.1;
        }
        x
    }

    fn heading(&self, left: f32, y: f32, width: f32, title: f32, sub: f32) {
        let player = self.player;
        if !player.options.title {
            return;
        }
        let episode = player
            .episode_title
            .as_deref()
            .filter(|text| !text.is_empty());
        let (title_y, sub_y) = match episode {
            Some(_) => (y - sub * 0.7, y + title * 0.6),
            None => (y, y),
        };
        self.text(
            Pos2::new(left, title_y),
            Align2::LEFT_CENTER,
            &player.title,
            title,
            width,
            255,
        );
        if let Some(episode) = episode {
            self.text(
                Pos2::new(left, sub_y),
                Align2::LEFT_CENTER,
                episode,
                sub,
                width,
                190,
            );
        }
    }

    fn transport(&self) -> Vec<Action> {
        let player = self.player;
        let mut actions = vec![(
            if player.paused {
                "PlayFilled"
            } else {
                "PauseFilled"
            },
            NODE_PLAYER_TOGGLE,
        )];
        if player.options.next_episode && player.has_next {
            actions.push(("SkipForward", NODE_PLAYER_NEXT));
        }
        actions
    }
}

pub(super) fn desktop_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    let margin = chrome.margin();
    let header_y = rect.top() + 36.0;
    let close_center = Pos2::new(margin + 20.0, header_y);
    chrome.button(
        ui,
        layout,
        close_center,
        42.0,
        "ArrowLeft",
        NODE_PLAYER_CLOSE,
    );
    if !player.has_video() || !player.controls_visible {
        if player.has_video() {
            chrome.surface(ui, layout, rect);
        }
        return;
    }
    scrims(chrome.painter, rect);
    let header_right = chrome.row_right(
        ui,
        layout,
        rect.right() - margin,
        header_y,
        44.0,
        &chrome.header_actions(false),
    );
    chrome.heading(
        close_center.x + 34.0,
        header_y,
        (header_right - close_center.x - 48.0).max(0.0),
        19.0,
        13.0,
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
    let mut x = chrome.row_left(ui, layout, margin, y, 44.0, &chrome.transport());
    if player.options.volume {
        let mute = if player.muted {
            "VolumeMuted"
        } else {
            "Volume"
        };
        chrome.button(
            ui,
            layout,
            Pos2::new(x + 22.0, y),
            44.0,
            mute,
            NODE_PLAYER_MUTE,
        );
        x += 48.0;
        x = chrome.volume_slider(ui, layout, x, y);
        let bolt = if player.boost { "ZapFilled" } else { "Zap" };
        chrome.button(
            ui,
            layout,
            Pos2::new(x + 18.0, y),
            40.0,
            bolt,
            NODE_PLAYER_BOOST,
        );
        x += 48.0;
    }
    let mut right = rect.right() - margin;
    if player.options.fullscreen {
        right = chrome.row_right(
            ui,
            layout,
            right,
            y,
            44.0,
            &[("Maximize", NODE_PLAYER_FULLSCREEN)],
        );
    }
    let right = chrome.row_right(ui, layout, right, y, 44.0, &chrome.actions(false));
    chrome.text(
        Pos2::new(x + 8.0, y),
        Align2::LEFT_CENTER,
        &chrome.times(position),
        13.0,
        (right - x - 24.0).max(0.0),
        218,
    );
}

pub(super) fn mobile_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    let margin = chrome.margin();
    let header_y = rect.top() + 38.0;
    let close_center = Pos2::new(margin + 20.0, header_y);
    if !player.has_video() {
        chrome.button(
            ui,
            layout,
            close_center,
            42.0,
            "ArrowLeft",
            NODE_PLAYER_CLOSE,
        );
        return;
    }
    chrome.surface(ui, layout, rect);
    if !player.controls_visible {
        return;
    }
    scrims(chrome.painter, rect);
    chrome.button(
        ui,
        layout,
        close_center,
        42.0,
        "ArrowLeft",
        NODE_PLAYER_CLOSE,
    );
    let right = chrome.row_right(
        ui,
        layout,
        rect.right() - margin,
        header_y,
        40.0,
        &chrome.header_actions(true),
    );
    let title_x = close_center.x + 34.0;
    chrome.heading(
        title_x,
        header_y,
        (right - title_x - 8.0).max(0.0),
        17.0,
        12.0,
    );
    let center = rect.center();
    let options = &player.options;
    if options.center_controls {
        let gap = (rect.width() * 0.22).min(130.0);
        let play = if player.paused {
            "PlayFilled"
        } else {
            "PauseFilled"
        };
        chrome.button(ui, layout, center, 76.0, play, NODE_PLAYER_TOGGLE);
        if options.next_episode && player.has_previous {
            chrome.button(
                ui,
                layout,
                center - Vec2::new(gap, 0.0),
                52.0,
                "SkipBack",
                NODE_PLAYER_PREVIOUS,
            );
        }
        if options.next_episode && player.has_next {
            chrome.button(
                ui,
                layout,
                center + Vec2::new(gap, 0.0),
                52.0,
                "SkipForward",
                NODE_PLAYER_NEXT,
            );
        }
    }
    let y = rect.bottom() - 30.0 - chrome.viewport.safe_bottom;
    let track = Rect::from_min_size(
        Pos2::new(margin, y - 62.0),
        Vec2::new((rect.width() - margin * 2.0).max(80.0), 20.0),
    );
    let position = chrome.seek_bar(ui, layout, track, 3.0);
    let label_y = track.top() - 6.0;
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
    let mut bottom = Vec::new();
    if !options.center_controls {
        bottom.extend(chrome.transport());
    }
    bottom.extend(chrome.secondary());
    chrome.row_left(ui, layout, margin - 6.0, y, 40.0, &bottom);
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
    if player.options.title {
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
    }
    chrome.text(
        Pos2::new(track.right(), track.top() - 16.0),
        Align2::RIGHT_BOTTOM,
        &chrome.times(position),
        17.0,
        track.width() * 0.3,
        218,
    );
    let mut actions = chrome.transport();
    actions.extend(chrome.actions(false));
    chrome.row_left(ui, layout, margin - 8.0, y, 56.0, &actions);
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

pub(super) fn icon_control(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    texture: Option<TextureId>,
    node: u64,
    scale: f32,
) -> egui::Response {
    let response = ui.interact(rect, Id::new(("fluxa-player-button", node)), Sense::click());
    let alpha = if response.hovered() || response.is_pointer_button_down_on() {
        255
    } else {
        225
    };
    if let Some(texture) = texture {
        painter.image(
            texture,
            Rect::from_center_size(rect.center(), Vec2::splat(rect.width() * scale)),
            full_uv(),
            Color32::from_white_alpha(alpha),
        );
    }
    response
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
