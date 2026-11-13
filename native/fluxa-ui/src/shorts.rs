use super::*;
use crate::dp::Dp;
use crate::draw_text;

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

fn page_column(viewport: Viewport) -> Rect {
    let metrics = UiMetrics::for_viewport(viewport);
    let width = if viewport.is_compact() {
        viewport.width
    } else {
        (viewport.height * metrics.screen_shorts_aspect).min(viewport.width)
    };
    let screen = crate::gfx::rect_at(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    crate::flex::anchored(
        screen,
        Align2::CENTER_TOP,
        Vec2::new(width, viewport.height),
        0.0,
    )
}

fn step_from_input(context: &crate::gfx::Context, column: Rect) -> Option<i32> {
    let swipe_id = Id::new("fluxa-shorts-swipe");
    let wheel_id = Id::new("fluxa-shorts-wheel");
    let metrics = crate::metrics::active();
    let (pressed, released, position, wheel, now) = context.input(|input| {
        (
            input.pointer().primary_pressed(),
            input.pointer().primary_released(),
            input.pointer().interact_pos(),
            input.smooth_scroll_delta().y,
            input.time(),
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
            if delta.y.abs() > Dp::D56.px() && delta.y.abs() > delta.x.abs() * 1.5 {
                return Some(if delta.y < 0.0 { 1 } else { -1 });
            }
        }
    }
    if wheel.abs() > 1.0 && position.is_some_and(|position| column.contains(position)) {
        let last = context
            .data(|data| data.get_temp::<f64>(wheel_id))
            .unwrap_or(0.0);
        if now - last > f64::from(metrics.screen_shorts_wheel_cooldown) {
            context.data_mut(|data| data.insert_temp(wheel_id, now));
            return Some(if wheel < 0.0 { 1 } else { -1 });
        }
    }
    None
}

pub fn draw_shorts(
    context: &crate::gfx::Context,
    viewport: Viewport,
    shorts: &ShortsModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
    let scale = metrics.shorts_scale;
    let screen = crate::gfx::rect_at(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(crate::gfx::LayerId::background());
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
            components::skeleton_card(&painter, page_column(viewport), metrics);
        } else {
            draw_text(
                &painter,
                screen.center(),
                Align2::CENTER_CENTER,
                localized("shorts.empty", &shorts.language),
                FontId::proportional(metrics.nav_label_size + 2.0),
                metrics.text_secondary,
                None,
            );
        }
        return layout;
    }

    let column = page_column(viewport);
    let pad = Dp::D20.px() * scale;
    let button_height = metrics.shorts_button_height * scale;
    let bottom_pad = if compact {
        metrics.screen_shorts_bottom_pad_compact
    } else {
        Dp::D44.px() * scale
    };
    let button_top = viewport.height - bottom_pad - button_height;
    let stack_inset = viewport.height - button_top + Dp::D16.px() * scale;
    let index = shorts.index.min(shorts.items.len() - 1);
    layout.short_step = step_from_input(context, column);
    let position = crate::motion::ease(
        context,
        Id::new("fluxa-shorts-position"),
        index as f32,
        crate::motion::slow(),
        crate::motion::Ease::Linear,
    );
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
        components::placed(
            context,
            Id::new("fluxa-shorts-actions"),
            Pos2::new(column.left() + pad, button_top),
            |ui| {
                kit::view()
                    .class("shorts-actions")
                    .row()
                    .hug()
                    .show(ui, |ui| {
                        let text_size = metrics.nav_label_size + 2.0;
                        let label = play_label(&shorts.language, None, None);
                        let play = components::play_button(
                            ui,
                            assets,
                            &label,
                            Some(metrics.screen_shorts_play_width * scale),
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
                            text_primary(),
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
                            text_primary(),
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
            },
        );
    }

    if !compact && !viewport.is_tv() {
        let size = Vec2::splat(Dp::D44.px());
        let x = column.right() + Space::S20.px(&metrics);
        if x + size.x < viewport.width {
            let steps = crate::flex::column([
                crate::flex::leaf(size).id(0),
                crate::flex::leaf(Vec2::new(0.0, Space::S12.px(&metrics))),
                crate::flex::leaf(size).id(1),
            ])
            .justify(crate::flex::Justify::Center)
            .align(crate::flex::Align::Start)
            .solve(crate::gfx::rect_at(
                Pos2::new(x, 0.0),
                Vec2::new(size.x, viewport.height),
            ));
            for (step, icon, slot) in [(-1, "ChevronUp", 0), (1, "ChevronDown", 1)] {
                let y = steps.find(slot).map_or(0.0, |rect| rect.top());
                let enabled = if step < 0 {
                    index > 0
                } else {
                    index + 1 < shorts.items.len()
                };
                components::placed(
                    context,
                    Id::new(("fluxa-shorts-step", step)),
                    Pos2::new(x, y),
                    |ui| {
                        let response = components::icon_button_sized(
                            ui,
                            assets.icon(icon),
                            size,
                            text_primary(),
                            true,
                            false,
                            enabled,
                        );
                        if response.clicked() {
                            layout.short_step = Some(step);
                        }
                    },
                );
            }
        }
    }

    layout
}

fn draw_page(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
    rect: Rect,
    hero: &HomeHero,
    trailer: Option<&HeroTrailer>,
    metrics: UiMetrics,
    scale: f32,
    stack_inset: f32,
    assets: &mut impl HomeAssets,
) {
    let clipped = painter.with_clip_rect(rect);
    let target = artwork_target_size(rect.size(), context.pixels_per_point());
    if let Some(texture) = assets.texture_for(
        hero.background_url.as_deref(),
        target,
        ArtworkPriority::Hero,
    ) {
        let size = assets
            .texture_size(hero.background_url.as_deref())
            .unwrap_or([1920, 1080]);
        clipped.image(
            texture,
            rect,
            cover_uv(size, rect),
            crate::stylesheet::color("text.subtle"),
        );
    } else {
        crate::components::failed_dom_image(
            &clipped,
            assets,
            rect,
            0.0,
            hero.background_url.as_deref(),
            target,
            crate::ImageFit::Cover,
            [0.5, 0.5],
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
    let scrim = metrics
        .background
        .gamma_multiply(metrics.screen_shorts_scrim_alpha);
    let fade_start = metrics.screen_shorts_fade_start;
    let fade_span = metrics.screen_shorts_fade_span;
    let bands = crate::flex::column([
        crate::flex::fill().grow(fade_start),
        crate::flex::fill().grow(fade_span).id(0),
        crate::flex::fill().grow(1.0 - fade_start - fade_span).id(1),
    ])
    .align(crate::flex::Align::Start)
    .solve(rect);
    if let (Some(upper), Some(lower)) = (bands.find(0), bands.find(1)) {
        paint_vertical_gradient(&clipped, upper, Color32::TRANSPARENT, scrim);
        paint_vertical_gradient(&clipped, lower, scrim, metrics.background);
    }

    let pad = Dp::D20.px() * scale;
    let width = rect.width() - pad * 2.0;
    let logo_height = Dp::D72.px() * scale;
    let bounds = Vec2::new(width * metrics.screen_shorts_logo_width_ratio, logo_height);
    let logo = hero
        .logo_url
        .as_deref()
        .is_some_and(|url| !url.trim().is_empty())
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
    crate::gfx::Area::new(Id::new(("fluxa-shorts-stack", rect.min.y.to_bits())))
        .constrain(false)
        .order(crate::gfx::Order::Background)
        .fixed_pos(rect.min)
        .show(context, |ui| {
            ui.set_clip_rect(rect);
            let stack = crate::home::lane(
                rect,
                crate::flex::Insets {
                    left: pad,
                    top: 0.0,
                    right: pad,
                    bottom: stack_inset,
                },
            );
            let description = (!hero.description.is_empty()).then(|| {
                let mut job = crate::gfx::text::LayoutJob::simple(
                    hero.description.clone(),
                    crate::fonts::regular(Dp::D14.px() * scale),
                    crate::stylesheet::color("text"),
                    width,
                );
                job.wrap.max_rows = 3;
                job.wrap.overflow_character = Some('…');
                ui.painter().layout_job(job)
            });
            let eyebrow = (!hero.eyebrow.is_empty()).then(|| {
                ui.painter().layout_no_wrap(
                    truncate_text(&hero.eyebrow, 60),
                    FontId::proportional(Dp::D12.px() * scale),
                    metrics.text_secondary,
                )
            });
            let mut rows = vec![crate::flex::leaf(bounds).id(0)];
            if let Some(galley) = &eyebrow {
                rows.push(crate::flex::leaf(Vec2::new(0.0, Space::S12.px(&metrics))));
                rows.push(crate::flex::leaf(galley.size()).id(1));
            }
            if let Some(galley) = &description {
                rows.push(crate::flex::leaf(Vec2::new(0.0, Space::S10.px(&metrics))));
                rows.push(crate::flex::leaf(galley.size()).id(2));
            }
            let placed = crate::flex::column(rows)
                .justify(crate::flex::Justify::End)
                .align(crate::flex::Align::Start)
                .solve(stack);
            if let (Some(galley), Some(at)) = (eyebrow, placed.find(1)) {
                ui.painter().galley(at.min, galley, metrics.text_secondary);
            }
            if let (Some(galley), Some(at)) = (description, placed.find(2)) {
                ui.painter()
                    .galley(at.min, galley, crate::stylesheet::color("text"));
            }
            {
                let slot = placed.find(0).unwrap_or(stack);
                match logo {
                    Some((texture, size)) => {
                        let logo_rect = crate::flex::anchored(slot, Align2::LEFT_BOTTOM, size, 0.0);
                        ui.painter()
                            .image(texture, logo_rect, full_uv(), Color32::WHITE);
                    }
                    None if hero
                        .logo_url
                        .as_deref()
                        .is_some_and(|url| !url.trim().is_empty())
                        && crate::components::failed_dom_image(
                            ui.painter(),
                            assets,
                            slot,
                            0.0,
                            hero.logo_url.as_deref(),
                            artwork_target_size(bounds, context.pixels_per_point()),
                            crate::ImageFit::Contain,
                            [0.0, 1.0],
                        ) => {}
                    None => {
                        draw_text(
                            ui.painter(),
                            slot.left_bottom(),
                            Align2::LEFT_BOTTOM,
                            truncate_text(&hero.title, 28),
                            FontId::proportional(Dp::D28.px() * scale),
                            text_primary(),
                            None,
                        );
                    }
                }
            }
        });
}
