use super::cards::paint_elided_text;
use super::*;
use crate::{DetailModel, metrics_for_assets};

const PANEL_MAX_WIDTH: f32 = 980.0;
const PANEL_MARGIN: f32 = 24.0;
const BUTTON_HEIGHT: f32 = 48.0;
const PAD: f32 = 28.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailModalOutcome {
    Close,
    Play,
    Watchlist,
    OpenPage,
}

fn open_id() -> Id {
    Id::new("fluxa-detail-modal-open")
}

fn outcome_id() -> Id {
    Id::new("fluxa-detail-modal-outcome")
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
    let appear = crate::motion::appear(
        context,
        Id::new("fluxa-detail-modal-appear"),
        crate::motion::normal(),
        crate::motion::Ease::Out,
    );
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let width = (viewport.width - PANEL_MARGIN * 2.0).min(PANEL_MAX_WIDTH);
    let image_height = (width * 9.0 / 16.0).min((viewport.height - PANEL_MARGIN * 2.0) * 0.55);
    let body_size = (metrics.screen_card_subtitle_size + 3.0).max(15.0);
    let text_width = if viewport.is_compact() {
        width - PAD * 2.0
    } else {
        (width - PAD * 3.0) * 0.62
    };
    let mut job = egui::text::LayoutJob::single_section(
        detail.description.clone(),
        egui::TextFormat::simple(crate::fonts::regular(body_size), metrics.ink(tone::SUBTLE)),
    );
    job.wrap.max_width = text_width;
    job.wrap.max_rows = 6;
    let synopsis = context.fonts_mut(|fonts| fonts.layout_job(job));
    let facts = detail.show_info.iter().take(5).collect::<Vec<_>>();
    let meta_height = body_size + 10.0;
    let side_height = facts.len() as f32 * (body_size + 10.0);
    let body_height = (meta_height + synopsis.size().y + 12.0).max(side_height);
    let height = (image_height + PAD + BUTTON_HEIGHT + PAD * 0.75 + body_height + PAD)
        .min(viewport.height - PANEL_MARGIN * 2.0);
    let scale = 0.94 + 0.06 * appear;
    let panel = Rect::from_center_size(screen.center(), Vec2::new(width, height));
    let layer = egui::LayerId::new(egui::Order::Debug, Id::new("fluxa-detail-modal"));
    context.set_transform_layer(
        layer,
        egui::emath::TSTransform::new(panel.center().to_vec2() * (1.0 - scale), scale),
    );
    let mut outcome = None;
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
            let image = Rect::from_min_size(panel.min, Vec2::new(width, image_height));
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
                    image,
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
            let close = Rect::from_center_size(
                Pos2::new(image.right() - 28.0, image.top() + 28.0),
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

            let row_y = image.bottom() + PAD * 0.5;
            let round = BUTTON_HEIGHT;
            let gap = 10.0;
            let more = Rect::from_min_size(
                Pos2::new(panel.right() - PAD - round, row_y),
                Vec2::splat(round),
            );
            let watchlist = Rect::from_min_size(
                Pos2::new(more.left() - gap - round, row_y),
                Vec2::splat(round),
            );
            let play_rect = Rect::from_min_max(
                Pos2::new(panel.left() + PAD, row_y),
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

            let left = panel.left() + PAD;
            let mut y = row_y + BUTTON_HEIGHT + PAD * 0.75;
            if !detail.meta_line.is_empty() {
                paint_elided_text(
                    &painter,
                    Pos2::new(left, y),
                    &detail.meta_line,
                    crate::fonts::regular(body_size),
                    text_width,
                    metrics.ink(tone::BODY),
                );
            }
            y += meta_height + 2.0;
            painter.galley(Pos2::new(left, y), synopsis, metrics.text_primary);

            if !viewport.is_compact() {
                let side_left = left + text_width + PAD;
                let side_width = panel.right() - PAD - side_left;
                let mut side_y = row_y + BUTTON_HEIGHT + PAD * 0.75;
                for (name, value) in facts {
                    let label_width = side_width * 0.38;
                    paint_elided_text(
                        &painter,
                        Pos2::new(side_left, side_y),
                        name,
                        crate::fonts::regular(body_size),
                        label_width,
                        metrics.ink(tone::SUBTLE),
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
            }
            painter.rect_stroke(
                panel,
                radius,
                egui::Stroke::new(stroke::HAIRLINE, metrics.border),
                egui::StrokeKind::Inside,
            );
        });
    if let Some(outcome) = outcome {
        context.data_mut(|data| data.insert_temp(outcome_id(), outcome));
    }
}
