use super::*;

mod overlay;
mod panel;

pub use overlay::{ChapterSpan, NextEpisodeCard, SkipCard, SkipKind};
pub use panel::{PanelRow, PlayerPanel};

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

#[derive(Clone, Debug, Default)]
pub struct PlayerModel {
    pub title: String,
    pub video: Option<TextureId>,
    pub passthrough: bool,
    pub status: Option<(String, String)>,
    pub error: Option<String>,
    pub position: f64,
    pub duration: f64,
    pub paused: bool,
    pub muted: bool,
    pub volume: f64,
    pub controls_visible: bool,
    pub show_pause_info: bool,
    pub logo: Option<String>,
    pub episode_title: Option<String>,
    pub description: Option<String>,
    pub chapters: Vec<(f64, String)>,
    pub chapter_spans: Vec<ChapterSpan>,
    pub skip: Option<SkipCard>,
    pub next_episode: Option<NextEpisodeCard>,
    pub thumbnail: Option<(f64, TextureId)>,
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
    pub toast: Option<PlayerToast>,
    pub panel: Option<PlayerPanel>,
    pub dim: f32,
    pub options: PlayerOptions,
    pub has_next: bool,
    pub has_previous: bool,
    pub has_episodes: bool,
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
    pub upscaling: bool,
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
            upscaling: true,
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
    "Maximize",
    "AudioSubtitles",
    "Gauge",
    "Sliders",
    "List",
    "Cast",
    "Flag",
    "Sparkles",
    "Check",
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
}

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
        draw_focus_ring(context, &layout, focused);
        return layout;
    }
    if player.sources.is_some() && !player.has_video() {
        draw_sources(context, viewport, rect, player, assets, &mut layout);
        draw_focus_ring(context, &layout, focused);
        return layout;
    }
    let painter = context.layer_painter(egui::LayerId::background());
    match player.video {
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
    let next_thumbnail = player
        .next_episode
        .as_ref()
        .and_then(|card| assets.texture(card.thumbnail.as_deref()));
    egui::Area::new(Id::new("fluxa-player-controls"))
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(rect.size());
            match viewport.form_factor {
                UiFormFactor::Tv => tv_controls(&chrome, ui, &mut layout),
                UiFormFactor::Mobile => mobile_controls(&chrome, ui, &mut layout),
                UiFormFactor::Desktop => desktop_controls(&chrome, ui, &mut layout),
            }
            if player.has_video() {
                overlay::draw_skip_card(&chrome, ui, &mut layout);
                overlay::draw_next_episode_card(&chrome, ui, &mut layout, next_thumbnail);
            }
            if let Some(panel) = player.panel.as_ref() {
                panel::draw_panel(&chrome, ui, &mut layout, panel);
            }
        });
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
    draw_focus_ring(context, &layout, focused);
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
        let min = if self.viewport.is_compact() {
            14.0
        } else {
            22.0
        };
        (self.rect.width() * 0.035).clamp(min, 64.0)
    }

    fn toast_top(&self) -> f32 {
        match self.viewport.form_factor {
            UiFormFactor::Tv => 48.0,
            UiFormFactor::Mobile => 30.0,
            UiFormFactor::Desktop => 28.0,
        }
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
            if node == NODE_PLAYER_TOGGLE { 0.6 } else { 0.5 },
        );
        layout.focusable.push((node, response.rect));
        if response.clicked() {
            layout.activated = Some(node);
        }
    }

    fn text(&self, pos: Pos2, align: Align2, text: &str, size: f32, max_width: f32, alpha: u8) {
        let font = FontId::proportional(size);
        self.painter.text(
            pos,
            align,
            truncate_to_width(self.painter, text, &font, max_width),
            font,
            Color32::from_white_alpha(alpha),
        );
    }

    fn surface(&self, ui: &mut egui::Ui, layout: &mut HomeLayout, area: Rect) {
        let mobile = self.viewport.form_factor == UiFormFactor::Mobile;
        let sense = if mobile {
            Sense::click_and_drag()
        } else {
            Sense::click()
        };
        let response = ui.interact(area, Id::new("fluxa-player-surface"), sense);
        if !mobile {
            if response.clicked() {
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
        if response.dragged()
            && self.player.options.gestures
            && !holding
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let delta = response.drag_delta();
            if delta.y.abs() > delta.x.abs() {
                let fraction = -delta.y / (area.height() * 0.8);
                layout.player_gesture = Some(if pointer.x < area.center().x {
                    PlayerGesture::Brightness(fraction)
                } else {
                    PlayerGesture::Volume(fraction)
                });
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
        if (seek.clicked() || seek.dragged())
            && let Some(pointer) = seek.interact_pointer_pos()
        {
            position = duration * ratio(pointer.x);
            layout.seek_to = Some(position);
        }
        let y = track.center().y;
        if duration <= 0.0 {
            painter_bar(
                self.painter,
                track,
                thickness,
                Color32::from_white_alpha(95),
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
        );
        let active = seek.hovered()
            || seek.dragged()
            || player.scrub.is_some()
            || self.focused == Some(NODE_PLAYER_SEEK);
        if active {
            self.painter.circle_filled(
                Pos2::new(track.left() + played, y),
                thickness * 1.8,
                Color32::WHITE,
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
        position
    }

    fn times(&self, position: f64) -> String {
        format!(
            "{}  /  {}",
            format_time(position),
            format_time(self.player.duration.max(0.0))
        )
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
