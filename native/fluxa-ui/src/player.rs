use super::*;
use crate::draw_text;
use crate::flex;

mod overlay;
mod panel;
mod stats;

mod tokens;

pub use overlay::{ChapterSpan, NextEpisodeCard, SegmentSpan, SkipCard, SkipKind};
pub use panel::{PanelRow, PanelStep, PlayerPanel, SyncRole};
pub use stats::StatsSection;
pub use tokens::subtitle_lift;

mod controls;
mod loading;
mod panels;
mod sources;

pub(crate) use controls::*;
use loading::*;
use panels::*;
use sources::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerToast {
    pub text: String,
    pub level: Option<f32>,
    pub opacity: f32,
    pub gauge: Option<ToastGauge>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToastGauge {
    Brightness,
    Volume,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerModel {
    pub title: String,
    pub video: Option<TextureId>,
    pub video_hole: bool,
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
    pub sources: Option<std::sync::Arc<Vec<PlayerSource>>>,
    pub playing_badges: Vec<SourceBadge>,
    pub source_filter: Option<String>,
    pub source_chips_offset: f32,
    pub sources_loading: bool,
    pub sources_pending: Vec<String>,
    pub toast: Option<PlayerToast>,
    pub panel: Option<PlayerPanel>,
    pub dim: f32,
    pub options: PlayerOptions,
    pub has_next: bool,
    pub has_previous: bool,
    pub has_episodes: bool,
    pub can_change_source: bool,
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
    pub chapters: bool,
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
            chapters: true,
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
    "Sun",
    "Rewind10",
    "Forward10",
    "Zap",
    "ZapFilled",
    "CornersOut",
    "Users",
    "AudioSubtitles",
    "Gauge",
    "Lock",
    "Unlock",
    "Settings",
    "Queue",
    "Cast",
    "Info",
    "Swap",
    "Flag",
    "Sparkles",
    "Check",
    "ArrowRight",
    "PictureInPicture",
    "AudioLines",
    "Captions",
    "MessageSquare",
    "ChevronUp",
    "ChevronDown",
    "ChevronRight",
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
    pub actions: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SourceBadge {
    pub label: String,
    pub fill: Option<Color32>,
    pub text: Option<Color32>,
    pub border: Option<Color32>,
    pub outline: bool,
    pub image: Option<String>,
    pub essential: bool,
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
        for source in self.sources.iter().flat_map(|sources| sources.iter()) {
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
            .flat_map(|sources| sources.iter())
            .enumerate()
            .filter(|(_, source)| {
                self.source_filter
                    .as_deref()
                    .is_none_or(|addon| addon == source.addon)
            })
    }
}

pub fn is_sources_modal(viewport: Viewport, player: &PlayerModel) -> bool {
    sources_modal(viewport)
        && player.recommendations.is_empty()
        && player.sources.is_some()
        && !player.has_video()
}

pub fn player_skinnable(player: &PlayerModel) -> bool {
    player.recommendations.is_empty() && !player.minimized
}

pub fn draw_player_with(
    context: &crate::gfx::Context,
    viewport: Viewport,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let layout = HomeLayout::default();
    let rect = crate::gfx::rect_at(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    if !player.recommendations.is_empty() || (player.sources.is_some() && !player.has_video()) {
        return layout;
    }
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    let (rect, side) = match player.watch.as_ref().filter(|_| !player.passthrough) {
        Some(_) => {
            let width = if viewport.form_factor == UiFormFactor::Tv {
                UiMetrics::for_viewport(viewport)
                    .player_panel_wide
                    .of(rect.width())
            } else {
                UiMetrics::for_viewport(viewport)
                    .player_panel_narrow
                    .of(rect.width())
            };
            let (left, right) = rect.split_left_right_at_x(rect.right() - width);
            let surface = UiMetrics::for_viewport(viewport).surface;
            context
                .layer_painter(crate::gfx::LayerId::background())
                .rect_filled(left, 0.0, surface);
            (left.shrink(40.0), Some(right))
        }
        None => (rect, None),
    };
    match player.video {
        Some(_) if player.video_hole && side.is_some() => {
            painter.rect_filled(rect, 18.0, Color32::BLACK);
            crate::components::dom_video(&painter, fit_16_9(rect));
        }
        Some(_) if player.video_hole => {
            painter.rect_filled(rect, 0.0, Color32::BLACK);
            crate::components::dom_video(&painter, rect);
        }
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
    if player.has_video() {
        draw_buffering(
            context,
            &painter,
            rect,
            player,
            assets,
            UiMetrics::for_viewport(viewport),
        );
    }
    if let Some(text) = &player.subtitle {
        let lift = if player.controls_visible { 150.0 } else { 56.0 };
        paint_subtitle_cue(&painter, rect, text, lift);
    }
    if player.show_pause_info {
        draw_pause_info(context, &painter, rect, player, assets);
    }
    if player.controls_visible
        && let Some(Some((track, pointer_x, time))) = context
            .data(|data| data.get_temp::<Option<(Rect, f32, f64)>>(Id::new("player-seek-hover")))
    {
        draw_seek_preview(
            context,
            track,
            pointer_x,
            time,
            player,
            UiMetrics::for_viewport(viewport),
        );
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
        seek_track: std::cell::Cell::new(None),
    };
    crate::kit::provide(context, UiMetrics::for_viewport(viewport));
    stats::draw_stats(&chrome);
    if player.has_video() {
        crate::gfx::Area::new(Id::new("fluxa-player-controls"))
            .fixed_pos(Pos2::ZERO)
            .interactable(false)
            .show(context, |ui| {
                ui.set_min_size(rect.size());
                overlay::draw_playing_badges(&chrome, ui, assets);
            });
    }
    if player.dim > 0.0 {
        let veil = context.layer_painter(crate::gfx::LayerId::new(
            crate::gfx::Order::Foreground,
            Id::new("fluxa-player-dim"),
        ));
        veil.rect_filled(
            rect,
            0.0,
            Color32::from_black_alpha((player.dim * 255.0) as u8),
        );
    }
    if let Some(toast) = player.toast.as_ref() {
        match toast.gauge.zip(toast.level) {
            Some((gauge, level)) if viewport.form_factor == UiFormFactor::Mobile => {
                chrome.gauge(gauge, level, toast.opacity);
            }
            _ => components::toast(
                context,
                viewport,
                Pos2::new(rect.center().x, rect.top() + chrome.toast_top()),
                &toast.text,
                toast.level,
                toast.opacity,
            ),
        }
    }
    layout
}

struct Chrome<'a> {
    icons: &'a Icons,
    context: &'a crate::gfx::Context,
    painter: &'a crate::gfx::Painter,
    viewport: Viewport,
    rect: Rect,
    player: &'a PlayerModel,
    focused: Option<u64>,
    seek_track: std::cell::Cell<Option<Rect>>,
}

impl Chrome<'_> {
    fn margin(&self) -> f32 {
        tokens::PlayerTokens::for_form_factor(self.viewport.form_factor)
            .margin
            .of(self.rect.width())
    }

    fn toast_top(&self) -> f32 {
        Space::S32.px(&UiMetrics::for_viewport(self.viewport))
    }

    fn gauge(&self, gauge: ToastGauge, level: f32, opacity: f32) {
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
        let height = self.rect.height() * 0.34;
        let width = 6.0;
        let (icon, left_side) = match gauge {
            ToastGauge::Brightness => ("Sun", true),
            ToastGauge::Volume => ("Volume", false),
        };
        let edge = self.margin() + 4.0;
        let size = 28.0;
        let items = if left_side {
            [
                flex::leaf(Vec2::new(width, height)).id(0),
                flex::leaf(Vec2::splat(size)).id(1),
            ]
        } else {
            [
                flex::leaf(Vec2::splat(size)).id(1),
                flex::leaf(Vec2::new(width, height)).id(0),
            ]
        };
        let placed = flex::row(items)
            .gap(14.0)
            .align(flex::Align::Center)
            .justify(if left_side {
                flex::Justify::Start
            } else {
                flex::Justify::End
            })
            .padding(flex::Insets::xy(edge, 0.0))
            .solve(self.rect);
        let track = placed.find(0).unwrap_or(self.rect);
        self.painter
            .rect_filled(track, width * 0.5, Color32::from_white_alpha(alpha / 3));
        let filled = height * level.clamp(0.0, 1.0);
        let fill = Rect::from_min_max(Pos2::new(track.left(), track.bottom() - filled), track.max);
        self.painter
            .rect_filled(fill, width * 0.5, Color32::from_white_alpha(alpha));
        if let Some(texture) = self.icons.get(icon) {
            self.painter.image(
                texture,
                placed.find(1).unwrap_or(self.rect),
                full_uv(),
                Color32::from_white_alpha(alpha),
            );
        }
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
    flex::anchored(area, Align2::CENTER_CENTER, size, 0.0)
}

pub fn player_data(player: &PlayerModel) -> serde_json::Value {
    player_data_for(player, false)
}

pub fn player_data_for(player: &PlayerModel, mobile: bool) -> serde_json::Value {
    let language = player.language.as_str();
    let t = |key: &str| localized(key, language);
    let episode = player
        .season_episode
        .map(|(season, number)| format!("S{season}E{number}"))
        .into_iter()
        .chain(player.episode_title.clone())
        .collect::<Vec<_>>()
        .join("  ·  ");
    let position = player.scrub.unwrap_or(player.position);
    serde_json::json!({
        "title": player.title,
        "episode": episode,
        "visible": !player.pip && (!player.has_video() || player.controls_visible || player.paused || player.panel.is_some()),
        "hasVideo": player.has_video(),
        "trailer": player.trailer,
        "controlsOn": player.controls_visible,
        "chapterMarks": if player.options.chapters {
            player
                .chapter_spans
                .iter()
                .skip(1)
                .map(|chapter| serde_json::json!({"start": chapter.start_fraction, "span": 0.0015}))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        },
        "segmentBands": if player.options.segments {
            player
                .segment_spans
                .iter()
                .map(|segment| serde_json::json!({
                    "start": segment.start_fraction,
                    "span": (segment.end_fraction - segment.start_fraction).max(0.0),
                    "tint": segment.kind.band(),
                }))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        },
        "panelOpen": player.panel.is_some(),
        "paused": player.paused,
        "playIcon": if player.paused { "PlayFilled" } else { "PauseFilled" },
        "muted": player.muted,
        "muteIcon": if player.muted || player.volume <= 0.0 { "VolumeMuted" } else { "Volume" },
        "position": position,
        "duration": player.duration,
        "buffered": player.buffered,
        "played": if player.duration > 0.0 { (position / player.duration).clamp(0.0, 1.0) } else { 0.0 },
        "bufferedTo": if player.duration > 0.0 { ((position + player.buffered) / player.duration).clamp(0.0, 1.0) } else { 0.0 },
        "elapsed": format_time(position),
        "remaining": format_time((player.duration - position).max(0.0)),
        "hasPrevious": player.has_previous,
        "hasNext": player.has_next,
        "hasEpisodes": player.has_episodes && player.options.episodes,
        "audioSubtitles": player.options.audio_subtitles,
        "speed": player.options.speed,
        "settings": player.options.settings,
        "volume": player.options.volume,
        "fullscreen": player.options.fullscreen,
        "cast": player.options.cast,
        "info": !player.title.is_empty(),
        "panel": player.panel.as_ref().map(|panel| panel_data(panel, mobile, language)),
        "watch": player.watch.as_ref().map(crate::watch_data),
        "skip": player.skip.as_ref().map(|card| serde_json::json!({
            "label": t(card.kind.label_key()),
            "progress": card.progress,
        })),
        "next": player.next_episode.as_ref().map(|card| serde_json::json!({
            "label": t("player.next_label").replace("%s", &card.label),
            "title": card.title,
            "thumb": card.thumbnail,
            "play": match card.countdown {
                Some(seconds) => t("player.playing_in_seconds").replace("%s", &seconds.to_string()),
                None => t("player.play"),
            },
            "cancel": t("common.cancel"),
        })),
        "warnings": if player.warnings_elapsed.is_some() {
            player.warnings.iter().map(|(label, severity)| serde_json::json!({"text": format!("{label} · {severity}")})).collect()
        } else {
            Vec::new()
        },
        "labels": {
            "close": t("common.close"),
        },
    })
}

pub fn sources_data(player: &PlayerModel) -> serde_json::Value {
    let language = player.language.as_str();
    let t = |key: &str| localized(key, language);
    let hex = |color: Option<Color32>| {
        color
            .map(|color| format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b()))
            .unwrap_or_default()
    };
    let show_addon = player.source_filter.is_none();
    let rows: Vec<_> = player
        .visible_sources()
        .map(|(index, source)| {
            let badges: Vec<_> = source
                .badges
                .iter()
                .map(|badge| {
                    serde_json::json!({
                        "label": badge.label,
                        "fill": if badge.outline { String::new() } else { hex(badge.fill) },
                        "text": hex(badge.text),
                        "border": hex(badge.border.or(badge.outline.then_some(badge.fill).flatten())),
                    })
                })
                .collect();
            serde_json::json!({
                "node": (NODE_PLAYER_SOURCE_BASE + index as u64).to_string(),
                "name": source.name,
                "detail": source.detail,
                "addon": if show_addon { source.addon.clone() } else { String::new() },
                "badges": badges,
                "badgesTop": source.badges_top,
                "badgesBottom": !source.badges_top,
                "actions": source.actions,
                "copy": (NODE_PLAYER_SOURCE_COPY_BASE + index as u64).to_string(),
                "download": (NODE_PLAYER_SOURCE_DOWNLOAD_BASE + index as u64).to_string(),
            })
        })
        .collect();
    let addons = player.addons();
    let filters: Vec<_> = std::iter::once((None, t("player.streams_all")))
        .filter(|_| !addons.is_empty())
        .chain(addons.iter().map(|addon| (Some(*addon), addon.to_string())))
        .enumerate()
        .map(|(index, (addon, label))| {
            serde_json::json!({
                "node": (NODE_PLAYER_SOURCE_FILTER_BASE + index as u64).to_string(),
                "label": label,
                "selected": player.source_filter.as_deref() == addon,
            })
        })
        .collect();
    let season = player.season_episode.map(|(season, episode)| {
        t("player.next_episode_number_format")
            .replacen("%s", &season.to_string(), 1)
            .replacen("%s", &episode.to_string(), 1)
    });
    let waiting = player.sources_loading && !player.sources_pending.is_empty();
    serde_json::json!({
        "title": player.title,
        "logo": player.logo,
        "backdrop": player.episode_image.as_deref().or(player.background.as_deref()),
        "season": season,
        "episode": player.episode_title,
        "overview": player.synopsis,
        "waiting": if waiting {
            t("player.streams_waiting").replace("%s", &player.sources_pending.join(", "))
        } else {
            String::new()
        },
        "empty": rows.is_empty() && !player.sources_loading,
        "emptyText": t("player.streams_empty"),
        "loading": player.sources_loading,
        "retryNode": NODE_PLAYER_SOURCES_RETRY.to_string(),
        "closeNode": NODE_PLAYER_CLOSE.to_string(),
        "filters": filters,
        "rows": rows,
    })
}

pub fn recommendations_data(player: &PlayerModel) -> serde_json::Value {
    let language = player.language.as_str();
    let count = player.recommendations.len();
    let index = player.recommendation_index.min(count.saturating_sub(1));
    let Some(hero) = player.recommendations.get(index) else {
        return serde_json::Value::Null;
    };
    let dots: Vec<_> = (0..count)
        .filter(|_| count > 1)
        .map(|dot| {
            serde_json::json!({
                "node": (NODE_PLAYER_RECOMMENDATION_BASE + dot as u64).to_string(),
                "active": dot == index,
            })
        })
        .collect();
    serde_json::json!({
        "backdrop": hero.background_url,
        "logo": hero.logo_url.as_deref().filter(|url| !url.trim().is_empty()),
        "title": hero.title,
        "eyebrow": hero.eyebrow,
        "description": hero.description,
        "dots": dots,
        "play": localized("common.play", language),
        "details": localized("hero.more_info", language),
        "playNode": NODE_PLAYER_RECOMMENDATION_PLAY.to_string(),
        "detailsNode": NODE_PLAYER_RECOMMENDATION_DETAILS.to_string(),
        "closeNode": NODE_PLAYER_RECOMMENDATIONS_CLOSE.to_string(),
    })
}

pub fn has_recommendations_view(player: &PlayerModel) -> bool {
    !player.recommendations.is_empty()
}

pub fn has_sources_view(player: &PlayerModel) -> bool {
    player.recommendations.is_empty() && player.sources.is_some() && !player.has_video()
}

fn color_hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

fn sync_data(panel: &PlayerPanel) -> serde_json::Value {
    let find = |role: SyncRole| {
        panel
            .rows
            .iter()
            .enumerate()
            .find(|(_, row)| row.sync == Some(role))
    };
    if find(SyncRole::Reading).is_none() {
        return serde_json::Value::Null;
    }
    let item = |role: SyncRole| {
        find(role).map(|(index, row)| {
            serde_json::json!({"index": index, "label": row.label, "detail": row.detail})
        })
    };
    let reading = find(SyncRole::Reading).map(|(_, row)| {
        let (number, suffix) = row
            .label
            .strip_suffix('s')
            .map_or((row.label.as_str(), ""), |number| (number, "s"));
        serde_json::json!({"number": number, "suffix": suffix, "status": row.value})
    });
    let rows = |role: SyncRole| -> Vec<_> {
        panel
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.sync == Some(role))
            .map(|(index, row)| {
                serde_json::json!({
                    "index": index,
                    "label": row.label,
                    "detail": row.detail,
                    "selected": row.selected,
                })
            })
            .collect()
    };
    serde_json::json!({
        "back": item(SyncRole::Back),
        "reset": item(SyncRole::Reset),
        "reading": reading,
        "minus": item(SyncRole::Minus),
        "plus": item(SyncRole::Plus),
        "auto": item(SyncRole::Auto),
        "methods": rows(SyncRole::Method),
        "choices": rows(SyncRole::Choice),
    })
}

fn panel_data(panel: &PlayerPanel, mobile: bool, language: &str) -> serde_json::Value {
    let row_data = |index: usize, row: &PanelRow| {
        let kind = match row.step {
            Some(PanelStep::Value) => "value",
            Some(PanelStep::Swatches(_)) => "swatches",
            Some(PanelStep::Choices(_)) => "choices",
            _ => "plain",
        };
        let trail: Vec<_> = match row.step {
            Some(PanelStep::Swatches(count) | PanelStep::Choices(count)) => (1..=count)
                .filter_map(|offset| {
                    let part = panel.rows.get(index + offset)?;
                    Some(serde_json::json!({
                        "index": index + offset,
                        "label": part.label,
                        "color": part.value,
                        "selected": part.selected,
                    }))
                })
                .collect(),
            _ => Vec::new(),
        };
        let mut value = serde_json::Map::new();
        value.insert("index".into(), index.into());
        value.insert("label".into(), row.label.clone().into());
        value.insert("value".into(), row.value.clone().into());
        value.insert("selected".into(), row.selected.into());
        value.insert("primary".into(), row.primary.into());
        value.insert("action".into(), true.into());
        value.insert("switch".into(), row.switch.is_some().into());
        value.insert("on".into(), row.switch.unwrap_or(false).into());
        value.insert(
            "detail".into(),
            row.detail.clone().or_else(|| row.subline.clone()).into(),
        );
        value.insert("thumb".into(), row.thumbnail.clone().into());
        value.insert("badge".into(), row.badge.clone().into());
        value.insert("icon".into(), row.icon.clone().into());
        value.insert("chevron".into(), row.chevron.into());
        value.insert("separated".into(), row.separated.into());
        value.insert("kind".into(), kind.into());
        value.insert("plain".into(), (kind == "plain").into());
        value.insert("value_kind".into(), (kind == "value").into());
        value.insert("minus".into(), (index + 1).into());
        value.insert("plus".into(), (index + 2).into());
        value.insert("trail".into(), trail.into());
        value.insert("chips".into(), row.chips.clone().into());
        value.insert("swatches".into(), (kind == "swatches").into());
        value.insert("choices".into(), (kind == "choices").into());
        serde_json::Value::Object(value)
    };
    let mut back = None;
    let mut columns: Vec<(Option<String>, Vec<serde_json::Value>)> = Vec::new();
    for (index, row) in panel.rows.iter().enumerate().take(PLAYER_PANEL_ROW_LIMIT) {
        if row.back {
            back = Some(index);
        } else if row.heading || row.group {
            columns.push((Some(row.label.clone()), Vec::new()));
        } else if row.step.is_some_and(PanelStep::follows_lead) {
            continue;
        } else {
            if columns.is_empty() {
                columns.push((None, Vec::new()));
            }
            if let Some(column) = columns.last_mut() {
                column.1.push(row_data(index, row));
            }
        }
    }
    let tall = columns.iter().any(|(_, rows)| rows.len() > 9);
    let columns: Vec<_> = columns
        .into_iter()
        .map(|(heading, rows)| serde_json::json!({"heading": heading, "rows": rows}))
        .collect();
    serde_json::json!({
        "tall": tall,
        "title": panel.title,
        "message": panel.message,
        "back": back,
        "hasBack": back.is_some(),
        "columns": columns,
        "sync": sync_data(panel),
        "chipsSheet": panel.chips && mobile,
        "current": panel
            .rows
            .iter()
            .find(|row| row.selected)
            .map(|row| row.value.clone().unwrap_or_else(|| row.label.clone())),
        "chipRows": panel
            .rows
            .iter()
            .enumerate()
            .take(PLAYER_PANEL_ROW_LIMIT)
            .filter(|(_, row)| !row.heading)
            .map(|(index, row)| serde_json::json!({"index": index, "label": row.label, "selected": row.selected}))
            .collect::<Vec<_>>(),
        "look": panel.look.as_ref().map(|look| serde_json::json!({
            "sample": localized("settings.subtitle_preview_text", language),
            "text": color_hex(look.text),
            "background": color_hex(look.background),
            "bold": look.bold,
        })),
    })
}
