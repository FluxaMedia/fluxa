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
    fn actions(&self) -> Vec<Action> {
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
        if self.viewport.form_factor != UiFormFactor::Mobile {
            if options.mark_segment {
                actions.push(("Flag", NODE_PLAYER_SUBMIT));
            }
            actions.push(("Lock", NODE_PLAYER_LOCK));
        }
        actions.push(("Info", NODE_PLAYER_INFO));
        actions
    }

    fn header_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
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
        let amount = crate::motion::toggle(ui.ctx(), id.with("open"), open, crate::motion::fast());
        let width = full * amount;
        if width < 2.0 {
            return x;
        }
        let max = if player.boost { 200.0 } else { 100.0 };
        let track = Rect::from_min_size(Pos2::new(x + 4.0, y - 12.0), Vec2::new(width - 8.0, 24.0));
        let shown = if player.muted {
            0.0
        } else {
            player.volume as f32
        };
        let ratio = (shown / max).clamp(0.0, 1.0);
        let output = crate::components::slider(
            ui,
            self.painter,
            crate::components::Slider {
                id,
                track,
                ratio,
                fill: accent(),
                overdrive: player.boost.then_some(0.5),
                amount,
            },
        );
        ui.data_mut(|data| data.insert_temp(id, output.response.dragged()));
        if let Some(value) = output.value {
            layout.player_gesture = Some(PlayerGesture::VolumeSet(value * max));
        }
        if output.response.hovered() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll.abs() > 0.5 {
                layout.player_gesture = Some(PlayerGesture::Volume(scroll.signum() * 0.05));
            }
        }
        let alpha = (255.0 * amount) as u8;
        draw_text(
            self.painter,
            Pos2::new(
                track.left() + track.width() * ratio,
                track.center().y - 20.0,
            ),
            Align2::CENTER_CENTER,
            format!("{}%", shown.round() as i32),
            FontId::proportional(12.0),
            ink(alpha),
            None,
        );
        x + width + 8.0
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

    fn locked(&self, ui: &mut egui::Ui, layout: &mut HomeLayout) -> bool {
        if !self.player.locked {
            return false;
        }
        if self.player.has_video() {
            self.surface(ui, layout, self.rect);
            if self.player.controls_visible {
                scrims(self.painter, self.rect);
                let y = self.rect.bottom() - 48.0 - self.viewport.safe_bottom;
                let center = Pos2::new(self.margin() + self.icon() * 0.5, y);
                self.button(ui, layout, center, self.icon(), "Unlock", NODE_PLAYER_LOCK);
            }
        }
        true
    }

    fn close_button(&self, ui: &mut egui::Ui, layout: &mut HomeLayout, y: f32) -> Pos2 {
        let center = Pos2::new(self.margin() + 20.0, y);
        self.button(
            ui,
            layout,
            center,
            self.icon(),
            "ArrowLeft",
            NODE_PLAYER_CLOSE,
        );
        center
    }

    fn header(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        close: Pos2,
        icon: f32,
        sizes: (f32, f32),
        watch: bool,
    ) {
        let mut right = self.row_right(
            ui,
            layout,
            self.rect.right() - self.margin(),
            close.y,
            icon,
            &self.header_actions(),
        );
        if watch && self.player.watch_button {
            right = self.row_right(
                ui,
                layout,
                right,
                close.y,
                icon,
                &[("Users", NODE_PLAYER_WATCH)],
            );
        }
        let left = close.x + 34.0;
        self.heading(
            left,
            close.y,
            (right - left - 14.0).max(0.0),
            sizes.0,
            sizes.1,
        );
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
    if chrome.locked(ui, layout) {
        return;
    }
    if player.has_video() && player.controls_visible {
        scrims(chrome.painter, rect);
    }
    let close = chrome.close_button(ui, layout, header_y);
    if !player.has_video() || !player.controls_visible {
        if player.has_video() {
            chrome.surface(ui, layout, rect);
        }
        return;
    }
    chrome.header(ui, layout, close, chrome.icon(), (19.0, 13.0), true);
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
    let mut x = chrome.row_left(ui, layout, margin, y, chrome.icon(), &chrome.transport());
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
            chrome.icon(),
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
            chrome.icon_small(),
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
            chrome.icon(),
            &[("Maximize", NODE_PLAYER_FULLSCREEN)],
        );
    }
    let right = chrome.row_right(ui, layout, right, y, chrome.icon(), &chrome.actions());
    chrome.toggle_remaining(
        ui,
        Rect::from_min_size(Pos2::new(x + 8.0, y - 14.0), Vec2::new(96.0, 28.0)),
    );
    chrome.text(
        Pos2::new(x + 8.0, y),
        Align2::LEFT_CENTER,
        &chrome.times(ui, position),
        13.0,
        (right - x - 24.0).max(0.0),
        218,
    );
}

pub(super) fn trailer_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    let margin = chrome.margin();
    chrome.button(
        ui,
        layout,
        Pos2::new(margin + 20.0, rect.top() + 36.0),
        chrome.icon(),
        "ArrowLeft",
        NODE_PLAYER_CLOSE,
    );
    if !player.has_video() {
        return;
    }
    chrome.surface(ui, layout, rect);
    if !player.controls_visible {
        return;
    }
    scrims(chrome.painter, rect);
    let center = Pos2::new(
        rect.center().x,
        rect.bottom() - 64.0 - chrome.viewport.safe_bottom,
    );
    let mute = if player.muted {
        "VolumeMuted"
    } else {
        "Volume"
    };
    chrome.button(
        ui,
        layout,
        Pos2::new(center.x - 170.0, center.y),
        chrome.icon(),
        mute,
        NODE_PLAYER_MUTE,
    );
    chrome.volume_slider(ui, layout, center.x - 122.0, center.y);
    let play = if player.paused {
        "PlayFilled"
    } else {
        "PauseFilled"
    };
    chrome.button(
        ui,
        layout,
        center,
        chrome.icon() + 16.0,
        play,
        NODE_PLAYER_TOGGLE,
    );
    chrome.button(
        ui,
        layout,
        Pos2::new(center.x + 96.0, center.y),
        chrome.icon(),
        "AudioSubtitles",
        NODE_PLAYER_SUBTITLES,
    );
}

pub(super) fn mobile_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    let margin = chrome.margin();
    let header_y = rect.top() + 38.0;
    if !player.has_video() {
        chrome.close_button(ui, layout, header_y);
        return;
    }
    chrome.surface(ui, layout, rect);
    if !player.controls_visible {
        return;
    }
    if player.locked {
        let center = Pos2::new(margin + 20.0, rect.center().y);
        chrome.button(
            ui,
            layout,
            center,
            chrome.icon(),
            "Unlock",
            NODE_PLAYER_LOCK,
        );
        return;
    }
    scrims(chrome.painter, rect);
    let close = chrome.close_button(ui, layout, header_y);
    chrome.header(ui, layout, close, chrome.icon_small(), (17.0, 12.0), false);
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
    let mut left = Vec::new();
    if !options.center_controls {
        left.extend(chrome.transport());
    }
    left.extend(chrome.actions());
    let after = chrome.row_left(ui, layout, margin - 6.0, y, chrome.icon_small(), &left);
    let mut right = vec![("Lock", NODE_PLAYER_LOCK)];
    if options.settings {
        right.push(("Settings", NODE_PLAYER_SETTINGS));
    }
    right.extend(chrome.secondary());
    let right_edge = chrome.row_right(
        ui,
        layout,
        rect.right() - margin + 6.0,
        y,
        chrome.icon_small(),
        &right,
    );
    let time_width = (right_edge - after - 12.0).max(0.0);
    chrome.toggle_remaining(
        ui,
        Rect::from_min_size(
            Pos2::new(after + 4.0, y - 16.0),
            Vec2::new(time_width, 32.0),
        ),
    );
    chrome.text(
        Pos2::new(after + 8.0, y),
        Align2::LEFT_CENTER,
        &chrome.times(ui, position),
        13.0,
        time_width,
        218,
    );
}

pub(super) fn tv_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    if chrome.locked(ui, layout) || !player.has_video() || !player.controls_visible {
        return;
    }
    scrim_edge(
        chrome.painter,
        rect,
        (rect.height() * 0.3).min(220.0),
        true,
        110,
    );
    scrim_edge(
        chrome.painter,
        rect,
        (rect.height() * 0.5).min(420.0),
        false,
        255,
    );
    let margin = chrome.margin();
    let header_y = rect.top() + 56.0;
    chrome.close_button(ui, layout, header_y);
    chrome.row_right(
        ui,
        layout,
        rect.right() - margin,
        header_y,
        chrome.icon(),
        &chrome.header_actions(),
    );
    let y = rect.bottom() - 44.0 - chrome.viewport.safe_bottom;
    let track = Rect::from_min_size(
        Pos2::new(margin, y - 62.0),
        Vec2::new(rect.width() - margin * 2.0, 20.0),
    );
    let position = chrome.seek_bar(ui, layout, track, 4.0);
    let width = track.width() * 0.7;
    if player.options.title {
        let top = rect.top() + 36.0;
        let title_x = margin + 64.0;
        chrome.text(
            Pos2::new(title_x, top),
            Align2::LEFT_TOP,
            &player.title,
            28.0,
            width,
            255,
        );
        if let Some(episode) = player.episode_title.as_deref() {
            chrome.text(
                Pos2::new(title_x, top + 38.0),
                Align2::LEFT_TOP,
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
        &chrome.times(ui, position),
        17.0,
        track.width() * 0.3,
        218,
    );
    let mut actions = chrome.transport();
    actions.extend(chrome.actions());
    chrome.row_left(ui, layout, margin - 8.0, y, chrome.icon(), &actions);
}

pub(super) fn draw_focus_ring(
    context: &egui::Context,
    viewport: Viewport,
    layout: &HomeLayout,
    focused: Option<u64>,
) {
    if let Some((node, rect)) = layout
        .focusable
        .iter()
        .find(|(id, _)| Some(*id) == focused && *id != NODE_PLAYER_SEEK)
    {
        let radius = if (NODE_PLAYER_SOURCE_BASE..NODE_PLAYER_PANEL_ROW_BASE).contains(node) {
            UiMetrics::for_viewport(viewport).card_radius + 3.0
        } else {
            rect.height().min(rect.width()) * 0.5
        };
        context
            .layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                Id::new("fluxa-player-focus"),
            ))
            .rect_stroke(
                rect.expand(3.0),
                radius,
                egui::Stroke::new(stroke::EMPHASIS, text_primary()),
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
    control_with_id(
        ui,
        painter,
        rect,
        icon,
        prominent,
        Id::new(("fluxa-player-button", icon)),
    )
}

pub(super) fn control_with_id(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    icon: &str,
    prominent: bool,
    id: Id,
) -> egui::Response {
    let response = ui.interact(rect, id, Sense::click());
    if prominent || response.hovered() || response.is_pointer_button_down_on() {
        crate::components::glass(
            painter,
            rect,
            rect.width() * 0.5,
            if prominent {
                ink(tone::RAISED)
            } else {
                shade(Shade::Faint)
            },
        );
    }
    let color = if response.hovered() {
        text_primary()
    } else {
        ink(tone::TEXT)
    };
    let c = rect.center();
    let s = (rect.width() / 48.0).max(1.0);
    let stroke = egui::Stroke::new(stroke::EMPHASIS, color);
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
            draw_text(
                painter,
                c + s * Vec2::new(0.0, 7.0),
                Align2::CENTER_CENTER,
                "10",
                FontId::proportional(9.0 * s),
                color,
                None,
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
                let thin = egui::Stroke::new(stroke::ICON, color);
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
                    egui::Stroke::new(stroke::ICON, color),
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
        "retry" | "retry-spin" => {
            let radius = 8.0 * s;
            let turn = if icon == "retry-spin" {
                ui.ctx().request_repaint();
                (ui.input(|input| input.time) as f32 * 540.0) % 360.0
            } else {
                0.0
            };
            let at = |degrees: f32| {
                let angle = (degrees + turn).to_radians();
                c + radius * Vec2::new(angle.cos(), angle.sin())
            };
            let points: Vec<Pos2> = (0..=40)
                .map(|step| at(step as f32 * 290.0 / 40.0))
                .collect();
            let end = points[40];
            let angle = (290.0_f32 + turn).to_radians();
            let forward = Vec2::new(-angle.sin(), angle.cos());
            let side = Vec2::new(-forward.y, forward.x);
            painter.circle_filled(points[0], stroke.width * 0.5, color);
            painter.add(egui::Shape::line(points, stroke));
            painter.add(egui::Shape::convex_polygon(
                vec![
                    end + forward * 5.5 * s,
                    end + side * 3.5 * s,
                    end - side * 3.5 * s,
                ],
                color,
                egui::Stroke::NONE,
            ));
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
