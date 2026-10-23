use super::*;
use crate::effects::{Outline, Path, Placement, layers, linear};
use crate::{WatchTicket, localized};
use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Clone, Copy)]
pub(crate) enum TicketAction {
    Join,
}

pub(crate) struct TicketOutput {
    pub action: Option<TicketAction>,
    pub code: Option<CodeInput>,
}

const RADIUS: f32 = 26.0;
const NOTCH: f32 = 13.0;
const STUB: f32 = 124.0;
const DETAILS: f32 = 46.0;
const TEAR_SECONDS: f32 = 0.55;
const SETTLE_SECONDS: f32 = 1.9;

pub(crate) fn ticket_height(width: f32) -> f32 {
    stage_height(width) + STUB
}

fn stage_height(width: f32) -> f32 {
    (width * 0.62).clamp(236.0, 272.0)
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn black(alpha: f32) -> Color32 {
    Color32::from_black_alpha((alpha * 255.0) as u8)
}

fn white(alpha: f32) -> Color32 {
    Color32::from_white_alpha((alpha * 255.0) as u8)
}

fn half(rect: Rect, tear: f32, top: bool) -> Outline {
    let (left, right) = (rect.left(), rect.right());
    if top {
        Path::start(Pos2::new(left + RADIUS, rect.top()))
            .line_to(Pos2::new(right - RADIUS, rect.top()))
            .arc(
                Pos2::new(right - RADIUS, rect.top() + RADIUS),
                RADIUS,
                -FRAC_PI_2,
                0.0,
            )
            .line_to(Pos2::new(right, tear - NOTCH))
            .arc(Pos2::new(right, tear), NOTCH, -FRAC_PI_2, -PI)
            .line_to(Pos2::new(left + NOTCH, tear))
            .arc(Pos2::new(left, tear), NOTCH, 0.0, -FRAC_PI_2)
            .line_to(Pos2::new(left, rect.top() + RADIUS))
            .arc(
                Pos2::new(left + RADIUS, rect.top() + RADIUS),
                RADIUS,
                PI,
                PI + FRAC_PI_2,
            )
            .close()
    } else {
        Path::start(Pos2::new(left + NOTCH, tear))
            .line_to(Pos2::new(right - NOTCH, tear))
            .arc(Pos2::new(right, tear), NOTCH, PI, FRAC_PI_2)
            .line_to(Pos2::new(right, rect.bottom() - RADIUS))
            .arc(
                Pos2::new(right - RADIUS, rect.bottom() - RADIUS),
                RADIUS,
                0.0,
                FRAC_PI_2,
            )
            .line_to(Pos2::new(left + RADIUS, rect.bottom()))
            .arc(
                Pos2::new(left + RADIUS, rect.bottom() - RADIUS),
                RADIUS,
                FRAC_PI_2,
                PI,
            )
            .line_to(Pos2::new(left, tear + NOTCH))
            .arc(Pos2::new(left, tear), NOTCH, FRAC_PI_2, 0.0)
            .close()
    }
}

struct Timeline {
    since: f32,
}

impl Timeline {
    fn load(context: &egui::Context, joined: bool) -> Self {
        let id = Id::new("fluxa-ticket-timeline");
        let now = context.input(|input| input.time) as f32;
        let (was_joined, started) = context
            .data(|data| data.get_temp::<(bool, f32)>(id))
            .unwrap_or((joined, now - SETTLE_SECONDS - 1.0));
        let started = if joined && !was_joined { now } else { started };
        context.data_mut(|data| data.insert_temp(id, (joined, started)));
        let since = if joined { now - started } else { 0.0 };
        if joined && since < SETTLE_SECONDS {
            context.request_repaint();
        }
        Self { since }
    }

    fn tear(&self) -> f32 {
        let t = (self.since / TEAR_SECONDS).clamp(0.0, 1.0);
        if t >= 1.0 { 0.0 } else { (t * PI).sin() }
    }

    fn poster(&self) -> f32 {
        ease((self.since - 0.2) / 0.7)
    }

    fn reveal(&self, delay: f32) -> f32 {
        ease((self.since - delay) / 0.45)
    }
}

fn serial(code: &str) -> u32 {
    code.bytes().fold(17u32, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(byte as u32)
    })
}

fn episode_parts(episode: &str) -> (String, Option<String>) {
    let mut parts = episode.splitn(3, " · ");
    let season = parts.next().unwrap_or_default();
    match (parts.next(), parts.next()) {
        (Some(number), title) => (format!("{season} · {number}"), title.map(ToOwned::to_owned)),
        (None, _) => (season.to_owned(), None),
    }
}

fn scrim(stage: Rect, alpha: f32) -> impl Fn(Pos2) -> Color32 {
    layers(vec![Box::new(linear(
        stage.left_top(),
        stage.left_bottom(),
        vec![
            (0.0, black(0.7 * alpha)),
            (0.28, black(0.18 * alpha)),
            (0.46, black(0.84 * alpha)),
            (0.66, black(0.97 * alpha)),
            (1.0, black(0.98 * alpha)),
        ],
    ))])
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
    let joined = ticket.room.is_some();
    let narrow = width < 400.0;
    let timeline = Timeline::load(ui.ctx(), joined);
    let bump = timeline.tear();

    let top_place = Placement::tilted(
        Pos2::new(rect.left(), tear),
        -0.009 * bump,
        Vec2::new(0.0, -4.0 * bump),
    );
    let stub_place = Placement::tilted(
        Pos2::new(rect.left(), tear),
        0.024 * bump,
        Vec2::new(0.0, 12.0 * bump),
    );
    let stage_shift = top_place.offset;
    let stub_shift = stub_place.offset;

    let target = artwork_target_size(rect.size(), ui.ctx().pixels_per_point());
    let backdrop = ticket.backdrop.as_deref().and_then(|url| {
        let texture = assets.texture_for(Some(url), target, ArtworkPriority::Visible)?;
        Some((texture, assets.texture_size(Some(url))?))
    });
    let dom_backdrop = backdrop.is_none()
        && crate::components::failed_dom_image(
            &painter,
            assets,
            rect,
            RADIUS,
            ticket.backdrop.as_deref(),
            target,
            crate::ImageFit::Cover,
            [0.5, 0.5],
        );
    let poster = backdrop.is_some() || dom_backdrop;
    let poster_alpha = if joined { timeline.poster() } else { 0.0 };

    let top_shape = half(stage, tear, true);
    let bottom_shape = half(stub, tear, false);
    let border = egui::Stroke::new(stroke::hairline(), white(0.1));

    if !dom_backdrop {
        top_shape.fill(&painter, top_place, 18.0, &|_| metrics.surface);
    }
    if let Some((texture, size)) = backdrop {
        let uv = cover_uv(size, stage);
        let tint = white(poster_alpha);
        top_shape.fill_texture(
            &painter,
            top_place,
            18.0,
            texture,
            &|p| {
                Pos2::new(
                    uv.left() + (p.x - stage.left()) / stage.width() * uv.width(),
                    uv.top() + (p.y - stage.top()) / stage.height() * uv.height(),
                )
            },
            &|_| tint,
        );
    }
    if poster {
        top_shape.fill(&painter, top_place, 18.0, &scrim(stage, poster_alpha));
    }
    top_shape.stroke(&painter, top_place, border);

    bottom_shape.fill(&painter, stub_place, 24.0, &|_| metrics.surface_raised);
    bottom_shape.stroke(&painter, stub_place, border);

    let mut x = rect.left() + NOTCH + 10.0;
    let dash = white(0.45 * (1.0 - bump));
    while x < rect.right() - NOTCH - 10.0 {
        painter.line_segment(
            [
                Pos2::new(x, tear),
                Pos2::new((x + 7.0).min(rect.right() - NOTCH - 10.0), tear),
            ],
            egui::Stroke::new(1.5, dash),
        );
        x += 13.0;
    }

    let inner = stage.translate(stage_shift).shrink2(Vec2::new(24.0, 22.0));
    let muted = metrics.text_muted;
    let faint = white(0.34);
    let number = match ticket.room.as_deref() {
        Some(room) => format!("№ {:04}", serial(room) % 10_000),
        None => "№ ——".to_owned(),
    };
    painter.text(
        inner.right_top(),
        Align2::RIGHT_TOP,
        number,
        crate::fonts::bold(11.0),
        faint,
    );
    if let (Some(_), Some(host)) = (ticket.room.as_ref(), ticket.host.as_ref()) {
        let fade = timeline.reveal(0.5);
        let center = Pos2::new(inner.left() + 15.0, inner.top() + 15.0 + (1.0 - fade) * 6.0);
        let mut hosted = painter.clone();
        hosted.set_opacity(fade);
        avatar(
            &hosted,
            assets,
            center,
            15.0,
            &host.name,
            host.avatar.as_deref(),
            metrics,
        );
        hosted.text(
            Pos2::new(center.x + 24.0, center.y),
            Align2::LEFT_CENTER,
            t("watch.ticket.hosted_by").replace("{name}", &host.name),
            crate::fonts::bold(13.0),
            white(0.95),
        );
    }

    let divider = inner.bottom() - DETAILS;
    let mut baseline = divider - 16.0;
    if joined {
        let fade = timeline.reveal(0.45);
        let lift = (1.0 - fade) * 8.0;
        let mut shown = painter.clone();
        shown.set_opacity(fade);
        if let Some(episode) = ticket.episode.as_deref() {
            let (badge, title) = episode_parts(episode);
            let badge_font = crate::fonts::bold(12.0);
            let galley = shown.layout_no_wrap(badge, badge_font, Color32::WHITE);
            let badge_rect = Rect::from_min_size(
                Pos2::new(inner.left(), baseline - 22.0 + lift),
                galley.size() + Vec2::new(14.0, 6.0),
            );
            shown.rect_stroke(
                badge_rect,
                5.0,
                egui::Stroke::new(1.0, white(0.4)),
                egui::StrokeKind::Inside,
            );
            shown.galley(badge_rect.min + Vec2::new(7.0, 3.0), galley, Color32::WHITE);
            if let Some(title) = title {
                let left = badge_rect.right() + 9.0;
                shown.text(
                    Pos2::new(left, badge_rect.center().y),
                    Align2::LEFT_CENTER,
                    truncate_to_width(
                        &shown,
                        &title,
                        &crate::fonts::bold(14.0),
                        inner.right() - left,
                    ),
                    crate::fonts::bold(14.0),
                    white(0.92),
                );
            }
            baseline -= 34.0;
        }
        let logo = ticket.logo.as_deref().and_then(|url| {
            title_logo(
                ui.ctx().pixels_per_point(),
                Some(url),
                Vec2::new(inner.width() * 0.7, 54.0),
                ArtworkPriority::Visible,
                assets,
            )
        });
        match logo {
            Some((texture, size)) => {
                let place =
                    Rect::from_min_size(Pos2::new(inner.left(), baseline - size.y + lift), size);
                shown.image(texture, place, full_uv(), Color32::WHITE);
            }
            None => {
                let title = ticket.title.clone().unwrap_or_else(|| t("watch.title"));
                let font = crate::fonts::bold(if narrow { 26.0 } else { 32.0 });
                shown.text(
                    Pos2::new(inner.left(), baseline + lift),
                    Align2::LEFT_BOTTOM,
                    truncate_to_width(&shown, &title, &font, inner.width()),
                    font,
                    Color32::WHITE,
                );
            }
        }
    } else {
        painter.text(
            Pos2::new(inner.left(), baseline),
            Align2::LEFT_BOTTOM,
            t("watch.ticket.prompt"),
            crate::fonts::regular(14.0),
            muted,
        );
        painter.text(
            Pos2::new(inner.left(), baseline - 24.0),
            Align2::LEFT_BOTTOM,
            t("watch.title"),
            crate::fonts::bold(if narrow { 28.0 } else { 34.0 }),
            Color32::WHITE,
        );
    }

    painter.line_segment(
        [
            Pos2::new(inner.left(), divider),
            Pos2::new(inner.right(), divider),
        ],
        egui::Stroke::new(stroke::hairline(), white(0.14)),
    );
    let room = ticket.room.as_deref().unwrap_or("------");
    let (reveal, dim) = if joined {
        (timeline.reveal(0.7), 1.0)
    } else {
        (1.0, 0.28)
    };
    let seat = ticket
        .members
        .iter()
        .position(|member| member.you)
        .map_or(ticket.members.len().max(1), |index| index + 1);
    let row = (b'A' + (serial(room) % 8) as u8) as char;
    let cells = [
        (t("watch.ticket.hall"), "1".to_owned()),
        (t("watch.ticket.row"), row.to_string()),
        (t("watch.ticket.seat_no"), seat.to_string()),
    ];
    let column = inner.width() / 3.0;
    for (index, (label, value)) in cells.into_iter().enumerate() {
        let left = inner.left() + column * index as f32;
        painter.text(
            Pos2::new(left, divider + 14.0),
            Align2::LEFT_TOP,
            label,
            crate::fonts::bold(9.5),
            white(0.62 * reveal),
        );
        painter.text(
            Pos2::new(left, divider + 27.0),
            Align2::LEFT_TOP,
            if joined { value } else { "—".to_owned() },
            crate::fonts::bold(19.0),
            white(reveal * if joined { 1.0 } else { dim }),
        );
    }

    let inner = stub.translate(stub_shift).shrink2(Vec2::new(24.0, 20.0));
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
        faint
    };
    painter.text(
        inner.left_top(),
        Align2::LEFT_TOP,
        label,
        crate::fonts::bold(10.0),
        label_color,
    );
    let row_rect = Rect::from_min_max(Pos2::new(inner.left(), inner.top() + 24.0), inner.max);
    match ticket.room.as_deref() {
        None => {
            let field =
                Rect::from_min_size(row_rect.min, Vec2::new(row_rect.width().min(392.0), 60.0));
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(field));
            let input =
                code_input_styled(&mut child, code, 6, field.width(), metrics, CodeLook::Boxes);
            if input.submitted {
                output.action = Some(TicketAction::Join);
            }
            output.code = Some(input);
        }
        Some(room) => {
            let shown = if narrow {
                room.to_owned()
            } else {
                room.chars().map(String::from).collect::<Vec<_>>().join(" ")
            };
            painter.text(
                Pos2::new(row_rect.left(), row_rect.center().y),
                Align2::LEFT_CENTER,
                shown,
                crate::fonts::bold(if narrow { 22.0 } else { 26.0 }),
                Color32::WHITE,
            );
            let seats = seat_count(ticket.members.len(), ticket.max_members);
            if seats > 0 {
                let radius = if narrow { 14.0 } else { 16.0 };
                let step = radius * 2.0 - 5.0;
                let span = step * (seats - 1) as f32 + radius * 2.0;
                let start = inner.right() - span;
                for index in 0..seats {
                    let fade = timeline.reveal(0.8 + index as f32 * 0.07);
                    let mut cell = painter.clone();
                    cell.set_opacity(fade);
                    let center = Pos2::new(
                        start + radius + step * index as f32,
                        row_rect.center().y + (1.0 - fade) * 8.0,
                    );
                    match ticket.members.get(index) {
                        Some(member) => {
                            cell.circle_filled(center, radius + 2.0, metrics.surface_raised);
                            avatar(
                                &cell,
                                assets,
                                center,
                                radius,
                                &member.name,
                                member.avatar.as_deref(),
                                metrics,
                            );
                            if member.you {
                                cell.circle_stroke(
                                    center,
                                    radius + 1.0,
                                    egui::Stroke::new(2.0, Color32::WHITE),
                                );
                            }
                        }
                        None => {
                            cell.circle_stroke(center, radius, egui::Stroke::new(1.2, white(0.28)));
                            cell.text(
                                center,
                                Align2::CENTER_CENTER,
                                "+",
                                crate::fonts::bold(13.0),
                                white(0.34),
                            );
                        }
                    }
                }
                let total = ticket.max_members.max(ticket.members.len() as u32);
                painter.text(
                    Pos2::new(inner.right(), inner.top()),
                    Align2::RIGHT_TOP,
                    format!("{} / {}", ticket.members.len(), total),
                    crate::fonts::bold(11.0),
                    muted,
                );
            }
        }
    }
    output
}
