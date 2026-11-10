use super::*;
use crate::dp::Dp;
use crate::effects::{Outline, Path, Placement, layers, linear};
use crate::{Space, WatchTicket, localized};
use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Clone, Copy)]
pub(crate) enum TicketAction {
    Join,
}

pub(crate) struct TicketOutput {
    pub action: Option<TicketAction>,
    pub code: Option<CodeInput>,
}

const TEAR_SECONDS: f32 = 0.55;
const SETTLE_SECONDS: f32 = 1.9;

pub(crate) fn ticket_height(width: f32) -> f32 {
    stage_height(width) + crate::metrics::active().surface_ticket_stub
}

fn stage_height(width: f32) -> f32 {
    crate::metrics::active().ticket_width.of(width)
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

fn half(rect: Rect, tear: f32, top: bool, radius: f32, notch: f32) -> Outline {
    let (left, right) = (rect.left(), rect.right());
    if top {
        Path::start(Pos2::new(left + radius, rect.top()))
            .line_to(Pos2::new(right - radius, rect.top()))
            .arc(
                Pos2::new(right - radius, rect.top() + radius),
                radius,
                -FRAC_PI_2,
                0.0,
            )
            .line_to(Pos2::new(right, tear - notch))
            .arc(Pos2::new(right, tear), notch, -FRAC_PI_2, -PI)
            .line_to(Pos2::new(left + notch, tear))
            .arc(Pos2::new(left, tear), notch, 0.0, -FRAC_PI_2)
            .line_to(Pos2::new(left, rect.top() + radius))
            .arc(
                Pos2::new(left + radius, rect.top() + radius),
                radius,
                PI,
                PI + FRAC_PI_2,
            )
            .close()
    } else {
        Path::start(Pos2::new(left + notch, tear))
            .line_to(Pos2::new(right - notch, tear))
            .arc(Pos2::new(right, tear), notch, PI, FRAC_PI_2)
            .line_to(Pos2::new(right, rect.bottom() - radius))
            .arc(
                Pos2::new(right - radius, rect.bottom() - radius),
                radius,
                0.0,
                FRAC_PI_2,
            )
            .line_to(Pos2::new(left + radius, rect.bottom()))
            .arc(
                Pos2::new(left + radius, rect.bottom() - radius),
                radius,
                FRAC_PI_2,
                PI,
            )
            .line_to(Pos2::new(left, tear + notch))
            .arc(Pos2::new(left, tear), notch, FRAC_PI_2, 0.0)
            .close()
    }
}

struct Timeline {
    since: f32,
}

impl Timeline {
    fn load(context: &crate::gfx::Context, joined: bool) -> Self {
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
    let narrow = width < metrics.surface_ticket_narrow_width;
    let (radius, notch) = (Dp::D24.px(), Dp::D12.px());
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
            radius,
            ticket.backdrop.as_deref(),
            target,
            crate::ImageFit::Cover,
            [0.5, 0.5],
        );
    let poster = backdrop.is_some() || dom_backdrop;
    let poster_alpha = if joined { timeline.poster() } else { 0.0 };

    let top_shape = half(stage, tear, true, radius, notch);
    let bottom_shape = half(stub, tear, false, radius, notch);
    let border = crate::gfx::Stroke::new(stroke::hairline(), white(0.1));

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

    let dash = white(0.45 * (1.0 - bump));
    let seam = crate::flex::anchored(
        Rect::from_min_size(Pos2::new(rect.left(), tear), Vec2::new(rect.width(), 0.0)),
        Align2::CENTER_CENTER,
        Vec2::new(rect.width() - (notch + Dp::D10.px()) * 2.0, 0.0),
        0.0,
    );
    let mut x = seam.left();
    while x < seam.right() {
        painter.line_segment(
            [
                Pos2::new(x, tear),
                Pos2::new((x + Dp::D6.px()).min(seam.right()), tear),
            ],
            crate::gfx::Stroke::new(1.5, dash),
        );
        x += Dp::D12.px();
    }

    let muted = metrics.text_muted;
    let faint = white(0.34);
    let number = match ticket.room.as_deref() {
        Some(room) => format!("№ {:04}", serial(room) % 10_000),
        None => "№ ——".to_owned(),
    };
    let pad_x = Dp::D24.px();
    let content_width = rect.width() - pad_x * 2.0;
    let inline_gap = Dp::D8.px();
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
    let row_letter = (b'A' + (serial(room) % 8) as u8) as char;
    let cells = [
        (t("watch.ticket.hall"), "1".to_owned()),
        (t("watch.ticket.row"), row_letter.to_string()),
        (t("watch.ticket.seat_no"), seat.to_string()),
    ];
    let hosted = ticket
        .room
        .as_ref()
        .zip(ticket.host.as_ref())
        .map(|(_, host)| host);
    let hosted_text = hosted
        .map(|host| t("watch.ticket.hosted_by").replace("{name}", &host.name))
        .unwrap_or_default();
    let number_font = crate::fonts::bold(11.0);
    let hosted_font = crate::fonts::bold(13.0);

    let (top_id, host_id, avatar_id, hosted_id, number_id) = (1, 2, 3, 4, 5);
    let (copy_id, main_id, episode_id, badge_id, badge_text_id, episode_title_id) =
        (6, 7, 8, 9, 10, 11);
    let (details_id, cell_id, label_cell_id, prompt_id) = (12, 100, 200, 13);
    let host_node = match hosted {
        Some(_) => crate::flex::row([
            crate::flex::leaf(Vec2::splat(Dp::D14.px() * 2.0)).id(avatar_id),
            crate::flex::text(&painter, &hosted_text, &hosted_font).id(hosted_id),
        ])
        .gap(inline_gap)
        .align(crate::flex::Align::Center)
        .id(host_id),
        None => crate::flex::leaf(Vec2::ZERO).id(host_id),
    };
    let top_row = crate::flex::row([
        host_node,
        crate::flex::fill(),
        crate::flex::text(&painter, &number, &number_font).id(number_id),
    ])
    .id(top_id)
    .align(crate::flex::Align::Start);

    let episode = ticket
        .episode
        .as_deref()
        .filter(|_| joined)
        .map(episode_parts);
    let logo = ticket.logo.as_deref().filter(|_| joined).and_then(|url| {
        title_logo(
            ui.ctx().pixels_per_point(),
            Some(url),
            Vec2::new(
                content_width * metrics.surface_ticket_logo_share,
                Dp::D56.px(),
            ),
            ArtworkPriority::Visible,
            assets,
        )
    });
    let title = ticket.title.clone().unwrap_or_else(|| t("watch.title"));
    let title_font = crate::fonts::bold(if narrow { 26.0 } else { 32.0 });
    let idle_title_font = crate::fonts::bold(if narrow { 28.0 } else { 34.0 });
    let idle_title = t("watch.title");
    let prompt_font = crate::fonts::regular(14.0);
    let badge_font = crate::fonts::bold(12.0);
    let episode_title_font = crate::fonts::bold(14.0);
    let mut copy_children = Vec::new();
    if joined {
        copy_children.push(match &logo {
            Some((_, size)) => crate::flex::leaf(*size).id(main_id),
            None => {
                let size = crate::flex::text_size(&painter, &title, &title_font);
                crate::flex::leaf(Vec2::new(size.x.min(content_width), size.y)).id(main_id)
            }
        });
        if let Some((badge, title)) = &episode {
            let mut parts = vec![
                crate::flex::row([
                    crate::flex::text(&painter, badge, &badge_font).id(badge_text_id)
                ])
                .padding(crate::flex::Insets::xy(Dp::D6.px(), Dp::D3.px()))
                .id(badge_id)
                .align(crate::flex::Align::Start),
            ];
            if title.is_some() {
                parts.push(crate::flex::fill().id(episode_title_id));
            }
            copy_children.push(
                crate::flex::row(parts)
                    .gap(inline_gap)
                    .align(crate::flex::Align::Center)
                    .width(content_width)
                    .height(Dp::D20.px())
                    .id(episode_id),
            );
        }
    } else {
        copy_children.push(crate::flex::text(&painter, &idle_title, &idle_title_font).id(main_id));
        copy_children.push(
            crate::flex::leaf(Vec2::new(content_width, Space::S24.px(&metrics))).id(prompt_id),
        );
    }
    let copy = crate::flex::column(copy_children)
        .gap(if joined { Space::S12.px(&metrics) } else { 0.0 })
        .id(copy_id)
        .align(crate::flex::Align::Start);

    let details = crate::flex::row((0..cells.len()).map(|index| {
        crate::flex::column([
            crate::flex::leaf(Vec2::new(0.0, Dp::D12.px())).id(label_cell_id + index),
            crate::flex::leaf(Vec2::ZERO).id(cell_id + index),
        ])
        .grow(1.0)
    }))
    .height(Dp::D44.px())
    .padding(crate::flex::Insets {
        left: 0.0,
        top: Dp::D14.px(),
        right: 0.0,
        bottom: 0.0,
    })
    .id(details_id)
    .align(crate::flex::Align::Start);
    let stage_content = crate::flex::column([
        top_row,
        crate::flex::fill(),
        copy,
        crate::flex::leaf(Vec2::new(0.0, Space::S16.px(&metrics))),
        details,
    ])
    .align(crate::flex::Align::Stretch)
    .padding(crate::flex::Insets::xy(pad_x, Dp::D20.px()));
    let placed = stage_content.solve(stage.translate(stage_shift));
    let rect_of = |id| placed.find(id).unwrap_or(stage);

    painter.text(
        rect_of(number_id).left_top(),
        Align2::LEFT_TOP,
        &number,
        number_font,
        faint,
    );
    if let Some(host) = hosted {
        let fade = timeline.reveal(0.5);
        let lift = Vec2::new(0.0, (1.0 - fade) * Space::S6.px(&metrics));
        let mut faded = painter.clone();
        faded.set_opacity(fade);
        avatar(
            &faded,
            assets,
            rect_of(avatar_id).translate(lift).center(),
            Dp::D14.px(),
            &host.name,
            host.avatar.as_deref(),
            metrics,
        );
        faded.text(
            rect_of(hosted_id).translate(lift).left_center(),
            Align2::LEFT_CENTER,
            &hosted_text,
            hosted_font,
            white(0.95),
        );
    }

    if joined {
        let fade = timeline.reveal(0.45);
        let lift = Vec2::new(0.0, (1.0 - fade) * Space::S8.px(&metrics));
        let mut shown = painter.clone();
        shown.set_opacity(fade);
        if let Some((badge, title)) = &episode {
            let badge_slot = rect_of(badge_id).translate(lift);
            shown.rect_stroke(
                badge_slot,
                5.0,
                crate::gfx::Stroke::new(1.0, white(0.4)),
                crate::gfx::StrokeKind::Inside,
            );
            shown.text(
                rect_of(badge_text_id).translate(lift).left_top(),
                Align2::LEFT_TOP,
                badge,
                badge_font,
                Color32::WHITE,
            );
            if let Some(title) = title {
                crate::flex::paint_text(
                    &shown,
                    rect_of(episode_title_id).translate(lift),
                    title,
                    &episode_title_font,
                    white(0.92),
                    Align2::LEFT_CENTER,
                );
            }
        }
        let main = rect_of(main_id).translate(lift);
        match logo {
            Some((texture, _)) => {
                shown.image(texture, main, full_uv(), Color32::WHITE);
            }
            None => crate::flex::paint_text(
                &shown,
                main,
                &title,
                &title_font,
                Color32::WHITE,
                Align2::LEFT_BOTTOM,
            ),
        }
    } else {
        painter.text(
            rect_of(prompt_id).left_bottom(),
            Align2::LEFT_BOTTOM,
            t("watch.ticket.prompt"),
            prompt_font,
            muted,
        );
        painter.text(
            rect_of(main_id).left_bottom(),
            Align2::LEFT_BOTTOM,
            &idle_title,
            idle_title_font,
            Color32::WHITE,
        );
    }

    let divider = rect_of(details_id).top();
    painter.line_segment(
        [
            Pos2::new(rect_of(details_id).left(), divider),
            Pos2::new(rect_of(details_id).right(), divider),
        ],
        crate::gfx::Stroke::new(stroke::hairline(), white(0.14)),
    );
    for (index, (label, value)) in cells.into_iter().enumerate() {
        let slot = rect_of(cell_id + index);
        painter.text(
            rect_of(label_cell_id + index).left_top(),
            Align2::LEFT_TOP,
            label,
            crate::fonts::bold(9.5),
            white(0.62 * reveal),
        );
        painter.text(
            slot.left_top(),
            Align2::LEFT_TOP,
            if joined { value } else { "—".to_owned() },
            crate::fonts::bold(19.0),
            white(reveal * if joined { 1.0 } else { dim }),
        );
    }

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
    let (label_id, body_id) = (1, 2);
    let stub_content = crate::flex::column([
        crate::flex::leaf(Vec2::new(0.0, Dp::D24.px())).id(label_id),
        crate::flex::fill().id(body_id),
    ])
    .align(crate::flex::Align::Stretch)
    .padding(crate::flex::Insets::xy(pad_x, Dp::D20.px()));
    let stub_placed = stub_content.solve(stub.translate(stub_shift));
    let label_slot = stub_placed.find(label_id).unwrap_or(stub);
    let row_rect = stub_placed.find(body_id).unwrap_or(stub);
    painter.text(
        label_slot.left_top(),
        Align2::LEFT_TOP,
        label,
        crate::fonts::bold(10.0),
        label_color,
    );
    match ticket.room.as_deref() {
        None => {
            let field = crate::flex::row([crate::flex::leaf(Vec2::new(
                row_rect.width().min(metrics.surface_ticket_field_max_width),
                Dp::D56.px(),
            ))
            .id(1)])
            .align(crate::flex::Align::Start)
            .solve(row_rect)
            .find(1)
            .unwrap_or(row_rect);
            let mut child = ui.new_child(crate::gfx::UiBuilder::new().max_rect(field));
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
            let seats = seat_count(ticket.members.len(), ticket.max_members);
            let radius = if narrow { Dp::D14.px() } else { Dp::D16.px() };
            let step = radius * 2.0 - Dp::D4.px();
            let mut line = vec![crate::flex::fill().id(0)];
            line.extend((0..seats).map(|index| {
                let width = if index + 1 == seats {
                    radius * 2.0
                } else {
                    step
                };
                crate::flex::leaf(Vec2::new(width, radius * 2.0)).id(index + 1)
            }));
            let line = crate::flex::row(line)
                .align(crate::flex::Align::Center)
                .solve(row_rect);
            painter.text(
                line.find(0).unwrap_or(row_rect).left_center(),
                Align2::LEFT_CENTER,
                shown,
                crate::fonts::bold(if narrow { 22.0 } else { 26.0 }),
                Color32::WHITE,
            );
            if seats > 0 {
                for index in 0..seats {
                    let fade = timeline.reveal(0.8 + index as f32 * 0.07);
                    let mut cell = painter.clone();
                    cell.set_opacity(fade);
                    let slot = line.find(index + 1).unwrap_or(row_rect);
                    let center = Pos2::new(
                        slot.left() + radius,
                        slot.center().y + (1.0 - fade) * Space::S8.px(&metrics),
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
                                    crate::gfx::Stroke::new(2.0, Color32::WHITE),
                                );
                            }
                        }
                        None => {
                            cell.circle_stroke(
                                center,
                                radius,
                                crate::gfx::Stroke::new(1.2, white(0.28)),
                            );
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
                    label_slot.right_top(),
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
