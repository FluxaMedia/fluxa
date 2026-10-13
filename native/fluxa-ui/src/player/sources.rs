use super::*;
use crate::draw_text;

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
        Color32::WHITE,
        assets,
    );
    paint_vertical_gradient(
        &painter,
        rect,
        shade(Shade::Faint),
        shade(Shade::Strong),
    );
    let metrics = metrics_for_assets(viewport, assets);
    let margin = metrics.overlay_margin;
    let wide = rect.width() >= 900.0;
    let panel_width = if wide {
        (rect.width() * 0.34).clamp(320.0, 440.0)
    } else {
        0.0
    };
    let panel_gap = if wide {
        (rect.width() * 0.04).clamp(32.0, 56.0)
    } else {
        0.0
    };
    let width = (rect.width() - margin * 2.0 - panel_width - panel_gap).min(900.0);
    let left = if wide {
        (rect.width() - width - panel_width - panel_gap) / 2.0 + panel_width + panel_gap
    } else {
        (rect.width() - width) / 2.0
    };
    let inset = if viewport.is_compact() {
        metrics.content_header_top_mobile - 8.0
    } else {
        24.0
    };
    let top = 24.0 - viewport.scroll_y;
    let language = &player.language;
    egui::Area::new(Id::new("fluxa-player-sources"))
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(rect.size());
            let origin = -viewport.scroll_y;
            let hero_height = if wide { 0.0 } else { 240.0 + inset };
            let hero = Rect::from_min_size(
                Pos2::new(rect.left(), origin),
                Vec2::new(rect.width(), hero_height),
            );
            if !wide {
                draw_hero(ui, hero, left, width, player, metrics, assets);
            }
            let arrow_slot = layout.focusable.len();
            let base = if wide {
                top + 62.0
            } else {
                hero.bottom() + 16.0
            };
            let waiting = player.sources_loading && !player.sources_pending.is_empty();
            let chips_top = base + if waiting { 30.0 } else { 0.0 };
            if waiting {
                draw_text(
                    ui.painter(),
                    Pos2::new(left, base + 8.0),
                    Align2::LEFT_TOP,
                    localized("player.streams_waiting", language)
                        .replace("%s", &player.sources_pending.join(", ")),
                    crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0),
                    metrics.text_secondary,
                    None,
                );
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

            let chips_start = layout.focusable.len();
            let show_addon = player.source_filter.is_none();
            let mut y = chips_top + metrics.screen_control_height + 20.0;
            let mut empty = true;
            for (slot, (index, source)) in player.visible_sources().enumerate() {
                empty = false;
                let response = ui
                    .scope_builder(
                        egui::UiBuilder::new()
                            .id_salt(("source-row", slot))
                            .max_rect(Rect::from_min_size(
                                Pos2::new(left, y),
                                Vec2::new(width, f32::INFINITY),
                            )),
                        |ui| components::stream_row(ui, source, show_addon, width, metrics, assets),
                    )
                    .inner;
                y = response.rect.bottom() + 12.0;
                let node = NODE_PLAYER_SOURCE_BASE + index as u64;
                components::focus_action(layout, node, &response);
            }
            let pinned_top = chips_top.max(inset);
            let shift = ((inset + 42.0 - chips_top) / 42.0).clamp(0.0, 1.0) * 50.0;
            let chips_from = layout.focusable.len();
            if pinned_top > chips_top {
                let (strip_left, strip_right) = if wide {
                    (left - 12.0, left + width + 12.0)
                } else {
                    (rect.left(), rect.right())
                };
                let strip = Rect::from_min_max(
                    Pos2::new(strip_left, rect.top()),
                    Pos2::new(
                        strip_right,
                        pinned_top + metrics.screen_control_height + 10.0,
                    ),
                );
                ui.painter().rect_filled(strip, 0.0, shade(Shade::Opaque));
                paint_vertical_gradient(
                    ui.painter(),
                    Rect::from_min_max(
                        Pos2::new(strip.left(), strip.bottom()),
                        Pos2::new(strip.right(), strip.bottom() + 14.0),
                    ),
                    shade(Shade::Opaque),
                    Color32::TRANSPARENT,
                );
            }
            let chip_size = metrics.screen_card_subtitle_size + 2.0;
            let all = localized("player.streams_all", language);
            let addons = player.addons();
            let filters = std::iter::once((None, all.as_str()))
                .filter(|_| !addons.is_empty())
                .chain(addons.iter().map(|addon| (Some(*addon), *addon)));
            let retry = Rect::from_min_size(
                Pos2::new(left + shift, pinned_top),
                Vec2::splat(metrics.screen_control_height),
            );
            let painter = ui.painter().clone();
            let response = control_with_id(
                ui,
                &painter,
                retry,
                if player.sources_loading {
                    "retry-spin"
                } else {
                    "retry"
                },
                true,
                Id::new("fluxa-player-sources-retry"),
            );
            layout
                .focusable
                .push((NODE_PLAYER_SOURCES_RETRY, response.rect));
            if response.clicked() {
                layout.activated = Some(NODE_PLAYER_SOURCES_RETRY);
            }
            let mut x = retry.right() + 8.0;
            for (index, (addon, label)) in filters.enumerate() {
                let selected = player.source_filter.as_deref() == addon;
                let kind = if selected {
                    components::ButtonKind::Selected
                } else {
                    components::ButtonKind::Secondary
                };
                let response = ui
                    .scope_builder(
                        egui::UiBuilder::new()
                            .id_salt(("source-chip", index))
                            .max_rect(Rect::from_min_size(
                                Pos2::new(x, pinned_top),
                                Vec2::new(rect.right() - x, metrics.screen_control_height),
                            )),
                        |ui| components::button_auto_width(ui, label, kind, chip_size, metrics),
                    )
                    .inner;
                let node = NODE_PLAYER_SOURCE_FILTER_BASE + index as u64;
                components::focus_action(layout, node, &response);
                x = response.rect.right() + 8.0;
            }
            let chip_count = layout.focusable.len() - chips_from;
            layout.focusable[chips_start..].rotate_right(chip_count);
            let natural = if wide { top } else { origin + inset };
            let close = Rect::from_min_size(
                Pos2::new(
                    left,
                    natural.max(
                        inset + (metrics.screen_control_height - metrics.icon_button_size) / 2.0,
                    ),
                ),
                Vec2::splat(metrics.icon_button_size),
            );
            let back = ui
                .scope_builder(egui::UiBuilder::new().max_rect(close), |ui| {
                    components::back_button(ui, assets.icon("ArrowLeft"), metrics)
                })
                .inner;
            layout
                .focusable
                .insert(arrow_slot, (NODE_PLAYER_CLOSE, close));
            if back.clicked() {
                layout.activated = Some(NODE_PLAYER_CLOSE);
            }

            layout.scroll_max = Some(
                (y + viewport.scroll_y + 24.0 + viewport.safe_bottom - rect.height()).max(0.0),
            );
            if empty && !player.sources_loading {
                draw_text(
                    ui.painter(),
                    Pos2::new(left, y + 12.0),
                    Align2::LEFT_TOP,
                    localized("player.streams_empty", language),
                    crate::fonts::regular(metrics.screen_card_subtitle_size),
                    metrics.text_secondary,
                    None,
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
            crate::fonts::bold(metrics.screen_title_size),
            text_primary(),
            2,
        )),
    };
    let episode = player.episode_title.as_deref().map(|episode| {
        text(
            episode,
            FontId::proportional(metrics.screen_card_title_size + 6.0),
            text_primary(),
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
        (title, 20.0, text_primary()),
        (episode, 14.0, text_primary()),
        (synopsis, 0.0, metrics.text_secondary),
    ] {
        if let Some(galley) = galley {
            let height = galley.size().y;
            painter.galley(Pos2::new(rect.left(), y), galley, color);
            y += height + gap;
        }
    }
}

fn draw_hero(
    ui: &mut egui::Ui,
    hero: Rect,
    left: f32,
    width: f32,
    player: &PlayerModel,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    let painter = ui.painter().clone();
    let ppp = ui.ctx().pixels_per_point();
    components::artwork_image(
        &painter,
        hero,
        player
            .episode_image
            .as_deref()
            .or(player.background.as_deref()),
        backdrop_target_size(hero.width(), ppp),
        ArtworkPriority::Visible,
        Color32::WHITE,
        assets,
    );
    paint_vertical_gradient(&painter, hero, Color32::TRANSPARENT, Color32::BLACK);
    let mut y = hero.bottom() - 16.0;
    if let Some(episode) = player.episode_title.as_deref() {
        let font = FontId::proportional(metrics.screen_card_title_size + 2.0);
        let text = truncate_to_width(&painter, episode, &font, width);
        let rect = draw_text(
            &painter,
            Pos2::new(left, y),
            Align2::LEFT_BOTTOM,
            text,
            font,
            text_primary(),
            None,
        );
        y = rect.top() - 4.0;
    }
    if let Some((season, episode)) = player.season_episode {
        let label = localized("player.next_episode_number_format", &player.language)
            .replacen("%s", &season.to_string(), 1)
            .replacen("%s", &episode.to_string(), 1);
        let rect = draw_text(
            &painter,
            Pos2::new(left, y),
            Align2::LEFT_BOTTOM,
            label,
            crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0),
            metrics.text_secondary,
            None,
        );
        y = rect.top() - 10.0;
    }
    let logo = components::title_logo(
        ppp,
        player.logo.as_deref(),
        Vec2::new(width * 0.6, 64.0),
        ArtworkPriority::Visible,
        assets,
    );
    match logo {
        Some((texture, size)) => {
            let rect = Rect::from_min_size(Pos2::new(left, y - size.y), size);
            painter.image(texture, rect, full_uv(), Color32::WHITE);
        }
        None => {
            let font = crate::fonts::bold(metrics.screen_title_size);
            draw_text(
                &painter,
                Pos2::new(left, y),
                Align2::LEFT_BOTTOM,
                &player.title,
                font,
                text_primary(),
                Some(width),
            );
        }
    }
}
