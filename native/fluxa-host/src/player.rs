use std::sync::{Arc, mpsc::Receiver};
use std::time::Duration;

use fluxa_core::player::stream_policy;
use fluxa_ui::{PlayerModel, SettingsModel};
use serde_json::{Value, json};
use web_time::Instant;

use crate::{
    NativeAction, RendererState, Route, SessionHandle, core_value, host_log, profile_language,
};

mod casting;
mod filmstrip;
mod input;
mod menu;
mod overlay;
mod resolve;
mod seek_previews;
mod shuffle;
mod stats;
mod submit;
mod subtitle_sync;
mod trailer;
mod trickplay;
mod upnext;
pub(crate) use input::*;
use overlay::Overlay;
use resolve::*;
pub(crate) use shuffle::*;
pub use stats::PlaybackStats;
use upnext::*;

const CONTROLS_TIMEOUT: Duration = Duration::from_secs(3);
const DEFAULT_SEEK_STEP: f64 = 10.0;
const SCRUB_COMMIT: Duration = Duration::from_millis(1800);
const TORRENT_UNRESPONSIVE_AFTER: Duration = Duration::from_secs(20);
const MIN_BRIGHTNESS: f32 = 0.15;
const MEDIA_DRIFT: f64 = 1.5;
const FAST_SPEED: f64 = 2.0;
const SEEK_RUN: Duration = Duration::from_millis(1400);
const TOAST_FADE: Duration = Duration::from_millis(250);
const THUMBNAIL_INTERVAL: Duration = Duration::from_millis(75);

#[derive(Clone, Debug, Default)]
pub struct VideoStatus {
    pub position: f64,
    pub duration: f64,
    pub paused: bool,
    pub muted: bool,
    pub volume: f64,
    pub has_frame: bool,
    pub error: Option<String>,
    pub chapters: Vec<(f64, String)>,
    pub buffering: Option<f32>,
    pub stalled: bool,
    pub buffered: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpeechEnergy {
    pub start: f64,
    pub frame_seconds: f64,
    pub energies: Vec<f64>,
}

pub struct Thumbnail {
    pub time: f64,
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThumbnailPlan {
    pub enabled: bool,
    pub url: Option<String>,
    pub headers: Vec<(String, String)>,
    pub interval: f64,
    pub sheet: bool,
    pub hold: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ThumbnailProgress {
    pub done: usize,
    pub count: usize,
}

#[derive(Clone, Debug)]
pub struct VideoTrack {
    pub id: String,
    pub subtitle: bool,
    pub language: Option<String>,
    pub title: Option<String>,
    pub selected: bool,
    pub external: bool,
    pub forced: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TrackSelection {
    pub audio: Option<String>,
    pub subtitle: Option<String>,
    pub secondary_subtitle: Option<String>,
    pub subtitles_off: bool,
}

#[derive(Clone, Debug)]
pub enum VideoCommand {
    TogglePause,
    Seek(f64),
    SeekTo(f64),
    ToggleMute,
    SetVolume(f64),
    SetSpeed(f64),
    Shaders(String),
    Motion(String),
    Fit(String),
    SelectTracks(TrackSelection),
    Mpv(Vec<String>),
}

pub type DeviceOpener = Arc<
    dyn Fn(
            &wgpu::Adapter,
            &wgpu::DeviceDescriptor<'_>,
        ) -> Result<(wgpu::Device, wgpu::Queue), String>
        + Send
        + Sync,
>;

pub trait VideoBackend: Send {
    fn device_opener(&self) -> Option<DeviceOpener> {
        None
    }
    fn load(&mut self, instance: &wgpu::Instance, device: &wgpu::Device, url: &str);
    fn configure(&mut self, _settings: &Value) {}
    fn load_preview(&mut self, instance: &wgpu::Instance, device: &wgpu::Device, url: &str) {
        self.load(instance, device, url);
    }
    fn place_preview(&mut self, _rect: Option<[f32; 4]>) {}
    fn stop(&mut self);
    fn media_session(&mut self, _plan: &Value) {}
    fn command(&mut self, command: VideoCommand);
    fn render(&mut self, device: &wgpu::Device) -> Option<wgpu::TextureView>;
    fn status(&mut self) -> VideoStatus;
    fn tracks(&mut self) -> Vec<VideoTrack> {
        Vec::new()
    }
    fn analyze_audio(
        &mut self,
        _url: &str,
        _start: f64,
        _length: f64,
    ) -> Option<Receiver<Option<SpeechEnergy>>> {
        None
    }
    fn thumbnails(&mut self, _plan: &ThumbnailPlan) {}
    fn thumbnail_progress(&mut self) -> Option<ThumbnailProgress> {
        None
    }
    fn release_start(&mut self) {}
    fn request_thumbnail(&mut self, _time: f64) {}
    fn take_thumbnail(&mut self) -> Option<Thumbnail> {
        None
    }
    fn passthrough(&self) -> bool {
        false
    }
    fn frame_driven(&self) -> bool {
        self.passthrough()
    }
    fn stats(&mut self) -> Option<PlaybackStats> {
        None
    }
    fn subtitle(&mut self) -> Option<String> {
        None
    }
}

pub(crate) struct PlayerSession {
    pub(crate) stale: Option<Value>,
    stale_until: Instant,
    snapshot_logo: Option<String>,
    logo_requested: bool,
    meta: Value,
    local: bool,
    started: bool,
    loaded_url: Option<String>,
    pub(crate) cast_target: Option<crate::cast::Device>,
    pub(crate) texture: Option<egui::TextureId>,
    last_activity: Instant,
    pub(crate) locked: bool,
    style: Value,
    style_hidden: bool,
    sub_pos: Option<String>,
    sync: subtitle_sync::SyncState,
    torrent_link: Option<String>,
    torrent_since: Option<Instant>,
    torrent_rx: Option<Receiver<Value>>,
    torrent_status: Option<Value>,
    error: Option<String>,
    pub(crate) status: VideoStatus,
    language: String,
    description: Option<String>,
    episode_title: Option<String>,
    episode_image: Option<String>,
    season_episode: Option<(i64, i64)>,
    synopsis: Option<String>,
    stream_addons: Vec<String>,
    warnings_rx: Option<Receiver<Option<Value>>>,
    warnings_requested: bool,
    warnings: Vec<(String, String)>,
    warnings_clock: Option<f32>,
    last_pump: Instant,
    thumbnail: Option<(f64, egui::TextureHandle)>,
    thumbnail_requested: Option<(f64, Instant)>,
    filmstrip: filmstrip::Filmstrip,
    trickplay: Option<trickplay::Trickplay>,
    seek_previews: Option<seek_previews::SeekPreviews>,
    outro_reached: bool,
    recommendations_rx: Option<Receiver<Vec<Value>>>,
    recommendations: Vec<Value>,
    recommendation_index: usize,
    load_progress: f32,
    scrub: Option<(f64, Instant)>,
    scrub_streak: u32,
    passthrough: bool,
    subtitle: Option<String>,
    dispatched: Option<Value>,
    overlay: Overlay,
    panel: Option<Panel>,
    pub(crate) cast: crate::cast::Cast,
    trailer: Option<trailer::TrailerPlay>,
    sources: Option<Vec<Value>>,
    opened_ms: u128,
    feed_version: u64,
    stream_parts: std::collections::BTreeMap<usize, Vec<Value>>,
    streams_requested: bool,
    streams_done: bool,
    chosen: Option<usize>,
    source_filter: Option<String>,
    pub(crate) sources_scroll_max: f32,
    manual: bool,
    toast: Option<(Value, Instant)>,
    seek_run: Option<(f64, Instant)>,
    media_sent: Option<(Instant, String, f64)>,
    brightness: f32,
    speed_held: bool,
    pub(crate) speed: f64,
    tracks: Vec<VideoTrack>,
    track_lang: Option<(bool, String)>,
    aspect: usize,
    audio_delay: f64,
    subtitle_delay: f64,
    sleep: usize,
    sleep_at: Option<Instant>,
    tracks_restored: bool,
    resume_at: Option<f64>,
    pub(crate) anime4k: bool,
    pub(crate) upscaling: String,
    pub(crate) fit: &'static str,
    pub(crate) compact: bool,
    volume_pin: Option<f64>,
    pub(crate) stats_visible: bool,
    stats: Vec<fluxa_ui::StatsSection>,
    stats_at: Option<Instant>,
    last_stats: Option<PlaybackStats>,
    playing_badges: Vec<fluxa_ui::SourceBadge>,
    pub(crate) minimized: bool,
}

enum Panel {
    Submit(submit::Submit),
    Cast,
    Menu(menu::Menu),
}

impl PlayerSession {
    pub(crate) fn trailer(meta: Value, urls: &[String]) -> Self {
        let mut player = Self::new(meta);
        player.trailer = Some(trailer::TrailerPlay::new(urls));
        player.started = true;
        player
    }

    fn model_logo_from_meta(&self) -> Option<&str> {
        ["logo", "logoUrl", "clearLogo", "titleLogo", "titleLogoUrl"]
            .iter()
            .find_map(|key| {
                self.meta
                    .get(*key)
                    .and_then(Value::as_str)
                    .filter(|url| !url.trim().is_empty())
            })
    }

    pub(crate) fn with_logo_hint(mut self, logo: Option<String>) -> Self {
        if logo.is_some() {
            self.snapshot_logo = logo;
        }
        self
    }

    pub(crate) fn new(meta: Value) -> Self {
        Self {
            stale: None,
            stale_until: Instant::now() + STALE_GRACE,
            snapshot_logo: None,
            logo_requested: false,
            meta,
            local: false,
            started: false,
            loaded_url: None,
            cast_target: None,
            texture: None,
            last_activity: Instant::now(),
            locked: false,
            style: Value::Null,
            style_hidden: false,
            sub_pos: None,
            sync: subtitle_sync::SyncState::default(),
            torrent_link: None,
            torrent_since: None,
            torrent_rx: None,
            torrent_status: None,
            error: None,
            status: VideoStatus::default(),
            language: "en".to_owned(),
            description: None,
            episode_title: None,
            episode_image: None,
            season_episode: None,
            synopsis: None,
            stream_addons: Vec::new(),
            warnings_rx: None,
            warnings_requested: false,
            warnings: Vec::new(),
            warnings_clock: None,
            last_pump: Instant::now(),
            thumbnail: None,
            thumbnail_requested: None,
            filmstrip: filmstrip::Filmstrip::default(),
            trickplay: None,
            seek_previews: None,
            outro_reached: false,
            recommendations_rx: None,
            recommendations: Vec::new(),
            passthrough: false,
            subtitle: None,
            dispatched: None,
            recommendation_index: 0,
            load_progress: 0.0,
            sources: None,
            opened_ms: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_millis()),
            feed_version: 0,
            stream_parts: std::collections::BTreeMap::new(),
            streams_requested: false,
            streams_done: false,
            chosen: None,
            source_filter: None,
            sources_scroll_max: 0.0,
            manual: false,
            scrub: None,
            scrub_streak: 0,
            overlay: Overlay::default(),
            panel: None,
            cast: crate::cast::Cast::default(),
            trailer: None,
            toast: None,
            seek_run: None,
            media_sent: None,
            brightness: 1.0,
            speed_held: false,
            speed: 1.0,
            tracks: Vec::new(),
            track_lang: None,
            aspect: 0,
            audio_delay: 0.0,
            subtitle_delay: 0.0,
            sleep: 0,
            sleep_at: None,
            tracks_restored: false,
            resume_at: None,
            anime4k: false,
            upscaling: String::new(),
            fit: "fit",
            compact: false,
            volume_pin: None,
            stats_visible: false,
            stats: Vec::new(),
            stats_at: None,
            last_stats: None,
            playing_badges: Vec::new(),
            minimized: false,
        }
    }

    fn title(&self) -> String {
        self.meta
            .get("name")
            .or_else(|| self.meta.get("title"))
            .and_then(Value::as_str)
            .unwrap_or("Fluxa")
            .to_owned()
    }

    pub(crate) fn start_offset_ms(&self) -> i64 {
        self.meta
            .get("timeOffset")
            .and_then(Value::as_f64)
            .filter(|offset| *offset > 5.0)
            .map_or(0, |offset| (offset * 1000.0) as i64)
    }

    pub(crate) fn watch_content(&self) -> Option<Value> {
        if self.local || self.trailer.is_some() {
            return None;
        }
        let id = self.meta.get("id").and_then(Value::as_str)?;
        let video = ["lastVideoId", "videoId", "video_id"]
            .iter()
            .find_map(|key| self.meta.get(*key).and_then(Value::as_str));
        let season_episode = self.season_episode.or_else(|| {
            let mut parts = video?.rsplit(':');
            let episode = parts.next()?.parse().ok()?;
            Some((parts.next()?.parse().ok()?, episode))
        });
        let artwork = json!({
            "backdrop": self.meta.get("background").and_then(Value::as_str),
            "logo": self.model_logo_from_meta().or(self.snapshot_logo.as_deref()),
            "episodeTitle": self.episode_title,
            "season": season_episode.map(|(season, _)| season),
            "episode": season_episode.map(|(_, episode)| episode),
        });
        Some(json!({
            "id": id.split(':').next().unwrap_or(id),
            "contentType": self.meta.get("type").and_then(Value::as_str).unwrap_or("movie"),
            "videoId": video,
            "title": self.title(),
            "artwork": artwork,
        }))
    }

    pub(crate) fn controls_visible(&self) -> bool {
        !self.status.has_frame || self.last_activity.elapsed() < CONTROLS_TIMEOUT
    }

    fn hide(&mut self) {
        self.scrub = None;
        if let Some(past) = Instant::now().checked_sub(CONTROLS_TIMEOUT) {
            self.last_activity = past;
        }
    }

    pub(crate) fn seek_step(&self) -> f64 {
        self.overlay.seek_seconds().unwrap_or(DEFAULT_SEEK_STEP)
    }

    fn scrub(&mut self, direction: f64) {
        let (base, streak) = match self.scrub {
            Some((time, at)) if at.elapsed() < SCRUB_COMMIT => (time, self.scrub_streak + 1),
            _ => (self.status.position, 0),
        };
        self.scrub_streak = streak;
        let step = self.seek_step() * (1 + streak / 4).min(6) as f64;
        let target = (base + direction * step).clamp(0.0, self.status.duration.max(0.0));
        self.scrub = Some((target, Instant::now()));
        self.touch();
    }

    pub(crate) fn toast(&mut self, kind: &str, value: f64) {
        if let Some(plan) = core_value("playerToastPlan", json!({"kind": kind, "value": value})) {
            self.toast = Some((plan, Instant::now()));
        }
    }

    pub(crate) fn toast_seek(&mut self, delta: f64) {
        let total = match self.seek_run {
            Some((total, at)) if at.elapsed() < SEEK_RUN && total * delta > 0.0 => total + delta,
            _ => delta,
        };
        self.seek_run = Some((total, Instant::now()));
        self.toast("seek", total);
    }

    pub(crate) fn toast_skip(&mut self) {
        let Some(skip) = self.overlay.skip() else {
            return;
        };
        let input = json!({
            "kind": "skip",
            "segment": skip["kind"],
            "label": skip["provider"].as_str().unwrap_or("Chapters"),
        });
        if let Some(plan) = core_value("playerToastPlan", input) {
            self.toast = Some((plan, Instant::now()));
        }
    }

    fn toast_model(&self) -> Option<fluxa_ui::PlayerToast> {
        let (plan, at) = self.toast.as_ref()?;
        let hold = Duration::from_millis(plan["holdMs"].as_u64().unwrap_or(1200));
        let elapsed = at.elapsed();
        if elapsed >= hold + TOAST_FADE {
            return None;
        }
        let opacity = match elapsed.checked_sub(hold) {
            Some(fade) => 1.0 - fade.as_secs_f32() / TOAST_FADE.as_secs_f32(),
            None => 1.0,
        };
        let text = fluxa_ui::localized(plan["key"].as_str()?, &self.language)
            .replace("%s", plan["value"].as_str().unwrap_or_default());
        Some(fluxa_ui::PlayerToast {
            text,
            level: plan["level"].as_f64().map(|level| level as f32),
            opacity,
            gauge: match plan["key"].as_str()? {
                "player.toast.brightness" => Some(fluxa_ui::ToastGauge::Brightness),
                "player.toast.volume" => Some(fluxa_ui::ToastGauge::Volume),
                _ => None,
            },
        })
    }

    pub(crate) fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    fn episode_detail(&self) -> Option<String> {
        let numbers = self
            .meta
            .get("lastVideoId")
            .and_then(Value::as_str)
            .and_then(|video| {
                let mut parts = video.rsplit(':');
                let episode = parts.next()?.parse::<i64>().ok()?;
                let season = parts.next()?.parse::<i64>().ok()?;
                Some((season, episode))
            });
        match (numbers, self.episode_title.as_deref()) {
            (Some((season, episode)), Some(title)) => Some(format!("S{season}:E{episode} {title}")),
            (Some((season, episode)), None) => Some(format!("S{season}:E{episode}")),
            (None, title) => title.map(ToOwned::to_owned),
        }
    }

    pub(crate) fn presence(&self) -> crate::presence::Presence {
        if !self.wants_landscape() {
            return crate::presence::Presence::Viewing {
                title: self.title(),
                poster: self
                    .meta
                    .get("poster")
                    .and_then(Value::as_str)
                    .filter(|url| url.starts_with("http"))
                    .map(ToOwned::to_owned),
            };
        }
        crate::presence::Presence::Playing {
            title: self.title(),
            detail: self.episode_detail(),
            paused: self.status.paused,
            position: self.status.position,
            duration: self.status.duration,
            poster: self
                .meta
                .get("poster")
                .and_then(Value::as_str)
                .filter(|url| url.starts_with("http"))
                .map(ToOwned::to_owned),
            party: None,
        }
    }

    pub(crate) fn pick_manually(&mut self, snapshot: Option<&Value>) {
        let profile = snapshot.and_then(|snapshot| snapshot.pointer("/profile/active"));
        self.manual = self.meta.get("pickSource").is_some()
            || (profile
                .and_then(|profile| profile.get("streamSourceSelectionMode"))
                .and_then(Value::as_str)
                .unwrap_or("manual")
                == "manual"
                && !stream_policy::resumes_saved_stream(&self.meta, profile));
        self.resume_at = self
            .meta
            .get("timeOffset")
            .and_then(Value::as_f64)
            .filter(|offset| *offset > 5.0);
    }

    fn streams_listing(&self) -> bool {
        self.streams_requested && !self.streams_done
    }

    pub(crate) fn listed_stream(&self, index: usize) -> Option<Value> {
        self.listed_streams().into_iter().nth(index)
    }

    fn listed_streams(&self) -> Vec<Value> {
        self.stream_parts.values().flatten().cloned().collect()
    }

    fn poll_streams(&mut self, session: &fluxa_effects::SessionHandle) {
        if !self.manual || self.local || self.chosen.is_some() || self.streams_done {
            return;
        }
        self.streams_requested = true;
        let Some(feed) = session.stream_feed_since(self.opened_ms, self.feed_version) else {
            return;
        };
        self.feed_version = feed.version;
        self.stream_parts = feed.parts;
        self.streams_done = feed.done;
        let listed = self.listed_streams();
        if !listed.is_empty() || self.streams_done {
            self.sources = Some(listed);
        }
    }

    pub(crate) fn wants_landscape(&self) -> bool {
        !(self.manual && self.chosen.is_none())
    }

    fn saved_stream_failed(&self) -> bool {
        let saved = ["lastStreamUrl", "lastStream"].iter().any(|key| {
            self.meta
                .get(*key)
                .is_some_and(|value| !value.is_null() && value.as_str() != Some(""))
        });
        saved
            && self.chosen.is_none()
            && !self.local
            && !self.status.has_frame
            && (self.error.is_some() || self.status.error.is_some())
    }

    fn load_stage(&self) -> String {
        let text = |key: &str| fluxa_ui::localized(key, &self.language);
        if self.loaded_url.is_none() {
            let previews = self.seek_previews.as_ref().is_some_and(|p| p.preparing());
            return text(if previews {
                "player.status_seek_previews"
            } else {
                "player.status_preparing"
            });
        }
        if let Some(progress) = self.seek_previews.as_ref().and_then(|p| p.holding()) {
            let label = text("player.status_seek_previews");
            return match progress {
                Some(progress) => {
                    format!("{label} {}%", progress.done * 100 / progress.count.max(1))
                }
                None => label,
            };
        }
        match self.status.buffering {
            Some(fraction) => text("player.status_buffering_percent")
                .replace("%s", &((fraction * 100.0).round() as u32).to_string()),
            None => text("player.status_connecting_source"),
        }
    }

    fn torrent_unresponsive(&self) -> bool {
        let Some(status) = self.torrent_status.as_ref() else {
            return false;
        };
        let unresolved = status
            .get("hash")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty);
        let peers = status
            .get("total_peers")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        unresolved
            && peers > 0
            && self
                .torrent_since
                .is_some_and(|since| since.elapsed() >= TORRENT_UNRESPONSIVE_AFTER)
    }

    fn torrent_lines(&self) -> (String, String) {
        if self.torrent_unresponsive() {
            let text = |key: &str| fluxa_ui::localized(key, &self.language);
            return (
                text("player.torrent_unresponsive"),
                text("player.torrent_unresponsive_hint"),
            );
        }
        fluxa_ui::torrent_status_lines(self.torrent_status.as_ref())
    }

    pub(crate) fn model(&self) -> PlayerModel {
        let holding = self
            .seek_previews
            .as_ref()
            .is_some_and(|p| p.holding().is_some());
        let status = if self.texture.is_some() && !holding {
            None
        } else if self.torrent_link.is_some() {
            Some(self.torrent_lines())
        } else {
            Some((self.load_stage(), String::new()))
        };
        PlayerModel {
            title: self.title(),
            video: self.texture.filter(|_| self.status.has_frame),
            passthrough: self.passthrough && self.status.has_frame,
            subtitle: self.subtitle.clone(),
            watch_button: false,
            locked: self.locked,
            watch: None,
            status,
            error: self.error.clone().or_else(|| self.status.error.clone()),
            position: self.status.position,
            duration: self.status.duration,
            paused: self.status.paused,
            muted: self.status.muted,
            volume: self.status.volume,
            boost: false,
            stats: self.stats.clone(),
            controls_visible: self.controls_visible(),
            show_pause_info: self.status.paused
                && self.status.has_frame
                && !self.controls_visible(),
            logo: ["logo", "logoUrl", "clearLogo", "titleLogo", "titleLogoUrl"]
                .iter()
                .find_map(|key| {
                    self.meta
                        .get(*key)
                        .and_then(Value::as_str)
                        .filter(|url| !url.trim().is_empty())
                })
                .map(ToOwned::to_owned)
                .or_else(|| self.snapshot_logo.clone()),
            episode_title: self.episode_title.clone(),
            episode_image: self.episode_image.clone(),
            season_episode: self.season_episode,
            synopsis: self.synopsis.clone(),
            description: self.description.clone(),
            chapters: self.status.chapters.clone(),
            buffered: self.status.buffered,
            chapter_spans: self.overlay.chapter_spans(),
            segment_spans: self.overlay.segment_spans(),
            mark_preview: match &self.panel {
                Some(Panel::Submit(submit)) => {
                    submit.preview(self.status.position, self.status.duration)
                }
                _ => None,
            },
            skip: self.overlay.skip_card(self.status.position),
            next_episode: self.overlay.next_card(),
            thumbnail: self
                .thumbnail
                .as_ref()
                .map(|(time, texture)| (*time, texture.id())),
            filmstrip: self.filmstrip.textures(),
            filmstrip_step: self.filmstrip_step(),
            warnings: self.warnings.clone(),
            warnings_elapsed: self.warnings_clock,
            language: self.language.clone(),
            upscaling: String::new(),
            recommendations: self
                .recommendations
                .iter()
                .map(|item| fluxa_ui::hero_from_meta(item, &self.language))
                .collect(),
            recommendation_index: self.recommendation_index,
            background: self
                .meta
                .get("background")
                .and_then(Value::as_str)
                .filter(|url| !url.trim().is_empty())
                .map(ToOwned::to_owned),
            load_progress: (self.load_progress > 0.0).then_some(self.load_progress),
            scrub: self.scrub.map(|(time, _)| time),
            playing_badges: self.playing_badges.clone(),
            sources: match &self.sources {
                Some(streams) => Some(streams.iter().map(source_row).collect()),
                None if self.manual && self.chosen.is_none() && self.error.is_none() => {
                    Some(Vec::new())
                }
                None => None,
            },
            sources_loading: self.sources.is_none() || self.streams_listing(),
            sources_pending: if self.sources.is_none() || self.streams_listing() {
                let listed = self.listed_streams();
                self.stream_addons
                    .iter()
                    .filter(|addon| {
                        !listed.iter().any(|stream| {
                            stream.get("addonName").and_then(Value::as_str) == Some(addon.as_str())
                        })
                    })
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            },
            source_filter: self.source_filter.clone(),
            toast: self.toast_model(),
            panel: self.panel.as_ref().map(|panel| match panel {
                Panel::Submit(submit) => submit.model(&self.language),
                Panel::Cast => self.cast.model(&self.language),
                Panel::Menu(menu) => menu::model(self, *menu),
            }),
            has_next: self.overlay.upcoming().is_some(),
            has_previous: menu::has_previous(self),
            trailer: self.trailer.is_some(),
            minimized: self.minimized,
            minimizable: false,
            pip: false,
            has_episodes: menu::has_episodes(self),
            dim: 1.0 - self.brightness,
            options: fluxa_ui::PlayerOptions::default(),
        }
    }
}

fn source_row(stream: &Value) -> fluxa_ui::PlayerSource {
    let lines = |key: &str| {
        stream
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let text = |key: &str| {
        stream
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut detail = text("description");
    if detail.is_empty() {
        detail = text("title");
    }
    let video_size = stream
        .pointer("/behaviorHints/videoSize")
        .and_then(Value::as_i64);
    let filename = stream
        .pointer("/behaviorHints/filename")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let raw = format!(
        "{}\n{}\n{}\n{filename}",
        lines("name"),
        text("description"),
        text("title")
    );
    let (badges, badges_top) = crate::stream_badges::badges_for(&raw, video_size);
    let mut name = text("name");
    if let Some((laid_name, laid_detail)) = crate::stream_badges::lay_out(&raw, video_size) {
        if !laid_name.is_empty() {
            name = laid_name;
        }
        if !laid_detail.is_empty() {
            detail = laid_detail;
        }
    }
    fluxa_ui::PlayerSource {
        addon: lines("addonName"),
        name: crate::stream_badges::format_text(&name),
        detail: crate::stream_badges::format_text(&detail),
        badges,
        badges_top,
        actions: !crate::stream_menu::entries(stream).is_empty(),
    }
}

pub(crate) fn cast_to(
    state: &mut RendererState,
    device: crate::cast::Device,
    url: &str,
    title: &str,
) {
    casting::cast_to(state, device, url, title);
}

pub(crate) fn download_stream(state: &mut RendererState, index: usize) {
    input::download_source(state, index);
}

const STALE_GRACE: Duration = Duration::from_secs(3);

pub(crate) fn direct_playback_command(item: &Value, profile: &Value) -> Value {
    if stream_policy::resumes_saved_stream(item, Some(profile)) {
        return json!({
            "type": "continueWatchingPlaybackRequested",
            "item": item,
            "language": profile_language(profile),
            "profile": profile,
        });
    }
    json!({
        "type": "directPlaybackRequested",
        "meta": item,
        "language": profile_language(profile),
        "profile": profile,
    })
}

pub(crate) fn preview_scene(state: &mut RendererState, scene: &str) {
    let track =
        |id: &str, subtitle: bool, language: &str, title: &str, selected: bool| VideoTrack {
            id: id.to_owned(),
            subtitle,
            language: Some(language.to_owned()),
            title: Some(title.to_owned()),
            selected,
            external: false,
            forced: false,
        };
    let style = state.settings.values.clone();
    let Some(player) = state.player.as_mut() else {
        return;
    };
    player.tracks = vec![
        track("a1", false, "jpn", "Japonca 5.1", true),
        track("a2", false, "eng", "İngilizce 5.1", false),
        track("a3", false, "tur", "Türkçe 2.0", false),
        track("s1", true, "tur", "Türkçe", true),
        track("s2", true, "eng", "İngilizce", false),
        track("s3", true, "jpn", "Japonca", false),
    ];
    player.style = style;
    player.status.chapters = vec![
        (0.0, "Açılış".to_owned()),
        (95.0, "Fener Bekçisi".to_owned()),
        (610.0, "Fırtına".to_owned()),
        (1380.0, "Kayıp Defter".to_owned()),
        (2400.0, "Jenerik".to_owned()),
    ];
    player.status.duration = 2712.0;
    player.status.position = 754.0;
    player.panel = match scene {
        "chapters" => Some(Panel::Menu(menu::Menu::Chapters)),
        "tracks" => Some(Panel::Menu(menu::Menu::Tracks)),
        "speed" => Some(Panel::Menu(menu::Menu::Speed)),
        "episodes" => Some(Panel::Menu(menu::Menu::Episodes)),
        "style" => Some(Panel::Menu(menu::Menu::Style)),
        _ => None,
    };
}

pub(crate) fn open_file(state: &mut RendererState, uri: &str, title: &str) -> Value {
    let mut player = PlayerSession::new(json!({"name": title, "type": "movie"}));
    player.local = true;
    player.started = true;
    state.player = Some(player);
    state.shuffle = None;
    state.screen_scroll_offsets.remove(&Route::Player);
    crate::reset_ui(state);
    json!({"type": "playerResolvePlaybackRequested", "url": uri, "title": title})
}

fn seek_logo(player: &mut PlayerSession, session: &SessionHandle, snapshot: &Value) {
    if player.snapshot_logo.is_some() {
        return;
    }
    let id = player
        .meta
        .get("id")
        .and_then(Value::as_str)
        .map(|id| id.split(':').next().unwrap_or_default().to_owned())
        .filter(|id| !id.is_empty());
    let Some(id) = id else {
        return;
    };
    if !player.logo_requested {
        if player.model_logo_from_meta().is_some() {
            return;
        }
        player.logo_requested = true;
        let content_type = player
            .meta
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let _ = session.dispatch(json!({
            "type": "metaDetailRequested",
            "contentType": content_type,
            "id": id,
            "language": player.language,
            "profile": snapshot.pointer("/profile/active"),
        }));
        return;
    }
    let found = snapshot.pointer("/lookup/metaDetail").and_then(|detail| {
        let meta = detail.get("meta").unwrap_or(detail);
        let same = meta
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|found| found.split(':').next() == Some(id.as_str()));
        same.then(|| meta.get("logo").and_then(Value::as_str))
            .flatten()
            .filter(|url| !url.is_empty())
            .map(str::to_owned)
    });
    if found.is_some() {
        player.snapshot_logo = found;
    }
}

fn detail_logo(snapshot: &Value, meta: &Value) -> Option<String> {
    let detail = snapshot.get("detail")?;
    let id = |value: &Value| {
        ["id", "itemId"]
            .iter()
            .find_map(|key| value.get(*key).and_then(Value::as_str))
            .map(|id| id.split(':').next().unwrap_or_default().to_owned())
    };
    let played = id(meta)?;
    let same = id(detail).is_some_and(|id| id == played)
        || detail
            .get("meta")
            .and_then(id)
            .is_some_and(|id| id == played);
    if !same {
        return None;
    }
    detail
        .pointer("/fanartArtwork/hdLogo")
        .or_else(|| detail.pointer("/meta/logo"))
        .and_then(Value::as_str)
        .filter(|url| !url.is_empty())
        .map(str::to_owned)
}

fn fall_back_to_sources(state: &mut RendererState) {
    if !state
        .player
        .as_ref()
        .is_some_and(PlayerSession::saved_stream_failed)
    {
        return;
    }
    let Some(mut item) = state.player.as_ref().map(|player| player.meta.clone()) else {
        return;
    };
    if let Some(fields) = item.as_object_mut() {
        for key in [
            "lastStream",
            "lastStreamUrl",
            "lastStreamTitle",
            "lastStreamIndex",
        ] {
            fields.remove(key);
        }
        fields.insert("pickSource".to_owned(), true.into());
    }
    close(state);
    state
        .pending_native_actions
        .push(NativeAction::StartPlayback { item });
}

pub(crate) fn pump(state: &mut RendererState) {
    fall_back_to_sources(state);
    if let Some((time, _)) = state
        .player
        .as_mut()
        .and_then(|player| player.scrub.take_if(|(_, at)| at.elapsed() >= SCRUB_COMMIT))
    {
        command(state, VideoCommand::SeekTo(time));
    }
    if let Some(Panel::Submit(submit)) = state
        .player
        .as_mut()
        .and_then(|player| player.panel.as_mut())
    {
        submit.poll();
    }
    if let Some(player) = state.player.as_mut() {
        player.cast.poll();
    }
    advance_shuffle(state);
    menu::tick_sleep(state);
    menu::tick_style(state);
    menu::tick_subtitle_lift(state);
    subtitle_sync::tick(state);
    let RendererState {
        shuffle,
        player,
        session,
        video,
        gpu,
        settings,
        core_snapshot,
        pending_native_actions,
        ..
    } = state;
    let Some(player) = player.as_mut() else {
        return;
    };
    let session = session.as_ref();
    let snapshot = match (session, core_snapshot.as_ref()) {
        (Some(session), _) => session.snapshot(),
        (None, Some(snapshot)) => snapshot.clone(),
        (None, None) => return,
    };
    player.passthrough = video.as_ref().is_some_and(|video| video.passthrough());
    if player.snapshot_logo.is_none() {
        player.snapshot_logo = detail_logo(&snapshot, &player.meta);
    }
    if player.stale.is_some() {
        if player.stale.as_ref() == snapshot.get("player") && Instant::now() < player.stale_until {
            if let Some(session) = session {
                refresh_display(player, session, settings, &snapshot);
            }
            return;
        }
        player.stale = None;
    }
    if !player.started && !session.is_some_and(|session| session.has_queued_dispatches()) {
        if let Some(command) = resolution_command(&snapshot, player) {
            let starts = command_starts_playback(&snapshot);
            match session {
                Some(session) => match session.dispatch(command) {
                    Ok(()) => player.started = starts,
                    Err(error) => player.error = Some(error),
                },
                None if player.dispatched.as_ref() != Some(&command) => {
                    player.dispatched = Some(command.clone());
                    pending_native_actions.push(NativeAction::CoreCommand { command });
                    player.started = starts;
                }
                None => {}
            }
        }
    }
    let Some(session) = session else {
        load_resolved(player, video, gpu, settings, None, &snapshot);
        if let Some(video) = video.as_mut() {
            player.status = video.status();
        }
        player.episode_title = episode_title(&player.meta, &snapshot);
        return;
    };
    if player.trailer.is_some() {
        trailer::tick(player, session, video, gpu, &snapshot);
        if let Some(video) = video.as_mut() {
            player.status = video.status();
        }
        player.language =
            profile_language(snapshot.pointer("/profile/active").unwrap_or(&Value::Null));
        return;
    }
    poll_torrent(player, session, &snapshot);
    load_resolved(player, video, gpu, settings, Some(session), &snapshot);
    let mut advance = false;
    if let Some(video) = video.as_mut() {
        player.status = video.status();
        player.pin_volume();
        player.subtitle = video.subtitle();
        if player.status.has_frame && !player.tracks_restored {
            player.tracks_restored = true;
            menu::restore_tracks(player, video.as_mut(), settings);
            if let Some(at) = player.resume_at.take() {
                video.command(VideoCommand::SeekTo(at));
            }
        }
        if let Some(previews) = player.seek_previews.as_mut() {
            previews.tick(video.as_mut());
            if let Some(source) = previews.server_source() {
                player.trickplay = Some(trickplay::from_source(source));
            }
        }
        advance = overlay::tick(player, session, video.as_mut(), &snapshot);
        refresh_stats(player, video.as_mut(), &snapshot);
        let thumbnail = player
            .trickplay
            .as_mut()
            .and_then(|trickplay| trickplay.poll(session))
            .or_else(|| video.take_thumbnail());
        let thumbnail = match (thumbnail, gpu.as_ref()) {
            (Some(thumbnail), Some(gpu)) => player.filmstrip.accept(thumbnail, &gpu.egui_context),
            (thumbnail, _) => thumbnail,
        };
        if let Some(time) = player.filmstrip.next() {
            match player
                .trickplay
                .as_mut()
                .filter(|trickplay| trickplay.usable())
            {
                Some(trickplay) => trickplay.request(time),
                None => video.request_thumbnail(time),
            }
        }
        if let (Some(thumbnail), Some(gpu)) = (thumbnail, gpu.as_ref()) {
            let image = egui::ColorImage::from_rgba_unmultiplied(thumbnail.size, &thumbnail.rgba);
            match player.thumbnail.as_mut() {
                Some((time, texture)) => {
                    texture.set(image, egui::TextureOptions::LINEAR);
                    *time = thumbnail.time;
                }
                None => {
                    player.thumbnail = Some((
                        thumbnail.time,
                        gpu.egui_context.load_texture(
                            "fluxa-seek-thumbnail",
                            image,
                            egui::TextureOptions::LINEAR,
                        ),
                    ))
                }
            }
        }
    }
    if !player.status.has_frame {
        let preload = player
            .torrent_link
            .as_ref()
            .and(player.torrent_status.as_ref())
            .and_then(|status| status.get("preload"))
            .and_then(Value::as_f64)
            .filter(|percent| *percent > 0.0)
            .map(|percent| (percent / 100.0) as f32);
        if let Some(progress) = preload.or(player.status.buffering) {
            player.load_progress = player.load_progress.max(progress.clamp(0.045, 1.0));
        }
    }
    refresh_display(player, session, settings, &snapshot);
    tick_warnings(player);
    if shuffle.is_none() && !player.local {
        tick_recommendations(player, session, settings, &snapshot);
    }
    publish_media(player, video);
    if advance {
        play_next(state);
    }
}

fn refresh_display(
    player: &mut PlayerSession,
    session: &SessionHandle,
    settings: &SettingsModel,
    snapshot: &Value,
) {
    player.episode_title = episode_title(&player.meta, snapshot);
    player.episode_image = episode_image(&player.meta, snapshot);
    player.season_episode = season_episode(&player.meta, snapshot);
    player.synopsis = synopsis(&player.meta, snapshot);
    player.stream_addons = stream_addon_names(snapshot);
    seek_logo(player, session, snapshot);
    if !player.warnings_requested {
        player.warnings_requested = true;
        player.language =
            profile_language(snapshot.pointer("/profile/active").unwrap_or(&Value::Null));
        player.description = player
            .meta
            .get("description")
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .map(|text| {
                if !settings.bool_value("shortenSynopsis") {
                    return text.to_owned();
                }
                core_value("shortenSynopsis", json!({"text": text}))
                    .and_then(|value| value.as_str().map(ToOwned::to_owned))
                    .unwrap_or_else(|| text.to_owned())
            });
        player.warnings_rx =
            content_warning_url(&player.meta, snapshot).map(|url| session.fetch_json(url));
    }
    player.poll_streams(session);
}

fn publish_media(player: &mut PlayerSession, video: &mut Option<Box<dyn VideoBackend>>) {
    let Some(video) = video.as_mut().filter(|_| player.status.has_frame) else {
        return;
    };
    let speed = if player.speed_held { FAST_SPEED } else { 1.0 };
    let has_next = player.overlay.upcoming().is_some();
    let key = format!(
        "{}|{:?}|{}|{}|{}|{}",
        player.title(),
        player.episode_title,
        player.status.paused,
        player.status.duration as i64,
        has_next,
        speed
    );
    let position = player.status.position;
    let unchanged = player.media_sent.as_ref().is_some_and(|(at, sent, from)| {
        let advance = if player.status.paused {
            0.0
        } else {
            at.elapsed().as_secs_f64() * speed
        };
        *sent == key && (from + advance - position).abs() < MEDIA_DRIFT
    });
    if unchanged {
        return;
    }
    let poster = player
        .meta
        .get("poster")
        .and_then(Value::as_str)
        .filter(|url| url.starts_with("http"));
    let input = json!({
        "title": player.title(),
        "subtitle": player.episode_title,
        "poster": poster,
        "paused": player.status.paused,
        "buffering": player.status.buffering.is_some(),
        "position": position,
        "duration": player.status.duration,
        "speed": speed,
        "hasNext": has_next,
    });
    if let Some(plan) = core_value("playerMediaSessionPlan", input) {
        video.media_session(&plan);
    }
    player.media_sent = Some((Instant::now(), key, position));
}

pub(crate) fn media_command(state: &mut RendererState, name: &str, value: f64) {
    if name == "fitStep" {
        step_fit(state, value as i32, false);
        return;
    }
    let Some(player) = state.player.as_ref() else {
        return;
    };
    let Some(plan) = core_value(
        "playerMediaCommandPlan",
        json!({
            "command": name,
            "value": value,
            "paused": player.status.paused,
            "position": player.status.position,
            "duration": player.status.duration,
            "hasNext": player.overlay.upcoming().is_some(),
        }),
    ) else {
        return;
    };
    let amount = plan["value"].as_f64().unwrap_or_default();
    match plan["action"].as_str() {
        Some("togglePause") => command(state, VideoCommand::TogglePause),
        Some("seekBy") => command(state, VideoCommand::Seek(amount)),
        Some("seekTo") => command(state, VideoCommand::SeekTo(amount)),
        Some("next") => start_next(state, true),
        Some("close") => close(state),
        _ => {}
    }
}

fn play_next(state: &mut RendererState) {
    start_next(state, false);
}

fn current_binge_group(state: &RendererState) -> Option<String> {
    let snapshot = state.session.as_ref()?.snapshot();
    let index = snapshot.pointer("/player/currentStreamIndex")?.as_u64()? as usize;
    snapshot
        .pointer("/player/currentStreams")?
        .as_array()?
        .get(index)?
        .get("bingeGroup")?
        .as_str()
        .map(ToOwned::to_owned)
}

fn start_next(state: &mut RendererState, any: bool) {
    let Some(player) = state.player.as_ref() else {
        return;
    };
    let next = if any {
        player.overlay.upcoming()
    } else {
        player.overlay.next_video()
    };
    let Some(next) = next else {
        return;
    };
    let Some(id) = next.get("id").and_then(Value::as_str) else {
        return;
    };
    let mut item = player.meta.clone();
    let group = state
        .settings
        .bool_value("reuseBingeGroup")
        .then(|| current_binge_group(state))
        .flatten();
    if let Some(fields) = item.as_object_mut() {
        fields.retain(|key, _| {
            !key.starts_with("last") && key != "timeOffset" && key != "preferredBingeGroup"
        });
        fields.insert("lastVideoId".to_owned(), json!(id));
        if let Some(group) = group {
            fields.insert("preferredBingeGroup".to_owned(), json!(group));
        }
    }
    close(state);
    state
        .pending_native_actions
        .push(NativeAction::StartPlayback { item });
}

fn load_resolved(
    player: &mut PlayerSession,
    video: &mut Option<Box<dyn VideoBackend>>,
    gpu: &Option<crate::Gpu>,
    settings: &SettingsModel,
    session: Option<&SessionHandle>,
    snapshot: &Value,
) {
    if player.loaded_url.is_none()
        && let Some(url) = snapshot
            .pointer("/player/resolvedUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
    {
        if let Some(device) = player.cast_target.take() {
            let resume = player.resume_at.take().unwrap_or(0.0);
            player.cast.start(device, url, &player.title(), resume);
            player.loaded_url = Some(url.to_owned());
            player.panel = Some(Panel::Cast);
            return;
        }
        match (video.as_mut(), gpu.as_ref()) {
            (Some(video), Some(gpu)) => {
                let trickplay = trickplay::from_snapshot(snapshot);
                let previews = player.seek_previews.get_or_insert_with(|| {
                    seek_previews::SeekPreviews::begin(
                        settings,
                        snapshot,
                        &player.meta,
                        session,
                        trickplay.is_some(),
                    )
                });
                if !previews.ready_to_load() {
                    return;
                }
                host_log(format!("loading player url: {url}"));
                video.configure(&settings.values);
                video.thumbnails(&previews.initial_plan(url));
                #[cfg(not(target_arch = "wasm32"))]
                fluxa_effects::set_stream_headers(
                    url,
                    fluxa_core::player::stream_policy::selected_stream_headers(snapshot),
                );
                video.load(&gpu.instance, &gpu.device, url);
                video.command(VideoCommand::Shaders(shader_chain(settings)));
                video.command(VideoCommand::Motion(motion_profile(settings)));
                video.command(VideoCommand::Fit(video_fit(settings).to_owned()));
                apply_subtitle_style(video.as_mut(), &settings.values);
                player.loaded_url = Some(url.to_owned());
                previews.loaded();
                player.trickplay = trickplay;
            }
            (None, _) => player.error = Some("No video backend on this platform".to_owned()),
            _ => {}
        }
    }
}

impl PlayerSession {
    pub(crate) fn set_volume_pin(&mut self, volume: f64) {
        self.volume_pin = Some(volume);
    }

    fn pin_volume(&mut self) {
        if let Some(volume) = self.volume_pin {
            self.status.volume = volume;
        }
    }

    fn filmstrip_step(&self) -> f64 {
        self.trickplay
            .as_ref()
            .and_then(|trickplay| trickplay.step())
            .or_else(|| {
                self.seek_previews
                    .as_ref()
                    .map(|previews| previews.interval())
            })
            .unwrap_or(10.0)
    }
}

pub(crate) fn want_filmstrip(state: &mut RendererState, slots: Option<Vec<f64>>) {
    if !state.settings.bool_value("seekThumbnailEnabled") {
        return;
    }
    if let Some(player) = state.player.as_mut() {
        player.filmstrip.want(slots);
    }
}

pub(crate) fn hover_seek(state: &mut RendererState, time: Option<f64>) {
    if !state.settings.bool_value("seekThumbnailEnabled") {
        return;
    }
    let (Some(player), Some(video)) = (state.player.as_mut(), state.video.as_mut()) else {
        return;
    };
    let Some(time) = time else {
        return;
    };
    player.touch();
    if player.filmstrip.active() {
        return;
    }
    if let Some(trickplay) = player
        .trickplay
        .as_mut()
        .filter(|trickplay| trickplay.usable())
    {
        trickplay.request(time);
        return;
    }
    let time = (time * 2.0).round() / 2.0;
    if player
        .thumbnail_requested
        .is_some_and(|(last, at)| last == time || at.elapsed() < THUMBNAIL_INTERVAL)
    {
        return;
    }
    player.thumbnail_requested = Some((time, Instant::now()));
    video.request_thumbnail(time);
}

pub(crate) fn is_mini_node(node: u64) -> bool {
    matches!(
        node,
        fluxa_ui::NODE_PLAYER_TOGGLE | fluxa_ui::NODE_PLAYER_CLOSE | fluxa_ui::NODE_PLAYER_EXPAND
    )
}

impl PlayerSession {
    pub(crate) fn can_pip(&self) -> bool {
        self.status.has_frame && self.trailer.is_none() && self.error.is_none()
    }

    pub(crate) fn can_minimize(&self) -> bool {
        self.status.has_frame
            && self.trailer.is_none()
            && self.recommendations.is_empty()
            && self.panel.is_none()
    }
}

pub(crate) fn minimize(state: &mut RendererState) {
    let Some(player) = state.player.as_mut().filter(|player| player.can_minimize()) else {
        return;
    };
    player.minimized = true;
    player.hide();
    crate::reset_ui(state);
}

pub(crate) fn expand(state: &mut RendererState) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    player.minimized = false;
    player.touch();
    crate::reset_ui(state);
}

pub(crate) fn leave(state: &mut RendererState) {
    let desktop = state.home.form_factor == fluxa_ui::UiFormFactorJson::Desktop;
    if desktop
        && state
            .player
            .as_ref()
            .is_some_and(|player| player.can_minimize())
    {
        minimize(state);
    } else {
        close(state);
    }
}

pub(crate) fn close(state: &mut RendererState) {
    let Some(player) = state.player.take() else {
        return;
    };
    remember_summary(state, &player);
    if let Some(session) = state.session.as_ref() {
        let snapshot = session.snapshot();
        if player.overlay.scrobbling() {
            scrobble(session, &player, &snapshot, "stop");
        }
        let tracks = state
            .video
            .as_mut()
            .map(|video| video.tracks())
            .unwrap_or_default();
        if !player.local {
            overlay::finish(&player, session, &snapshot, &tracks);
        }
        if let Some(link) = player.torrent_link.clone() {
            session.deactivate_torrent(link);
        }
    }
    if let Some(video) = state.video.as_mut() {
        video.stop();
    }
    if let (Some(texture), Some(gpu)) = (player.texture, state.gpu.as_mut()) {
        gpu.egui_renderer.free_texture(&texture);
    }
    crate::reset_ui(state);
}

pub(crate) fn video_belongs_to(video: &str, meta: &Value) -> bool {
    let id = meta.get("id").and_then(Value::as_str).unwrap_or_default();
    id.is_empty()
        || video == id
        || video
            .strip_prefix(id)
            .is_some_and(|rest| rest.starts_with(':'))
}

fn save_progress(session: &SessionHandle, player: &PlayerSession, snapshot: &Value) {
    if player.local || player.status.duration <= 0.0 || player.status.position < 5.0 {
        return;
    }
    let stream = overlay::current_stream(player, snapshot);
    let command = json!({
        "type": "savePlaybackProgressRequested",
        "profile": session.active_profile(),
        "meta": player.meta,
        "timeOffset": player.status.position as i64,
        "duration": player.status.duration as i64,
        "lastVideoId": snapshot
            .pointer("/player/currentVideoId")
            .filter(|video| video_belongs_to(video.as_str().unwrap_or_default(), &player.meta)),
        "lastEpisodeName": player.episode_title,
        "lastEpisodeThumbnail": player.episode_image,
        "lastEpisodeOverview": player.synopsis,
        "lastStreamIndex": snapshot.pointer("/player/currentStreamIndex"),
        "lastStreamUrl": stream.and_then(|stream| stream.get("playableUrl").or_else(|| stream.get("url"))),
        "lastStreamTitle": stream.and_then(|stream| stream.get("title").or_else(|| stream.get("name"))),
    });
    if let Err(error) = session.dispatch(command) {
        host_log(format!("core dispatch failed: {error}"));
    }
}

fn scrobble(session: &SessionHandle, player: &PlayerSession, snapshot: &Value, action: &str) {
    let item_id = snapshot
        .pointer("/player/currentVideoId")
        .or_else(|| player.meta.get("id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if player.local || item_id.is_empty() || player.status.duration <= 0.0 {
        return;
    }
    let command = json!({
        "type": "scrobbleRequested",
        "token": "",
        "metaType": player.meta.get("type").and_then(Value::as_str).unwrap_or("movie"),
        "itemId": item_id,
        "progress": player.status.position / player.status.duration * 100.0,
        "actionName": action,
        "profile": session.active_profile(),
        "meta": player.meta,
    });
    if let Err(error) = session.dispatch(command) {
        host_log(format!("core dispatch failed: {error}"));
    }
}

pub(crate) fn seek(state: &mut RendererState, direction: f64) {
    let step = state
        .player
        .as_ref()
        .map_or(DEFAULT_SEEK_STEP, PlayerSession::seek_step);
    command(state, VideoCommand::Seek(direction * step));
}

pub(crate) fn command(state: &mut RendererState, command: VideoCommand) {
    if !crate::watch_together::intercept(state, &command) {
        return;
    }
    if let Some(player) = state.player.as_mut() {
        player.touch();
        match &command {
            VideoCommand::Seek(delta) => player.toast_seek(*delta),
            VideoCommand::ToggleMute => player.toast("muted", f64::from(!player.status.muted)),
            VideoCommand::SetSpeed(rate) => player.toast("speed", *rate),
            _ => {}
        }
    }
    if let Some(video) = state.video.as_mut() {
        video.command(command);
    }
}

pub(crate) fn apply_options(model: &mut PlayerModel, settings: &SettingsModel) {
    let on = |key: &str| settings.bool_value(key);
    let options = fluxa_ui::PlayerOptions {
        title: on("playerShowTitle"),
        up_next: on("playerShowUpNextCard"),
        audio_subtitles: on("playerShowAudioSubtitles"),
        speed: on("playerShowSpeed"),
        episodes: on("playerShowEpisodes"),
        next_episode: on("playerShowNextEpisode"),
        settings: on("playerShowSettings"),
        volume: on("playerShowVolume"),
        fullscreen: on("playerShowFullscreen"),
        cast: on("playerShowCast"),
        mark_segment: on("playerShowMarkSegment"),
        segments: on("playerShowSegments"),
        gestures: on("playerGestures"),
        double_tap: on("playerDoubleTapSeek"),
        center_controls: on("playerCenterControls"),
    };
    model.boost = on("playerAudioBoost");
    model.show_pause_info &= on("pauseMetadataOverlayEnabled");
    if !on("contentWarningsEnabled") {
        model.warnings.clear();
        model.warnings_elapsed = None;
    }
    if !options.up_next {
        model.next_episode = None;
    }
    model.options = options;
}

pub(crate) const UPSCALING_MODES: [&str; 6] = ["a", "b", "c", "aa", "bb", "ca"];

pub(crate) fn upscaling(settings: &SettingsModel) -> &str {
    if settings
        .str_value("animeUpscalingMode")
        .is_none_or(|mode| mode == "off")
    {
        return "off";
    }
    match settings.str_value("animeUpscalingModePreset") {
        Some(mode) if UPSCALING_MODES.contains(&mode) => mode,
        _ => "a",
    }
}

const FITS: [&str; 4] = ["fit", "fill", "stretch", "original"];

pub(crate) fn video_fit(settings: &SettingsModel) -> &'static str {
    FITS.into_iter()
        .find(|fit| settings.str_value("playerVideoFit") == Some(*fit))
        .unwrap_or(FITS[0])
}

pub(crate) fn step_fit(state: &mut RendererState, direction: i32, wrap: bool) {
    let current = FITS
        .iter()
        .position(|fit| *fit == video_fit(&state.settings))
        .unwrap_or(0) as i32;
    let count = FITS.len() as i32;
    let next = if wrap {
        (current + direction).rem_euclid(count)
    } else {
        (current + direction).clamp(0, count - 1)
    };
    if next == current {
        return;
    }
    let value = json!(FITS[next as usize]);
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert("playerVideoFit".to_owned(), value.clone());
    }
    state
        .pending_native_actions
        .push(crate::NativeAction::SettingsChange {
            key: "playerVideoFit".to_owned(),
            value,
        });
    command(state, VideoCommand::Fit(FITS[next as usize].to_owned()));
    if let Some(player) = state.player.as_mut() {
        let label = fluxa_ui::localized(
            &format!("player.fit_{}", FITS[next as usize]),
            &player.language,
        );
        if let Some(plan) = core_value("playerToastPlan", json!({"kind": "fit", "label": label})) {
            player.toast = Some((plan, Instant::now()));
        }
    }
}

fn apply_subtitle_style(video: &mut dyn VideoBackend, settings: &Value) {
    let Some(Value::Array(options)) = core_value("subtitleStylePlan", settings.clone()) else {
        return;
    };
    for option in options {
        if let (Some(name), Some(value)) = (option[0].as_str(), option[1].as_str()) {
            video.command(VideoCommand::Mpv(vec![
                "set".to_owned(),
                name.to_owned(),
                value.to_owned(),
            ]));
        }
    }
}

fn motion_profile(settings: &SettingsModel) -> String {
    match settings.str_value("frameInterpolationMode") {
        Some("display_resample") => "motion-resample",
        Some("smooth") => "smooth-motion",
        _ => "",
    }
    .to_owned()
}

fn shader_chain(settings: &SettingsModel) -> String {
    let mode = upscaling(settings);
    if mode == "off" {
        return String::new();
    }
    let tier = settings
        .str_value("animeUpscalingQuality")
        .unwrap_or("anime4k_m");
    core_value("anime4kProfile", json!({"tier": tier, "mode": mode}))
        .and_then(|profile| serde_json::from_value(profile).ok())
        .unwrap_or_default()
}

pub(crate) fn toggle_upscaling(state: &mut RendererState) {
    let next = if upscaling(&state.settings) == "off" {
        "a"
    } else {
        "off"
    };
    set_upscaling(state, next);
}

pub(crate) fn step_upscaling_mode(state: &mut RendererState, direction: i32) {
    if upscaling(&state.settings) == "off" {
        return;
    }
    let current = UPSCALING_MODES
        .iter()
        .position(|mode| *mode == upscaling(&state.settings))
        .unwrap_or(0) as i32;
    let next =
        UPSCALING_MODES[(current + direction).rem_euclid(UPSCALING_MODES.len() as i32) as usize];
    set_upscaling(state, next);
}

pub(crate) fn previous_episode(state: &mut RendererState) {
    menu::previous(state);
}

pub(crate) fn open_submit(state: &mut RendererState) {
    if let Some(player) = state.player.as_mut() {
        submit::open(player);
    }
}

pub(crate) fn set_upscaling(state: &mut RendererState, next: &str) {
    let mut changes = vec![(
        "animeUpscalingMode",
        json!(if next == "off" { "off" } else { "auto" }),
    )];
    if next != "off" {
        changes.push(("animeUpscalingModePreset", json!(next)));
    }
    for (key, value) in changes {
        if let Some(values) = state.settings.values.as_object_mut() {
            values.insert(key.to_owned(), value.clone());
        }
        state
            .pending_native_actions
            .push(crate::NativeAction::SettingsChange {
                key: key.to_owned(),
                value,
            });
    }
    let chain = shader_chain(&state.settings);
    command(state, VideoCommand::Shaders(chain));
    if let Some(player) = state.player.as_mut()
        && let Some(plan) = core_value(
            "playerToastPlan",
            json!({"kind": "anime4k", "value": if next == "off" { 0.0 } else { 1.0 }}),
        )
    {
        player.toast = Some((plan, Instant::now()));
    }
}

pub(crate) fn upload_frame(state: &mut RendererState) {
    let RendererState {
        player, video, gpu, ..
    } = state;
    let (Some(player), Some(video), Some(gpu)) = (player.as_mut(), video.as_mut(), gpu.as_mut())
    else {
        return;
    };
    if let Some(view) = video.render(&gpu.device) {
        if let Some(old) = player.texture.take() {
            gpu.egui_renderer.free_texture(&old);
        }
        player.texture = Some(gpu.egui_renderer.register_native_texture(
            &gpu.device,
            &view,
            wgpu::FilterMode::Linear,
        ));
    }
}

fn refresh_stats(player: &mut PlayerSession, video: &mut dyn VideoBackend, snapshot: &Value) {
    let interval = if player.stats_visible {
        stats::REFRESH
    } else {
        stats::SAMPLE
    };
    if !player.stats_visible {
        player.stats.clear();
    }
    if player.stats_at.is_some_and(|at| at.elapsed() < interval) {
        return;
    }
    player.stats_at = Some(Instant::now());
    let stats = video.stats();
    let playing = stats
        .as_ref()
        .map(PlaybackStats::playing)
        .unwrap_or_default();
    let stream = overlay::current_stream(player, snapshot).unwrap_or(&Value::Null);
    player.playing_badges = crate::stream_badges::playing_badges(stream, &playing);
    let Some(stats) = stats else {
        return;
    };
    if player.stats_visible {
        player.stats = stats.sections();
    }
    player.last_stats = Some(stats);
}

fn remember_summary(state: &mut RendererState, player: &PlayerSession) {
    let live = state.video.as_mut().and_then(|video| video.stats());
    let Some(stats) = live
        .filter(|stats| stats.video_codec.is_some())
        .or_else(|| player.last_stats.clone())
    else {
        return;
    };
    let watched = (player.status.position > 0.0).then(|| {
        let position = fluxa_ui::format_time(player.status.position);
        match player.status.duration {
            duration if duration > 0.0 => {
                format!("{position} / {}", fluxa_ui::format_time(duration))
            }
            _ => position,
        }
    });
    let details = vec![
        ("settings.summary_title_label", Some(player.title())),
        ("settings.summary_watched", watched),
    ];
    state.settings.playback_summary = stats.summary(details);
}
