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
    let margin = metrics.overlay_margin;
    let wide = rect.width() >= 900.0;
    let panel_width = if wide { 440.0 } else { 0.0 };
    let panel_gap = if wide { 56.0 } else { 0.0 };
    let width = (rect.width() - margin * 2.0 - panel_width - panel_gap).min(900.0);
    let left = if wide {
        (rect.width() - width - panel_width - panel_gap) / 2.0 + panel_width + panel_gap
    } else {
        (rect.width() - width) / 2.0
    };
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

            let waiting = player.sources_loading && !player.sources_pending.is_empty();
            let chips_top = top + 62.0 + if waiting { 30.0 } else { 0.0 };
            if waiting {
                ui.painter().text(
                    Pos2::new(left, top + 62.0 + 8.0),
                    Align2::LEFT_TOP,
                    localized("player.streams_waiting", language)
                        .replace("%s", &player.sources_pending.join(", ")),
                    crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0),
                    metrics.text_secondary,
                );
            }
            let chip_size = metrics.screen_card_subtitle_size + 2.0;
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
                            Pos2::new(x, chips_top),
                            Vec2::new(rect.right() - x, metrics.screen_control_height),
                        )),
                        |ui| components::button_auto_width(ui, label, kind, chip_size, metrics),
                    )
                    .inner;
                let node = NODE_PLAYER_SOURCE_FILTER_BASE + index as u64;
                layout.focusable.push((node, response.rect));
                if response.clicked() {
                    layout.activated = Some(node);
                }
                x = response.rect.right() + 8.0;
            }
            if !player.sources_loading {
                let response = ui
                    .scope_builder(
                        egui::UiBuilder::new().max_rect(Rect::from_min_size(
                            Pos2::new(x, chips_top),
                            Vec2::new(rect.right() - x, metrics.screen_control_height),
                        )),
                        |ui| {
                            components::button_auto_width(
                                ui,
                                &localized("player.streams_retry", language),
                                components::ButtonKind::Secondary,
                                chip_size,
                                metrics,
                            )
                        },
                    )
                    .inner;
                layout
                    .focusable
                    .push((NODE_PLAYER_SOURCES_RETRY, response.rect));
                if response.clicked() {
                    layout.activated = Some(NODE_PLAYER_SOURCES_RETRY);
                }
            }

            if wide {
                draw_info_panel(
                    ui,
                    Rect::from_min_size(
                        Pos2::new(left - panel_gap - panel_width, 24.0),
                        Vec2::new(panel_width, rect.height() - 48.0),
                    ),
                    player,
                    metrics,
                    assets,
                );
            }

            let show_addon = player.source_filter.is_none();
            let mut y = chips_top + metrics.screen_control_height + 20.0;
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
                y = response.rect.bottom() + 12.0;
                let node = NODE_PLAYER_SOURCE_BASE + index as u64;
                layout.focusable.push((node, response.rect));
                if response.clicked() {
                    layout.activated = Some(node);
                }
            }
            layout.scroll_max = Some(
                (y + viewport.scroll_y + 24.0 + viewport.safe_bottom - rect.height()).max(0.0),
            );
            if empty && !player.sources_loading {
                ui.painter().text(
                    Pos2::new(left, y + 12.0),
                    Align2::LEFT_TOP,
                    localized("player.streams_empty", language),
                    crate::fonts::regular(metrics.screen_card_subtitle_size),
                    metrics.text_secondary,
                );
            }
        });
}

fn draw_info_panel(
    ui: &mut egui::Ui,
    rect: Rect,
    player: &PlayerModel,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    let painter = ui.painter().clone();
    let ppp = ui.ctx().pixels_per_point();
    let text = |text: &str, font: FontId, color: Color32, rows: usize| {
        let mut job = egui::text::LayoutJob::simple(text.to_owned(), font, color, rect.width());
        job.wrap.max_rows = rows;
        painter.layout_job(job)
    };
    let logo = components::title_logo(
        ppp,
        player.logo.as_deref(),
        Vec2::new(rect.width(), 120.0),
        ArtworkPriority::Visible,
        assets,
    );
    let title = match logo {
        Some(_) => None,
        None => Some(text(
            &player.title,
            FontId::proportional(metrics.screen_title_size),
            Color32::WHITE,
            2,
        )),
    };
    let episode = player.episode_title.as_deref().map(|episode| {
        text(
            episode,
            FontId::proportional(metrics.screen_card_title_size + 6.0),
            Color32::WHITE,
            2,
        )
    });
    let synopsis = player
        .synopsis
        .as_deref()
        .or(player.description.as_deref())
        .map(|synopsis| {
            text(
                synopsis,
                crate::fonts::regular(metrics.screen_body_size + 2.0),
                metrics.text_secondary,
                9,
            )
        });
    let image_height = if player.episode_image.is_some() {
        rect.width() * 9.0 / 16.0 + 28.0
    } else {
        0.0
    };
    let logo_height = logo.map_or(0.0, |(_, size)| size.y + 24.0);
    let title_height = title.as_ref().map_or(0.0, |galley| galley.size().y + 20.0);
    let episode_height = episode
        .as_ref()
        .map_or(0.0, |galley| galley.size().y + 14.0);
    let synopsis_height = synopsis.as_ref().map_or(0.0, |galley| galley.size().y);
    let total = image_height + logo_height + title_height + episode_height + synopsis_height;
    let mut y = (rect.center().y - total / 2.0).max(rect.top());
    if player.episode_image.is_some() {
        let image = Rect::from_min_size(
            Pos2::new(rect.left(), y),
            Vec2::new(rect.width(), rect.width() * 9.0 / 16.0),
        );
        painter.rect_filled(image, metrics.card_radius, metrics.surface);
        components::rounded_artwork(
            &painter,
            image,
            metrics.card_radius,
            player.episode_image.as_deref(),
            artwork_target_size(image.size(), ppp),
            ArtworkPriority::Visible,
            Color32::WHITE,
            assets,
        );
        y = image.bottom() + 28.0;
    }
    if let Some((texture, size)) = logo {
        let logo_rect = Rect::from_min_size(Pos2::new(rect.left(), y), size);
        painter.image(texture, logo_rect, full_uv(), Color32::WHITE);
        y = logo_rect.bottom() + 24.0;
    }
    for (galley, gap, color) in [
        (title, 20.0, Color32::WHITE),
        (episode, 14.0, Color32::WHITE),
        (synopsis, 0.0, metrics.text_secondary),
    ] {
        if let Some(galley) = galley {
            let height = galley.size().y;
            painter.galley(Pos2::new(rect.left(), y), galley, color);
            y += height + gap;
        }
    }
}
