use super::*;
use crate::dp::Dp;
use crate::kit;
use std::cell::{Cell, RefCell};

mod episodes;

fn season_tile(metrics: UiMetrics) -> Vec2 {
    Vec2::new(
        metrics.screen_season_tile_width,
        metrics.screen_season_tile_height,
    )
}

fn trailer_size(compact: bool, metrics: UiMetrics) -> Vec2 {
    let width = if compact {
        metrics.detail_trailer_width_mobile
    } else {
        metrics.detail_trailer_width
    };
    Vec2::new(width, width * metrics.screen_video_aspect)
}

fn similar_size(metrics: UiMetrics) -> (Vec2, f32) {
    let poster = Vec2::new(metrics.poster_card_width, metrics.poster_card_height);
    let text = metrics.control_gap * 2.0
        + metrics.screen_card_title_size
        + metrics.screen_card_subtitle_size;
    (poster, poster.y + text)
}

fn compact_image_height(viewport: Viewport) -> f32 {
    let metrics = UiMetrics::for_viewport(viewport);
    (viewport.height * metrics.screen_detail_image_ratio).max(metrics.screen_detail_image_min)
}

fn hero_height(viewport: Viewport, metrics: UiMetrics) -> f32 {
    if viewport.is_compact() {
        compact_image_height(viewport) + metrics.detail_hero_extra_mobile
    } else {
        metrics.detail_hero_height.of(viewport.height)
    }
}

fn row_scroll<R>(
    ui: &mut crate::gfx::Ui,
    viewport: Viewport,
    detail: &DetailModel,
    row: usize,
    id: impl std::hash::Hash + std::fmt::Debug,
    add: impl FnOnce(&mut crate::gfx::Ui) -> R,
) -> crate::gfx::ScrollOutput<R> {
    crate::components::pan_row(
        ui,
        id,
        |area| {
            area.horizontal_scroll_offset(detail.row_scroll_offsets[row])
                .scroll_source(crate::gfx::ScrollSource::NONE)
        },
        add,
    )
}

fn record_row_max<R>(ui: &crate::gfx::Ui, row: usize, output: &crate::gfx::ScrollOutput<R>) {
    crate::report_scroll_region(
        ui.ctx(),
        crate::ScrollRegion {
            id: crate::ScrollId::Row(row),
            axis: crate::ScrollAxis::Horizontal,
            rect: output.inner_rect,
            max: (output.content_size.x - output.inner_rect.width()).max(0.0),
        },
    );
}

pub fn draw_detail(
    context: &crate::gfx::Context,
    viewport: Viewport,
    detail: &DetailModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let hero_height = hero_height(viewport, metrics);
    let ppp = context.pixels_per_point();
    let compact = viewport.is_compact();
    let shuffle = detail.episodes.len() > 1;
    let language = detail.language.as_str();
    let t = |key: &str| localized(key, language);
    let screen = crate::gfx::rect_at(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let margin = screen_margin(metrics);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 0, assets);

    let seasons = detail.seasons();
    let selected_season = detail.current_season();
    let season_episodes = detail
        .episodes
        .iter()
        .enumerate()
        .filter(|(_, episode)| episode.season == selected_season)
        .collect::<Vec<_>>();
    let play_target = season_episodes
        .first()
        .map(|(index, episode)| (*index, *episode));

    let mut prefetch_episodes = season_episodes.clone();
    prefetch_episodes.extend(
        detail
            .episodes
            .iter()
            .enumerate()
            .filter(|(_, episode)| episode.season != selected_season),
    );
    prefetch_artwork(
        detail,
        &prefetch_episodes,
        viewport,
        margin,
        Vec2::new(metrics.poster_card_width, metrics.poster_card_height),
        context.pixels_per_point(),
        assets,
    );

    let measured = kit::scroll_state(context, "detail");
    let title_size = metrics.detail_title_size;
    let row_width = viewport.width - margin;
    kit::screen_body(context, viewport, "detail", &detail.id, |ui| {
        let hero = crate::flex::anchored(
            ui.max_rect(),
            Align2::LEFT_TOP,
            Vec2::new(viewport.width, hero_height),
            0.0,
        );
        let backdrop = detail
            .background_url
            .as_deref()
            .or(detail.poster_url.as_deref());
        let target = backdrop_target_size(viewport.width, ppp);
        let fade = crate::motion::appear(
            context,
            Id::new(("fluxa-detail-backdrop", &detail.id)),
            crate::motion::normal(),
            crate::motion::Ease::Linear,
        );
        let image = if compact {
            band(hero, compact_image_height(viewport))
        } else {
            hero
        };
        components::artwork_image(
            &painter,
            image,
            backdrop,
            target,
            ArtworkPriority::Hero,
            Color32::from_white_alpha((225.0 * fade) as u8),
            assets,
        );
        if let Some(texture) = detail.trailer {
            crate::paint_trailer(
                context,
                &painter,
                texture,
                image,
                Some(detail.id.as_str()),
                255,
            );
        }
        if compact {
            let scrim = metrics
                .background
                .gamma_multiply(metrics.screen_detail_scrim_alpha);
            let fade_start = metrics.screen_detail_fade_start;
            let fade_span = metrics.screen_detail_fade_span;
            let bands = crate::flex::column([
                crate::flex::fill().grow(fade_start),
                crate::flex::fill().grow(fade_span).id(0),
                crate::flex::fill().grow(1.0 - fade_start - fade_span).id(1),
            ])
            .align(crate::flex::Align::Start)
            .solve(image);
            if let (Some(upper), Some(lower)) = (bands.find(0), bands.find(1)) {
                paint_vertical_gradient(&painter, upper, Color32::TRANSPARENT, scrim);
                paint_vertical_gradient(&painter, lower, scrim, metrics.background);
            }
            painter.rect_filled(
                Rect::from_min_max(image.left_bottom(), hero.right_bottom()),
                0.0,
                metrics.background,
            );
            paint_vertical_gradient(
                &painter,
                band(below(hero), Dp::D80.px()),
                metrics.background,
                Color32::TRANSPARENT,
            );
        } else {
            paint_hero_scrim(&painter, hero, metrics);
            paint_vertical_gradient(
                &painter,
                band(below(hero), metrics.screen_detail_fade_bottom),
                metrics.background,
                Color32::TRANSPARENT,
            );
        }
        paint_vertical_gradient(
            &painter,
            band(hero, metrics.screen_detail_fade_top),
            shade(Shade::Veil),
            Color32::TRANSPARENT,
        );

        if let Some(text) = detail
            .trailer_subtitle
            .as_deref()
            .filter(|_| detail.trailer.is_some())
        {
            crate::paint_trailer_subtitle(context, image, text);
        }

        let content_width = if compact {
            viewport.width - margin * 2.0
        } else {
            metrics.detail_info_width.of(viewport.width)
        };
        let action_rects = RefCell::new(Vec::new());
        let activated = Cell::new(None);
        let reveal = crate::motion::appear(
            context,
            Id::new(("fluxa-detail-reveal", &detail.id)),
            crate::motion::slow(),
            crate::motion::Ease::Out,
        );
        crate::gfx::Area::new(Id::new("fluxa-detail-hero-content"))
            .constrain(false)
            .pivot(Align2::LEFT_BOTTOM)
            .fixed_pos(Pos2::new(
                margin,
                hero.bottom() - metrics.detail_hero_bottom_inset + 16.0 * (1.0 - reveal),
            ))
            .show(context, |ui| {
                ui.multiply_opacity(reveal);
                ui.set_max_width(content_width);
                let assets_cell = RefCell::new(&mut *assets);
                let resume = detail.resume.as_ref();
                let play_label = crate::play_label(
                    language,
                    resume,
                    play_target
                        .filter(|_| detail.is_series())
                        .map(|(_, episode)| {
                            (episode.season, episode.number, Some(episode.title.as_str()))
                        }),
                );
                let play_node = match play_target {
                    _ if resume.is_some() => NODE_DETAIL_PLAY,
                    Some((index, _)) if detail.is_series() => {
                        NODE_DETAIL_EPISODE_BASE + index as u64
                    }
                    _ => NODE_DETAIL_PLAY,
                };
                let slots = fluxa_theme::user::detail_actions();
                let loading_title = if detail.is_loading && detail.title.is_empty() {
                    t("common.loading")
                } else {
                    detail.title.clone()
                };
                let error = detail.streams_error.as_deref().or(detail.error.as_deref());
                let meta_label = |ui: &mut crate::gfx::Ui, text: &str, color: Color32| {
                    ui.label(RichText::new(text).size(metrics.text.meta).color(color));
                };
                if compact {
                    ui.set_min_width(content_width);
                    let logo_bounds = Vec2::new(content_width.min(260.0), 96.0);
                    let centered = || crate::emoji::Label {
                        wrap: true,
                        center: true,
                        ..Default::default()
                    };
                    let facts = detail
                        .facts
                        .iter()
                        .cloned()
                        .chain(detail.genres.iter().take(2).cloned())
                        .collect::<Vec<_>>()
                        .join("  ·  ");
                    let show = |name: &str| {
                        slots
                            .as_ref()
                            .is_none_or(|slots| slots.iter().any(|slot| slot == name))
                    };
                    let mut hero = vec![
                        kit::widget(ui, |ui| {
                            let mut guard = assets_cell.borrow_mut();
                            let assets = &mut **guard;
                            let logo = components::title_logo(
                                ppp,
                                detail.logo_url.as_deref(),
                                logo_bounds,
                                ArtworkPriority::Hero,
                                assets,
                            );
                            if let Some((texture, logo_size)) = logo {
                                ui.add(crate::gfx::Image::new((texture, logo_size)));
                            } else if components::failed_logo_slot(
                                ui,
                                assets,
                                detail.logo_url.as_deref(),
                                logo_bounds,
                                [0.5, 0.5],
                            ) {
                            } else {
                                crate::emoji::label(
                                    ui,
                                    assets,
                                    &loading_title,
                                    metrics.text.display,
                                    text_primary(),
                                    centered(),
                                );
                            }
                        }),
                        vspace(Space::S12.px(&metrics)),
                        kit::widget(ui, |ui| {
                            let mut guard = assets_cell.borrow_mut();
                            crate::emoji::label(
                                ui,
                                &mut **guard,
                                &facts,
                                metrics.text.meta,
                                text_primary(),
                                centered(),
                            );
                        }),
                    ];
                    if !detail.ratings.is_empty() {
                        hero.push(vspace(Space::S4.px(&metrics)));
                        hero.push(kit::widget(ui, |ui| {
                            kit::view().row().wrap().gap(Space::S4).show(ui, |ui| {
                                let mut guard = assets_cell.borrow_mut();
                                components::rating_row(
                                    ui,
                                    &detail.ratings,
                                    metrics.text.meta,
                                    text_primary(),
                                    metrics.text_secondary,
                                    &mut **guard,
                                );
                            });
                        }));
                    }
                    for line in &detail.info_lines {
                        hero.push(vspace(Space::S4.px(&metrics)));
                        hero.push(kit::widget(ui, |ui| {
                            let mut guard = assets_cell.borrow_mut();
                            crate::emoji::label(
                                ui,
                                &mut **guard,
                                line,
                                metrics.text.meta,
                                metrics.text_muted,
                                centered(),
                            );
                        }));
                    }
                    hero.push(vspace(Space::S16.px(&metrics)));
                    if show("play") {
                        hero.push(kit::widget(ui, |ui| {
                            let mut guard = assets_cell.borrow_mut();
                            let play = components::play_button(
                                ui,
                                &mut **guard,
                                &play_label,
                                Some(content_width),
                                48.0,
                                metrics.text.subtitle,
                                resume
                                    .map(|card| card.progress)
                                    .filter(|progress| *progress > 0.0),
                            );
                            if crate::kit::probing() {
                                return;
                            }
                            action_rects
                                .borrow_mut()
                                .push((NODE_DETAIL_PLAY, play.rect));
                            if play.clicked() {
                                activated.set(Some(play_node));
                            }
                        }));
                        hero.push(vspace(Space::S16.px(&metrics)));
                    }
                    hero.push(kit::widget(ui, |ui| {
                        kit::view().row().show(ui, |ui| {
                            let mut guard = assets_cell.borrow_mut();
                            let assets = &mut **guard;
                            let mut actions = vec![
                                (
                                    NODE_DETAIL_WATCHLIST,
                                    if detail.in_watchlist { "Check" } else { "Plus" },
                                    detail.in_watchlist,
                                    t("library.watchlist"),
                                ),
                                (
                                    NODE_DETAIL_COMPLETED,
                                    "CircleCheck",
                                    detail.completed,
                                    t("library.completed"),
                                ),
                                (
                                    NODE_DETAIL_DROPPED,
                                    "Ban",
                                    detail.dropped,
                                    t("library.dropped"),
                                ),
                            ];
                            if detail.favorites_enabled {
                                actions.push((
                                    NODE_DETAIL_FAVORITE,
                                    if detail.favorite {
                                        "HeartFilled"
                                    } else {
                                        "Heart"
                                    },
                                    detail.favorite,
                                    t("library.favorites"),
                                ));
                            }
                            if shuffle {
                                actions.push((
                                    NODE_DETAIL_SHUFFLE,
                                    "Shuffle",
                                    false,
                                    t("common.shuffle"),
                                ));
                            }
                            if let Some(slots) = slots.as_ref() {
                                let name = |node: u64| match node {
                                    NODE_DETAIL_WATCHLIST => "watchlist",
                                    NODE_DETAIL_COMPLETED => "completed",
                                    NODE_DETAIL_DROPPED => "dropped",
                                    NODE_DETAIL_FAVORITE => "favorite",
                                    _ => "shuffle",
                                };
                                actions.retain(|action| {
                                    slots.iter().any(|slot| slot == name(action.0))
                                });
                                actions.sort_by_key(|action| {
                                    slots.iter().position(|slot| slot == name(action.0))
                                });
                            }
                            let width = content_width / actions.len().max(1) as f32;
                            for (node, icon, active, label) in actions {
                                let response = components::labeled_action(
                                    ui,
                                    assets.icon(icon),
                                    &label,
                                    width,
                                    metrics.text.label,
                                    active,
                                );
                                if crate::kit::probing() {
                                    continue;
                                }
                                action_rects.borrow_mut().push((node, response.rect));
                                if response.clicked() {
                                    activated.set(Some(node));
                                }
                            }
                        });
                    }));
                    kit::view().class("detail-hero").items(ui, hero);
                    if let Some(error) = error {
                        let below = vec![
                            vspace(Space::S8.px(&metrics)),
                            kit::widget(ui, |ui| meta_label(ui, error, metrics.text_secondary)),
                        ];
                        kit::lay(ui, below);
                    }
                    return;
                }
                ui.set_item_spacing(Vec2::new(Space::S8.px(&metrics), 0.0));
                let logo_bounds = Vec2::new(content_width.min(440.0), 150.0);
                let mut stack = vec![
                    kit::widget(ui, |ui| {
                        let mut guard = assets_cell.borrow_mut();
                        let assets = &mut **guard;
                        let logo = components::title_logo(
                            ppp,
                            detail.logo_url.as_deref(),
                            logo_bounds,
                            ArtworkPriority::Hero,
                            assets,
                        );
                        if let Some((texture, logo_size)) = logo {
                            ui.add(crate::gfx::Image::new((texture, logo_size)));
                        } else if components::failed_logo_slot(
                            ui,
                            assets,
                            detail.logo_url.as_deref(),
                            logo_bounds,
                            [0.0, 0.5],
                        ) {
                        } else {
                            ui.label(
                                RichText::new(&loading_title)
                                    .size(metrics.text.display)
                                    .strong()
                                    .color(text_primary()),
                            );
                        }
                    }),
                    vspace(Space::S24.px(&metrics)),
                    kit::widget(ui, |ui| {
                        kit::view().row().wrap().gap(Space::S8).show(ui, |ui| {
                            for fact in &detail.facts {
                                ui.label(
                                    RichText::new(fact)
                                        .size(metrics.text.meta)
                                        .strong()
                                        .color(metrics.text_primary),
                                );
                                ui.label(
                                    RichText::new("·")
                                        .size(metrics.text.meta)
                                        .color(metrics.text_muted),
                                );
                            }
                        });
                    }),
                ];
                if !detail.ratings.is_empty() {
                    stack.push(vspace(Space::S8.px(&metrics)));
                    stack.push(kit::widget(ui, |ui| {
                        kit::view().row().wrap().gap(Space::S4).show(ui, |ui| {
                            let mut guard = assets_cell.borrow_mut();
                            components::rating_row(
                                ui,
                                &detail.ratings,
                                metrics.text.meta,
                                text_primary(),
                                metrics.text_secondary,
                                &mut **guard,
                            );
                        });
                    }));
                }
                if !detail.genres.is_empty() {
                    stack.push(vspace(Space::S8.px(&metrics)));
                    stack.push(kit::widget(ui, |ui| {
                        meta_label(ui, &detail.genres.join("  ·  "), text_primary());
                    }));
                }
                for line in &detail.info_lines {
                    stack.push(vspace(Space::S4.px(&metrics)));
                    stack.push(kit::widget(ui, |ui| {
                        meta_label(ui, line, metrics.text_muted);
                    }));
                }
                stack.push(vspace(Space::S24.px(&metrics)));
                stack.push(kit::widget(ui, |ui| {
                    kit::view().class("detail-actions").show(ui, |ui| {
                        let mut guard = assets_cell.borrow_mut();
                        let assets = &mut **guard;
                        let height = metrics.detail_play_height;
                        let icons = vec![
                            (NODE_DETAIL_SHUFFLE, "Shuffle", false, t("common.shuffle")),
                            (
                                NODE_DETAIL_COMPLETED,
                                "CircleCheck",
                                detail.completed,
                                t("library.completed"),
                            ),
                            (
                                NODE_DETAIL_DROPPED,
                                "Ban",
                                detail.dropped,
                                t("library.dropped"),
                            ),
                            (
                                NODE_DETAIL_FAVORITE,
                                if detail.favorite {
                                    "HeartFilled"
                                } else {
                                    "Heart"
                                },
                                detail.favorite,
                                t("library.favorites"),
                            ),
                        ];
                        let icons: Vec<_> = icons
                            .into_iter()
                            .filter(|(node, ..)| *node != NODE_DETAIL_SHUFFLE || shuffle)
                            .filter(|(node, ..)| {
                                *node != NODE_DETAIL_FAVORITE || detail.favorites_enabled
                            })
                            .collect();
                        let order = slots.clone().unwrap_or_else(|| {
                            ["play", "watchlist", "shuffle", "completed", "dropped"]
                                .map(str::to_owned)
                                .to_vec()
                        });
                        for slot in &order {
                            match slot.as_str() {
                                "play" => {
                                    let play = components::play_button(
                                        ui,
                                        assets,
                                        &play_label,
                                        Some(
                                            (ui.painter()
                                                .layout_no_wrap(
                                                    play_label.clone(),
                                                    crate::gfx::FontId::proportional(
                                                        metrics.text.subtitle,
                                                    ),
                                                    text_primary(),
                                                )
                                                .size()
                                                .x
                                                + 70.0)
                                                .min(340.0),
                                        ),
                                        height,
                                        metrics.text.subtitle,
                                        resume
                                            .map(|card| card.progress)
                                            .filter(|progress| *progress > 0.0),
                                    );
                                    if crate::kit::probing() {
                                        continue;
                                    }
                                    action_rects
                                        .borrow_mut()
                                        .push((NODE_DETAIL_PLAY, play.rect));
                                    if play.clicked() {
                                        activated.set(Some(play_node));
                                    }
                                }
                                "watchlist" => {
                                    let watchlist = components::icon_button_sized(
                                        ui,
                                        assets.icon(if detail.in_watchlist {
                                            "Check"
                                        } else {
                                            "Plus"
                                        }),
                                        Vec2::splat(height),
                                        text_primary(),
                                        true,
                                        detail.in_watchlist,
                                        true,
                                    )
                                    .on_hover_text(t("library.watchlist"));
                                    if crate::kit::probing() {
                                        continue;
                                    }
                                    action_rects
                                        .borrow_mut()
                                        .push((NODE_DETAIL_WATCHLIST, watchlist.rect));
                                    if watchlist.clicked() {
                                        activated.set(Some(NODE_DETAIL_WATCHLIST));
                                    }
                                }
                                other => {
                                    let node = match other {
                                        "shuffle" => NODE_DETAIL_SHUFFLE,
                                        "completed" => NODE_DETAIL_COMPLETED,
                                        "dropped" => NODE_DETAIL_DROPPED,
                                        _ => NODE_DETAIL_FAVORITE,
                                    };
                                    let Some((node, icon, active, hint)) =
                                        icons.iter().find(|(candidate, ..)| *candidate == node)
                                    else {
                                        continue;
                                    };
                                    let (node, icon, active) = (*node, *icon, *active);
                                    let response = components::icon_button_sized(
                                        ui,
                                        assets.icon(icon),
                                        Vec2::splat(height),
                                        text_primary(),
                                        true,
                                        active,
                                        true,
                                    )
                                    .on_hover_text(hint);
                                    if crate::kit::probing() {
                                        continue;
                                    }
                                    action_rects.borrow_mut().push((node, response.rect));
                                    if response.clicked() {
                                        activated.set(Some(node));
                                    }
                                }
                            }
                        }
                    });
                }));
                if !detail.description.is_empty() {
                    stack.push(vspace(Space::S24.px(&metrics)));
                    stack.push(kit::widget(ui, |ui| {
                        let expanded_id = Id::new(("fluxa-detail-synopsis", &detail.id));
                        let expanded = context
                            .data(|data| data.get_temp::<bool>(expanded_id).unwrap_or(false));
                        let limit = 320;
                        let long = detail.description.chars().count() > limit;
                        let text = if long && !expanded {
                            format!(
                                "{}…",
                                detail
                                    .description
                                    .chars()
                                    .take(limit - 1)
                                    .collect::<String>()
                                    .trim_end()
                            )
                        } else {
                            detail.description.clone()
                        };
                        let mut guard = assets_cell.borrow_mut();
                        let synopsis = crate::emoji::label(
                            ui,
                            &mut **guard,
                            &text,
                            metrics.text.body,
                            metrics.text_primary,
                            crate::emoji::Label {
                                wrap: true,
                                line_height: Some(metrics.text.body * 1.5),
                                sense: if long { Sense::click() } else { Sense::hover() },
                                ..Default::default()
                            },
                        );
                        if long && synopsis.clicked() {
                            context.data_mut(|data| data.insert_temp(expanded_id, !expanded));
                        }
                    }));
                }
                if detail.is_loading_streams {
                    stack.push(vspace(Space::S12.px(&metrics)));
                    stack.push(kit::widget(ui, |ui| {
                        meta_label(ui, &t("common.loading"), metrics.text_secondary);
                    }));
                } else if let Some(error) = error {
                    stack.push(vspace(Space::S12.px(&metrics)));
                    stack.push(kit::widget(ui, |ui| {
                        meta_label(ui, error, metrics.text_secondary);
                    }));
                }
                kit::lay(ui, stack);
            });
        layout.focusable.extend(action_rects.into_inner());
        if activated.get().is_some() {
            layout.activated = activated.get();
        }
        kit::view()
            .column()
            .max_width(viewport.width)
            .show(ui, |ui| {
                ui.set_min_height(hero_height + Space::S12.px(&metrics));
            });
        crate::gfx::Frame::NONE
            .inner_margin(crate::gfx::Margin {
                left: margin as i8,
                right: margin as i8,
                top: 0,
                bottom: metrics.detail_section_gap as i8,
            })
            .show(ui, |ui| {
                ui.set_max_width(row_width);
                let mut lead = 0.0;
                for section in detail.section_order() {
                    if !detail.has_content(section) {
                        continue;
                    }
                    match section {
                        DetailSection::Episodes => {
                            section_heading(
                                ui,
                                lead,
                                metrics.detail_section_title,
                                (&t("auto.seasons"), title_size),
                            );
                            ui.scope(|ui| {
                                ui.set_max_width(row_width - margin);
                                let output = row_scroll(
                                    ui,
                                    viewport,
                                    detail,
                                    0,
                                    "fluxa-detail-season-scroll",
                                    |ui| {
                                        kit::view().row().gap(Space::S8).show(ui, |ui| {
                                            let season_label = |season: i64| {
                                                if season == 0 {
                                                    t("auto.specials")
                                                } else {
                                                    t("format.season_number").replacen(
                                                        "%s",
                                                        &season.to_string(),
                                                        1,
                                                    )
                                                }
                                            };
                                            let mode = detail.season_selector_mode();
                                            if mode == SeasonSelector::Dropdown {
                                                let options = seasons
                                                    .iter()
                                                    .map(|season| {
                                                        (season.to_string(), season_label(*season))
                                                    })
                                                    .collect::<Vec<_>>();
                                                let selected = selected_season.to_string();
                                                let width = components::dropdown_width_for_label(
                                                    ui,
                                                    &season_label(selected_season),
                                                    metrics,
                                                    160.0,
                                                    320.0,
                                                );
                                                let (response, changed) = components::dropdown(
                                                    ui,
                                                    "fluxa-detail-season-dropdown",
                                                    &selected,
                                                    &options,
                                                    width,
                                                    compact,
                                                    metrics,
                                                );
                                                layout
                                                    .focusable
                                                    .push((NODE_DETAIL_SEASON_BASE, response.rect));
                                                layout.choices.extend(components::sheet_choice(
                                                    NODE_DETAIL_SEASON_BASE,
                                                    "detail:season",
                                                    t("settings.season_selector"),
                                                    &options,
                                                    &selected,
                                                ));
                                                if let Some(value) = changed {
                                                    layout.filter_change =
                                                        Some(("detail:season".to_owned(), value));
                                                }
                                                return;
                                            }
                                            for season in &seasons {
                                                let label = season_label(*season);
                                                let response = if mode == SeasonSelector::Chips {
                                                    components::button_auto_width(
                                                        ui,
                                                        &label,
                                                        if *season == selected_season {
                                                            components::ButtonKind::Selected
                                                        } else {
                                                            components::ButtonKind::Secondary
                                                        },
                                                        metrics.nav_label_size,
                                                        metrics,
                                                    )
                                                } else {
                                                    let poster = detail
                                                        .season_posters
                                                        .iter()
                                                        .find(|(number, _)| number == season)
                                                        .map(|(_, url)| url.as_str())
                                                        .or(detail.poster_url.as_deref());
                                                    components::season_tile(
                                                        ui,
                                                        assets,
                                                        poster,
                                                        &label,
                                                        *season == selected_season,
                                                        season_tile(metrics),
                                                        metrics.detail_season_tile_label,
                                                        metrics,
                                                    )
                                                };
                                                layout.focusable.push((
                                                    NODE_DETAIL_SEASON_BASE
                                                        + (*season).clamp(0, 99) as u64,
                                                    response.rect,
                                                ));
                                                if response.clicked() {
                                                    layout.activated = Some(
                                                        NODE_DETAIL_SEASON_BASE
                                                            + (*season).clamp(0, 99) as u64,
                                                    );
                                                }
                                            }
                                        });
                                    },
                                );
                                record_row_max(ui, 0, &output);
                            });
                            section_heading(
                                ui,
                                metrics.detail_section_gap,
                                metrics.detail_section_title,
                                (&t("auto.episodes"), title_size),
                            );
                            let thumb = Vec2::new(
                                metrics.detail_episode_width,
                                metrics.detail_episode_width * 9.0 / 16.0,
                            );
                            if detail.episode_layout_for(compact) == EpisodeLayout::Cards {
                                ui.scope(|ui| {
                                    ui.set_max_width(row_width);
                                    let output = row_scroll(
                                        ui,
                                        viewport,
                                        detail,
                                        1,
                                        ("fluxa-detail-episode-scroll", selected_season),
                                        |ui| {
                                            kit::view().row().gap(Space::S16).show(ui, |ui| {
                                                for (index, episode) in &season_episodes {
                                                    let size = Vec2::new(
                                                        metrics.detail_episode_width,
                                                        episodes::episodes_height(
                                                            viewport, detail, margin,
                                                        ),
                                                    );
                                                    let node =
                                                        NODE_DETAIL_EPISODE_BASE + *index as u64;
                                                    let response = tile(
                                                        ui,
                                                        (
                                                            "fluxa-detail-episode-card",
                                                            selected_season,
                                                            *index,
                                                        ),
                                                        size,
                                                        Sense::click(),
                                                        |ui, rect| {
                                                            if !ui.is_rect_visible(rect) {
                                                                return;
                                                            }
                                                            let card =
                                                                crate::flex::column(
                                                                    [
                                                                        crate::flex::leaf(thumb)
                                                                            .id(0),
                                                                        crate::flex::slot(
                                                                            Space::S12.px(&metrics),
                                                                        )
                                                                        .height(
                                                                            Space::S12.px(&metrics),
                                                                        ),
                                                                        crate::flex::fill().id(1),
                                                                    ],
                                                                )
                                                                .align(crate::flex::Align::Start)
                                                                .solve(rect);
                                                            let image =
                                                                card.find(0).unwrap_or(rect);
                                                            let text = card.find(1).unwrap_or(rect);
                                                            let view = detail.episode_view(episode);
                                                            components::episode_artwork(
                                                                ui.painter(),
                                                                assets,
                                                                image,
                                                                episode,
                                                                view.hide_still,
                                                                ui.rect_contains_pointer(rect),
                                                                false,
                                                                metrics,
                                                            );
                                                            components::episode_text(
                                                                ui.painter(),
                                                                text.min,
                                                                metrics.detail_episode_width,
                                                                episode.number,
                                                                view.title,
                                                                view.overview.map(|text| (text, 2)),
                                                            );
                                                        },
                                                    );
                                                    layout.focusable.push((node, response.rect));
                                                    if response.clicked() {
                                                        layout.activated = Some(node);
                                                    }
                                                }
                                            });
                                        },
                                    );
                                    record_row_max(ui, 1, &output);
                                });
                            } else {
                                let context_data = episodes::EpisodeContext {
                                    viewport,
                                    detail,
                                    episodes: &season_episodes,
                                    season: selected_season,
                                    metrics,
                                    margin,
                                };
                                episodes::draw_episodes(ui, &context_data, assets, &mut layout);
                            }
                        }
                        DetailSection::Cast => {
                            section_heading(
                                ui,
                                lead,
                                metrics.detail_section_title,
                                (
                                    &t(if detail.crew.is_empty() {
                                        "auto.cast"
                                    } else {
                                        "detail.cast_crew"
                                    }),
                                    title_size,
                                ),
                            );
                            ui.scope(|ui| {
                                ui.set_max_width(row_width);
                                let output = row_scroll(
                                    ui,
                                    viewport,
                                    detail,
                                    2,
                                    "fluxa-detail-cast-scroll",
                                    |ui| {
                                        kit::view().row().gap(Space::S16).show(ui, |ui| {
                                            for (index, member) in
                                                detail.cast.iter().chain(&detail.crew).enumerate()
                                            {
                                                let size = Vec2::new(
                                                    metrics.detail_cast_size
                                                        + Space::S16.px(&metrics),
                                                    metrics.detail_cast_size
                                                        + metrics.detail_cast_text,
                                                );
                                                tile(
                                                    ui,
                                                    ("fluxa-detail-cast-card", index),
                                                    size,
                                                    Sense::hover(),
                                                    |ui, rect| {
                                                        if !ui.is_rect_visible(rect) {
                                                            return;
                                                        }
                                                        let painter = ui.painter();
                                                        let name_height = Dp::D16.px();
                                                        let cast = crate::flex::column([
                                                            crate::flex::leaf(Vec2::splat(
                                                                metrics.detail_cast_size,
                                                            ))
                                                            .id(0),
                                                            crate::flex::leaf(Vec2::new(
                                                                0.0,
                                                                Space::S10.px(&metrics),
                                                            )),
                                                            crate::flex::leaf(Vec2::new(
                                                                rect.width(),
                                                                name_height,
                                                            ))
                                                            .id(1),
                                                            crate::flex::leaf(Vec2::new(
                                                                rect.width(),
                                                                name_height,
                                                            ))
                                                            .id(2),
                                                        ])
                                                        .align(crate::flex::Align::Center)
                                                        .solve(rect);
                                                        let circle = cast.find(0).unwrap_or(rect);
                                                        components::avatar(
                                                            painter,
                                                            assets,
                                                            circle.center(),
                                                            circle.width() / 2.0,
                                                            &member.name,
                                                            member.photo.as_deref(),
                                                            metrics,
                                                        );
                                                        let font =
                                                            FontId::proportional(metrics.text.meta);
                                                        draw_text(
                                                            painter,
                                                            cast.find(1)
                                                                .unwrap_or(rect)
                                                                .center_top(),
                                                            Align2::CENTER_TOP,
                                                            &member.name,
                                                            font,
                                                            text_primary(),
                                                            Some(rect.width()),
                                                        );
                                                        if let Some(role) = member.role.as_deref() {
                                                            let font = FontId::proportional(
                                                                metrics.text.label,
                                                            );
                                                            draw_text(
                                                                painter,
                                                                cast.find(2)
                                                                    .unwrap_or(rect)
                                                                    .center_top(),
                                                                Align2::CENTER_TOP,
                                                                role,
                                                                font,
                                                                metrics.text_muted,
                                                                Some(rect.width()),
                                                            );
                                                        }
                                                    },
                                                );
                                            }
                                        });
                                    },
                                );
                                record_row_max(ui, 2, &output);
                            });
                        }
                        DetailSection::Franchise | DetailSection::Recommendations => {
                            let (title, cards, base, row, id) =
                                if section == DetailSection::Franchise {
                                    (
                                        t(DetailSection::Franchise.title_key()),
                                        &detail.franchise,
                                        NODE_DETAIL_FRANCHISE_BASE,
                                        5,
                                        "fluxa-detail-franchise",
                                    )
                                } else {
                                    (
                                        t("player.recommendations"),
                                        &detail.similar,
                                        NODE_DETAIL_SIMILAR_BASE,
                                        3,
                                        "fluxa-detail-similar",
                                    )
                                };
                            section_heading(
                                ui,
                                lead,
                                metrics.detail_section_title,
                                (&title, title_size),
                            );
                            let (poster, slot_height) = similar_size(metrics);
                            ui.scope(|ui| {
                                ui.set_max_width(row_width);
                                let output =
                                    row_scroll(ui, viewport, detail, row, (id, "scroll"), |ui| {
                                        kit::view().row().gap(Space::S14).show(ui, |ui| {
                                            for (index, card) in cards.iter().enumerate() {
                                                let node = base + index as u64;
                                                let response = tile(
                                                    ui,
                                                    (id, index),
                                                    Vec2::new(poster.x, slot_height),
                                                    Sense::click(),
                                                    |ui, rect| {
                                                        if !ui.is_rect_visible(rect) {
                                                            return;
                                                        }
                                                        components::poster_card(
                                                            ui.painter(),
                                                            crate::flex::anchored(
                                                                rect,
                                                                Align2::LEFT_TOP,
                                                                poster,
                                                                0.0,
                                                            ),
                                                            card,
                                                            metrics,
                                                            assets,
                                                            false,
                                                        );
                                                    },
                                                );
                                                layout.focusable.push((node, response.rect));
                                                if response.clicked() {
                                                    layout.activated = Some(node);
                                                }
                                            }
                                        });
                                    });
                                record_row_max(ui, row, &output);
                            });
                        }
                        DetailSection::Trailers => {
                            section_heading(
                                ui,
                                lead,
                                metrics.detail_section_title,
                                (&t(DetailSection::Trailers.title_key()), title_size),
                            );
                            let size = trailer_size(compact, metrics);
                            ui.scope(|ui| {
                                ui.set_max_width(row_width);
                                let output = row_scroll(
                                    ui,
                                    viewport,
                                    detail,
                                    4,
                                    "fluxa-detail-trailer-scroll",
                                    |ui| {
                                        kit::view().row().gap(Space::S16).show(ui, |ui| {
                                            for (index, trailer) in
                                                detail.trailer_cards.iter().enumerate()
                                            {
                                                let node = NODE_DETAIL_TRAILER_BASE + index as u64;
                                                let response = tile(
                                                    ui,
                                                    ("fluxa-detail-trailer-card", index),
                                                    Vec2::new(
                                                        size.x,
                                                        size.y + metrics.detail_trailer_text,
                                                    ),
                                                    Sense::click(),
                                                    |ui, rect| {
                                                        if !ui.is_rect_visible(rect) {
                                                            return;
                                                        }
                                                        components::trailer_card(
                                                            ui.painter(),
                                                            assets,
                                                            crate::flex::anchored(
                                                                rect,
                                                                Align2::LEFT_TOP,
                                                                size,
                                                                0.0,
                                                            ),
                                                            trailer,
                                                            ui.rect_contains_pointer(rect),
                                                            metrics,
                                                        );
                                                    },
                                                );
                                                layout.focusable.push((node, response.rect));
                                                if response.clicked() {
                                                    layout.activated = Some(node);
                                                }
                                            }
                                        });
                                    },
                                );
                                record_row_max(ui, 4, &output);
                            });
                        }
                        DetailSection::ShowInfo => {
                            section_heading(
                                ui,
                                lead,
                                metrics.detail_section_title,
                                (&t(DetailSection::ShowInfo.title_key()), title_size),
                            );
                            show_info(ui, detail, viewport, metrics);
                        }
                    }
                    lead = metrics.detail_section_gap;
                }
            });
    });
    let scroll_y = measured.map_or(0.0, |(offset, _)| offset);
    let sticky_on = scroll_y > viewport.height * metrics.screen_detail_sticky_ratio;
    let back_rect = crate::flex::stack(
        Align2::LEFT_TOP,
        [crate::flex::leaf(Vec2::splat(metrics.icon_button_size))
            .id(0)
            .margin(crate::flex::Insets {
                left: margin,
                top: metrics.detail_back_top,
                ..Default::default()
            })],
    )
    .solve(screen)
    .find(0)
    .unwrap_or(screen);
    if !sticky_on {
        components::placed_front(context, Id::new("fluxa-detail-back"), back_rect.min, |ui| {
            let response = components::back_button(ui, assets.icon("ArrowLeft"), metrics)
                .on_hover_text(t("auto.back"));
            if response.clicked() {
                layout.activated = Some(NODE_DETAIL_BACK);
            }
        });
        layout.focusable.push((NODE_DETAIL_BACK, back_rect));
    }
    let sticky_logo = components::title_logo(
        ppp,
        detail.logo_url.as_deref(),
        Vec2::new(metrics.screen_detail_sticky_logo_width, Dp::D28.px()),
        ArtworkPriority::Visible,
        assets,
    );
    let sticky = components::sticky_bar(
        context,
        "fluxa-detail",
        viewport,
        margin,
        sticky_on,
        components::StickyBar {
            back_node: NODE_DETAIL_BACK,
            back_icon: assets.icon("ArrowLeft"),
            logo: sticky_logo,
            title: detail.title.as_str(),
            actions: vec![
                components::StickyAction {
                    node: NODE_DETAIL_PLAY,
                    icon: assets.icon("PlayFilled"),
                    active: false,
                    hint: t("common.play"),
                },
                components::StickyAction {
                    node: NODE_DETAIL_WATCHLIST,
                    icon: assets.icon(if detail.in_watchlist { "Check" } else { "Plus" }),
                    active: detail.in_watchlist,
                    hint: t("library.watchlist"),
                },
            ],
        },
        metrics,
    );
    if let Some(sticky) = sticky {
        for (node, rect) in sticky.rects {
            layout.focusable.retain(|(existing, _)| *existing != node);
            layout.focusable.push((node, rect));
        }
        layout.activated = match sticky.clicked {
            Some(NODE_DETAIL_PLAY) => Some(match play_target {
                _ if detail.resume.is_some() => NODE_DETAIL_PLAY,
                Some((index, _)) if detail.is_series() => NODE_DETAIL_EPISODE_BASE + index as u64,
                _ => NODE_DETAIL_PLAY,
            }),
            other => other.or(layout.activated),
        };
    }

    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    layout
}

pub(crate) fn tile(
    ui: &mut crate::gfx::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    size: Vec2,
    sense: Sense,
    add: impl FnOnce(&mut crate::gfx::Ui, Rect),
) -> crate::gfx::Response {
    let rect = kit::lay(
        ui,
        vec![
            kit::item(size, move |ui, rect| add(ui, rect))
                .shrink(0.0)
                .align_self(crate::flex::Align::Start),
        ],
    );
    ui.interact(rect, Id::new(id), sense)
}

fn vspace<'a>(px: f32) -> kit::Item<'a> {
    kit::layout_items::gap(px).height(px)
}

fn band(bounds: Rect, height: f32) -> Rect {
    crate::flex::anchored(
        bounds,
        Align2::LEFT_TOP,
        Vec2::new(bounds.width(), height),
        0.0,
    )
}

fn below(rect: Rect) -> Rect {
    rect.translate(Vec2::new(0.0, rect.height()))
}

fn section_heading(ui: &mut crate::gfx::Ui, lead: f32, height: f32, (text, size): (&str, f32)) {
    let heading = kit::widget(ui, |ui| {
        kit::view().column().show(ui, |ui| {
            ui.set_min_height(height);
            ui.label(
                RichText::new(text)
                    .size(size)
                    .strong()
                    .color(text_primary()),
            );
        });
    });
    kit::lay(ui, vec![vspace(lead), heading]);
}

fn show_info(
    ui: &mut crate::gfx::Ui,
    detail: &DetailModel,
    viewport: Viewport,
    metrics: UiMetrics,
) {
    let width = ui.available_width().min(720.0);
    let label_width = if viewport.is_compact() {
        metrics.detail_info_label * metrics.detail_info_label_scale_mobile
    } else {
        metrics.detail_info_label
    };
    for (index, (label, value)) in detail.show_info.iter().enumerate() {
        let rect = kit::view()
            .row()
            .max_width(width)
            .show(ui, |ui| ui.set_min_height(metrics.detail_info_row))
            .response
            .rect;
        let painter = ui.painter();
        if index > 0 {
            painter.hline(
                rect.x_range(),
                rect.top(),
                crate::gfx::Stroke::new(stroke::hairline(), metrics.ink(tone::faint())),
            );
        }
        draw_text(
            painter,
            rect.left_center(),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(metrics.text.meta),
            metrics.text_muted,
            Some(label_width - 12.0),
        );
        draw_text(
            painter,
            rect.left_center() + Vec2::new(label_width, 0.0),
            Align2::LEFT_CENTER,
            value,
            FontId::proportional(metrics.text.meta),
            text_primary(),
            Some(width - label_width),
        );
    }
}

fn prefetch_artwork(
    detail: &DetailModel,
    episodes: &[(usize, &DetailEpisode)],
    viewport: Viewport,
    margin: f32,
    poster: Vec2,
    pixels_per_point: f32,
    assets: &mut impl HomeAssets,
) {
    let metrics = UiMetrics::for_viewport(viewport);
    let target = |size: Vec2| artwork_target_size(size, pixels_per_point);
    if detail.shows(DetailSection::Episodes) {
        let season_posters = detail
            .season_posters
            .iter()
            .map(|(_, url)| url.as_str())
            .chain(detail.poster_url.as_deref());
        for url in season_posters {
            assets.prefetch_for(
                Some(url),
                target(season_tile(metrics)),
                ArtworkPriority::Prefetch,
            );
        }
        episodes::prefetch(detail, episodes, viewport, margin, pixels_per_point, assets);
    }
    if detail.shows(DetailSection::Cast) {
        for member in detail.cast.iter().chain(&detail.crew) {
            assets.prefetch_for(
                member.photo.as_deref(),
                target(Vec2::splat(metrics.detail_cast_size)),
                ArtworkPriority::Prefetch,
            );
        }
    }
    if detail.shows(DetailSection::Recommendations) {
        for card in &detail.similar {
            assets.prefetch_for(card.poster_art(), target(poster), ArtworkPriority::Prefetch);
        }
    }
    if detail.shows(DetailSection::Franchise) {
        for card in &detail.franchise {
            assets.prefetch_for(card.poster_art(), target(poster), ArtworkPriority::Prefetch);
        }
    }
    if detail.shows(DetailSection::Trailers) {
        let size = trailer_size(viewport.is_compact(), metrics);
        for trailer in &detail.trailer_cards {
            assets.prefetch_for(
                trailer.thumbnail.as_deref(),
                target(size),
                ArtworkPriority::Prefetch,
            );
        }
    }
}
