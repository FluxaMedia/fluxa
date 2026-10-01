use super::*;

#[derive(Clone, Debug, Default)]
pub struct ShortsModel {
    pub items: Vec<HomeHero>,
    pub index: usize,
    pub trailer: Option<HeroTrailer>,
    pub loading: bool,
    pub language: String,
}

pub fn shorts_feed(home: &HomeModel) -> Vec<HomeHero> {
    let cards = home
        .rows
        .iter()
        .find(|row| row.kind == HomeRowKind::Poster && !row.cards.is_empty())
        .map(|row| row.cards.as_slice())
        .unwrap_or(home.cards.as_slice());
    cards
        .iter()
        .filter(|card| card.id.is_some() && card.item_type.is_some() && card.raw.is_object())
        .take(30)
        .map(|card| core_home_hero(&card.raw, &home.language))
        .collect()
}

const SWIPE_DISTANCE: f32 = 60.0;
const WHEEL_COOLDOWN: f64 = 0.55;

fn page_column(viewport: Viewport) -> Rect {
    let width = if viewport.is_compact() {
        viewport.width
    } else {
        (viewport.height * 9.0 / 16.0).min(viewport.width)
    };
    Rect::from_min_size(
        Pos2::new((viewport.width - width) * 0.5, 0.0),
        Vec2::new(width, viewport.height),
    )
}

fn step_from_input(context: &egui::Context, column: Rect) -> Option<i32> {
    let swipe_id = Id::new("fluxa-shorts-swipe");
    let wheel_id = Id::new("fluxa-shorts-wheel");
    let (pressed, released, position, wheel, now) = context.input(|input| {
        (
            input.pointer.primary_pressed(),
            input.pointer.primary_released(),
            input.pointer.interact_pos(),
            input.smooth_scroll_delta.y,
            input.time,
        )
    });
    if pressed {
        let start = position.filter(|position| column.contains(*position));
        context.data_mut(|data| data.insert_temp(swipe_id, start));
    }
    if released {
        let start = context.data_mut(|data| data.remove_temp::<Option<Pos2>>(swipe_id));
        if let (Some(Some(start)), Some(end)) = (start, position) {
            let delta = end - start;
            if delta.y.abs() > SWIPE_DISTANCE && delta.y.abs() > delta.x.abs() * 1.5 {
                return Some(if delta.y < 0.0 { 1 } else { -1 });
            }
        }
    }
    if wheel.abs() > 1.0 && position.is_some_and(|position| column.contains(position)) {
        let last = context
            .data(|data| data.get_temp::<f64>(wheel_id))
            .unwrap_or(0.0);
        if now - last > WHEEL_COOLDOWN {
            context.data_mut(|data| data.insert_temp(wheel_id, now));
            return Some(if wheel < 0.0 { 1 } else { -1 });
        }
    }
    None
}

pub fn draw_shorts(
    context: &egui::Context,
    viewport: Viewport,
    shorts: &ShortsModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
    let scale = if viewport.is_tv() { 1.35 } else { 1.0 };
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    painter.rect_filled(screen, 0.0, metrics.background);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 3, assets);
    if !compact {
        layout
            .focusable
            .extend(navigation_focus_rects(viewport, metrics));
    }
    if shorts.items.is_empty() {
        if shorts.loading {
            draw_loading_screen(context, &painter, screen, assets);
        } else {
            painter.text(
                screen.center(),
                Align2::CENTER_CENTER,
                localized("shorts.empty", &shorts.language),
                FontId::proportional(metrics.nav_label_size + 2.0),
                metrics.text_secondary,
            );
        }
        return layout;
    }

    let column = page_column(viewport);
    let pad = 20.0 * scale;
    let button_height = if compact { 42.0 } else { 46.0 } * scale;
    let bottom_pad = if compact { 104.0 } else { 44.0 * scale };
    let button_top = viewport.height - bottom_pad - button_height;
    let stack_inset = viewport.height - button_top + 16.0 * scale;
    let index = shorts.index.min(shorts.items.len() - 1);
    layout.short_step = step_from_input(context, column);
    let position =
        context.animate_value_with_time(Id::new("fluxa-shorts-position"), index as f32, 0.3);
    let first = position.floor().max(0.0) as usize;
    let last = (position.ceil().max(0.0) as usize).min(shorts.items.len() - 1);
    for page in first..=last {
        let offset = (page as f32 - position) * viewport.height;
        let rect = column.translate(Vec2::new(0.0, offset));
        let playing = page == index;
        draw_page(
            context,
            &painter,
            rect,
            &shorts.items[page],
            playing.then_some(shorts.trailer.as_ref()).flatten(),
            metrics,
            scale,
            stack_inset,
            assets,
        );
    }
    for neighbor in [index.checked_sub(1), Some(index + 1)]
        .into_iter()
        .flatten()
    {
        if let Some(hero) = shorts.items.get(neighbor) {
            assets.prefetch_for(
                hero.background_url.as_deref(),
                artwork_target_size(column.size(), context.pixels_per_point()),
                ArtworkPriority::Prefetch,
            );
        }
    }

    let hero = &shorts.items[index];
    let settled = (position - index as f32).abs() < 0.02;
    if settled {
        egui::Area::new(Id::new("fluxa-shorts-actions"))
            .constrain(false)
            .fixed_pos(Pos2::new(column.left() + pad, button_top))
            .show(context, |ui| {
                ui.spacing_mut().item_spacing.x = 12.0 * scale;
                ui.horizontal(|ui| {
                    let text_size = metrics.nav_label_size + 2.0;
                    let label = play_label(&shorts.language, None, None);
                    let play = components::play_button(
                        ui,
                        assets,
                        &label,
                        Some(150.0 * scale),
                        button_height,
                        text_size,
                        None,
                    );
                    if play.clicked() {
                        layout.activated = Some(NODE_SHORTS_PLAY);
                    }
                    layout.focusable.push((NODE_SHORTS_PLAY, play.rect));
                    let saved = hero
                        .item_id
                        .as_deref()
                        .and_then(|id| poster_overlay::personal_for(ui.ctx(), id))
                        .is_some_and(|personal| personal.saved);
                    let watchlist = components::icon_button_sized(
                        ui,
                        assets.icon(if saved { "Check" } else { "Plus" }),
                        Vec2::splat(button_height),
                        Color32::WHITE,
                        true,
                        saved,
                        true,
                    )
                    .on_hover_text(localized("library.watchlist", &shorts.language));
                    if watchlist.clicked() {
                        layout.activated = Some(NODE_SHORTS_WATCHLIST);
                    }
                    layout
                        .focusable
                        .push((NODE_SHORTS_WATCHLIST, watchlist.rect));
                    let info = components::icon_button_sized(
                        ui,
                        assets.icon("Info"),
                        Vec2::splat(button_height),
                        Color32::WHITE,
                        true,
                        false,
                        true,
                    )
                    .on_hover_text(localized("shorts.more_info", &shorts.language));
                    if info.clicked() {
                        layout.activated = Some(NODE_SHORTS_INFO);
                    }
                    layout.focusable.push((NODE_SHORTS_INFO, info.rect));
                });
            });
    }

    if !compact && !viewport.is_tv() {
        let size = Vec2::splat(44.0);
        let x = column.right() + 20.0;
        if x + size.x < viewport.width {
            for (step, icon, y) in [
                (-1, "ChevronUp", screen.center().y - size.y - 6.0),
                (1, "ChevronDown", screen.center().y + 6.0),
            ] {
                let enabled = if step < 0 {
                    index > 0
                } else {
                    index + 1 < shorts.items.len()
                };
                egui::Area::new(Id::new(("fluxa-shorts-step", step)))
                    .constrain(false)
                    .fixed_pos(Pos2::new(x, y))
                    .show(context, |ui| {
                        let response = components::icon_button_sized(
                            ui,
                            assets.icon(icon),
                            size,
                            Color32::WHITE,
                            true,
                            false,
                            enabled,
                        );
                        if response.clicked() {
                            layout.short_step = Some(step);
                        }
                    });
            }
        }
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

fn draw_page(
    context: &egui::Context,
    painter: &egui::Painter,
    rect: Rect,
    hero: &HomeHero,
    trailer: Option<&HeroTrailer>,
    metrics: UiMetrics,
    scale: f32,
    stack_inset: f32,
    assets: &mut impl HomeAssets,
) {
    let clipped = painter.with_clip_rect(rect);
    if let Some(texture) = assets.texture_for(
        hero.background_url.as_deref(),
        artwork_target_size(rect.size(), context.pixels_per_point()),
        ArtworkPriority::Hero,
    ) {
        let size = assets
            .texture_size(hero.background_url.as_deref())
            .unwrap_or([1920, 1080]);
        clipped.image(
            texture,
            rect,
            cover_uv(size, rect),
            Color32::from_white_alpha(150),
        );
    }
    if let Some(texture) = trailer
        .filter(|trailer| hero.item_id.as_deref() == Some(trailer.item_id.as_str()))
        .and_then(|trailer| trailer.texture)
    {
        paint_trailer(
            context,
            &clipped,
            texture,
            rect,
            hero.item_id.as_deref(),
            255,
        );
    }
    let scrim = metrics.background.gamma_multiply(0.92);
    paint_vertical_gradient(
        &clipped,
        Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + rect.height() * 0.5),
            Pos2::new(rect.right(), rect.top() + rect.height() * 0.84),
        ),
        Color32::TRANSPARENT,
        scrim,
    );
    paint_vertical_gradient(
        &clipped,
        Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + rect.height() * 0.84),
            rect.right_bottom(),
        ),
        scrim,
        metrics.background,
    );

    let pad = 20.0 * scale;
    let width = rect.width() - pad * 2.0;
    let mut y = rect.bottom() - stack_inset;

    if !hero.description.is_empty() {
        let mut job = egui::text::LayoutJob::simple(
            hero.description.clone(),
            crate::fonts::regular(14.0 * scale),
            Color32::from_white_alpha(235),
            width,
        );
        job.wrap.max_rows = 3;
        job.wrap.overflow_character = Some('…');
        let galley = context.fonts_mut(|fonts| fonts.layout_job(job));
        y -= galley.size().y;
        clipped.galley(Pos2::new(rect.left() + pad, y), galley, Color32::WHITE);
        y -= 10.0 * scale;
    }
    if !hero.eyebrow.is_empty() {
        y -= 18.0 * scale;
        clipped.text(
            Pos2::new(rect.left() + pad, y),
            Align2::LEFT_TOP,
            truncate_text(&hero.eyebrow, 60),
            FontId::proportional(13.0 * scale),
            metrics.text_secondary,
        );
        y -= 12.0 * scale;
    }
    let logo_height = 72.0 * scale;
    let has_logo = hero
        .logo_url
        .as_deref()
        .is_some_and(|url| !url.trim().is_empty());
    let bounds = Vec2::new(width * 0.7, logo_height);
    let logo = has_logo
        .then(|| {
            components::title_logo(
                context.pixels_per_point(),
                hero.logo_url.as_deref(),
                bounds,
                ArtworkPriority::Hero,
                assets,
            )
        })
        .flatten();
    y -= logo_height;
    match logo {
        Some((texture, size)) => {
            let logo_rect =
                Rect::from_min_size(Pos2::new(rect.left() + pad, y + logo_height - size.y), size);
            clipped.image(texture, logo_rect, full_uv(), Color32::WHITE);
        }
        None => {
            clipped.text(
                Pos2::new(rect.left() + pad, y + logo_height),
                Align2::LEFT_BOTTOM,
                truncate_text(&hero.title, 28),
                FontId::proportional(28.0 * scale),
                Color32::WHITE,
            );
        }
    }
}
