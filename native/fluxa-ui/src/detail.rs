use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

mod episodes;

const SECTION_TITLE: f32 = 44.0;
const SECTION_GAP: f32 = 36.0;
const SEASON_ROW: f32 = 54.0;
const SEASON_TILE: Vec2 = Vec2::new(84.0, 126.0);
const SEASON_TILE_LABEL: f32 = 30.0;
const EPISODE_WIDTH: f32 = 300.0;
const CAST_SIZE: f32 = 92.0;
const CAST_TEXT: f32 = 48.0;

fn season_row(detail: &DetailModel) -> f32 {
    match detail.season_selector_mode() {
        SeasonSelector::Posters => SEASON_TILE.y + SEASON_TILE_LABEL + 12.0,
        SeasonSelector::Chips | SeasonSelector::Dropdown => SEASON_ROW,
    }
}

fn similar_size(metrics: UiMetrics) -> (Vec2, f32) {
    let poster = Vec2::new(metrics.poster_card_width, metrics.poster_card_height);
    let text = metrics.control_gap * 2.0
        + metrics.screen_card_title_size
        + metrics.screen_card_subtitle_size;
    (poster, poster.y + text)
}

struct DetailGeometry {
    margin: f32,
    hero_height: f32,
    episodes_top: Option<f32>,
    cast_top: Option<f32>,
    similar_top: Option<f32>,
    bottom: f32,
}

fn compact_image_height(viewport: Viewport) -> f32 {
    (viewport.height * 0.56).max(380.0)
}

fn detail_geometry(viewport: Viewport, detail: &DetailModel) -> DetailGeometry {
    let metrics = UiMetrics::for_viewport(viewport);
    let compact = viewport.is_compact();
    let margin = screen_margin(metrics);
    let hero_height = if compact {
        compact_image_height(viewport) + 250.0
    } else {
        (viewport.height * 0.84).clamp(480.0, 980.0)
    };
    let mut y = hero_height + 12.0;
    let episodes_top = (!detail.episodes.is_empty()).then(|| {
        let top = y;
        y += SECTION_TITLE
            + season_row(detail)
            + episodes::episodes_height(viewport, detail, margin)
            + SECTION_GAP;
        top
    });
    let cast_top = (!detail.cast.is_empty() || !detail.crew.is_empty()).then(|| {
        let top = y;
        y += SECTION_TITLE + CAST_SIZE + CAST_TEXT + SECTION_GAP;
        top
    });
    let similar_top = (!detail.similar.is_empty()).then(|| {
        let top = y;
        y += SECTION_TITLE + similar_size(metrics).1 + SECTION_GAP;
        top
    });
    DetailGeometry {
        margin,
        hero_height,
        episodes_top,
        cast_top,
        similar_top,
        bottom: y,
    }
}

static ROW_SCROLL_MAX: [AtomicU32; 4] = [const { AtomicU32::new(0) }; 4];

pub fn detail_row_scroll_max(row: usize) -> f32 {
    ROW_SCROLL_MAX
        .get(row)
        .map_or(0.0, |max| f32::from_bits(max.load(Ordering::Relaxed)))
}

pub fn detail_row_at_y(viewport: Viewport, detail: &DetailModel, y: f32) -> Option<usize> {
    let metrics = UiMetrics::for_viewport(viewport);
    let geometry = detail_geometry(viewport, detail);
    let scroll_y = viewport
        .scroll_y
        .clamp(0.0, detail_scroll_max(viewport, detail));
    let y = y + scroll_y;
    let mut rows = Vec::new();
    if let Some(top) = geometry.episodes_top {
        let seasons = top + SECTION_TITLE;
        rows.push((0, seasons, seasons + season_row(detail)));
        let episodes = seasons + season_row(detail);
        rows.push((
            1,
            episodes,
            episodes + episodes::episodes_height(viewport, detail, geometry.margin),
        ));
    }
    if let Some(top) = geometry.cast_top {
        let cast = top + SECTION_TITLE;
        rows.push((2, cast, cast + CAST_SIZE + CAST_TEXT));
    }
    if let Some(top) = geometry.similar_top {
        let similar = top + SECTION_TITLE;
        rows.push((3, similar, similar + similar_size(metrics).1));
    }
    rows.into_iter()
        .find(|(_, start, end)| (*start..*end).contains(&y))
        .map(|(row, _, _)| row)
}

fn row_scroll(
    viewport: Viewport,
    detail: &DetailModel,
    row: usize,
    id: impl std::hash::Hash,
) -> egui::ScrollArea {
    let area = egui::ScrollArea::horizontal()
        .id_salt(id)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden);
    if viewport.form_factor == UiFormFactor::Mobile {
        area.horizontal_scroll_offset(detail.row_scroll_offsets[row])
    } else {
        area
    }
}

fn record_row_max<R>(row: usize, output: &egui::scroll_area::ScrollAreaOutput<R>) {
    let max = (output.content_size.x - output.inner_rect.width()).max(0.0);
    ROW_SCROLL_MAX[row].store(max.to_bits(), Ordering::Relaxed);
}

pub fn detail_scroll_max(viewport: Viewport, detail: &DetailModel) -> f32 {
    let geometry = detail_geometry(viewport, detail);
    (geometry.bottom - (viewport.height - mobile_scroll_reserve(viewport))).max(0.0)
}

fn section_title(painter: &egui::Painter, pos: Pos2, text: &str, size: f32) {
    painter.text(
        pos,
        Align2::LEFT_TOP,
        text,
        FontId::proportional(size),
        Color32::WHITE,
    );
}

pub fn draw_detail(
    context: &egui::Context,
    viewport: Viewport,
    detail: &DetailModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let geometry = detail_geometry(viewport, detail);
    let scroll_y = resolve_screen_scroll(
        context,
        viewport,
        Id::new("fluxa-screen-scroll-detail"),
        detail_scroll_max(viewport, detail),
    );
    let ppp = context.pixels_per_point();
    let compact = viewport.is_compact();
    let shuffle = detail.episodes.len() > 1;
    let language = detail.language.as_str();
    let t = |key: &str| localized(key, language);
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let margin = geometry.margin;

    let hero = Rect::from_min_size(
        Pos2::new(0.0, -scroll_y),
        Vec2::new(viewport.width, geometry.hero_height),
    );
    let backdrop = detail
        .background_url
        .as_deref()
        .or(detail.poster_url.as_deref());
    let target = backdrop_target_size(viewport.width, ppp);
    let fade =
        context.animate_bool_with_time(Id::new(("fluxa-detail-backdrop", &detail.id)), true, 0.25);
    let image = if compact {
        Rect::from_min_size(
            hero.min,
            Vec2::new(hero.width(), compact_image_height(viewport)),
        )
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
        let mid = image.top() + image.height() * 0.7;
        paint_vertical_gradient(
            &painter,
            Rect::from_min_max(
                Pos2::new(image.left(), image.top() + image.height() * 0.3),
                Pos2::new(image.right(), mid),
            ),
            Color32::TRANSPARENT,
            metrics.background.gamma_multiply(0.9),
        );
        paint_vertical_gradient(
            &painter,
            Rect::from_min_max(Pos2::new(image.left(), mid), image.right_bottom()),
            metrics.background.gamma_multiply(0.9),
            metrics.background,
        );
        painter.rect_filled(
            Rect::from_min_max(image.left_bottom(), hero.right_bottom()),
            0.0,
            metrics.background,
        );
        paint_vertical_gradient(
            &painter,
            Rect::from_min_size(hero.left_bottom(), Vec2::new(hero.width(), 80.0)),
            metrics.background,
            Color32::TRANSPARENT,
        );
    } else {
        paint_hero_scrim(&painter, hero, compact, metrics.background);
        paint_vertical_gradient(
            &painter,
            Rect::from_min_size(hero.left_bottom(), Vec2::new(hero.width(), 200.0)),
            metrics.background,
            Color32::TRANSPARENT,
        );
    }
    paint_vertical_gradient(
        &painter,
        Rect::from_min_size(hero.left_top(), Vec2::new(hero.width(), 120.0)),
        Color32::from_black_alpha(150),
        Color32::TRANSPARENT,
    );

    if let Some(text) = detail.trailer_subtitle.as_deref().filter(|_| detail.trailer.is_some()) {
        crate::paint_trailer_subtitle(context, image, text);
    }

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

    prefetch_artwork(
        detail,
        &season_episodes,
        viewport,
        margin,
        Vec2::new(metrics.poster_card_width, metrics.poster_card_height),
        context.pixels_per_point(),
        assets,
    );

    let content_width = if compact {
        viewport.width - margin * 2.0
    } else {
        (viewport.width * 0.42).clamp(420.0, 560.0)
    };
    let mut action_rects = Vec::new();
    let reveal =
        context.animate_bool_with_time(Id::new(("fluxa-detail-reveal", &detail.id)), true, 0.45);
    egui::Area::new(Id::new("fluxa-detail-hero-content"))
        .constrain(false)
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(Pos2::new(
            margin,
            hero.bottom() - if compact { 24.0 } else { 56.0 } + 16.0 * (1.0 - reveal),
        ))
        .show(context, |ui| {
            ui.multiply_opacity(reveal);
            ui.set_max_width(content_width);
            if compact {
                ui.set_min_width(content_width);
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    let logo = components::title_logo(
                        ppp,
                        detail.logo_url.as_deref(),
                        Vec2::new(content_width.min(260.0), 96.0),
                        ArtworkPriority::Hero,
                        assets,
                    );
                    if let Some((texture, logo_size)) = logo {
                        ui.add(egui::Image::new((texture, logo_size)));
                    } else {
                        ui.add(
                            egui::Label::new(
                                RichText::new(if detail.is_loading && detail.title.is_empty() {
                                    t("common.loading")
                                } else {
                                    detail.title.clone()
                                })
                                .size(metrics.text.display)
                                .strong()
                                .color(Color32::WHITE),
                            )
                            .halign(egui::Align::Center),
                        );
                    }
                    ui.add_space(space::MD);
                    let facts = detail
                        .facts
                        .iter()
                        .cloned()
                        .chain(
                            detail
                                .ratings
                                .iter()
                                .take(1)
                                .map(|(source, score)| format!("{source} {score}")),
                        )
                        .chain(detail.genres.iter().take(2).cloned())
                        .collect::<Vec<_>>()
                        .join("  ·  ");
                    ui.add(
                        egui::Label::new(
                            RichText::new(facts)
                                .size(metrics.text.meta)
                                .color(Color32::WHITE),
                        )
                        .halign(egui::Align::Center),
                    );
                    for line in &detail.info_lines {
                        ui.add_space(space::XS);
                        ui.add(
                            egui::Label::new(
                                RichText::new(line)
                                    .size(metrics.text.meta)
                                    .color(metrics.text_muted),
                            )
                            .halign(egui::Align::Center),
                        );
                    }
                    ui.add_space(space::LG);
                    let resume = detail.resume.as_ref();
                    let label = crate::play_label(
                        language,
                        resume,
                        play_target
                            .filter(|_| detail.is_series())
                            .map(|(_, episode)| {
                                (episode.season, episode.number, Some(episode.title.as_str()))
                            }),
                    );
                    let play = components::play_button(
                        ui,
                        assets,
                        &label,
                        Some(content_width),
                        48.0,
                        metrics.text.subtitle,
                        resume
                            .map(|card| card.progress)
                            .filter(|progress| *progress > 0.0),
                    );
                    action_rects.push((NODE_DETAIL_PLAY, play.rect));
                    if play.clicked() {
                        layout.activated = Some(match play_target {
                            _ if resume.is_some() => NODE_DETAIL_PLAY,
                            Some((index, _)) if detail.is_series() => {
                                NODE_DETAIL_EPISODE_BASE + index as u64
                            }
                            _ => NODE_DETAIL_PLAY,
                        });
                    }
                    ui.add_space(space::LG);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
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
                        if !detail.trailers.is_empty() {
                            actions.push((
                                NODE_DETAIL_TRAILER,
                                "Film",
                                false,
                                t("detail.watch_trailer"),
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
                        let width = content_width / actions.len() as f32;
                        for (node, icon, active, label) in actions {
                            let response = components::labeled_action(
                                ui,
                                assets.icon(icon),
                                &label,
                                width,
                                metrics.text.label,
                                active,
                            );
                            action_rects.push((node, response.rect));
                            if response.clicked() {
                                layout.activated = Some(node);
                            }
                        }
                    });
                });
                ui.add_space(space::LG);
                let expanded_id = Id::new(("fluxa-detail-synopsis", &detail.id));
                let expanded = context
                    .data(|data| data.get_temp::<bool>(expanded_id))
                    .unwrap_or(false);
                let limit = 150;
                let long = detail.description.chars().count() > limit;
                let text = if long && !expanded {
                    format!(
                        "{}…",
                        detail
                            .description
                            .chars()
                            .take(limit)
                            .collect::<String>()
                            .trim_end()
                    )
                } else {
                    detail.description.clone()
                };
                let synopsis = ui.add(
                    egui::Label::new(
                        RichText::new(text)
                            .size(metrics.text.body)
                            .line_height(Some(metrics.text.body * 1.45))
                            .color(metrics.text_primary),
                    )
                    .sense(Sense::click()),
                );
                if long && synopsis.clicked() {
                    context.data_mut(|data| data.insert_temp(expanded_id, !expanded));
                }
                if let Some(error) = detail.streams_error.as_deref().or(detail.error.as_deref()) {
                    ui.add_space(space::SM);
                    ui.label(
                        RichText::new(error)
                            .size(metrics.text.meta)
                            .color(metrics.text_secondary),
                    );
                }
                return;
            }
            ui.set_max_width(content_width);
            ui.spacing_mut().item_spacing = Vec2::new(space::SM, 0.0);
            let logo = components::title_logo(
                ppp,
                detail.logo_url.as_deref(),
                Vec2::new(content_width.min(440.0), 150.0),
                ArtworkPriority::Hero,
                assets,
            );
            if let Some((texture, logo_size)) = logo {
                ui.add(egui::Image::new((texture, logo_size)));
            } else {
                ui.label(
                    RichText::new(if detail.is_loading && detail.title.is_empty() {
                        t("common.loading")
                    } else {
                        detail.title.clone()
                    })
                    .size(metrics.text.display)
                    .strong()
                    .color(Color32::WHITE),
                );
            }
            ui.add_space(space::XL);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(space::SM, space::SM);
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
                for (source, score) in detail.ratings.iter().take(4) {
                    let logo = crate::rating_logo(source, score).and_then(|name| assets.logo(name));
                    if let Some((texture, aspect)) = logo {
                        ui.add(egui::Image::new((texture, aspect * 16.0)))
                            .on_hover_text(source.as_str());
                    } else {
                        ui.label(
                            RichText::new(source)
                                .size(metrics.text.meta)
                                .color(metrics.text_secondary),
                        );
                    }
                    ui.label(
                        RichText::new(score)
                            .size(metrics.text.meta)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(space::XS);
                }
            });
            if !detail.genres.is_empty() {
                ui.add_space(space::SM);
                ui.label(
                    RichText::new(detail.genres.join("  ·  "))
                        .size(metrics.text.meta)
                        .color(metrics.text_primary),
                );
            }
            for line in &detail.info_lines {
                ui.add_space(space::XS);
                ui.label(
                    RichText::new(line)
                        .size(metrics.text.meta)
                        .color(metrics.text_muted),
                );
            }
            ui.add_space(space::XL);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(space::MD, 0.0);
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
                let height = metrics.detail_play_height;
                let play = components::play_button(
                    ui,
                    assets,
                    &play_label,
                    None,
                    height,
                    metrics.text.subtitle,
                    resume
                        .map(|card| card.progress)
                        .filter(|progress| *progress > 0.0),
                );
                action_rects.push((NODE_DETAIL_PLAY, play.rect));
                if play.clicked() {
                    layout.activated = Some(match play_target {
                        _ if resume.is_some() => NODE_DETAIL_PLAY,
                        Some((index, _)) if detail.is_series() => {
                            NODE_DETAIL_EPISODE_BASE + index as u64
                        }
                        _ => NODE_DETAIL_PLAY,
                    });
                }
                let watchlist = components::pill_button(
                    ui,
                    assets.icon(if detail.in_watchlist { "Check" } else { "Plus" }),
                    &t("library.watchlist"),
                    None,
                    height,
                    metrics.text.subtitle,
                    false,
                    None,
                );
                action_rects.push((NODE_DETAIL_WATCHLIST, watchlist.rect));
                if watchlist.clicked() {
                    layout.activated = Some(NODE_DETAIL_WATCHLIST);
                }
                let icons = [
                    (NODE_DETAIL_SHUFFLE, "Shuffle", false, t("common.shuffle")),
                    (
                        NODE_DETAIL_TRAILER,
                        "Film",
                        false,
                        t("detail.watch_trailer"),
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
                for (node, icon, active, hint) in icons
                    .into_iter()
                    .filter(|(node, ..)| *node != NODE_DETAIL_SHUFFLE || shuffle)
                    .filter(|(node, ..)| {
                        *node != NODE_DETAIL_TRAILER || !detail.trailers.is_empty()
                    })
                {
                    let response = components::icon_button_sized(
                        ui,
                        assets.icon(icon),
                        Vec2::splat(height),
                        Color32::WHITE,
                        true,
                        active,
                        true,
                    )
                    .on_hover_text(&hint);
                    action_rects.push((node, response.rect));
                    if response.clicked() {
                        layout.activated = Some(node);
                    }
                }
            });
            if !detail.description.is_empty() {
                ui.add_space(space::XL);
                let expanded_id = Id::new(("fluxa-detail-synopsis", &detail.id));
                let expanded =
                    context.data(|data| data.get_temp::<bool>(expanded_id).unwrap_or(false));
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
                let synopsis = ui.add(
                    egui::Label::new(
                        RichText::new(text)
                            .size(metrics.text.body)
                            .line_height(Some(metrics.text.body * 1.5))
                            .color(metrics.text_primary),
                    )
                    .sense(if long { Sense::click() } else { Sense::hover() }),
                );
                if long && synopsis.clicked() {
                    context.data_mut(|data| data.insert_temp(expanded_id, !expanded));
                }
            }
            if detail.is_loading_streams {
                ui.add_space(space::MD);
                ui.label(
                    RichText::new(t("common.loading"))
                        .size(metrics.text.meta)
                        .color(metrics.text_secondary),
                );
            } else if let Some(error) = detail.streams_error.as_deref().or(detail.error.as_deref())
            {
                ui.add_space(space::MD);
                ui.label(
                    RichText::new(error)
                        .size(metrics.text.meta)
                        .color(metrics.text_secondary),
                );
            }
        });
    layout.focusable.extend(action_rects);

    let back_rect = Rect::from_min_size(
        Pos2::new(margin, metrics.detail_back_top - scroll_y),
        Vec2::splat(40.0),
    );
    egui::Area::new(Id::new("fluxa-detail-back"))
        .constrain(false)
        .fixed_pos(back_rect.min)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let response = components::icon_button(
                ui,
                assets.icon("ArrowLeft"),
                40.0,
                Color32::WHITE,
                true,
                false,
                true,
            )
            .on_hover_text(t("auto.back"));
            if response.clicked() {
                layout.activated = Some(NODE_DETAIL_BACK);
            }
        });
    layout.focusable.push((NODE_DETAIL_BACK, back_rect));

    let title_size = metrics.detail_title_size;
    let row_width = viewport.width - margin;
    let visible = Rect::from_min_max(
        Pos2::new(0.0, 0.0),
        Pos2::new(
            viewport.width,
            viewport.height - mobile_scroll_reserve(viewport),
        ),
    );

    if let Some(top) = geometry.episodes_top {
        let top = top - scroll_y;
        section_title(
            &painter,
            Pos2::new(margin, top),
            &t("auto.episodes"),
            title_size,
        );
        egui::Area::new(Id::new("fluxa-detail-seasons"))
            .constrain(false)
            .fixed_pos(Pos2::new(margin, top + SECTION_TITLE))
            .show(context, |ui| {
                ui.set_clip_rect(visible);
                ui.set_max_width(row_width - margin);
                let output =
                    row_scroll(viewport, detail, 0, "fluxa-detail-season-scroll").show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            let season_label = |season: i64| {
                                if season == 0 {
                                    t("auto.specials")
                                } else {
                                    t("format.season_number").replacen("%s", &season.to_string(), 1)
                                }
                            };
                            let mode = detail.season_selector_mode();
                            if mode == SeasonSelector::Dropdown {
                                let options = seasons
                                    .iter()
                                    .map(|season| (season.to_string(), season_label(*season)))
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
                                        SEASON_TILE,
                                        SEASON_TILE_LABEL,
                                        metrics,
                                    )
                                };
                                layout.focusable.push((
                                    NODE_DETAIL_SEASON_BASE + (*season).clamp(0, 99) as u64,
                                    response.rect,
                                ));
                                if response.clicked() {
                                    layout.activated = Some(
                                        NODE_DETAIL_SEASON_BASE + (*season).clamp(0, 99) as u64,
                                    );
                                }
                            }
                        });
                    });
                record_row_max(0, &output);
            });
        let thumb = Vec2::new(EPISODE_WIDTH, EPISODE_WIDTH * 9.0 / 16.0);
        let episodes_pos = Pos2::new(margin, top + SECTION_TITLE + season_row(detail));
        if detail.episode_layout_for(compact) == EpisodeLayout::Cards {
            egui::Area::new(Id::new("fluxa-detail-episodes"))
                .constrain(false)
                .fixed_pos(episodes_pos)
                .show(context, |ui| {
                    ui.set_clip_rect(visible);
                    ui.set_max_width(row_width);
                    let output = row_scroll(
                        viewport,
                        detail,
                        1,
                        ("fluxa-detail-episode-scroll", selected_season),
                    )
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 16.0;
                            for (index, episode) in &season_episodes {
                                let (rect, response) = ui.allocate_exact_size(
                                    Vec2::new(
                                        EPISODE_WIDTH,
                                        episodes::episodes_height(viewport, detail, margin),
                                    ),
                                    Sense::click(),
                                );
                                let node = NODE_DETAIL_EPISODE_BASE + *index as u64;
                                layout.focusable.push((node, rect));
                                if response.clicked() {
                                    layout.activated = Some(node);
                                }
                                if !ui.is_rect_visible(rect) {
                                    continue;
                                }
                                let image = Rect::from_min_size(rect.min, thumb);
                                let view = detail.episode_view(episode);
                                components::episode_artwork(
                                    ui.painter(),
                                    assets,
                                    image,
                                    episode,
                                    view.hide_still,
                                    response.hovered(),
                                    false,
                                    metrics,
                                );
                                components::episode_text(
                                    ui.painter(),
                                    Pos2::new(rect.left(), image.bottom() + 12.0),
                                    EPISODE_WIDTH,
                                    episode.number,
                                    view.title,
                                    view.overview.map(|text| (text, 2)),
                                );
                            }
                        });
                    });
                    record_row_max(1, &output);
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
            egui::Area::new(Id::new("fluxa-detail-episodes"))
                .constrain(false)
                .fixed_pos(episodes_pos)
                .show(context, |ui| {
                    ui.set_clip_rect(visible);
                    episodes::draw_episodes(ui, &context_data, assets, &mut layout);
                });
        }
    }

    if let Some(top) = geometry.cast_top {
        let top = top - scroll_y;
        section_title(
            &painter,
            Pos2::new(margin, top),
            &t(if detail.crew.is_empty() {
                "auto.cast"
            } else {
                "detail.cast_crew"
            }),
            title_size,
        );
        egui::Area::new(Id::new("fluxa-detail-cast"))
            .constrain(false)
            .fixed_pos(Pos2::new(margin, top + SECTION_TITLE))
            .show(context, |ui| {
                ui.set_clip_rect(visible);
                ui.set_max_width(row_width);
                let output =
                    row_scroll(viewport, detail, 2, "fluxa-detail-cast-scroll").show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 18.0;
                            for member in detail.cast.iter().chain(&detail.crew) {
                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::new(CAST_SIZE + 16.0, CAST_SIZE + CAST_TEXT),
                                    Sense::hover(),
                                );
                                if !ui.is_rect_visible(rect) {
                                    continue;
                                }
                                let painter = ui.painter();
                                let circle = Rect::from_center_size(
                                    Pos2::new(rect.center().x, rect.top() + CAST_SIZE * 0.5),
                                    Vec2::splat(CAST_SIZE),
                                );
                                components::avatar(
                                    painter,
                                    assets,
                                    circle.center(),
                                    CAST_SIZE * 0.5,
                                    &member.name,
                                    member.photo.as_deref(),
                                    metrics,
                                );
                                let font = FontId::proportional(13.5);
                                painter.text(
                                    Pos2::new(rect.center().x, circle.bottom() + 10.0),
                                    Align2::CENTER_TOP,
                                    truncate_to_width(painter, &member.name, &font, rect.width()),
                                    font,
                                    Color32::WHITE,
                                );
                                if let Some(role) = member.role.as_deref() {
                                    let font = FontId::proportional(12.0);
                                    painter.text(
                                        Pos2::new(rect.center().x, circle.bottom() + 28.0),
                                        Align2::CENTER_TOP,
                                        truncate_to_width(painter, role, &font, rect.width()),
                                        font,
                                        metrics.text_muted,
                                    );
                                }
                            }
                        });
                    });
                record_row_max(2, &output);
            });
    }

    if let Some(top) = geometry.similar_top {
        let top = top - scroll_y;
        section_title(
            &painter,
            Pos2::new(margin, top),
            &t("player.recommendations"),
            title_size,
        );
        let (poster, slot_height) = similar_size(metrics);
        egui::Area::new(Id::new("fluxa-detail-similar"))
            .constrain(false)
            .fixed_pos(Pos2::new(margin, top + SECTION_TITLE))
            .show(context, |ui| {
                ui.set_clip_rect(visible);
                ui.set_max_width(row_width);
                let output =
                    row_scroll(viewport, detail, 3, "fluxa-detail-similar-scroll").show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 14.0;
                            for (index, card) in detail.similar.iter().enumerate() {
                                let (rect, response) = ui.allocate_exact_size(
                                    Vec2::new(poster.x, slot_height),
                                    Sense::click(),
                                );
                                let node = NODE_DETAIL_SIMILAR_BASE + index as u64;
                                layout.focusable.push((node, rect));
                                if response.clicked() {
                                    layout.activated = Some(node);
                                }
                                if !ui.is_rect_visible(rect) {
                                    continue;
                                }
                                components::poster_card(
                                    ui.painter(),
                                    Rect::from_min_size(rect.min, poster),
                                    card,
                                    metrics,
                                    assets,
                                    false,
                                );
                            }
                        });
                    });
                record_row_max(3, &output);
            });
    }

    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    components::focus_ring(&painter, &layout.focusable, focused, viewport, metrics);
    layout
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
    let target = |size: Vec2| artwork_target_size(size, pixels_per_point);
    for (_, url) in &detail.season_posters {
        assets.prefetch_for(Some(url), target(SEASON_TILE), ArtworkPriority::Prefetch);
    }
    episodes::prefetch(detail, episodes, viewport, margin, pixels_per_point, assets);
    for member in detail.cast.iter().chain(&detail.crew) {
        assets.prefetch_for(
            member.photo.as_deref(),
            target(Vec2::splat(CAST_SIZE)),
            ArtworkPriority::Prefetch,
        );
    }
    for card in &detail.similar {
        assets.prefetch_for(card.poster_art(), target(poster), ArtworkPriority::Prefetch);
    }
}
