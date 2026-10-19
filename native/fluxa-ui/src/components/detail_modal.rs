use super::cards::paint_elided_text;
use super::*;
use crate::{DetailModel, localized, metrics_for_assets};
use crate::{first_value_string, hero_meta_line};

const PANEL_MAX_WIDTH: f32 = 980.0;
const PANEL_MAX_HEIGHT: f32 = 900.0;
const PANEL_MARGIN: f32 = 24.0;
const BUTTON_HEIGHT: f32 = 48.0;
const PAD: f32 = 28.0;
const TITLE_HEIGHT: f32 = 48.0;
const EPISODE_HEIGHT: f32 = 92.0;
const CAST_SIZE: f32 = 108.0;
const SIMILAR_WIDTH: f32 = 128.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailModalOutcome {
    Close,
    Play,
    Watchlist,
    OpenPage,
    Episode(usize),
    Similar(usize),
    SimilarPlay(usize),
    SimilarWatchlist(usize),
}

fn open_id() -> Id {
    Id::new("fluxa-detail-modal-open")
}

fn outcome_id() -> Id {
    Id::new("fluxa-detail-modal-outcome")
}

pub(crate) fn detail_modal_is_open(context: &egui::Context) -> bool {
    context
        .data(|data| data.get_temp::<bool>(open_id()))
        .unwrap_or(false)
}

pub fn set_detail_modal_open(context: &egui::Context, open: bool) {
    let was = context
        .data(|data| data.get_temp::<bool>(open_id()))
        .unwrap_or(false);
    if was != open {
        crate::motion::reset(context, Id::new("fluxa-detail-modal-appear"));
    }
    context.data_mut(|data| data.insert_temp(open_id(), open));
}

pub fn take_detail_modal_outcome(context: &egui::Context) -> Option<DetailModalOutcome> {
    context.data_mut(|data| {
        let outcome = data.get_temp::<DetailModalOutcome>(outcome_id());
        data.remove::<DetailModalOutcome>(outcome_id());
        outcome
    })
}

fn section_title(ui: &mut egui::Ui, text: &str, width: f32, metrics: UiMetrics) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, TITLE_HEIGHT), Sense::hover());
    ui.painter().text(
        Pos2::new(rect.left() + PAD, rect.center().y + 6.0),
        Align2::LEFT_CENTER,
        text,
        crate::fonts::bold(metrics.screen_card_title_size + 4.0),
        metrics.text_primary,
    );
}

fn horizontal_row(ui: &mut egui::Ui, id: impl std::hash::Hash, add: impl FnOnce(&mut egui::Ui)) {
    egui::ScrollArea::horizontal()
        .id_salt(id)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(14.0, 0.0);
                ui.add_space(PAD);
                add(ui);
                ui.add_space(PAD);
            });
        });
}

pub fn draw_detail_modal(
    context: &egui::Context,
    viewport: Viewport,
    detail: &DetailModel,
    assets: &mut impl HomeAssets,
) {
    if !context
        .data(|data| data.get_temp::<bool>(open_id()))
        .unwrap_or(false)
    {
        return;
    }
    let metrics = metrics_for_assets(viewport, assets);
    let language = detail.language.as_str();
    let t = |key: &str| localized(key, language);
    let appear = crate::motion::appear(
        context,
        Id::new("fluxa-detail-modal-appear"),
        crate::motion::normal(),
        crate::motion::Ease::Out,
    );
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let width = (viewport.width - PANEL_MARGIN * 2.0).min(PANEL_MAX_WIDTH);
    let height = (viewport.height - PANEL_MARGIN * 2.0).min(PANEL_MAX_HEIGHT);
    let image_height = (width * 9.0 / 16.0).min(height * 0.55);
    let body_size = (metrics.screen_card_subtitle_size + 3.0).max(15.0);
    let text_width = (width - PAD * 3.0) * 0.62;
    let mut job = egui::text::LayoutJob::single_section(
        detail.description.clone(),
        egui::TextFormat::simple(crate::fonts::regular(body_size), metrics.ink(tone::subtle())),
    );
    job.wrap.max_width = text_width;
    job.wrap.max_rows = 8;
    let synopsis = context.fonts_mut(|fonts| fonts.layout_job(job));
    let facts = detail.show_info.iter().take(6).collect::<Vec<_>>();
    let meta_height = body_size + 10.0;
    let side_height = facts.len() as f32 * (body_size + 10.0);
    let text_block = (meta_height + synopsis.size().y + 8.0).max(side_height);

    let scale = 0.94 + 0.06 * appear;
    let panel = Rect::from_center_size(screen.center(), Vec2::new(width, height));
    let layer = egui::LayerId::new(egui::Order::Debug, Id::new("fluxa-detail-modal"));
    context.set_transform_layer(
        layer,
        egui::emath::TSTransform::new(panel.center().to_vec2() * (1.0 - scale), scale),
    );
    let seasons = detail.seasons();
    let season_id = Id::new(("fluxa-detail-modal-season", &detail.id));
    let season = context
        .data(|data| data.get_temp::<i64>(season_id))
        .filter(|season| seasons.contains(season))
        .unwrap_or_else(|| detail.current_season());
    let mut chosen_season = None;
    let mut outcome = None;
    let preview_id = Id::new("fluxa-modal-preview-state");
    let preview_state = context.data(|data| data.get_temp::<(usize, f64, Rect, Rect)>(preview_id));
    let pointer = context.input(|input| input.pointer.hover_pos());
    let now = context.input(|input| input.time);
    let mut hovered_card: Option<(usize, Rect)> = None;
    egui::Area::new(Id::new("fluxa-detail-modal"))
        .fixed_pos(Pos2::ZERO)
        .order(egui::Order::Debug)
        .show(context, |ui| {
            ui.set_opacity(appear);
            let painter = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            let backdrop = ui.interact(screen, Id::new("fluxa-detail-modal-scrim"), Sense::click());
            painter.rect_filled(screen, 0.0, Color32::from_black_alpha(170));
            let inside = ui.interact(panel, Id::new("fluxa-detail-modal-panel"), Sense::click());
            if backdrop.clicked() && !inside.hovered() {
                outcome = Some(DetailModalOutcome::Close);
            }
            let radius = metrics.dialog_radius;
            painter.rect_filled(panel, radius, metrics.surface);

            ui.scope_builder(egui::UiBuilder::new().max_rect(panel), |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(("fluxa-detail-modal-scroll", &detail.id))
                    .auto_shrink([false, false])
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        ui.set_width(width);
                        let (image, _) =
                            ui.allocate_exact_size(Vec2::new(width, image_height), Sense::hover());
                        let painter = ui.painter().clone();
                        rounded_artwork(
                            &painter,
                            Rect::from_min_size(image.min, Vec2::new(width, image_height + radius)),
                            radius,
                            detail
                                .background_url
                                .as_deref()
                                .or(detail.poster_url.as_deref()),
                            artwork_target_size(image.size(), context.pixels_per_point()),
                            ArtworkPriority::Visible,
                            Color32::WHITE,
                            assets,
                        );
                        if let Some(texture) = detail.trailer {
                            rounded_image(
                                &painter,
                                texture,
                                Rect::from_min_size(
                                    image.min,
                                    Vec2::new(width, image_height + radius),
                                ),
                                radius,
                                cover_uv([1920, 1080], image),
                                Color32::WHITE,
                            );
                        }
                        paint_vertical_gradient(
                            &painter,
                            Rect::from_min_max(
                                Pos2::new(image.left(), image.top() + image_height * 0.45),
                                image.right_bottom(),
                            ),
                            Color32::TRANSPARENT,
                            metrics.surface,
                        );
                        painter.rect_filled(
                            Rect::from_min_size(
                                image.left_bottom(),
                                Vec2::new(width, radius + 1.0),
                            ),
                            0.0,
                            metrics.surface,
                        );
                        let logo = title_logo(
                            context.pixels_per_point(),
                            detail.logo_url.as_deref(),
                            Vec2::new(width * 0.4, image_height * 0.3),
                            ArtworkPriority::Visible,
                            assets,
                        );
                        match logo {
                            Some((texture, size)) => texture_image(
                                &painter,
                                texture,
                                Rect::from_min_size(
                                    Pos2::new(image.left() + PAD, image.bottom() - size.y - 8.0),
                                    size,
                                ),
                                full_uv(),
                                Color32::WHITE,
                            ),
                            None => {
                                let size = metrics.screen_card_title_size + 10.0;
                                paint_elided_text(
                                    &painter,
                                    Pos2::new(image.left() + PAD, image.bottom() - size - 12.0),
                                    &detail.title,
                                    crate::fonts::medium(size),
                                    width - PAD * 2.0,
                                    metrics.text_primary,
                                );
                            }
                        }

                        let (row, _) = ui.allocate_exact_size(
                            Vec2::new(width, BUTTON_HEIGHT + PAD * 1.8),
                            Sense::hover(),
                        );
                        let row_y = row.top() + PAD * 0.5;
                        let round = BUTTON_HEIGHT;
                        let gap = 10.0;
                        let more = Rect::from_min_size(
                            Pos2::new(row.right() - PAD - round, row_y),
                            Vec2::splat(round),
                        );
                        let watchlist = Rect::from_min_size(
                            Pos2::new(more.left() - gap - round, row_y),
                            Vec2::splat(round),
                        );
                        let play_rect = Rect::from_min_max(
                            Pos2::new(row.left() + PAD, row_y),
                            Pos2::new(watchlist.left() - gap, row_y + BUTTON_HEIGHT),
                        );
                        let label = crate::play_label(language, detail.resume.as_ref(), None);
                        let play = ui
                            .scope_builder(egui::UiBuilder::new().max_rect(play_rect), |ui| {
                                play_button(
                                    ui,
                                    assets,
                                    &label,
                                    Some(play_rect.width().min(320.0)),
                                    BUTTON_HEIGHT,
                                    body_size + 1.0,
                                    detail
                                        .resume
                                        .as_ref()
                                        .map(|card| card.progress)
                                        .filter(|progress| *progress > 0.0),
                                )
                            })
                            .inner;
                        if play.clicked() {
                            outcome = Some(DetailModalOutcome::Play);
                        }
                        for (rect, id, icon, result) in [
                            (
                                watchlist,
                                "fluxa-detail-modal-watchlist",
                                if detail.in_watchlist { "Check" } else { "Plus" },
                                DetailModalOutcome::Watchlist,
                            ),
                            (
                                more,
                                "fluxa-detail-modal-more",
                                "Info",
                                DetailModalOutcome::OpenPage,
                            ),
                        ] {
                            let response = ui.interact(rect, Id::new(id), Sense::click());
                            painter.rect_filled(
                                rect,
                                round * 0.5,
                                metrics.ink(if response.hovered() { 46 } else { 28 }),
                            );
                            if let Some(icon) = assets.icon(icon) {
                                painter.image(
                                    icon,
                                    Rect::from_center_size(rect.center(), Vec2::splat(20.0)),
                                    full_uv(),
                                    Color32::WHITE,
                                );
                            }
                            if response.clicked() {
                                outcome = Some(result);
                            }
                        }

                        ui.add_space(PAD);
                        let (block, _) = ui.allocate_exact_size(
                            Vec2::new(width, text_block + PAD * 0.5),
                            Sense::hover(),
                        );
                        let left = block.left() + PAD;
                        if !detail.meta_line.is_empty() {
                            paint_elided_text(
                                &painter,
                                Pos2::new(left, block.top()),
                                &detail.meta_line,
                                crate::fonts::regular(body_size),
                                text_width,
                                metrics.ink(tone::body()),
                            );
                        }
                        painter.galley(
                            Pos2::new(left, block.top() + meta_height + 2.0),
                            synopsis.clone(),
                            metrics.text_primary,
                        );
                        let side_left = left + text_width + PAD;
                        let side_width = block.right() - PAD - side_left;
                        let label_width = side_width * 0.38;
                        let mut side_y = block.top();
                        for (name, value) in &facts {
                            paint_elided_text(
                                &painter,
                                Pos2::new(side_left, side_y),
                                name,
                                crate::fonts::regular(body_size),
                                label_width,
                                metrics.ink(tone::subtle()),
                            );
                            paint_elided_text(
                                &painter,
                                Pos2::new(side_left + label_width + 8.0, side_y),
                                value,
                                crate::fonts::regular(body_size),
                                side_width - label_width - 8.0,
                                metrics.text_primary,
                            );
                            side_y += body_size + 10.0;
                        }

                        if detail.is_series() && !detail.episodes.is_empty() {
                            section_title(ui, &t("auto.episodes"), width, metrics);
                            if seasons.len() > 1 {
                                let (chips, _) =
                                    ui.allocate_exact_size(Vec2::new(width, 46.0), Sense::hover());
                                let mut x = chips.left() + PAD;
                                for number in &seasons {
                                    let text = t("format.season_number").replacen(
                                        "%s",
                                        &number.to_string(),
                                        1,
                                    );
                                    let font = crate::fonts::medium(body_size);
                                    let size = Vec2::new(
                                        crate::paint::text_width(&painter, &text, font.clone())
                                            + 28.0,
                                        36.0,
                                    );
                                    let rect = Rect::from_min_size(Pos2::new(x, chips.top()), size);
                                    let response = ui.interact(
                                        rect,
                                        Id::new(("fluxa-detail-modal-season-chip", number)),
                                        Sense::click(),
                                    );
                                    let active = *number == season;
                                    painter.rect_filled(
                                        rect,
                                        size.y * 0.5,
                                        if active {
                                            Color32::WHITE
                                        } else {
                                            metrics.ink(if response.hovered() { 46 } else { 28 })
                                        },
                                    );
                                    painter.text(
                                        rect.center(),
                                        Align2::CENTER_CENTER,
                                        &text,
                                        font,
                                        if active {
                                            Color32::BLACK
                                        } else {
                                            metrics.text_primary
                                        },
                                    );
                                    if response.clicked() {
                                        chosen_season = Some(*number);
                                    }
                                    x += size.x + 8.0;
                                }
                            }
                            for (index, episode) in detail
                                .episodes
                                .iter()
                                .enumerate()
                                .filter(|(_, episode)| episode.season == season)
                            {
                                let (rect, response) = ui.allocate_exact_size(
                                    Vec2::new(width, EPISODE_HEIGHT),
                                    Sense::click(),
                                );
                                if response.hovered() {
                                    painter.rect_filled(rect, 0.0, metrics.ink(14));
                                }
                                let hidden = (detail.hide_spoilers || detail.blur_unwatched)
                                    && !episode.watched;
                                let thumb = Rect::from_min_size(
                                    Pos2::new(rect.left() + PAD, rect.top() + 6.0),
                                    Vec2::new(136.0, 136.0 * 9.0 / 16.0),
                                );
                                painter.rect_filled(thumb, 8.0, metrics.ink(20));
                                if !hidden {
                                    rounded_artwork(
                                        &painter,
                                        thumb,
                                        8.0,
                                        episode.thumbnail.as_deref(),
                                        artwork_target_size(
                                            thumb.size(),
                                            context.pixels_per_point(),
                                        ),
                                        ArtworkPriority::Visible,
                                        Color32::WHITE,
                                        assets,
                                    );
                                }
                                if episode.progress > 0.0 {
                                    let bar = Rect::from_min_size(
                                        Pos2::new(thumb.left(), thumb.bottom() - 4.0),
                                        Vec2::new(thumb.width() * episode.progress.min(1.0), 4.0),
                                    );
                                    painter.rect_filled(bar, 0.0, metrics.accent);
                                }
                                let text_left = thumb.right() + 16.0;
                                let text_room = rect.right() - PAD - text_left - 28.0;
                                paint_elided_text(
                                    &painter,
                                    Pos2::new(text_left, rect.top() + 8.0),
                                    &format!("{}. {}", episode.number, episode.title),
                                    crate::fonts::medium(body_size + 1.0),
                                    text_room,
                                    metrics.text_primary,
                                );
                                if detail.show_episode_descriptions
                                    && !hidden
                                    && !episode.overview.is_empty()
                                {
                                    let mut job = egui::text::LayoutJob::single_section(
                                        episode.overview.clone(),
                                        egui::TextFormat::simple(
                                            crate::fonts::regular(body_size - 1.0),
                                            metrics.ink(tone::subtle()),
                                        ),
                                    );
                                    job.wrap.max_width = text_room;
                                    job.wrap.max_rows = 2;
                                    let galley = context.fonts_mut(|fonts| fonts.layout_job(job));
                                    painter.galley(
                                        Pos2::new(text_left, rect.top() + 12.0 + body_size + 4.0),
                                        galley,
                                        metrics.text_primary,
                                    );
                                }
                                if episode.watched
                                    && let Some(icon) = assets.icon("Check")
                                {
                                    painter.image(
                                        icon,
                                        Rect::from_center_size(
                                            Pos2::new(rect.right() - PAD - 10.0, rect.center().y),
                                            Vec2::splat(18.0),
                                        ),
                                        full_uv(),
                                        metrics.ink(tone::subtle()),
                                    );
                                }
                                if response.clicked() {
                                    outcome = Some(DetailModalOutcome::Episode(index));
                                }
                            }
                        }

                        let people = detail.cast.iter().chain(detail.crew.iter()).take(24);
                        if people.clone().next().is_some() {
                            section_title(ui, &t("detail.cast_crew"), width, metrics);
                            horizontal_row(ui, ("fluxa-detail-modal-cast", &detail.id), |ui| {
                                for person in people {
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::new(CAST_SIZE + 24.0, CAST_SIZE + 48.0),
                                        Sense::hover(),
                                    );
                                    let photo = Rect::from_min_size(
                                        Pos2::new(rect.center().x - CAST_SIZE * 0.5, rect.top()),
                                        Vec2::splat(CAST_SIZE),
                                    );
                                    let painter = ui.painter();
                                    painter.circle_filled(
                                        photo.center(),
                                        CAST_SIZE * 0.5,
                                        metrics.ink(20),
                                    );
                                    rounded_artwork(
                                        painter,
                                        photo,
                                        CAST_SIZE * 0.5,
                                        person.photo.as_deref(),
                                        artwork_target_size(
                                            photo.size(),
                                            context.pixels_per_point(),
                                        ),
                                        ArtworkPriority::Visible,
                                        Color32::WHITE,
                                        assets,
                                    );
                                    paint_elided_text(
                                        painter,
                                        Pos2::new(rect.left(), photo.bottom() + 8.0),
                                        &person.name,
                                        crate::fonts::regular(body_size - 2.0),
                                        rect.width(),
                                        metrics.text_primary,
                                    );
                                    if let Some(role) = person.role.as_deref() {
                                        paint_elided_text(
                                            painter,
                                            Pos2::new(
                                                rect.left(),
                                                photo.bottom() + 10.0 + body_size,
                                            ),
                                            role,
                                            crate::fonts::regular(body_size - 3.0),
                                            rect.width(),
                                            metrics.ink(tone::subtle()),
                                        );
                                    }
                                }
                            });
                            ui.add_space(12.0);
                        }

                        if !detail.similar.is_empty() {
                            section_title(ui, &t("player.recommendations"), width, metrics);
                            horizontal_row(ui, ("fluxa-detail-modal-similar", &detail.id), |ui| {
                                for (index, card) in detail.similar.iter().take(30).enumerate() {
                                    let (rect, response) = ui.allocate_exact_size(
                                        Vec2::new(SIMILAR_WIDTH, SIMILAR_WIDTH * 1.5),
                                        Sense::click(),
                                    );
                                    poster_card(ui.painter(), rect, card, metrics, assets, false);
                                    if let Some(position) = pointer
                                        && ui.clip_rect().intersect(rect).contains(position)
                                    {
                                        hovered_card = Some((index, rect));
                                    }
                                    if response.clicked() {
                                        outcome = Some(DetailModalOutcome::Similar(index));
                                    }
                                }
                            });
                        }
                        ui.add_space(PAD);
                    });
            });

            let close = Rect::from_center_size(
                Pos2::new(panel.right() - 28.0, panel.top() + 28.0),
                Vec2::splat(36.0),
            );
            let close_response =
                ui.interact(close, Id::new("fluxa-detail-modal-close"), Sense::click());
            painter.circle_filled(
                close.center(),
                18.0,
                Color32::from_black_alpha(if close_response.hovered() { 190 } else { 130 }),
            );
            if let Some(icon) = assets.icon("Close") {
                painter.image(
                    icon,
                    Rect::from_center_size(close.center(), Vec2::splat(18.0)),
                    full_uv(),
                    Color32::WHITE,
                );
            }
            if close_response.clicked() {
                outcome = Some(DetailModalOutcome::Close);
            }
            painter.rect_stroke(
                panel,
                radius,
                egui::Stroke::new(stroke::hairline(), metrics.border),
                egui::StrokeKind::Inside,
            );
        });
    let hovered = hovered_card.or_else(|| {
        preview_state
            .filter(|state| pointer.is_some_and(|position| state.3.contains(position)))
            .map(|state| (state.0, state.2))
    });
    let previewed_id = Id::new("fluxa-previewed-poster");
    let preview_card = hovered.and_then(|(index, anchor)| {
        detail
            .similar
            .get(index)
            .filter(|card| card.id.is_some())
            .map(|card| (index, anchor, card))
    });
    match preview_card {
        None => {
            hover_preview_reset(context, egui::Order::Debug);
            context.data_mut(|data| {
                data.remove::<(usize, f64, Rect, Rect)>(preview_id);
                data.remove::<crate::ExpandedPoster>(previewed_id);
            });
        }
        Some((index, anchor, card)) => {
            let since = preview_state
                .filter(|state| state.0 == index)
                .map_or(now, |state| state.1);
            let synopsis =
                first_value_string(&card.raw, &["description", "overview"]).unwrap_or_default();
            let (area, galley) = hover_preview_rect(context, anchor, screen, &synopsis, metrics);
            context.data_mut(|data| data.insert_temp(preview_id, (index, since, anchor, area)));
            let remaining = 0.5 - (now - since);
            if remaining > 0.0 {
                hover_preview_reset(context, egui::Order::Debug);
                context.data_mut(|data| data.remove::<crate::ExpandedPoster>(previewed_id));
                context.request_repaint_after(std::time::Duration::from_secs_f64(remaining));
            } else {
                if let Some(id) = card.id.clone() {
                    context.data_mut(|data| {
                        data.insert_temp(
                            previewed_id,
                            crate::ExpandedPoster {
                                node: index as u64,
                                id,
                                raw: card.raw.clone(),
                            },
                        )
                    });
                }
                let trailer = detail
                    .preview_trailer
                    .as_ref()
                    .filter(|trailer| card.id.as_deref() == Some(trailer.item_id.as_str()))
                    .and_then(|trailer| trailer.texture);
                let meta = hero_meta_line(&card.raw, language);
                let series = matches!(card.item_type.as_deref(), Some("series" | "tv" | "show"));
                let label = crate::play_label(language, None, series.then_some((1, 1, None)));
                let (response, action) = hover_preview(
                    context,
                    area,
                    card,
                    trailer,
                    &meta,
                    galley,
                    &label,
                    None,
                    egui::Order::Debug,
                    metrics,
                    assets,
                );
                outcome = outcome.or(match action {
                    Some(PreviewAction::Play) => Some(DetailModalOutcome::SimilarPlay(index)),
                    Some(PreviewAction::Watchlist) => {
                        Some(DetailModalOutcome::SimilarWatchlist(index))
                    }
                    None if response.clicked() => Some(DetailModalOutcome::Similar(index)),
                    None => None,
                });
            }
        }
    }
    if let Some(number) = chosen_season {
        context.data_mut(|data| data.insert_temp(season_id, number));
    }
    if let Some(outcome) = outcome {
        context.data_mut(|data| data.insert_temp(outcome_id(), outcome));
    }
}
