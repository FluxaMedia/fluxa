use super::*;

pub(super) fn draw_sources(
    context: &egui::Context,
    viewport: Viewport,
    rect: Rect,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    let target = backdrop_target_size(rect.width(), context.pixels_per_point());
    components::artwork_image(
        &painter,
        rect,
        player.background.as_deref(),
        target,
        ArtworkPriority::Visible,
        Color32::from_white_alpha(40),
        assets,
    );
    paint_vertical_gradient(
        &painter,
        rect,
        Color32::from_black_alpha(120),
        Color32::from_black_alpha(230),
    );
    let metrics = metrics_for_assets(viewport, assets);
    let margin = if viewport.is_compact() { 16.0 } else { 56.0 };
    let width = (rect.width() - margin * 2.0).min(900.0);
    let left = (rect.width() - width) / 2.0;
    let top = 24.0 - viewport.scroll_y;
    let language = &player.language;
    egui::Area::new(Id::new("fluxa-player-sources"))
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(rect.size());
            let close = Rect::from_min_size(Pos2::new(left, top), Vec2::splat(42.0));
            let painter = ui.painter().clone();
            let response = control(ui, &painter, close, "close", false);
            layout.focusable.push((NODE_PLAYER_CLOSE, response.rect));
            if response.clicked() {
                layout.activated = Some(NODE_PLAYER_CLOSE);
            }
            let title_font = FontId::proportional(metrics.screen_card_title_size + 6.0);
            ui.painter().text(
                Pos2::new(close.right() + 12.0, close.center().y - 10.0),
                Align2::LEFT_CENTER,
                localized("player.streams", language),
                title_font,
                Color32::WHITE,
            );
            let subtitle = player.episode_title.as_deref().unwrap_or(&player.title);
            let subtitle_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
            ui.painter().text(
                Pos2::new(close.right() + 12.0, close.center().y + 12.0),
                Align2::LEFT_CENTER,
                truncate_to_width(ui.painter(), subtitle, &subtitle_font, width - 60.0),
                subtitle_font,
                metrics.text_secondary,
            );

            let all = localized("player.streams_all", language);
            let addons = player.addons();
            let filters = std::iter::once((None, all.as_str()))
                .filter(|_| !addons.is_empty())
                .chain(addons.iter().map(|addon| (Some(*addon), *addon)));
            let mut x = left;
            for (index, (addon, label)) in filters.enumerate() {
                let selected = player.source_filter.as_deref() == addon;
                let kind = if selected {
                    components::ButtonKind::Selected
                } else {
                    components::ButtonKind::Secondary
                };
                let response = ui
                    .scope_builder(
                        egui::UiBuilder::new().max_rect(Rect::from_min_size(
                            Pos2::new(x, top + 62.0),
                            Vec2::new(rect.right() - x, metrics.screen_control_height),
                        )),
                        |ui| {
                            components::button_auto_width(
                                ui,
                                label,
                                kind,
                                metrics.screen_card_subtitle_size,
                                metrics,
                            )
                        },
                    )
                    .inner;
                let node = NODE_PLAYER_SOURCE_FILTER_BASE + index as u64;
                layout.focusable.push((node, response.rect));
                if response.clicked() {
                    layout.activated = Some(node);
                }
                x = response.rect.right() + 8.0;
            }

            let show_addon = player.source_filter.is_none();
            let mut y = top + 62.0 + metrics.screen_control_height + 20.0;
            let mut empty = true;
            for (index, source) in player.visible_sources() {
                empty = false;
                let response = ui
                    .scope_builder(
                        egui::UiBuilder::new().max_rect(Rect::from_min_size(
                            Pos2::new(left, y),
                            Vec2::new(width, f32::INFINITY),
                        )),
                        |ui| components::stream_row(ui, source, show_addon, width, metrics, assets),
                    )
                    .inner;
                y = response.rect.bottom() + 10.0;
                let node = NODE_PLAYER_SOURCE_BASE + index as u64;
                layout.focusable.push((node, response.rect));
                if response.clicked() {
                    layout.activated = Some(node);
                }
            }
            layout.scroll_max = Some(
                (y + viewport.scroll_y + 24.0 + viewport.safe_bottom - rect.height()).max(0.0),
            );
            if empty {
                ui.painter().text(
                    Pos2::new(left, y + 12.0),
                    Align2::LEFT_TOP,
                    localized(
                        if player.sources_loading {
                            "player.streams_loading"
                        } else {
                            "player.streams_empty"
                        },
                        language,
                    ),
                    crate::fonts::regular(metrics.screen_card_subtitle_size),
                    metrics.text_secondary,
                );
            }
        });
}
