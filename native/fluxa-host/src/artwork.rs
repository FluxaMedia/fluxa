use super::*;

pub(super) const MAX_ARTWORK_SIDE: u32 = 1536;
// Android uses the same initial-document prefetch contract as desktop. This
// is still bounded, but avoids evicting visible cards while the remaining
// shelves finish decoding.
pub(super) const MAX_ARTWORK_TEXTURES: usize = 128;

pub(super) struct ArtworkLoader {
    pub(super) fetcher: ArtworkFetcher,
    textures: HashMap<String, egui::TextureHandle>,
    animations: HashMap<String, ArtworkAnimation>,
    animation_checked: HashSet<String>,
    active_animations: HashSet<String>,
    animation_slots: HashSet<String>,
    latest_keys: HashMap<String, String>,
    texture_last_used: HashMap<String, u64>,
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
        Self {
            fetcher: ArtworkFetcher::new(cache_dir, 4),
            textures: HashMap::new(),
            animations: HashMap::new(),
            animation_checked: HashSet::new(),
            active_animations: HashSet::new(),
            animation_slots: HashSet::new(),
            latest_keys: HashMap::new(),
            texture_last_used: HashMap::new(),
            transparent_pixel_ratio: HashMap::new(),
            tones: HashMap::new(),
            access_counter: 0,
            disabled: std::env::var_os("FLUXA_NATIVE_DISABLE_ARTWORK").is_some(),
        }
    }

    pub(super) fn poll(&mut self, context: &egui::Context) {
        for prepared in self.fetcher.poll(2) {
            let url = prepared.source_url;
            let target_size = prepared.target_size;
            let animation_requested = prepared.animation_requested;
            let animation_atlas = prepared.animation_atlas;
            let image_pixels = animation_atlas
                .as_ref()
                .map(|atlas| &atlas.image)
                .unwrap_or(&prepared.image);
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
            if image_pixels.width() > max_side || image_pixels.height() > max_side {
                host_log(format!(
                    "Artwork skipped: {}x{} exceeds {max_side}",
                    image_pixels.width(),
                    image_pixels.height()
                ));
                continue;
            }
            let image = egui::ColorImage::from_rgba_premultiplied(
                [
                    image_pixels.width() as usize,
                    image_pixels.height() as usize,
                ],
                image_pixels.as_raw(),
            );
            let transparent_ratio = prepared.transparent_ratio;
            let tones = prepared.tones;
            let texture = context.load_texture(
                format!("fluxa-native-artwork-{key}"),
                image,
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
            self.transparent_pixel_ratio
                .insert(key.clone(), transparent_ratio);
            self.tones.insert(key.clone(), tones);
            self.textures.insert(key.clone(), texture);
            if let Some(atlas) = animation_atlas.filter(|atlas| atlas.frames.len() > 1) {
                self.animations.insert(
                    key,
                    ArtworkAnimation {
                        frames: atlas.frames,
                        frame_index: 0,
                        started_at: None,
                        next_frame_at: Instant::now(),
                    },
                );
            }
            while self.textures.len() > MAX_ARTWORK_TEXTURES {
                let Some(oldest) = self
                    .texture_last_used
                    .iter()
                    .min_by_key(|(_, last_used)| *last_used)
                    .map(|(key, _)| key.clone())
                else {
                    break;
                };
                self.textures.remove(&oldest);
                self.texture_last_used.remove(&oldest);
                self.transparent_pixel_ratio.remove(&oldest);
                self.tones.remove(&oldest);
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
        if let Some(texture) = self.textures.get(&key) {
            self.texture_last_used
                .insert(key.clone(), self.access_counter);
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
        if self.disabled {
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
            self.texture_last_used.insert(key, self.access_counter);
            return Some(texture.id());
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
