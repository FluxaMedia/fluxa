use super::*;
use crate::flex::Space;
use crate::kit::view;

const VOLUME_MAX: f32 = 100.0;
const BOOST_VOLUME_MAX: f32 = 200.0;
const OVERDRIVE_MARK: f32 = VOLUME_MAX / BOOST_VOLUME_MAX;
const WHEEL_VOLUME_STEP: f32 = 0.05;
const WHEEL_DEADZONE: f32 = 0.5;
const MIN_LANE_WIDTH: f32 = 2.0;
const SKIP_SECONDS: u32 = 10;
const INK_SUBDUED: u8 = 190;
const INK_TIMES: u8 = 218;
const INK_SKIP: u8 = 235;
const ICON_IDLE: u8 = 225;
const GLOW_ALPHA: u8 = 46;

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

    fn volume_lane(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        full: f32,
        height: f32,
        reflow: bool,
    ) {
        let player = self.player;
        let tokens = tokens::PlayerTokens::for_form_factor(self.viewport.form_factor);
        let metrics = UiMetrics::for_viewport(self.viewport);
        let inset = Space::S4.px(&metrics);
        let id = Id::new("fluxa-player-volume");
        let home = Rect::from_min_size(ui.next_widget_position(), Vec2::new(full, height));
        let zone = home.expand2(Vec2::new(tokens.volume_reach, 0.0));
        let dragging: bool = ui.data(|data| data.get_temp(id)).unwrap_or(false);
        let open = dragging || ui.rect_contains_pointer(zone);
        let amount = crate::motion::toggle(ui.ctx(), id.with("open"), open, crate::motion::fast());
        let drawn = full * amount;
        ui.allocate_space(Vec2::new(if reflow { drawn } else { full }, height));
        if drawn < MIN_LANE_WIDTH {
            return;
        }
        let max = if player.boost {
            BOOST_VOLUME_MAX
        } else {
            VOLUME_MAX
        };
        let track = Align2::LEFT_CENTER.anchor_size(
            home.left_center() + Vec2::new(inset, 0.0),
            Vec2::new((drawn - inset - inset).max(0.0), tokens.volume_track),
        );
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
                overdrive: player.boost.then_some(OVERDRIVE_MARK),
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
            if scroll.abs() > WHEEL_DEADZONE {
                layout.player_gesture =
                    Some(PlayerGesture::Volume(scroll.signum() * WHEEL_VOLUME_STEP));
            }
        }
        let alpha = (f32::from(u8::MAX) * amount) as u8;
        draw_text(
            self.painter,
            Pos2::new(
                track.left() + track.width() * ratio,
                track.top() - Space::S8.px(&metrics),
            ),
            Align2::CENTER_CENTER,
            format!("{}%", shown.round() as i32),
            FontId::proportional(tokens.label_text),
            ink(alpha),
            None,
        );
    }

    fn slot(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        size: f32,
        cell: f32,
        icon: &str,
        node: u64,
    ) -> Rect {
        let slot = ui.allocate_space(Vec2::new(size, cell)).1;
        self.button(ui, layout, slot.center(), size, icon, node);
        slot
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

    fn centered(&self, ui: &mut egui::Ui, salt: &'static str, height: f32) -> egui::Ui {
        let margin = self.margin();
        let middle = self.rect.center().y;
        ui.new_child(
            egui::UiBuilder::new()
                .id_salt(salt)
                .max_rect(Rect::from_min_max(
                    Pos2::new(self.rect.left() + margin, middle - height * 0.5),
                    Pos2::new(self.rect.right() - margin, middle + height * 0.5),
                ))
                .layout(egui::Layout::top_down(egui::Align::Min)),
        )
    }

    fn title_lines(&self, sub: Option<String>) -> Option<(String, Option<String>)> {
        if !self.player.options.title {
            return None;
        }
        Some((
            self.player.title.clone(),
            sub.filter(|text| !text.is_empty()),
        ))
    }

    fn header(
        &self,
        band: &mut egui::Ui,
        layout: &mut HomeLayout,
        close: f32,
        small: f32,
        actions: &[Action],
        title: Option<(String, Option<String>)>,
    ) -> Rect {
        let tokens = tokens::PlayerTokens::for_form_factor(self.viewport.form_factor);
        band.add_space(tokens.header_top);
        view()
            .class("player-header")
            .row()
            .show(band, |ui| {
                self.slot(ui, layout, close, close, "ArrowLeft", NODE_PLAYER_CLOSE);
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), close),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        for &(icon, node) in actions.iter().rev() {
                            self.slot(ui, layout, small, close, icon, node);
                        }
                        if let Some((title, sub)) = title {
                            let area = ui.allocate_space(Vec2::new(ui.available_width(), close)).1;
                            let width = area.width();
                            match sub {
                                None => self.text(
                                    area.left_center(),
                                    Align2::LEFT_CENTER,
                                    &title,
                                    tokens.header_title,
                                    width,
                                    u8::MAX,
                                ),
                                Some(sub) => {
                                    self.text(
                                        area.left_center(),
                                        Align2::LEFT_BOTTOM,
                                        &title,
                                        tokens.header_title,
                                        width,
                                        u8::MAX,
                                    );
                                    self.text(
                                        area.left_center(),
                                        Align2::LEFT_TOP,
                                        &sub,
                                        tokens.header_sub,
                                        width,
                                        INK_SUBDUED,
                                    );
                                }
                            }
                        }
                    },
                );
            })
            .response
            .rect
    }

    fn trailing(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        size: f32,
        position: f64,
        actions: &[Action],
        toggle: bool,
    ) {
        let tokens = tokens::PlayerTokens::for_form_factor(self.viewport.form_factor);
        let metrics = UiMetrics::for_viewport(self.viewport);
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), size),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                for &(icon, node) in actions.iter().rev() {
                    self.slot(ui, layout, size, size, icon, node);
                }
                let flex = ui.allocate_space(Vec2::new(ui.available_width(), size)).1;
                let times = self.times_only(ui, position);
                let font = FontId::proportional(tokens.times_text);
                let measured = self
                    .painter
                    .layout_no_wrap(times.clone(), font, ink(INK_TIMES))
                    .size()
                    .x;
                let times_slot = Rect::from_min_size(
                    flex.left_top(),
                    Vec2::new(measured.max(tokens.times_width), size),
                );
                if toggle {
                    self.toggle_remaining(ui, times_slot);
                }
                self.text(
                    times_slot.left_center(),
                    Align2::LEFT_CENTER,
                    &times,
                    tokens.times_text,
                    times_slot.width(),
                    INK_TIMES,
                );
                if let Some(chapter) = self.player.chapter_at(position) {
                    let start = times_slot.right() + Space::S14.px(&metrics);
                    let slot =
                        Rect::from_min_max(Pos2::new(start, flex.top()), flex.right_bottom());
                    let hit = slot.expand2(Vec2::new(Space::S6.px(&metrics), 0.0));
                    let response = ui.interact(
                        hit,
                        Id::new(("fluxa-player-button", NODE_PLAYER_CHAPTERS)),
                        Sense::click(),
                    );
                    components::focus_action(layout, NODE_PLAYER_CHAPTERS, &response);
                    let alpha = if response.hovered() || response.is_pointer_button_down_on() {
                        u8::MAX
                    } else {
                        INK_SUBDUED
                    };
                    self.text(
                        slot.left_center(),
                        Align2::LEFT_CENTER,
                        chapter,
                        tokens.times_text,
                        slot.width(),
                        alpha,
                    );
                }
            },
        );
    }

    fn skip_label(&self, at: Pos2, size: f32) {
        draw_text(
            self.painter,
            at,
            Align2::CENTER_CENTER,
            &SKIP_SECONDS.to_string(),
            FontId::proportional(size),
            ink(INK_SKIP),
            None,
        );
    }

    fn locked(&self, ui: &mut egui::Ui, layout: &mut HomeLayout) -> bool {
        if !self.player.locked {
            return false;
        }
        if self.player.has_video() {
            self.surface(ui, layout, self.rect);
            if self.player.controls_visible {
                scrims(self.painter, self.rect);
                let mut band = self.band(ui, "lock");
                view()
                    .class("player-lock")
                    .row()
                    .dock_bottom()
                    .show(&mut band, |ui| {
                        self.slot(
                            ui,
                            layout,
                            self.icon(),
                            self.icon(),
                            "Unlock",
                            NODE_PLAYER_LOCK,
                        );
                    });
            }
        }
        true
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
    let tokens = tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let icon = chrome.icon();
    if chrome.locked(ui, layout) {
        return;
    }
    let shown = player.has_video() && player.controls_visible;
    if shown {
        scrims(chrome.painter, rect);
    }
    let mut top = chrome.band(ui, "top");
    let header = if shown {
        let mut actions = Vec::new();
        if player.watch_button {
            actions.push(("Users", NODE_PLAYER_WATCH));
        }
        actions.extend(chrome.header_actions());
        chrome.header(
            &mut top,
            layout,
            icon,
            icon,
            &actions,
            chrome.title_lines(player.episode_title.clone()),
        )
    } else {
        chrome.header(&mut top, layout, icon, icon, &[], None)
    };
    if !shown {
        if player.has_video() {
            chrome.surface(ui, layout, rect);
        }
        return;
    }
    let mut bottom = chrome.band(ui, "bottom");
    let controls = view()
        .class("player-bottom")
        .dock_bottom()
        .column()
        .show(&mut bottom, |ui| {
            let track = ui
                .allocate_space(Vec2::new(ui.available_width(), tokens.seek_track))
                .1;
            let position = chrome.seek_bar(ui, layout, track, tokens.seek_thickness);
            view().class("player-controls").row().show(ui, |ui| {
                for (name, node) in chrome.transport() {
                    chrome.slot(ui, layout, icon, icon, name, node);
                }
                if player.options.volume {
                    let mute = if player.muted {
                        "VolumeMuted"
                    } else {
                        "Volume"
                    };
                    let bolt = if player.boost { "ZapFilled" } else { "Zap" };
                    ui.scope(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        chrome.slot(ui, layout, icon, icon, mute, NODE_PLAYER_MUTE);
                        chrome.volume_lane(ui, layout, tokens.volume_width, icon, true);
                        chrome.slot(
                            ui,
                            layout,
                            chrome.icon_small(),
                            icon,
                            bolt,
                            NODE_PLAYER_BOOST,
                        );
                    });
                }
                let mut trailing = chrome.actions();
                if player.options.fullscreen {
                    trailing.push(("CornersOut", NODE_PLAYER_FULLSCREEN));
                }
                chrome.trailing(ui, layout, icon, position, &trailing, true);
            });
            ui.add_space(tokens.controls_bottom);
        });
    chrome.surface(
        ui,
        layout,
        Rect::from_min_max(
            Pos2::new(rect.left(), header.bottom()),
            Pos2::new(rect.right(), controls.response.rect.top()),
        ),
    );
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
            chrome.volume_lane(ui, layout, tokens.volume_width, cell, false);
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
    let tokens = tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let metrics = UiMetrics::for_viewport(chrome.viewport);
    let icon = chrome.icon();
    let small = chrome.icon_small();
    if !player.has_video() {
        let mut top = chrome.band(ui, "top");
        chrome.header(&mut top, layout, icon, small, &[], None);
        return;
    }
    chrome.surface(ui, layout, rect);
    if !player.controls_visible {
        return;
    }
    if player.locked {
        let mut side = chrome.centered(ui, "side", icon);
        view().class("player-side").column().show(&mut side, |ui| {
            chrome.slot(ui, layout, icon, icon, "Unlock", NODE_PLAYER_LOCK);
        });
        return;
    }
    scrims(chrome.painter, rect);
    let episode = player
        .episode_title
        .as_deref()
        .filter(|text| !text.is_empty());
    let numbered = match (player.season_episode, episode) {
        (Some((season, number)), Some(name)) => Some(format!("S{season}:E{number} \"{name}\"")),
        (None, Some(name)) => Some(name.to_owned()),
        _ => None,
    };
    let mut right = chrome.source_tools();
    if player.watch_button {
        right.push(("Users", NODE_PLAYER_WATCH));
    }
    right.extend(chrome.header_actions());
    let mut top = chrome.band(ui, "top");
    chrome.header(
        &mut top,
        layout,
        icon,
        small,
        &right,
        chrome.title_lines(numbered),
    );
    if player.options.center_controls {
        let mut center = chrome.centered(ui, "center", tokens.center_play);
        let play = if player.paused {
            "PlayFilled"
        } else {
            "PauseFilled"
        };
        view()
            .class("player-center")
            .row()
            .center_x()
            .hug()
            .show(&mut center, |ui| {
                let back = chrome.slot(
                    ui,
                    layout,
                    tokens.center_skip,
                    tokens.center_play,
                    "Rewind10",
                    NODE_PLAYER_REWIND,
                );
                chrome.slot(
                    ui,
                    layout,
                    tokens.center_play,
                    tokens.center_play,
                    play,
                    NODE_PLAYER_TOGGLE,
                );
                let forward = chrome.slot(
                    ui,
                    layout,
                    tokens.center_skip,
                    tokens.center_play,
                    "Forward10",
                    NODE_PLAYER_FORWARD,
                );
                chrome.skip_label(back.center(), tokens.skip_text);
                chrome.skip_label(forward.center(), tokens.skip_text);
            });
    }
    let options = &player.options;
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
    let mut bottom = chrome.band(ui, "bottom");
    view()
        .class("player-bottom")
        .dock_bottom()
        .column()
        .show(&mut bottom, |ui| {
            let track = ui
                .allocate_space(Vec2::new(ui.available_width(), tokens.seek_track))
                .1;
            let position = chrome.seek_bar(ui, layout, track, tokens.seek_thickness);
            view().class("player-controls").row().show(ui, |ui| {
                chrome.trailing(ui, layout, small, position, &actions, false);
            });
            ui.add_space(tokens.controls_bottom);
        });
    let on = !player.upscaling.is_empty() && player.upscaling != "off";
    let side_height = icon * tokens.side_icons + Space::S16.px(&metrics);
    let mut side = chrome.centered(ui, "side", side_height);
    view().class("player-side").column().show(&mut side, |ui| {
        let anime = ui.allocate_space(Vec2::new(icon, icon)).1;
        if on {
            chrome.painter.circle_filled(
                anime.center(),
                tokens.glow,
                Color32::from_white_alpha(GLOW_ALPHA),
            );
        }
        chrome.button(
            ui,
            layout,
            anime.center(),
            icon,
            "Sparkles",
            NODE_PLAYER_ANIME4K,
        );
        chrome.slot(ui, layout, icon, icon, "Lock", NODE_PLAYER_LOCK);
    });
}

pub(super) fn tv_controls(chrome: &Chrome, ui: &mut egui::Ui, layout: &mut HomeLayout) {
    let (rect, player) = (chrome.rect, chrome.player);
    let tokens = tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    if chrome.locked(ui, layout) || !player.has_video() || !player.controls_visible {
        return;
    }
    scrim_edge(
        chrome.painter,
        rect,
        tokens.scrim_top.of(rect.height()),
        true,
        tokens.scrim_top_alpha as u8,
    );
    scrim_edge(
        chrome.painter,
        rect,
        tokens.scrim_bottom.of(rect.height()),
        false,
        tokens.scrim_bottom_alpha as u8,
    );
    let icon = chrome.icon();
    let mut top = chrome.band(ui, "top");
    let actions = chrome.header_actions();
    chrome.header(
        &mut top,
        layout,
        icon,
        icon,
        &actions,
        chrome.title_lines(player.episode_title.clone()),
    );
    let mut bottom = chrome.band(ui, "bottom");
    view()
        .class("player-bottom")
        .dock_bottom()
        .column()
        .show(&mut bottom, |ui| {
            let times_row = ui
                .allocate_space(Vec2::new(ui.available_width(), tokens.time_row))
                .1;
            let track = ui
                .allocate_space(Vec2::new(ui.available_width(), tokens.seek_track))
                .1;
            let position = chrome.seek_bar(ui, layout, track, tokens.seek_thickness);
            let times = chrome.times(ui, position);
            chrome.text(
                times_row.right_bottom(),
                Align2::RIGHT_BOTTOM,
                &times,
                tokens.times_text,
                times_row.width(),
                INK_TIMES,
            );
            let mut controls = chrome.transport();
            controls.extend(chrome.actions());
            view().class("player-controls").row().show(ui, |ui| {
                for (name, node) in controls {
                    chrome.slot(ui, layout, icon, icon, name, node);
                }
            });
            ui.add_space(tokens.controls_bottom);
        });
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
        u8::MAX
    } else {
        ICON_IDLE
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
                &SKIP_SECONDS.to_string(),
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
