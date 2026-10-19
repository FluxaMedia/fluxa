use super::*;
use crate::{WatchTicket, localized};

#[derive(Clone, Copy)]
pub(crate) enum TicketAction {
    Join,
}

pub(crate) struct TicketOutput {
    pub action: Option<TicketAction>,
    pub code: Option<CodeInput>,
}

const RADIUS: f32 = 22.0;
const NOTCH: f32 = 13.0;
const STUB: f32 = 118.0;

pub(crate) fn ticket_height(width: f32) -> f32 {
    stage_height(width) + STUB
}

fn stage_height(width: f32) -> f32 {
    (width * 0.42).clamp(176.0, 236.0)
}

pub(crate) fn watch_ticket(
    ui: &mut Ui,
    ticket: &WatchTicket,
    code: &mut String,
    width: f32,
    language: &str,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
) -> TicketOutput {
    let t = |key: &str| localized(key, language);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, ticket_height(width)), Sense::hover());
    let tear = rect.top() + stage_height(width);
    let stage = Rect::from_min_max(rect.min, Pos2::new(rect.right(), tear));
    let stub = Rect::from_min_max(Pos2::new(rect.left(), tear), rect.max);
    let painter = ui.painter().clone();

    let backdrop = ticket.backdrop.as_deref().and_then(|url| {
        let texture = assets.texture_for(
            Some(url),
            artwork_target_size(rect.size(), ui.ctx().pixels_per_point()),
            ArtworkPriority::Visible,
        )?;
        Some((texture, assets.texture_size(Some(url))?))
    });
    match backdrop {
        Some((texture, size)) => {
            rounded_image(
                &painter,
                texture,
                rect,
                RADIUS,
                cover_uv(size, rect),
                Color32::WHITE,
            );
            painter.rect_filled(rect, RADIUS, Color32::from_black_alpha(90));
            paint_vertical_gradient(
                &painter,
                Rect::from_min_max(Pos2::new(stage.left(), stage.center().y), stage.max),
                Color32::TRANSPARENT,
                Color32::from_black_alpha(210),
            );
            painter.rect_filled(
                stub,
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: RADIUS as u8,
                    se: RADIUS as u8,
                },
                Color32::from_black_alpha(232),
            );
        }
        None => {
            painter.rect_filled(rect, RADIUS, metrics.surface_raised);
            painter.rect_filled(
                stub,
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: RADIUS as u8,
                    se: RADIUS as u8,
                },
                metrics.surface,
            );
        }
    }
    painter.rect_stroke(
        rect,
        RADIUS,
        egui::Stroke::new(stroke::hairline(), Color32::from_white_alpha(28)),
        egui::StrokeKind::Inside,
    );
    for x in [rect.left(), rect.right()] {
        painter.circle_filled(Pos2::new(x, tear), NOTCH, metrics.background);
    }
    let mut x = rect.left() + NOTCH + 10.0;
    while x < rect.right() - NOTCH - 10.0 {
        painter.line_segment(
            [
                Pos2::new(x, tear),
                Pos2::new((x + 7.0).min(rect.right() - NOTCH - 10.0), tear),
            ],
            egui::Stroke::new(1.5, Color32::from_white_alpha(110)),
        );
        x += 13.0;
    }

    let inner = stage.shrink2(Vec2::new(26.0, 22.0));
    let soft = Color32::from_white_alpha(170);
    painter.text(
        inner.left_top(),
        Align2::LEFT_TOP,
        t("watch.ticket.admit"),
        crate::fonts::bold(11.0),
        soft,
    );
    if let Some(room) = ticket.room.as_deref() {
        painter.text(
            inner.right_top(),
            Align2::RIGHT_TOP,
            t("watch.ticket.room").replace("{code}", room),
            crate::fonts::bold(11.0),
            soft,
        );
    }
    if let (Some(_), Some(host)) = (ticket.room.as_ref(), ticket.host.as_ref()) {
        let center = Pos2::new(inner.left() + 12.0, inner.top() + 38.0);
        avatar(
            &painter,
            assets,
            center,
            12.0,
            &host.name,
            host.avatar.as_deref(),
            metrics,
        );
        painter.text(
            Pos2::new(center.x + 20.0, center.y),
            Align2::LEFT_CENTER,
            t("watch.ticket.hosted_by").replace("{name}", &host.name),
            crate::fonts::bold(13.0),
            Color32::from_white_alpha(230),
        );
    }
    let (headline, subline) = match (&ticket.room, &ticket.title) {
        (Some(_), Some(title)) => (title.clone(), ticket.episode.clone()),
        (Some(_), None) => (t("watch.title"), None),
        (None, _) => (t("watch.title"), Some(t("watch.ticket.prompt"))),
    };
    let mut baseline = inner.bottom();
    if let Some(subline) = subline {
        painter.text(
            Pos2::new(inner.left(), baseline),
            Align2::LEFT_BOTTOM,
            subline,
            crate::fonts::regular(14.0),
            Color32::from_white_alpha(210),
        );
        baseline -= 24.0;
    }
    let logo = ticket
        .room
        .as_ref()
        .and(ticket.logo.as_deref())
        .and_then(|url| {
            title_logo(
                ui.ctx().pixels_per_point(),
                Some(url),
                Vec2::new(inner.width() * 0.55, 70.0),
                ArtworkPriority::Visible,
                assets,
            )
        });
    match logo {
        Some((texture, size)) => {
            let place = Rect::from_min_size(Pos2::new(inner.left(), baseline - size.y), size);
            painter.image(texture, place, full_uv(), Color32::WHITE);
        }
        None => {
            painter.text(
                Pos2::new(inner.left(), baseline),
                Align2::LEFT_BOTTOM,
                truncate_to_width(
                    &painter,
                    &headline,
                    &crate::fonts::bold(28.0),
                    inner.width(),
                ),
                crate::fonts::bold(28.0),
                Color32::WHITE,
            );
        }
    }

    let inner = stub.shrink2(Vec2::new(26.0, 18.0));
    let mut output = TicketOutput {
        action: None,
        code: None,
    };
    let label = match (&ticket.room, ticket.connecting, &ticket.error) {
        (_, true, _) => t("watch.connecting"),
        (None, _, Some(error)) => t(error),
        (Some(_), _, _) => t("watch.ticket.seat"),
        (None, _, None) => t("watch.ticket.code"),
    };
    let label_color = if ticket.error.is_some() && ticket.room.is_none() {
        danger_color()
    } else {
        soft
    };
    painter.text(
        inner.left_top(),
        Align2::LEFT_TOP,
        label,
        crate::fonts::bold(11.0),
        label_color,
    );
    let row = Rect::from_min_max(Pos2::new(inner.left(), inner.top() + 22.0), inner.max);
    match ticket.room.as_deref() {
        None => {
            let field = Rect::from_min_size(row.min, Vec2::new(row.width().min(360.0), 60.0));
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(field));
            let input = code_input(&mut child, code, 6, field.width(), metrics);
            if input.submitted {
                output.action = Some(TicketAction::Join);
            }
            output.code = Some(input);
        }
        Some(room) => {
            let narrow = width < 480.0;
            let shown = if narrow {
                room.to_owned()
            } else {
                room.chars().map(String::from).collect::<Vec<_>>().join(" ")
            };
            painter.text(
                Pos2::new(row.left(), row.center().y),
                Align2::LEFT_CENTER,
                shown,
                crate::fonts::bold(if narrow { 22.0 } else { 28.0 }),
                Color32::WHITE,
            );
            let seats = seat_count(ticket.members.len(), ticket.max_members);
            if seats > 0 {
                let radius = if narrow { 13.0 } else { 16.0 };
                let span = seat_row_width(seats, radius);
                let start = Pos2::new(inner.right() - span, row.center().y);
                seat_row(
                    &painter,
                    assets,
                    &ticket.members,
                    seats,
                    start,
                    radius,
                    metrics,
                );
                let total = ticket.max_members.max(ticket.members.len() as u32);
                painter.text(
                    Pos2::new(inner.right(), inner.top()),
                    Align2::RIGHT_TOP,
                    t("watch.seats")
                        .replace("{taken}", &ticket.members.len().to_string())
                        .replace("{total}", &total.to_string()),
                    crate::fonts::bold(11.0),
                    soft,
                );
            }
        }
    }
    output
}
