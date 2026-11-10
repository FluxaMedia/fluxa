use super::*;
use crate::dp::Dp;
use crate::draw_text;
use crate::flex;
use crate::kit;
use std::cell::{Cell, RefCell};

enum Placed {
    Skipped(Rect),
    Drawn(crate::gfx::Response, Option<components::StreamAction>),
}

const CONTENT: usize = 0;
const HERO_LOGO: usize = 1;
const HERO_SEASON: usize = 2;
const HERO_EPISODE: usize = 3;
const WAITING: usize = 4;
const CHIPS: usize = 5;
const RETRY: usize = 6;
const CLOSE: usize = 7;
const CHIP_BASE: usize = 100;

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
    context: &crate::gfx::Context,
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
            metrics
                .playerctl_modal
                .of(full.width())
                .min(full.width() - Space::S48.px(&metrics)),
            (full.height() - Space::S32.px(&metrics) * 2.0).min(metrics.playerctl_modal_max_height),
        );
        flex::anchored(full, Align2::CENTER_CENTER, size, 0.0)
    } else {
        context
            .layer_painter(crate::gfx::LayerId::background())
            .rect_filled(full, 0.0, Color32::BLACK);
        full
    };
    let order = if modal {
        crate::gfx::Order::Tooltip
    } else {
        crate::gfx::Order::Middle
    };
    let margin = metrics.overlay_margin;
    let content = flex::column([flex::fill().id(CONTENT)])
        .align(flex::Align::Stretch)
        .padding(flex::Insets::xy(margin, 0.0))
        .solve(rect)
        .find(CONTENT)
        .unwrap_or(rect);
    let (left, width) = (content.left(), content.width());
    let inset = rect.top() + metrics.playerctl_sources_inset;
    let language = &player.language;
    crate::gfx::Area::new(Id::new("fluxa-player-sources"))
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
            ui.scope_builder(crate::gfx::UiBuilder::new().max_rect(rect), |ui| {
                crate::kit::scroll_body_keyed(ui, viewport, "player-sources", (), |ui| {
                    let origin = ui.available_rect_before_wrap().min.y;
                    let hero_height = metrics.player_sources_hero_height + inset - rect.top();
                    let arrow_slot = layout.focusable.len();
                    let waiting = player.sources_loading && !player.sources_pending.is_empty();
                    let show_addon = player.source_filter.is_none();
                    let row_gap = Space::S12.px(&metrics);
                    let side = flex::Insets::xy(margin, 0.0);
                    let heights_id = Id::new("fluxa-player-source-heights");
                    let heights = RefCell::new(
                        ui.ctx()
                            .data_mut(|data| data.get_temp::<HashMap<u64, f32>>(heights_id))
                            .unwrap_or_default(),
                    );
                    let assets = RefCell::new(&mut *assets);
                    let placed_rows = RefCell::new(Vec::new());
                    let chips_top = Cell::new(rect.top());
                    let empty = player.visible_sources().next().is_none();
                    let mut flow = vec![
                        kit::item(Vec2::new(rect.width(), hero_height), |ui, hero| {
                            draw_hero(
                                ui,
                                hero,
                                Rect::from_x_y_ranges(content.x_range(), hero.y_range()),
                                modal,
                                player,
                                metrics,
                                &mut **assets.borrow_mut(),
                            );
                        }),
                        kit::layout_items::gap(Space::S16.px(&metrics)),
                    ];
                    if waiting {
                        flow.push(kit::layout_items::gap(Space::S8.px(&metrics)));
                        flow.push(
                            kit::item(Vec2::new(width, Space::S24.px(&metrics)), |ui, slot| {
                                draw_text(
                                    ui.painter(),
                                    slot.left_top(),
                                    Align2::LEFT_TOP,
                                    localized("player.streams_waiting", language)
                                        .replace("%s", &player.sources_pending.join(", ")),
                                    crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0),
                                    metrics.text_secondary,
                                    None,
                                );
                            })
                            .margin(side),
                        );
                    }
                    flow.push(
                        kit::item(
                            Vec2::new(width, metrics.screen_control_height),
                            |_, slot| chips_top.set(slot.top()),
                        )
                        .margin(side),
                    );
                    flow.push(kit::layout_items::gap(Space::S20.px(&metrics)));
                    for (slot, (index, source)) in player.visible_sources().enumerate() {
                        let key = row_key(source, show_addon, width, metrics);
                        let cached = heights.borrow().get(&key).copied();
                        let height = cached.unwrap_or_else(|| {
                            let mut probe = ui.new_child(
                                crate::gfx::UiBuilder::new()
                                    .max_rect(Rect::from_min_size(
                                        ui.available_rect_before_wrap().min,
                                        Vec2::new(width, f32::INFINITY),
                                    ))
                                    .sizing_pass()
                                    .invisible(),
                            );
                            components::stream_row(
                                &mut probe,
                                source,
                                show_addon,
                                width,
                                metrics,
                                &mut **assets.borrow_mut(),
                            );
                            probe.min_rect().height()
                        });
                        heights.borrow_mut().insert(key, height);
                        let (heights, assets, placed_rows) = (&heights, &assets, &placed_rows);
                        flow.push(
                            kit::item(Vec2::new(width, height), move |ui, row| {
                                if row.bottom() < rect.top() - metrics.playerctl_row_cull
                                    || row.top() > rect.bottom() + metrics.playerctl_row_cull
                                {
                                    placed_rows.borrow_mut().push((index, Placed::Skipped(row)));
                                    return;
                                }
                                let (response, action) = ui
                                    .scope_builder(
                                        crate::gfx::UiBuilder::new()
                                            .id_salt(("source-row", slot))
                                            .max_rect(row),
                                        |ui| {
                                            components::stream_row(
                                                ui,
                                                source,
                                                show_addon,
                                                width,
                                                metrics,
                                                &mut **assets.borrow_mut(),
                                            )
                                        },
                                    )
                                    .inner;
                                heights.borrow_mut().insert(key, response.rect.height());
                                placed_rows
                                    .borrow_mut()
                                    .push((index, Placed::Drawn(response, action)));
                            })
                            .margin(side),
                        );
                        flow.push(kit::layout_items::gap(row_gap));
                    }
                    if empty && !player.sources_loading {
                        let font = crate::fonts::regular(metrics.screen_card_subtitle_size);
                        let message = localized("player.streams_empty", language);
                        let size = flex::text_size(ui.painter(), &message, &font);
                        flow.push(kit::layout_items::gap(Space::S12.px(&metrics)));
                        flow.push(
                            kit::item(size, move |ui, slot| {
                                draw_text(
                                    ui.painter(),
                                    slot.left_top(),
                                    Align2::LEFT_TOP,
                                    message,
                                    font,
                                    metrics.text_secondary,
                                    None,
                                );
                            })
                            .margin(side),
                        );
                    }
                    flow.push(kit::layout_items::gap(
                        Space::S24.px(&metrics) + viewport.safe_bottom,
                    ));
                    kit::lay(ui, flow);
                    let chips_top = chips_top.get();
                    let chips_start = layout.focusable.len();
                    for (index, placed) in placed_rows.into_inner() {
                        let node = NODE_PLAYER_SOURCE_BASE + index as u64;
                        match placed {
                            Placed::Skipped(row) => layout.focusable.push((node, row)),
                            Placed::Drawn(response, action) => {
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
                        }
                    }
                    ui.ctx()
                        .data_mut(|data| data.insert_temp(heights_id, heights.into_inner()));
                    let pinned_top = chips_top.max(inset);
                    let shift = ((inset + Space::S48.px(&metrics) - chips_top)
                        / Space::S48.px(&metrics))
                    .clamp(0.0, 1.0)
                        * Space::S48.px(&metrics);
                    let chips_from = layout.focusable.len();
                    if pinned_top > chips_top {
                        let mut strip = rect;
                        strip.set_bottom(
                            pinned_top
                                + metrics.screen_control_height
                                + metrics.player_sources_strip_lip,
                        );
                        ui.painter().rect_filled(strip, 0.0, shade(Shade::Opaque));
                        paint_vertical_gradient(
                            ui.painter(),
                            Rect::from_min_size(
                                strip.left_bottom(),
                                Vec2::new(strip.width(), metrics.player_sources_strip_fade),
                            ),
                            shade(Shade::Opaque),
                            Color32::TRANSPARENT,
                        );
                    }
                    let chip_size = metrics.screen_card_subtitle_size + 2.0;
                    let all = localized("player.streams_all", language);
                    let addons = player.addons();
                    let filters: Vec<(Option<&str>, &str)> = std::iter::once((None, all.as_str()))
                        .filter(|_| !addons.is_empty())
                        .chain(addons.iter().map(|addon| (Some(*addon), *addon)))
                        .collect();
                    let strip_row = flex::row([
                        flex::leaf(Vec2::splat(metrics.screen_control_height)).id(RETRY),
                        flex::fill().id(CHIPS),
                    ])
                    .gap(Space::S8.px(&metrics))
                    .align(crate::flex::Align::Start)
                    .solve(Rect::from_min_max(
                        Pos2::new(left + shift, pinned_top),
                        Pos2::new(rect.right(), pinned_top + metrics.screen_control_height),
                    ));
                    let retry = strip_row.find(RETRY).unwrap_or(rect);
                    let strip_rect = strip_row.find(CHIPS).unwrap_or(rect);
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
                    ui.scope_builder(
                        crate::gfx::UiBuilder::new()
                            .id_salt("source-chips")
                            .max_rect(strip_rect),
                        |ui| {
                            let output = components::pan_area(
                                ui,
                                "source-chips-scroll",
                                |area| {
                                    area.horizontal_scroll_offset(player.source_chips_offset)
                                        .scroll_source(crate::gfx::ScrollSource::NONE)
                                },
                                |ui| {
                                    let sizes: Vec<Vec2> = filters
                                        .iter()
                                        .map(|(_, label)| {
                                            Vec2::new(
                                                flex::text_size(
                                                    ui.painter(),
                                                    label,
                                                    &FontId::proportional(chip_size),
                                                )
                                                .x + metrics.control_gap * 2.0,
                                                metrics.screen_control_height,
                                            )
                                        })
                                        .collect();
                                    let chips =
                                        flex::row(sizes.iter().enumerate().map(|(index, size)| {
                                            flex::leaf(*size).id(CHIP_BASE + index)
                                        }))
                                        .gap(Space::S8.px(&metrics))
                                        .align(crate::flex::Align::Start);
                                    let area = ui.allocate_space(chips.outer_size()).1;
                                    let placed = chips.solve(area);
                                    for (index, (addon, label)) in filters.iter().enumerate() {
                                        let Some(slot) = placed.find(CHIP_BASE + index) else {
                                            continue;
                                        };
                                        let selected = player.source_filter.as_deref() == *addon;
                                        let kind = if selected {
                                            components::ButtonKind::Selected
                                        } else {
                                            components::ButtonKind::Secondary
                                        };
                                        let backing = ui.painter().add(crate::gfx::Shape::Noop);
                                        let response = ui
                                            .scope_builder(
                                                crate::gfx::UiBuilder::new().max_rect(slot),
                                                |ui| {
                                                    components::button_with_text_size(
                                                        ui,
                                                        label,
                                                        slot.width(),
                                                        slot.height(),
                                                        kind,
                                                        chip_size,
                                                        metrics,
                                                    )
                                                },
                                            )
                                            .inner;
                                        ui.painter().set(
                                            backing,
                                            crate::gfx::Shape::rect_filled(
                                                response.rect,
                                                response.rect.height() * 0.5,
                                                shade(Shade::Opaque),
                                            ),
                                        );
                                        let node = NODE_PLAYER_SOURCE_FILTER_BASE + index as u64;
                                        if focused == Some(node) {
                                            response.scroll_to_me(Some(crate::gfx::Align::Center));
                                        }
                                        components::focus_action(layout, node, &response);
                                    }
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
                    let close =
                        flex::row([flex::leaf(Vec2::splat(metrics.icon_button_size)).id(CLOSE)])
                            .align(flex::Align::Center)
                            .height(metrics.screen_control_height)
                            .solve(Rect::from_min_size(
                                Pos2::new(left, inset),
                                Vec2::new(width, metrics.screen_control_height),
                            ))
                            .find(CLOSE)
                            .unwrap_or(rect);
                    let close = close.translate(Vec2::new(0.0, (natural - close.top()).max(0.0)));
                    if !modal {
                        let back = ui
                            .scope_builder(crate::gfx::UiBuilder::new().max_rect(close), |ui| {
                                components::back_button(
                                    ui,
                                    assets.borrow_mut().icon("ArrowLeft"),
                                    metrics,
                                )
                            })
                            .inner;
                        layout
                            .focusable
                            .insert(arrow_slot, (NODE_PLAYER_CLOSE, close));
                        if back.clicked() {
                            layout.activated = Some(NODE_PLAYER_CLOSE);
                        }
                    }
                });
            });
        });
}

fn paint_eased_fade(painter: &crate::gfx::Painter, rect: Rect) {
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
    ui: &mut crate::gfx::Ui,
    hero: Rect,
    content: Rect,
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
    let width = content.width();
    let episode_font = FontId::proportional(metrics.screen_card_title_size + 2.0);
    let episode = player
        .episode_title
        .as_deref()
        .map(|episode| truncate_to_width(&painter, episode, &episode_font, width));
    let season_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
    let season = player.season_episode.map(|(season, episode)| {
        localized("player.next_episode_number_format", &player.language)
            .replacen("%s", &season.to_string(), 1)
            .replacen("%s", &episode.to_string(), 1)
    });
    let logo = components::title_logo(
        ui.ctx().pixels_per_point(),
        player.logo.as_deref(),
        Vec2::new(
            width.min(metrics.playerctl_sources_logo_width),
            Dp::D64.px(),
        ),
        ArtworkPriority::Visible,
        assets,
    );
    let title_font = crate::fonts::bold(metrics.screen_title_size);
    let logo_size = match &logo {
        Some((_, size)) => *size,
        None => {
            let size = flex::text_size(&painter, &player.title, &title_font);
            Vec2::new(size.x.min(width), size.y)
        }
    };
    let mut stack = vec![flex::leaf(logo_size).id(HERO_LOGO)];
    if let Some(label) = &season {
        stack.push(flex::slot(0.0).height(Space::S10.px(&metrics)));
        stack.push(flex::leaf(flex::text_size(&painter, label, &season_font)).id(HERO_SEASON));
    }
    if let Some(text) = &episode {
        stack.push(flex::slot(0.0).height(Space::S4.px(&metrics)));
        stack.push(flex::leaf(flex::text_size(&painter, text, &episode_font)).id(HERO_EPISODE));
    }
    let placed = flex::column(stack)
        .justify(flex::Justify::End)
        .padding(flex::Insets {
            bottom: Space::S16.px(&metrics),
            ..Default::default()
        })
        .align(crate::flex::Align::Start)
        .solve(content);
    if let Some(slot) = placed.find(HERO_EPISODE) {
        draw_text(
            &painter,
            slot.left_bottom(),
            Align2::LEFT_BOTTOM,
            episode.unwrap_or_default(),
            episode_font,
            text_primary(),
            None,
        );
    }
    if let Some(slot) = placed.find(HERO_SEASON) {
        draw_text(
            &painter,
            slot.left_bottom(),
            Align2::LEFT_BOTTOM,
            season.unwrap_or_default(),
            season_font,
            metrics.text_secondary,
            None,
        );
    }
    let Some(slot) = placed.find(HERO_LOGO) else {
        return;
    };
    match logo {
        Some((texture, _)) => {
            painter.image(texture, slot, full_uv(), Color32::WHITE);
        }
        None => {
            draw_text(
                &painter,
                slot.left_bottom(),
                Align2::LEFT_BOTTOM,
                &player.title,
                title_font,
                text_primary(),
                Some(width),
            );
        }
    }
}
