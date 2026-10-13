use super::*;

pub(super) const MAX_ARTWORK_SIDE: u32 = 1536;
// Android uses the same initial-document prefetch contract as desktop. This
// is still bounded, but avoids evicting visible cards while the remaining
// shelves finish decoding.
const MAX_CONVERTING: usize = 3;
const MAX_ARTWORK_TEXTURES: usize = 128;
const MAX_DECODED_BYTES: usize = 96 * 1024 * 1024;
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
    frames: Option<Vec<fluxa_artwork::AnimatedAtlasFrame>>,
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
    frames: Vec<fluxa_artwork::AnimatedAtlasFrame>,
    frame_index: usize,
    started_at: Option<Instant>,
    next_frame_at: Instant,
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
            let mut animation_atlas = prepared.animation_atlas.take();
            let source = match animation_atlas.as_mut() {
                Some(atlas) => &mut atlas.image,
                None => &mut prepared.image,
            };
            let (width, height) = (source.width(), source.height());
            let raw = std::mem::replace(source, image::RgbaImage::new(0, 0)).into_raw();
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
                    frames: animation_atlas.map(|atlas| atlas.frames),
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
            let key = if animation_requested {
                format!("{base_key}#animated-atlas")
            } else {
                base_key
            };
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
            if !animation_requested {
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
            if !animation_requested {
                self.latest_keys.insert(url, key.clone());
            }
            if animation_requested {
                self.animation_checked.insert(key.clone());
            }
            self.access_counter = self.access_counter.wrapping_add(1);
            self.texture_last_used
                .insert(key.clone(), self.access_counter);
            self.textures.insert(key.clone(), texture);
            if let Some(frames) = converted.frames.filter(|frames| frames.len() > 1) {
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
                        frames,
                        frame_index: 0,
                        started_at: None,
                        next_frame_at: Instant::now(),
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
            context.request_repaint();
        }
        let now = Instant::now();
        let active = self.active_animations.iter().cloned().collect::<Vec<_>>();
        let mut next_frame_in = None;
        for key in active {
            let Some(animation) = self.animations.get_mut(&key) else {
                continue;
            };
            if let Some(started_at) = animation.started_at
                && now >= animation.next_frame_at
            {
                let (frame_index, next_frame_at) =
                    fluxa_artwork::animation_frame_at(&animation.frames, started_at, now)
                        .unwrap_or((animation.frame_index, now + Duration::from_millis(100)));
                if frame_index != animation.frame_index {
                    animation.frame_index = frame_index;
                    context.request_repaint();
                }
                animation.next_frame_at = next_frame_at;
            }
            let wait = animation.next_frame_at.saturating_duration_since(now);
            next_frame_in = Some(next_frame_in.map_or(wait, |current: Duration| current.min(wait)));
        }
        if let Some(wait) = next_frame_in {
            context.request_repaint_after(wait.max(Duration::from_millis(1)));
        }
    }

    pub(super) fn next_animation_frame(&self) -> Option<Instant> {
        self.active_animations
            .iter()
            .filter_map(|key| self.animations.get(key))
            .map(|animation| animation.next_frame_at)
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
        let key = format!("{base_key}#animated-atlas");
        if self.animation_checked.contains(&key) && !self.animations.contains_key(&key) {
            return None;
        }
        if !self.animation_slots.contains(&key) {
            if self
                .fetcher
                .animated_request_is_backing_off(&source_url, target_size)
            {
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
        if self.animations.contains_key(&key) {
            let now = Instant::now();
            let animation = self.animations.get_mut(&key)?;
            animation.started_at.get_or_insert(now);
            let (frame_index, next_frame_at) =
                fluxa_artwork::animation_frame_at(&animation.frames, animation.started_at?, now)?;
            animation.frame_index = frame_index;
            animation.next_frame_at = next_frame_at;
            let frame = &animation.frames[frame_index];
            let texture = self.textures.get(&key)?;
            self.active_animations.insert(key.clone());
            return Some(AnimatedTexture {
                texture: texture.id(),
                uv: egui::Rect::from_min_max(
                    egui::pos2(frame.uv[0], frame.uv[1]),
                    egui::pos2(frame.uv[2], frame.uv[3]),
                ),
                image_size: frame.image_size,
            });
        }
        if self.animation_checked.contains(&key) {
            return None;
        }
        let _ = self
            .fetcher
            .request_animated(Some(&source_url), target_size, priority);
        None
    }

    pub(super) fn prefetch_animated_for_priority(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkFetchPriority,
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
            "{}#animated-atlas",
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
            .prefetch_animated(Some(&source_url), target_size, priority);
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
