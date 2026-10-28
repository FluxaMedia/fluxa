//! The single artwork pipeline used by every native Fluxa host.
//!
//! A provider may expose a smaller URL (TMDB currently does), but that is an
//! optional fetch optimization. The important invariant is that every source
//! is fetched asynchronously, decoded away from the UI thread, reduced to the
//! component's target size, and cached by source URL plus target size.

use image::{ImageReader, RgbaImage, imageops::FilterType};
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

pub mod stream;
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
const LOW_MEMORY_ARTWORK_WIDTH: u32 = 1280;
const LOW_MEMORY_ARTWORK_HEIGHT: u32 = 720;

static LOW_MEMORY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_low_memory(low: bool) {
    LOW_MEMORY.store(low, std::sync::atomic::Ordering::Relaxed);
}

pub fn low_memory() -> bool {
    LOW_MEMORY.load(std::sync::atomic::Ordering::Relaxed)
}
// A home feed can contain several shelves. Keep enough requests in flight to
// prepare the initial document before the user scrolls, while still bounding
// network and decode pressure. Decoding itself remains on the artwork worker
// runtime, never on the UI thread.
pub const MAX_IN_FLIGHT: usize = 32;
// Keep background warming from occupying every network/decode slot before
// newly visible artwork gets a chance to start.
const MAX_PREFETCH_IN_FLIGHT: usize = 16;
const MAX_PREFETCH_DECODE_IN_FLIGHT: usize = 3;
// Animated WebP/GIF composition is substantially more CPU- and memory-heavy
// than a poster raster decode. Keep a burst of newly-visible animations from
// occupying every decode worker while the user is scrolling.
const MAX_ANIMATION_DECODE_IN_FLIGHT: usize = 2;
pub const MAX_CACHE_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_CACHE_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const FAILED_REQUEST_RETRY_DELAY: Duration = Duration::from_secs(60);

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
    pub animation_source: Option<stream::MediaSource>,
    pub animation_requested: bool,
    pub transparent_ratio: f32,
    pub tones: [[u8; 3]; 2],
}

#[derive(Clone, Debug)]
struct PreparedImage {
    image: RgbaImage,
    animation: Option<stream::MediaSource>,
    warm_only: bool,
}

pub struct ArtworkFetcher {
    client: reqwest::Client,
    #[cfg(not(target_arch = "wasm32"))]
    runtime: tokio::runtime::Runtime,
    decode_slots: Arc<Semaphore>,
    hero_decode_slots: Arc<Semaphore>,
    animation_decode_slots: Arc<Semaphore>,
    prefetch_decode_slots: Arc<Semaphore>,
    sender: Sender<(
        String,
        [u32; 2],
        bool,
        Result<(PreparedImage, f32, [[u8; 3]; 2]), String>,
    )>,
    receiver: Receiver<(
        String,
        [u32; 2],
        bool,
        Result<(PreparedImage, f32, [[u8; 3]; 2]), String>,
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
}

impl ArtworkFetcher {
    pub fn new(cache_dir: Option<PathBuf>, worker_threads: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let decode_concurrency = worker_threads.clamp(2, 6);
        #[cfg(not(target_arch = "wasm32"))]
        let client = reqwest::Client::builder()
            .tls_certs_only(
                webpki_root_certs::TLS_SERVER_ROOT_CERTS
                    .iter()
                    .filter_map(|der| reqwest::Certificate::from_der(der).ok()),
            )
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
            .max_blocking_threads(decode_concurrency + 1)
            .enable_all()
            .thread_name("fluxa-artwork")
            .on_thread_start(lower_thread_priority)
            .build()
            .expect("Fluxa artwork runtime must start");
        Self {
            client,
            #[cfg(not(target_arch = "wasm32"))]
            runtime,
            decode_slots: Arc::new(Semaphore::new(decode_concurrency)),
            hero_decode_slots: Arc::new(Semaphore::new(1)),
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
        self.request_mode(source_url, target_size, priority, false)
    }

    pub fn request_animated(
        &mut self,
        source_url: Option<&str>,
        target_size: [u32; 2],
        priority: Priority,
    ) -> Option<String> {
        self.request_mode(source_url, target_size, priority, true)
    }

    pub fn prefetch_animated(
        &mut self,
        source_url: Option<&str>,
        target_size: [u32; 2],
    ) -> Option<String> {
        self.request_mode(source_url, target_size, Priority::Prefetch, true)
    }

    fn request_mode(
        &mut self,
        source_url: Option<&str>,
        target_size: [u32; 2],
        priority: Priority,
        animated: bool,
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
                queued.sequence = self.next_sequence;
                self.next_sequence = self.next_sequence.wrapping_add(1);
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
        let decode_slots = Arc::clone(if request.priority == Priority::Hero {
            &self.hero_decode_slots
        } else {
            &self.decode_slots
        });
        let animation_decode_slots = Arc::clone(&self.animation_decode_slots);
        let prefetch_decode_slots = Arc::clone(&self.prefetch_decode_slots);
        let key_for_task = request.key;
        let source_url = request.source_url;
        let target_size = request.target_size;
        let priority = request.priority;
        let animated = request.animated;
        self.spawn_task(async move {
            let cache_path = cache_dir
                .as_deref()
                .map(|directory| cache_path(directory, &key_for_task));
            let result = if animated {
                prepare_animation(
                    &client,
                    cache_path,
                    &source_url,
                    target_size,
                    priority,
                    decode_slots,
                    animation_decode_slots,
                    prefetch_decode_slots,
                )
                .await
            } else {
                prepare_raster(
                    &client,
                    cache_path,
                    &source_url,
                    target_size,
                    priority,
                    decode_slots,
                    animation_decode_slots,
                    prefetch_decode_slots,
                )
                .await
            };
            let result = result.map(|mut prepared| {
                if prepared.animation.is_none() && !prepared.warm_only {
                    trim_transparent_margins(&mut prepared.image);
                }
                let ratio = transparent_ratio(&prepared.image);
                let tones = edge_tones(&prepared.image);
                (prepared, ratio, tones)
            });
            let _ = sender.send((source_url, target_size, animated, result));
        });
    }

    pub fn poll(&mut self, limit: usize) -> Vec<PreparedArtwork> {
        let mut ready = Vec::new();
        for _ in 0..limit {
            let Ok((source_url, target_size, animated, result)) = self.receiver.try_recv() else {
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
                Ok((image, transparent_ratio, tones)) => {
                    self.failed.remove(&key);
                    if image.warm_only {
                        continue;
                    }
                    ready.push(PreparedArtwork {
                        source_url,
                        target_size,
                        image: image.image,
                        animation_source: image.animation,
                        transparent_ratio,
                        tones,
                        animation_requested: animated,
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

    pub fn request_is_backing_off(&self, source_url: &str, target_size: [u32; 2]) -> bool {
        let Some(source_url) = normalize_url(source_url) else {
            return false;
        };
        let key = request_key(&source_url, bounded_target(target_size));
        self.failed
            .get(&key)
            .is_some_and(|failed_at| failed_at.elapsed() < FAILED_REQUEST_RETRY_DELAY)
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

async fn prepare_raster(
    client: &reqwest::Client,
    cache_path: Option<PathBuf>,
    source_url: &str,
    target_size: [u32; 2],
    priority: Priority,
    decode_slots: Arc<Semaphore>,
    animation_decode_slots: Arc<Semaphore>,
    prefetch_decode_slots: Arc<Semaphore>,
) -> Result<PreparedImage, String> {
    let raster = |image| PreparedImage {
        image,
        animation: None,
        warm_only: false,
    };
    if let Some(path) = cache_path.clone() {
        let (decode_permit, prefetch_permit, _) = acquire_decode_permits(
            Arc::clone(&decode_slots),
            animation_decode_slots,
            Arc::clone(&prefetch_decode_slots),
            priority,
            false,
        )
        .await;
        let cached = blocking(move || {
            let _decode_permit = decode_permit;
            let _prefetch_permit = prefetch_permit;
            read_raster_cache(&path)
        })
        .await
        .ok()
        .flatten();
        if let Some(image) = cached {
            return Ok(raster(image));
        }
    }
    let bytes = fetch_bytes(client, &fetch_url(source_url, target_size)).await?;
    let (decode_permit, prefetch_permit, _) = acquire_decode_permits(
        decode_slots,
        Arc::new(Semaphore::new(1)),
        prefetch_decode_slots,
        priority,
        false,
    )
    .await;
    blocking(move || {
        let _decode_permit = decode_permit;
        let _prefetch_permit = prefetch_permit;
        let image = decode_target(&bytes, target_size)?;
        if let Some(path) = cache_path.as_deref() {
            write_raster_cache(path, &image);
        }
        Ok(raster(image))
    })
    .await
    .map_err(|error| error.to_string())
    .and_then(|result| result)
}

async fn prepare_animation(
    client: &reqwest::Client,
    cache_path: Option<PathBuf>,
    source_url: &str,
    target_size: [u32; 2],
    priority: Priority,
    decode_slots: Arc<Semaphore>,
    animation_decode_slots: Arc<Semaphore>,
    prefetch_decode_slots: Arc<Semaphore>,
) -> Result<PreparedImage, String> {
    if stream::is_video_url(source_url) {
        return Ok(PreparedImage {
            image: RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 255])),
            animation: (priority != Priority::Prefetch)
                .then(|| stream::MediaSource::Video(source_url.to_owned())),
            warm_only: priority == Priority::Prefetch,
        });
    }
    let cache_path = cache_path
        .filter(|_| !source_url.starts_with("file://"))
        .map(|path| path.with_extension("source"));
    let cached = match cache_path.clone() {
        Some(path) => blocking(move || read_source_cache(&path))
            .await
            .ok()
            .flatten(),
        None => None,
    };
    let bytes: Arc<[u8]> = match cached {
        Some(bytes) => bytes.into(),
        None => {
            let bytes = fetch_bytes(client, source_url).await?;
            if let Some(path) = cache_path {
                let copy = bytes.clone();
                let _ = blocking(move || write_source_cache(&path, &copy)).await;
            }
            bytes.into()
        }
    };
    if priority == Priority::Prefetch {
        return Ok(PreparedImage {
            image: RgbaImage::new(1, 1),
            animation: None,
            warm_only: true,
        });
    }
    let (decode_permit, prefetch_permit, animation_permit) = acquire_decode_permits(
        decode_slots,
        animation_decode_slots,
        prefetch_decode_slots,
        priority,
        true,
    )
    .await;
    blocking(move || {
        let _decode_permit = decode_permit;
        let _prefetch_permit = prefetch_permit;
        let _animation_permit = animation_permit;
        match stream::FrameStream::open(Arc::clone(&bytes), target_size) {
            Ok(Some(stream)) => Ok(PreparedImage {
                image: stream.poster().clone(),
                animation: Some(stream::MediaSource::Bytes(bytes)),
                warm_only: false,
            }),
            _ => Ok(PreparedImage {
                image: decode_target(&bytes, target_size)?,
                animation: None,
                warm_only: false,
            }),
        }
    })
    .await
    .map_err(|error| error.to_string())
    .and_then(|result| result)
}

fn read_source_cache(path: &Path) -> Option<Vec<u8>> {
    let metadata = std::fs::metadata(path).ok()?;
    let fresh = metadata
        .modified()
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age <= MAX_CACHE_AGE);
    if !fresh || metadata.len() > MAX_RESPONSE_BYTES {
        let _ = std::fs::remove_file(path);
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if let Ok(file) = std::fs::File::open(path) {
        let _ = file.set_times(std::fs::FileTimes::new().set_modified(SystemTime::now()));
    }
    Some(bytes)
}

fn write_source_cache(path: &Path, bytes: &[u8]) {
    let Some(directory) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(directory).is_err() {
        return;
    }
    let temporary = path.with_extension("part");
    if std::fs::write(&temporary, bytes).is_ok() && std::fs::rename(temporary, path).is_ok() {
        evict_raster_cache(directory);
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
                if item.priority == Priority::Prefetch {
                    Reverse(item.sequence)
                } else {
                    Reverse(u64::MAX - item.sequence)
                },
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
    let (max_width, max_height) = if low_memory() {
        (LOW_MEMORY_ARTWORK_WIDTH, LOW_MEMORY_ARTWORK_HEIGHT)
    } else {
        (MAX_ARTWORK_WIDTH, MAX_ARTWORK_HEIGHT)
    };
    let scale = (max_width as f32 / width as f32)
        .min(max_height as f32 / height as f32)
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

#[cfg(all(unix, not(target_arch = "wasm32")))]
pub fn lower_thread_priority() {
    unsafe {
        libc::setpriority(libc::PRIO_PROCESS, 0, 10);
    }
}

#[cfg(any(not(unix), target_arch = "wasm32"))]
pub fn lower_thread_priority() {}

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

pub type PlatformDecoder = fn(&[u8], [u32; 2]) -> Option<(u32, u32, Vec<u8>)>;

static PLATFORM_DECODER: std::sync::OnceLock<PlatformDecoder> = std::sync::OnceLock::new();

pub fn set_platform_decoder(decoder: PlatformDecoder) {
    let _ = PLATFORM_DECODER.set(decoder);
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
            .min(target_size[1] as f32 / size.height().max(1.0));
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
    if let Some(decoder) = PLATFORM_DECODER.get()
        && let Some((width, height, pixels)) = decoder(bytes, target_size)
        && let Some(image) = RgbaImage::from_raw(width, height, pixels)
    {
        return Ok(resize_premultiplied(image, target_size));
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

fn trim_transparent_margins(image: &mut RgbaImage) {
    let (width, height) = image.dimensions();
    let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0[3] > 8 {
            left = left.min(x);
            top = top.min(y);
            right = right.max(x + 1);
            bottom = bottom.max(y + 1);
        }
    }
    if right <= left || bottom <= top || (right - left == width && bottom - top == height) {
        return;
    }
    *image = image::imageops::crop_imm(image, left, top, right - left, bottom - top).to_image();
}

fn transparent_ratio(image: &RgbaImage) -> f32 {
    let pixels = image.pixels().len().max(1);
    image.pixels().filter(|pixel| pixel.0[3] < 250).count() as f32 / pixels as f32
}

fn edge_tones(image: &RgbaImage) -> [[u8; 3]; 2] {
    let (width, height) = image.dimensions();
    let average = |rows: std::ops::Range<u32>| {
        let mut sum = [0u64; 4];
        for y in rows.step_by(4) {
            for x in (0..width).step_by(4) {
                let [r, g, b, a] = image.get_pixel(x, y).0;
                sum[0] += r as u64;
                sum[1] += g as u64;
                sum[2] += b as u64;
                sum[3] += a as u64;
            }
        }
        let alpha = sum[3].max(1);
        [0, 1, 2].map(|channel| (sum[channel] * 255 / alpha).min(255) as u8)
    };
    [
        average(0..height / 4),
        average(height - height * 3 / 10..height),
    ]
}

const CACHE_MAGIC: &[u8] = b"FXAR4";

fn resize_rgba_preserving_alpha(image: RgbaImage, target_size: [u32; 2]) -> RgbaImage {
    resize_rgba_preserving_alpha_with_filter(image, target_size, FilterType::Lanczos3)
}

fn resize_rgba_preserving_alpha_with_filter(
    image: RgbaImage,
    target_size: [u32; 2],
    filter: FilterType,
) -> RgbaImage {
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
    resize_premultiplied_with_filter(premultiplied, target_size, filter)
}

fn resize_premultiplied(image: RgbaImage, target_size: [u32; 2]) -> RgbaImage {
    resize_premultiplied_with_filter(image, target_size, FilterType::Lanczos3)
}

fn resize_premultiplied_with_filter(
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
    if width == source_width && height == source_height {
        return image;
    }
    image::imageops::resize(&image, width, height, filter)
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
        if !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("raster" | "source")
        ) {
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
    fn transparent_margins_are_cropped_to_content() {
        let mut image = RgbaImage::new(20, 10);
        for x in 4..12 {
            image.put_pixel(x, 3, image::Rgba([255, 255, 255, 255]));
        }
        trim_transparent_margins(&mut image);
        assert_eq!(image.dimensions(), (8, 1));
    }

    #[test]
    fn artwork_targets_cap_at_1080p_without_changing_aspect_ratio() {
        assert_eq!(bounded_target([3840, 2160]), [1920, 1080]);
        assert_eq!(bounded_target([2160, 3840]), [608, 1080]);
        assert_eq!(bounded_target([640, 480]), [640, 480]);
        assert_eq!(bounded_target([0, 0]), [1, 1]);
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
            },
            QueuedArtwork {
                key: "visible-first".to_owned(),
                source_url: String::new(),
                target_size: [1, 1],
                priority: Priority::Visible,
                sequence: 1,
                animated: false,
            },
            QueuedArtwork {
                key: "hero".to_owned(),
                source_url: String::new(),
                target_size: [1, 1],
                priority: Priority::Hero,
                sequence: 2,
                animated: false,
            },
            QueuedArtwork {
                key: "visible-second".to_owned(),
                source_url: String::new(),
                target_size: [1, 1],
                priority: Priority::Visible,
                sequence: 3,
                animated: false,
            },
            QueuedArtwork {
                key: "visible-motion".to_owned(),
                source_url: String::new(),
                target_size: [1, 1],
                priority: Priority::Visible,
                sequence: 4,
                animated: true,
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
    fn most_recently_requested_visible_artwork_starts_first() {
        let item = |key: &str, sequence| QueuedArtwork {
            key: key.to_owned(),
            source_url: String::new(),
            target_size: [1, 1],
            priority: Priority::Visible,
            sequence,
            animated: false,
        };
        assert_eq!(
            next_queued_index(&[item("scrolled-past", 1), item("on-screen", 9)]),
            Some(1)
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
