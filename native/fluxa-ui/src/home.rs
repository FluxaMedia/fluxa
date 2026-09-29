use super::*;

pub trait HomeAssets {
    fn background(&self) -> TextureId;
    fn texture(&mut self, url: Option<&str>) -> Option<TextureId>;
    /// Requests artwork at the raster size this component actually needs.
    /// Hosts may map this to a provider-sized URL and/or a bounded decode;
    /// the fallback keeps small test hosts source-compatible.
    fn texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        _priority: ArtworkPriority,
    ) -> Option<TextureId> {
        let _ = target_size;
        self.texture(url)
    }
    /// Starts loading artwork without requiring its row/card to be painted yet.
    /// Hosts can use this to prepare the next shelf while the current shelf is
    /// on screen. The default keeps simple/test hosts compatible.
    fn prefetch(&mut self, url: Option<&str>) {
        let _ = self.texture(url);
    }
    fn prefetch_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) {
        let _ = self.texture_for(url, target_size, priority);
    }
    fn prefetch_animated_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        _priority: ArtworkPriority,
    ) {
        let _ = (url, target_size);
    }
    fn texture_size(&self, _url: Option<&str>) -> Option<[u32; 2]> {
        None
    }
    fn has_native_emoji(&self) -> bool {
        false
    }
    fn emoji(&mut self, _cluster: &str) -> Option<TextureId> {
        None
    }
    fn artwork_tones(&self, _url: Option<&str>) -> Option<[[u8; 3]; 2]> {
        None
    }
    fn animated_texture_for(
        &mut self,
        _url: Option<&str>,
        _target_size: [u32; 2],
        _priority: ArtworkPriority,
    ) -> Option<AnimatedTexture> {
        None
    }
    fn cached_texture(&self, _url: Option<&str>) -> Option<TextureId> {
        None
    }
    /// Returns a texture rasterized from the shared native SVG icon set.
    /// Hosts upload the same SVG bytes; the UI never substitutes glyphs or
    /// hand-drawn approximations when an icon is requested.
    fn icon(&self, _name: &str) -> Option<TextureId> {
        None
    }
    fn logo(&self, _name: &str) -> Option<(TextureId, Vec2)> {
        None
    }
    fn active_profile_name(&self) -> Option<&str> {
        None
    }
    fn brand_mark(&self) -> Option<TextureId> {
        None
    }
    fn brand_colors(&self) -> Option<[Color32; 2]> {
        None
    }
    fn app_icon(&self, _id: &str) -> Option<TextureId> {
        None
    }
    fn ambient_glow(&self) -> Option<TextureId> {
        None
    }
    fn custom_background_url(&self) -> Option<&str> {
        None
    }
    fn active_profile_avatar_url(&self) -> Option<&str> {
        None
    }
    fn accent_color(&self) -> Option<Color32> {
        None
    }
}

pub fn metrics_for_assets(viewport: Viewport, assets: &impl HomeAssets) -> UiMetrics {
    let mut metrics = UiMetrics::for_viewport(viewport);
    if let Some(accent) = assets.accent_color() {
        metrics.accent = accent;
        let luminance =
            0.2126 * accent.r() as f32 + 0.7152 * accent.g() as f32 + 0.0722 * accent.b() as f32;
        metrics.accent_foreground = if luminance > 140.0 {
            Color32::from_rgb(20, 20, 22)
        } else {
            Color32::WHITE
        };
    }
    metrics
}

#[derive(Clone, Debug, Default)]
pub(crate) struct HeroFade {
    key: String,
    from: Option<(TextureId, [u32; 2])>,
    current: Option<(TextureId, [u32; 2])>,
    started: f64,
}

#[derive(Clone)]
pub(crate) struct LastReadyHeroTexture {
    texture: TextureId,
    size: [u32; 2],
    hero: HomeHero,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArtworkPriority {
    Hero,
    #[default]
    Visible,
    Prefetch,
}

/// Selects a provider-sized source when the URL is a TMDB image URL. Other
/// providers keep their original URL; their loaders still bound the decoded
/// raster before uploading it to the GPU.
pub fn artwork_request_url(url: &str, target_size: [u32; 2]) -> String {
    if !url.contains("image.tmdb.org") {
        return url.to_owned();
    }
    let marker = "/t/p/";
    let Some(marker_start) = url.find(marker) else {
        return url.to_owned();
    };
    let path_start = marker_start + marker.len();
    let Some((_, path)) = url[path_start..].split_once('/') else {
        return url.to_owned();
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
        "original"
    };
    format!("{}{variant}/{path}", &url[..path_start])
}

pub(crate) fn home_row_dimensions(metrics: UiMetrics, kind: HomeRowKind) -> (f32, f32, f32) {
    match kind {
        HomeRowKind::Continue => (
            metrics.home_continue_card_width,
            metrics.home_continue_card_height,
            metrics.home_continue_card_height,
        ),
        HomeRowKind::Poster => (
            metrics.poster_card_width,
            metrics.poster_card_height,
            metrics.poster_card_height
                + metrics.control_gap
                + metrics.screen_card_title_size
                + metrics.screen_card_subtitle_size
                + metrics.control_gap,
        ),
        HomeRowKind::Collection => (
            156.0,
            234.0,
            234.0 + metrics.control_gap + metrics.screen_card_title_size,
        ),
    }
}

pub(crate) fn home_collection_card_dimensions(metrics: UiMetrics, card: &HomeCard) -> (f32, f32) {
    let (width, height) = match card
        .collection_shape
        .as_deref()
        .unwrap_or("poster")
        .to_ascii_lowercase()
        .as_str()
    {
        "wide" | "landscape" => (280.0, 158.0),
        "square" => (150.0, 150.0),
        _ => (156.0, 234.0),
    };
    (
        width * metrics.home_collection_scale,
        height * metrics.home_collection_scale,
    )
}

pub(crate) fn home_card_dimensions(
    metrics: UiMetrics,
    card: &HomeCard,
    kind: HomeRowKind,
) -> (f32, f32) {
    if kind == HomeRowKind::Collection {
        return home_collection_card_dimensions(metrics, card);
    }
    let (width, height, _) = home_row_dimensions(metrics, kind);
    if kind != HomeRowKind::Poster || card.is_landscape() {
        return (width, height);
    }
    match card.poster_shape.as_deref() {
        Some("landscape") => (height * 16.0 / 9.0, height),
        Some("square") => (height, height),
        _ => (width, height),
    }
}

pub(crate) fn home_row_body_height(
    metrics: UiMetrics,
    cards: &[HomeCard],
    kind: HomeRowKind,
) -> f32 {
    if kind != HomeRowKind::Collection {
        return home_row_dimensions(metrics, kind).2;
    }
    let max_tile_height = cards
        .iter()
        .map(|card| home_collection_card_dimensions(metrics, card))
        .map(|(_, height)| height)
        .fold(0.0_f32, f32::max);
    max_tile_height + metrics.control_gap + metrics.screen_card_title_size
}

pub(crate) fn home_metrics(viewport: Viewport) -> UiMetrics {
    let mut metrics = UiMetrics::for_viewport(viewport);
    if viewport.is_compact() {
        metrics.home_continue_card_width *= 1.15;
        metrics.home_continue_card_height *= 1.15;
    } else if !viewport.is_tv() {
        metrics.home_continue_card_width *= 1.10;
        metrics.home_continue_card_height *= 1.10;
    }
    metrics
}

/// Height occupied by a shelf heading and the gap between it and its cards.
/// Keep this in the shared geometry calculation: omitting it made each next
/// shelf start while the previous shelf's cards were still on screen.
pub(crate) fn home_row_heading_height(metrics: UiMetrics, tv: bool) -> f32 {
    let title_size = if tv {
        metrics.catalog_title_size
    } else {
        metrics.catalog_title_size - 4.0
    };
    title_size * 1.25 + metrics.control_gap
}

pub(crate) fn home_artwork_signature(home: &HomeModel) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    home.background_url.hash(&mut hasher);
    home.logo_url.hash(&mut hasher);
    for hero in &home.hero_slides {
        hero.background_url.hash(&mut hasher);
        hero.logo_url.hash(&mut hasher);
    }
    for card in &home.cards {
        card.artwork_url.hash(&mut hasher);
        card.motion_url.hash(&mut hasher);
        card.motion_enabled.hash(&mut hasher);
    }
    home.gif_autoplay_enabled.hash(&mut hasher);
    for row in &home.rows {
        row.title.hash(&mut hasher);
        row.kind.hash(&mut hasher);
        for card in &row.cards {
            card.artwork_url.hash(&mut hasher);
            card.motion_url.hash(&mut hasher);
            card.motion_enabled.hash(&mut hasher);
        }
    }
    hasher.finish()
}

pub(crate) fn should_prefetch_home_artwork(context: &egui::Context, home: &HomeModel) -> bool {
    let signature = *home
        .artwork_signature
        .get_or_init(|| home_artwork_signature(home));
    context.data_mut(|data| {
        let id = Id::new("fluxa-home-artwork-prefetch");
        if data.get_temp::<u64>(id) == Some(signature) {
            false
        } else {
            data.insert_temp(id, signature);
            true
        }
    })
}

pub(crate) fn home_hero_height(viewport: Viewport) -> f32 {
    let base_height = if viewport.is_compact() {
        if viewport.width > viewport.height * 1.15 {
            (viewport.width * 5.0 / 12.0).min(440.0)
        } else {
            (viewport.width * 4.0 / 3.0).min(560.0)
        }
    } else {
        (viewport.height * 0.66 + 120.0).clamp(728.0, 984.0)
    };
    base_height * 0.93
}

/// The first shelf starts after the complete hero viewport, not after an
/// estimate of where the hero buttons happen to land. This keeps the hero
/// controls and the shelf heading in separate visual blocks on every size.
pub(crate) fn home_row_start(
    viewport: Viewport,
    metrics: UiMetrics,
    hero_height: f32,
    show_hero: bool,
) -> f32 {
    if !show_hero {
        return metrics.page_padding / 2.0;
    }
    hero_height
        + if viewport.is_compact() {
            metrics.control_gap.max(8.0)
        } else {
            metrics.section_gap.max(metrics.control_gap * 2.0)
        }
}

#[derive(Clone, Debug)]
pub struct ChoiceRequest {
    pub key: &'static str,
    pub title: String,
    pub options: Vec<(String, String)>,
    pub selected: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerGesture {
    Volume(f32),
    Brightness(f32),
}

#[derive(Clone, Debug, Default)]
pub struct HomeLayout {
    pub choices: Vec<(u64, ChoiceRequest)>,
    pub focusable: Vec<(u64, Rect)>,
    pub activated: Option<u64>,
    pub text_input: Option<String>,
    pub text_input_node: Option<u64>,
    pub setting_change: Option<(String, serde_json::Value)>,
    pub filter_change: Option<(String, String)>,
    /// Core page requests generated when a shelf approaches its loaded end.
    pub load_more: Vec<serde_json::Value>,
    pub seek_to: Option<f64>,
    pub seek_hover: Option<f64>,
    pub player_gesture: Option<PlayerGesture>,
    pub player_speed_hold: bool,
    pub profiles: Option<ProfilesRequest>,
    pub scroll_max: Option<f32>,
}

/// consume the same responsive rectangles as the painter.
pub fn home_layout(viewport: Viewport, home: &HomeModel) -> HomeLayout {
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let show_hero = home.show_hero_section && home.item_id.is_some();
    let metrics = home_metrics(viewport);
    let margin = if compact {
        metrics.page_padding
    } else if tv {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let mut layout = HomeLayout::default();
    if !compact {
        layout
            .focusable
            .extend(navigation_focus_rects(viewport, metrics));
    }
    let hero_height = home_hero_height(viewport);
    if show_hero {
        let play_y = hero_height - if compact { 92.0 } else { 194.0 } - home.scroll_offset;
        let size = Vec2::new(
            if compact {
                108.0
            } else {
                (metrics.horizontal_card_width * 0.46).max(160.0)
            },
            if compact { 42.0 } else { 50.0 },
        );
        let play = Rect::from_min_size(
            Pos2::new(
                if compact {
                    (viewport.width - size.x) * 0.5
                } else {
                    margin
                },
                play_y,
            ),
            size,
        );
        layout.focusable.push((NODE_PLAY, play));
    }
    let row_start = home_row_start(viewport, metrics, hero_height, show_hero);
    let mut flat = 0;
    let mut row_y = row_start;
    for (row_index, (_, cards, kind)) in home
        .content_rows_with_kind()
        .into_iter()
        .enumerate()
        .take(64)
    {
        let (_, _, body_height) = home_row_dimensions(metrics, kind);
        let row_height = home_row_heading_height(metrics, viewport.is_tv()) + body_height;
        let visible_y = row_y - home.scroll_offset;
        if visible_y + row_height < 0.0 || visible_y > viewport.height {
            row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
            flat += cards.len();
            continue;
        }
        let row_scroll = home
            .row_scroll_offsets
            .get(row_index)
            .copied()
            .unwrap_or(0.0);
        let mut card_x = margin - row_scroll;
        for card in cards {
            let (item_width, item_height) = home_card_dimensions(metrics, card, kind);
            layout.focusable.push((
                NODE_CARD_BASE + flat as u64,
                Rect::from_min_size(
                    Pos2::new(
                        card_x,
                        visible_y + home_row_heading_height(metrics, viewport.is_tv()),
                    ),
                    Vec2::new(item_width, item_height),
                ),
            ));
            card_x += item_width + metrics.horizontal_spacing;
            flat += 1;
        }
        row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
    }
    if compact {
        layout
            .focusable
            .extend(navigation_focus_rects(viewport, metrics));
    }
    layout
}

/// The shelf under the given touch point, if it is over a row's cards rather
/// than the hero or the space between shelves.
pub fn home_row_at_y(viewport: Viewport, home: &HomeModel, y: f32) -> Option<usize> {
    let metrics = home_metrics(viewport);
    let show_hero = home.show_hero_section && home.item_id.is_some();
    let hero_height = home_hero_height(viewport);
    let mut row_y = home_row_start(viewport, metrics, hero_height, show_hero);
    let content_y = y + home.scroll_offset;
    for (index, (_, cards, kind)) in home.content_rows_with_kind().into_iter().enumerate() {
        let body_height = home_row_body_height(metrics, cards, kind);
        let heading_height = home_row_heading_height(metrics, viewport.is_tv());
        if content_y >= row_y + heading_height && content_y <= row_y + heading_height + body_height
        {
            return Some(index);
        }
        row_y += heading_height + body_height + metrics.section_gap + metrics.vertical_spacing;
        if cards.is_empty() {
            continue;
        }
    }
    None
}

/// Maximum horizontal drag offset for a Home shelf.
pub fn home_row_scroll_max(viewport: Viewport, home: &HomeModel, row_index: usize) -> f32 {
    let Some((_, cards, kind)) = home.content_rows_with_kind().nth(row_index) else {
        return 0.0;
    };
    let metrics = home_metrics(viewport);
    home_row_scroll_max_for_cards(viewport, metrics, cards, kind)
}

pub(crate) fn home_row_scroll_max_for_cards(
    viewport: Viewport,
    metrics: UiMetrics,
    cards: &[HomeCard],
    kind: HomeRowKind,
) -> f32 {
    let margin = if viewport.is_compact() {
        metrics.page_padding
    } else if viewport.is_tv() {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let content_width = cards
        .iter()
        .map(|card| home_card_dimensions(metrics, card, kind).0 + metrics.horizontal_spacing)
        .sum::<f32>()
        - metrics.horizontal_spacing;
    (content_width - (viewport.width - margin * 2.0)).max(0.0)
}

/// Maximum vertical offset for Home's touch/scroll adapter.
pub fn home_scroll_max(viewport: Viewport, home: &HomeModel) -> f32 {
    let compact = viewport.is_compact();
    let show_hero = home.show_hero_section && home.item_id.is_some();
    let metrics = home_metrics(viewport);
    let hero_height = home_hero_height(viewport);
    let mut content_height = home_row_start(viewport, metrics, hero_height, show_hero);
    for (_, cards, kind) in home.content_rows_with_kind().into_iter().take(64) {
        content_height += home_row_heading_height(metrics, viewport.is_tv())
            + home_row_body_height(metrics, cards, kind)
            + metrics.section_gap
            + metrics.vertical_spacing;
    }
    content_height -= metrics.section_gap + metrics.vertical_spacing;
    let bottom_navigation_reserve = if compact {
        mobile_nav_reserve(viewport)
    } else {
        0.0
    };
    (content_height - viewport.height + bottom_navigation_reserve + metrics.page_padding).max(0.0)
}

pub fn draw_home(
    context: &egui::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    draw_home_with_options(context, viewport, home, assets, focused, true)
}

pub fn draw_home_content(
    context: &egui::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    draw_home_with_options(context, viewport, home, assets, focused, false)
}

pub(crate) fn draw_home_with_options(
    context: &egui::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
    draw_top_bar: bool,
) -> HomeLayout {
    let mut metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    if compact {
        metrics.home_continue_card_width *= 1.15;
        metrics.home_continue_card_height *= 1.15;
    } else if !tv {
        metrics.home_continue_card_width *= 1.10;
        metrics.home_continue_card_height *= 1.10;
    }
    let scroll_offset = if viewport.form_factor == UiFormFactor::Desktop {
        resolve_screen_scroll(
            context,
            viewport,
            Id::new("fluxa-screen-scroll-home"),
            home_scroll_max(viewport, home),
        )
    } else {
        home.scroll_offset
    };
    let active_hero_index = home_hero_index(context, home);
    let fallback_hero = HomeHero {
        title: home.title.clone(),
        eyebrow: home.eyebrow.clone(),
        description: home.description.clone(),
        background_url: home.background_url.clone(),
        logo_url: home.logo_url.clone(),
        item_id: home.item_id.clone(),
        item_type: home.item_type.clone(),
        trailers: Vec::new(),
        raw: serde_json::Value::Null,
    };
    let mut hero = home
        .hero_slides
        .get(active_hero_index)
        .unwrap_or(&fallback_hero)
        .clone();
    let show_hero = home.show_hero_section && hero.item_id.is_some();
    let hero_height = home_hero_height(viewport);
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let hero_rect = Rect::from_min_size(
        Pos2::new(0.0, -scroll_offset),
        Vec2::new(viewport.width, hero_height),
    );
    // The content geometry ends at `hero_rect`, but the artwork must continue
    // underneath the gap before the first shelf. This is what gives the web
    // and Compose heroes their soft fade instead of a hard image-to-black
    // edge.
    let hero_fade_height = if compact { 96.0 } else { 176.0 };
    let hero_visual_rect = Rect::from_min_size(
        hero_rect.min,
        Vec2::new(viewport.width, hero_height + hero_fade_height),
    );
    let mut hero_slide_offset = 0.0;
    let mut hero_opacity = 1.0;
    let prefetch_home_artwork = should_prefetch_home_artwork(context, home);
    if prefetch_home_artwork {
        let full_target = backdrop_target_size(viewport.width, context.pixels_per_point());
        assets.prefetch_for(
            hero.background_url.as_deref(),
            full_target,
            ArtworkPriority::Prefetch,
        );
        for slide in &home.hero_slides {
            assets.prefetch_for(
                slide.background_url.as_deref(),
                full_target,
                ArtworkPriority::Prefetch,
            );
        }
    }
    if show_hero {
        let full_target = backdrop_target_size(viewport.width, context.pixels_per_point());
        let full_texture = assets.texture_for(
            hero.background_url.as_deref(),
            full_target,
            ArtworkPriority::Hero,
        );
        let cached_texture = full_texture.or_else(|| {
            // A different target may already be decoded (for example after a
            // resize). Reuse it while the exact hero-sized raster is being
            // prepared instead of flashing to a black rectangle.
            assets.cached_texture(hero.background_url.as_deref())
        });
        let last_ready_id = Id::new("fluxa-last-ready-hero-texture");
        let last_ready = context.data(|data| data.get_temp::<LastReadyHeroTexture>(last_ready_id));
        let (hero_texture, hero_size) = if let Some(texture) = cached_texture {
            let size = assets
                .texture_size(hero.background_url.as_deref())
                .unwrap_or(full_target);
            context.data_mut(|data| {
                data.insert_temp(
                    last_ready_id,
                    LastReadyHeroTexture {
                        texture,
                        size,
                        hero: hero.clone(),
                    },
                );
            });
            (Some(texture), size)
        } else if let Some(last_ready) = last_ready {
            // Keep the previous *complete* hero during a carousel transition
            // or an artwork decode gap. Pairing its texture with the new
            // slide's title/logo made backgrounded windows resume with mixed
            // content until the next image finished loading.
            hero = last_ready.hero;
            hero_slide_offset = 0.0;
            (Some(last_ready.texture), last_ready.size)
        } else {
            (None, full_target)
        };
        let fade_id = Id::new("fluxa-home-hero-fade");
        let now = context.input(|input| input.time);
        let key = hero
            .item_id
            .clone()
            .or_else(|| hero.background_url.clone())
            .unwrap_or_else(|| hero.title.clone());
        let mut fade = context
            .data(|data| data.get_temp::<HeroFade>(fade_id))
            .unwrap_or_default();
        let current = hero_texture.map(|texture| (texture, hero_size));
        if fade.key != key {
            fade.from = if fade.key.is_empty() {
                None
            } else {
                fade.current
            };
            fade.key = key;
            fade.started = now;
        }
        fade.current = current;
        context.data_mut(|data| data.insert_temp(fade_id, fade.clone()));
        let linear = ((now - fade.started) / 0.8).clamp(0.0, 1.0) as f32;
        let t = 1.0 - (1.0 - linear).powi(3);
        if linear < 1.0 {
            context.request_repaint();
        }
        if fade.from.is_some() {
            hero_slide_offset = 28.0 * (1.0 - t);
            hero_opacity = t;
        }
        if let Some((texture, size)) = fade.from.filter(|_| linear < 1.0) {
            painter.image(
                texture,
                hero_visual_rect,
                cover_uv(size, hero_visual_rect),
                Color32::from_white_alpha((210.0 * (1.0 - t)) as u8),
            );
        }
        if let Some(hero_texture) = hero_texture {
            let zoom = if fade.from.is_some() {
                1.0 + 0.035 * (1.0 - t)
            } else {
                1.0
            };
            let hero_image_rect =
                Rect::from_center_size(hero_visual_rect.center(), hero_visual_rect.size() * zoom);
            let uv = cover_uv(hero_size, hero_image_rect);
            let alpha = if fade.from.is_some() { t } else { 1.0 };
            painter.with_clip_rect(hero_visual_rect).image(
                hero_texture,
                hero_image_rect,
                uv,
                Color32::from_white_alpha((210.0 * alpha) as u8),
            );
        }
        if let Some(texture) = home
            .trailer
            .as_ref()
            .filter(|trailer| hero.item_id.as_deref() == Some(trailer.item_id.as_str()))
            .and_then(|trailer| trailer.texture)
        {
            paint_trailer(
                context,
                &painter,
                texture,
                hero_visual_rect,
                hero.item_id.as_deref(),
                210,
            );
        }
        paint_hero_scrim(&painter, hero_visual_rect, compact, metrics.background);
    }
    let margin = if compact {
        16.0
    } else if tv {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let mut activated = None;
    let mut hero_actions: Option<Rect> = None;
    let mut hero_watchlist: Option<Rect> = None;
    let has_home_content = show_hero || !home.cards.is_empty() || !home.rows.is_empty();
    let loading_screen = !has_home_content && home.is_loading;
    if draw_top_bar && !loading_screen {
        let profile_avatar_url = home
            .profile_avatar_url
            .clone()
            .or_else(|| assets.active_profile_avatar_url().map(ToOwned::to_owned));
        let profile_name = home
            .profile_name
            .clone()
            .or_else(|| assets.active_profile_name().map(ToOwned::to_owned))
            .unwrap_or_else(|| "Profile".to_owned());
        if let Some(avatar_url) = profile_avatar_url.as_deref() {
            let _ = assets.texture_for(Some(avatar_url), [96, 96], ArtworkPriority::Prefetch);
        }
        activated = draw_navigation_bar_with_profile(
            context,
            viewport,
            0,
            assets,
            profile_avatar_url.as_deref(),
            &profile_name,
        );
    }
    if !has_home_content {
        let layout = HomeLayout {
            focusable: if loading_screen {
                Vec::new()
            } else {
                navigation_focus_rects(viewport, metrics)
            },
            activated,
            text_input: None,
            text_input_node: None,
            setting_change: None,
            filter_change: None,
            load_more: Vec::new(),
            ..HomeLayout::default()
        };
        if loading_screen {
            draw_loading_screen(context, &painter, screen, assets);
        } else {
            painter.text(
                screen.center(),
                Align2::CENTER_CENTER,
                "Home",
                FontId::proportional(metrics.screen_title_size_mobile),
                metrics.text_primary,
            );
        }
        components::focus_ring(
            &painter,
            &layout.focusable,
            focused.filter(|id| !is_navigation_node(*id)),
            viewport,
            metrics,
        );
        return layout;
    }
    let hero_width = if compact {
        viewport.width - margin * 2.0
    } else {
        (viewport.width * 0.58).min(if tv {
            980.0
        } else {
            metrics.home_hero_content_max_width_desktop
        })
    };
    if show_hero {
        let bottom_padding = if compact { 44.0 } else { 48.0 };
        let has_logo = hero
            .logo_url
            .as_deref()
            .is_some_and(|url| !url.trim().is_empty());
        let title_height = if has_logo {
            if compact {
                72.0
            } else if tv {
                112.0
            } else {
                metrics.home_hero_logo_height_desktop
            }
        } else if compact {
            metrics.catalog_title_size * 2.25
        } else {
            metrics.catalog_title_size * 1.9
        };
        let metadata_height = if home.eyebrow.is_empty() {
            0.0
        } else if compact {
            18.0
        } else {
            22.0
        };
        let synopsis_size = if compact {
            metrics.card_title_size + 1.0
        } else if tv {
            metrics.card_title_size + 6.0
        } else {
            metrics.home_hero_synopsis_size_desktop
        };
        let synopsis_width = if compact {
            hero_width
        } else {
            (hero_width * 0.55).clamp(420.0, 720.0).min(hero_width)
        };
        let synopsis_color = Color32::from_white_alpha(240);
        let synopsis_galley = if compact || hero.description.is_empty() {
            None
        } else {
            let mut job = egui::text::LayoutJob::simple(
                hero.description.clone(),
                crate::fonts::regular(synopsis_size),
                synopsis_color,
                synopsis_width,
            );
            job.wrap.max_rows = if compact { 2 } else { 3 };
            job.wrap.overflow_character = Some('…');
            Some(context.fonts_mut(|fonts| fonts.layout_job(job)))
        };
        let synopsis_height = synopsis_galley
            .as_ref()
            .map_or(0.0, |galley| galley.size().y);
        let play_height = if compact { 42.0 } else { 44.0 };
        let synopsis_button_gap = if compact { 8.0 } else { 22.0 };
        let block_height = title_height
            + metadata_height
            + synopsis_height
            + play_height
            + if compact {
                26.0
            } else {
                20.0 + synopsis_button_gap
            };
        let content_top = (hero_height - bottom_padding - block_height).max(if compact {
            16.0
        } else {
            metrics.nav_vertical_padding * 2.0
        }) - scroll_offset
            + hero_slide_offset;
        egui::Area::new(Id::new("fluxa-shared-hero"))
            .constrain(false)
            .fixed_pos(Pos2::new(margin, content_top))
            .show(context, |ui| {
                // The Compose hero owns a fixed 3:4/12:5 slide. Clip every
                // label/control to that slide so long metadata cannot paint
                // over the next shelf.
                ui.set_clip_rect(ui.clip_rect().intersect(hero_rect).intersect(screen));
                ui.multiply_opacity(hero_opacity);
                ui.set_min_width(hero_width);
                ui.set_max_width(hero_width);
                // Keep the widget column left-anchored and calculate every
                // hero element against the same rect. Relying on Label's
                // halign here is subtly wrong: a Label can allocate only its
                // galley width, so fallback titles and metadata end up with
                // a different x than an explicitly allocated logo.
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                    // Keep transparent title marks away from the clip edge.
                    // Some provider logos have visible pixels right at the
                    // source bitmap boundary.
                    let logo_inset = if compact { 6.0 } else { 8.0 };
                    let max_logo_size = Vec2::new(
                        if compact {
                            220.0
                        } else if tv {
                            380.0
                        } else {
                            metrics.home_hero_logo_max_width_desktop.min(460.0)
                        },
                        (title_height - logo_inset * 2.0).max(1.0),
                    );
                    if home.hero_slides.len() > 1 {
                        let next =
                            &home.hero_slides[(active_hero_index + 1) % home.hero_slides.len()];
                        assets.texture_for(
                            next.logo_url.as_deref(),
                            artwork_target_size(max_logo_size, context.pixels_per_point()),
                            ArtworkPriority::Hero,
                        );
                    }
                    let title_font_size = if compact {
                        metrics.catalog_title_size + 2.0
                    } else if tv {
                        metrics.catalog_title_size * 2.0
                    } else {
                        metrics.catalog_title_size * 1.65
                    };
                    let title_rect = ui
                        .allocate_exact_size(Vec2::new(hero_width, title_height), Sense::hover())
                        .0;
                    let title_anchor = if compact {
                        egui::Align2::CENTER_TOP
                    } else {
                        egui::Align2::LEFT_TOP
                    };
                    if has_logo {
                        if let Some((texture, size)) = components::title_logo(
                            context.pixels_per_point(),
                            hero.logo_url.as_deref(),
                            max_logo_size,
                            ArtworkPriority::Hero,
                            assets,
                        ) {
                            let logo_x = if compact {
                                title_rect.center().x - size.x * 0.5
                            } else {
                                title_rect.left()
                            };
                            let logo_rect = Rect::from_min_size(
                                Pos2::new(logo_x, title_rect.center().y - size.y * 0.5),
                                size,
                            );
                            ui.painter()
                                .image(texture, logo_rect, full_uv(), Color32::WHITE);
                        }
                    } else {
                        let title = truncate_text(&hero.title, if compact { 52 } else { 72 });
                        ui.painter().text(
                            title_rect.min,
                            title_anchor,
                            title,
                            egui::FontId::proportional(title_font_size),
                            Color32::WHITE,
                        );
                    }
                    ui.add_space(if compact { 6.0 } else { 16.0 });
                    if !hero.eyebrow.is_empty() {
                        let metadata_rect = ui
                            .allocate_exact_size(
                                Vec2::new(hero_width, metadata_height),
                                Sense::hover(),
                            )
                            .0;
                        ui.painter().text(
                            if compact {
                                metadata_rect.center_top()
                            } else {
                                metadata_rect.left_top()
                            },
                            if compact {
                                egui::Align2::CENTER_TOP
                            } else {
                                egui::Align2::LEFT_TOP
                            },
                            truncate_text(&hero.eyebrow, if compact { 64 } else { 90 }),
                            egui::FontId::proportional(if tv {
                                metrics.nav_label_size + 4.0
                            } else {
                                metrics.nav_label_size
                            }),
                            Color32::from_white_alpha(185),
                        );
                    }
                    ui.add_space(if compact { 6.0 } else { 4.0 });
                    if let Some(galley) = synopsis_galley.as_ref() {
                        let synopsis_rect = ui
                            .allocate_exact_size(
                                Vec2::new(synopsis_width, synopsis_height),
                                Sense::hover(),
                            )
                            .0;
                        let synopsis_x = if compact {
                            synopsis_rect.center().x - galley.size().x * 0.5
                        } else {
                            synopsis_rect.left()
                        };
                        ui.painter().galley(
                            Pos2::new(synopsis_x, synopsis_rect.top()),
                            galley.clone(),
                            synopsis_color,
                        );
                    }
                    ui.add_space(if compact { 6.0 } else { synopsis_button_gap });
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        let resume = hero.item_id.as_deref().and_then(|id| home.resume_for(id));
                        let series =
                            matches!(hero.item_type.as_deref(), Some("series" | "tv" | "show"));
                        let label =
                            play_label(&home.language, resume, series.then_some((1, 1, None)));
                        let text_size = metrics.nav_label_size + 2.0;
                        let measure = |ui: &egui::Ui, text: &str| {
                            ui.painter()
                                .layout_no_wrap(
                                    text.to_owned(),
                                    egui::FontId::proportional(text_size),
                                    Color32::WHITE,
                                )
                                .size()
                                .x
                        };
                        let play_width = measure(ui, &label) + 70.0;
                        let watchlist_label = localized("library.watchlist", &home.language);
                        let watchlist_width = measure(ui, &watchlist_label) + 70.0;
                        if compact {
                            let total = play_width + 12.0 + watchlist_width;
                            ui.add_space(((ui.available_width() - total) * 0.5).max(0.0));
                        }
                        let play = components::play_button(
                            ui,
                            assets,
                            &label,
                            Some(play_width),
                            play_height,
                            text_size,
                            resume
                                .map(|card| card.progress)
                                .filter(|progress| *progress > 0.0),
                        );
                        hero_actions = Some(play.rect);
                        if play.clicked() {
                            activated = Some(NODE_PLAY);
                        }
                        let saved = hero
                            .item_id
                            .as_deref()
                            .and_then(|id| poster_overlay::personal_for(ui.ctx(), id))
                            .is_some_and(|personal| personal.saved);
                        let watchlist = components::pill_button(
                            ui,
                            assets.icon(if saved { "Check" } else { "Plus" }),
                            &watchlist_label,
                            Some(watchlist_width),
                            play_height,
                            text_size,
                            false,
                            None,
                        );
                        hero_watchlist = Some(watchlist.rect);
                        if watchlist.clicked() {
                            activated = Some(NODE_HERO_WATCHLIST);
                        }
                    });
                });
            });
    }

    let row_start = home_row_start(viewport, metrics, hero_height, show_hero);
    let mut layout = HomeLayout::default();
    let mut flat_index = 0usize;
    let mut row_y = row_start;
    for (row_index, (title, cards, kind)) in home
        .content_rows_with_kind()
        .into_iter()
        .enumerate()
        .take(64)
    {
        let (card_width, card_height, _) = home_row_dimensions(metrics, kind);
        let body_height = home_row_body_height(metrics, cards, kind);
        let row_scroll_max = home_row_scroll_max_for_cards(viewport, metrics, cards, kind);
        let row_height = home_row_heading_height(metrics, tv) + body_height;
        let visible_y = row_y - scroll_offset;
        let row_is_visible = visible_y + row_height >= 0.0 && visible_y <= viewport.height;

        // Prefetch the complete home document, not just the visible row. This
        // is still lazy from the renderer's perspective: requests are async,
        // bounded by the host fetcher, decoded off-thread, and uploaded only
        // as completed images arrive. It prevents the web/Compose behaviour
        // difference where scrolling is required to start loading artwork.
        if prefetch_home_artwork {
            for card in cards {
                assets.prefetch_for(
                    card.poster_art(),
                    artwork_target_size(
                        Vec2::from(home_card_dimensions(metrics, card, kind)),
                        context.pixels_per_point(),
                    ),
                    ArtworkPriority::Prefetch,
                );
                if home.gif_autoplay_enabled
                    && kind == HomeRowKind::Collection
                    && card.motion_enabled
                    && !row_is_visible
                    && let Some(url) = card.motion_url.as_deref()
                {
                    let (width, height) = home_collection_card_dimensions(metrics, card);
                    assets.prefetch_animated_for(
                        Some(url),
                        fluxa_artwork::animation_target_size(artwork_target_size(
                            Vec2::new(width, height),
                            context.pixels_per_point(),
                        )),
                        ArtworkPriority::Prefetch,
                    );
                }
            }
        }
        if visible_y + row_height < 0.0 || visible_y > viewport.height {
            row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
            flat_index += cards.len();
            continue;
        }
        let heading_height = home_row_heading_height(metrics, tv);
        let type_label = row_index
            .checked_sub(usize::from(!home.cards.is_empty()))
            .and_then(|index| home.rows.get(index))
            .and_then(|row| row.type_label.as_deref());
        egui::Area::new(Id::new(format!("fluxa-shared-row-heading-{row_index}")))
            .fixed_pos(Pos2::new(margin, visible_y))
            // Catalog shelves are part of a scrolling document, not floating
            // dialogs. egui's default constraint pulls an off-screen shelf up
            // to fit the viewport, which makes it paint over the prior shelf.
            .constrain(false)
            .show(context, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(screen));
                let title_size = if tv {
                    metrics.catalog_title_size
                } else {
                    metrics.catalog_title_size - 4.0
                };
                let title = match type_label {
                    Some(label) => format!("{title} - {label}"),
                    None => title.to_owned(),
                };
                ui.label(RichText::new(title).size(title_size).strong());
                ui.add_space(metrics.control_gap);
            });
        let row_clip = Rect::from_min_max(
            Pos2::new(margin, visible_y + heading_height),
            Pos2::new(
                (viewport.width - margin).max(margin),
                (visible_y + heading_height + body_height).min(viewport.height),
            ),
        );
        let row_scroll_offset = if viewport.form_factor == UiFormFactor::Desktop {
            resolve_desktop_horizontal_scroll(
                context,
                Id::new(("fluxa-home-row-scroll", row_index)),
                row_clip,
                home.row_scroll_offsets
                    .get(row_index)
                    .copied()
                    .unwrap_or(0.0),
                row_scroll_max,
            )
        } else {
            home.row_scroll_offsets
                .get(row_index)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, row_scroll_max)
        };
        if viewport.form_factor == UiFormFactor::Desktop
            && let Some(row) = home
                .rows
                .get(row_index.saturating_sub(usize::from(!home.cards.is_empty())))
            && row.can_load_more
            && !cards.is_empty()
            && let Some(row_id) = row.id.as_deref().filter(|value| !value.is_empty())
        {
            let max_offset = row_scroll_max;
            let card_pitch = (card_width + metrics.horizontal_spacing).max(1.0);
            let prefetch = (viewport.width * 2.0).max(card_pitch * 6.0);
            if row_scroll_offset >= (max_offset - prefetch).max(0.0) {
                let request_id = Id::new(("fluxa-home-load-more", row_id));
                let already_requested = context.data_mut(|data| {
                    let previous = data.get_temp::<usize>(request_id).unwrap_or(0);
                    if previous == cards.len() {
                        true
                    } else {
                        data.insert_temp(request_id, cards.len());
                        false
                    }
                });
                if !already_requested && let Some(category) = row.catalog_page.as_ref() {
                    let content_type = category
                        .get("contentType")
                        .or_else(|| category.get("type"))
                        .and_then(serde_json::Value::as_str)
                        .filter(|value| matches!(*value, "movie" | "series"))
                        .unwrap_or("movie");
                    layout.load_more.push(serde_json::json!({
                        "type": "catalogPageRequested",
                        "categoryId": row_id,
                        "transportUrl": category.get("addonTransportUrl").or_else(|| category.get("transportUrl")),
                        "contentType": content_type,
                        "catalogId": category.get("catalogId").or_else(|| category.get("id")),
                        "skip": category.get("skip").and_then(serde_json::Value::as_i64).unwrap_or(0) + cards.len() as i64,
                        "genre": category.get("addonGenre").or_else(|| category.get("genre")),
                        "remoteSource": category.get("remoteSources").or_else(|| category.get("remoteSource")),
                    }));
                }
            }
        }
        egui::Area::new(Id::new(format!("fluxa-shared-row-cards-{row_index}")))
            .fixed_pos(Pos2::new(
                margin - row_scroll_offset,
                visible_y + heading_height,
            ))
            .constrain(false)
            .show(context, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(row_clip));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = metrics.horizontal_spacing;
                    for (column_index, card) in cards.iter().enumerate() {
                        let is_poster = !matches!(kind, HomeRowKind::Continue);
                        let (item_width, item_height) = home_card_dimensions(metrics, card, kind);
                        let slot_height = if is_poster { body_height } else { card_height };
                        let (widget_id, slot_rect) =
                            ui.allocate_space(Vec2::new(item_width, slot_height));
                        let rect =
                            Rect::from_min_size(slot_rect.min, Vec2::new(item_width, item_height));
                        let node_id = NODE_CARD_BASE + flat_index as u64;
                        layout.focusable.push((node_id, rect));
                        let card_visible = rect.intersects(screen) && rect.intersects(row_clip);
                        let response = card_visible
                            .then(|| ui.interact(slot_rect, widget_id, egui::Sense::click()));
                        if response.as_ref().is_some_and(|response| response.clicked()) {
                            activated = Some(node_id);
                        }
                        if card_visible {
                            if is_poster {
                                components::poster_card(
                                    ui.painter(),
                                    rect,
                                    card,
                                    column_index,
                                    viewport,
                                    metrics,
                                    assets,
                                    card.motion_enabled
                                        && card.motion_url.is_some()
                                        && ((home.gif_autoplay_enabled
                                            && mostly_visible(rect, screen.intersect(row_clip)))
                                            || focused == Some(node_id)
                                            || response
                                                .as_ref()
                                                .is_some_and(|response| response.hovered())),
                                );
                            } else {
                                components::continue_card(
                                    ui.painter(),
                                    rect,
                                    card,
                                    column_index,
                                    viewport,
                                    metrics,
                                    assets,
                                );
                            }
                        }
                        flat_index += 1;
                    }
                });
            });
        row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
    }
    let hero_click = context.input(|input| {
        input
            .pointer
            .primary_clicked()
            .then(|| input.pointer.interact_pos())
            .flatten()
    });
    if activated.is_none()
        && show_hero
        && !compact
        && hero_click.is_some_and(|pos| pos.y > 80.0 && hero_rect.intersect(screen).contains(pos))
    {
        activated = Some(NODE_MORE_INFO);
    }
    layout.activated = activated;
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    if show_hero {
        layout
            .focusable
            .push((NODE_MORE_INFO, hero_rect.intersect(screen)));
    }
    if let Some(watchlist) = hero_watchlist {
        layout.focusable.push((NODE_HERO_WATCHLIST, watchlist));
    }
    if let Some(play) = hero_actions {
        layout.focusable.push((NODE_PLAY, play));
    } else if show_hero {
        let play_y = hero_height - if compact { 92.0 } else { 194.0 } - scroll_offset;
        layout.focusable.push((
            NODE_PLAY,
            Rect::from_min_size(
                Pos2::new(
                    if compact {
                        (viewport.width - 108.0) * 0.5
                    } else {
                        margin
                    },
                    play_y,
                ),
                Vec2::new(
                    if compact {
                        108.0
                    } else {
                        (metrics.horizontal_card_width * 0.46).max(160.0)
                    },
                    if compact { 42.0 } else { 50.0 },
                ),
            ),
        ));
    }
    components::focus_ring(
        &painter,
        &layout.focusable,
        focused.filter(|id| !is_navigation_node(*id)),
        viewport,
        metrics,
    );
    layout
}

fn mostly_visible(rect: Rect, viewport: Rect) -> bool {
    let visible = rect.intersect(viewport);
    visible.is_positive() && visible.area() >= rect.area() * 0.6
}
