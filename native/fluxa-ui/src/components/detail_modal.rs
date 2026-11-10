use super::cards::{paint_elided_text, pinned};
use super::*;
use crate::dp::Dp;
use crate::flex::Space;
use crate::kit;
use crate::{DetailModel, localized, metrics_for_assets};
use crate::{first_value_string, hero_meta_line};

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

pub(crate) fn detail_modal_is_open(context: &crate::gfx::Context) -> bool {
    context
        .data(|data| data.get_temp::<bool>(open_id()))
        .unwrap_or(false)
}

fn covered_id() -> Id {
    Id::new("fluxa-screen-covered")
}

pub fn set_screen_covered(context: &crate::gfx::Context, covered: bool) {
    context.data_mut(|data| data.insert_temp(covered_id(), covered));
}

const SETTLED_ID: &str = "fluxa-ui-settled";

pub fn set_ui_settled(context: &crate::gfx::Context, settled: bool) {
    context.data_mut(|data| data.insert_temp(crate::gfx::Id::new(SETTLED_ID), settled));
}

pub(crate) fn ui_settled(context: &crate::gfx::Context) -> bool {
    context
        .data(|data| data.get_temp::<bool>(crate::gfx::Id::new(SETTLED_ID)))
        .unwrap_or(true)
}

pub(crate) fn screen_covered(context: &crate::gfx::Context) -> bool {
    detail_modal_is_open(context)
        || crate::gfx::Popup::is_any_open(context)
        || context
            .data(|data| data.get_temp::<bool>(covered_id()))
            .unwrap_or(false)
}

pub fn set_detail_modal_open(context: &crate::gfx::Context, open: bool) {
    let was = context
        .data(|data| data.get_temp::<bool>(open_id()))
        .unwrap_or(false);
    if was != open {
        crate::motion::reset(context, Id::new("fluxa-detail-modal-appear"));
    }
    context.data_mut(|data| data.insert_temp(open_id(), open));
}

pub fn take_detail_modal_outcome(context: &crate::gfx::Context) -> Option<DetailModalOutcome> {
    context.data_mut(|data| {
        let outcome = data.get_temp::<DetailModalOutcome>(outcome_id());
        data.remove::<DetailModalOutcome>(outcome_id());
        outcome
    })
}

fn section_title(ui: &mut crate::gfx::Ui, text: &str, width: f32, metrics: UiMetrics) {
    let rect = kit::view()
        .column()
        .max_width(width)
        .show(ui, |ui| {
            ui.set_min_height(metrics.detail_modal_title_height)
        })
        .response
        .rect;
    let font = crate::fonts::bold(metrics.screen_card_title_size + Space::S4.px(&metrics));
    let label = crate::flex::row([crate::flex::text(ui.painter(), text, &font)])
        .align(crate::flex::Align::Center)
        .padding(crate::flex::Insets {
            left: metrics.detail_modal_pad,
            top: Space::S12.px(&metrics),
            right: 0.0,
            bottom: 0.0,
        });
    let placed = label.solve(rect);
    if let Some(slot) = placed.children.first() {
        ui.painter().text(
            slot.rect.left_center(),
            Align2::LEFT_CENTER,
            text,
            font,
            metrics.text_primary,
        );
    }
}

fn horizontal_row(
    ui: &mut crate::gfx::Ui,
    metrics: UiMetrics,
    id: impl std::hash::Hash + std::fmt::Debug,
    add: impl FnOnce(&mut crate::gfx::Ui),
) {
    super::pan_area(
        ui,
        id,
        |area| area,
        |ui| {
            kit::view().row().gap(Space::S14).show(ui, |ui| {
                ui.add_space(metrics.detail_modal_pad);
                add(ui);
                ui.add_space(metrics.detail_modal_pad);
            });
        },
    );
}

fn first_episode(detail: &DetailModel) -> Option<(i64, i64, Option<&str>)> {
    let season = detail.current_season();
    detail
        .episodes
        .iter()
        .filter(|episode| episode.season == season)
        .min_by_key(|episode| episode.number)
        .map(|episode| (episode.season, episode.number, Some(episode.title.as_str())))
}

pub fn draw_detail_modal(
    context: &crate::gfx::Context,
    viewport: Viewport,
    detail: &DetailModel,
    assets: &mut impl HomeAssets,
) {
    let open = detail_modal_is_open(context);
    let visible_id = Id::new("fluxa-detail-modal-visible");
    let exit_id = Id::new("fluxa-detail-modal-exit");
    let visible = context
        .data(|data| data.get_temp::<bool>(visible_id))
        .unwrap_or(false);
    if !open && !visible {
        return;
    }
    let metrics = metrics_for_assets(viewport, assets);
    let language = detail.language.as_str();
    let t = |key: &str| localized(key, language);
    let appear = if open {
        crate::motion::reset(context, exit_id);
        context.data_mut(|data| data.insert_temp(visible_id, true));
        crate::motion::appear(
            context,
            Id::new("fluxa-detail-modal-appear"),
            crate::motion::normal(),
            crate::motion::Ease::Out,
        )
    } else {
        let eased = crate::motion::disappear(
            context,
            exit_id,
            crate::motion::normal(),
            crate::motion::Ease::Token,
        );
        if eased <= 0.001 {
            crate::motion::reset(context, exit_id);
            context.data_mut(|data| data.insert_temp(visible_id, false));
            return;
        }
        eased
    };
    let screen = crate::gfx::rect_at(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let width = (viewport.width - metrics.detail_modal_panel_margin * 2.0)
        .min(metrics.detail_modal_panel_max_width);
    let height = (viewport.height - metrics.detail_modal_panel_margin * 2.0)
        .min(metrics.detail_modal_panel_max_height);
    let image_height = (width * 9.0 / 16.0).min(height * metrics.surface_detail_image_max_share);
    let body_size = (metrics.screen_card_subtitle_size + 3.0).max(15.0);
    let synopsis_size = body_size + 2.0;
    let text_width = (width - metrics.detail_modal_pad * 3.0) * metrics.surface_detail_text_share;
    let mut job = crate::gfx::text::LayoutJob::single_section(
        detail.description.clone(),
        crate::gfx::TextFormat::simple(
            crate::fonts::regular(synopsis_size),
            metrics.ink(tone::subtle()),
        ),
    );
    job.wrap.max_width = text_width;
    job.wrap.max_rows = 8;
    let synopsis = context.fonts_mut(|fonts| fonts.layout_job(job));
    let facts = detail.show_info.iter().take(6).collect::<Vec<_>>();
    let fact_height = body_size + Space::S10.px(&metrics);
    let meta_height = fact_height;
    let side_height = facts.len() as f32 * fact_height;
    let text_block = (meta_height + synopsis.size().y + Space::S8.px(&metrics)).max(side_height);

    let scale = 0.94 + 0.06 * appear;
    let panel = crate::flex::centered(screen, Vec2::new(width, height));
    let layer = crate::gfx::LayerId::new(crate::gfx::Order::Debug, Id::new("fluxa-detail-modal"));
    context.set_transform_layer(
        layer,
        crate::gfx::emath::TSTransform::new(panel.center().to_vec2() * (1.0 - scale), scale),
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
    let pointer = context.input(|input| input.pointer().hover_pos());
    let now = context.input(|input| input.time());
    let mut hovered_card: Option<(usize, Rect)> = None;
    crate::gfx::Area::new(Id::new("fluxa-detail-modal"))
        .fixed_pos(Pos2::ZERO)
        .order(crate::gfx::Order::Debug)
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

            ui.scope_builder(crate::gfx::UiBuilder::new().max_rect(panel), |ui| {
                super::pan_area(
                    ui,
                    ("fluxa-detail-modal-scroll", &detail.id),
                    |area| area.auto_shrink([false, false]),
                    |ui| {
                        ui.set_item_spacing(Vec2::ZERO);
                        ui.set_width(width);
                        let (image, _) =
                            ui.allocate_exact_size(Vec2::new(width, image_height), Sense::hover());
                        let painter = ui.painter().clone();
                        rounded_artwork(
                            &painter,
                            crate::gfx::rect_at(image.min, Vec2::new(width, image_height + radius)),
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
                                crate::gfx::rect_at(
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
                            crate::flex::below(image, Vec2::new(width, radius + 1.0), 0.0),
                            0.0,
                            metrics.surface,
                        );
                        let detail_logo = Vec2::new(
                            width * metrics.surface_detail_logo_width_share,
                            image_height * metrics.surface_detail_logo_height_share,
                        );
                        let logo_inset = crate::flex::Insets {
                            left: metrics.detail_modal_pad,
                            top: 0.0,
                            right: 0.0,
                            bottom: Space::S8.px(&metrics),
                        };
                        let logo = title_logo(
                            context.pixels_per_point(),
                            detail.logo_url.as_deref(),
                            detail_logo,
                            ArtworkPriority::Visible,
                            assets,
                        );
                        match logo {
                            Some((texture, size)) => texture_image(
                                &painter,
                                texture,
                                pinned(image, Align2::LEFT_BOTTOM, size, logo_inset),
                                full_uv(),
                                Color32::WHITE,
                            ),
                            None if failed_dom_image(
                                &painter,
                                assets,
                                pinned(image, Align2::LEFT_BOTTOM, detail_logo, logo_inset),
                                0.0,
                                detail.logo_url.as_deref(),
                                artwork_target_size(detail_logo, context.pixels_per_point()),
                                ImageFit::Contain,
                                [0.0, 1.0],
                            ) => {}
                            None => {
                                let size = metrics.screen_card_title_size + Space::S10.px(&metrics);
                                paint_elided_text(
                                    &painter,
                                    pinned(
                                        image,
                                        Align2::LEFT_BOTTOM,
                                        Vec2::splat(size),
                                        crate::flex::Insets {
                                            bottom: Space::S12.px(&metrics),
                                            ..logo_inset
                                        },
                                    )
                                    .min,
                                    &detail.title,
                                    crate::fonts::medium(size),
                                    width - metrics.detail_modal_pad * 2.0,
                                    metrics.text_primary,
                                );
                            }
                        }

                        let row = kit::view()
                            .row()
                            .max_width(width)
                            .show(ui, |ui| {
                                ui.set_min_height(
                                    metrics.detail_modal_button_height
                                        + Space::S14.px(&metrics) * 2.0,
                                )
                            })
                            .response
                            .rect;
                        let round = metrics.detail_modal_button_height;
                        let (play_id, watchlist_id, more_id) = (1, 2, 3);
                        let actions = crate::flex::row([
                            crate::flex::fill().height(round).id(play_id),
                            crate::flex::leaf(Vec2::splat(round)).id(watchlist_id),
                            crate::flex::leaf(Vec2::splat(round)).id(more_id),
                        ])
                        .gap(Space::S10.px(&metrics))
                        .padding(crate::flex::Insets::xy(
                            metrics.detail_modal_pad,
                            Space::S14.px(&metrics),
                        ))
                        .align(crate::flex::Align::Start);
                        let placed = actions.solve(row);
                        let rect_of = |id| placed.find(id).unwrap_or(row);
                        let (play_rect, watchlist, more) =
                            (rect_of(play_id), rect_of(watchlist_id), rect_of(more_id));
                        let label = crate::play_label(
                            language,
                            detail.resume.as_ref(),
                            first_episode(detail).filter(|_| detail.is_series()),
                        );
                        let play = ui
                            .scope_builder(crate::gfx::UiBuilder::new().max_rect(play_rect), |ui| {
                                play_button(
                                    ui,
                                    assets,
                                    &label,
                                    Some(
                                        play_rect
                                            .width()
                                            .min(metrics.surface_detail_play_max_width),
                                    ),
                                    metrics.detail_modal_button_height,
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
                                    crate::flex::centered(rect, Vec2::splat(Dp::D20.px())),
                                    full_uv(),
                                    Color32::WHITE,
                                );
                            }
                            if response.clicked() {
                                outcome = Some(result);
                            }
                        }

                        ui.add_space(metrics.detail_modal_pad);
                        let block = kit::view()
                            .column()
                            .max_width(width)
                            .show(ui, |ui| {
                                ui.set_min_height(text_block + Space::S14.px(&metrics))
                            })
                            .response
                            .rect;
                        let (copy_id, meta_id, synopsis_id, side_id) = (1, 2, 3, 4);
                        let columns = crate::flex::row([
                            crate::flex::column([
                                crate::flex::leaf(Vec2::new(text_width, meta_height)).id(meta_id),
                                crate::flex::leaf(synopsis.size()).id(synopsis_id),
                            ])
                            .gap(Space::S2.px(&metrics))
                            .width(text_width)
                            .id(copy_id),
                            crate::flex::fill().id(side_id),
                        ])
                        .gap(metrics.detail_modal_pad)
                        .padding(crate::flex::Insets::xy(metrics.detail_modal_pad, 0.0))
                        .align(crate::flex::Align::Start);
                        let placed = columns.solve(block);
                        if let Some(meta) = placed.find(meta_id)
                            && !detail.meta_line.is_empty()
                        {
                            paint_elided_text(
                                &painter,
                                meta.min,
                                &detail.meta_line,
                                crate::fonts::regular(body_size),
                                text_width,
                                metrics.ink(tone::body()),
                            );
                        }
                        if let Some(copy) = placed.find(synopsis_id) {
                            painter.galley(copy.min, synopsis.clone(), metrics.text_primary);
                        }
                        let side = placed.find(side_id).unwrap_or(block);
                        let label_width = side.width() * metrics.surface_detail_fact_label_share;
                        let fact_rows =
                            crate::flex::column(facts.iter().enumerate().map(|(index, _)| {
                                crate::flex::row([
                                    crate::flex::slot(label_width)
                                        .height(fact_height)
                                        .id(index * 2),
                                    crate::flex::fill().height(fact_height).id(index * 2 + 1),
                                ])
                                .gap(Space::S8.px(&metrics))
                            }))
                            .align(crate::flex::Align::Start);
                        let placed = fact_rows.solve(side);
                        for (index, (name, value)) in facts.iter().enumerate() {
                            let (Some(name_slot), Some(value_slot)) =
                                (placed.find(index * 2), placed.find(index * 2 + 1))
                            else {
                                continue;
                            };
                            paint_elided_text(
                                &painter,
                                name_slot.min,
                                name,
                                crate::fonts::regular(body_size),
                                name_slot.width(),
                                metrics.ink(tone::subtle()),
                            );
                            paint_elided_text(
                                &painter,
                                value_slot.min,
                                value,
                                crate::fonts::regular(body_size),
                                value_slot.width(),
                                metrics.text_primary,
                            );
                        }

                        if detail.is_series() && !detail.episodes.is_empty() {
                            section_title(ui, &t("auto.episodes"), width, metrics);
                            if seasons.len() > 1 {
                                kit::view().column().max_width(width).show(ui, |ui| {
                                    horizontal_row(
                                        ui,
                                        metrics,
                                        ("fluxa-detail-modal-season-chips", &detail.id),
                                        |ui| {
                                            ui.set_item_spacing_x(Space::S8.px(&metrics));
                                            let painter = ui.painter().clone();
                                            for number in &seasons {
                                                let text = t("format.season_number").replacen(
                                                    "%s",
                                                    &number.to_string(),
                                                    1,
                                                );
                                                let font = crate::fonts::medium(body_size);
                                                let chip = crate::flex::row([crate::flex::text(
                                                    &painter, &text, &font,
                                                )])
                                                .align(crate::flex::Align::Center)
                                                .padding(crate::flex::Insets::xy(
                                                    Space::S14.px(&metrics),
                                                    0.0,
                                                ))
                                                .height(Dp::D36.px());
                                                let size = chip.outer_size();
                                                let (rect, response) =
                                                    ui.allocate_exact_size(size, Sense::click());
                                                let label = chip
                                                    .solve(rect)
                                                    .children
                                                    .first()
                                                    .map_or(rect, |slot| slot.rect);
                                                let active = *number == season;
                                                painter.rect_filled(
                                                    rect,
                                                    size.y * 0.5,
                                                    if active {
                                                        Color32::WHITE
                                                    } else {
                                                        metrics.ink(if response.hovered() {
                                                            46
                                                        } else {
                                                            28
                                                        })
                                                    },
                                                );
                                                painter.text(
                                                    label.center(),
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
                                            }
                                        },
                                    );
                                });
                            }
                            for (index, episode) in detail
                                .episodes
                                .iter()
                                .enumerate()
                                .filter(|(_, episode)| episode.season == season)
                            {
                                let response = crate::detail::tile(
                                    ui,
                                    ("fluxa-detail-modal-episode", &detail.id, index),
                                    Vec2::new(width, metrics.detail_modal_episode_height),
                                    Sense::click(),
                                    |ui, rect| {
                                        if ui.rect_contains_pointer(rect) {
                                            painter.rect_filled(rect, 0.0, metrics.ink(14));
                                        }
                                        let hidden = (detail.hide_spoilers
                                            || detail.blur_unwatched)
                                            && !episode.watched;
                                        let thumb_width = metrics.detail_modal_episode_thumb_width;
                                        let (thumb_id, text_id) = (1, 2);
                                        let cells = crate::flex::row([
                                            crate::flex::leaf(Vec2::new(
                                                thumb_width,
                                                thumb_width * 9.0 / 16.0,
                                            ))
                                            .id(thumb_id),
                                            crate::flex::fill().id(text_id),
                                        ])
                                        .gap(Space::S16.px(&metrics))
                                        .padding(crate::flex::Insets {
                                            left: metrics.detail_modal_pad,
                                            top: Space::S10.px(&metrics),
                                            right: metrics.detail_modal_pad
                                                + Dp::D16.px()
                                                + Space::S10.px(&metrics),
                                            bottom: 0.0,
                                        })
                                        .align(crate::flex::Align::Start);
                                        let placed = cells.solve(rect);
                                        let thumb = placed.find(thumb_id).unwrap_or(rect);
                                        let text_area = placed.find(text_id).unwrap_or(rect);
                                        let thumb_radius = Dp::D8.px();
                                        painter.rect_filled(thumb, thumb_radius, metrics.ink(20));
                                        if !hidden {
                                            rounded_artwork(
                                                &painter,
                                                thumb,
                                                thumb_radius,
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
                                            let bar_height = Dp::D4.px();
                                            let track = crate::flex::anchored(
                                                thumb,
                                                Align2::LEFT_BOTTOM,
                                                Vec2::new(
                                                    thumb.width() * episode.progress.min(1.0),
                                                    bar_height,
                                                ),
                                                0.0,
                                            );
                                            painter.rect_filled(track, 0.0, metrics.accent);
                                        }
                                        let text_room = text_area.width();
                                        paint_elided_text(
                                            &painter,
                                            Pos2::new(
                                                text_area.left(),
                                                rect.top() + Space::S12.px(&metrics),
                                            ),
                                            &if detail.show_episode_titles {
                                                format!("{}. {}", episode.number, episode.title)
                                            } else {
                                                episode.number.to_string()
                                            },
                                            crate::fonts::medium(body_size + 1.0),
                                            text_room,
                                            metrics.text_primary,
                                        );
                                        if detail.show_episode_descriptions
                                            && !hidden
                                            && !episode.overview.is_empty()
                                        {
                                            let mut job =
                                                crate::gfx::text::LayoutJob::single_section(
                                                    episode.overview.clone(),
                                                    crate::gfx::TextFormat::simple(
                                                        crate::fonts::regular(body_size - 1.0),
                                                        metrics.ink(tone::subtle()),
                                                    ),
                                                );
                                            job.wrap.max_width = text_room;
                                            job.wrap.max_rows = 4;
                                            let galley =
                                                context.fonts_mut(|fonts| fonts.layout_job(job));
                                            painter.galley(
                                                Pos2::new(
                                                    text_area.left(),
                                                    rect.top()
                                                        + Space::S16.px(&metrics)
                                                        + body_size
                                                        + Space::S4.px(&metrics),
                                                ),
                                                galley,
                                                metrics.text_primary,
                                            );
                                        }
                                        if episode.watched
                                            && let Some(icon) = assets.icon("Check")
                                        {
                                            painter.image(
                                                icon,
                                                crate::flex::anchored(
                                                    rect,
                                                    Align2::RIGHT_CENTER,
                                                    Vec2::splat(Dp::D16.px()),
                                                    metrics.detail_modal_pad,
                                                ),
                                                full_uv(),
                                                metrics.ink(tone::subtle()),
                                            );
                                        }
                                    },
                                );
                                if response.clicked() {
                                    outcome = Some(DetailModalOutcome::Episode(index));
                                }
                            }
                        }

                        let people = detail.cast.iter().chain(detail.crew.iter()).take(24);
                        if people.clone().next().is_some() {
                            section_title(ui, &t("detail.cast_crew"), width, metrics);
                            horizontal_row(
                                ui,
                                metrics,
                                ("fluxa-detail-modal-cast", &detail.id),
                                |ui| {
                                    for (index, person) in people.enumerate() {
                                        crate::detail::tile(
                                            ui,
                                            ("fluxa-detail-modal-cast", index),
                                            Vec2::new(
                                                metrics.detail_modal_cast_size + Dp::D40.px(),
                                                metrics.detail_modal_cast_size + Dp::D64.px(),
                                            ),
                                            Sense::hover(),
                                            |ui, rect| {
                                                let cast_size = metrics.detail_modal_cast_size;
                                                let (photo_id, name_id, role_id) = (1, 2, 3);
                                                let column = crate::flex::column([
                                                    crate::flex::leaf(Vec2::splat(cast_size))
                                                        .id(photo_id),
                                                    crate::flex::leaf(Vec2::new(
                                                        rect.width(),
                                                        body_size,
                                                    ))
                                                    .id(name_id),
                                                    crate::flex::leaf(Vec2::new(
                                                        rect.width(),
                                                        body_size,
                                                    ))
                                                    .id(role_id),
                                                ])
                                                .gap(Space::S4.px(&metrics))
                                                .align(crate::flex::Align::Center);
                                                let placed = column.solve(rect);
                                                let photo = placed.find(photo_id).unwrap_or(rect);
                                                let name_slot =
                                                    placed.find(name_id).unwrap_or(rect);
                                                let role_slot =
                                                    placed.find(role_id).unwrap_or(rect);
                                                let painter = ui.painter();
                                                painter.circle_filled(
                                                    photo.center(),
                                                    metrics.detail_modal_cast_size * 0.5,
                                                    metrics.ink(20),
                                                );
                                                rounded_artwork(
                                                    painter,
                                                    photo,
                                                    metrics.detail_modal_cast_size * 0.5,
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
                                                    Pos2::new(
                                                        rect.left(),
                                                        name_slot.top() + Space::S6.px(&metrics),
                                                    ),
                                                    &person.name,
                                                    crate::fonts::medium(body_size),
                                                    rect.width(),
                                                    metrics.text_primary,
                                                );
                                                if let Some(role) = person.role.as_deref() {
                                                    paint_elided_text(
                                                        painter,
                                                        Pos2::new(
                                                            rect.left(),
                                                            role_slot.top()
                                                                + Space::S6.px(&metrics),
                                                        ),
                                                        role,
                                                        crate::fonts::regular(body_size - 1.0),
                                                        rect.width(),
                                                        metrics.ink(tone::subtle()),
                                                    );
                                                }
                                            },
                                        );
                                    }
                                },
                            );
                            ui.add_space(Space::S12.px(&metrics));
                        }

                        if !detail.similar.is_empty() {
                            section_title(ui, &t("player.recommendations"), width, metrics);
                            horizontal_row(
                                ui,
                                metrics,
                                ("fluxa-detail-modal-similar", &detail.id),
                                |ui| {
                                    for (index, card) in detail.similar.iter().take(30).enumerate()
                                    {
                                        let response = crate::detail::tile(
                                            ui,
                                            ("fluxa-detail-modal-similar", index),
                                            Vec2::new(
                                                metrics.detail_modal_similar_width,
                                                metrics.detail_modal_similar_width * 1.5
                                                    + Dp::D56.px(),
                                            ),
                                            Sense::click(),
                                            |ui, rect| {
                                                poster_card(
                                                    ui.painter(),
                                                    crate::flex::anchored(
                                                        rect,
                                                        Align2::LEFT_TOP,
                                                        Vec2::new(
                                                            metrics.detail_modal_similar_width,
                                                            metrics.detail_modal_similar_width
                                                                * 1.5,
                                                        ),
                                                        0.0,
                                                    ),
                                                    card,
                                                    metrics,
                                                    assets,
                                                    false,
                                                );
                                            },
                                        );
                                        let rect = response.rect;
                                        if let Some(position) = pointer
                                            && ui.clip_rect().intersect(rect).contains(position)
                                        {
                                            hovered_card = Some((index, rect));
                                        }
                                        if response.clicked() {
                                            outcome = Some(DetailModalOutcome::Similar(index));
                                        }
                                    }
                                },
                            );
                        }
                        ui.add_space(metrics.detail_modal_pad);
                    },
                );
            });

            let close = crate::flex::anchored(
                panel,
                Align2::RIGHT_TOP,
                Vec2::splat(Dp::D36.px()),
                Space::S10.px(&metrics),
            );
            let close_response =
                ui.interact(close, Id::new("fluxa-detail-modal-close"), Sense::click());
            painter.circle_filled(
                close.center(),
                close.width() * 0.5,
                Color32::from_black_alpha(if close_response.hovered() { 190 } else { 130 }),
            );
            if let Some(icon) = assets.icon("Close") {
                painter.image(
                    icon,
                    crate::flex::anchored(
                        close,
                        Align2::CENTER_CENTER,
                        Vec2::splat(Dp::D16.px()),
                        0.0,
                    ),
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
                crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
                crate::gfx::StrokeKind::Inside,
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
            hover_preview_reset(context, crate::gfx::Order::Debug);
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
            let (area, galley) = hover_preview_rect(
                context,
                anchor,
                screen,
                &synopsis,
                preview_is_compact(&card),
                metrics,
            );
            context.data_mut(|data| data.insert_temp(preview_id, (index, since, anchor, area)));
            let remaining = 0.5 - (now - since);
            if remaining > 0.0 {
                hover_preview_reset(context, crate::gfx::Order::Debug);
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
                                trailer: true,
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
                    None,
                    crate::gfx::Order::Debug,
                    metrics,
                    assets,
                );
                outcome = outcome.or(match action {
                    Some(PreviewAction::Play) => Some(DetailModalOutcome::SimilarPlay(index)),
                    Some(PreviewAction::Watchlist) => {
                        Some(DetailModalOutcome::SimilarWatchlist(index))
                    }
                    Some(_) => None,
                    None if response.clicked() => Some(DetailModalOutcome::Similar(index)),
                    None => None,
                });
            }
        }
    }
    if let Some(number) = chosen_season {
        context.data_mut(|data| data.insert_temp(season_id, number));
    }
    if let Some(outcome) = outcome.filter(|_| open) {
        context.data_mut(|data| data.insert_temp(outcome_id(), outcome));
    }
}
