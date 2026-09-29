use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use fluxa_host::Thumbnail;

const THUMBNAIL_SIZE: [usize; 2] = [320, 180];
const CHAPTER_REFRESH: Duration = Duration::from_secs(2);

pub fn thumbnail_url(url: &str) -> String {
    if url.contains("/stream/fname") {
        format!("{url}&role=auxiliary")
    } else {
        url.to_owned()
    }
}

pub struct ThumbnailWorker {
    pub requests: Sender<f64>,
    pub results: Receiver<Thumbnail>,
}

impl ThumbnailWorker {
    pub fn spawn(url: String) -> Self {
        let (requests, request_rx) = mpsc::channel::<f64>();
        let (result_tx, results) = mpsc::channel();
        std::thread::spawn(move || {
            let mut renderer = match fluxa_mpv::MpvThumbnailRenderer::new()
                .and_then(|mut renderer| renderer.load_thumbnail(&url).map(|_| renderer))
            {
                Ok(renderer) => renderer,
                Err(error) => {
                    eprintln!("[fluxa-desktop] seek thumbnails unavailable: {error}");
                    return;
                }
            };
            while let Ok(mut time) = request_rx.recv() {
                while let Ok(next) = request_rx.try_recv() {
                    time = next;
                }
                match render_thumbnail(&mut renderer, time) {
                    Ok(rgba) => {
                        let thumbnail = Thumbnail {
                            time,
                            size: THUMBNAIL_SIZE,
                            rgba,
                        };
                        if result_tx.send(thumbnail).is_err() {
                            return;
                        }
                    }
                    Err(error) => eprintln!("[fluxa-desktop] seek thumbnail failed: {error}"),
                }
            }
        });
        Self { requests, results }
    }

    pub fn latest(&self) -> Option<Thumbnail> {
        let mut latest = None;
        while let Ok(thumbnail) = self.results.try_recv() {
            latest = Some(thumbnail);
        }
        latest
    }
}

fn render_thumbnail(
    renderer: &mut fluxa_mpv::MpvThumbnailRenderer,
    time: f64,
) -> Result<Vec<u8>, String> {
    renderer.set_paused(false)?;
    renderer.seek_thumbnail_to(time)?;
    renderer.pump_events();
    std::thread::sleep(Duration::from_millis(50));
    renderer.set_paused(true)?;
    renderer.pump_events();
    std::thread::sleep(Duration::from_millis(20));
    renderer.render_thumbnail(THUMBNAIL_SIZE[0] as i32, THUMBNAIL_SIZE[1] as i32)
}

#[derive(Default)]
pub struct Chapters {
    list: Vec<(f64, String)>,
    at: Option<Instant>,
}

impl Chapters {
    pub fn get(&mut self, client: &fluxa_mpv::MpvClientHandle) -> &[(f64, String)] {
        if self.at.is_none_or(|at| at.elapsed() >= CHAPTER_REFRESH) {
            self.at = Some(Instant::now());
            self.list = client
                .chapters_json()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .and_then(|value| {
                    value.get("chapters")?.as_array().map(|chapters| {
                        chapters
                            .iter()
                            .filter_map(|chapter| {
                                Some((
                                    chapter.get("startMs")?.as_u64()? as f64 / 1000.0,
                                    chapter.get("title")?.as_str()?.to_owned(),
                                ))
                            })
                            .collect()
                    })
                })
                .unwrap_or_default();
        }
        &self.list
    }
}

pub fn buffering(position: &fluxa_mpv::PlayerPositionStatus) -> Option<f32> {
    let number = |value: Option<&String>| value.and_then(|value| value.parse::<f32>().ok());
    if position.paused_for_cache.as_deref() == Some("yes")
        && let Some(state) = number(position.cache_buffering_state.as_ref())
    {
        return Some(state / 100.0);
    }
    number(position.demuxer_cache_duration.as_ref())
        .filter(|seconds| *seconds > 0.0)
        .map(|seconds| (seconds / 5.0).min(1.0))
}

pub fn shader_dir() -> Option<PathBuf> {
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("mpv-shaders/anime4k")));
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fluxa-mpv/shaders/anime4k");
    [beside, Some(source)]
        .into_iter()
        .flatten()
        .find(|dir| dir.is_dir())
}
