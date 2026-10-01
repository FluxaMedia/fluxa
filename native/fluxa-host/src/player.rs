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
mod input;
mod overlay;
mod resolve;
mod shuffle;
mod submit;
mod trailer;
mod upnext;
pub(crate) use input::*;
use overlay::Overlay;
use resolve::*;
pub(crate) use shuffle::*;
use upnext::*;

const CONTROLS_TIMEOUT: Duration = Duration::from_secs(3);
const DEFAULT_SEEK_STEP: f64 = 10.0;
const SCRUB_COMMIT: Duration = Duration::from_millis(900);
const MIN_BRIGHTNESS: f32 = 0.15;
const MEDIA_DRIFT: f64 = 1.5;
const FAST_SPEED: f64 = 2.0;
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
}

pub struct Thumbnail {
    pub time: f64,
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct VideoTrack {
    pub id: String,
    pub subtitle: bool,
    pub language: Option<String>,
    pub title: Option<String>,
    pub selected: bool,
    pub external: bool,
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
    Shaders(Vec<String>),
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
    fn stop(&mut self);
    fn media_session(&mut self, _plan: &Value) {}
    fn command(&mut self, command: VideoCommand);
    fn render(&mut self, device: &wgpu::Device) -> Option<wgpu::TextureView>;
    fn status(&mut self) -> VideoStatus;
    fn tracks(&mut self) -> Vec<VideoTrack> {
        Vec::new()
    }
    fn request_thumbnail(&mut self, _time: f64) {}
    fn take_thumbnail(&mut self) -> Option<Thumbnail> {
        None
    }
    fn passthrough(&self) -> bool {
        false
    }
}

pub(crate) struct PlayerSession {
    pub(crate) stale: Option<Value>,
    meta: Value,
    started: bool,
    loaded_url: Option<String>,
    pub(crate) texture: Option<egui::TextureId>,
    last_activity: Instant,
    torrent_link: Option<String>,
    torrent_rx: Option<Receiver<Value>>,
    torrent_status: Option<Value>,
    error: Option<String>,
    pub(crate) status: VideoStatus,
    language: String,
    description: Option<String>,
    episode_title: Option<String>,
    warnings_rx: Option<Receiver<Option<Value>>>,
    warnings_requested: bool,
    warnings: Vec<(String, String)>,
    warnings_clock: Option<f32>,
    last_pump: Instant,
    thumbnail: Option<(f64, egui::TextureHandle)>,
    thumbnail_requested: Option<(f64, Instant)>,
    outro_reached: bool,
    recommendations_rx: Option<Receiver<Vec<Value>>>,
    recommendations: Vec<Value>,
    recommendation_index: usize,
    load_progress: f32,
    scrub: Option<(f64, Instant)>,
    scrub_streak: u32,
    passthrough: bool,
    dispatched: Option<Value>,
    overlay: Overlay,
    panel: Option<Panel>,
    cast: crate::cast::Cast,
    trailer: Option<trailer::TrailerPlay>,
    sources: Option<Vec<Value>>,
    chosen: Option<usize>,
    source_filter: Option<String>,
    pub(crate) sources_scroll_max: f32,
    manual: bool,
    toast: Option<(Value, Instant)>,
    media_sent: Option<(Instant, String, f64)>,
    brightness: f32,
    speed_held: bool,
    speed: f64,
}

enum Panel {
    Submit(submit::Submit),
    Cast,
}

impl PlayerSession {
    pub(crate) fn trailer(meta: Value, urls: &[String]) -> Self {
        let mut player = Self::new(meta);
        player.trailer = Some(trailer::TrailerPlay::new(urls));
        player.started = true;
        player
    }

    pub(crate) fn new(meta: Value) -> Self {
        Self {
            stale: None,
            meta,
            started: false,
            loaded_url: None,
            texture: None,
            last_activity: Instant::now(),
            torrent_link: None,
            torrent_rx: None,
            torrent_status: None,
            error: None,
            status: VideoStatus::default(),
            language: "en".to_owned(),
            description: None,
            episode_title: None,
            warnings_rx: None,
            warnings_requested: false,
            warnings: Vec::new(),
            warnings_clock: None,
            last_pump: Instant::now(),
            thumbnail: None,
            thumbnail_requested: None,
            outro_reached: false,
            recommendations_rx: None,
            recommendations: Vec::new(),
            passthrough: false,
            dispatched: None,
            recommendation_index: 0,
            load_progress: 0.0,
            sources: None,
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
            media_sent: None,
            brightness: 1.0,
            speed_held: false,
            speed: 1.0,
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
        })
    }

    pub(crate) fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    pub(crate) fn presence(&self) -> crate::presence::Presence {
        crate::presence::Presence::Playing {
            title: self.title(),
            detail: self.episode_title.clone(),
            paused: self.status.paused,
            position: self.status.position,
            duration: self.status.duration,
            poster: self
                .meta
                .get("poster")
                .and_then(Value::as_str)
                .filter(|url| url.starts_with("http"))
                .map(ToOwned::to_owned),
        }
    }

    pub(crate) fn pick_manually(&mut self, snapshot: Option<&Value>) {
        self.manual = snapshot
            .and_then(|snapshot| snapshot.pointer("/profile/active/streamSourceSelectionMode"))
            .and_then(Value::as_str)
            .unwrap_or("manual")
            == "manual";
    }

    pub(crate) fn model(&self) -> PlayerModel {
        let status = (self.texture.is_none() && self.torrent_link.is_some())
            .then(|| fluxa_ui::torrent_status_lines(self.torrent_status.as_ref()));
        PlayerModel {
            title: self.title(),
            video: self.texture.filter(|_| self.status.has_frame),
            passthrough: self.passthrough && self.status.has_frame,
            status,
            error: self.error.clone().or_else(|| self.status.error.clone()),
            position: self.status.position,
            duration: self.status.duration,
            paused: self.status.paused,
            muted: self.status.muted,
            volume: self.status.volume,
            controls_visible: self.controls_visible(),
            show_pause_info: self.status.paused
                && self.status.has_frame
                && !self.controls_visible(),
            logo: self
                .meta
                .get("logo")
                .or_else(|| self.meta.get("logoUrl"))
                .and_then(Value::as_str)
                .filter(|url| !url.trim().is_empty())
                .map(ToOwned::to_owned),
            episode_title: self.episode_title.clone(),
            description: self.description.clone(),
            chapters: self.status.chapters.clone(),
            chapter_spans: self.overlay.chapter_spans(),
            skip: self.overlay.skip_card(),
            next_episode: self.overlay.next_card(),
            thumbnail: self
                .thumbnail
                .as_ref()
                .map(|(time, texture)| (*time, texture.id())),
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
            sources: match &self.sources {
                Some(streams) => Some(streams.iter().map(source_row).collect()),
                None if self.manual && self.chosen.is_none() && self.error.is_none() => {
                    Some(Vec::new())
                }
                None => None,
            },
            sources_loading: self.sources.is_none(),
            source_filter: self.source_filter.clone(),
            toast: self.toast_model(),
            panel: self.panel.as_ref().map(|panel| match panel {
                Panel::Submit(submit) => submit.model(&self.language),
                Panel::Cast => self.cast.model(&self.language),
            }),
            dim: 1.0 - self.brightness,
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
    fluxa_ui::PlayerSource {
        addon: lines("addonName"),
        name: lines("name"),
        detail,
    }
}

pub(crate) fn direct_playback_command(item: &Value, profile: &Value) -> Value {
    json!({
        "type": "directPlaybackRequested",
        "meta": item,
        "language": profile_language(profile),
        "profile": profile,
    })
}

pub(crate) fn pump(state: &mut RendererState) {
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
    if player.stale.is_some() {
        if player.stale.as_ref() == snapshot.get("player") {
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
        load_resolved(player, video, gpu, settings, &snapshot);
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
    load_resolved(player, video, gpu, settings, &snapshot);
    let mut advance = false;
    if let Some(video) = video.as_mut() {
        player.status = video.status();
        advance = overlay::tick(player, session, video.as_mut(), &snapshot);
        if let (Some(thumbnail), Some(gpu)) = (video.take_thumbnail(), gpu.as_ref()) {
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
    player.episode_title = episode_title(&player.meta, &snapshot);
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
                core_value("shortenSynopsis", json!({"text": text}))
                    .and_then(|value| value.as_str().map(ToOwned::to_owned))
                    .unwrap_or_else(|| text.to_owned())
            });
        player.warnings_rx =
            content_warning_url(&player.meta, &snapshot).map(|url| session.fetch_json(url));
    }
    tick_warnings(player);
    if shuffle.is_none() {
        tick_recommendations(player, session, settings, &snapshot);
    }
    publish_media(player, video);
    if advance {
        play_next(state);
    }
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
    if let Some(fields) = item.as_object_mut() {
        fields.retain(|key, _| !key.starts_with("last") && key != "timeOffset");
        fields.insert("lastVideoId".to_owned(), json!(id));
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
    snapshot: &Value,
) {
    if player.loaded_url.is_none()
        && let Some(url) = snapshot
            .pointer("/player/resolvedUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
    {
        match (video.as_mut(), gpu.as_ref()) {
            (Some(video), Some(gpu)) => {
                host_log(format!("loading player url: {url}"));
                video.configure(&settings.values);
                video.load(&gpu.instance, &gpu.device, url);
                video.command(VideoCommand::Shaders(shader_chain(settings)));
                apply_subtitle_style(video.as_mut(), &settings.values);
                player.loaded_url = Some(url.to_owned());
            }
            (None, _) => player.error = Some("No video backend on this platform".to_owned()),
            _ => {}
        }
    }
}

pub(crate) fn hover_seek(state: &mut RendererState, time: Option<f64>) {
    let (Some(player), Some(video)) = (state.player.as_mut(), state.video.as_mut()) else {
        return;
    };
    let Some(time) = time else {
        return;
    };
    player.touch();
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

pub(crate) fn close(state: &mut RendererState) {
    let Some(player) = state.player.take() else {
        return;
    };
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
        overlay::finish(&player, session, &snapshot, &tracks);
    }
    if let Some(video) = state.video.as_mut() {
        video.stop();
    }
    if let (Some(texture), Some(gpu)) = (player.texture, state.gpu.as_mut()) {
        gpu.egui_renderer.free_texture(&texture);
    }
    crate::reset_ui(state);
}

fn save_progress(session: &SessionHandle, player: &PlayerSession, snapshot: &Value) {
    if player.status.duration <= 0.0 || player.status.position < 5.0 {
        return;
    }
    let command = json!({
        "type": "savePlaybackProgressRequested",
        "profile": session.active_profile(),
        "meta": player.meta,
        "timeOffset": player.status.position as i64,
        "duration": player.status.duration as i64,
        "lastVideoId": snapshot.pointer("/player/currentVideoId"),
        "lastEpisodeName": player.episode_title,
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
    if item_id.is_empty() || player.status.duration <= 0.0 {
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
    if let Some(player) = state.player.as_mut() {
        player.touch();
        match &command {
            VideoCommand::Seek(delta) => player.toast("seek", *delta),
            VideoCommand::ToggleMute => player.toast("muted", f64::from(!player.status.muted)),
            VideoCommand::SetSpeed(rate) => player.toast("speed", *rate),
            _ => {}
        }
    }
    if let Some(video) = state.video.as_mut() {
        video.command(command);
    }
}

pub(crate) fn upscaling(settings: &SettingsModel) -> &str {
    if settings
        .str_value("animeUpscalingMode")
        .is_none_or(|mode| mode == "off")
    {
        return "off";
    }
    match settings.str_value("animeUpscalingModePreset") {
        Some(mode @ ("b" | "c")) => mode,
        _ => "a",
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

fn shader_chain(settings: &SettingsModel) -> Vec<String> {
    let mode = upscaling(settings);
    if mode == "off" {
        return Vec::new();
    }
    let tier = settings
        .str_value("animeUpscalingQuality")
        .unwrap_or("anime4k_m");
    core_value("anime4kShaderChain", json!({"tier": tier, "mode": mode}))
        .and_then(|chain| serde_json::from_value(chain).ok())
        .unwrap_or_default()
}

fn cycle_upscaling(state: &mut RendererState) {
    let next = match upscaling(&state.settings) {
        "off" => "a",
        "a" => "b",
        "b" => "c",
        _ => "off",
    };
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
