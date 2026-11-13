use super::*;
use crate::dp::Dp;
use crate::draw_text;
use crate::kit::view;

mod category_bar;
mod hero;
mod layout;
mod metrics;
mod rows;
use category_bar::{category_bar_space, draw_category_bar};
use hero::HeroStyle;
pub use layout::*;
pub use metrics::*;
pub(crate) use rows::*;
pub use rows::{ExpandedPoster, expanded_poster, previewed_poster, set_episode_details};

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
    fn dom_images(&self) -> bool {
        false
    }
    fn artwork_failed(&self, _url: Option<&str>, _target_size: [u32; 2]) -> bool {
        false
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
    fn artwork_focus(&self, _url: Option<&str>) -> Option<std::sync::Arc<[f32]>> {
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
    fn media_texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) -> Option<AnimatedTexture> {
        let texture = self.texture_for(url, target_size, priority)?;
        Some(AnimatedTexture {
            texture,
            uv: Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            image_size: self.texture_size(url)?,
        })
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
    fn background_image_size(&self) -> Option<[u32; 2]> {
        None
    }
    fn custom_background_url(&self) -> Option<&str> {
        None
    }
    fn active_profile_avatar_url(&self) -> Option<&str> {
        None
    }
    fn playback_unavailable(&self) -> Option<&str> {
        None
    }
}

pub fn draw_home(
    context: &crate::gfx::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    draw_home_with_options(context, viewport, home, assets, focused, true)
}

pub fn draw_home_content(
    context: &crate::gfx::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    draw_home_with_options(context, viewport, home, assets, focused, false)
}

pub(crate) fn draw_home_with_options(
    context: &crate::gfx::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
    draw_top_bar: bool,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
    let flow = home_flow(viewport, metrics, home);
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
        badges: Vec::new(),
    };
    let pinned = home_hero_pinned(viewport, home);
    context.data_mut(|data| data.insert_temp(Id::new("fluxa-hero-pinned"), pinned));
    let mut hero = match pinned.then(|| focus_hero(context, home, focused)).flatten() {
        Some(focused_hero) => std::borrow::Cow::Owned(focused_hero),
        None => std::borrow::Cow::Borrowed(
            home.hero_slides
                .get(active_hero_index)
                .unwrap_or(&fallback_hero),
        ),
    };
    let hero_has_item = hero.item_id.is_some();
    let show_hero = home.show_hero_section
        && (hero_has_item || !home.cards.is_empty() || !home.rows.is_empty());
    let hero_height = home_hero_height_for(viewport, home);
    let screen = crate::gfx::rect_at(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let rows_screen = if pinned {
        let rows_top = home_row_start(viewport, metrics, hero_height, true);
        Rect::from_min_max(Pos2::new(0.0, rows_top), screen.max)
    } else {
        screen
    };
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let mut style = HeroStyle::new(viewport, metrics, metrics.screen_margin);
    if pinned {
        style.synopsis_rows = 2;
        style.bottom_padding = Dp::D28.px();
        style.logo_height = style
            .logo_height
            .min(hero_height * metrics.screen_hero_pinned_logo_ratio);
    }
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
    if pinned
        && let Some(node) =
            focused.filter(|node| (NODE_CARD_BASE..NODE_PREVIEW_BASE).contains(node))
    {
        let full_target = backdrop_target_size(viewport.width, context.pixels_per_point());
        let flat = (node - NODE_CARD_BASE) as usize;
        for index in flat.saturating_sub(3)..flat + 4 {
            if let Some(card) = home.card_at(index) {
                assets.prefetch_for(
                    focus_backdrop(card).as_deref(),
                    full_target,
                    ArtworkPriority::Prefetch,
                );
            }
        }
    }
    let margin = metrics.screen_margin;
    let mut activated = None;
    let mut hero_actions: Option<Rect> = None;
    let mut hero_watchlist: Option<Rect> = None;
    let has_home_content = hero_has_item || !home.cards.is_empty() || !home.rows.is_empty();
    if draw_top_bar {
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
            home_nav_viewport(viewport, home),
            0,
            assets,
            profile_avatar_url.as_deref(),
            &profile_name,
        );
    }
    if !has_home_content {
        let mut layout = HomeLayout {
            focusable: navigation_focus_rects(home_nav_viewport(viewport, home), metrics),
            activated,
            text_input: None,
            text_input_node: None,
            setting_change: None,
            filter_change: None,
            load_more: Vec::new(),
            ..HomeLayout::default()
        };
        if home.is_loading {
            draw_home_skeleton(context, &painter, screen, metrics, margin, assets);
        } else {
            let state_rect = screen.translate(Vec2::new(0.0, -metrics.screen_title_size));
            components::empty_state(
                &painter,
                state_rect,
                assets.icon("Discover"),
                &localized("home.no_catalogs", &home.language),
                &localized("home.no_catalogs_hint", &home.language),
                metrics,
            );
            let title_size = metrics.screen_section_title_size;
            let button_top = state_rect.center().y
                + (36.0 + metrics.section_gap + title_size * 2.6) * 0.5
                + metrics.screen_body_size * 2.4
                + metrics.section_gap;
            let label = localized("home.open_addons", &home.language);
            let width = (screen.width() - margin * 2.0).min(280.0);
            let mut button_rect = None;
            let mut clicked = false;
            crate::gfx::Area::new(Id::new("fluxa-home-open-addons"))
                .fixed_pos(Pos2::new(screen.center().x - width * 0.5, button_top))
                .show(context, |ui| {
                    let response = components::primary_button(ui, &label, width, true);
                    button_rect = Some(response.rect);
                    clicked = response.clicked();
                });
            if let Some(rect) = button_rect {
                layout.focusable.push((NODE_HOME_ADD_ADDONS, rect));
            }
            if clicked {
                layout.activated = Some(NODE_HOME_ADD_ADDONS);
            }
        }
        return layout;
    }
    let mut hero_rect = Rect::NOTHING;
    let mut shown_scroll = 0.0f32;
    let mut layout = HomeLayout::default();
    let pinned_top = if pinned { flow.rows_top } else { 0.0 };
    let rows_lead = if pinned { 0.0 } else { flow.rows_top };
    let rows_tail = if pinned {
        let last = home_shelves(metrics, home, 0.0)
            .take(64)
            .last()
            .map_or(0.0, |shelf| shelf.height());
        (viewport.height - pinned_top - last).max(0.0)
    } else {
        metrics.page_padding
            + if viewport.is_tv() {
                viewport.safe_bottom
            } else {
                0.0
            }
    };
    crate::kit::screen_body_below(context, viewport, "home", (), pinned_top, |ui| {
        let scroll_offset = pinned_top - ui.max_rect().min.y;
        shown_scroll = scroll_offset;
        let hero_scroll = if pinned {
            0.0
        } else {
            scroll_offset - flow.hero_top
        };
        hero_rect = screen.translate(Vec2::new(0.0, -hero_scroll));
        hero_rect.set_height(hero_height);
        let hero_fade_height = style.fade_height;
        let mut hero_visual_rect = hero_rect;
        hero_visual_rect.set_height(hero_height + hero_fade_height);
        let mut hero_slide_offset = 0.0;
        let mut hero_opacity = 1.0;
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
            let last_ready =
                context.data(|data| data.get_temp::<LastReadyHeroTexture>(last_ready_id));
            let (hero_texture, hero_size) = if let Some(texture) = cached_texture {
                let size = assets
                    .texture_size(hero.background_url.as_deref())
                    .unwrap_or(full_target);
                let remembered = match last_ready.as_ref().map(|ready| &ready.hero) {
                    Some(previous)
                        if previous.item_id == hero.item_id
                            && previous.title == hero.title
                            && previous.background_url == hero.background_url
                            && previous.logo_url == hero.logo_url
                            && previous.description == hero.description
                            && previous.badges.len() == hero.badges.len() =>
                    {
                        std::sync::Arc::clone(previous)
                    }
                    _ => std::sync::Arc::new(HomeHero::clone(&hero)),
                };
                context.data_mut(|data| {
                    data.insert_temp(
                        last_ready_id,
                        LastReadyHeroTexture {
                            texture,
                            size,
                            hero: remembered,
                        },
                    );
                });
                (Some(texture), size)
            } else if let Some(last_ready) = last_ready.filter(|_| hero_has_item) {
                // Keep the previous *complete* hero during a carousel transition
                // or an artwork decode gap. Pairing its texture with the new
                // slide's title/logo made backgrounded windows resume with mixed
                // content until the next image finished loading.
                hero = std::borrow::Cow::Owned(HomeHero::clone(&last_ready.hero));
                hero_slide_offset = 0.0;
                (Some(last_ready.texture), last_ready.size)
            } else {
                (None, full_target)
            };
            let fade_id = Id::new("fluxa-home-hero-fade");
            let now = context.input(|input| input.time());
            let key = hero
                .item_id
                .clone()
                .or_else(|| hero.background_url.clone())
                .unwrap_or_else(|| hero.title.clone());
            let mut fade = context
                .data(|data| data.get_temp::<HeroFade>(fade_id))
                .unwrap_or_default();
            let hero_focus = framed_focus(
                hero_size,
                hero_visual_rect,
                assets
                    .artwork_focus(hero.background_url.as_deref())
                    .as_deref(),
            );
            let current = hero_texture.map(|texture| (texture, hero_size, hero_focus));
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
            if let Some((texture, size, focus)) = fade.from.filter(|_| linear < 1.0) {
                painter.image(
                    texture,
                    hero_visual_rect,
                    cover_uv_focus(size, hero_visual_rect, focus),
                    Color32::from_white_alpha((210.0 * (1.0 - t)) as u8),
                );
            }
            if hero_texture.is_none() {
                crate::components::failed_dom_image(
                    &painter,
                    assets,
                    hero_visual_rect,
                    0.0,
                    hero.background_url.as_deref(),
                    full_target,
                    crate::ImageFit::Cover,
                    [0.5, 0.5],
                );
            }
            if let Some(hero_texture) = hero_texture {
                let zoom = if fade.from.is_some() {
                    1.0 + 0.035 * (1.0 - t)
                } else {
                    1.0
                };
                let hero_image_rect = hero_visual_rect.scale_from_center(zoom);
                let uv = cover_uv_focus(hero_size, hero_image_rect, hero_focus);
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
                    255,
                );
            }
            paint_hero_scrim(&painter, hero_visual_rect, metrics);
            paint_ambient_fade(
                &painter,
                Rect::from_min_max(
                    Pos2::new(
                        hero_visual_rect.left(),
                        hero_visual_rect.bottom() - hero_fade_height,
                    ),
                    hero_visual_rect.right_bottom(),
                ),
                screen,
                assets,
            );
            if let Some(text) = home
                .trailer
                .as_ref()
                .filter(|trailer| hero.item_id.as_deref() == Some(trailer.item_id.as_str()))
                .and_then(|trailer| trailer.subtitle.as_deref())
            {
                paint_trailer_subtitle(context, hero_rect, text);
            }
            let swipe_id = Id::new("fluxa-home-hero-swipe");
            let (pressed, released, position) = context.input(|input| {
                (
                    input.pointer().primary_pressed(),
                    input.pointer().primary_released(),
                    input.pointer().interact_pos(),
                )
            });
            if pressed {
                let start = position.filter(|position| hero_rect.contains(*position));
                context.data_mut(|data| data.insert_temp(swipe_id, start));
            }
            if released {
                let start = context.data_mut(|data| data.remove_temp::<Option<Pos2>>(swipe_id));
                if let (Some(Some(start)), Some(end)) = (start, position) {
                    let delta = end - start;
                    if delta.x.abs() > 70.0 && delta.x.abs() > delta.y.abs() * 1.5 {
                        hero_shift(context, home, if delta.x < 0.0 { 1 } else { -1 });
                    }
                }
            }
        }
        let hero_width = style.width;
        if show_hero && hero_has_item {
            let has_logo = hero
                .logo_url
                .as_deref()
                .is_some_and(|url| !url.trim().is_empty());
            let max_logo_size = Vec2::new(
                style.logo_max_width,
                (style.logo_height - style.logo_inset * 2.0).max(1.0),
            );
            let fitted_logo = if has_logo {
                components::title_logo(
                    context.pixels_per_point(),
                    hero.logo_url.as_deref(),
                    max_logo_size,
                    ArtworkPriority::Hero,
                    assets,
                )
            } else {
                None
            };
            let titled = !home.hero_poster;
            let title_height = if !titled {
                0.0
            } else if let Some((_, size)) = fitted_logo {
                size.y
            } else if has_logo {
                style.logo_height
            } else {
                style.text_title_height
            };
            let hero_ratings = crate::mdblist_ratings(&hero.raw);
            let ratings_height = if hero_ratings.is_empty() {
                0.0
            } else {
                style.metadata_height * 1.3
            };
            let metadata_height = if home.eyebrow.is_empty() {
                0.0
            } else {
                style.metadata_height
            };
            let synopsis_size = style.synopsis_size;
            let synopsis_width = style.synopsis_width();
            let synopsis_color = crate::stylesheet::color("text");
            let synopsis_galley = if !style.shows_synopsis || hero.description.is_empty() {
                None
            } else {
                let mut job = crate::gfx::text::LayoutJob::simple(
                    hero.description.clone(),
                    crate::fonts::regular(synopsis_size),
                    synopsis_color,
                    synopsis_width,
                );
                job.wrap.max_rows = style.synopsis_rows;
                job.wrap.overflow_character = Some('…');
                Some(context.fonts_mut(|fonts| fonts.layout_job(job)))
            };
            let synopsis_height = synopsis_galley
                .as_ref()
                .map_or(0.0, |galley| galley.size().y);
            let trailer_playing = home
                .trailer
                .as_ref()
                .filter(|trailer| hero.item_id.as_deref() == Some(trailer.item_id.as_str()))
                .is_some_and(|trailer| trailer.texture.is_some());
            let text_fade = 1.0
                - crate::motion::toggle(
                    context,
                    Id::new("fluxa-hero-text-fade"),
                    trailer_playing,
                    crate::motion::slow(),
                );
            let metadata_gap = style.metadata_gap.px(&metrics) * text_fade;
            let metadata_height = metadata_height * text_fade;
            let ratings_height = ratings_height * text_fade;
            let synopsis_height = synopsis_height * text_fade;
            let play_height = if pinned { 0.0 } else { style.button_height };
            let block_id = Id::new("fluxa-hero-block");
            let block_height = context
                .data(|data| data.get_temp::<f32>(block_id))
                .unwrap_or(
                    title_height + metadata_height + ratings_height + synopsis_height + play_height,
                );
            let content_top = (hero_height - style.bottom_padding - block_height)
                .max(style.min_top)
                - hero_scroll
                + hero_slide_offset;
            let hero_block = components::placed(
                context,
                Id::new("fluxa-shared-hero"),
                Pos2::new(margin, content_top),
                |ui| {
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
                    let body = ui.available_rect_before_wrap();
                    crate::kit::at(ui, body, |ui| {
                        if home.hero_slides.len() > 1 {
                            let next =
                                &home.hero_slides[(active_hero_index + 1) % home.hero_slides.len()];
                            assets.texture_for(
                                next.logo_url.as_deref(),
                                artwork_target_size(max_logo_size, context.pixels_per_point()),
                                ArtworkPriority::Hero,
                            );
                        }
                        let show_eyebrow = !hero.eyebrow.is_empty();
                        let show_ratings = !hero_ratings.is_empty() && text_fade > 0.05;
                        let show_synopsis = synopsis_galley.is_some();
                        let mut stack = Vec::new();
                        stack.push(crate::flex::band(title_height).id(0));
                        if titled {
                            stack.push(crate::flex::band(style.title_gap.px(&metrics)).id(10));
                        }
                        if show_eyebrow {
                            stack.push(crate::flex::band(metadata_height).id(1));
                        }
                        if show_ratings {
                            stack.push(crate::flex::band(ratings_height).id(2));
                        }
                        stack.push(crate::flex::band(metadata_gap).id(11));
                        if show_synopsis {
                            stack.push(crate::flex::band(synopsis_height).id(3));
                        }
                        let origin = ui.max_rect().min;
                        let column = crate::flex::column(stack).width(hero_width).solve(
                            crate::gfx::rect_at(origin, Vec2::new(hero_width, f32::MAX / 4.0)),
                        );
                        let column_height = column
                            .find(if show_synopsis { 3 } else { 11 })
                            .map_or(0.0, |rect| rect.bottom() - origin.y);
                        let title_rect = column.find(0).unwrap_or(Rect::NOTHING);
                        let title_anchor = if style.centered {
                            crate::gfx::Align2::CENTER_TOP
                        } else {
                            crate::gfx::Align2::LEFT_TOP
                        };
                        let logo_anchor = if style.centered {
                            crate::gfx::Align2::CENTER_CENTER
                        } else {
                            crate::gfx::Align2::LEFT_CENTER
                        };
                        if !titled {
                        } else if has_logo {
                            if let Some((texture, size)) = fitted_logo {
                                let logo_rect =
                                    crate::flex::anchored(title_rect, logo_anchor, size, 0.0);
                                ui.painter()
                                    .image(texture, logo_rect, full_uv(), Color32::WHITE);
                            } else {
                                components::failed_dom_image(
                                    ui.painter(),
                                    assets,
                                    crate::flex::anchored(
                                        title_rect,
                                        logo_anchor,
                                        max_logo_size,
                                        0.0,
                                    ),
                                    0.0,
                                    hero.logo_url.as_deref(),
                                    artwork_target_size(max_logo_size, context.pixels_per_point()),
                                    crate::ImageFit::Contain,
                                    [if style.centered { 0.5 } else { 0.0 }, 0.5],
                                );
                            }
                        } else {
                            crate::flex::paint_text(
                                ui.painter(),
                                title_rect,
                                &hero.title,
                                &crate::fonts::bold(style.title_size),
                                text_primary(),
                                title_anchor,
                            );
                        }
                        if show_eyebrow {
                            let metadata_rect = column.find(1).unwrap_or(Rect::NOTHING);
                            let text_rect = draw_text(
                                ui.painter(),
                                if style.centered {
                                    metadata_rect.center_top()
                                } else {
                                    metadata_rect.left_top()
                                },
                                if style.centered {
                                    crate::gfx::Align2::CENTER_TOP
                                } else {
                                    crate::gfx::Align2::LEFT_TOP
                                },
                                truncate_text(&hero.eyebrow, style.metadata_chars),
                                crate::gfx::FontId::proportional(style.metadata_size),
                                metrics.text_secondary.gamma_multiply(text_fade),
                                None,
                            );
                            if text_fade > 0.05 {
                                let mut chip_painter = ui.painter().clone();
                                chip_painter.multiply_opacity(text_fade);
                                let font_size = style.metadata_size * 0.85;
                                let chip_gap = Space::S6.px(&metrics);
                                let lead = Space::S12.px(&metrics);
                                let mut left = text_rect.right() + lead;
                                let mut chips = Vec::new();
                                for badge in hero.badges.iter().filter(|badge| badge.essential) {
                                    let (galley, size) = components::badge_chip_layout(
                                        &chip_painter,
                                        badge,
                                        font_size,
                                    );
                                    if left + size.x > metadata_rect.right() {
                                        break;
                                    }
                                    left += size.x + chip_gap;
                                    chips.push((badge, galley, size));
                                }
                                let nodes = chips
                                    .iter()
                                    .enumerate()
                                    .map(|(index, (_, _, size))| crate::flex::leaf(*size).id(index))
                                    .collect::<Vec<_>>();
                                let placed = crate::flex::row(nodes)
                                    .gap(chip_gap)
                                    .align(crate::flex::Align::Center)
                                    .solve(Rect::from_min_max(
                                        Pos2::new(text_rect.right() + lead, text_rect.top()),
                                        Pos2::new(metadata_rect.right(), text_rect.bottom()),
                                    ));
                                for (index, (badge, galley, _)) in chips.into_iter().enumerate() {
                                    if let Some(chip) = placed.find(index) {
                                        components::paint_badge_chip_rounded(
                                            &chip_painter,
                                            chip,
                                            galley,
                                            badge,
                                            assets,
                                            Dp::D3.px(),
                                        );
                                    }
                                }
                            }
                        }
                        if show_ratings {
                            let ratings_rect = column.find(2).unwrap_or(Rect::NOTHING);
                            let mut row = ui.new_child(
                                crate::gfx::UiBuilder::new().max_rect(ratings_rect).layout(
                                    crate::gfx::Layout::left_to_right(crate::gfx::Align::Center)
                                        .with_main_align(if style.centered {
                                            crate::gfx::Align::Center
                                        } else {
                                            crate::gfx::Align::Min
                                        }),
                                ),
                            );
                            row.set_opacity(text_fade);
                            row.set_item_spacing_x(4.0);
                            components::rating_row(
                                &mut row,
                                &hero_ratings,
                                style.metadata_size,
                                text_primary(),
                                metrics.text_secondary,
                                Sense::hover(),
                                assets,
                            );
                        }
                        if let Some(galley) = synopsis_galley.as_ref() {
                            let synopsis_rect = column.find(3).unwrap_or(Rect::NOTHING);
                            let synopsis_at = crate::flex::anchored(
                                synopsis_rect,
                                if style.centered {
                                    crate::gfx::Align2::CENTER_TOP
                                } else {
                                    crate::gfx::Align2::LEFT_TOP
                                },
                                galley.size(),
                                0.0,
                            );
                            ui.painter().galley_with_override_text_color(
                                synopsis_at.min,
                                galley.clone(),
                                synopsis_color.gamma_multiply(text_fade),
                            );
                        }
                        crate::kit::lay(
                            ui,
                            vec![
                                crate::kit::item(Vec2::new(hero_width, column_height), |_, _| {})
                                    .margin(crate::flex::Insets {
                                        bottom: style.button_gap.px(&metrics),
                                        ..Default::default()
                                    }),
                            ],
                        );
                        if !pinned {
                            let resume = hero.item_id.as_deref().and_then(|id| home.resume_for(id));
                            let series =
                                matches!(hero.item_type.as_deref(), Some("series" | "tv" | "show"));
                            let label = home.playback_unavailable.clone().unwrap_or_else(|| {
                                let first = series
                                    .then(|| first_episode(&hero.raw))
                                    .flatten()
                                    .unwrap_or((1, 1, String::new()));
                                play_label(
                                    &home.language,
                                    resume,
                                    series.then_some((first.0, first.1, Some(first.2.as_str()))),
                                )
                            });
                            let text_size = metrics.nav_label_size + 2.0;
                            let measure = |ui: &crate::gfx::Ui, text: &str| {
                                ui.painter()
                                    .layout_no_wrap(
                                        text.to_owned(),
                                        crate::gfx::FontId::proportional(text_size),
                                        text_primary(),
                                    )
                                    .size()
                                    .x
                            };
                            let play_width = (measure(ui, &label) + 70.0).min(320.0);
                            let watchlist_label = localized("library.watchlist", &home.language);
                            let slots = fluxa_theme::user::hero_actions();
                            let total: f32 = slots
                                .iter()
                                .map(|slot| {
                                    if slot == "play" {
                                        play_width
                                    } else {
                                        play_height
                                    }
                                })
                                .sum::<f32>()
                                + Space::S12.px(&metrics) * slots.len().saturating_sub(1) as f32;
                            let actions = view().class("home-actions").row();
                            let actions = if style.centered {
                                actions.center_x().max_width(total)
                            } else {
                                actions
                            };
                            actions.show(ui, |ui| {
                                for slot in &slots {
                                    if slot == "play" {
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
                                    } else {
                                        let saved = hero
                                            .item_id
                                            .as_deref()
                                            .and_then(|id| {
                                                poster_overlay::personal_for(ui.ctx(), id)
                                            })
                                            .is_some_and(|personal| personal.saved);
                                        let watchlist = components::icon_button_sized(
                                            ui,
                                            assets.icon(if saved { "Check" } else { "Plus" }),
                                            Vec2::splat(play_height),
                                            text_primary(),
                                            true,
                                            saved,
                                            true,
                                        )
                                        .on_hover_text(&watchlist_label);
                                        hero_watchlist = Some(watchlist.rect);
                                        if watchlist.clicked() {
                                            activated = Some(NODE_HERO_WATCHLIST);
                                        }
                                    }
                                }
                            });
                        }
                    });
                },
            );
            let measured = hero_block.response.rect.height();
            if (measured - block_height).abs() > 0.5 {
                context.data_mut(|data| data.insert_temp(block_id, measured));
                context.request_repaint();
            }
        }
        ui.allocate_space(Vec2::new(viewport.width, rows_lead));
        draw_home_rows(
            ui,
            viewport,
            home,
            assets,
            focused,
            metrics,
            rows_tail,
            prefetch_home_artwork,
            margin,
            rows_screen,
            &mut layout,
            &mut activated,
        );
    });
    let hero_click = context.input(|input| {
        input
            .pointer()
            .primary_clicked()
            .then(|| input.pointer().interact_pos())
            .flatten()
    });
    if activated.is_none()
        && show_hero
        && hero_has_item
        && !compact
        && !pinned
        && hero_click.is_some_and(|pos| pos.y > 80.0 && hero_rect.intersect(screen).contains(pos))
    {
        activated = Some(NODE_MORE_INFO);
    }
    layout.activated = activated;
    layout.focusable.extend(navigation_focus_rects(
        home_nav_viewport(viewport, home),
        metrics,
    ));
    draw_category_bar(context, viewport, home, metrics, &mut layout);
    if show_hero && hero_has_item && !pinned {
        layout
            .focusable
            .push((NODE_MORE_INFO, hero_rect.intersect(screen)));
    }
    if let Some(watchlist) = hero_watchlist {
        layout.focusable.push((NODE_HERO_WATCHLIST, watchlist));
    }
    if let Some(play) = hero_actions {
        layout.focusable.push((NODE_PLAY, play));
    } else if show_hero
        && hero_has_item
        && !pinned
        && fluxa_theme::user::hero_actions()
            .iter()
            .any(|slot| slot == "play")
    {
        let play = style.fallback_play(metrics, viewport.width, margin);
        layout.focusable.push((
            NODE_PLAY,
            play.translate(Vec2::new(0.0, flow.hero_top + hero_height - shown_scroll)),
        ));
    }
    layout
}

#[derive(Clone)]
struct FocusHeroMemo(HomeHero);

pub fn pinned_focus_hero(context: &crate::gfx::Context) -> Option<HomeHero> {
    let pinned = context
        .data(|data| data.get_temp::<bool>(Id::new("fluxa-hero-pinned")))
        .unwrap_or(false);
    pinned
        .then(|| {
            context.data(|data| data.get_temp::<FocusHeroMemo>(Id::new("fluxa-focus-hero-memo")))
        })
        .flatten()
        .map(|memo| memo.0)
}

fn focus_backdrop(card: &HomeCard) -> Option<String> {
    card.backdrop_url
        .clone()
        .or_else(|| card.artwork_url.clone())
}

fn focus_hero(
    context: &crate::gfx::Context,
    home: &HomeModel,
    focused: Option<u64>,
) -> Option<HomeHero> {
    let memo_id = Id::new("fluxa-focus-hero-memo");
    let card = focused
        .filter(|node| (NODE_CARD_BASE..NODE_PREVIEW_BASE).contains(node))
        .and_then(|node| home.card_at((node - NODE_CARD_BASE) as usize))
        .filter(|card| card.id.is_some());
    let Some(card) = card else {
        return context.data(|data| data.get_temp::<FocusHeroMemo>(memo_id).map(|memo| memo.0));
    };
    let (mut eyebrow, description) = card_summary(card, &home.language);
    if eyebrow.is_empty() {
        eyebrow = card.subtitle.clone();
    }
    let hero = HomeHero {
        title: card.title.clone(),
        eyebrow,
        description,
        background_url: focus_backdrop(card),
        logo_url: card.logo_url.clone(),
        item_id: card.id.clone(),
        item_type: card.item_type.clone(),
        trailers: trailer_urls(&card.raw),
        raw: card.raw.cloned(),
        badges: Vec::new(),
    };
    context.data_mut(|data| data.insert_temp(memo_id, FocusHeroMemo(hero.clone())));
    Some(hero)
}

fn mostly_visible(rect: Rect, viewport: Rect) -> bool {
    let visible = rect.intersect(viewport);
    visible.is_positive() && visible.area() >= rect.area() * 0.6
}

pub(crate) fn lane(bounds: Rect, padding: crate::flex::Insets) -> Rect {
    crate::flex::stack(Align2::LEFT_TOP, [crate::flex::fill().id(0)])
        .padding(padding)
        .solve(bounds)
        .find(0)
        .unwrap_or(bounds)
}
