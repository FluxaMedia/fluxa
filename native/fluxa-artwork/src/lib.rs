//! The single artwork pipeline used by every native Fluxa host.
//!
//! A provider may expose a smaller URL (TMDB currently does), but that is an
//! optional fetch optimization. The important invariant is that every source
//! is fetched asynchronously, decoded away from the UI thread, reduced to the
//! component's target size, and cached by source URL plus target size.

use image::{
    AnimationDecoder, Frame, ImageDecoder, ImageFormat, ImageReader, RgbaImage,
    codecs::{gif::GifDecoder, webp::WebPDecoder},
    imageops::FilterType,
};
use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    io::Cursor,
    path::{Path, PathBuf},
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, SystemTime},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use web_time::Instant;

pub mod webp_animation;

static FETCH_PROXY: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn set_fetch_proxy(base_url: &str) {
    let _ = FETCH_PROXY.set(base_url.trim_end_matches('/').to_owned());
}

fn proxied(url: &str) -> std::borrow::Cow<'_, str> {
    match FETCH_PROXY.get() {
        Some(base) if url.starts_with("http://") || url.starts_with("https://") => {
            let encoded: String = url
                .bytes()
                .map(|byte| match byte {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                        (byte as char).to_string()
                    }
                    _ => format!("%{byte:02X}"),
                })
                .collect();
            std::borrow::Cow::Owned(format!("{base}/proxy?url={encoded}"))
        }
        _ => std::borrow::Cow::Borrowed(url),
    }
}

pub const MAX_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_DECODE_SIDE: u32 = 4096;
pub const MAX_ARTWORK_WIDTH: u32 = 1920;
pub const MAX_ARTWORK_HEIGHT: u32 = 1080;
// A home feed can contain several shelves. Keep enough requests in flight to
// prepare the initial document before the user scrolls, while still bounding
// network and decode pressure. Decoding itself remains on the artwork worker
// runtime, never on the UI thread.
pub const MAX_IN_FLIGHT: usize = 32;
// Keep background warming from occupying every network/decode slot before
// newly visible artwork gets a chance to start.
const MAX_PREFETCH_IN_FLIGHT: usize = 8;
const MAX_PREFETCH_DECODE_IN_FLIGHT: usize = 1;
// Animated WebP/GIF composition is substantially more CPU- and memory-heavy
// than a poster raster decode. Keep a burst of newly-visible animations from
// occupying every decode worker while the user is scrolling.
const MAX_ANIMATION_DECODE_IN_FLIGHT: usize = 2;
pub const MAX_CACHE_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_CACHE_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);
#[cfg(not(target_arch = "wasm32"))]
pub const MAX_ANIMATION_BYTES: usize = 64 * 1024 * 1024;
#[cfg(target_arch = "wasm32")]
pub const MAX_ANIMATION_BYTES: usize = 16 * 1024 * 1024;
#[cfg(not(target_arch = "wasm32"))]
pub const MAX_ANIMATION_ATLAS_SIDE: u32 = 4096;
#[cfg(target_arch = "wasm32")]
pub const MAX_ANIMATION_ATLAS_SIDE: u32 = 2048;
const MAX_SOURCE_ANIMATION_FRAMES: usize = 4096;
#[cfg(not(target_arch = "wasm32"))]
const MAX_ANIMATION_FRAMES: usize = 180;
#[cfg(target_arch = "wasm32")]
const MAX_ANIMATION_FRAMES: usize = 72;
#[cfg(not(target_arch = "wasm32"))]
const MIN_ANIMATION_FRAME_INTERVAL: Duration = Duration::from_millis(33);
#[cfg(target_arch = "wasm32")]
const MIN_ANIMATION_FRAME_INTERVAL: Duration = Duration::from_millis(80);
const ANIMATION_CACHE_MAGIC: &[u8] = b"FXAN2";
const FAILED_REQUEST_RETRY_DELAY: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Priority {
    Hero,
    #[default]
    Visible,
    Prefetch,
}

#[derive(Clone, Debug)]
pub struct PreparedArtwork {
    /// The original normalized source URL. Hosts use this to associate the
    /// decoded raster with their texture registry.
    pub source_url: String,
    pub target_size: [u32; 2],
    pub image: RgbaImage,
    pub animation_atlas: Option<AnimatedAtlas>,
    pub animation_requested: bool,
    pub animation_started_at: Option<Instant>,
    pub transparent_ratio: f32,
}

#[derive(Clone, Debug)]
pub struct AnimatedFrame {
    pub image: RgbaImage,
    pub duration: Duration,
}

#[derive(Clone, Debug)]
pub struct AnimatedAtlas {
    pub image: RgbaImage,
    pub frames: Vec<AnimatedAtlasFrame>,
}

#[derive(Clone, Copy, Debug)]
pub struct AnimatedAtlasFrame {
    /// Normalized UV coordinates of the frame tile within the atlas.
    pub uv: [f32; 4],
    pub image_size: [u32; 2],
    pub duration: Duration,
}

pub fn animation_frame_at(
    frames: &[AnimatedAtlasFrame],
    started_at: Instant,
    now: Instant,
) -> Option<(usize, Instant)> {
    if frames.is_empty() {
        return None;
    }

    let frame_duration = |frame: &AnimatedAtlasFrame| frame.duration.max(Duration::from_millis(1));
    let cycle_duration = frames
        .iter()
        .map(frame_duration)
        .fold(Duration::ZERO, Duration::saturating_add);
    let elapsed = now.saturating_duration_since(started_at);
    let position_nanos = elapsed.as_nanos() % cycle_duration.as_nanos();
    let mut position = Duration::from_nanos(position_nanos as u64);

    for (index, frame) in frames.iter().enumerate() {
        let duration = frame_duration(frame);
        if position < duration {
            return Some((index, now + (duration - position)));
        }
        position -= duration;
    }

    let duration = frame_duration(&frames[0]);
    Some((0, now + duration))
}

#[derive(Clone, Debug)]
struct PreparedImage {
    image: RgbaImage,
    animation_atlas: Option<AnimatedAtlas>,
}

pub struct ArtworkFetcher {
    client: reqwest::Client,
    #[cfg(not(target_arch = "wasm32"))]
    runtime: tokio::runtime::Runtime,
    decode_slots: Arc<Semaphore>,
    animation_decode_slots: Arc<Semaphore>,
    prefetch_decode_slots: Arc<Semaphore>,
    sender: Sender<(
        String,
        [u32; 2],
        bool,
        Option<Instant>,
        Result<(PreparedImage, f32), String>,
    )>,
    receiver: Receiver<(
        String,
        [u32; 2],
        bool,
        Option<Instant>,
        Result<(PreparedImage, f32), String>,
    )>,
    /// Covers both queued and active work, so repeated immediate-mode draws
    /// coalesce into one fetch instead of filling the queue with duplicates.
    pending: HashSet<String>,
    in_flight: HashSet<String>,
    in_flight_prefetch: HashSet<String>,
    queued: Vec<QueuedArtwork>,
    next_sequence: u64,
    failed: HashMap<String, Instant>,
    cache_dir: Option<PathBuf>,
}

#[derive(Clone)]
struct QueuedArtwork {
    key: String,
    source_url: String,
    target_size: [u32; 2],
    priority: Priority,
    sequence: u64,
    animated: bool,
    animation_started_at: Option<Instant>,
}

impl ArtworkFetcher {
    pub fn new(cache_dir: Option<PathBuf>, worker_threads: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let decode_concurrency = worker_threads.clamp(2, 6);
        #[cfg(not(target_arch = "wasm32"))]
        let client = reqwest::Client::builder()
            .user_agent("Fluxa/1.0")
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::Client::new();
        #[cfg(not(target_arch = "wasm32"))]
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(worker_threads.max(1))
            .max_blocking_threads(decode_concurrency)
            .enable_all()
            .thread_name("fluxa-artwork")
            .build()
            .expect("Fluxa artwork runtime must start");
        Self {
            client,
            #[cfg(not(target_arch = "wasm32"))]
            runtime,
            decode_slots: Arc::new(Semaphore::new(decode_concurrency)),
            animation_decode_slots: Arc::new(Semaphore::new(MAX_ANIMATION_DECODE_IN_FLIGHT)),
            prefetch_decode_slots: Arc::new(Semaphore::new(MAX_PREFETCH_DECODE_IN_FLIGHT)),
            sender,
            receiver,
            pending: HashSet::new(),
            in_flight: HashSet::new(),
            in_flight_prefetch: HashSet::new(),
            queued: Vec::new(),
            next_sequence: 0,
            failed: HashMap::new(),
            cache_dir,
        }
    }

    pub fn request(
        &mut self,
        source_url: Option<&str>,
        target_size: [u32; 2],
        priority: Priority,
    ) -> Option<String> {
        self.request_mode(source_url, target_size, priority, false, None)
    }

    pub fn request_animated(
        &mut self,
        source_url: Option<&str>,
        target_size: [u32; 2],
        priority: Priority,
    ) -> Option<String> {
        self.request_mode(
            source_url,
            target_size,
            priority,
            true,
            Some(Instant::now()),
        )
    }

    pub fn prefetch_animated(
        &mut self,
        source_url: Option<&str>,
        target_size: [u32; 2],
        priority: Priority,
    ) -> Option<String> {
        self.request_mode(source_url, target_size, priority, true, None)
    }

    fn request_mode(
        &mut self,
        source_url: Option<&str>,
        target_size: [u32; 2],
        priority: Priority,
        animated: bool,
        animation_started_at: Option<Instant>,
    ) -> Option<String> {
        let source_url = normalize_url(source_url?)?;
        let target_size = bounded_target(target_size);
        let raster_key = request_key(&source_url, target_size);
        let key = if animated {
            format!("{raster_key}#animated")
        } else {
            raster_key.clone()
        };
        if self
            .failed
            .get(&key)
            .is_some_and(|failed_at| failed_at.elapsed() < FAILED_REQUEST_RETRY_DELAY)
        {
            return Some(raster_key);
        }
        self.failed.remove(&key);

        if self.pending.contains(&key) {
            if let Some(queued) = self.queued.iter_mut().find(|queued| queued.key == key) {
                if priority_rank(priority) > priority_rank(queued.priority) {
                    queued.priority = priority;
                }
                if queued.animation_started_at.is_none() {
                    queued.animation_started_at = animation_started_at;
                }
            }
            return Some(key);
        }

        self.pending.insert(key.clone());
        let result_key = key.clone();
        let request = QueuedArtwork {
            key,
            source_url,
            target_size,
            priority,
            sequence: self.next_sequence,
            animated,
            animation_started_at,
        };
        self.next_sequence = self.next_sequence.wrapping_add(1);
        if self.can_start(priority) {
            self.start_request(request);
        } else {
            self.queued.push(request);
        }
        Some(if animated { raster_key } else { result_key })
    }

    fn start_request(&mut self, request: QueuedArtwork) {
        if request.priority == Priority::Prefetch {
            self.in_flight_prefetch.insert(request.key.clone());
        }
        self.in_flight.insert(request.key.clone());
        let sender = self.sender.clone();
        let client = self.client.clone();
        let cache_dir = self.cache_dir.clone();
        let decode_slots = Arc::clone(&self.decode_slots);
        let animation_decode_slots = Arc::clone(&self.animation_decode_slots);
        let prefetch_decode_slots = Arc::clone(&self.prefetch_decode_slots);
        let key_for_task = request.key;
        let source_url = request.source_url;
        let target_size = request.target_size;
        let priority = request.priority;
        let animated = request.animated;
        let request_id = request.sequence;
        let animation_started_at = request.animation_started_at;
        self.spawn_task(async move {
            let cache_path = cache_dir
                .as_deref()
                .map(|directory| cache_path(directory, &key_for_task));
            let cached = if let Some(path) = cache_path.clone() {
                let (decode_permit, prefetch_permit, animation_decode_permit) = acquire_decode_permits(
                    Arc::clone(&decode_slots),
                    Arc::clone(&animation_decode_slots),
                    Arc::clone(&prefetch_decode_slots),
                    priority,
                    animated,
                )
                .await;
                blocking(move || {
                    let _decode_permit = decode_permit;
                    let _animation_decode_permit = animation_decode_permit;
                    let _prefetch_permit = prefetch_permit;
                    if animated {
                        read_animation_cache(&path)
                    } else {
                        read_raster_cache(&path).map(|image| PreparedImage {
                            image,
                            animation_atlas: None,
                        })
                    }
                })
                    .await
                    .ok()
                    .flatten()
            } else {
                None
            };

            let result = if let Some(prepared) = cached {
                Ok(prepared)
            } else {
                let resource_url = if animated {
                    source_url.clone()
                } else {
                    fetch_url(&source_url, target_size)
                };
                let fetch_started_at = Instant::now();
                let bytes = fetch_bytes(&client, &resource_url).await;
                match bytes {
                    Ok(bytes) => {
                        let fetch_elapsed = fetch_started_at.elapsed();
                        let path_for_write = cache_path.clone();
                        let (decode_permit, prefetch_permit, animation_decode_permit) = acquire_decode_permits(
                            decode_slots,
                            animation_decode_slots,
                            prefetch_decode_slots,
                            priority,
                            animated,
                        )
                        .await;
                        blocking(move || {
                            let _decode_permit = decode_permit;
                            let _animation_decode_permit = animation_decode_permit;
                            let _prefetch_permit = prefetch_permit;
                            let preparation_started_at = Instant::now();
                            let prepared = if animated {
                                match decode_animated_target(&bytes, target_size) {
                                    Ok(Some(frames)) if frames.len() > 1 => {
                                        let image = frames[0].image.clone();
                                        match pack_animation_atlas(frames, MAX_ANIMATION_ATLAS_SIDE) {
                                            Ok(atlas) => PreparedImage {
                                                image,
                                                animation_atlas: atlas,
                                                                        },
                                            Err(error) => {
                                                eprintln!(
                                                    "[fluxa-native] animation atlas preparation failed: {error}"
                                                );
                                                PreparedImage {
                                                    image,
                                                    animation_atlas: None,
                                                                                }
                                            }
                                        }
                                    }
                                    _ => PreparedImage {
                                        image: decode_target(&bytes, target_size)?,
                                        animation_atlas: None,
                                                        },
                                }
                            } else {
                                PreparedImage {
                                    image: decode_target(&bytes, target_size)?,
                                    animation_atlas: None,
                                                }
                            };
                            if animated {
                                eprintln!(
                                    "[fluxa-native] animation job {request_id}: fetch={}ms, prepare={}ms, frames={}, bytes={}",
                                    fetch_elapsed.as_millis(),
                                    preparation_started_at.elapsed().as_millis(),
                                    prepared.animation_atlas.as_ref().map_or(1, |atlas| atlas.frames.len()),
                                    bytes.len()
                                );
                            }
                            if let Some(path) = path_for_write.as_deref() {
                                match &prepared.animation_atlas {
                                    Some(atlas) => write_animation_cache(path, atlas),
                                    None if !animated => write_raster_cache(path, &prepared.image),
                                    None => {}
                                }
                            }
                            Ok(prepared)
                        })
                        .await
                        .map_err(|error| error.to_string())
                        .and_then(|result| result)
                    }
                    Err(error) => Err(error),
                }
            };
            let result = result.map(|prepared| {
                let ratio = transparent_ratio(
                    prepared
                        .animation_atlas
                        .as_ref()
                        .map_or(&prepared.image, |atlas| &atlas.image),
                );
                (prepared, ratio)
            });
            let _ = sender.send((
                source_url,
                target_size,
                animated,
                animation_started_at,
                result,
            ));
        });
    }

    pub fn poll(&mut self, limit: usize) -> Vec<PreparedArtwork> {
        let mut ready = Vec::new();
        for _ in 0..limit {
            let Ok((source_url, target_size, animated, animation_started_at, result)) =
                self.receiver.try_recv()
            else {
                break;
            };
            let raster_key = request_key(&source_url, target_size);
            let key = if animated {
                format!("{raster_key}#animated")
            } else {
                raster_key
            };
            self.pending.remove(&key);
            self.in_flight.remove(&key);
            self.in_flight_prefetch.remove(&key);
            match result {
                Ok((image, transparent_ratio)) => {
                    self.failed.remove(&key);
                    ready.push(PreparedArtwork {
                        source_url,
                        target_size,
                        image: image.image,
                        animation_atlas: image.animation_atlas,
                        transparent_ratio,
                        animation_requested: animated,
                        animation_started_at,
                    });
                }
                Err(error) => {
                    self.failed.insert(key, Instant::now());
                    eprintln!(
                        "[fluxa-native] artwork preparation failed for {source_url}: {error}"
                    );
                }
            }
        }
        self.start_queued();
        ready
    }

    fn start_queued(&mut self) {
        while self.in_flight.len() < MAX_IN_FLIGHT && !self.queued.is_empty() {
            let Some(index) = next_queued_index(&self.queued) else {
                break;
            };
            if self.queued[index].priority == Priority::Prefetch
                && self.in_flight_prefetch.len() >= MAX_PREFETCH_IN_FLIGHT
            {
                break;
            }
            let request = self.queued.swap_remove(index);
            self.start_request(request);
        }
    }

    fn can_start(&self, priority: Priority) -> bool {
        self.in_flight.len() < MAX_IN_FLIGHT
            && (priority != Priority::Prefetch
                || self.in_flight_prefetch.len() < MAX_PREFETCH_IN_FLIGHT)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_task(&self, task: impl std::future::Future<Output = ()> + Send + 'static) {
        self.runtime.spawn(task);
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn_task(&self, task: impl std::future::Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(task);
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn animated_request_is_backing_off(&self, source_url: &str, target_size: [u32; 2]) -> bool {
        let Some(source_url) = normalize_url(source_url) else {
            return false;
        };
        let key = format!(
            "{}#animated",
            request_key(&source_url, bounded_target(target_size))
        );
        self.failed
            .get(&key)
            .is_some_and(|failed_at| failed_at.elapsed() < FAILED_REQUEST_RETRY_DELAY)
    }
}

async fn acquire_decode_permits(
    decode_slots: Arc<Semaphore>,
    animation_decode_slots: Arc<Semaphore>,
    prefetch_decode_slots: Arc<Semaphore>,
    priority: Priority,
    animated: bool,
) -> (
    OwnedSemaphorePermit,
    Option<OwnedSemaphorePermit>,
    Option<OwnedSemaphorePermit>,
) {
    let prefetch_permit = if priority == Priority::Prefetch {
        Some(
            prefetch_decode_slots
                .acquire_owned()
                .await
                .expect("artwork prefetch semaphore remains open"),
        )
    } else {
        None
    };
    let animation_permit = if animated {
        Some(
            animation_decode_slots
                .acquire_owned()
                .await
                .expect("animation decode semaphore remains open"),
        )
    } else {
        None
    };
    let decode_permit = decode_slots
        .acquire_owned()
        .await
        .expect("artwork decode semaphore remains open");
    (decode_permit, prefetch_permit, animation_permit)
}

const fn priority_rank(priority: Priority) -> u8 {
    match priority {
        Priority::Hero => 2,
        Priority::Visible => 1,
        Priority::Prefetch => 0,
    }
}

fn next_queued_index(queued: &[QueuedArtwork]) -> Option<usize> {
    queued
        .iter()
        .enumerate()
        .max_by_key(|(_, item)| {
            (
                priority_rank(item.priority),
                item.animated,
                Reverse(item.sequence),
            )
        })
        .map(|(index, _)| index)
}

pub fn normalize_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.starts_with("file://") {
        return Some(value.to_owned());
    }
    if Path::new(value).is_absolute() || value.starts_with("./") || value.starts_with("../") {
        return Some(format!("file://{value}"));
    }
    if value.starts_with("https://") || value.starts_with("http://") {
        Some(value.to_owned())
    } else {
        value
            .strip_prefix("//")
            .map(|rest| format!("https://{rest}"))
    }
}

pub fn bounded_target(target_size: [u32; 2]) -> [u32; 2] {
    let width = target_size[0].max(1);
    let height = target_size[1].max(1);
    let scale = (MAX_ARTWORK_WIDTH as f32 / width as f32)
        .min(MAX_ARTWORK_HEIGHT as f32 / height as f32)
        .min(1.0);
    [
        (width as f32 * scale).round().max(1.0) as u32,
        (height as f32 * scale).round().max(1.0) as u32,
    ]
}

/// Return the requested animation dimensions. The decoder adjusts this to
/// fit the animation's actual frame count, preserving every source frame.
pub fn animation_target_size(target_size: [u32; 2]) -> [u32; 2] {
    bounded_target(target_size)
}

/// Return a per-frame size that fits the animation atlas and byte budget.
pub fn animation_size_for_frame_count(
    target_size: [u32; 2],
    frame_count: usize,
) -> Option<[u32; 2]> {
    animation_size_for_frame_count_with_limits(
        target_size,
        frame_count,
        MAX_ANIMATION_BYTES,
        MAX_ANIMATION_ATLAS_SIDE,
    )
}

pub fn animation_size_for_frame_count_with_limits(
    target_size: [u32; 2],
    frame_count: usize,
    max_bytes: usize,
    max_atlas_side: u32,
) -> Option<[u32; 2]> {
    if frame_count == 0 {
        return Some(target_size);
    }
    if max_atlas_side == 0 || max_bytes == 0 {
        return None;
    }

    let mut scale = 1.0_f32;
    for _ in 0..96 {
        let candidate = [
            ((target_size[0] as f32 * scale).round() as u32).max(1),
            ((target_size[1] as f32 * scale).round() as u32).max(1),
        ];
        let frame_bytes = (candidate[0] as usize)
            .checked_mul(candidate[1] as usize)?
            .checked_mul(4)?;
        let frames_per_atlas =
            (max_atlas_side / candidate[0]) as usize * (max_atlas_side / candidate[1]) as usize;
        if frame_bytes.checked_mul(frame_count)? <= max_bytes && frames_per_atlas >= frame_count {
            return Some(candidate);
        }
        if candidate[0] == 1 && candidate[1] == 1 {
            break;
        }
        scale *= 0.95;
    }
    None
}

pub fn fetch_url(source_url: &str, target_size: [u32; 2]) -> String {
    if !source_url.contains("image.tmdb.org") {
        return source_url.to_owned();
    }
    let marker = "/t/p/";
    let Some(marker_start) = source_url.find(marker) else {
        return source_url.to_owned();
    };
    let path_start = marker_start + marker.len();
    let Some((_, path)) = source_url[path_start..].split_once('/') else {
        return source_url.to_owned();
    };
    let max_side = target_size[0].max(target_size[1]);
    let variant = if max_side <= 400 {
        "w342"
    } else if max_side <= 650 {
        "w500"
    } else if max_side <= 1000 {
        "w780"
    } else if max_side <= 1280 {
        "w1280"
    } else {
        // Hero surfaces can be wider than the largest named TMDB resize
        // variant. Keep the source resolution in that case and let the
        // bounded async decoder produce the exact GPU target size.
        "original"
    };
    format!("{}{variant}/{path}", &source_url[..path_start])
}

pub fn request_key(source_url: &str, target_size: [u32; 2]) -> String {
    format!("{}#{}x{}", source_url, target_size[0], target_size[1])
}

#[cfg(not(target_arch = "wasm32"))]
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| error.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn blocking<T>(work: impl FnOnce() -> T) -> Result<T, String> {
    Ok(work())
}

#[cfg(not(target_arch = "wasm32"))]
async fn retry_delay(delay: Duration) {
    tokio::time::sleep(delay).await;
}

#[cfg(target_arch = "wasm32")]
async fn retry_delay(_delay: Duration) {}

async fn fetch_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    if let Some(path) = url.strip_prefix("file://") {
        return std::fs::read(path).map_err(|error| error.to_string());
    }
    let mut last_error = String::from("unknown artwork error");
    for attempt in 0..3 {
        let response = match client
            .get(proxied(url).as_ref())
            .header(reqwest::header::ACCEPT_ENCODING, "identity")
            .send()
            .await
        {
            Ok(response) => match response.error_for_status() {
                Ok(response) => response,
                Err(error) => {
                    last_error = error.to_string();
                    continue;
                }
            },
            Err(error) => {
                last_error = error.to_string();
                if attempt < 2 {
                    retry_delay(Duration::from_millis(150 * (attempt + 1))).await;
                }
                continue;
            }
        };
        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES)
        {
            return Err("artwork response exceeds 16 MiB".to_owned());
        }
        let bytes = response.bytes().await.map_err(|error| error.to_string())?;
        if bytes.len() as u64 > MAX_RESPONSE_BYTES {
            return Err("artwork response exceeds 16 MiB".to_owned());
        }
        return Ok(bytes.to_vec());
    }
    Err(last_error)
}

fn decode_target(bytes: &[u8], target_size: [u32; 2]) -> Result<RgbaImage, String> {
    let without_bom = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    let trimmed = without_bom
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .map_or(without_bom, |index| &without_bom[index..]);
    if trimmed.starts_with(b"<svg") || trimmed.starts_with(b"<?xml") {
        let tree = resvg::usvg::Tree::from_data(trimmed, &resvg::usvg::Options::default())
            .map_err(|error| error.to_string())?;
        let size = tree.size();
        let scale = (target_size[0] as f32 / size.width().max(1.0))
            .min(target_size[1] as f32 / size.height().max(1.0))
            .min(1.0);
        let width = (size.width() * scale).round().max(1.0) as u32;
        let height = (size.height() * scale).round().max(1.0) as u32;
        let mut pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or_else(|| "could not allocate SVG pixmap".to_owned())?;
        resvg::render(
            &tree,
            tiny_skia::Transform::from_scale(scale, scale),
            &mut pixmap.as_mut(),
        );
        // tiny-skia already produces premultiplied RGBA. Keep that layout:
        // egui-wgpu uses `src_factor = One`, so handing it straight-alpha
        // pixels creates bright/dark fringes around transparent logos.
        return RgbaImage::from_raw(width, height, pixmap.data().to_vec())
            .ok_or_else(|| "SVG pixmap had an invalid RGBA buffer".to_owned());
    }
    let mut reader = ImageReader::new(Cursor::new(bytes));
    reader = reader
        .with_guessed_format()
        .map_err(|error| error.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DECODE_SIDE);
    limits.max_image_height = Some(MAX_DECODE_SIDE);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|error| error.to_string())?
        .to_rgba8();
    Ok(resize_rgba_preserving_alpha(image, target_size))
}

fn decode_animated_target(
    bytes: &[u8],
    target_size: [u32; 2],
) -> Result<Option<Vec<AnimatedFrame>>, String> {
    let format = image::guess_format(bytes).map_err(|error| error.to_string())?;
    match format {
        ImageFormat::Gif => {
            let mut decoder =
                GifDecoder::new(Cursor::new(bytes)).map_err(|error| error.to_string())?;
            let mut limits = image::Limits::default();
            limits.max_image_width = Some(MAX_DECODE_SIDE);
            limits.max_image_height = Some(MAX_DECODE_SIDE);
            limits.max_alloc = Some(128 * 1024 * 1024);
            decoder
                .set_limits(limits)
                .map_err(|error| error.to_string())?;
            collect_animation_frames(decoder.into_frames(), target_size, None).map(Some)
        }
        ImageFormat::WebP => {
            let decoder =
                WebPDecoder::new(Cursor::new(bytes)).map_err(|error| error.to_string())?;
            if !decoder.has_animation() {
                return Ok(None);
            }
            let timeline = webp_timeline(bytes);
            #[cfg(target_os = "linux")]
            if let Some(frames) =
                decode_webp_animation_with_system_libwebp(bytes, target_size, timeline)
            {
                return Ok(Some(frames));
            }
            let (width, height) = decoder.dimensions();
            if width > MAX_DECODE_SIDE || height > MAX_DECODE_SIDE {
                return Err(format!(
                    "animated WebP dimensions exceed {MAX_DECODE_SIDE}px"
                ));
            }
            collect_animation_frames(decoder.into_frames(), target_size, timeline).map(Some)
        }
        _ => Ok(None),
    }
}

#[cfg(target_os = "linux")]
fn decode_webp_animation_with_system_libwebp(
    bytes: &[u8],
    target_size: [u32; 2],
    timeline: Option<(usize, Duration)>,
) -> Option<Vec<AnimatedFrame>> {
    use libloading::Library;
    use std::ffi::c_void;

    #[repr(C)]
    struct WebPData {
        bytes: *const u8,
        size: usize,
    }

    #[repr(C)]
    struct DecoderOptions {
        color_mode: i32,
        use_threads: i32,
        padding: [u32; 7],
    }

    #[repr(C)]
    struct AnimationInfo {
        canvas_width: u32,
        canvas_height: u32,
        loop_count: u32,
        background_color: u32,
        frame_count: u32,
        padding: [u32; 4],
    }

    type OptionsInit = unsafe extern "C" fn(*mut DecoderOptions, i32) -> i32;
    type DecoderNew =
        unsafe extern "C" fn(*const WebPData, *const DecoderOptions, i32) -> *mut c_void;
    type GetInfo = unsafe extern "C" fn(*const c_void, *mut AnimationInfo) -> i32;
    type HasMore = unsafe extern "C" fn(*const c_void) -> i32;
    type GetNext = unsafe extern "C" fn(*mut c_void, *mut *mut u8, *mut i32) -> i32;
    type DecoderDelete = unsafe extern "C" fn(*mut c_void);

    // Keep the Library alive until the animation decoder has been deleted.
    let library =
        unsafe { Library::new("libwebpdemux.so.2").or_else(|_| Library::new("libwebpdemux.so")) }
            .ok()?;
    unsafe {
        let options_init: OptionsInit =
            *library.get(b"WebPAnimDecoderOptionsInitInternal\0").ok()?;
        let decoder_new: DecoderNew = *library.get(b"WebPAnimDecoderNewInternal\0").ok()?;
        let get_info: GetInfo = *library.get(b"WebPAnimDecoderGetInfo\0").ok()?;
        let has_more: HasMore = *library.get(b"WebPAnimDecoderHasMoreFrames\0").ok()?;
        let get_next: GetNext = *library.get(b"WebPAnimDecoderGetNext\0").ok()?;
        let decoder_delete: DecoderDelete = *library.get(b"WebPAnimDecoderDelete\0").ok()?;

        // WEBP_DEMUX_ABI_VERSION from libwebp's public demux API; MODE_RGBA.
        let mut options = DecoderOptions {
            color_mode: 1,
            use_threads: 1,
            padding: [0; 7],
        };
        if options_init(&mut options, 0x0107) == 0 {
            return None;
        }
        let data = WebPData {
            bytes: bytes.as_ptr(),
            size: bytes.len(),
        };
        let decoder = decoder_new(&data, &options, 0x0107);
        if decoder.is_null() {
            return None;
        }

        let decoded = (|| {
            let mut info = AnimationInfo {
                canvas_width: 0,
                canvas_height: 0,
                loop_count: 0,
                background_color: 0,
                frame_count: 0,
                padding: [0; 4],
            };
            if get_info(decoder, &mut info) == 0
                || info.frame_count < 2
                || info.frame_count as usize > MAX_SOURCE_ANIMATION_FRAMES
                || info.canvas_width == 0
                || info.canvas_height == 0
                || info.canvas_width > MAX_DECODE_SIDE
                || info.canvas_height > MAX_DECODE_SIDE
            {
                return None;
            }
            let mut sampler = FrameSampler::new(timeline.map(|(_, total)| total));
            let frame_size = animation_size_for_frame_count(
                target_size,
                sampler.frame_budget(info.frame_count as usize),
            )?;
            let source_bytes = (info.canvas_width as usize)
                .checked_mul(info.canvas_height as usize)?
                .checked_mul(4)?;
            let mut frames = Vec::with_capacity(MAX_ANIMATION_FRAMES);
            let mut previous_timestamp = 0_i32;
            let mut decoded = 0;
            while has_more(decoder) != 0 {
                let mut rgba = std::ptr::null_mut();
                let mut timestamp = 0_i32;
                if get_next(decoder, &mut rgba, &mut timestamp) == 0 || rgba.is_null() {
                    return None;
                }
                decoded += 1;
                let duration = Duration::from_millis(
                    timestamp
                        .saturating_sub(previous_timestamp)
                        .clamp(20, 10_000) as u64,
                );
                previous_timestamp = timestamp;
                if !sampler.keep(&mut frames, duration) {
                    continue;
                }
                let source = std::slice::from_raw_parts(rgba, source_bytes);
                let image =
                    RgbaImage::from_raw(info.canvas_width, info.canvas_height, source.to_vec())?;
                frames.push(AnimatedFrame {
                    image: resize_rgba_exact_with_filter(image, frame_size, FilterType::Triangle),
                    duration,
                });
            }
            (decoded == info.frame_count as usize).then_some(frames)
        })();
        decoder_delete(decoder);
        decoded
    }
}

struct FrameSampler {
    interval: Duration,
    since_kept: Duration,
}

impl FrameSampler {
    fn new(total: Option<Duration>) -> Self {
        let spread = total.map_or(Duration::ZERO, |total| total / MAX_ANIMATION_FRAMES as u32);
        Self {
            interval: spread.max(MIN_ANIMATION_FRAME_INTERVAL),
            since_kept: Duration::ZERO,
        }
    }

    fn frame_budget(&self, source_frames: usize) -> usize {
        source_frames.clamp(1, MAX_ANIMATION_FRAMES)
    }

    fn keep(&mut self, frames: &mut [AnimatedFrame], duration: Duration) -> bool {
        let keep = frames.is_empty()
            || (frames.len() < MAX_ANIMATION_FRAMES && self.since_kept >= self.interval);
        if keep {
            self.since_kept = duration;
        } else {
            self.since_kept += duration;
            if let Some(last) = frames.last_mut() {
                last.duration += duration;
            }
        }
        keep
    }
}

fn webp_timeline(bytes: &[u8]) -> Option<(usize, Duration)> {
    let animation = webp_animation::parse(bytes).ok().flatten()?;
    let total = animation
        .frames
        .iter()
        .map(|frame| Duration::from_millis(frame.duration_ms.clamp(20, 10_000) as u64))
        .sum();
    Some((animation.frames.len(), total))
}

fn collect_animation_frames(
    frames: impl Iterator<Item = image::ImageResult<Frame>>,
    target_size: [u32; 2],
    timeline: Option<(usize, Duration)>,
) -> Result<Vec<AnimatedFrame>, String> {
    let mut sampler = FrameSampler::new(timeline.map(|(_, total)| total));
    let fit = |count: usize| {
        animation_size_for_frame_count(target_size, count).ok_or_else(|| {
            "animation cannot fit its frames in the memory budget and texture atlas".to_owned()
        })
    };
    let mut frame_size = fit(sampler.frame_budget(timeline.map_or(1, |(count, _)| count)))?;
    let mut output: Vec<AnimatedFrame> = Vec::new();
    for (index, frame) in frames.enumerate() {
        if index >= MAX_SOURCE_ANIMATION_FRAMES {
            return Err(format!(
                "animation exceeds {MAX_SOURCE_ANIMATION_FRAMES} source frames"
            ));
        }
        let frame = frame.map_err(|error| error.to_string())?;
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        let delay_ms = if denominator == 0 {
            100
        } else {
            numerator / denominator
        }
        .clamp(20, 10_000);
        let duration = Duration::from_millis(delay_ms as u64);
        if !sampler.keep(&mut output, duration) {
            continue;
        }
        if timeline.is_none() {
            let size = fit(output.len() + 1)?;
            if size != frame_size {
                for retained in &mut output {
                    retained.image = resize_rgba_exact_with_filter(
                        std::mem::take(&mut retained.image),
                        size,
                        FilterType::Triangle,
                    );
                }
                frame_size = size;
            }
        }
        output.push(AnimatedFrame {
            image: resize_rgba_exact_with_filter(
                frame.into_buffer(),
                frame_size,
                FilterType::Triangle,
            ),
            duration,
        });
    }
    Ok(output)
}

// Sample inside the outermost pixel centers so linear filtering can't bleed into the next tile.
fn atlas_frame(
    x: u32,
    y: u32,
    size: [u32; 2],
    atlas: [u32; 2],
    duration: Duration,
) -> AnimatedAtlasFrame {
    AnimatedAtlasFrame {
        uv: [
            (x as f32 + 0.5) / atlas[0] as f32,
            (y as f32 + 0.5) / atlas[1] as f32,
            (x as f32 + size[0] as f32 - 0.5) / atlas[0] as f32,
            (y as f32 + size[1] as f32 - 0.5) / atlas[1] as f32,
        ],
        image_size: size,
        duration,
    }
}

fn transparent_ratio(image: &RgbaImage) -> f32 {
    let pixels = image.pixels().len().max(1);
    image.pixels().filter(|pixel| pixel.0[3] < 250).count() as f32 / pixels as f32
}

fn pack_animation_atlas(
    frames: Vec<AnimatedFrame>,
    max_side: u32,
) -> Result<Option<AnimatedAtlas>, String> {
    if frames.len() < 2 {
        return Ok(None);
    }
    let (frame_width, frame_height) = frames[0].image.dimensions();
    if frame_width == 0 || frame_height == 0 || frame_width > max_side || frame_height > max_side {
        return Err("animation frame does not fit in the atlas texture".to_owned());
    }
    if frames
        .iter()
        .any(|frame| frame.image.dimensions() != (frame_width, frame_height))
    {
        return Err("animation frames have inconsistent dimensions".to_owned());
    }

    let max_columns = (max_side / frame_width).max(1) as usize;
    let max_rows = (max_side / frame_height).max(1) as usize;
    if frames.len() > max_columns.saturating_mul(max_rows) {
        return Err("animation cannot fit all frames in the atlas texture".to_owned());
    }

    let columns = frames.len().min(max_columns);
    let rows = frames.len().div_ceil(columns);
    let atlas_width = frame_width * columns as u32;
    let atlas_height = frame_height * rows as u32;
    let mut atlas_image = RgbaImage::new(atlas_width, atlas_height);
    let mut atlas_frames = Vec::with_capacity(frames.len());

    for (index, frame) in frames.into_iter().enumerate() {
        let column = index % columns;
        let row = index / columns;
        let x = column as u32 * frame_width;
        let y = row as u32 * frame_height;
        image::imageops::replace(&mut atlas_image, &frame.image, x as i64, y as i64);

        atlas_frames.push(atlas_frame(
            x,
            y,
            [frame_width, frame_height],
            [atlas_width, atlas_height],
            frame.duration,
        ));
    }

    Ok(Some(AnimatedAtlas {
        image: atlas_image,
        frames: atlas_frames,
    }))
}

/// Versioned because older caches used straight-alpha upload/resampling.
/// Those rasters can contain halos around transparent logos and must not be
/// reused by the premultiplied-alpha renderer.
const CACHE_MAGIC: &[u8] = b"FXAR3";

fn resize_rgba_preserving_alpha(image: RgbaImage, target_size: [u32; 2]) -> RgbaImage {
    resize_rgba_preserving_alpha_with_filter(image, target_size, FilterType::Lanczos3)
}

fn resize_rgba_preserving_alpha_with_filter(
    image: RgbaImage,
    target_size: [u32; 2],
    filter: FilterType,
) -> RgbaImage {
    let (source_width, source_height) = image.dimensions();
    let scale = (target_size[0] as f32 / source_width.max(1) as f32)
        .min(target_size[1] as f32 / source_height.max(1) as f32)
        .min(1.0);
    let width = (source_width as f32 * scale).round().max(1.0) as u32;
    let height = (source_height as f32 * scale).round().max(1.0) as u32;
    // Resampling straight-alpha RGB mixes hidden transparent RGB values into
    // edge pixels. Premultiply first and keep the result premultiplied for
    // egui-wgpu's `src_factor = One` blending.
    let mut premultiplied = image;
    for pixel in premultiplied.pixels_mut() {
        let alpha = pixel[3] as u16;
        for channel in 0..3 {
            pixel[channel] = ((pixel[channel] as u16 * alpha + 127) / 255) as u8;
        }
    }
    if width == source_width && height == source_height {
        return premultiplied;
    }
    image::imageops::resize(&premultiplied, width, height, filter)
}

/// Resize an already-composited animation canvas to an exact tile size.
/// Unlike static artwork, every frame in an atlas must have identical
/// dimensions; aspect-preserving "fit inside" rounding can otherwise leave
/// adjacent frames one pixel apart after repeated budget-driven resizes.
fn resize_rgba_exact_with_filter(
    image: RgbaImage,
    target_size: [u32; 2],
    filter: FilterType,
) -> RgbaImage {
    let mut premultiplied = image;
    for pixel in premultiplied.pixels_mut() {
        let alpha = pixel[3] as u16;
        for channel in 0..3 {
            pixel[channel] = ((pixel[channel] as u16 * alpha + 127) / 255) as u8;
        }
    }
    if premultiplied.dimensions() == (target_size[0], target_size[1]) {
        return premultiplied;
    }
    image::imageops::resize(&premultiplied, target_size[0], target_size[1], filter)
}

fn cache_path(directory: &Path, key: &str) -> PathBuf {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    directory.join(format!("{:016x}.raster", hasher.finish()))
}

fn read_raster_cache(path: &Path) -> Option<RgbaImage> {
    let metadata = std::fs::metadata(path).ok()?;
    let fresh = metadata
        .modified()
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age <= MAX_CACHE_AGE);
    if !fresh || metadata.len() < 13 || metadata.len() > MAX_RESPONSE_BYTES {
        let _ = std::fs::remove_file(path);
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if !bytes.starts_with(CACHE_MAGIC) {
        let _ = std::fs::remove_file(path);
        return None;
    }
    let width = u32::from_le_bytes(bytes[5..9].try_into().ok()?);
    let height = u32::from_le_bytes(bytes[9..13].try_into().ok()?);
    let expected = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)?;
    if width == 0 || height == 0 || expected != bytes.len().saturating_sub(13) {
        let _ = std::fs::remove_file(path);
        return None;
    }
    let image = RgbaImage::from_raw(width, height, bytes[13..].to_vec())?;
    if let Ok(file) = std::fs::File::open(path) {
        let times = std::fs::FileTimes::new().set_modified(SystemTime::now());
        let _ = file.set_times(times);
    }
    Some(image)
}

fn write_raster_cache(path: &Path, image: &RgbaImage) {
    let Some(directory) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(directory).is_err() {
        return;
    }
    let mut bytes = Vec::with_capacity(13 + image.as_raw().len());
    bytes.extend_from_slice(CACHE_MAGIC);
    bytes.extend_from_slice(&image.width().to_le_bytes());
    bytes.extend_from_slice(&image.height().to_le_bytes());
    bytes.extend_from_slice(image.as_raw());
    if bytes.len() as u64 > MAX_RESPONSE_BYTES {
        return;
    }
    let temporary = path.with_extension("part");
    if std::fs::write(&temporary, bytes).is_ok() {
        if std::fs::rename(temporary, path).is_ok() {
            evict_raster_cache(directory);
        }
    }
}

fn read_animation_cache(path: &Path) -> Option<PreparedImage> {
    let metadata = std::fs::metadata(path).ok()?;
    let fresh = metadata
        .modified()
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age <= MAX_CACHE_AGE);
    if !fresh || metadata.len() > MAX_ANIMATION_BYTES as u64 + 64 * 1024 {
        let _ = std::fs::remove_file(path);
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let parsed = parse_animation_cache(&bytes);
    if parsed.is_none() {
        let _ = std::fs::remove_file(path);
        return None;
    }
    if let Ok(file) = std::fs::File::open(path) {
        let _ = file.set_times(std::fs::FileTimes::new().set_modified(SystemTime::now()));
    }
    parsed
}

fn parse_animation_cache(bytes: &[u8]) -> Option<PreparedImage> {
    let mut cursor = bytes.strip_prefix(ANIMATION_CACHE_MAGIC)?;
    let mut next = || -> Option<u32> {
        let (head, rest) = cursor.split_first_chunk::<4>()?;
        cursor = rest;
        Some(u32::from_le_bytes(*head))
    };
    let width = next()?;
    let height = next()?;
    let count = next()? as usize;
    if count < 2 || count > MAX_ANIMATION_FRAMES {
        return None;
    }
    let mut frames = Vec::with_capacity(count);
    for _ in 0..count {
        let x = next()?;
        let y = next()?;
        let frame_width = next()?;
        let frame_height = next()?;
        let duration = Duration::from_millis(next()? as u64);
        if frame_width == 0
            || frame_height == 0
            || x + frame_width > width
            || y + frame_height > height
        {
            return None;
        }
        frames.push((x, y, frame_width, frame_height, duration));
    }
    let atlas = RgbaImage::from_raw(width, height, cursor.to_vec())?;
    let (x, y, frame_width, frame_height, _) = frames[0];
    let image = image::imageops::crop_imm(&atlas, x, y, frame_width, frame_height).to_image();
    let frames = frames
        .into_iter()
        .map(|(x, y, frame_width, frame_height, duration)| {
            atlas_frame(x, y, [frame_width, frame_height], [width, height], duration)
        })
        .collect();
    Some(PreparedImage {
        image,
        animation_atlas: Some(AnimatedAtlas {
            image: atlas,
            frames,
        }),
    })
}

fn write_animation_cache(path: &Path, atlas: &AnimatedAtlas) {
    let Some(directory) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(directory).is_err() {
        return;
    }
    let (width, height) = atlas.image.dimensions();
    let mut bytes = Vec::with_capacity(17 + atlas.frames.len() * 20 + atlas.image.as_raw().len());
    bytes.extend_from_slice(ANIMATION_CACHE_MAGIC);
    for value in [width, height, atlas.frames.len() as u32] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for frame in &atlas.frames {
        let x = (frame.uv[0] * width as f32 - 0.5).round() as u32;
        let y = (frame.uv[1] * height as f32 - 0.5).round() as u32;
        let duration = frame.duration.as_millis().min(u32::MAX as u128) as u32;
        for value in [x, y, frame.image_size[0], frame.image_size[1], duration] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    bytes.extend_from_slice(atlas.image.as_raw());
    let temporary = path.with_extension("part");
    if std::fs::write(&temporary, bytes).is_ok() && std::fs::rename(temporary, path).is_ok() {
        evict_raster_cache(directory);
    }
}

/// Keep the persistent raster cache bounded. Modified time is refreshed by
/// `read_raster_cache`, so oldest-first deletion approximates an LRU without
/// maintaining a separate index that could become stale after a crash.
fn evict_raster_cache(directory: &Path) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut files = Vec::new();
    let mut total_bytes = 0_u64;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("raster") {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
        total_bytes = total_bytes.saturating_add(metadata.len());
        files.push((path, metadata.len(), modified));
    }
    for path in eviction_candidates(files, total_bytes, MAX_CACHE_BYTES) {
        let _ = std::fs::remove_file(path);
    }
}

fn eviction_candidates(
    mut files: Vec<(PathBuf, u64, SystemTime)>,
    total_bytes: u64,
    max_bytes: u64,
) -> Vec<PathBuf> {
    if total_bytes <= max_bytes {
        return Vec::new();
    }
    files.sort_by_key(|(_, _, modified)| *modified);
    let mut remaining = total_bytes;
    let mut paths = Vec::new();
    for (path, size, _) in files {
        if remaining <= max_bytes {
            break;
        }
        remaining = remaining.saturating_sub(size);
        paths.push(path);
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_clock_keeps_late_ready_artwork_on_the_shared_timeline() {
        let started_at = Instant::now();
        let frames = [
            AnimatedAtlasFrame {
                uv: [0.0, 0.0, 1.0, 1.0],
                image_size: [1, 1],
                duration: Duration::from_millis(100),
            },
            AnimatedAtlasFrame {
                uv: [0.0, 0.0, 1.0, 1.0],
                image_size: [1, 1],
                duration: Duration::from_millis(200),
            },
        ];

        let now = started_at + Duration::from_millis(175);
        let (frame_index, next_frame_at) =
            animation_frame_at(&frames, started_at, now).expect("non-empty animation");
        assert_eq!(frame_index, 1);
        assert_eq!(next_frame_at, started_at + Duration::from_millis(300));

        let now = started_at + Duration::from_millis(350);
        let (frame_index, next_frame_at) =
            animation_frame_at(&frames, started_at, now).expect("non-empty animation");
        assert_eq!(frame_index, 0);
        assert_eq!(next_frame_at, started_at + Duration::from_millis(400));
    }

    #[test]
    fn animation_target_scales_for_the_actual_frame_count() {
        let target = animation_target_size([324, 486]);
        assert_eq!(target, [324, 486]);

        let target = animation_size_for_frame_count([378, 214], 400)
            .expect("90 full-timeline frames should fit after scaling");
        assert!(target[0] < 378 && target[1] < 214);
        assert!((target[0] as usize * target[1] as usize * 4) * 400 <= MAX_ANIMATION_BYTES);
        assert!(
            (MAX_ANIMATION_ATLAS_SIDE / target[0]) as usize
                * (MAX_ANIMATION_ATLAS_SIDE / target[1]) as usize
                >= 400
        );
        assert_eq!(animation_target_size([120, 180]), [120, 180]);
    }

    #[test]
    fn artwork_targets_cap_at_1080p_without_changing_aspect_ratio() {
        assert_eq!(bounded_target([3840, 2160]), [1920, 1080]);
        assert_eq!(bounded_target([2160, 3840]), [608, 1080]);
        assert_eq!(bounded_target([640, 480]), [640, 480]);
        assert_eq!(bounded_target([0, 0]), [1, 1]);
    }

    #[test]
    fn larger_gpu_budget_preserves_display_size_for_long_visible_animations() {
        let target = [378, 214];
        let frame_count = 156;
        let frame_bytes = target[0] as usize * target[1] as usize * 4;
        assert!(frame_bytes * frame_count > 16 * 1024 * 1024);

        let size =
            animation_size_for_frame_count_with_limits(target, frame_count, 64 * 1024 * 1024, 4096)
                .expect("hardware atlas and budget can fit the displayed frames");

        assert_eq!(size, target);
    }

    #[test]
    fn gif_decoder_keeps_composited_frames_and_frame_delays() {
        let red = Frame::from_parts(
            RgbaImage::from_pixel(8, 8, image::Rgba([255, 0, 0, 255])),
            0,
            0,
            image::Delay::from_numer_denom_ms(80, 1),
        );
        let blue = Frame::from_parts(
            RgbaImage::from_pixel(8, 8, image::Rgba([0, 0, 255, 255])),
            0,
            0,
            image::Delay::from_numer_denom_ms(120, 1),
        );
        let mut bytes = Vec::new();
        image::codecs::gif::GifEncoder::new(&mut bytes)
            .encode_frames([red, blue])
            .expect("encode two-frame GIF fixture");

        let frames = decode_animated_target(&bytes, [96, 96])
            .expect("decode GIF fixture")
            .expect("GIF should be recognized as animated");
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].duration, Duration::from_millis(80));
        assert_eq!(frames[1].duration, Duration::from_millis(120));
        assert_ne!(frames[0].image, frames[1].image);
    }

    #[test]
    fn animation_atlas_normalizes_one_pixel_rounding_variance() {
        let source_frames = [(300, 200), (299, 200), (300, 199), (299, 199)]
            .into_iter()
            .enumerate()
            .map(|(index, (width, height))| {
                Ok::<_, image::ImageError>(Frame::from_parts(
                    RgbaImage::from_pixel(
                        width,
                        height,
                        image::Rgba([index as u8 * 40, 0, 0, 255]),
                    ),
                    0,
                    0,
                    image::Delay::from_numer_denom_ms(40, 1),
                ))
            });

        let decoded = collect_animation_frames(source_frames, [300, 200], None)
            .expect("decode variable-sized animation frames");
        assert!(
            decoded
                .iter()
                .all(|frame| frame.image.dimensions() == (300, 200))
        );
        assert!(
            pack_animation_atlas(decoded, 1024)
                .expect("pack normalized frames")
                .is_some()
        );
    }

    fn solid_frames(
        count: usize,
        side: u32,
        delay_ms: u32,
    ) -> impl Iterator<Item = image::ImageResult<Frame>> {
        (0..count).map(move |index| {
            Ok(Frame::from_parts(
                RgbaImage::from_pixel(side, side, image::Rgba([index as u8, 0, 0, 255])),
                0,
                0,
                image::Delay::from_numer_denom_ms(delay_ms, 1),
            ))
        })
    }

    #[test]
    fn fast_animations_drop_to_thirty_fps_without_shortening_the_loop() {
        let decoded = collect_animation_frames(solid_frames(120, 8, 20), [8, 8], None)
            .expect("decode fixture");

        assert_eq!(decoded.len(), 60);
        assert_eq!(
            decoded.iter().map(|frame| frame.duration).sum::<Duration>(),
            Duration::from_millis(120 * 20)
        );
        assert_eq!(decoded[1].image.get_pixel(0, 0)[0], 2);
    }

    #[test]
    fn long_animations_spread_the_frame_budget_over_the_whole_loop() {
        let timeline = Some((1000, Duration::from_millis(1000 * 40)));
        let decoded = collect_animation_frames(solid_frames(1000, 8, 40), [8, 8], timeline)
            .expect("decode fixture");

        assert!(decoded.len() <= MAX_ANIMATION_FRAMES);
        assert!(decoded.last().unwrap().image.get_pixel(0, 0)[0] as usize > 900 % 256 - 20);
        assert_eq!(
            decoded.iter().map(|frame| frame.duration).sum::<Duration>(),
            Duration::from_millis(1000 * 40)
        );
    }

    #[test]
    fn cached_animation_round_trips_its_atlas() {
        let decoded = collect_animation_frames(solid_frames(8, 8, 100), [8, 8], None)
            .expect("decode fixture");
        let atlas = pack_animation_atlas(decoded, 64).unwrap().unwrap();
        let directory =
            std::env::temp_dir().join(format!("fluxa-anim-cache-{}", std::process::id()));
        let path = cache_path(&directory, "fixture#animated");
        write_animation_cache(&path, &atlas);

        let cached = read_animation_cache(&path).expect("read cached animation");
        let _ = std::fs::remove_dir_all(&directory);
        let restored = cached.animation_atlas.unwrap();
        assert_eq!(restored.image, atlas.image);
        assert_eq!(restored.frames.len(), 8);
        assert_eq!(restored.frames[7].uv, atlas.frames[7].uv);
        assert_eq!(restored.frames[7].duration, Duration::from_millis(100));
        assert_eq!(cached.image.get_pixel(0, 0)[0], 0);
    }

    #[test]
    fn queued_visible_artwork_moves_ahead_of_background_prefetch() {
        let queued = [
            QueuedArtwork {
                key: "prefetch".to_owned(),
                source_url: String::new(),
                target_size: [1, 1],
                priority: Priority::Prefetch,
                sequence: 0,
                animated: false,
                animation_started_at: None,
            },
            QueuedArtwork {
                key: "visible-first".to_owned(),
                source_url: String::new(),
                target_size: [1, 1],
                priority: Priority::Visible,
                sequence: 1,
                animated: false,
                animation_started_at: None,
            },
            QueuedArtwork {
                key: "hero".to_owned(),
                source_url: String::new(),
                target_size: [1, 1],
                priority: Priority::Hero,
                sequence: 2,
                animated: false,
                animation_started_at: None,
            },
            QueuedArtwork {
                key: "visible-second".to_owned(),
                source_url: String::new(),
                target_size: [1, 1],
                priority: Priority::Visible,
                sequence: 3,
                animated: false,
                animation_started_at: None,
            },
            QueuedArtwork {
                key: "visible-motion".to_owned(),
                source_url: String::new(),
                target_size: [1, 1],
                priority: Priority::Visible,
                sequence: 4,
                animated: true,
                animation_started_at: Some(Instant::now()),
            },
        ];

        assert_eq!(next_queued_index(&queued), Some(2));
        assert_eq!(
            next_queued_index(&[
                queued[0].clone(),
                queued[1].clone(),
                queued[3].clone(),
                queued[4].clone(),
            ]),
            Some(3)
        );
    }

    #[test]
    fn requests_over_the_worker_limit_stay_queued_and_upgrade_priority() {
        let mut fetcher = ArtworkFetcher::new(None, 1);
        for index in 0..MAX_IN_FLIGHT {
            fetcher.in_flight.insert(format!("active-{index}"));
        }

        let key = fetcher
            .request(
                Some("https://artwork.example/poster.jpg"),
                [320, 480],
                Priority::Prefetch,
            )
            .expect("valid request should be accepted");
        assert_eq!(fetcher.queued.len(), 1);
        assert_eq!(fetcher.queued[0].priority, Priority::Prefetch);

        assert_eq!(
            fetcher.request(
                Some("https://artwork.example/poster.jpg"),
                [320, 480],
                Priority::Visible,
            ),
            Some(key)
        );
        assert_eq!(fetcher.queued.len(), 1);
        assert_eq!(fetcher.queued[0].priority, Priority::Visible);
    }

    #[test]
    fn disk_cache_evicts_oldest_rasters_until_under_budget() {
        let old = std::time::UNIX_EPOCH + Duration::from_secs(1);
        let recent = std::time::UNIX_EPOCH + Duration::from_secs(3);
        let newest = std::time::UNIX_EPOCH + Duration::from_secs(5);
        let candidates = eviction_candidates(
            vec![
                (PathBuf::from("old.raster"), 60, old),
                (PathBuf::from("recent.raster"), 30, recent),
                (PathBuf::from("newest.raster"), 40, newest),
            ],
            130,
            69,
        );

        assert_eq!(
            candidates,
            vec![PathBuf::from("old.raster"), PathBuf::from("recent.raster")]
        );
    }

    #[test]
    fn non_tmdb_sources_keep_their_url_but_get_a_target_cache_key() {
        let url = "https://provider.example/poster.webp";
        assert_eq!(fetch_url(url, [320, 180]), url);
        assert_ne!(request_key(url, [320, 180]), request_key(url, [640, 360]));
    }

    #[test]
    fn tmdb_source_uses_the_smallest_suitable_variant() {
        let url = "https://image.tmdb.org/t/p/original/poster.jpg";
        assert!(fetch_url(url, [320, 450]).contains("/t/p/w500/poster.jpg"));
        assert!(fetch_url(url, [1920, 1080]).contains("/t/p/original/poster.jpg"));
    }

    #[test]
    fn resized_rgba_is_premultiplied_at_transparent_edges() {
        let mut image = RgbaImage::new(2, 1);
        image.put_pixel(0, 0, image::Rgba([255, 255, 255, 0]));
        image.put_pixel(1, 0, image::Rgba([255, 0, 0, 255]));

        let resized = resize_rgba_preserving_alpha(image, [1, 1]);
        let pixel = resized.get_pixel(0, 0);
        assert!(pixel[0] <= pixel[3]);
        assert!(pixel[1] <= pixel[3]);
        assert!(pixel[2] <= pixel[3]);
    }
}

#[cfg(test)]
mod proxy_tests {
    #[test]
    fn remote_urls_are_percent_encoded_behind_the_proxy() {
        super::set_fetch_proxy("http://127.0.0.1:19876/");
        assert_eq!(
            super::proxied("https://img.example/a b?x=1"),
            "http://127.0.0.1:19876/proxy?url=https%3A%2F%2Fimg.example%2Fa%20b%3Fx%3D1"
        );
        assert_eq!(super::proxied("file:///tmp/a.png"), "file:///tmp/a.png");
    }
}
