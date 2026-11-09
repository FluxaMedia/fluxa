use super::*;
use crate::draw_text;
use crate::flex;

const LOGO: usize = 0;
const EYEBROW: usize = 1;
const BODY: usize = 2;
const PLAY: usize = 3;
const DETAILS: usize = 4;
const DOT_BASE: usize = 10;

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

    let margin = metrics.overlay_margin;
    let panel_width = (rect.width() - margin * 2.0).min(metrics.player_tile_width);
    let slide = Space::S24.px(&metrics) * (1.0 - reveal);

    egui::Area::new(Id::new("fluxa-player-recommendations"))
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(rect.size());
            let painter = ui.painter().clone();
            let text_alpha = (255.0 * reveal) as u8;

            let galley = (!hero.description.is_empty()).then(|| {
                let mut job = egui::text::LayoutJob::simple(
                    hero.description.clone(),
                    FontId::proportional(metrics.text.body),
                    ink(text_alpha.min(225)),
                    panel_width,
                );
                job.wrap.max_rows = 3;
                job.wrap.overflow_character = Some('…');
                context.fonts_mut(|fonts| fonts.layout_job(job))
            });
            let logo_box = Vec2::new(
                panel_width.min(metrics.playerctl_reco_logo_width),
                metrics.hero_logo_height,
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
            let play_label = localized("common.play", &player.language);
            let details_label = localized("hero.more_info", &player.language);
            let button_font = metrics.nav_label_size + 2.0;
            let button_size = |label: &str| {
                Vec2::new(
                    flex::text_size(&painter, label, &FontId::proportional(button_font)).x
                        + metrics.control_gap * 2.0,
                    metrics.screen_control_height,
                )
            };
            let dots = player.recommendations.len();
            let mut column =
                vec![flex::leaf(fitted_logo.map_or(logo_box, |(_, fitted)| fitted)).id(LOGO)];
            if !hero.eyebrow.is_empty() {
                column
                    .push(flex::leaf(Vec2::new(panel_width, Space::S32.px(&metrics))).id(EYEBROW));
            }
            if let Some(galley) = &galley {
                column.push(flex::slot(0.0).height(Space::S14.px(&metrics)));
                column.push(flex::leaf(Vec2::new(panel_width, galley.size().y)).id(BODY));
            }
            if dots > 1 {
                column.push(flex::slot(0.0).height(Space::S10.px(&metrics)));
                column.push(
                    flex::row((0..dots).map(|dot| {
                        flex::leaf(Vec2::splat(Space::S16.px(&metrics))).id(DOT_BASE + dot)
                    }))
                    .gap(Space::S4.px(&metrics)),
                );
                column.push(flex::slot(0.0).height(Space::S16.px(&metrics)));
            } else {
                column.push(flex::slot(0.0).height(Space::S20.px(&metrics)));
            }
            column.push(
                flex::row([
                    flex::leaf(button_size(&play_label)).id(PLAY),
                    flex::leaf(button_size(&details_label)).id(DETAILS),
                ])
                .gap(Space::S12.px(&metrics)),
            );
            let panel = flex::column(column).width(panel_width);
            let area = flex::stack(Align2::RIGHT_BOTTOM, [panel]).padding(flex::Insets {
                right: metrics.playerctl_reco_right_inset,
                bottom: Space::S48.px(&metrics) + Space::S8.px(&metrics) + viewport.safe_bottom,
                ..Default::default()
            });
            let placed = area.solve(rect);
            let shifted = |slot: Rect| slot.translate(Vec2::new(slide, 0.0));

            if let (Some(slot), Some(galley)) = (placed.find(BODY), galley) {
                painter.galley(shifted(slot).min, galley, text_primary());
            }
            if let Some(slot) = placed.find(EYEBROW) {
                let slot = shifted(slot);
                draw_text(
                    &painter,
                    slot.left_bottom(),
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
            if let Some(slot) = placed.find(LOGO) {
                let slot = shifted(slot);
                match fitted_logo {
                    Some((texture, _)) => {
                        painter.image(
                            texture,
                            slot,
                            full_uv(),
                            Color32::from_white_alpha(text_alpha),
                        );
                    }
                    None => {
                        if !components::failed_dom_image(
                            &painter,
                            assets,
                            slot,
                            0.0,
                            logo,
                            artwork_target_size(logo_box, context.pixels_per_point()),
                            crate::ImageFit::Contain,
                            [0.0, 1.0],
                        ) {
                            draw_text(
                                &painter,
                                slot.left_bottom(),
                                Align2::LEFT_BOTTOM,
                                &hero.title,
                                FontId::proportional(metrics.text.display),
                                ink(text_alpha),
                                Some(panel_width),
                            );
                        }
                    }
                }
            }
            for dot in 0..dots {
                let Some(hit) = placed.find(DOT_BASE + dot) else {
                    continue;
                };
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
                    hit.center(),
                    if active {
                        Space::S6.px(&metrics)
                    } else {
                        Space::S4.px(&metrics)
                    },
                    ink(if active {
                        255
                    } else if response.hovered() {
                        170
                    } else {
                        90
                    }),
                );
            }
            let buttons = [
                (
                    PLAY,
                    &play_label,
                    components::ButtonKind::Primary,
                    NODE_PLAYER_RECOMMENDATION_PLAY,
                ),
                (
                    DETAILS,
                    &details_label,
                    components::ButtonKind::Secondary,
                    NODE_PLAYER_RECOMMENDATION_DETAILS,
                ),
            ];
            for (id, label, kind, node) in buttons {
                let Some(slot) = placed.find(id) else {
                    continue;
                };
                let response = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(slot), |ui| {
                        components::button_with_text_size(
                            ui,
                            label,
                            slot.width(),
                            slot.height(),
                            kind,
                            button_font,
                            metrics,
                        )
                    })
                    .inner;
                layout.focusable.push((node, response.rect));
                if response.clicked() {
                    layout.activated = Some(node);
                }
            }

            let close = flex::stack(
                Align2::RIGHT_TOP,
                [flex::leaf(Vec2::splat(metrics.icon_button_size)).id(0)],
            )
            .padding(flex::Insets {
                right: metrics.playerctl_dismiss_inset,
                top: metrics.playerctl_dismiss_top,
                ..Default::default()
            })
            .solve(rect);
            if let Some(slot) = close.find(0) {
                let dismiss = control(ui, &painter, slot, "close", false);
                layout
                    .focusable
                    .push((NODE_PLAYER_RECOMMENDATIONS_CLOSE, dismiss.rect));
                if dismiss.clicked() {
                    layout.activated = Some(NODE_PLAYER_RECOMMENDATIONS_CLOSE);
                }
            }

            draw_mini_player(ui, &painter, rect, compact, player, layout);
        });
}
