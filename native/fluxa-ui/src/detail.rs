use super::*;

const SECTION_TITLE: f32 = 44.0;
const SECTION_GAP: f32 = 36.0;
const SEASON_ROW: f32 = 54.0;
const EPISODE_WIDTH: f32 = 300.0;
const EPISODE_TEXT: f32 = 74.0;
const CAST_SIZE: f32 = 92.0;
const CAST_TEXT: f32 = 48.0;

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

fn detail_geometry(viewport: Viewport, detail: &DetailModel) -> DetailGeometry {
    let metrics = UiMetrics::for_viewport(viewport);
    let compact = viewport.is_compact();
    let margin = screen_margin(viewport, metrics);
    let hero_height = if compact {
        (viewport.height * 0.78).max(520.0)
    } else {
        (viewport.height * 0.84).clamp(480.0, 980.0)
    };
    let mut y = hero_height + 12.0;
    let episodes_top = (!detail.episodes.is_empty()).then(|| {
        let top = y;
        y += SECTION_TITLE + SEASON_ROW + EPISODE_WIDTH * 9.0 / 16.0 + EPISODE_TEXT + SECTION_GAP;
        top
    });
    let cast_top = (!detail.cast.is_empty()).then(|| {
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

pub fn detail_scroll_max(viewport: Viewport, detail: &DetailModel) -> f32 {
    let geometry = detail_geometry(viewport, detail);
    (geometry.bottom - (viewport.height - mobile_scroll_reserve(viewport))).max(0.0)
}

fn icon_button(
    ui: &mut egui::Ui,
    icon: Option<TextureId>,
    active: bool,
    hint: &str,
    size: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let fill = if active {
        Color32::from_white_alpha(46)
    } else if response.hovered() {
        Color32::from_white_alpha(34)
    } else {
        Color32::from_white_alpha(18)
    };
    ui.painter().circle_filled(rect.center(), size * 0.5, fill);
    ui.painter().circle_stroke(
        rect.center(),
        size * 0.5 - 0.5,
        egui::Stroke::new(1.0, Color32::from_white_alpha(if active { 90 } else { 36 })),
    );
    if let Some(icon) = icon {
        ui.painter().image(
            icon,
            Rect::from_center_size(rect.center(), Vec2::splat(size * 0.44)),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    response.on_hover_text(hint)
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
    let tv = viewport.is_tv();
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
    let backdrop = detail.background_url.as_deref().or(detail.poster_url.as_deref());
    let target = backdrop_target_size(viewport.width, ppp);
    let fade = context.animate_bool_with_time(Id::new(("fluxa-detail-backdrop", &detail.id)), true, 0.25);
    components::artwork_image(
        &painter,
        hero,
        backdrop,
        target,
        ArtworkPriority::Hero,
        Color32::from_white_alpha((225.0 * fade) as u8),
        assets,
    );
    if let Some(texture) = detail.trailer {
        crate::paint_trailer(context, &painter, texture, hero, Some(detail.id.as_str()), 225);
    }
    paint_hero_scrim(&painter, hero, compact, metrics.background);
    paint_vertical_gradient(
        &painter,
        Rect::from_min_size(hero.left_top(), Vec2::new(hero.width(), 120.0)),
        Color32::from_black_alpha(150),
        Color32::TRANSPARENT,
    );

    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 0, assets);

    let seasons = detail.seasons();
    let season_id = Id::new(("fluxa-detail-season", &detail.id));
    let selected_season = context
        .data(|data| data.get_temp::<i64>(season_id))
        .filter(|season| seasons.contains(season))
        .or_else(|| seasons.iter().copied().find(|season| *season > 0))
        .or_else(|| seasons.first().copied())
        .unwrap_or(1);
    let season_episodes = detail
        .episodes
        .iter()
        .enumerate()
        .filter(|(_, episode)| episode.season == selected_season)
        .collect::<Vec<_>>();
    let play_target = season_episodes.first().map(|(index, episode)| (*index, *episode));

    let content_width = if compact {
        viewport.width - margin * 2.0
    } else {
        (viewport.width * 0.46).clamp(420.0, 720.0)
    };
    let mut action_rects = Vec::new();
    let reveal = context.animate_bool_with_time(Id::new(("fluxa-detail-reveal", &detail.id)), true, 0.45);
    egui::Area::new(Id::new("fluxa-detail-hero-content"))
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(Pos2::new(margin, hero.bottom() - if compact { 24.0 } else { 56.0 } + 16.0 * (1.0 - reveal)))
        .show(context, |ui| {
            ui.multiply_opacity(reveal);
            ui.set_max_width(content_width);
            ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);
            let logo_box = Vec2::new(content_width.min(if compact { 260.0 } else { 440.0 }), if compact { 96.0 } else { 150.0 });
            let logo = components::title_logo(ppp, detail.logo_url.as_deref(), logo_box, ArtworkPriority::Hero, assets);
            if let Some((texture, logo_size)) = logo {
                ui.add(egui::Image::new((texture, logo_size)));
            } else {
                ui.label(
                    RichText::new(if detail.is_loading && detail.title.is_empty() {
                        t("common.loading")
                    } else {
                        detail.title.clone()
                    })
                    .size(if compact { 30.0 } else if tv { 54.0 } else { 46.0 })
                    .strong()
                    .color(Color32::WHITE),
                );
            }
            ui.add_space(16.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                for fact in &detail.facts {
                    ui.label(RichText::new(fact).size(15.0).strong().color(Color32::from_white_alpha(225)));
                    ui.label(RichText::new("·").size(15.0).color(Color32::from_white_alpha(120)));
                }
                for (source, score) in detail.ratings.iter().take(4) {
                    egui::Frame::new()
                        .fill(Color32::from_white_alpha(16))
                        .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(30)))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            let logo = crate::rating_logo(source, score)
                                .and_then(|name| assets.logo(name));
                            let Some((texture, aspect)) = logo else {
                                ui.label(
                                    RichText::new(format!("{source}  {score}"))
                                        .size(12.5)
                                        .strong()
                                        .color(Color32::WHITE),
                                );
                                return;
                            };
                            ui.spacing_mut().item_spacing.x = 6.0;
                            ui.add(egui::Image::new((texture, aspect * 16.0)))
                                .on_hover_text(source.as_str());
                            ui.label(RichText::new(score).size(12.5).strong().color(Color32::WHITE));
                        });
                }
            });
            if !detail.genres.is_empty() {
                ui.add_space(10.0);
                ui.label(
                    RichText::new(detail.genres.join("  ·  "))
                        .size(14.0)
                        .color(Color32::from_white_alpha(165)),
                );
            }
            ui.add_space(16.0);
            let description = if detail.description.chars().count() > 360 {
                format!("{}…", detail.description.chars().take(357).collect::<String>().trim_end())
            } else {
                detail.description.clone()
            };
            ui.label(
                RichText::new(description)
                    .size(if tv { 19.0 } else { 16.0 })
                    .line_height(Some(if tv { 28.0 } else { 24.0 }))
                    .color(Color32::from_white_alpha(200)),
            );
            if let Some(error) = detail.error.as_deref() {
                ui.add_space(10.0);
                ui.label(RichText::new(error).color(Color32::from_white_alpha(150)));
            }
            ui.add_space(24.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(12.0, 12.0);
                let resume = detail.resume.as_ref();
                let play_label = crate::play_label(
                    language,
                    resume,
                    play_target
                        .filter(|_| detail.is_series())
                        .map(|(_, episode)| (episode.season, episode.number)),
                );
                let play = components::play_button(
                    ui,
                    assets,
                    &play_label,
                    compact.then_some(content_width),
                    48.0,
                    17.0,
                    resume.map(|card| card.progress).filter(|progress| *progress > 0.0),
                );
                let rect = play.rect;
                action_rects.push((NODE_DETAIL_PLAY, rect));
                if play.clicked() {
                    layout.activated = Some(match play_target {
                        _ if resume.is_some() => NODE_DETAIL_PLAY,
                        Some((index, _)) if detail.is_series() => NODE_DETAIL_EPISODE_BASE + index as u64,
                        _ => NODE_DETAIL_PLAY,
                    });
                }
                for (node, icon, active, hint) in [
                    (
                        NODE_DETAIL_WATCHLIST,
                        if detail.in_watchlist { "Check" } else { "Plus" },
                        detail.in_watchlist,
                        t("library.watchlist"),
                    ),
                    (NODE_DETAIL_COMPLETED, "CircleCheck", detail.completed, t("library.completed")),
                    (NODE_DETAIL_DROPPED, "Ban", detail.dropped, t("library.dropped")),
                    (
                        NODE_DETAIL_FAVORITE,
                        if detail.favorite { "HeartFilled" } else { "Heart" },
                        detail.favorite,
                        t("library.favorites"),
                    ),
                ] {
                    let response = icon_button(ui, assets.icon(icon), active, &hint, 48.0);
                    action_rects.push((node, response.rect));
                    if response.clicked() {
                        layout.activated = Some(node);
                    }
                }
            });
            if detail.is_loading_streams {
                ui.add_space(12.0);
                ui.label(RichText::new(t("common.loading")).size(14.0).color(Color32::from_white_alpha(150)));
            } else if let Some(error) = detail.streams_error.as_deref() {
                ui.add_space(12.0);
                ui.label(RichText::new(error).size(14.0).color(Color32::from_white_alpha(150)));
            }
        });
    layout.focusable.extend(action_rects);

    let back_rect = Rect::from_min_size(Pos2::new(margin, if compact { 12.0 } else { 14.0 }), Vec2::splat(40.0));
    egui::Area::new(Id::new("fluxa-detail-back"))
        .fixed_pos(back_rect.min)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let response = icon_button(ui, assets.icon("ArrowLeft"), false, &t("auto.back"), 40.0);
            if response.clicked() {
                layout.activated = Some(NODE_DETAIL_BACK);
            }
        });
    layout.focusable.push((NODE_DETAIL_BACK, back_rect));

    let title_size = if tv { 26.0 } else { 21.0 };
    let row_width = viewport.width - margin;
    let visible = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(viewport.width, viewport.height - mobile_scroll_reserve(viewport)));

    if let Some(top) = geometry.episodes_top {
        let top = top - scroll_y;
        section_title(&painter, Pos2::new(margin, top), &t("auto.episodes"), title_size);
        egui::Area::new(Id::new("fluxa-detail-seasons"))
            .constrain(false)
            .fixed_pos(Pos2::new(margin, top + SECTION_TITLE))
            .show(context, |ui| {
                ui.set_clip_rect(visible);
                ui.set_max_width(row_width - margin);
                egui::ScrollArea::horizontal()
                    .id_salt("fluxa-detail-season-scroll")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            for season in &seasons {
                                let label = if *season == 0 {
                                    t("auto.specials")
                                } else {
                                    t("format.season_number").replacen("%s", &season.to_string(), 1)
                                };
                                let response = components::button_auto_width(
                                    ui,
                                    &label,
                                    if *season == selected_season {
                                        components::ButtonKind::Selected
                                    } else {
                                        components::ButtonKind::Secondary
                                    },
                                    metrics.nav_label_size,
                                    metrics,
                                );
                                layout
                                    .focusable
                                    .push((NODE_DETAIL_SEASON_BASE + (*season).clamp(0, 99) as u64, response.rect));
                                if response.clicked() {
                                    context.data_mut(|data| data.insert_temp(season_id, *season));
                                }
                            }
                        });
                    });
            });
        let thumb = Vec2::new(EPISODE_WIDTH, EPISODE_WIDTH * 9.0 / 16.0);
        egui::Area::new(Id::new("fluxa-detail-episodes"))
            .constrain(false)
            .fixed_pos(Pos2::new(margin, top + SECTION_TITLE + SEASON_ROW))
            .show(context, |ui| {
                ui.set_clip_rect(visible);
                ui.set_max_width(row_width);
                egui::ScrollArea::horizontal()
                    .id_salt(("fluxa-detail-episode-scroll", selected_season))
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 16.0;
                            for (index, episode) in &season_episodes {
                                let (rect, response) = ui.allocate_exact_size(
                                    Vec2::new(EPISODE_WIDTH, thumb.y + EPISODE_TEXT),
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
                                let painter = ui.painter();
                                let image = Rect::from_min_size(rect.min, thumb);
                                painter.rect_filled(image, 10.0, Color32::from_white_alpha(14));
                                if !components::rounded_artwork(painter, image, 10.0, episode.thumbnail.as_deref(), artwork_target_size(image.size(), ppp), ArtworkPriority::Visible, Color32::WHITE, assets) {
                                    painter.text(
                                        image.center(),
                                        Align2::CENTER_CENTER,
                                        format!("E{}", episode.number),
                                        FontId::proportional(22.0),
                                        Color32::from_white_alpha(90),
                                    );
                                }
                                if response.hovered() {
                                    painter.rect_filled(image, 10.0, Color32::from_black_alpha(90));
                                    if let Some(icon) = assets.icon("PlayFilled") {
                                        painter.image(
                                            icon,
                                            Rect::from_center_size(image.center(), Vec2::splat(34.0)),
                                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                            Color32::WHITE,
                                        );
                                    }
                                }
                                let title = if episode.title.is_empty() {
                                    format!("{}", episode.number)
                                } else {
                                    format!("{}. {}", episode.number, episode.title)
                                };
                                painter.text(
                                    Pos2::new(rect.left(), image.bottom() + 12.0),
                                    Align2::LEFT_TOP,
                                    truncate_to_width(painter, &title, &FontId::proportional(15.0), EPISODE_WIDTH),
                                    FontId::proportional(15.0),
                                    Color32::WHITE,
                                );
                                let overview = components::wrapped_text(
                                    painter,
                                    &episode.overview,
                                    13.0,
                                    Color32::from_white_alpha(140),
                                    EPISODE_WIDTH,
                                    2,
                                );
                                painter.galley(
                                    Pos2::new(rect.left(), image.bottom() + 34.0),
                                    overview,
                                    Color32::from_white_alpha(140),
                                );
                            }
                        });
                    });
            });
    }

    if let Some(top) = geometry.cast_top {
        let top = top - scroll_y;
        section_title(&painter, Pos2::new(margin, top), &t("auto.cast"), title_size);
        egui::Area::new(Id::new("fluxa-detail-cast"))
            .constrain(false)
            .fixed_pos(Pos2::new(margin, top + SECTION_TITLE))
            .show(context, |ui| {
                ui.set_clip_rect(visible);
                ui.set_max_width(row_width);
                egui::ScrollArea::horizontal()
                    .id_salt("fluxa-detail-cast-scroll")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 18.0;
                            for member in &detail.cast {
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
                                components::avatar(painter, assets, circle.center(), CAST_SIZE * 0.5, &member.name, member.photo.as_deref());
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
                                        Color32::from_white_alpha(130),
                                    );
                                }
                            }
                        });
                    });
            });
    }

    if let Some(top) = geometry.similar_top {
        let top = top - scroll_y;
        section_title(&painter, Pos2::new(margin, top), &t("player.recommendations"), title_size);
        let (poster, slot_height) = similar_size(metrics);
        egui::Area::new(Id::new("fluxa-detail-similar"))
            .constrain(false)
            .fixed_pos(Pos2::new(margin, top + SECTION_TITLE))
            .show(context, |ui| {
                ui.set_clip_rect(visible);
                ui.set_max_width(row_width);
                egui::ScrollArea::horizontal()
                    .id_salt("fluxa-detail-similar-scroll")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 14.0;
                            for (index, card) in detail.similar.iter().enumerate() {
                                let (rect, response) =
                                    ui.allocate_exact_size(Vec2::new(poster.x, slot_height), Sense::click());
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
                                    index,
                                    viewport,
                                    metrics,
                                    assets,
                                    false,
                                );
                            }
                        });
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
