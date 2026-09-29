use std::sync::{Arc, Mutex};

use crate::{VideoBackend, VideoCommand, VideoStatus};
use serde_json::{Value, json};

#[derive(Default)]
pub struct VideoBridge {
    requests: Vec<Value>,
    status: VideoStatus,
}

impl VideoBridge {
    pub fn take_requests(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.requests)
    }

    pub fn set_status(&mut self, status: VideoStatus) {
        self.status = status;
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
        let text = |key: &str| {
            settings
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
        };
        self.send(json!({
            "type": "configure",
            "mpvOptions": text("mpvCustomOptions"),
            "audioProcessingMode": text("audioProcessingMode"),
            "audioLanguage": text("preferredAudioLanguage"),
            "subtitleLanguage": text("preferredSubtitleLanguage"),
        }));
    }

    fn load(&mut self, _instance: &wgpu::Instance, _device: &wgpu::Device, url: &str) {
        if let Ok(mut bridge) = self.0.lock() {
            bridge.status = VideoStatus::default();
        }
        self.send(json!({"type": "load", "url": url}));
    }

    fn stop(&mut self) {
        if let Ok(mut bridge) = self.0.lock() {
            bridge.status = VideoStatus::default();
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
            VideoCommand::Shaders(_) | VideoCommand::SelectTracks(_) => return,
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

    fn passthrough(&self) -> bool {
        true
    }
}
