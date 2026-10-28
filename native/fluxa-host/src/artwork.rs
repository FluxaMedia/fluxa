use super::*;

pub(super) const MAX_ARTWORK_SIDE: u32 = 1536;
// Android uses the same initial-document prefetch contract as desktop. This
// is still bounded, but avoids evicting visible cards while the remaining
// shelves finish decoding.
const MAX_CONVERTING: usize = 3;
const MAX_ARTWORK_TEXTURES: usize = 128;
const MAX_DECODED_BYTES: usize = 96 * 1024 * 1024;
const MAX_IDLE_ANIMATION_BYTES: usize = 96 * 1024 * 1024;
const LOW_MEMORY_ARTWORK_TEXTURES: usize = 48;
const LOW_MEMORY_DECODED_BYTES: usize = 32 * 1024 * 1024;
const LOW_MEMORY_TOTAL_KB: u64 = 3 * 1024 * 1024;

fn color_image(size: [usize; 2], raw: Vec<u8>) -> egui::ColorImage {
    egui::ColorImage::new(size, bytemuck::cast_slice(&raw).to_vec())
}

fn total_memory_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/meminfo")
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

struct ImageWeight;

impl clru::WeightScale<String, Arc<egui::ColorImage>> for ImageWeight {
    fn weight(&self, _key: &String, image: &Arc<egui::ColorImage>) -> usize {
        image.pixels.len() * 4
    }
}

type DecodedCache = clru::CLruCache<
    String,
    Arc<egui::ColorImage>,
    std::collections::hash_map::RandomState,
    ImageWeight,
>;

struct ConvertedArtwork {
    source_url: String,
    target_size: [u32; 2],
    animation_requested: bool,
    animation_source: Option<fluxa_artwork::stream::MediaSource>,
    width: u32,
    height: u32,
    image: egui::ColorImage,
    transparent_ratio: f32,
    tones: [[u8; 3]; 2],
}

pub(super) struct ArtworkLoader {
    converted_tx: std::sync::mpsc::Sender<ConvertedArtwork>,
    converted_rx: std::sync::mpsc::Receiver<ConvertedArtwork>,
    converting: Arc<std::sync::atomic::AtomicUsize>,
    pub(super) fetcher: ArtworkFetcher,
    textures: HashMap<String, egui::TextureHandle>,
    emoji: HashMap<String, Option<egui::TextureHandle>>,
    emoji_pending: Vec<String>,
    animations: HashMap<String, ArtworkAnimation>,
    animation_checked: HashSet<String>,
    active_animations: HashSet<String>,
    animation_slots: HashSet<String>,
    animation_inflight: HashSet<String>,
    latest_keys: HashMap<String, String>,
    texture_last_used: HashMap<String, u64>,
    texture_frame: HashMap<String, u64>,
    prefetch_keys: HashSet<String>,
    decoded: DecodedCache,
    context: Option<egui::Context>,
    frame: u64,
    transparent_pixel_ratio: HashMap<String, f32>,
    pub(super) tones: HashMap<String, [[u8; 3]; 2]>,
    access_counter: u64,
    disabled: bool,
}

pub(super) struct ArtworkAnimation {
    source: fluxa_artwork::stream::MediaSource,
    target_size: [u32; 2],
    size: [u32; 2],
    stream: Option<AnimationStream>,
}

struct AnimationStream {
    frames: std::sync::mpsc::Receiver<fluxa_artwork::stream::StreamFrame>,
    held: Option<fluxa_artwork::stream::StreamFrame>,
    next_frame_at: Instant,
}

#[cfg(not(target_arch = "wasm32"))]
struct VideoClip(fluxa_mpv::MpvClip);

#[cfg(not(target_arch = "wasm32"))]
impl VideoClip {
    fn open(
        url: &str,
        target_size: [u32; 2],
    ) -> Result<fluxa_artwork::stream::FrameStream, String> {
        let clip = fluxa_mpv::MpvClip::open(url, target_size)?;
        Ok(fluxa_artwork::stream::FrameStream::from_decoder(
            Box::new(Self(clip)),
            target_size,
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl fluxa_artwork::stream::FrameDecoder for VideoClip {
    fn next(&mut self) -> Result<Option<fluxa_artwork::stream::StreamFrame>, String> {
        let frame = self.0.next_frame()?;
        let image = image::RgbaImage::from_raw(frame.size[0], frame.size[1], frame.rgba)
            .ok_or_else(|| "video frame had an invalid size".to_owned())?;
        Ok(Some(fluxa_artwork::stream::StreamFrame {
            image,
            duration: frame.duration,
        }))
    }

    fn rewind(&mut self) -> Result<(), String> {
        Ok(())
    }
}

impl AnimationStream {
    #[cfg(not(target_arch = "wasm32"))]
    fn spawn(source: fluxa_artwork::stream::MediaSource, target_size: [u32; 2]) -> Self {
        let (sender, frames) = std::sync::mpsc::sync_channel(2);
        let _ = std::thread::Builder::new()
            .name("fluxa-animation".into())
            .spawn(move || {
                fluxa_artwork::lower_thread_priority();
                let opened = match source {
                    fluxa_artwork::stream::MediaSource::Bytes(bytes) => {
                        fluxa_artwork::stream::FrameStream::open(bytes, target_size)
                    }
                    fluxa_artwork::stream::MediaSource::Video(url) => {
                        VideoClip::open(&url, target_size).map(Some)
                    }
                };
                let Ok(Some(mut stream)) = opened else {
                    return;
                };
                while let Ok(frame) = stream.next_frame() {
                    if sender.send(frame).is_err() {
                        break;
                    }
                }
            });
        Self {
            frames,
            held: None,
            next_frame_at: Instant::now(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn(_source: fluxa_artwork::stream::MediaSource, _target_size: [u32; 2]) -> Self {
        let (_, frames) = std::sync::mpsc::sync_channel(1);
        Self {
            frames,
            held: None,
            next_frame_at: Instant::now(),
        }
    }
}

impl ArtworkLoader {
    pub(super) fn new(cache_dir: Option<PathBuf>) -> Self {
        let low_memory = total_memory_kb().is_some_and(|total| total < LOW_MEMORY_TOTAL_KB);
        fluxa_artwork::set_low_memory(low_memory);
        let workers = std::thread::available_parallelism()
            .map_or(2, |cores| cores.get() / 2)
            .clamp(2, 4);
        let (converted_tx, converted_rx) = std::sync::mpsc::channel();
        Self {
            fetcher: ArtworkFetcher::new(cache_dir, workers),
            textures: HashMap::new(),
            emoji: HashMap::new(),
            emoji_pending: Vec::new(),
            animations: HashMap::new(),
            animation_checked: HashSet::new(),
            converted_tx,
            converted_rx,
            converting: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            active_animations: HashSet::new(),
            animation_slots: HashSet::new(),
            animation_inflight: HashSet::new(),
            latest_keys: HashMap::new(),
            texture_last_used: HashMap::new(),
            texture_frame: HashMap::new(),
            prefetch_keys: HashSet::new(),
            decoded: DecodedCache::with_config(
                clru::CLruCacheConfig::new(
                    std::num::NonZeroUsize::new(if low_memory {
                        LOW_MEMORY_DECODED_BYTES
                    } else {
                        MAX_DECODED_BYTES
                    })
                    .unwrap(),
                )
                .with_scale(ImageWeight),
            ),
            context: None,
            frame: 0,
            transparent_pixel_ratio: HashMap::new(),
            tones: HashMap::new(),
            access_counter: 0,
            disabled: std::env::var_os("FLUXA_NATIVE_DISABLE_ARTWORK").is_some(),
        }
    }

    pub(super) fn emoji(&mut self, cluster: &str) -> Option<egui::TextureId> {
        match self.emoji.get(cluster) {
            Some(texture) => texture.as_ref().map(|texture| texture.id()),
            None => {
                if !self.emoji_pending.iter().any(|pending| pending == cluster) {
                    self.emoji_pending.push(cluster.to_owned());
                }
                None
            }
        }
    }

    fn rasterize_emoji(&mut self, context: &egui::Context) {
        let Some(rasterize) = crate::EMOJI_RASTERIZER.get() else {
            return;
        };
        let pending = std::mem::take(&mut self.emoji_pending);
        if !pending.is_empty() {
            context.request_repaint();
        }
        for cluster in pending {
            let texture = rasterize(&cluster, 96).map(|(width, height, rgba)| {
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &rgba,
                );
                context.load_texture(
                    format!("emoji:{cluster}"),
                    image,
                    egui::TextureOptions::LINEAR,
                )
            });
            self.emoji.insert(cluster, texture);
        }
    }

    pub(super) fn poll(&mut self, context: &egui::Context) {
        self.context.get_or_insert_with(|| context.clone());
        self.rasterize_emoji(context);
        let uploads = if fluxa_artwork::low_memory() { 1 } else { 2 };
        let room = MAX_CONVERTING
            .saturating_sub(self.converting.load(std::sync::atomic::Ordering::Relaxed));
        for mut prepared in self.fetcher.poll(room) {
            let animation_source = prepared.animation_source.take();
            let (width, height) = (prepared.image.width(), prepared.image.height());
            let raw =
                std::mem::replace(&mut prepared.image, image::RgbaImage::new(0, 0)).into_raw();
            let sender = self.converted_tx.clone();
            self.converting
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let job = move || {
                fluxa_artwork::lower_thread_priority();
                let image = color_image([width as usize, height as usize], raw);
                let _ = sender.send(ConvertedArtwork {
                    source_url: prepared.source_url,
                    target_size: prepared.target_size,
                    animation_requested: prepared.animation_requested,
                    animation_source,
                    width,
                    height,
                    image,
                    transparent_ratio: prepared.transparent_ratio,
                    tones: prepared.tones,
                });
            };
            #[cfg(not(target_arch = "wasm32"))]
            let _ = std::thread::Builder::new()
                .name("fluxa-convert".into())
                .spawn(job);
            #[cfg(target_arch = "wasm32")]
            job();
        }
        let ready = self
            .converted_rx
            .try_iter()
            .take(uploads)
            .collect::<Vec<_>>();
        for converted in ready {
            self.converting
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            let url = converted.source_url;
            let target_size = converted.target_size;
            let animation_requested = converted.animation_requested;
            let (width, height) = (converted.width, converted.height);
            let base_key = fluxa_artwork::request_key(&url, target_size);
            if animation_requested {
                self.animation_inflight
                    .remove(&format!("{base_key}#animated"));
            }
            let animated = converted.animation_source.is_some();
            let key = if animated {
                format!("{base_key}#animated")
            } else {
                base_key.clone()
            };
            if animation_requested && !animated {
                self.animation_checked
                    .insert(format!("{base_key}#animated"));
            }
            // Artwork is premultiplied before upload. Passing it through the
            // straight-alpha constructor applies alpha a second time and
            // creates the dark/bright fringe visible around transparent logos.
            let max_side = context.input(|input| input.max_texture_side) as u32;
            if width > max_side || height > max_side {
                host_log(format!(
                    "Artwork skipped: {width}x{height} exceeds {max_side}"
                ));
                continue;
            }
            let transparent_ratio = converted.transparent_ratio;
            let tones = converted.tones;
            let image = Arc::new(converted.image);
            if !animated {
                self.transparent_pixel_ratio
                    .insert(key.clone(), transparent_ratio);
                self.tones.insert(key.clone(), tones);
                let _ = self.decoded.put_with_weight(key.clone(), image.clone());
                if self.prefetch_keys.remove(&key) {
                    continue;
                }
            }
            let texture = context.load_texture(
                format!("fluxa-native-artwork-{key}"),
                egui::ImageData::Color(image),
                egui::TextureOptions::LINEAR,
            );
            if !animated {
                self.latest_keys.insert(url, key.clone());
            } else {
                self.animation_checked.insert(key.clone());
            }
            self.access_counter = self.access_counter.wrapping_add(1);
            self.texture_last_used
                .insert(key.clone(), self.access_counter);
            self.textures.insert(key.clone(), texture);
            if let Some(source) = converted.animation_source {
                if fluxa_artwork::low_memory() {
                    let stale = self
                        .animations
                        .keys()
                        .filter(|other| **other != key)
                        .cloned()
                        .collect::<Vec<_>>();
                    for other in stale {
                        self.textures.remove(&other);
                        self.texture_last_used.remove(&other);
                        self.texture_frame.remove(&other);
                        self.animations.remove(&other);
                        self.animation_checked.remove(&other);
                    }
                }
                self.animations.insert(
                    key,
                    ArtworkAnimation {
                        source,
                        target_size,
                        size: [width, height],
                        stream: None,
                    },
                );
            }
            let texture_cap = if fluxa_artwork::low_memory() {
                LOW_MEMORY_ARTWORK_TEXTURES
            } else {
                MAX_ARTWORK_TEXTURES
            };
            while self.textures.len() > texture_cap {
                let frame = self.frame;
                let texture_frame = &self.texture_frame;
                let Some(oldest) = self
                    .texture_last_used
                    .iter()
                    .filter(|(key, _)| texture_frame.get(*key).is_none_or(|used| used + 1 < frame))
                    .min_by_key(|(_, last_used)| *last_used)
                    .map(|(key, _)| key.clone())
                else {
                    break;
                };
                self.textures.remove(&oldest);
                self.texture_last_used.remove(&oldest);
                self.texture_frame.remove(&oldest);
                self.animations.remove(&oldest);
                self.animation_checked.remove(&oldest);
            }
            self.evict_idle_animations();
            let (static_bytes, animation_bytes) = self.resident_bytes();
            if animated {
                host_log(format!(
                    "Artwork resident: static {:.1}MB in {} textures, animations {:.1}MB in {}",
                    static_bytes as f64 / 1048576.0,
                    self.textures.len() - self.animations.len(),
                    animation_bytes as f64 / 1048576.0,
                    self.animations.len(),
                ));
            }
            context.request_repaint();
        }
        let now = Instant::now();
        let active = std::mem::take(&mut self.active_animations);
        for (key, animation) in &mut self.animations {
            if !active.contains(key) {
                animation.stream = None;
            }
        }
        let mut next_frame_in = None;
        for key in &active {
            let Some(animation) = self.animations.get_mut(key) else {
                continue;
            };
            let Some(stream) = animation.stream.as_mut() else {
                continue;
            };
            if now >= stream.next_frame_at {
                if stream.held.is_none() {
                    stream.held = stream.frames.try_recv().ok();
                }
                if let Some(frame) = stream.held.take() {
                    let behind = now.saturating_duration_since(stream.next_frame_at);
                    let base = if behind > frame.duration * 2 {
                        now
                    } else {
                        stream.next_frame_at
                    };
                    stream.next_frame_at = base + frame.duration;
                    if let Some(texture) = self.textures.get_mut(key) {
                        let (width, height) = (frame.image.width(), frame.image.height());
                        animation.size = [width, height];
                        let image =
                            color_image([width as usize, height as usize], frame.image.into_raw());
                        texture.set(
                            egui::ImageData::Color(Arc::new(image)),
                            egui::TextureOptions::LINEAR,
                        );
                    }
                    context.request_repaint();
                }
            }
            let wait = if now >= stream.next_frame_at {
                Duration::from_millis(4)
            } else {
                stream.next_frame_at - now
            };
            next_frame_in = Some(next_frame_in.map_or(wait, |current: Duration| current.min(wait)));
        }
        self.active_animations = active;
        if let Some(wait) = next_frame_in {
            context.request_repaint_after(wait.max(Duration::from_millis(1)));
        }
    }

    pub(super) fn resident_bytes(&self) -> (usize, usize) {
        let mut totals = (0, 0);
        for (key, texture) in &self.textures {
            let [width, height] = texture.size();
            let bytes = width * height * 4;
            if self.animations.contains_key(key) {
                totals.1 += bytes;
            } else {
                totals.0 += bytes;
            }
        }
        totals
    }

    fn evict_idle_animations(&mut self) {
        let budget = if fluxa_artwork::low_memory() {
            MAX_IDLE_ANIMATION_BYTES / 4
        } else {
            MAX_IDLE_ANIMATION_BYTES
        };
        while self.resident_bytes().1 > budget {
            let frame = self.frame;
            let Some(oldest) = self
                .animations
                .keys()
                .filter(|key| {
                    !self.active_animations.contains(*key)
                        && self
                            .texture_frame
                            .get(*key)
                            .is_none_or(|used| used + 1 < frame)
                })
                .min_by_key(|key| self.texture_last_used.get(*key).copied().unwrap_or(0))
                .cloned()
            else {
                break;
            };
            self.textures.remove(&oldest);
            self.texture_last_used.remove(&oldest);
            self.texture_frame.remove(&oldest);
            self.animations.remove(&oldest);
            self.animation_checked.remove(&oldest);
        }
    }

    pub(super) fn converting(&self) -> bool {
        self.converting.load(std::sync::atomic::Ordering::Relaxed) > 0
    }

    pub(super) fn next_animation_frame(&self) -> Option<Instant> {
        self.active_animations
            .iter()
            .filter_map(|key| self.animations.get(key)?.stream.as_ref())
            .map(|stream| stream.next_frame_at)
            .min()
    }

    pub(super) fn begin_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.active_animations.clear();
        self.animation_slots.clear();
    }

    pub(super) fn animated_texture_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkFetchPriority,
    ) -> Option<AnimatedTexture> {
        if self.disabled {
            return None;
        }
        let target_size = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_ARTWORK_SIDE),
            target_size[1].min(MAX_ARTWORK_SIDE),
        ]);
        let source_url = fluxa_artwork::normalize_url(url?)?;
        let base_key = fluxa_artwork::request_key(&source_url, target_size);
        let key = format!("{base_key}#animated");
        if self.animation_checked.contains(&key) && !self.animations.contains_key(&key) {
            return None;
        }
        if !self.animation_slots.contains(&key) {
            if self
                .fetcher
                .animated_request_is_backing_off(&source_url, target_size)
            {
                self.animation_inflight.remove(&key);
                return None;
            }
            self.animation_slots.insert(key.clone());
        }
        self.access_counter = self.access_counter.wrapping_add(1);
        if self.textures.contains_key(&key) {
            self.texture_last_used
                .insert(key.clone(), self.access_counter);
            self.texture_frame.insert(key.clone(), self.frame);
        }
        if let Some(animation) = self.animations.get_mut(&key) {
            if animation.stream.is_none() {
                animation.stream = Some(AnimationStream::spawn(
                    animation.source.clone(),
                    animation.target_size,
                ));
            }
            let size = animation.size;
            let texture = self.textures.get(&key)?;
            self.active_animations.insert(key.clone());
            return Some(AnimatedTexture {
                texture: texture.id(),
                uv: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                image_size: size,
            });
        }
        if self.animation_checked.contains(&key) || !self.animation_inflight.insert(key) {
            return None;
        }
        let _ = self
            .fetcher
            .request_animated(Some(&source_url), target_size, priority);
        None
    }

    pub(super) fn media_texture_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkFetchPriority,
    ) -> Option<AnimatedTexture> {
        if let Some(animated) = self.animated_texture_for_priority(url, target_size, priority) {
            return Some(animated);
        }
        let source_url = fluxa_artwork::normalize_url(url?)?;
        let bounded = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_ARTWORK_SIDE),
            target_size[1].min(MAX_ARTWORK_SIDE),
        ]);
        let key = format!(
            "{}#animated",
            fluxa_artwork::request_key(&source_url, bounded)
        );
        if !self.animation_checked.contains(&key) {
            return None;
        }
        let texture = self.texture_for_priority(Some(&source_url), target_size, priority)?;
        Some(AnimatedTexture {
            texture,
            uv: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            image_size: self.size(Some(&source_url))?,
        })
    }

    pub(super) fn prefetch_animated_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
    ) {
        if self.disabled || fluxa_artwork::low_memory() {
            return;
        }
        let target_size = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_ARTWORK_SIDE),
            target_size[1].min(MAX_ARTWORK_SIDE),
        ]);
        let Some(source_url) = url.and_then(fluxa_artwork::normalize_url) else {
            return;
        };
        let key = format!(
            "{}#animated",
            fluxa_artwork::request_key(&source_url, target_size)
        );
        if self.textures.contains_key(&key)
            || self
                .fetcher
                .animated_request_is_backing_off(&source_url, target_size)
        {
            return;
        }
        let _ = self
            .fetcher
            .prefetch_animated(Some(&source_url), target_size);
    }

    fn upload(
        &mut self,
        context: &egui::Context,
        url: String,
        key: String,
        image: Arc<egui::ColorImage>,
    ) -> egui::TextureId {
        let texture = context.load_texture(
            format!("fluxa-native-artwork-{key}"),
            egui::ImageData::Color(image),
            egui::TextureOptions::LINEAR,
        );
        let id = texture.id();
        self.latest_keys.insert(url, key.clone());
        self.texture_last_used
            .insert(key.clone(), self.access_counter);
        self.texture_frame.insert(key.clone(), self.frame);
        self.textures.insert(key, texture);
        id
    }

    pub(super) fn texture(&mut self, url: Option<&str>) -> Option<egui::TextureId> {
        self.texture_for_priority(url, [MAX_ARTWORK_SIDE; 2], ArtworkFetchPriority::Visible)
    }

    pub(super) fn texture_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkFetchPriority,
    ) -> Option<egui::TextureId> {
        if self.disabled {
            return None;
        }
        let target_size = fluxa_artwork::bounded_target([
            target_size[0].min(MAX_ARTWORK_SIDE),
            target_size[1].min(MAX_ARTWORK_SIDE),
        ]);
        let url = fluxa_artwork::normalize_url(url?)?;
        let key = fluxa_artwork::request_key(&url, target_size);
        self.access_counter = self.access_counter.wrapping_add(1);
        if let Some(texture) = self.textures.get(&key) {
            if priority != ArtworkFetchPriority::Prefetch {
                self.texture_last_used
                    .insert(key.clone(), self.access_counter);
                self.texture_frame.insert(key, self.frame);
            }
            return Some(texture.id());
        }
        if priority == ArtworkFetchPriority::Prefetch {
            if self.decoded.contains(&key) {
                return None;
            }
            self.prefetch_keys.insert(key);
        } else {
            self.prefetch_keys.remove(&key);
            if let Some(image) = self.decoded.get(&key).cloned()
                && let Some(context) = self.context.clone()
            {
                return Some(self.upload(&context, url, key, image));
            }
        }
        let _ = self.fetcher.request(Some(&url), target_size, priority);
        None
    }

    pub(super) fn failed(&self, url: &str, target_size: [u32; 2]) -> bool {
        self.fetcher.request_is_backing_off(
            url,
            [
                target_size[0].min(MAX_ARTWORK_SIDE),
                target_size[1].min(MAX_ARTWORK_SIDE),
            ],
        )
    }

    pub(super) fn size(&self, url: Option<&str>) -> Option<[u32; 2]> {
        let url = fluxa_artwork::normalize_url(url?)?;
        self.latest_keys
            .get(&url)
            .and_then(|key| self.textures.get(key))
            .map(|texture| texture.size().map(|side| side as u32))
    }

    pub(super) fn tones(&self, url: Option<&str>) -> Option<[[u8; 3]; 2]> {
        let url = fluxa_artwork::normalize_url(url?)?;
        self.latest_keys
            .get(&url)
            .and_then(|key| self.tones.get(key))
            .copied()
    }

    pub(super) fn cached_texture(&self, url: Option<&str>) -> Option<egui::TextureId> {
        let url = fluxa_artwork::normalize_url(url?)?;
        self.latest_keys
            .get(&url)
            .and_then(|key| self.textures.get(key))
            .map(egui::TextureHandle::id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Delay, Frame, Rgba, RgbaImage};

    fn write_gif(path: &std::path::Path) {
        let frames = [255u8, 0, 128].map(|red| {
            Frame::from_parts(
                RgbaImage::from_pixel(16, 16, Rgba([red, 0, 0, 255])),
                0,
                0,
                Delay::from_numer_denom_ms(40, 1),
            )
        });
        let mut bytes = Vec::new();
        image::codecs::gif::GifEncoder::new(&mut bytes)
            .encode_frames(frames)
            .unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn animated_background_streams_into_one_texture() {
        let directory = std::env::temp_dir().join(format!("fluxa-media-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("loop.gif");
        write_gif(&path);
        let url = path.to_string_lossy().into_owned();

        let context = egui::Context::default();
        let mut loader = ArtworkLoader::new(None);
        let started = Instant::now();
        let mut first = None;
        let mut last = None;
        let mut advanced = 0;
        while started.elapsed() < Duration::from_secs(5) && advanced < 4 {
            loader.poll(&context);
            loader.begin_frame();
            if let Some(media) =
                loader.media_texture_for_priority(Some(&url), [64, 64], ArtworkFetchPriority::Hero)
            {
                assert_eq!(media.image_size, [16, 16]);
                first.get_or_insert(media.texture);
                assert_eq!(first, Some(media.texture));
                let next = loader.next_animation_frame();
                if next.is_some() && next != last {
                    advanced += 1;
                }
                last = next;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = std::fs::remove_dir_all(&directory);
        assert!(advanced >= 4, "animation never advanced");
    }

    #[test]
    fn static_image_falls_back_to_a_plain_texture() {
        let directory = std::env::temp_dir().join(format!("fluxa-still-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("still.png");
        RgbaImage::from_pixel(16, 16, Rgba([9, 9, 9, 255]))
            .save(&path)
            .unwrap();
        let url = path.to_string_lossy().into_owned();

        let context = egui::Context::default();
        let mut loader = ArtworkLoader::new(None);
        let started = Instant::now();
        let mut media = None;
        while started.elapsed() < Duration::from_secs(5) && media.is_none() {
            loader.poll(&context);
            loader.begin_frame();
            media =
                loader.media_texture_for_priority(Some(&url), [64, 64], ArtworkFetchPriority::Hero);
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = std::fs::remove_dir_all(&directory);
        assert!(media.is_some());
        assert!(loader.next_animation_frame().is_none());
    }

    #[test]
    fn video_background_plays_through_mpv() {
        let directory = std::env::temp_dir().join(format!("fluxa-video-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("clip.mp4");
        let made = std::process::Command::new("ffmpeg")
            .args([
                "-v",
                "quiet",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=160x90:rate=24",
            ])
            .args(["-t", "2", "-pix_fmt", "yuv420p"])
            .arg(&path)
            .status();
        if !made.is_ok_and(|status| status.success()) {
            let _ = std::fs::remove_dir_all(&directory);
            return;
        }
        let url = path.to_string_lossy().into_owned();

        let context = egui::Context::default();
        let mut loader = ArtworkLoader::new(None);
        let started = Instant::now();
        let mut sized = None;
        let mut advanced = 0;
        let mut last = None;
        while started.elapsed() < Duration::from_secs(20) && advanced < 6 {
            loader.poll(&context);
            loader.begin_frame();
            if let Some(media) =
                loader.media_texture_for_priority(Some(&url), [64, 64], ArtworkFetchPriority::Hero)
            {
                if media.image_size[0] > 1 {
                    sized = Some(media.image_size);
                    let next = loader.next_animation_frame();
                    if next != last {
                        advanced += 1;
                    }
                    last = next;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = std::fs::remove_dir_all(&directory);
        assert_eq!(sized, Some([64, 36]));
        assert!(advanced >= 6, "video never advanced");
    }
}
