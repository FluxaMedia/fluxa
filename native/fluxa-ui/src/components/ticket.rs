use super::*;
use crate::effects::{Outline, Path, Placement, layers, linear, paint_radial, radial};
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

const RADIUS: f32 = 24.0;
const NOTCH: f32 = 13.0;
const STUB: f32 = 118.0;
const META: f32 = 44.0;
const TEAR_SECONDS: f32 = 0.55;
const SETTLE_SECONDS: f32 = 1.9;
const WARM: [u8; 3] = [255, 170, 90];

pub(crate) fn ticket_height(width: f32) -> f32 {
    stage_height(width) + STUB
}

fn stage_height(width: f32) -> f32 {
    (width * 0.44).clamp(190.0, 252.0)
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn warm(alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(WARM[0], WARM[1], WARM[2], (alpha * 255.0) as u8)
}

fn black(alpha: f32) -> Color32 {
    Color32::from_black_alpha((alpha * 255.0) as u8)
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

fn paper(rect: Rect) -> impl Fn(Pos2) -> Color32 {
    layers(vec![
        Box::new(linear(
            rect.left_top(),
            rect.right_bottom(),
            vec![
                (0.0, Color32::from_rgb(46, 40, 38)),
                (1.0, Color32::from_rgb(24, 22, 24)),
            ],
        )),
        Box::new(linear(
            rect.left_top(),
            rect.left_bottom(),
            vec![
                (0.0, Color32::from_white_alpha(14)),
                (0.3, Color32::TRANSPARENT),
            ],
        )),
    ])
}

fn blend(under: Color32, over: Color32) -> Color32 {
    let a = over.a() as f32 / 255.0;
    let mix = |u: u8, o: u8| (u as f32 * (1.0 - a) + o as f32 * a) as u8;
    let alpha = (under.a() as f32 + over.a() as f32 * (1.0 - under.a() as f32 / 255.0)) as u8;
    let unmult = |u: u8, o: u8| if alpha == 0 { 0 } else { mix(u, o) };
    Color32::from_rgba_unmultiplied(unmult(under.r(), over.r()), unmult(under.g(), over.g()), unmult(under.b(), over.b()), alpha)
}

fn notch_shade(tear: f32, left: f32, right: f32) -> Vec<Box<dyn Fn(Pos2) -> Color32>> {
    [left, right]
        .into_iter()
        .map(|x| {
            Box::new(radial(
                Pos2::new(x, tear),
                Vec2::splat(NOTCH + 18.0),
                vec![
                    (NOTCH / (NOTCH + 18.0), black(0.5)),
                    (1.0, Color32::TRANSPARENT),
                ],
            )) as Box<dyn Fn(Pos2) -> Color32>
        })
        .collect()
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
        if t >= 1.0 {
            0.0
        } else {
            (t * PI).sin()
        }
    }

    fn poster(&self) -> f32 {
        ease((self.since - 0.2) / 0.7)
    }

    fn reveal(&self, delay: f32) -> f32 {
        ease((self.since - delay) / 0.45)
    }
}

fn serial(code: &str) -> u32 {
    code.bytes()
        .fold(17u32, |acc, byte| acc.wrapping_mul(31).wrapping_add(byte as u32))
}

fn paint_barcode(painter: &Painter, right_bottom: Pos2, code: &str, alpha: f32) {
    let mut seed = serial(code);
    let mut x = right_bottom.x;
    let color = Color32::from_white_alpha((alpha * 140.0) as u8);
    for _ in 0..22 {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        let width = 1.0 + ((seed >> 16) % 4) as f32 * 0.8;
        x -= width;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(x, right_bottom.y - 30.0),
                Pos2::new(x + width, right_bottom.y),
            ),
            0.0,
            color,
        );
        x -= 1.0 + ((seed >> 8) % 2) as f32;
    }
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
    let poster_alpha = if joined { timeline.poster() } else { 1.0 };

    let top_shape = half(stage, tear, true);
    let bottom_shape = half(stub, tear, false);
    let stage_h = stage.height();

    if !dom_backdrop {
        let base = paper(rect);
        top_shape.fill(&painter, top_place, 18.0, &base);
    }
    if let Some((texture, size)) = backdrop {
        let uv = cover_uv(size, rect);
        let tint = Color32::from_white_alpha((poster_alpha * 255.0) as u8);
        top_shape.fill_texture(
            &painter,
            top_place,
            18.0,
            texture,
            &|p| {
                Pos2::new(
                    uv.left() + (p.x - rect.left()) / rect.width() * uv.width(),
                    uv.top() + (p.y - rect.top()) / stage_h * uv.height(),
                )
            },
            &|_| tint,
        );
    }
    if poster {
        let ground = paper(rect);
        let top_dim = layers(vec![Box::new(linear(
            stage.left_top(),
            stage.left_bottom(),
            vec![
                (0.0, black(0.35 * poster_alpha)),
                (0.35, Color32::TRANSPARENT),
            ],
        ))]);
        let darken = move |p: Pos2| {
            let fade = (((p.y - stage.top()) / stage_h - 0.38) / 0.62).clamp(0.0, 1.0);
            let fade = fade * fade * (3.0 - 2.0 * fade) * 0.97 * poster_alpha;
            let base = ground(p);
            let cover = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), (fade * 255.0) as u8);
            let top = top_dim(p);
            blend(top, cover)
        };
        top_shape.fill(&painter, top_place, 18.0, &darken);
    }
    let shade_top = layers(notch_shade(tear, stage.left(), stage.right()));
    top_shape.fill(&painter, top_place, 12.0, &shade_top);
    top_shape.grain(&painter, top_place, 0.07);
    top_shape.stroke(
        &painter,
        top_place,
        egui::Stroke::new(stroke::hairline(), Color32::from_white_alpha(30)),
    );

    bottom_shape.fill(&painter, stub_place, 24.0, &paper(rect));
    let shade_bottom = layers(notch_shade(tear, stub.left(), stub.right()));
    bottom_shape.fill(&painter, stub_place, 12.0, &shade_bottom);
    bottom_shape.stroke(
        &painter,
        stub_place,
        egui::Stroke::new(stroke::hairline(), Color32::from_white_alpha(30)),
    );

    let mut x = rect.left() + NOTCH + 10.0;
    let dash = Color32::from_white_alpha((90.0 * (1.0 - bump)) as u8);
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

    let inner = stage.translate(stage_shift).shrink2(Vec2::new(26.0, 22.0));
    let soft = Color32::from_white_alpha(170);
    painter.text(
        inner.left_top(),
        Align2::LEFT_TOP,
        t("watch.ticket.admit"),
        crate::fonts::bold(11.0),
        soft,
    );
    let number = match ticket.room.as_deref() {
        Some(room) => format!("№ {:04}", serial(room) % 10_000),
        None => "№ ——".to_owned(),
    };
    painter.text(
        inner.right_top(),
        Align2::RIGHT_TOP,
        number,
        crate::fonts::bold(11.0),
        Color32::from_white_alpha(120),
    );
    if let (Some(_), Some(host)) = (ticket.room.as_ref(), ticket.host.as_ref()) {
        let fade = timeline.reveal(0.45);
        let lift = (1.0 - fade) * 8.0;
        let center = Pos2::new(inner.left() + 12.0, inner.top() + 38.0 + lift);
        let mut hosted = painter.clone();
        hosted.set_opacity(fade);
        avatar(
            &hosted,
            assets,
            center,
            12.0,
            &host.name,
            host.avatar.as_deref(),
            metrics,
        );
        hosted.text(
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
    let mut baseline = inner.bottom() - if joined { META } else { 0.0 };
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
            let fade = timeline.reveal(0.35);
            let place = Rect::from_min_size(
                Pos2::new(inner.left(), baseline - size.y + (1.0 - fade) * 14.0),
                size,
            );
            painter.image(
                texture,
                place,
                full_uv(),
                Color32::from_white_alpha((fade * 255.0) as u8),
            );
        }
        None => {
            painter.text(
                Pos2::new(inner.left(), baseline),
                Align2::LEFT_BOTTOM,
                truncate_to_width(
                    &painter,
                    &headline,
                    &crate::fonts::bold(28.0),
                    inner.width() - 90.0,
                ),
                crate::fonts::bold(28.0),
                Color32::WHITE,
            );
        }
    }
    if let Some(room) = ticket.room.as_deref() {
        let fade = timeline.reveal(0.6);
        let lift = (1.0 - fade) * 10.0;
        let row = (serial(room) % 8) as u8;
        let seat = ticket
            .members
            .iter()
            .position(|member| member.you)
            .map_or(ticket.members.len().max(1), |index| index + 1);
        let cells = [
            (t("watch.ticket.hall"), "1".to_owned()),
            (t("watch.ticket.row"), ((b'A' + row) as char).to_string()),
            (t("watch.ticket.seat_no"), seat.to_string()),
        ];
        let mut cursor = inner.left();
        let top = inner.bottom() - META + 10.0 + lift;
        for (label, value) in cells {
            let label_color = Color32::from_white_alpha((fade * 150.0) as u8);
            let value_color = Color32::from_white_alpha((fade * 255.0) as u8);
            let label_rect = painter.text(
                Pos2::new(cursor, top),
                Align2::LEFT_TOP,
                label,
                crate::fonts::bold(9.0),
                label_color,
            );
            painter.text(
                Pos2::new(cursor, top + 13.0),
                Align2::LEFT_TOP,
                value,
                crate::fonts::bold(17.0),
                value_color,
            );
            cursor += label_rect.width().max(34.0) + 24.0;
        }
        if width >= 480.0 {
            paint_barcode(
                &painter,
                Pos2::new(inner.right(), inner.bottom() - 2.0 + lift),
                room,
                fade,
            );
        }
    }

    let inner = stub.translate(stub_shift).shrink2(Vec2::new(26.0, 18.0));
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
            let field = Rect::from_min_size(row.min, Vec2::new(row.width().min(392.0), 60.0));
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(field));
            let input =
                code_input_styled(&mut child, code, 6, field.width(), metrics, CodeLook::Boxes);
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
                let pitch = seat_row_width(2, radius) - seat_row_width(1, radius);
                for index in 0..seats {
                    let fade = timeline.reveal(0.5 + index as f32 * 0.07);
                    let mut cell = painter.clone();
                    cell.set_opacity(fade);
                    let at = Pos2::new(start.x + index as f32 * pitch, start.y + (1.0 - fade) * 8.0);
                    let rest = ticket.members.get(index..).unwrap_or(&[]);
                    if rest.first().is_some_and(|member| member.you) {
                        let glow = 0.7 + 0.3 * (ui.input(|input| input.time) as f32 * 2.6).sin();
                        let center = at + Vec2::new(radius, 0.0);
                        paint_radial(
                            &cell,
                            Rect::from_center_size(center, Vec2::splat(radius * 6.0)),
                            center,
                            Vec2::splat(radius * 2.8),
                            warm(0.45 * glow),
                            Color32::TRANSPARENT,
                        );
                        cell.circle_stroke(
                            center,
                            radius + 4.0,
                            egui::Stroke::new(2.0, warm(0.9 * glow)),
                        );
                        ui.ctx().request_repaint();
                    }
                    seat_row(&cell, assets, rest, 1, at, radius, metrics);
                }
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
