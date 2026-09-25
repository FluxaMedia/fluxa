use super::*;

pub fn draw_detail(
    context: &egui::Context,
    viewport: Viewport,
    detail: &DetailModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let scroll_y = resolve_screen_scroll(
        context,
        viewport,
        Id::new("fluxa-screen-scroll-detail"),
        detail_scroll_max(viewport, detail),
    );
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    if !components::artwork_image(
        &painter,
        screen,
        detail.background_url.as_deref(),
        artwork_target_size(screen.size(), context.pixels_per_point()),
        ArtworkPriority::Hero,
        Color32::from_white_alpha(115),
        assets,
    ) {
        components::texture_image(
            &painter,
            assets.background(),
            screen,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::from_white_alpha(115),
        );
    }
    painter.rect_filled(screen, 0.0, Color32::from_black_alpha(178));
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::ZERO,
            Pos2::new(viewport.width * 0.58, viewport.height),
        ),
        0.0,
        Color32::from_black_alpha(80),
    );

    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 0, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let margin = if compact {
        metrics.page_padding
    } else if tv {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let top = (if compact {
        metrics.detail_header_top_mobile
    } else {
        metrics.detail_header_top
    }) - scroll_y;
    let available_width = (viewport.width - margin * 2.0).max(1.0);
    let poster_width = if compact {
        metrics.detail_poster_width_mobile
    } else if tv {
        metrics.detail_poster_width_tv
    } else {
        metrics.detail_poster_width_desktop
    }
    .min(available_width);
    let poster_height = poster_width * metrics.poster_height_ratio;
    let content_x = if compact {
        ((viewport.width - poster_width) / 2.0).max(margin)
    } else {
        margin + poster_width + metrics.detail_content_gap
    };
    let content_width = if compact {
        available_width
    } else {
        (viewport.width - content_x - margin).max(1.0)
    };

    {
        let poster_rect = Rect::from_min_size(
            Pos2::new(if compact { content_x } else { margin }, top),
            Vec2::new(poster_width, poster_height),
        );
        painter.rect_filled(
            poster_rect,
            metrics.card_radius,
            Color32::from_rgb(28, 28, 34),
        );
        components::artwork_image(
            &painter,
            poster_rect,
            detail.poster_url.as_deref(),
            artwork_target_size(poster_rect.size(), context.pixels_per_point()),
            ArtworkPriority::Visible,
            Color32::WHITE,
            assets,
        );
    }

    let mut action_rects = [None; 6];
    egui::Area::new(Id::new("fluxa-shared-detail-content"))
        .fixed_pos(Pos2::new(
            content_x,
            if compact {
                top + poster_height + metrics.detail_mobile_content_gap
            } else {
                top
            },
        ))
        .show(context, |ui| {
            ui.set_max_width(content_width);
            ui.label(
                RichText::new(if detail.is_loading {
                    "Loading details…"
                } else {
                    detail.title.as_str()
                })
                .size(if compact {
                    metrics.screen_title_size_mobile
                } else if tv {
                    metrics.screen_title_size_tv + metrics.nav_label_size
                } else {
                    metrics.screen_title_size_tv
                })
                .strong()
                .color(Color32::WHITE),
            );
            if !detail.meta_line.is_empty() {
                ui.add_space(metrics.control_gap);
                ui.label(
                    RichText::new(detail.meta_line.as_str())
                        .size(if tv {
                            metrics.screen_body_size_tv
                        } else {
                            metrics.screen_body_size
                        })
                        .strong()
                        .color(Color32::from_white_alpha(220)),
                );
            }
            if !detail.genres.is_empty() {
                ui.add_space(metrics.control_gap);
                ui.horizontal_wrapped(|ui| {
                    for genre in &detail.genres {
                        ui.label(
                            RichText::new(genre)
                                .size(metrics.screen_card_subtitle_size)
                                .color(Color32::from_white_alpha(185)),
                        );
                    }
                });
            }
            if !detail.ratings.is_empty() {
                ui.add_space(metrics.control_gap);
                ui.horizontal_wrapped(|ui| {
                    for (source, score) in &detail.ratings {
                        ui.label(
                            RichText::new(format!("{source}  {score}"))
                                .size(metrics.screen_card_subtitle_size)
                                .strong()
                                .color(Color32::from_rgb(245, 205, 115)),
                        );
                    }
                });
            }
            ui.add_space(metrics.section_gap);
            ui.label(
                RichText::new(detail.description.as_str())
                    .size(if tv {
                        metrics.screen_body_size_tv + metrics.control_gap
                    } else {
                        metrics.screen_body_size + metrics.control_gap
                    })
                    .color(Color32::from_white_alpha(190)),
            );
            if let Some(error) = detail.error.as_deref() {
                ui.add_space(metrics.detail_error_gap);
                ui.label(RichText::new(error).color(Color32::from_rgb(255, 145, 125)));
            }
            ui.add_space(metrics.section_gap + metrics.control_gap);
            ui.horizontal_wrapped(|ui| {
                let play = components::button(
                    ui,
                    "Play",
                    metrics.detail_play_width,
                    metrics.screen_control_height,
                    components::ButtonKind::Accent,
                    metrics,
                );
                action_rects[0] = Some(play.rect);
                if play.clicked() {
                    layout.activated = Some(NODE_DETAIL_PLAY);
                }
                let watchlist = components::button(
                    ui,
                    if detail.in_watchlist {
                        "✓ Watchlist"
                    } else {
                        "+ Watchlist"
                    },
                    metrics.detail_watchlist_width,
                    metrics.screen_control_height,
                    if detail.in_watchlist {
                        components::ButtonKind::Selected
                    } else {
                        components::ButtonKind::Secondary
                    },
                    metrics,
                );
                action_rects[1] = Some(watchlist.rect);
                if watchlist.clicked() {
                    layout.activated = Some(NODE_DETAIL_WATCHLIST);
                }
                let completed_label = if detail.completed {
                    format!("✓ {}", localized("library.completed", &detail.language))
                } else {
                    localized("library.completed", &detail.language)
                };
                let completed = components::button(
                    ui,
                    &completed_label,
                    metrics.detail_watchlist_width,
                    metrics.screen_control_height,
                    if detail.completed {
                        components::ButtonKind::Selected
                    } else {
                        components::ButtonKind::Secondary
                    },
                    metrics,
                );
                action_rects[2] = Some(completed.rect);
                if completed.clicked() {
                    layout.activated = Some(NODE_DETAIL_COMPLETED);
                }
                let dropped_label = if detail.dropped {
                    format!("✓ {}", localized("library.dropped", &detail.language))
                } else {
                    localized("library.dropped", &detail.language)
                };
                let dropped = components::button(
                    ui,
                    &dropped_label,
                    metrics.detail_watchlist_width,
                    metrics.screen_control_height,
                    if detail.dropped {
                        components::ButtonKind::Selected
                    } else {
                        components::ButtonKind::Secondary
                    },
                    metrics,
                );
                action_rects[3] = Some(dropped.rect);
                if dropped.clicked() {
                    layout.activated = Some(NODE_DETAIL_DROPPED);
                }
                let favorite_label = if detail.favorite {
                    format!("♥ {}", localized("library.favorites", &detail.language))
                } else {
                    format!("♡ {}", localized("library.favorites", &detail.language))
                };
                let favorite = components::button(
                    ui,
                    &favorite_label,
                    metrics.detail_watchlist_width,
                    metrics.screen_control_height,
                    if detail.favorite {
                        components::ButtonKind::Selected
                    } else {
                        components::ButtonKind::Secondary
                    },
                    metrics,
                );
                action_rects[4] = Some(favorite.rect);
                if favorite.clicked() {
                    layout.activated = Some(NODE_DETAIL_FAVORITE);
                }
                let back = components::button(
                    ui,
                    "Back",
                    metrics.detail_back_width,
                    metrics.screen_control_height,
                    components::ButtonKind::Secondary,
                    metrics,
                );
                action_rects[5] = Some(back.rect);
                if back.clicked() {
                    layout.activated = Some(NODE_DETAIL_BACK);
                }
            });
            if detail.is_loading_streams {
                ui.add_space(metrics.detail_stream_gap);
                ui.label(RichText::new("Finding streams…").color(Color32::from_white_alpha(150)));
            } else if let Some(error) = detail.streams_error.as_deref() {
                ui.add_space(metrics.detail_stream_gap);
                ui.label(RichText::new(error).color(Color32::from_white_alpha(150)));
            }
        });
    for (node, rect) in [
        NODE_DETAIL_PLAY,
        NODE_DETAIL_WATCHLIST,
        NODE_DETAIL_COMPLETED,
        NODE_DETAIL_DROPPED,
        NODE_DETAIL_FAVORITE,
        NODE_DETAIL_BACK,
    ]
    .into_iter()
    .zip(action_rects.into_iter().flatten())
    {
        layout.focusable.push((node, rect));
    }
    let similar_top = if compact {
        top + poster_height + metrics.detail_similar_top_mobile_offset
    } else {
        top + poster_height + metrics.detail_similar_top_offset
    };
    let similar_top = similar_top.max(
        action_rects
            .into_iter()
            .flatten()
            .map(|rect| rect.bottom())
            .fold(top + poster_height, f32::max)
            + metrics.section_gap,
    );
    if !detail.similar.is_empty() {
        egui::Area::new(Id::new("fluxa-shared-detail-similar"))
            .fixed_pos(Pos2::new(margin, similar_top))
            .show(context, |ui| {
                ui.label(
                    RichText::new("You may also like")
                        .size(if tv {
                            metrics.screen_section_title_size_tv
                        } else {
                            metrics.screen_section_title_size
                        })
                        .strong(),
                );
                ui.add_space(metrics.control_gap);
                ui.horizontal(|ui| {
                    for (index, card) in detail.similar.iter().enumerate().take(8) {
                        let width = metrics.horizontal_card_width * 0.60;
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::new(width, width * 0.56), Sense::click());
                        layout
                            .focusable
                            .push((NODE_DETAIL_SIMILAR_BASE + index as u64, rect));
                        if response.clicked() {
                            layout.activated = Some(NODE_DETAIL_SIMILAR_BASE + index as u64);
                        }
                        ui.painter().rect_filled(
                            rect,
                            metrics.card_radius,
                            Color32::from_rgb(28, 28, 34),
                        );
                        components::artwork_image(
                            ui.painter(),
                            rect,
                            card.artwork_url.as_deref(),
                            artwork_target_size(
                                Vec2::new(width, width * 0.56),
                                context.pixels_per_point(),
                            ),
                            ArtworkPriority::Visible,
                            Color32::WHITE,
                            assets,
                        );
                        ui.painter().rect_filled(
                            Rect::from_min_max(
                                Pos2::new(
                                    rect.left(),
                                    rect.bottom() - metrics.similar_overlay_height,
                                ),
                                rect.right_bottom(),
                            ),
                            metrics.card_radius,
                            Color32::from_black_alpha(180),
                        );
                        ui.painter().text(
                            rect.left_bottom()
                                + Vec2::new(metrics.control_gap, -metrics.similar_text_offset),
                            Align2::LEFT_TOP,
                            card.title.as_str(),
                            FontId::proportional(metrics.screen_card_subtitle_size),
                            Color32::WHITE,
                        );
                    }
                });
            });
    }
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    if let Some((_, rect)) = layout.focusable.iter().find(|(id, _)| Some(*id) == focused) {
        painter.rect_stroke(
            rect.expand(metrics.focus_ring_expand),
            metrics.focus_ring_radius,
            egui::Stroke::new(metrics.focus_ring_width, Color32::WHITE),
            egui::StrokeKind::Outside,
        );
    }
    layout
}
