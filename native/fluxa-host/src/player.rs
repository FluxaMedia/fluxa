use std::sync::{Arc, mpsc::Receiver};
use std::time::Duration;

use fluxa_core::FluxaCore;
use fluxa_ui::PlayerModel;
use serde_json::{Value, json};
use web_time::Instant;

use crate::{RendererState, UiTree, core_value, host_log, profile_language};

const CONTROLS_TIMEOUT: Duration = Duration::from_secs(3);
const SEEK_STEP: f64 = 10.0;
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
}

pub struct Thumbnail {
    pub time: f64,
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub enum VideoCommand {
    TogglePause,
    Seek(f64),
    SeekTo(f64),
    ToggleMute,
}

pub type DeviceOpener = Arc<
    dyn Fn(&wgpu::Adapter, &wgpu::DeviceDescriptor<'_>) -> Result<(wgpu::Device, wgpu::Queue), String>
        + Send
        + Sync,
>;

pub trait VideoBackend: Send {
    fn device_opener(&self) -> Option<DeviceOpener> {
        None
    }
    fn load(&mut self, instance: &wgpu::Instance, device: &wgpu::Device, url: &str);
    fn stop(&mut self);
    fn command(&mut self, command: VideoCommand);
    fn render(&mut self, device: &wgpu::Device) -> Option<wgpu::TextureView>;
    fn status(&mut self) -> VideoStatus;
    fn request_thumbnail(&mut self, _time: f64) {}
    fn take_thumbnail(&mut self) -> Option<Thumbnail> {
        None
    }
}

pub(crate) struct PlayerSession {
    meta: Value,
    started: bool,
    loaded_url: Option<String>,
    pub(crate) texture: Option<egui::TextureId>,
    last_activity: Instant,
    torrent_link: Option<String>,
    torrent_rx: Option<Receiver<Value>>,
    torrent_status: Option<Value>,
    error: Option<String>,
    status: VideoStatus,
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
}

impl PlayerSession {
    pub(crate) fn new(meta: Value) -> Self {
        Self {
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
        self.texture.is_none() || self.last_activity.elapsed() < CONTROLS_TIMEOUT
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

    pub(crate) fn model(&self) -> PlayerModel {
        let status = (self.texture.is_none() && self.torrent_link.is_some())
            .then(|| fluxa_ui::torrent_status_lines(self.torrent_status.as_ref()));
        PlayerModel {
            title: self.title(),
            video: self.texture.filter(|_| self.status.has_frame),
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
            thumbnail: self.thumbnail.as_ref().map(|(time, texture)| (*time, texture.id())),
            warnings: self.warnings.clone(),
            warnings_elapsed: self.warnings_clock,
            language: self.language.clone(),
        }
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
    let RendererState {
        player,
        session,
        video,
        gpu,
        ..
    } = state;
    let (Some(player), Some(session)) = (player.as_mut(), session.as_ref()) else {
        return;
    };
    let snapshot = session.snapshot();
    if !player.started && !session.has_queued_dispatches() {
        if let Some(command) = resolution_command(&snapshot, player) {
            match session.dispatch(command) {
                Ok(()) => {
                    if command_starts_playback(&snapshot) {
                        player.started = true;
                    }
                }
                Err(error) => player.error = Some(error),
            }
        }
    }
    poll_torrent(player, session, &snapshot);
    if player.loaded_url.is_none()
        && let Some(url) = snapshot
            .pointer("/player/resolvedUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
    {
        match (video.as_mut(), gpu.as_ref()) {
            (Some(video), Some(gpu)) => {
                host_log(format!("loading player url: {url}"));
                video.load(&gpu.instance, &gpu.device, url);
                player.loaded_url = Some(url.to_owned());
            }
            (None, _) => player.error = Some("No video backend on this platform".to_owned()),
            _ => {}
        }
    }
    if let Some(video) = video.as_mut() {
        player.status = video.status();
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
    player.episode_title = episode_title(&player.meta, &snapshot);
    if !player.warnings_requested {
        player.warnings_requested = true;
        player.language = profile_language(
            snapshot.pointer("/profile/active").unwrap_or(&Value::Null),
        );
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
        player.warnings_rx = content_warning_url(&player.meta, &snapshot)
            .map(|url| session.fetch_json(url));
    }
    tick_warnings(player);
}

fn episode_title(meta: &Value, snapshot: &Value) -> Option<String> {
    let id = snapshot.pointer("/player/currentVideoId").and_then(Value::as_str)?;
    meta.get("videos")?
        .as_array()?
        .iter()
        .find(|video| video.get("id").and_then(Value::as_str) == Some(id))?
        .get("name")
        .or_else(|| meta.get("title"))
        .and_then(Value::as_str)
        .filter(|title| !title.trim().is_empty())
        .map(ToOwned::to_owned)
}

fn content_warning_url(meta: &Value, snapshot: &Value) -> Option<String> {
    let candidates = [
        meta.get("id").and_then(Value::as_str),
        snapshot.pointer("/player/currentVideoId").and_then(Value::as_str),
    ];
    let imdb = candidates.into_iter().flatten().find_map(|id| {
        core_value("contentImdbId", json!({"id": id}))?
            .as_str()
            .filter(|id| !id.is_empty())
            .map(ToOwned::to_owned)
    })?;
    core_value("contentWarningUrl", json!({"imdbId": imdb}))?
        .as_str()
        .map(ToOwned::to_owned)
}

fn tick_warnings(player: &mut PlayerSession) {
    let delta = player.last_pump.elapsed().as_secs_f32();
    player.last_pump = Instant::now();
    if let Some(response) = player.warnings_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
        player.warnings_rx = None;
        player.warnings = response
            .and_then(|response| build_warnings(&response, &player.language))
            .unwrap_or_default();
    }
    if player.warnings.is_empty() || !player.status.has_frame {
        return;
    }
    let clock = player.warnings_clock.get_or_insert(0.0);
    if !player.status.paused {
        *clock += delta.min(0.25);
    }
    if *clock > fluxa_ui::content_warning_duration(player.warnings.len()) {
        player.warnings.clear();
        player.warnings_clock = None;
    }
}

fn build_warnings(response: &Value, language: &str) -> Option<Vec<(String, String)>> {
    let label = |key: &str| fluxa_ui::localized(&format!("content_warning.{key}"), language);
    let labels = ["nudity", "violence", "profanity", "alcohol", "frightening", "severe", "moderate", "mild"]
        .into_iter()
        .map(|key| (key.to_owned(), Value::String(label(key))))
        .collect::<serde_json::Map<_, _>>();
    let result = core_value(
        "buildContentWarnings",
        json!({"responseJson": response.to_string(), "labels": labels}),
    )?;
    Some(
        result
            .get("warnings")?
            .as_array()?
            .iter()
            .filter_map(|warning| {
                Some((
                    warning.get("label")?.as_str()?.to_owned(),
                    warning.get("severity")?.as_str()?.to_owned(),
                ))
            })
            .collect(),
    )
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

fn command_starts_playback(snapshot: &Value) -> bool {
    snapshot
        .pointer("/player/currentStreams")
        .and_then(Value::as_array)
        .is_some()
}

fn resolution_command(snapshot: &Value, player: &mut PlayerSession) -> Option<Value> {
    let meta = &player.meta;
    if let Some(streams) = snapshot
        .pointer("/player/currentStreams")
        .and_then(Value::as_array)
    {
        let index = snapshot
            .pointer("/player/currentStreamIndex")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0) as usize;
        let stream = streams.get(index).cloned().unwrap_or(Value::Null);
        let url = snapshot
            .pointer("/player/currentUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| playback_url(&stream, meta))?;
        return Some(json!({
            "type": "playerResolvePlaybackRequested",
            "url": url,
            "stream": stream,
            "currentVideoId": snapshot.pointer("/player/currentVideoId").cloned().unwrap_or(Value::Null),
            "title": player.title(),
        }));
    }
    if snapshot
        .pointer("/player/pendingStreamLoad")
        .is_some_and(Value::is_object)
    {
        return None;
    }
    let streams = snapshot
        .pointer("/player/directPlaybackTarget/streams")
        .and_then(Value::as_array)?;
    if streams.is_empty() {
        player.error = Some("No streams were found for this title".to_owned());
        return None;
    }
    let Some(content_id) = meta.get("id").and_then(Value::as_str) else {
        player.error = Some("This title has no content id".to_owned());
        return None;
    };
    let video_id = meta
        .get("lastVideoId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .unwrap_or(content_id);
    let profile = snapshot
        .pointer("/profile/active")
        .cloned()
        .unwrap_or(Value::Null);
    Some(json!({
        "type": "playerLoadStreamsRequested",
        "contentType": meta.get("type").and_then(Value::as_str).unwrap_or("movie"),
        "id": video_id,
        "currentVideoId": video_id,
        "initialVideoId": video_id,
        "initialStreams": streams,
        "initialStreamIndex": meta.get("lastStreamIndex").cloned().unwrap_or(json!(0)),
        "savedUrl": meta.get("lastStreamUrl").cloned().unwrap_or(Value::Null),
        "savedTitle": meta.get("lastStreamTitle").cloned().unwrap_or(Value::Null),
        "sourceSelectionMode": profile.get("streamSourceSelectionMode").and_then(Value::as_str).unwrap_or("manual"),
        "regexPattern": profile.get("streamSourceRegexPattern"),
        "title": meta.get("name").or_else(|| meta.get("title")),
        "originalName": meta.get("originalName"),
        "year": meta.get("year"),
        "language": profile_language(&profile),
        "profile": profile,
    }))
}

fn playback_url(stream: &Value, meta: &Value) -> Option<String> {
    let plan = core_value("playbackPreparePlan", json!({"stream": stream, "meta": meta}))?;
    let mode = plan.get("mode").and_then(Value::as_str)?;
    matches!(mode, "direct" | "torrent" | "external")
        .then(|| plan.get("url").and_then(Value::as_str))
        .flatten()
        .filter(|url| !url.is_empty())
        .map(ToOwned::to_owned)
}

fn poll_torrent(player: &mut PlayerSession, session: &fluxa_effects::SessionHandle, snapshot: &Value) {
    while let Some(status) = player.torrent_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
        player.torrent_status = Some(status);
    }
    let stream = snapshot
        .pointer("/player/currentStreamIndex")
        .and_then(Value::as_u64)
        .and_then(|index| {
            snapshot
                .pointer("/player/currentStreams")?
                .as_array()?
                .get(index as usize)
        });
    let Some(link) = stream.and_then(|stream| FluxaCore::stream_magnet_link_json(&stream.to_string()))
    else {
        return;
    };
    if player.torrent_link.as_deref() == Some(link.as_str()) {
        return;
    }
    let file_id = stream
        .and_then(|stream| stream.get("fileIdx").or_else(|| stream.get("fileIndex")))
        .and_then(Value::as_u64)
        .map(|index| index as usize);
    player.torrent_link = Some(link.clone());
    player.torrent_status = None;
    player.torrent_rx = Some(session.poll_torrent_status(link, file_id));
}

pub(crate) fn close(state: &mut RendererState) {
    let Some(player) = state.player.take() else {
        return;
    };
    if let Some(video) = state.video.as_mut() {
        video.stop();
    }
    if let (Some(texture), Some(gpu)) = (player.texture, state.gpu.as_mut()) {
        gpu.egui_renderer.free_texture(&texture);
    }
    state.ui = UiTree::default();
}

pub(crate) fn command(state: &mut RendererState, command: VideoCommand) {
    if let Some(player) = state.player.as_mut() {
        player.touch();
    }
    if let Some(video) = state.video.as_mut() {
        video.command(command);
    }
}

pub(crate) fn activate(state: &mut RendererState, node: u64) {
    match node {
        fluxa_ui::NODE_PLAYER_CLOSE => close(state),
        fluxa_ui::NODE_PLAYER_TOGGLE => command(state, VideoCommand::TogglePause),
        fluxa_ui::NODE_PLAYER_REWIND => command(state, VideoCommand::Seek(-SEEK_STEP)),
        fluxa_ui::NODE_PLAYER_FORWARD => command(state, VideoCommand::Seek(SEEK_STEP)),
        fluxa_ui::NODE_PLAYER_MUTE => command(state, VideoCommand::ToggleMute),
        fluxa_ui::NODE_PLAYER_FULLSCREEN => state.fullscreen_toggle = true,
        _ => {}
    }
}

pub(crate) enum KeyOutcome {
    Handled,
    Focus,
}

pub(crate) fn key(state: &mut RendererState, input: crate::KeyInput) -> KeyOutcome {
    use fluxa_renderer::ui::{GamepadButton, Key};
    use crate::KeyInput;
    let closes = matches!(
        input,
        KeyInput::Key(Key::Back | Key::Escape) | KeyInput::Gamepad(GamepadButton::East)
    );
    if closes {
        close(state);
        return KeyOutcome::Handled;
    }
    let Some(player) = state.player.as_mut() else {
        return KeyOutcome::Focus;
    };
    if player.controls_visible() {
        player.touch();
        return KeyOutcome::Focus;
    }
    player.touch();
    match input {
        KeyInput::Key(Key::Left) | KeyInput::Gamepad(GamepadButton::DPadLeft) => {
            command(state, VideoCommand::Seek(-SEEK_STEP))
        }
        KeyInput::Key(Key::Right) | KeyInput::Gamepad(GamepadButton::DPadRight) => {
            command(state, VideoCommand::Seek(SEEK_STEP))
        }
        KeyInput::Key(Key::Enter) | KeyInput::Gamepad(GamepadButton::South) => {
            command(state, VideoCommand::TogglePause)
        }
        _ => {}
    }
    KeyOutcome::Handled
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
