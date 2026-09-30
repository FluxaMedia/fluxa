use super::*;

pub(super) fn draw_recommendations(
    context: &egui::Context,
    viewport: Viewport,
    rect: Rect,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let index = player
        .recommendation_index
        .min(player.recommendations.len() - 1);
    let hero = &player.recommendations[index];
    let metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
    let painter = context.layer_painter(egui::LayerId::background());
    painter.rect_filled(rect, 0.0, Color32::BLACK);

    let reveal = context.animate_value_with_time(
        Id::new(("fluxa-player-recommendation-slide", index)),
        1.0,
        0.35,
    );
    let url = hero.background_url.as_deref();
    let target = backdrop_target_size(rect.width(), context.pixels_per_point());
    components::artwork_image(
        &painter,
        rect,
        url,
        target,
        ArtworkPriority::Visible,
        Color32::from_white_alpha((255.0 * reveal) as u8),
        assets,
    );
    let fade_top = rect.top() + rect.height() * 0.45;
    paint_vertical_gradient(
        &painter,
        Rect::from_min_max(Pos2::new(rect.left(), fade_top), rect.right_bottom()),
        Color32::TRANSPARENT,
        Color32::from_black_alpha(235),
    );
    paint_horizontal_gradient(
        &painter,
        Rect::from_min_max(Pos2::new(rect.center().x, rect.top()), rect.right_bottom()),
        Color32::TRANSPARENT,
        Color32::from_black_alpha(170),
    );

    let margin = if compact { 16.0 } else { 56.0 };
    let panel_width = if compact {
        rect.width() - margin * 2.0
    } else {
        496.0
    };
    let panel_left = if compact {
        margin
    } else {
        rect.right() - 120.0 - panel_width
    };
    let panel_bottom = rect.bottom() - 56.0 - viewport.safe_bottom;
    let slide = 24.0 * (1.0 - reveal);

    egui::Area::new(Id::new("fluxa-player-recommendations"))
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(rect.size());
            let painter = ui.painter().clone();
            let text_alpha = (255.0 * reveal) as u8;

            let button_height = metrics.screen_control_height;
            let buttons_top = panel_bottom - button_height;
            let dots_y = buttons_top - 26.0;
            let mut description_bottom = dots_y - 18.0;
            if player.recommendations.len() < 2 {
                description_bottom = buttons_top - 22.0;
            }

            let description_height = if hero.description.is_empty() {
                0.0
            } else {
                let mut job = egui::text::LayoutJob::simple(
                    hero.description.clone(),
                    FontId::proportional(if compact { 14.0 } else { 16.0 }),
                    Color32::from_white_alpha(text_alpha.min(225)),
                    panel_width,
                );
                job.wrap.max_rows = if compact { 3 } else { 4 };
                job.wrap.overflow_character = Some('…');
                let galley = context.fonts_mut(|fonts| fonts.layout_job(job));
                let height = galley.size().y;
                painter.galley(
                    Pos2::new(panel_left + slide, description_bottom - height),
                    galley,
                    Color32::WHITE,
                );
                height + 14.0
            };
            let mut cursor = description_bottom - description_height;
            if !hero.eyebrow.is_empty() {
                painter.text(
                    Pos2::new(panel_left + slide, cursor),
                    Align2::LEFT_BOTTOM,
                    truncate_to_width(
                        &painter,
                        &hero.eyebrow,
                        &FontId::proportional(14.0),
                        panel_width,
                    ),
                    FontId::proportional(14.0),
                    Color32::from_white_alpha(text_alpha.min(190)),
                );
                cursor -= 30.0;
            }
            let logo_box = Vec2::new(panel_width * 0.7, if compact { 72.0 } else { 120.0 });
            let logo = hero
                .logo_url
                .as_deref()
                .filter(|url| !url.trim().is_empty());
            match components::title_logo(
                context.pixels_per_point(),
                logo,
                logo_box,
                ArtworkPriority::Visible,
                assets,
            ) {
                Some((texture, fitted)) => {
                    painter.image(
                        texture,
                        Rect::from_min_size(
                            Pos2::new(panel_left + slide, cursor - fitted.y),
                            fitted,
                        ),
                        full_uv(),
                        Color32::from_white_alpha(text_alpha),
                    );
                }
                None => {
                    let font = FontId::proportional(if compact { 28.0 } else { 40.0 });
                    painter.text(
                        Pos2::new(panel_left + slide, cursor),
                        Align2::LEFT_BOTTOM,
                        truncate_to_width(&painter, &hero.title, &font, panel_width),
                        font,
                        Color32::from_white_alpha(text_alpha),
                    );
                }
            }

            if player.recommendations.len() > 1 {
                for dot in 0..player.recommendations.len() {
                    let center = Pos2::new(panel_left + 5.0 + dot as f32 * 20.0, dots_y);
                    let hit = Rect::from_center_size(center, Vec2::splat(18.0));
                    let response = ui.interact(
                        hit,
                        Id::new(("fluxa-player-recommendation-dot", dot)),
                        Sense::click(),
                    );
                    let node = NODE_PLAYER_RECOMMENDATION_BASE + dot as u64;
                    layout.focusable.push((node, hit));
                    if response.clicked() {
                        layout.activated = Some(node);
                    }
                    let active = dot == index;
                    painter.circle_filled(
                        center,
                        if active { 4.5 } else { 3.5 },
                        Color32::from_white_alpha(if active {
                            255
                        } else if response.hovered() {
                            170
                        } else {
                            90
                        }),
                    );
                }
            }

            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(
                Pos2::new(panel_left, buttons_top),
                Vec2::new(panel_width, button_height),
            )));
            child.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                let play = components::button_auto_width(
                    ui,
                    &localized("common.play", &player.language),
                    components::ButtonKind::Primary,
                    metrics.nav_label_size + 2.0,
                    metrics,
                );
                layout
                    .focusable
                    .push((NODE_PLAYER_RECOMMENDATION_PLAY, play.rect));
                if play.clicked() {
                    layout.activated = Some(NODE_PLAYER_RECOMMENDATION_PLAY);
                }
                let details = components::button_auto_width(
                    ui,
                    &localized("hero.more_info", &player.language),
                    components::ButtonKind::Secondary,
                    metrics.nav_label_size + 2.0,
                    metrics,
                );
                layout
                    .focusable
                    .push((NODE_PLAYER_RECOMMENDATION_DETAILS, details.rect));
                if details.clicked() {
                    layout.activated = Some(NODE_PLAYER_RECOMMENDATION_DETAILS);
                }
            });

            let dismiss = control(
                ui,
                &painter,
                Rect::from_center_size(
                    Pos2::new(rect.right() - margin * 0.5 - 20.0, rect.top() + 36.0),
                    Vec2::splat(42.0),
                ),
                "close",
                false,
            );
            layout
                .focusable
                .push((NODE_PLAYER_RECOMMENDATIONS_CLOSE, dismiss.rect));
            if dismiss.clicked() {
                layout.activated = Some(NODE_PLAYER_RECOMMENDATIONS_CLOSE);
            }

            draw_mini_player(ui, &painter, rect, compact, player, layout);
        });
}
