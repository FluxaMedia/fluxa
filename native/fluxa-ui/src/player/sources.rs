use super::*;
use crate::draw_text;

const MODAL_MAX_HEIGHT: f32 = 900.0;
const ROW_CULL_MARGIN: f32 = 240.0;

fn row_key(source: &PlayerSource, show_addon: bool, width: f32, metrics: UiMetrics) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.name.hash(&mut hasher);
    source.addon.hash(&mut hasher);
    source.detail.hash(&mut hasher);
    source.badges.len().hash(&mut hasher);
    for badge in &source.badges {
        badge.label.hash(&mut hasher);
    }
    source.actions.hash(&mut hasher);
    source.badges_top.hash(&mut hasher);
    show_addon.hash(&mut hasher);
    width.to_bits().hash(&mut hasher);
    metrics.screen_card_title_size.to_bits().hash(&mut hasher);
    metrics
        .screen_card_subtitle_size
        .to_bits()
        .hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn sources_modal(viewport: Viewport) -> bool {
    viewport.width >= 900.0
}

pub(super) fn draw_sources(
    context: &egui::Context,
    viewport: Viewport,
    rect: Rect,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
    focused: Option<u64>,
) {
    let metrics = metrics_for_assets(viewport, assets);
    let modal = sources_modal(viewport);
    let full = rect;
    let rect = if modal {
        let size = Vec2::new(
            (full.width() * 0.6)
                .clamp(720.0, 1000.0)
                .min(full.width() - 48.0),
            (full.height() - 64.0).min(MODAL_MAX_HEIGHT),
        );
        Rect::from_center_size(full.center(), size)
    } else {
        context
            .layer_painter(egui::LayerId::background())
            .rect_filled(full, 0.0, Color32::BLACK);
        full
    };
    let order = if modal {
        egui::Order::Tooltip
    } else {
        egui::Order::Middle
    };
    let margin = metrics.overlay_margin;
    let width = rect.width() - margin * 2.0;
    let left = rect.left() + margin;
    let inset = rect.top()
        + if viewport.is_compact() {
            metrics.content_header_top_mobile - 8.0
        } else {
            24.0
        };
    let language = &player.language;
    egui::Area::new(Id::new("fluxa-player-sources"))
        .fixed_pos(Pos2::ZERO)
        .order(order)
        .show(context, |ui| {
            if modal {
                let scrim =
                    ui.interact(full, Id::new("fluxa-player-sources-scrim"), Sense::click());
                ui.painter()
                    .rect_filled(full, 0.0, Color32::from_black_alpha(150));
                ui.painter()
                    .rect_filled(rect, metrics.card_radius, Color32::BLACK);
                let outside = scrim
                    .interact_pointer_pos()
                    .is_some_and(|pos| !rect.contains(pos));
                if scrim.clicked() && outside {
                    layout.activated = Some(NODE_PLAYER_CLOSE);
                }
            }
            ui.set_clip_rect(rect);
            ui.set_min_size(rect.size());
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                crate::kit::scroll_body_keyed(ui, viewport, "player-sources", (), |ui| {
                    let origin = ui.cursor().min.y;
                    let hero_height = 180.0 + inset - rect.top();
                    let hero = ui.allocate_space(Vec2::new(rect.width(), hero_height)).1;
                    draw_hero(ui, hero, left, width, modal, player, metrics, assets);
                    let arrow_slot = layout.focusable.len();
                    let base = hero.bottom() + 16.0;
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
                    let chips_start = layout.focusable.len();
                    let show_addon = player.source_filter.is_none();
                    ui.allocate_space(Vec2::new(
                        rect.width(),
                        chips_top + metrics.screen_control_height + 20.0 - hero.bottom(),
                    ));
                    let mut empty = true;
                    let heights_id = Id::new("fluxa-player-source-heights");
                    let mut heights = ui
                        .ctx()
                        .data_mut(|data| data.get_temp::<HashMap<u64, f32>>(heights_id))
                        .unwrap_or_default();
                    for (slot, (index, source)) in player.visible_sources().enumerate() {
                        empty = false;
                        let y = ui.cursor().min.y;
                        let key = row_key(source, show_addon, width, metrics);
                        if let Some(&height) = heights.get(&key)
                            && (y + height < rect.top() - ROW_CULL_MARGIN
                                || y > rect.bottom() + ROW_CULL_MARGIN)
                        {
                            let skipped =
                                Rect::from_min_size(Pos2::new(left, y), Vec2::new(width, height));
                            layout
                                .focusable
                                .push((NODE_PLAYER_SOURCE_BASE + index as u64, skipped));
                            ui.allocate_space(Vec2::new(rect.width(), height + 12.0));
                            continue;
                        }
                        let (response, action) = ui
                            .scope_builder(
                                egui::UiBuilder::new()
                                    .id_salt(("source-row", slot))
                                    .max_rect(Rect::from_min_size(
                                        Pos2::new(left, y),
                                        Vec2::new(width, f32::INFINITY),
                                    )),
                                |ui| {
                                    components::stream_row(
                                        ui, source, show_addon, width, metrics, assets,
                                    )
                                },
                            )
                            .inner;
                        heights.insert(key, response.rect.height());
                        ui.allocate_space(Vec2::new(rect.width(), response.rect.height() + 12.0));
                        let node = NODE_PLAYER_SOURCE_BASE + index as u64;
                        components::focus_action(layout, node, &response);
                        match action {
                            Some(components::StreamAction::Download) => {
                                layout.activated =
                                    Some(NODE_PLAYER_SOURCE_DOWNLOAD_BASE + index as u64);
                            }
                            Some(components::StreamAction::Copy) => {
                                layout.activated =
                                    Some(NODE_PLAYER_SOURCE_COPY_BASE + index as u64);
                            }
                            None => {}
                        }
                    }
                    ui.ctx()
                        .data_mut(|data| data.insert_temp(heights_id, heights));
                    let pinned_top = chips_top.max(inset);
                    let shift = ((inset + 42.0 - chips_top) / 42.0).clamp(0.0, 1.0) * 50.0;
                    let chips_from = layout.focusable.len();
                    if pinned_top > chips_top {
                        let strip = Rect::from_min_max(
                            Pos2::new(rect.left(), rect.top()),
                            Pos2::new(
                                rect.right(),
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
                    let strip_rect = Rect::from_min_max(
                        Pos2::new(retry.right() + 8.0, pinned_top),
                        Pos2::new(rect.right(), pinned_top + metrics.screen_control_height),
                    );
                    ui.scope_builder(
                        egui::UiBuilder::new()
                            .id_salt("source-chips")
                            .max_rect(strip_rect),
                        |ui| {
                            let output = components::pan_area(
                                ui,
                                "source-chips-scroll",
                                |area| {
                                    area.horizontal_scroll_offset(player.source_chips_offset)
                                        .scroll_source(egui::scroll_area::ScrollSource::NONE)
                                },
                                |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 8.0;
                                        for (index, (addon, label)) in filters.enumerate() {
                                            let selected = player.source_filter.as_deref() == addon;
                                            let kind = if selected {
                                                components::ButtonKind::Selected
                                            } else {
                                                components::ButtonKind::Secondary
                                            };
                                            let backing = ui.painter().add(egui::Shape::Noop);
                                            let response = components::button_auto_width(
                                                ui, label, kind, chip_size, metrics,
                                            );
                                            ui.painter().set(
                                                backing,
                                                egui::Shape::rect_filled(
                                                    response.rect,
                                                    response.rect.height() * 0.5,
                                                    shade(Shade::Opaque),
                                                ),
                                            );
                                            let node =
                                                NODE_PLAYER_SOURCE_FILTER_BASE + index as u64;
                                            if focused == Some(node) {
                                                response.scroll_to_me(Some(egui::Align::Center));
                                            }
                                            components::focus_action(layout, node, &response);
                                        }
                                    });
                                },
                            );
                            crate::report_scroll_region(
                                ui.ctx(),
                                crate::ScrollRegion {
                                    id: crate::ScrollId::Chips,
                                    axis: crate::ScrollAxis::Horizontal,
                                    rect: strip_rect,
                                    max: (output.content_size.x - output.inner_rect.width())
                                        .max(0.0),
                                },
                            );
                        },
                    );
                    let chip_count = layout.focusable.len() - chips_from;
                    layout.focusable[chips_start..].rotate_right(chip_count);
                    let natural = origin + inset;
                    let close = Rect::from_min_size(
                        Pos2::new(
                            left,
                            natural.max(
                                inset
                                    + (metrics.screen_control_height - metrics.icon_button_size)
                                        / 2.0,
                            ),
                        ),
                        Vec2::splat(metrics.icon_button_size),
                    );
                    if !modal {
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
                    }

                    if empty && !player.sources_loading {
                        draw_text(
                            ui.painter(),
                            Pos2::new(left, ui.cursor().min.y + 12.0),
                            Align2::LEFT_TOP,
                            localized("player.streams_empty", language),
                            crate::fonts::regular(metrics.screen_card_subtitle_size),
                            metrics.text_secondary,
                            None,
                        );
                    }
                    ui.add_space(24.0 + viewport.safe_bottom);
                });
            });
        });
}

fn paint_eased_fade(painter: &egui::Painter, rect: Rect) {
    const BANDS: usize = 16;
    let alpha = |t: f32| (t * t * (3.0 - 2.0 * t) * 255.0) as u8;
    let band = rect.height() / BANDS as f32;
    for i in 0..BANDS {
        let top = rect.top() + band * i as f32;
        let part = Rect::from_min_max(
            Pos2::new(rect.left(), top),
            Pos2::new(rect.right(), (top + band + 0.5).min(rect.bottom())),
        );
        paint_vertical_gradient(
            painter,
            part,
            Color32::from_black_alpha(alpha(i as f32 / BANDS as f32)),
            Color32::from_black_alpha(alpha((i + 1) as f32 / BANDS as f32)),
        );
    }
}

fn draw_hero(
    ui: &mut egui::Ui,
    hero: Rect,
    left: f32,
    width: f32,
    modal: bool,
    player: &PlayerModel,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    let painter = ui.painter().clone();
    let ppp = ui.ctx().pixels_per_point();
    let backdrop = player
        .episode_image
        .as_deref()
        .or(player.background.as_deref());
    if modal {
        components::rounded_artwork(
            &painter,
            hero,
            metrics.card_radius,
            backdrop,
            backdrop_target_size(hero.width(), ppp),
            ArtworkPriority::Visible,
            Color32::WHITE,
            assets,
        );
    } else {
        components::artwork_image(
            &painter,
            hero,
            backdrop,
            backdrop_target_size(hero.width(), ppp),
            ArtworkPriority::Visible,
            Color32::WHITE,
            assets,
        );
    }
    paint_eased_fade(&painter, hero);
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
