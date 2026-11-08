use super::*;
use crate::draw_text;

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

    let reveal = crate::motion::appear(
        context,
        Id::new(("fluxa-player-recommendation-slide", index)),
        crate::motion::slow(),
        crate::motion::Ease::Out,
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
        shade(Shade::Opaque),
    );
    paint_horizontal_gradient(
        &painter,
        Rect::from_min_max(Pos2::new(rect.center().x, rect.top()), rect.right_bottom()),
        Color32::TRANSPARENT,
        shade(Shade::Scrim),
    );

    let margin = metrics.player_recommendations_margin;
    let panel_width = if compact {
        rect.width() - margin * 2.0
    } else {
        metrics.player_recommendations_width
    };
    let panel_left = if compact {
        margin
    } else {
        rect.right() - metrics.player_recommendations_inset - panel_width
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
            let galley = (!hero.description.is_empty()).then(|| {
                let mut job = egui::text::LayoutJob::simple(
                    hero.description.clone(),
                    FontId::proportional(metrics.player_recommendations_font),
                    ink(text_alpha.min(225)),
                    panel_width,
                );
                job.wrap.max_rows = if compact {
                    metrics.player_recommendations_rows_compact
                } else {
                    metrics.player_recommendations_rows
                };
                job.wrap.overflow_character = Some('…');
                context.fonts_mut(|fonts| fonts.layout_job(job))
            });
            let logo_box = Vec2::new(
                panel_width * 0.7,
                metrics.player_recommendations_logo_height,
            );
            let logo = hero
                .logo_url
                .as_deref()
                .filter(|url| !url.trim().is_empty());
            let fitted_logo = components::title_logo(
                context.pixels_per_point(),
                logo,
                logo_box,
                ArtworkPriority::Visible,
                assets,
            );
            let mut stack = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_max(
                        Pos2::new(panel_left, rect.top()),
                        Pos2::new(panel_left + panel_width, panel_bottom),
                    ))
                    .layout(egui::Layout::bottom_up(egui::Align::Min)),
            );
            stack.spacing_mut().item_spacing = Vec2::ZERO;
            let buttons = stack
                .allocate_space(Vec2::new(panel_width, button_height))
                .1;
            if player.recommendations.len() > 1 {
                stack.add_space(17.0);
                let dots = stack
                    .allocate_space(Vec2::new(player.recommendations.len() as f32 * 20.0, 18.0))
                    .1;
                for dot in 0..player.recommendations.len() {
                    let center = Pos2::new(dots.left() + 5.0 + dot as f32 * 20.0, dots.center().y);
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
                        ink(if active {
                            255
                        } else if response.hovered() {
                            170
                        } else {
                            90
                        }),
                    );
                }
                stack.add_space(9.0);
            } else {
                stack.add_space(22.0);
            }
            if let Some(galley) = galley {
                let slot = stack
                    .allocate_space(Vec2::new(panel_width, galley.size().y))
                    .1;
                painter.galley(
                    Pos2::new(slot.left() + slide, slot.top()),
                    galley,
                    text_primary(),
                );
                stack.add_space(14.0);
            }
            if !hero.eyebrow.is_empty() {
                let slot = stack.allocate_space(Vec2::new(panel_width, 30.0)).1;
                draw_text(
                    &painter,
                    Pos2::new(slot.left() + slide, slot.bottom()),
                    Align2::LEFT_BOTTOM,
                    truncate_to_width(
                        &painter,
                        &hero.eyebrow,
                        &FontId::proportional(metrics.text.body),
                        panel_width,
                    ),
                    FontId::proportional(metrics.text.body),
                    ink(text_alpha.min(190)),
                    None,
                );
            }
            match fitted_logo {
                Some((texture, fitted)) => {
                    let slot = stack.allocate_space(Vec2::new(panel_width, fitted.y)).1;
                    painter.image(
                        texture,
                        Rect::from_min_size(Pos2::new(slot.left() + slide, slot.top()), fitted),
                        full_uv(),
                        Color32::from_white_alpha(text_alpha),
                    );
                }
                None => {
                    let slot = stack.allocate_space(logo_box).1;
                    let origin = Pos2::new(slot.left() + slide, slot.top());
                    if !components::failed_dom_image(
                        &painter,
                        assets,
                        Rect::from_min_size(origin, logo_box),
                        0.0,
                        logo,
                        artwork_target_size(logo_box, context.pixels_per_point()),
                        crate::ImageFit::Contain,
                        [0.0, 1.0],
                    ) {
                        let font = FontId::proportional(metrics.player_recommendations_title_font);
                        draw_text(
                            &painter,
                            Pos2::new(origin.x, slot.bottom()),
                            Align2::LEFT_BOTTOM,
                            &hero.title,
                            font,
                            ink(text_alpha),
                            Some(panel_width),
                        );
                    }
                }
            }

            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(buttons));
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
                    Vec2::splat(metrics.icon_button_size),
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
