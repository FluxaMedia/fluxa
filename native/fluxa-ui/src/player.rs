use super::*;
use crate::draw_text;

mod overlay;
mod panel;
mod stats;
mod tokens;

pub use overlay::{ChapterSpan, NextEpisodeCard, SegmentSpan, SkipCard, SkipKind};
pub use panel::{PanelRow, PlayerPanel};
pub use stats::StatsSection;

mod controls;
mod loading;
mod panels;
mod recommendations;
mod sources;
pub use controls::*;
pub use loading::*;
pub use panels::*;
pub use recommendations::*;
pub use sources::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerToast {
    pub text: String,
    pub level: Option<f32>,
    pub opacity: f32,
}

const FINE_SEEK_LIFT: f32 = 44.0;

#[derive(Clone, Debug, Default)]
pub struct PlayerModel {
    pub title: String,
    pub video: Option<TextureId>,
    pub passthrough: bool,
    pub subtitle: Option<String>,
    pub watch_button: bool,
    pub locked: bool,
    pub watch: Option<WatchTogetherModel>,
    pub status: Option<(String, String)>,
    pub error: Option<String>,
    pub position: f64,
    pub duration: f64,
    pub paused: bool,
    pub muted: bool,
    pub volume: f64,
    pub boost: bool,
    pub stats: Vec<StatsSection>,
    pub controls_visible: bool,
    pub show_pause_info: bool,
    pub logo: Option<String>,
    pub episode_title: Option<String>,
    pub episode_image: Option<String>,
    pub season_episode: Option<(i64, i64)>,
    pub synopsis: Option<String>,
    pub description: Option<String>,
    pub chapters: Vec<(f64, String)>,
    pub buffered: f64,
    pub chapter_spans: Vec<ChapterSpan>,
    pub segment_spans: Vec<SegmentSpan>,
    pub mark_preview: Option<SegmentSpan>,
    pub skip: Option<SkipCard>,
    pub next_episode: Option<NextEpisodeCard>,
    pub thumbnail: Option<(f64, TextureId)>,
    pub filmstrip: Vec<(f64, TextureId, Vec2)>,
    pub filmstrip_step: f64,
    pub warnings: Vec<(String, String)>,
    pub warnings_elapsed: Option<f32>,
    pub language: String,
    pub upscaling: String,
    pub recommendations: Vec<HomeHero>,
    pub recommendation_index: usize,
    pub background: Option<String>,
    pub load_progress: Option<f32>,
    pub scrub: Option<f64>,
    pub sources: Option<Vec<PlayerSource>>,
    pub source_filter: Option<String>,
    pub sources_loading: bool,
    pub sources_pending: Vec<String>,
    pub toast: Option<PlayerToast>,
    pub panel: Option<PlayerPanel>,
    pub dim: f32,
    pub options: PlayerOptions,
    pub has_next: bool,
    pub has_previous: bool,
    pub has_episodes: bool,
    pub trailer: bool,
    pub minimized: bool,
    pub minimizable: bool,
    pub pip: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerOptions {
    pub title: bool,
    pub up_next: bool,
    pub audio_subtitles: bool,
    pub speed: bool,
    pub episodes: bool,
    pub next_episode: bool,
    pub settings: bool,
    pub volume: bool,
    pub fullscreen: bool,
    pub cast: bool,
    pub mark_segment: bool,
    pub segments: bool,
    pub gestures: bool,
    pub double_tap: bool,
    pub center_controls: bool,
}

impl Default for PlayerOptions {
    fn default() -> Self {
        Self {
            title: true,
            up_next: true,
            audio_subtitles: true,
            speed: true,
            episodes: true,
            next_episode: true,
            settings: true,
            volume: true,
            fullscreen: true,
            cast: true,
            mark_segment: true,
            segments: true,
            gestures: true,
            double_tap: true,
            center_controls: true,
        }
    }
}

const PLAYER_ICONS: &[&str] = &[
    "ArrowLeft",
    "PlayFilled",
    "PauseFilled",
    "SkipForward",
    "SkipBack",
    "Volume",
    "VolumeMuted",
    "Zap",
    "ZapFilled",
    "Maximize",
    "Users",
    "AudioSubtitles",
    "Gauge",
    "Lock",
    "Unlock",
    "Settings",
    "ListVideo",
    "Cast",
    "Info",
    "Flag",
    "Sparkles",
    "Check",
    "ArrowRight",
    "PictureInPicture",
];

struct Icons(Vec<(&'static str, TextureId)>);

impl Icons {
    fn load(assets: &impl HomeAssets) -> Self {
        Self(
            PLAYER_ICONS
                .iter()
                .filter_map(|name| assets.icon(name).map(|texture| (*name, texture)))
                .collect(),
        )
    }

    fn get(&self, name: &str) -> Option<TextureId> {
        self.0
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, texture)| *texture)
    }
}

#[derive(Clone, Debug, Default)]
pub struct PlayerSource {
    pub addon: String,
    pub name: String,
    pub detail: String,
    pub badges: Vec<SourceBadge>,
    pub badges_top: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SourceBadge {
    pub label: String,
    pub fill: Option<Color32>,
    pub text: Option<Color32>,
    pub border: Option<Color32>,
    pub outline: bool,
    pub image: Option<String>,
}

const GESTURE_EDGE: f32 = 0.28;
const WARNING_BAR: f32 = 0.3;
const WARNING_STAGGER: f32 = 0.08;
const WARNING_FADE: f32 = 0.25;
const WARNING_HOLD: f32 = 5.0;

pub fn content_warning_duration(rows: usize) -> f32 {
    let rows = rows as f32 * WARNING_STAGGER + WARNING_FADE;
    WARNING_BAR * 2.0 + rows * 2.0 + WARNING_HOLD
}

impl PlayerModel {
    fn has_video(&self) -> bool {
        self.video.is_some() || self.passthrough
    }

    fn chapter_at(&self, time: f64) -> Option<&str> {
        self.chapters
            .iter()
            .rev()
            .find(|(start, _)| *start <= time)
            .map(|(_, title)| title.as_str())
            .filter(|title| !title.trim().is_empty())
    }

    pub fn addons(&self) -> Vec<&str> {
        let mut addons: Vec<&str> = Vec::new();
        for source in self.sources.iter().flatten() {
            if !addons.contains(&source.addon.as_str()) {
                addons.push(&source.addon);
            }
        }
        if self.sources_loading {
            for addon in &self.sources_pending {
                if !addons.contains(&addon.as_str()) {
                    addons.push(addon);
                }
            }
        }
        addons
    }

    fn visible_sources(&self) -> impl Iterator<Item = (usize, &PlayerSource)> {
        self.sources
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, source)| {
                self.source_filter
                    .as_deref()
                    .is_none_or(|addon| addon == source.addon)
            })
    }
}

pub fn draw_player(
    context: &egui::Context,
    viewport: Viewport,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let mut layout = HomeLayout::default();
    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    if !player.recommendations.is_empty() {
        draw_recommendations(context, viewport, rect, player, assets, &mut layout);
        draw_focus_ring(context, viewport, &layout, focused);
        return layout;
    }
    if player.sources.is_some() && !player.has_video() {
        draw_sources(context, viewport, rect, player, assets, &mut layout);
        draw_focus_ring(context, viewport, &layout, focused);
        return layout;
    }
    let painter = context.layer_painter(egui::LayerId::background());
    let (rect, side) = match player.watch.as_ref().filter(|_| !player.passthrough) {
        Some(_) => {
            let width = (rect.width() * 0.24).clamp(260.0, 340.0);
            let (left, right) = rect.split_left_right_at_x(rect.right() - width);
            let surface = UiMetrics::for_viewport(viewport).surface;
            context
                .layer_painter(egui::LayerId::background())
                .rect_filled(left, 0.0, surface);
            (left.shrink(40.0), Some(right))
        }
        None => (rect, None),
    };
    match player.video {
        Some(texture) if side.is_some() => {
            painter.rect_filled(rect, 18.0, Color32::BLACK);
            painter.image(texture, fit_16_9(rect), full_uv(), Color32::WHITE);
        }
        Some(texture) => {
            painter.rect_filled(rect, 0.0, Color32::BLACK);
            painter.image(texture, rect, full_uv(), Color32::WHITE);
        }
        None if player.passthrough => {}
        None => draw_loading(
            context,
            &painter,
            rect,
            player,
            assets,
            UiMetrics::for_viewport(viewport),
        ),
    }
    if player.pip {
        return layout;
    }
    if let Some(text) = &player.subtitle {
        let lift = if player.controls_visible { 150.0 } else { 56.0 };
        paint_subtitle_cue(&painter, rect, text, lift);
    }
    if player.show_pause_info {
        draw_pause_info(context, &painter, rect, player, assets);
    }
    let icons = Icons::load(assets);
    let chrome = Chrome {
        icons: &icons,
        context,
        painter: &painter,
        viewport,
        rect,
        player,
        focused,
    };
    if let Some(elapsed) = player.warnings_elapsed {
        let top = if player.controls_visible { 72.0 } else { 24.0 };
        draw_warnings(context, Pos2::new(chrome.margin(), top), player, elapsed);
    }
    stats::draw_stats(&chrome);
    let next_thumbnail = player
        .next_episode
        .as_ref()
        .and_then(|card| assets.texture(card.thumbnail.as_deref()));
    egui::Area::new(Id::new("fluxa-player-controls"))
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(rect.size());
            if player.trailer {
                trailer_controls(&chrome, ui, &mut layout);
            } else {
                match viewport.form_factor {
                    UiFormFactor::Tv => tv_controls(&chrome, ui, &mut layout),
                    UiFormFactor::Mobile => mobile_controls(&chrome, ui, &mut layout),
                    UiFormFactor::Desktop => desktop_controls(&chrome, ui, &mut layout),
                }
            }
            if player.has_video() {
                overlay::draw_skip_card(&chrome, ui, &mut layout);
                overlay::draw_next_episode_card(&chrome, ui, &mut layout, next_thumbnail);
            }
            if let Some(panel) = player.panel.as_ref() {
                panel::draw_panel(&chrome, ui, &mut layout, panel, assets);
            }
        });
    if let (Some(side), Some(model)) = (side, player.watch.as_ref()) {
        layout.watch_together = draw_watch_side_panel(context, side, model, assets);
    }
    if player.dim > 0.0 {
        let veil = context.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            Id::new("fluxa-player-dim"),
        ));
        veil.rect_filled(
            rect,
            0.0,
            Color32::from_black_alpha((player.dim * 255.0) as u8),
        );
    }
    if let Some(toast) = player.toast.as_ref() {
        components::toast(
            context,
            viewport,
            Pos2::new(rect.center().x, rect.top() + chrome.toast_top()),
            &toast.text,
            toast.level,
            toast.opacity,
        );
    }
    draw_focus_ring(context, viewport, &layout, focused);
    layout
}

struct Chrome<'a> {
    icons: &'a Icons,
    context: &'a egui::Context,
    painter: &'a egui::Painter,
    viewport: Viewport,
    rect: Rect,
    player: &'a PlayerModel,
    focused: Option<u64>,
}

impl Chrome<'_> {
    fn margin(&self) -> f32 {
        if self.viewport.form_factor == UiFormFactor::Tv {
            return (self.rect.width() * 0.05).max(48.0);
        }
        let min = if self.viewport.is_compact() {
            14.0
        } else {
            22.0
        };
        (self.rect.width() * 0.035).clamp(min, 64.0)
    }

    fn icon(&self) -> f32 {
        tokens::PlayerTokens::for_form_factor(self.viewport.form_factor).icon
    }

    fn icon_small(&self) -> f32 {
        self.icon() - 4.0
    }

    fn toast_top(&self) -> f32 {
        tokens::PlayerTokens::for_form_factor(self.viewport.form_factor).toast_top
    }

    fn button(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        center: Pos2,
        size: f32,
        icon: &str,
        node: u64,
    ) {
        let rect = Rect::from_center_size(center, Vec2::splat(size));
        let response = icon_control(
            ui,
            self.painter,
            rect,
            self.icons.get(icon),
            node,
            if node == NODE_PLAYER_TOGGLE {
                0.72
            } else {
                0.6
            },
        );
        components::focus_action(layout, node, &response);
    }

    fn text(&self, pos: Pos2, align: Align2, text: &str, size: f32, max_width: f32, alpha: u8) {
        let font = FontId::proportional(size);
        draw_text(
            self.painter,
            pos,
            align,
            text,
            font,
            ink(alpha),
            Some(max_width),
        );
    }

    fn surface(&self, ui: &mut egui::Ui, layout: &mut HomeLayout, area: Rect) {
        if self.player.panel.is_some() {
            return;
        }
        let mobile = self.viewport.form_factor == UiFormFactor::Mobile;
        let sense = if mobile {
            Sense::click_and_drag()
        } else {
            Sense::click()
        };
        let response = ui.interact(area, Id::new("fluxa-player-surface"), sense);
        if self.player.locked {
            if response.clicked() {
                layout.activated = Some(if self.player.controls_visible {
                    NODE_PLAYER_HIDE_CONTROLS
                } else {
                    NODE_PLAYER_CONTROLS
                });
            }
            return;
        }
        if !mobile {
            if response.double_clicked() {
                layout.activated = Some(NODE_PLAYER_FULLSCREEN);
            } else if response.clicked() {
                layout.activated = Some(NODE_PLAYER_TOGGLE);
            }
            return;
        }
        let held_id = Id::new("fluxa-player-speed-hold");
        let held: bool = ui.data(|data| data.get_temp(held_id)).unwrap_or(false);
        let pressing = response.is_pointer_button_down_on();
        let still =
            ui.input(
                |input| match (input.pointer.press_origin(), input.pointer.latest_pos()) {
                    (Some(origin), Some(now)) => origin.distance(now) < 12.0,
                    _ => false,
                },
            );
        let long = pressing
            && still
            && ui.input(|input| {
                input
                    .pointer
                    .press_start_time()
                    .is_some_and(|start| input.time - start > 0.45)
            });
        let holding = long || (held && pressing && still);
        if holding {
            ui.ctx().request_repaint();
        }
        ui.data_mut(|data| data.insert_temp(held_id, holding));
        layout.player_speed_hold = holding;
        if response.dragged() && self.player.options.gestures && !holding {
            let origin = ui
                .input(|input| input.pointer.press_origin())
                .or_else(|| response.interact_pointer_pos());
            let delta = response.drag_delta();
            if let Some(origin) = origin
                && delta.y.abs() > delta.x.abs()
            {
                let fraction = -delta.y / (area.height() * 0.8);
                let edge = area.width() * GESTURE_EDGE;
                if origin.x < area.left() + edge {
                    layout.player_gesture = Some(PlayerGesture::Brightness(fraction));
                } else if origin.x > area.right() - edge {
                    layout.player_gesture = Some(PlayerGesture::Volume(fraction));
                }
            }
            return;
        }
        if held {
            return;
        }
        if response.double_clicked()
            && self.player.options.double_tap
            && let Some(pointer) = response.interact_pointer_pos()
        {
            layout.activated = Some(if pointer.x < area.center().x {
                NODE_PLAYER_REWIND
            } else {
                NODE_PLAYER_FORWARD
            });
        } else if response.clicked() {
            layout.activated = Some(if self.player.controls_visible {
                NODE_PLAYER_HIDE_CONTROLS
            } else {
                NODE_PLAYER_CONTROLS
            });
        }
    }

    fn seek_bar(
        &self,
        ui: &mut egui::Ui,
        layout: &mut HomeLayout,
        track: Rect,
        thickness: f32,
    ) -> f64 {
        let player = self.player;
        let seek = ui.interact(
            track.expand2(Vec2::new(0.0, 10.0)),
            Id::new("fluxa-player-seek"),
            Sense::click_and_drag(),
        );
        layout.focusable.push((NODE_PLAYER_SEEK, track));
        let duration = player.duration.max(0.0);
        let ratio = |x: f32| ((x - track.left()) / track.width()).clamp(0.0, 1.0) as f64;
        let mut position = player.scrub.unwrap_or(player.position);
        let metrics = UiMetrics::for_viewport(self.viewport);
        let form_factor = self.viewport.form_factor;
        let lifted = form_factor == UiFormFactor::Mobile
            && duration > 0.0
            && seek
                .interact_pointer_pos()
                .filter(|_| seek.dragged())
                .is_some_and(|pointer| pointer.y < track.top() - FINE_SEEK_LIFT);
        if (seek.clicked() || seek.dragged())
            && !lifted
            && let Some(pointer) = seek.interact_pointer_pos()
        {
            position = duration * ratio(pointer.x);
            layout.seek_to = Some(position);
        }
        let drag_id = Id::new("fluxa-filmstrip-drag");
        let last_id = Id::new("fluxa-filmstrip-last");
        let mut forced = None;
        if lifted && let Some(pointer) = seek.interact_pointer_pos() {
            let (start_time, start_x) = self
                .context
                .data(|data| data.get_temp::<(f64, f32)>(drag_id))
                .unwrap_or((duration * ratio(pointer.x), pointer.x));
            let pitch =
                components::filmstrip_pitch(self.context, track, &player.filmstrip, metrics);
            let center = (start_time
                + ((pointer.x - start_x) / pitch) as f64 * player.filmstrip_step.max(1.0))
            .clamp(0.0, duration);
            self.context.data_mut(|data| {
                data.insert_temp(drag_id, (start_time, start_x));
                data.insert_temp(last_id, Some(center));
            });
            forced = Some(center);
            position = center;
        } else {
            let last = self
                .context
                .data_mut(|data| {
                    data.remove_temp::<(f64, f32)>(drag_id);
                    let last = data.get_temp::<Option<f64>>(last_id).flatten();
                    data.insert_temp(last_id, None::<f64>);
                    last
                })
                .filter(|_| seek.drag_stopped());
            if let Some(time) = last {
                position = time;
                layout.seek_to = Some(time);
            }
        }
        if form_factor == UiFormFactor::Tv && duration > 0.0 {
            forced = player.scrub;
        }
        let y = track.center().y;
        if duration <= 0.0 {
            painter_bar(
                self.painter,
                track,
                thickness,
                ink(tone::DIM),
                track.width(),
            );
            return position;
        }
        let played = track.width() * (position / duration).clamp(0.0, 1.0) as f32;
        overlay::segmented_track(
            self.painter,
            track,
            thickness,
            &player.chapter_spans,
            played,
            (track.width() * ((position + player.buffered) / duration).clamp(0.0, 1.0) as f32)
                .max(played),
        );
        if player.options.segments {
            overlay::segment_tints(self.painter, track, thickness, &player.segment_spans);
        }
        if let Some(preview) = player.mark_preview.as_ref() {
            overlay::segment_tints(
                self.painter,
                track,
                thickness,
                std::slice::from_ref(preview),
            );
        }
        let active = seek.hovered()
            || seek.dragged()
            || player.scrub.is_some()
            || self.focused == Some(NODE_PLAYER_SEEK);
        if active {
            self.painter.circle_filled(
                Pos2::new(track.left() + played, y),
                thickness * 1.8,
                accent(),
            );
        }
        let pointer = if self.viewport.form_factor == UiFormFactor::Mobile {
            seek.interact_pointer_pos().filter(|_| seek.dragged())
        } else {
            seek.hover_pos()
        };
        let preview = pointer
            .map(|pointer| (pointer.x, duration * ratio(pointer.x)))
            .or_else(|| player.scrub.map(|time| (track.left() + played, time)));
        let over_strip = components::filmstrip_area(self.context)
            .zip(self.context.input(|i| i.pointer.hover_pos()))
            .is_some_and(|(area, pointer)| area.contains(pointer));
        let filmstrip = forced.is_some()
            || (form_factor == UiFormFactor::Desktop
                && (seek.hovered() || over_strip)
                && duration > 0.0
                && self.context.input(|i| i.modifiers.shift));
        if filmstrip {
            let output = components::seek_filmstrip(
                self.context,
                track,
                seek.hover_pos().map(|pointer| duration * ratio(pointer.x)),
                duration,
                player.filmstrip_step,
                forced,
                if lifted { FINE_SEEK_LIFT * 2.0 } else { 0.0 },
                &player.filmstrip,
                metrics,
            );
            layout.filmstrip = Some(output.slots);
            if let Some(time) = output.seek {
                layout.seek_to = Some(time);
            }
        } else {
            components::filmstrip_close(self.context);
            if let Some((x, time)) = preview {
                layout.seek_hover = Some(time);
                draw_seek_preview(
                    self.context,
                    track,
                    x,
                    time,
                    player,
                    UiMetrics::for_viewport(self.viewport),
                );
            }
        }
        position
    }

    fn skippable(&self, from: f64) -> f64 {
        let duration = self.player.duration.max(0.0);
        self.player
            .segment_spans
            .iter()
            .map(|span| {
                let start = (span.start_fraction as f64 * duration).max(from);
                let end = span.end_fraction as f64 * duration;
                (end - start).max(0.0)
            })
            .sum()
    }

    fn remaining_mode(&self, ui: &egui::Ui) -> bool {
        ui.data(|data| data.get_temp(Id::new("fluxa-player-remaining")))
            .unwrap_or(false)
    }

    fn toggle_remaining(&self, ui: &mut egui::Ui, rect: Rect) {
        if ui
            .interact(rect, Id::new("fluxa-player-remaining-hit"), Sense::click())
            .clicked()
        {
            let id = Id::new("fluxa-player-remaining");
            let now = self.remaining_mode(ui);
            ui.data_mut(|data| data.insert_temp(id, !now));
        }
    }

    fn with_skips(&self, total: f64, skipped: f64, sign: &str) -> String {
        let base = format!("{sign}{}", format_time(total));
        if skipped >= 1.0 {
            format!("{base} ({sign}{})", format_time((total - skipped).max(0.0)))
        } else {
            base
        }
    }

    fn elapsed_label(&self, ui: &egui::Ui, position: f64) -> String {
        if self.remaining_mode(ui) {
            let left = (self.player.duration - position).max(0.0);
            self.with_skips(left, self.skippable(position), "-")
        } else {
            format_time(position)
        }
    }

    fn duration_label(&self) -> String {
        let duration = self.player.duration.max(0.0);
        self.with_skips(duration, self.skippable(0.0), "")
    }

    fn times(&self, ui: &egui::Ui, position: f64) -> String {
        let times = format!(
            "{} / {}",
            self.elapsed_label(ui, position),
            self.duration_label()
        );
        match self.player.chapter_at(position) {
            Some(chapter) => format!("{times}  {chapter}"),
            None => times,
        }
    }
}

pub fn format_time(seconds: f64) -> String {
    let total = seconds.max(0.0).floor() as u64;
    let (hours, minutes, seconds) = (total / 3600, (total / 60) % 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub fn torrent_status_lines(status: Option<&serde_json::Value>) -> (String, String) {
    let Some(status) = status else {
        return (
            "Resolving torrent metadata…".to_owned(),
            "Connecting to trackers and discovering peers".to_owned(),
        );
    };
    let text = |key: &str| status.get(key).and_then(serde_json::Value::as_str);
    let number = |key: &str| {
        status
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    if status.get("stat").and_then(serde_json::Value::as_i64) == Some(-1) {
        return (
            "Torrent stream reported an error".to_owned(),
            text("error")
                .filter(|error| !error.trim().is_empty())
                .unwrap_or("Still waiting for the selected torrent")
                .to_owned(),
        );
    }
    let phase = text("phase")
        .or_else(|| text("stat_string"))
        .unwrap_or("resolving_metadata");
    if phase == "initializing"
        && status.get("resolving").and_then(serde_json::Value::as_bool) == Some(false)
        && text("hash").is_none_or(str::is_empty)
    {
        return (
            "Torrent metadata has not resolved yet".to_owned(),
            "The selected source is still waiting for a metadata response".to_owned(),
        );
    }
    let peers = number("active_peers");
    let peer_total = number("total_peers");
    let percent = status
        .get("preload")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0)
        .round()
        .clamp(0.0, 100.0) as u64;
    let downloaded = number("loaded_size");
    let speed = status
        .get("download_speed")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let headline = match phase {
        "resolving_metadata" | "initializing" | "resolving" => "Fetching torrent metadata…",
        "connecting_peers" => "Discovering peers…",
        "buffering_startup" => "Downloading startup buffer…",
        "rebuffering" => "Refilling playback buffer…",
        "seeking" => "Preparing the requested playback position…",
        "streaming" => "Torrent buffer ready · starting video…",
        "stalled" => "Torrent transfer is paused",
        "error" => "Torrent stream reported an error",
        "status_unavailable" => "Checking torrent engine status…",
        _ if peers == 0 => "Discovering peers…",
        _ => "Downloading torrent data…",
    };
    if phase == "status_unavailable" {
        return (
            headline.to_owned(),
            text("error")
                .unwrap_or("No response from the torrent status endpoint")
                .to_owned(),
        );
    }
    let peers = if peer_total > 0 {
        format!("{peers}/{peer_total} peers")
    } else {
        format!("{peers} peers")
    };
    let mib = 1024.0 * 1024.0;
    let downloaded = if downloaded as f64 >= mib {
        format!("{:.1} MiB downloaded", downloaded as f64 / mib)
    } else {
        format!("{} KiB downloaded", downloaded / 1024)
    };
    let speed = if speed >= mib {
        format!("{:.1} MiB/s", speed / mib)
    } else if speed >= 1024.0 {
        format!("{:.0} KiB/s", speed / 1024.0)
    } else {
        format!("{speed:.0} B/s")
    };
    (
        headline.to_owned(),
        format!("{peers} · {percent}% buffer · {downloaded} · {speed}"),
    )
}

fn fit_16_9(area: Rect) -> Rect {
    let wide = area.width() / area.height().max(1.0) > 16.0 / 9.0;
    let size = if wide {
        Vec2::new(area.height() * 16.0 / 9.0, area.height())
    } else {
        Vec2::new(area.width(), area.width() * 9.0 / 16.0)
    };
    Rect::from_center_size(area.center(), size)
}
