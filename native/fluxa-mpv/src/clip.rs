use super::*;
use std::time::{Duration, Instant};

const OPEN_TIMEOUT: Duration = Duration::from_secs(20);
const STALL_TIMEOUT: Duration = Duration::from_secs(8);

pub struct MpvClip {
    player: MpvThumbnailRenderer,
    size: [u32; 2],
    fallback_interval: Duration,
    last_frame: Option<Instant>,
}

pub struct ClipFrame {
    pub rgba: Vec<u8>,
    pub size: [u32; 2],
    pub duration: Duration,
}

impl MpvClip {
    pub fn open(url: &str, target: [u32; 2]) -> Result<Self, String> {
        let (client, render) = MpvClientHandle::new_clip()?;
        let mut player = MpvThumbnailRenderer { client, render };
        player.load_thumbnail(url)?;
        let started = Instant::now();
        let (width, height) = loop {
            player.pump_events();
            let dimension = |name: &str| {
                player
                    .query_property(name)
                    .and_then(|value| value.parse::<u32>().ok())
                    .filter(|value| *value > 0)
            };
            if let (Some(width), Some(height)) =
                (dimension("video-params/w"), dimension("video-params/h"))
            {
                break (width, height);
            }
            if started.elapsed() > OPEN_TIMEOUT {
                return Err("video has no decodable frames".to_owned());
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let scale = (target[0] as f32 / width as f32)
            .min(target[1] as f32 / height as f32)
            .min(1.0);
        let size = [
            ((width as f32 * scale).round() as u32).max(2),
            ((height as f32 * scale).round() as u32).max(2),
        ];
        let fps = player
            .query_property("container-fps")
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|fps| *fps > 1.0)
            .unwrap_or(30.0);
        Ok(Self {
            player,
            size,
            fallback_interval: Duration::from_secs_f64(1.0 / fps),
            last_frame: None,
        })
    }

    pub fn next_frame(&mut self) -> Result<ClipFrame, String> {
        let started = Instant::now();
        loop {
            self.player.pump_events();
            let rgba = self
                .player
                .render
                .render_clip_frame(self.size[0] as i32, self.size[1] as i32)?;
            if let Some(rgba) = rgba {
                let now = Instant::now();
                let duration = self
                    .last_frame
                    .map_or(self.fallback_interval, |last| now - last)
                    .clamp(Duration::from_millis(8), Duration::from_secs(1));
                self.last_frame = Some(now);
                return Ok(ClipFrame {
                    rgba,
                    size: self.size,
                    duration,
                });
            }
            if started.elapsed() > STALL_TIMEOUT {
                return Err("video stalled".to_owned());
            }
            std::thread::sleep(Duration::from_millis(3));
        }
    }
}
