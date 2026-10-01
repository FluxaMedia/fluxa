use super::*;

mod hero;
mod layout;
mod metrics;
mod rows;
use hero::HeroStyle;
pub use layout::*;
pub use metrics::*;
pub(crate) use rows::*;

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
    let metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
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
    let style = HeroStyle::new(viewport, metrics, metrics.screen_margin);
    let hero_fade_height = style.fade_height;
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
                255,
            );
        }
        paint_hero_scrim(&painter, hero_visual_rect, compact, metrics.background);
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
                input.pointer.primary_pressed(),
                input.pointer.primary_released(),
                input.pointer.interact_pos(),
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
    let margin = metrics.screen_margin;
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
                FontId::proportional(metrics.screen_title_size),
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
    let hero_width = style.width;
    if show_hero {
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
        let title_height = if let Some((_, size)) = fitted_logo {
            size.y
        } else if has_logo {
            style.logo_height
        } else {
            style.text_title_height
        };
        let metadata_height = if home.eyebrow.is_empty() {
            0.0
        } else {
            style.metadata_height
        };
        let synopsis_size = style.synopsis_size;
        let synopsis_width = style.synopsis_width();
        let synopsis_color = Color32::from_white_alpha(240);
        let synopsis_galley = if !style.shows_synopsis || hero.description.is_empty() {
            None
        } else {
            let mut job = egui::text::LayoutJob::simple(
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
        let play_height = style.button_height;
        let block_height =
            title_height + metadata_height + synopsis_height + play_height + style.stack_extra;
        let content_top = (hero_height - style.bottom_padding - block_height).max(style.min_top)
            - scroll_offset
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
                    if home.hero_slides.len() > 1 {
                        let next =
                            &home.hero_slides[(active_hero_index + 1) % home.hero_slides.len()];
                        assets.texture_for(
                            next.logo_url.as_deref(),
                            artwork_target_size(max_logo_size, context.pixels_per_point()),
                            ArtworkPriority::Hero,
                        );
                    }
                    let title_rect = ui
                        .allocate_exact_size(Vec2::new(hero_width, title_height), Sense::hover())
                        .0;
                    let title_anchor = if style.centered {
                        egui::Align2::CENTER_TOP
                    } else {
                        egui::Align2::LEFT_TOP
                    };
                    if has_logo {
                        if let Some((texture, size)) = fitted_logo {
                            let logo_x = if style.centered {
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
                        let title = truncate_text(&hero.title, style.title_chars);
                        ui.painter().text(
                            title_rect.min,
                            title_anchor,
                            title,
                            egui::FontId::proportional(style.title_size),
                            Color32::WHITE,
                        );
                    }
                    ui.add_space(style.title_gap);
                    if !hero.eyebrow.is_empty() {
                        let metadata_rect = ui
                            .allocate_exact_size(
                                Vec2::new(hero_width, metadata_height),
                                Sense::hover(),
                            )
                            .0;
                        ui.painter().text(
                            if style.centered {
                                metadata_rect.center_top()
                            } else {
                                metadata_rect.left_top()
                            },
                            if style.centered {
                                egui::Align2::CENTER_TOP
                            } else {
                                egui::Align2::LEFT_TOP
                            },
                            truncate_text(&hero.eyebrow, style.metadata_chars),
                            egui::FontId::proportional(style.metadata_size),
                            metrics.text_secondary,
                        );
                    }
                    ui.add_space(style.metadata_gap);
                    if let Some(galley) = synopsis_galley.as_ref() {
                        let synopsis_rect = ui
                            .allocate_exact_size(
                                Vec2::new(synopsis_width, synopsis_height),
                                Sense::hover(),
                            )
                            .0;
                        let synopsis_x = if style.centered {
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
                    ui.add_space(style.button_gap);
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
                        let play_width = (measure(ui, &label) + 70.0).min(320.0);
                        let watchlist_label = localized("library.watchlist", &home.language);
                        if style.centered {
                            let total = play_width + 12.0 + play_height;
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
                        let watchlist = components::icon_button_sized(
                            ui,
                            assets.icon(if saved { "Check" } else { "Plus" }),
                            Vec2::splat(play_height),
                            Color32::WHITE,
                            true,
                            saved,
                            true,
                        )
                        .on_hover_text(&watchlist_label);
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
    draw_home_rows(
        context,
        viewport,
        home,
        assets,
        focused,
        metrics,
        scroll_offset,
        row_start,
        prefetch_home_artwork,
        margin,
        screen,
        &mut layout,
        &mut activated,
    );
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
        let play = style.fallback_play(metrics, viewport.width, margin);
        layout.focusable.push((
            NODE_PLAY,
            play.translate(Vec2::new(0.0, hero_height - scroll_offset)),
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
