use super::*;
use crate::kit::view;

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
            actions.push(("Queue", NODE_PLAYER_EPISODES));
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
        actions.extend(self.source_tools());
        actions
    }

    fn source_tools(&self) -> Vec<Action> {
        let mut actions = vec![("Info", NODE_PLAYER_INFO)];
        if self.player.can_change_source {
            actions.push(("Swap", NODE_PLAYER_CHANGE_SOURCE));
        }
        actions
    }

    fn header_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.player.options.cast {
            actions.push(("Cast", NODE_PLAYER_CAST));
        }
        if self.player.minimizable {
            actions.push(("PictureInPicture", NODE_PLAYER_MINIMIZE));
        }
        actions
    }

    fn volume_slider(&self, ui: &mut egui::Ui, layout: &mut HomeLayout, x: f32, y: f32) -> f32 {
        let player = self.player;
        let full = tokens::PlayerTokens::for_form_factor(self.viewport.form_factor).volume_width;
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
        let mut lane = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    Pos2::new(x, y - 12.0),
                    Vec2::new(width, 24.0),
                ))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        lane.spacing_mut().item_spacing = Vec2::ZERO;
        lane.allocate_space(Vec2::new(4.0, 24.0));
        let track = lane
            .allocate_space(Vec2::new((width - 8.0).max(0.0), 24.0))
            .1;
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
                height: crate::style::slider_track(),
                knob: crate::style::slider_knob(),
                bubble: None,
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

    fn row_left(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        x: f32,
        y: f32,
        size: f32,
        actions: &[Action],
    ) -> f32 {
        let mut row = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    Pos2::new(x, y - size * 0.5),
                    Vec2::new(self.rect.width(), size),
                ))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        row.spacing_mut().item_spacing = Vec2::new(size * 0.1, 0.0);
        for (icon, node) in actions {
            let slot = row.allocate_space(Vec2::splat(size)).1;
            self.button(ui, layout, slot.center(), size, icon, *node);
        }
        row.cursor().min.x
    }

    fn row_right(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        x: f32,
        y: f32,
        size: f32,
        actions: &[Action],
    ) -> f32 {
        let mut row = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    Pos2::new(x - self.rect.width(), y - size * 0.5),
                    Vec2::new(self.rect.width(), size),
                ))
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        row.spacing_mut().item_spacing = Vec2::new(size * 0.1, 0.0);
        for (icon, node) in actions.iter().rev() {
            let slot = row.allocate_space(Vec2::splat(size)).1;
            self.button(ui, layout, slot.center(), size, icon, *node);
        }
        row.cursor().max.x
    }

    fn band(&self, ui: &mut egui::Ui, salt: &'static str) -> egui::Ui {
        let margin = self.margin();
        ui.new_child(
            egui::UiBuilder::new()
                .id_salt(salt)
                .max_rect(Rect::from_min_max(
                    Pos2::new(self.rect.left() + margin, self.rect.top()),
                    Pos2::new(
                        self.rect.right() - margin,
                        self.rect.bottom() - self.viewport.safe_bottom,
                    ),
                ))
                .layout(egui::Layout::top_down(egui::Align::Min)),
        )
    }

    fn slot(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        size: f32,
        cell: f32,
        icon: &str,
        node: u64,
    ) {
        let slot = ui.allocate_space(Vec2::new(size, cell)).1;
        self.button(ui, layout, slot.center(), size, icon, node);
    }

    fn bottom_band(&self, ui: &mut egui::Ui) -> egui::Ui {
        let margin = self.margin();
        let mut band = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_max(
                    Pos2::new(margin, self.rect.top()),
                    Pos2::new(
                        self.rect.right() - margin,
                        self.rect.bottom() - self.viewport.safe_bottom,
                    ),
                ))
                .layout(egui::Layout::bottom_up(egui::Align::Min)),
        );
        band.spacing_mut().item_spacing = Vec2::ZERO;
        band
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

    fn mobile_heading(&self, left: f32, y: f32, width: f32) {
        let player = self.player;
        if !player.options.title {
            return;
        }
        let episode = player
            .episode_title
            .as_deref()
            .filter(|text| !text.is_empty());
        let numbered = match (player.season_episode, episode) {
            (Some((season, number)), Some(name)) => Some(format!("S{season}:E{number} \"{name}\"")),
            (None, Some(name)) => Some(name.to_owned()),
            _ => None,
        };
        let (title_y, sub_y) = match numbered {
            Some(_) => (y - 9.0, y + 10.0),
            None => (y, y),
        };
        self.text(
            Pos2::new(left, title_y),
            Align2::LEFT_CENTER,
            &player.title,
            16.0,
            width,
            255,
        );
        if let Some(line) = numbered {
            self.text(
                Pos2::new(left, sub_y),
                Align2::LEFT_CENTER,
                &line,
                12.0,
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
    let mut bottom = chrome.bottom_band(ui);
    bottom.allocate_space(Vec2::new(0.0, 24.0));
    let y = bottom.allocate_space(Vec2::new(0.0, 48.0)).1.center().y;
    bottom.allocate_space(Vec2::new(0.0, 5.0));
    let track_width = bottom.max_rect().width().max(80.0);
    let track = bottom.allocate_space(Vec2::new(track_width, 16.0)).1;
    chrome.surface(
        ui,
        layout,
        Rect::from_min_max(
            Pos2::new(rect.left(), header_y + 29.0),
            Pos2::new(rect.right(), track.top() - 8.0),
        ),
    );
    let position = chrome.seek_bar(ui, layout, track, 3.0);
    let mut x = chrome.row_left(ui, layout, margin, y, chrome.icon(), &chrome.transport());
    if player.options.volume {
        let mute = if player.muted {
            "VolumeMuted"
        } else {
            "Volume"
        };
        let mut group = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    Pos2::new(x, y - 24.0),
                    Vec2::new(rect.width(), 48.0),
                ))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        group.spacing_mut().item_spacing = Vec2::ZERO;
        let slot = group.allocate_space(Vec2::new(48.0, 48.0)).1;
        chrome.button(
            ui,
            layout,
            Pos2::new(slot.left() + 22.0, y),
            chrome.icon(),
            mute,
            NODE_PLAYER_MUTE,
        );
        let after_slider = chrome.volume_slider(ui, layout, slot.right(), y);
        group.allocate_space(Vec2::new(after_slider - slot.right(), 48.0));
        let bolt = if player.boost { "ZapFilled" } else { "Zap" };
        let slot = group.allocate_space(Vec2::new(48.0, 48.0)).1;
        chrome.button(
            ui,
            layout,
            Pos2::new(slot.left() + 18.0, y),
            chrome.icon_small(),
            bolt,
            NODE_PLAYER_BOOST,
        );
        x = slot.right();
    }
    let mut right = rect.right() - margin;
    if player.options.fullscreen {
        right = chrome.row_right(
            ui,
            layout,
            right,
            y,
            chrome.icon(),
            &[("CornersOut", NODE_PLAYER_FULLSCREEN)],
        );
    }
    let right = chrome.row_right(ui, layout, right, y, chrome.icon(), &chrome.actions());
    let times = chrome.times_only(ui, position);
    let times_width = chrome
        .painter
        .layout_no_wrap(times.clone(), FontId::proportional(13.0), ink(218))
        .size()
        .x;
    let mut line = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                Pos2::new(x, y - 14.0),
                Pos2::new(right - 16.0, y + 14.0),
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    line.spacing_mut().item_spacing = Vec2::ZERO;
    line.allocate_space(Vec2::new(8.0, 28.0));
    let room = line.available_width().max(0.0);
    let times_slot = line.allocate_space(Vec2::new(times_width, 28.0)).1;
    chrome.toggle_remaining(
        ui,
        Rect::from_min_size(times_slot.left_top(), Vec2::new(96.0, 28.0)),
    );
    chrome.text(
        times_slot.left_center(),
        Align2::LEFT_CENTER,
        &times,
        13.0,
        room.min(times_width + 4.0),
        218,
    );
    if let Some(chapter) = player.chapter_at(position) {
        line.allocate_space(Vec2::new(14.0, 28.0));
        let width = line.available_width().max(0.0);
        let slot = line.allocate_space(Vec2::new(width, 28.0)).1;
        let hit = Rect::from_min_size(
            Pos2::new(slot.left() - 6.0, y - 16.0),
            Vec2::new(slot.width() + 6.0, 32.0),
        );
        let response = ui.interact(
            hit,
            Id::new(("fluxa-player-button", NODE_PLAYER_CHAPTERS)),
            Sense::click(),
        );
        components::focus_action(layout, NODE_PLAYER_CHAPTERS, &response);
        chrome.text(
            slot.left_center(),
            Align2::LEFT_CENTER,
            chapter,
            13.0,
            slot.width(),
            if response.hovered() || response.is_pointer_button_down_on() {
                255
            } else {
                190
            },
        );
    }
}

pub(super) fn trailer_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    let tokens = tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let size = chrome.icon();
    let mut top = chrome.band(ui, "top");
    view().class("player-top").show(&mut top, |ui| {
        chrome.slot(ui, layout, size, size, "ArrowLeft", NODE_PLAYER_CLOSE);
    });
    if !player.has_video() {
        return;
    }
    chrome.surface(ui, layout, rect);
    if !player.controls_visible {
        return;
    }
    scrims(chrome.painter, rect);
    let mute = if player.muted {
        "VolumeMuted"
    } else {
        "Volume"
    };
    let play = if player.paused {
        "PlayFilled"
    } else {
        "PauseFilled"
    };
    let mut bottom = chrome.band(ui, "bottom");
    view()
        .class("player-trailer")
        .dock_bottom()
        .center_x()
        .show(&mut bottom, |ui| {
            let cell = size + tokens.play_boost;
            chrome.slot(ui, layout, size, cell, mute, NODE_PLAYER_MUTE);
            let lane = ui.allocate_space(Vec2::new(tokens.volume_width, cell)).1;
            chrome.volume_slider(ui, layout, lane.left(), lane.center().y);
            chrome.slot(ui, layout, cell, cell, play, NODE_PLAYER_TOGGLE);
            chrome.slot(
                ui,
                layout,
                size,
                cell,
                "AudioSubtitles",
                NODE_PLAYER_SUBTITLES,
            );
        });
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
    let side = Pos2::new(rect.right() - margin - chrome.icon() * 0.5, rect.center().y);
    if player.locked {
        chrome.button(
            ui,
            layout,
            side + Vec2::new(0.0, chrome.icon() * 0.7),
            chrome.icon(),
            "Unlock",
            NODE_PLAYER_LOCK,
        );
        return;
    }
    scrims(chrome.painter, rect);
    let close = chrome.close_button(ui, layout, header_y);
    let icon = chrome.icon_small();
    let right = chrome.row_right(
        ui,
        layout,
        rect.right() - margin,
        close.y,
        icon,
        &chrome.header_actions(),
    );
    let right = if player.watch_button {
        chrome.row_right(
            ui,
            layout,
            right,
            close.y,
            icon,
            &[("Users", NODE_PLAYER_WATCH)],
        )
    } else {
        right
    };
    let right = chrome.row_right(ui, layout, right, close.y, icon, &chrome.source_tools());
    let left = close.x + 34.0;
    chrome.mobile_heading(left, close.y, (right - left - 14.0).max(0.0));
    let center = rect.center();
    let options = &player.options;
    let gap = (rect.width() * 0.2).min(120.0);
    if options.center_controls {
        chrome.button(
            ui,
            layout,
            center,
            76.0,
            if player.paused {
                "PlayFilled"
            } else {
                "PauseFilled"
            },
            NODE_PLAYER_TOGGLE,
        );
        for (icon, node, direction) in [
            ("Rewind10", NODE_PLAYER_REWIND, -1.0),
            ("Forward10", NODE_PLAYER_FORWARD, 1.0),
        ] {
            let at = center + Vec2::new(gap * direction, 0.0);
            chrome.button(ui, layout, at, 60.0, icon, node);
            draw_text(
                chrome.painter,
                at + Vec2::new(0.0, 2.0),
                Align2::CENTER_CENTER,
                "10",
                FontId::proportional(14.0),
                ink(235),
                None,
            );
        }
    }
    let mut bottom = chrome.bottom_band(ui);
    bottom.allocate_space(Vec2::new(0.0, 10.0));
    let time_band = bottom.allocate_space(Vec2::new(0.0, 32.0)).1;
    let row_y = time_band.center().y;
    bottom.allocate_space(Vec2::new(0.0, 12.0));
    let track_width = bottom.max_rect().width().max(80.0);
    let track = bottom.allocate_space(Vec2::new(track_width, 20.0)).1;
    let position = chrome.seek_bar(ui, layout, track, 3.0);
    let mut actions = Vec::new();
    if options.episodes && player.has_episodes {
        actions.push(("Queue", NODE_PLAYER_EPISODES));
    }
    if options.next_episode && player.has_next {
        actions.push(("SkipForward", NODE_PLAYER_NEXT));
    }
    if options.speed {
        actions.push(("Gauge", NODE_PLAYER_SPEED));
    }
    if options.audio_subtitles {
        actions.push(("AudioSubtitles", NODE_PLAYER_TRACKS));
    }
    let right_edge = chrome.row_right(
        ui,
        layout,
        rect.right() - margin + 6.0,
        row_y,
        chrome.icon_small(),
        &actions,
    );
    let times = format!(
        "{} / {}",
        format_time(position),
        format_time(player.duration)
    );
    let font = FontId::proportional(13.0);
    let times_width = chrome
        .painter
        .layout_no_wrap(times.clone(), font, ink(218))
        .size()
        .x;
    let mut row = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                time_band.left_top(),
                Pos2::new(right_edge - 8.0, time_band.bottom()),
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    row.spacing_mut().item_spacing = Vec2::ZERO;
    let times_slot = row.allocate_space(Vec2::new(times_width, 32.0)).1;
    chrome.text(
        Pos2::new(times_slot.left() + 2.0, row_y),
        Align2::LEFT_CENTER,
        &times,
        13.0,
        times_width + 4.0,
        218,
    );
    if let Some(chapter) = player.chapter_at(position) {
        row.allocate_space(Vec2::new(14.0, 32.0));
        let width = row.available_width().max(0.0);
        let slot = row.allocate_space(Vec2::new(width, 32.0)).1;
        let hit = Rect::from_min_size(
            Pos2::new(slot.left() - 6.0, row_y - 16.0),
            Vec2::new(slot.width() + 6.0, 32.0),
        );
        let response = ui.interact(
            hit,
            Id::new(("fluxa-player-button", NODE_PLAYER_CHAPTERS)),
            Sense::click(),
        );
        components::focus_action(layout, NODE_PLAYER_CHAPTERS, &response);
        chrome.text(
            slot.left_center(),
            Align2::LEFT_CENTER,
            chapter,
            13.0,
            slot.width(),
            if response.is_pointer_button_down_on() {
                255
            } else {
                190
            },
        );
    }
    let on = !player.upscaling.is_empty() && player.upscaling != "off";
    let anime = side - Vec2::new(0.0, chrome.icon() * 0.7);
    if on {
        chrome
            .painter
            .circle_filled(anime, chrome.icon() * 0.5, Color32::from_white_alpha(46));
    }
    chrome.button(
        ui,
        layout,
        anime,
        chrome.icon(),
        "Sparkles",
        NODE_PLAYER_ANIME4K,
    );
    chrome.button(
        ui,
        layout,
        side + Vec2::new(0.0, chrome.icon() * 0.7),
        chrome.icon(),
        "Lock",
        NODE_PLAYER_LOCK,
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
    let mut bottom = chrome.bottom_band(ui);
    bottom.allocate_space(Vec2::new(0.0, 22.0));
    let y = bottom.allocate_space(Vec2::new(0.0, 44.0)).1.center().y;
    bottom.allocate_space(Vec2::new(0.0, 20.0));
    let track_width = bottom.max_rect().width();
    let track = bottom.allocate_space(Vec2::new(track_width, 20.0)).1;
    let position = chrome.seek_bar(ui, layout, track, 4.0);
    let width = track.width() * 0.7;
    if player.options.title {
        let mut column = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    Pos2::new(margin + 64.0, rect.top() + 36.0),
                    Vec2::new(width, rect.height()),
                ))
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        column.spacing_mut().item_spacing = Vec2::ZERO;
        let title_slot = column.allocate_space(Vec2::new(width, 38.0)).1;
        chrome.text(
            title_slot.left_top(),
            Align2::LEFT_TOP,
            &player.title,
            28.0,
            width,
            255,
        );
        if let Some(episode) = player.episode_title.as_deref() {
            chrome.text(
                title_slot.left_bottom(),
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

pub(crate) fn control(
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
                crate::stylesheet::color("raised")
            } else {
                shade(Shade::Faint)
            },
        );
    }
    let color = if response.hovered() {
        text_primary()
    } else {
        crate::stylesheet::color("text")
    };
    let c = rect.center();
    let s = (rect.width() / 48.0).max(1.0);
    let stroke = egui::Stroke::new(stroke::emphasis(), color);
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
                let thin = egui::Stroke::new(stroke::icon(), color);
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
                    egui::Stroke::new(stroke::icon(), color),
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
