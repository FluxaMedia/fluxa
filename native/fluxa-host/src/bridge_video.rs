use std::sync::{Arc, Mutex};

use crate::{
    Thumbnail, ThumbnailPlan, ThumbnailProgress, VideoBackend, VideoCommand, VideoStatus,
    VideoTrack,
};
use fluxa_effects::core_value;
use serde_json::{Value, json};

#[derive(Default)]
pub struct VideoBridge {
    requests: Vec<Value>,
    status: VideoStatus,
    tracks: Vec<VideoTrack>,
    subtitle: Option<String>,
    thumbnail: Option<Thumbnail>,
    thumbnail_progress: Option<ThumbnailProgress>,
}

impl VideoBridge {
    pub fn take_requests(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.requests)
    }

    pub fn set_status(&mut self, mut status: VideoStatus) {
        status.chapters = std::mem::take(&mut self.status.chapters);
        self.status = status;
    }

    pub fn set_thumbnail(&mut self, thumbnail: Thumbnail) {
        self.thumbnail = Some(thumbnail);
    }

    pub fn set_thumbnail_progress(&mut self, progress: ThumbnailProgress) {
        self.thumbnail_progress = Some(progress);
    }

    pub fn set_chapters(&mut self, chapters: Vec<(f64, String)>) {
        self.status.chapters = chapters;
    }

    pub fn set_subtitle(&mut self, text: Option<String>) {
        self.subtitle = text;
    }

    pub fn set_tracks(&mut self, tracks: Vec<VideoTrack>) {
        self.tracks = tracks;
    }

    pub fn set_tracks_json(&mut self, json: &str) {
        let Ok(items) = serde_json::from_str::<Vec<Value>>(json) else {
            return;
        };
        let text = |item: &Value, key: &str| {
            item.get(key)
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        };
        let flag = |item: &Value, key: &str| item.get(key).and_then(Value::as_bool).unwrap_or(false);
        self.tracks = items
            .iter()
            .filter_map(|item| {
                Some(VideoTrack {
                    id: text(item, "id")?,
                    subtitle: flag(item, "subtitle"),
                    language: text(item, "language"),
                    title: text(item, "title"),
                    selected: flag(item, "selected"),
                    external: flag(item, "external"),
                })
            })
            .collect();
    }

    pub fn set_chapters_json(&mut self, json: &str) {
        let Ok(items) = serde_json::from_str::<Vec<Value>>(json) else {
            return;
        };
        self.status.chapters = items
            .iter()
            .filter_map(|item| {
                Some((
                    item.get("startMs")?.as_u64()? as f64 / 1000.0,
                    item.get("title")?.as_str()?.to_owned(),
                ))
            })
            .collect();
    }
}

pub struct BridgeVideo(pub Arc<Mutex<VideoBridge>>);

impl BridgeVideo {
    fn send(&self, request: Value) {
        if let Ok(mut bridge) = self.0.lock() {
            bridge.requests.push(request);
        }
    }
}

impl VideoBackend for BridgeVideo {
    fn configure(&mut self, settings: &Value) {
        let Some(Value::Object(mut plan)) = core_value("videoConfigPlan", settings.clone()) else {
            return;
        };
        plan.insert("type".to_owned(), json!("configure"));
        self.send(Value::Object(plan));
    }

    fn load(&mut self, _instance: &wgpu::Instance, _device: &wgpu::Device, url: &str) {
        if let Ok(mut bridge) = self.0.lock() {
            bridge.status = VideoStatus::default();
            bridge.tracks.clear();
            bridge.subtitle = None;
            bridge.thumbnail = None;
            bridge.thumbnail_progress = None;
        }
        self.send(json!({"type": "load", "url": url}));
    }

    fn load_preview(&mut self, _instance: &wgpu::Instance, _device: &wgpu::Device, url: &str) {
        if let Ok(mut bridge) = self.0.lock() {
            bridge.status = VideoStatus::default();
        }
        self.send(json!({"type": "loadPreview", "url": url}));
    }

    fn place_preview(&mut self, rect: Option<[f32; 4]>) {
        self.send(json!({"type": "previewWindow", "rect": rect}));
    }

    fn stop(&mut self) {
        if let Ok(mut bridge) = self.0.lock() {
            bridge.status = VideoStatus::default();
            bridge.tracks.clear();
            bridge.subtitle = None;
            bridge.thumbnail = None;
            bridge.thumbnail_progress = None;
        }
        self.send(json!({"type": "stop"}));
    }

    fn media_session(&mut self, plan: &Value) {
        self.send(json!({"type": "mediaSession", "plan": plan}));
    }

    fn command(&mut self, command: VideoCommand) {
        let request = match command {
            VideoCommand::TogglePause => json!({"type": "togglePause"}),
            VideoCommand::Seek(delta) => json!({"type": "seek", "seconds": delta}),
            VideoCommand::SeekTo(position) => json!({"type": "seekTo", "seconds": position}),
            VideoCommand::ToggleMute => json!({"type": "toggleMute"}),
            VideoCommand::SetVolume(volume) => json!({"type": "setVolume", "value": volume}),
            VideoCommand::SetSpeed(rate) => json!({"type": "setSpeed", "value": rate}),
            VideoCommand::SelectTracks(selection) => {
                if let Ok(mut bridge) = self.0.lock() {
                    for track in bridge.tracks.iter_mut() {
                        let chosen = if track.subtitle {
                            selection.subtitle.as_ref()
                        } else {
                            selection.audio.as_ref()
                        };
                        if let Some(id) = chosen {
                            track.selected = &track.id == id;
                        } else if track.subtitle && selection.subtitles_off {
                            track.selected = false;
                        }
                    }
                }
                json!({
                    "type": "selectTracks",
                    "audio": selection.audio,
                    "subtitle": selection.subtitle,
                    "subtitlesOff": selection.subtitles_off,
                })
            }
            VideoCommand::Mpv(args) => json!({"type": "mpv", "args": args}),
            VideoCommand::Shaders(files) => json!({"type": "shaders", "files": files}),
            VideoCommand::Fit(mode) => json!({"type": "fit", "mode": mode}),
        };
        self.send(request);
    }

    fn render(&mut self, _device: &wgpu::Device) -> Option<wgpu::TextureView> {
        None
    }

    fn status(&mut self) -> VideoStatus {
        self.0
            .lock()
            .map(|bridge| bridge.status.clone())
            .unwrap_or_default()
    }

    fn subtitle(&mut self) -> Option<String> {
        self.0.lock().ok()?.subtitle.clone()
    }

    fn tracks(&mut self) -> Vec<VideoTrack> {
        self.0
            .lock()
            .map(|bridge| bridge.tracks.clone())
            .unwrap_or_default()
    }

    fn thumbnails(&mut self, plan: &ThumbnailPlan) {
        self.send(json!({
            "type": "thumbnails",
            "enabled": plan.enabled,
            "url": plan.url,
            "headers": plan.headers,
            "interval": plan.interval,
            "sheet": plan.sheet,
            "hold": plan.hold && plan.enabled,
        }));
    }

    fn thumbnail_progress(&mut self) -> Option<ThumbnailProgress> {
        self.0.lock().ok()?.thumbnail_progress
    }

    fn release_start(&mut self) {
        self.send(json!({"type": "releaseStart"}));
    }

    fn request_thumbnail(&mut self, time: f64) {
        self.send(json!({"type": "thumbnail", "time": time}));
    }

    fn take_thumbnail(&mut self) -> Option<Thumbnail> {
        self.0.lock().ok()?.thumbnail.take()
    }

    fn passthrough(&self) -> bool {
        true
    }
}
