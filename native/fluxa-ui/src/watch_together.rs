use super::*;
use crate::kit::layout_items as items;
use crate::kit::{lay, widget};
use std::cell::{Cell, RefCell};

#[derive(Clone, Debug, Default)]
pub struct WatchMemberView {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub ready: bool,
    pub buffering: bool,
    pub status: String,
    pub host: bool,
    pub you: bool,
}

#[derive(Clone, Debug, Default)]
pub struct WatchChatView {
    pub id: u64,
    pub by: String,
    pub name: String,
    pub avatar: Option<String>,
    pub text: String,
    pub you: bool,
}

#[derive(Clone, Debug, Default)]
pub struct WatchTicket {
    pub room: Option<String>,
    pub connecting: bool,
    pub error: Option<String>,
    pub title: Option<String>,
    pub logo: Option<String>,
    pub backdrop: Option<String>,
    pub episode: Option<String>,
    pub host: Option<WatchMemberView>,
    pub members: Vec<WatchMemberView>,
    pub max_members: u32,
}

#[derive(Clone, Debug, Default)]
pub struct WatchTogetherModel {
    pub language: String,
    pub code_input: String,
    pub room_code: Option<String>,
    pub members: Vec<WatchMemberView>,
    pub chat: Vec<WatchChatView>,
    pub host: bool,
    pub locked: bool,
    pub share_stream: bool,
    pub max_members: u32,
    pub connecting: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchTogetherRequest {
    Create,
    Join(String),
    Leave,
    End,
    Close,
    CopyCode,
    Kick(String),
    SetLocked(bool),
    SetShareStream(bool),
    Chat(String),
}

fn status_text(member: &WatchMemberView, t: &dyn Fn(&str) -> String) -> String {
    t(&format!("watch.status.{}", member.status))
}

fn status_color(member: &WatchMemberView, metrics: UiMetrics) -> Color32 {
    match member.status.as_str() {
        "buffering" | "picking" => accent(),
        "watching" => text_primary(),
        _ => metrics.text_muted,
    }
}

pub fn waiting_member(members: &[WatchMemberView]) -> Option<&WatchMemberView> {
    members
        .iter()
        .find(|member| member.buffering && !member.you)
}

fn glyph(
    painter: &crate::gfx::Painter,
    assets: &impl HomeAssets,
    name: &str,
    center: Pos2,
    size: f32,
    tint: Color32,
) {
    if let Some(texture) = assets.icon(name) {
        painter.image(
            texture,
            crate::gfx::square_around(center, size),
            full_uv(),
            tint,
        );
    }
}

type Sent = Cell<Option<WatchTogetherRequest>>;

fn emit(sent: &Sent, add: impl FnOnce(&mut Option<WatchTogetherRequest>)) {
    let mut local = None;
    add(&mut local);
    if local.is_some() {
        sent.set(local);
    }
}

fn gap_item<'a>(px: Space, metrics: UiMetrics) -> crate::kit::Item<'a> {
    items::gap(px.px(&metrics))
}

const SLOTS: usize = 6;
const EMPTY_SLOT: usize = 1000;
const NAME_LIMIT: usize = 10;

pub fn draw_watch_together(
    context: &crate::gfx::Context,
    viewport: Viewport,
    model: &mut WatchTogetherModel,
    assets: &mut impl HomeAssets,
) -> Option<WatchTogetherRequest> {
    let metrics = metrics_for_assets(viewport, assets);
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let close = assets.icon("Close");
    let error = model.error.clone();
    let room_code = model.room_code.clone();
    crate::kit::dialog_with(context, "fluxa-watch-together", &["wider"], |ui| {
        let closed = Cell::new(false);
        let sent = Cell::new(None);
        let bar = widget(ui, |ui| {
            if title_bar(ui, &t("watch.title"), close, metrics) {
                closed.set(true);
            }
        });
        let body = widget(ui, |ui| {
            emit(&sent, |request| match room_code.as_deref() {
                Some(code) => draw_room(ui, model, code, assets, metrics, &t, request),
                None => draw_entry(ui, model, metrics, &t, request),
            })
        });
        let mut children = vec![bar, gap_item(Space::S16, metrics), body];
        if let Some(error) = error.as_ref() {
            children.push(widget(ui, |ui| components::note(ui, &t(error), metrics)));
        }
        lay(ui, children);
        if closed.get() || ui.input(|input| input.key_pressed(crate::gfx::Key::Escape)) {
            request = Some(WatchTogetherRequest::Close);
        }
        if let Some(sent) = sent.take() {
            request = Some(sent);
        }
    });
    request
}

fn title_bar(
    ui: &mut crate::gfx::Ui,
    title: &str,
    close: Option<crate::gfx::TextureId>,
    metrics: UiMetrics,
) -> bool {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), metrics.icon_button_size),
        Sense::hover(),
    );
    let placed = crate::flex::row([
        crate::flex::fill().id(0).min_width(0.0),
        crate::flex::slot(metrics.icon_button_size).id(1),
    ])
    .align(crate::flex::Align::Center)
    .solve(rect);
    let title_rect = placed.find(0).unwrap_or(rect);
    let close_rect = placed.find(1).unwrap_or(rect);
    ui.put(
        title_rect,
        crate::gfx::Label::new(
            RichText::new(title)
                .size(22.0)
                .strong()
                .color(text_primary()),
        ),
    );
    let mut child = ui.new_child(
        crate::gfx::UiBuilder::new()
            .max_rect(close_rect)
            .layout(crate::gfx::Layout::top_down(crate::gfx::Align::Center)),
    );
    components::icon_button(
        &mut child,
        close,
        metrics.icon_button_size,
        crate::stylesheet::color("text.body"),
        true,
        false,
        true,
    )
    .clicked()
}

fn draw_entry(
    ui: &mut crate::gfx::Ui,
    model: &mut WatchTogetherModel,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let width = ui.available_width();
    if model.connecting {
        let note = widget(ui, |ui| {
            components::note(ui, &t("watch.connecting"), metrics)
        });
        lay(ui, vec![note]);
        return;
    }
    let sent = Cell::new(None);
    let start = widget(ui, |ui| {
        if components::primary_button(ui, &t("watch.start"), width, true).clicked() {
            sent.set(Some(WatchTogetherRequest::Create));
        }
    });
    let hint = widget(ui, |ui| {
        components::note(ui, &t("watch.join_hint"), metrics)
    });
    let input = widget(ui, |ui| {
        if components::code_input(ui, &mut model.code_input, 6, width, metrics).submitted {
            sent.set(Some(WatchTogetherRequest::Join(model.code_input.clone())));
        }
    });
    lay(
        ui,
        vec![
            start,
            gap_item(Space::S20, metrics),
            hint,
            gap_item(Space::S8, metrics),
            input,
        ],
    );
    if let Some(sent) = sent.take() {
        *request = Some(sent);
    }
}

fn draw_room(
    ui: &mut crate::gfx::Ui,
    model: &WatchTogetherModel,
    code: &str,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let width = ui.available_width();
    let sent = Cell::new(None);
    let assets = RefCell::new(assets);
    let (label, action) = if model.host {
        ("watch.end", WatchTogetherRequest::End)
    } else {
        ("watch.leave", WatchTogetherRequest::Leave)
    };
    let ready = model.members.iter().filter(|member| member.ready).count();
    let summary = match waiting_member(&model.members) {
        Some(member) => t("watch.waiting_for").replace("{name}", &member.name),
        None => t("watch.ready_count")
            .replace("{ready}", &ready.to_string())
            .replace("{total}", &model.members.len().to_string()),
    };
    let mut scene = crate::kit::Scene::default();
    let painter = ui.painter().clone();
    let name_font = FontId::proportional(metrics.text.meta);
    let status_font = FontId::proportional(metrics.text.label);
    let labels_width = model
        .members
        .iter()
        .flat_map(|member| {
            [
                crate::flex::text_size(&painter, &clipped(&member.name, NAME_LIMIT), &name_font).x,
                crate::flex::text_size(&painter, &status_text(member, t), &status_font).x,
            ]
        })
        .fold(0.0, f32::max);
    let cell_width = labels_width.max(
        (metrics.member_avatar_radius + metrics.member_ring_gap) * 2.0 + Space::S6.px(&metrics),
    );
    let mut avatars = Vec::new();
    let cells: Vec<crate::flex::Node> = (0..SLOTS)
        .map(|index| {
            let (avatar, labels) = match model.members.get(index) {
                Some(member) => {
                    let avatar = scene.add(
                        &painter,
                        crate::kit::Part::avatar(
                            metrics.member_avatar_radius,
                            &member.name,
                            member.avatar.as_deref(),
                        ),
                    );
                    let name = scene.add(
                        &painter,
                        crate::kit::Part::text(
                            clipped(&member.name, NAME_LIMIT),
                            name_font.clone(),
                            text_primary(),
                        ),
                    );
                    let status = scene.add(
                        &painter,
                        crate::kit::Part::text(
                            status_text(member, t),
                            status_font.clone(),
                            status_color(member, metrics),
                        ),
                    );
                    (avatar, vec![name, status])
                }
                None => (
                    crate::flex::leaf(Vec2::splat(metrics.member_avatar_radius * 2.0))
                        .id(EMPTY_SLOT + index),
                    Vec::new(),
                ),
            };
            avatars.push(avatar.id_of());
            let ringed = crate::flex::stack(Align2::CENTER_CENTER, [avatar])
                .padding(crate::flex::Insets::all(metrics.member_ring_gap));
            crate::flex::column(std::iter::once(ringed).chain(labels))
                .align(crate::flex::Align::Center)
                .gap(Space::S4.px(&metrics))
                .width(cell_width)
        })
        .collect();
    let grid = crate::flex::row(cells)
        .wrap()
        .justify(crate::flex::Justify::Center)
        .gap(Space::S6.px(&metrics))
        .line_gap(Space::S12.px(&metrics))
        .padding(crate::flex::Insets {
            top: Space::S16.px(&metrics),
            ..Default::default()
        })
        .align(crate::flex::Align::Start);
    let measured = grid.solve(Rect::from_min_max(Pos2::ZERO, Pos2::new(width, 1.0e4)));
    let bottom = measured
        .tagged()
        .iter()
        .map(|(_, rect)| rect.bottom())
        .fold(0.0, f32::max);
    let height = bottom + Space::S12.px(&metrics);
    let seats = crate::kit::item(Vec2::new(width, height), |ui, rect| {
        let placed = grid.solve(rect);
        scene.paint(&placed, ui.painter(), &mut **assets.borrow_mut(), metrics);
        for (index, id) in avatars.into_iter().enumerate() {
            let Some(center) = id.and_then(|id| placed.find(id)).map(|rect| rect.center()) else {
                continue;
            };
            match model.members.get(index) {
                Some(member) => {
                    draw_member(ui, member, center, &mut **assets.borrow_mut(), metrics)
                }
                None => draw_empty(ui, &**assets.borrow(), center, metrics),
            }
        }
    });
    let headline = widget(ui, |ui| {
        ui.label(
            RichText::new(summary.clone())
                .size(metrics.text.subtitle * 0.8)
                .strong()
                .color(text_primary()),
        );
    });
    let hint = widget(ui, |ui| {
        ui.label(
            RichText::new(t("watch.hint"))
                .size(metrics.text.label)
                .color(metrics.text_secondary),
        );
    });
    let summary_block = items::column(vec![headline, gap_item(Space::S4, metrics), hint])
        .align(crate::flex::Align::Center)
        .gap(ui.item_spacing().y);
    let leave = widget(ui, |ui| {
        if components::secondary_button(ui, &t(label), width).clicked() {
            sent.set(Some(action.clone()));
        }
    });
    let code_row = widget(ui, |ui| {
        emit(&sent, |request| {
            draw_code(ui, code, width, &**assets.borrow(), metrics, t, request)
        })
    });
    lay(
        ui,
        vec![
            code_row,
            gap_item(Space::S24, metrics),
            seats,
            summary_block,
            gap_item(Space::S16, metrics),
            leave,
        ],
    );
    if let Some(sent) = sent.take() {
        *request = Some(sent);
    }
}

fn draw_code(
    ui: &mut crate::gfx::Ui,
    code: &str,
    width: f32,
    assets: &impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 58.0), Sense::click());
    ui.painter()
        .rect_filled(rect, 14.0, crate::stylesheet::color("raised"));
    let spaced = code.chars().map(String::from).collect::<Vec<_>>().join(" ");
    ui.painter().text(
        Pos2::new(rect.left() + 20.0, rect.center().y),
        Align2::LEFT_CENTER,
        spaced,
        FontId::proportional(metrics.text.subtitle * 1.15),
        text_primary(),
    );
    glyph(
        ui.painter(),
        assets,
        "Copy",
        Pos2::new(rect.right() - 28.0, rect.center().y),
        20.0,
        metrics.text_secondary,
    );
    if response.on_hover_text(t("watch.copy_code")).clicked() {
        *request = Some(WatchTogetherRequest::CopyCode);
    }
}

fn clipped(name: &str, limit: usize) -> String {
    if name.chars().count() <= limit {
        return name.to_owned();
    }
    let head = name.chars().take(limit - 1).collect::<String>();
    format!("{head}…")
}

fn draw_member(
    ui: &mut crate::gfx::Ui,
    member: &WatchMemberView,
    center: Pos2,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
) {
    let painter = ui.painter();
    if member.buffering {
        let time = ui.input(|input| input.time()) as f32;
        let start = time * 4.0;
        let points = (0..=18)
            .map(|step| {
                let angle = start + step as f32 / 18.0 * 2.2;
                center
                    + Vec2::angled(angle) * (metrics.member_avatar_radius + metrics.member_ring_gap)
            })
            .collect::<Vec<_>>();
        painter.add(crate::gfx::Shape::line(
            points,
            crate::gfx::Stroke::new(3.0, accent()),
        ));
        ui.ctx().request_repaint();
    } else if member.ready {
        painter.circle_stroke(
            center,
            metrics.member_avatar_radius + metrics.member_ring_gap,
            crate::gfx::Stroke::new(3.0, accent()),
        );
        let badge = center
            + Vec2::new(
                metrics.member_avatar_radius * 0.72,
                metrics.member_avatar_radius * 0.72,
            );
        painter.circle_filled(badge, 9.0, accent());
        let foreground = if crate::settings::accent_needs_dark_foreground(accent()) {
            Color32::BLACK
        } else {
            Color32::WHITE
        };
        glyph(painter, assets, "Check", badge, 12.0, foreground);
    } else {
        painter.circle_stroke(
            center,
            metrics.member_avatar_radius + metrics.member_ring_gap,
            crate::gfx::Stroke::new(1.5, metrics.text_muted.gamma_multiply(0.5)),
        );
    }
    if member.host {
        glyph(
            painter,
            assets,
            "Crown",
            center + Vec2::new(0.0, -metrics.member_avatar_radius - 12.0),
            18.0,
            accent(),
        );
    }
}

fn draw_empty(ui: &mut crate::gfx::Ui, assets: &impl HomeAssets, center: Pos2, metrics: UiMetrics) {
    let painter = ui.painter();
    painter.circle_filled(
        center,
        metrics.member_avatar_radius,
        crate::stylesheet::color("raised").gamma_multiply(0.45),
    );
    glyph(
        painter,
        assets,
        "Plus",
        center,
        metrics.member_avatar_radius * 0.7,
        metrics.text_muted.gamma_multiply(0.6),
    );
}

pub fn draw_watch_side_panel(
    context: &crate::gfx::Context,
    rect: Rect,
    form_factor: UiFormFactor,
    model: &WatchTogetherModel,
    assets: &mut impl HomeAssets,
) -> Option<WatchTogetherRequest> {
    let metrics = UiMetrics::for_viewport(Viewport::new(
        rect.width().max(1.0) as u32,
        rect.height().max(1.0) as u32,
        form_factor,
    ));
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let close = assets.icon("Close");
    crate::gfx::Area::new(Id::new("fluxa-watch-side"))
        .order(crate::gfx::Order::Foreground)
        .fixed_pos(rect.min)
        .show(context, |ui| {
            ui.set_min_size(rect.size());
            ui.set_max_size(rect.size());
            ui.painter().rect_filled(rect, 0.0, metrics.surface);
            ui.painter().line_segment(
                [rect.left_top(), rect.left_bottom()],
                crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
            );
            let inner = rect.shrink2(Vec2::new(18.0, 18.0));
            let mut ui = ui.new_child(
                crate::gfx::UiBuilder::new()
                    .max_rect(inner)
                    .layout(crate::gfx::Layout::top_down(crate::gfx::Align::Min)),
            );
            let ui = &mut ui;
            let closed = Cell::new(false);
            let sent = Cell::new(None);
            let bar = widget(ui, |ui| {
                if title_bar(ui, &t("watch.title"), close, metrics) {
                    closed.set(true);
                }
            });
            let body = widget(ui, |ui| {
                emit(&sent, |request| match model.room_code.as_deref() {
                    Some(code) => side_room(ui, model, code, assets, metrics, &t, request),
                    None => side_entry(ui, model, metrics, &t, request),
                })
            });
            let mut children = vec![bar, gap_item(Space::S12, metrics), body];
            if let Some(error) = model.error.as_ref() {
                children.push(widget(ui, |ui| components::note(ui, &t(error), metrics)));
            }
            lay(ui, children);
            if closed.get() {
                request = Some(WatchTogetherRequest::Close);
            }
            if let Some(sent) = sent.take() {
                request = Some(sent);
            }
        });
    request
}

fn side_entry(
    ui: &mut crate::gfx::Ui,
    model: &WatchTogetherModel,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let width = ui.available_width();
    if model.connecting {
        let note = widget(ui, |ui| {
            components::note(ui, &t("watch.connecting"), metrics)
        });
        lay(ui, vec![note]);
        return;
    }
    let sent = Cell::new(None);
    let id = Id::new("fluxa-watch-side-code");
    let mut code = ui.data_mut(|data| data.get_temp::<String>(id).unwrap_or_default());
    let start = widget(ui, |ui| {
        if components::primary_button(ui, &t("watch.start"), width, true).clicked() {
            sent.set(Some(WatchTogetherRequest::Create));
        }
    });
    let hint = widget(ui, |ui| {
        components::note(ui, &t("watch.join_hint"), metrics)
    });
    let input = widget(ui, |ui| {
        if components::code_input(ui, &mut code, 6, width, metrics).submitted {
            sent.set(Some(WatchTogetherRequest::Join(code.clone())));
        }
    });
    lay(
        ui,
        vec![
            start,
            gap_item(Space::S16, metrics),
            hint,
            gap_item(Space::S12, metrics),
            input,
        ],
    );
    ui.data_mut(|data| data.insert_temp(id, code));
    if let Some(sent) = sent.take() {
        *request = Some(sent);
    }
}

fn side_room(
    ui: &mut crate::gfx::Ui,
    model: &WatchTogetherModel,
    code: &str,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let width = ui.available_width();
    let sent = Cell::new(None);
    let assets = RefCell::new(assets);
    let (label, action) = if model.host {
        ("watch.end", WatchTogetherRequest::End)
    } else {
        ("watch.leave", WatchTogetherRequest::Leave)
    };
    let ready = model.members.iter().filter(|member| member.ready).count();
    let summary = match waiting_member(&model.members) {
        Some(member) => t("watch.waiting_for").replace("{name}", &member.name),
        None if model.members.len() < 2 => t("watch.hint_alone"),
        None => t("watch.ready_count")
            .replace("{ready}", &ready.to_string())
            .replace("{total}", &model.members.len().to_string()),
    };
    let seats = widget(ui, |ui| {
        side_seats(ui, model, width, &mut **assets.borrow_mut(), metrics, t)
    });
    let summary = widget(ui, |ui| {
        ui.label(
            RichText::new(summary.clone())
                .size(metrics.text.meta)
                .color(metrics.text_secondary),
        );
    });
    let code_row = widget(ui, |ui| {
        emit(&sent, |request| {
            draw_code(ui, code, width, &**assets.borrow(), metrics, t, request)
        })
    });
    let mut upper = vec![
        code_row,
        gap_item(Space::S12, metrics),
        seats,
        gap_item(Space::S8, metrics),
        summary,
        gap_item(Space::S12, metrics),
    ];
    if model.host {
        let (title, description) = if model.locked {
            ("watch.host_only", "watch.host_only_desc")
        } else {
            ("watch.host_only", "watch.guest_controls_desc")
        };
        upper.push(widget(ui, |ui| {
            if toggle_row(
                ui,
                "fluxa-watch-host-only",
                (title, description),
                model.locked,
                width,
                metrics,
                t,
            ) {
                sent.set(Some(WatchTogetherRequest::SetLocked(!model.locked)));
            }
        }));
        upper.push(gap_item(Space::S6, metrics));
        let (title, description) = if model.share_stream {
            ("watch.share_stream", "watch.share_stream_desc")
        } else {
            ("watch.share_stream", "watch.own_stream_desc")
        };
        upper.push(widget(ui, |ui| {
            if toggle_row(
                ui,
                "fluxa-watch-share-stream",
                (title, description),
                model.share_stream,
                width,
                metrics,
                t,
            ) {
                sent.set(Some(WatchTogetherRequest::SetShareStream(
                    !model.share_stream,
                )));
            }
        }));
        upper.push(gap_item(Space::S6, metrics));
    }
    for member in &model.members {
        upper.push(widget(ui, |ui| {
            if side_member(
                ui,
                member,
                model.host,
                width,
                &mut **assets.borrow_mut(),
                metrics,
                t,
            ) {
                sent.set(Some(WatchTogetherRequest::Kick(member.id.clone())));
            }
        }));
    }
    lay(ui, upper);
    let height = ui.available_height() - 2.0 * button_height() - 140.0;
    let chat = widget(ui, |ui| {
        emit(&sent, |request| {
            side_chat(
                ui,
                model,
                width,
                height.max(80.0),
                &mut **assets.borrow_mut(),
                metrics,
                t,
                request,
            )
        })
    });
    let leave = widget(ui, |ui| {
        if components::secondary_button(ui, &t(label), width).clicked() {
            sent.set(Some(action.clone()));
        }
    });
    lay(
        ui,
        vec![
            gap_item(Space::S14, metrics),
            chat,
            gap_item(Space::S14, metrics),
            leave,
        ],
    );
    if let Some(sent) = sent.take() {
        *request = Some(sent);
    }
}

fn side_chat(
    ui: &mut crate::gfx::Ui,
    model: &WatchTogetherModel,
    width: f32,
    height: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let sent = Cell::new(None);
    let id = Id::new("fluxa-watch-chat-draft");
    let mut draft = ui.data_mut(|data| data.get_temp::<String>(id).unwrap_or_default());
    let assets = RefCell::new(assets);
    let title = widget(ui, |ui| {
        ui.label(
            RichText::new(t("watch.chat.title"))
                .size(metrics.text.meta)
                .strong()
                .color(metrics.text_secondary),
        );
    });
    let history = widget(ui, |ui| {
        crate::components::pan_area(
            ui,
            "fluxa-watch-chat",
            |area| {
                area.max_height(height)
                    .min_scrolled_height(height)
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
            },
            |ui| {
                ui.set_width(width);
                if model.chat.is_empty() {
                    ui.label(
                        RichText::new(t("watch.chat.empty"))
                            .size(metrics.text.meta)
                            .color(metrics.text_muted),
                    );
                }
                let mut previous: Option<&str> = None;
                for message in &model.chat {
                    let first = previous != Some(message.by.as_str());
                    previous = Some(message.by.as_str());
                    chat_message(
                        ui,
                        message,
                        first,
                        width,
                        &mut **assets.borrow_mut(),
                        metrics,
                        t,
                    );
                }
            },
        );
    });
    let (assets_ref, sent_ref) = (&assets, &sent);
    let reactions = items::row(
        ["😂", "😮", "😍", "😭", "🔥", "👍"]
            .into_iter()
            .map(|emoji| {
                crate::kit::item(Vec2::splat(34.0), move |ui, rect| {
                    let galley = ui.painter().layout_job(crate::emoji::job(
                        emoji,
                        FontId::proportional(20.0),
                        text_primary(),
                        f32::INFINITY,
                    ));
                    let response =
                        ui.interact(rect, Id::new(("watch-reaction", emoji)), Sense::click());
                    if response.hovered() {
                        ui.painter()
                            .rect_filled(rect, 17.0, crate::stylesheet::color("raised"));
                    }
                    let pos = rect.center() - galley.size() * 0.5;
                    crate::emoji::paint(ui.painter(), pos, galley, &mut **assets_ref.borrow_mut());
                    if response.clicked() {
                        sent_ref.set(Some(WatchTogetherRequest::Chat(emoji.to_owned())));
                    }
                })
                .shrink(0.0)
            })
            .collect(),
    )
    .align(crate::flex::Align::Start)
    .gap(ui.item_spacing().x);
    let input = widget(ui, |ui| {
        if components::message_input(
            ui,
            &mut draft,
            &t("watch.chat.hint"),
            width,
            &mut **assets.borrow_mut(),
            metrics,
        )
        .submitted
        {
            sent.set(Some(WatchTogetherRequest::Chat(std::mem::take(&mut draft))));
        }
    });
    lay(
        ui,
        vec![
            title,
            gap_item(Space::S6, metrics),
            history,
            gap_item(Space::S8, metrics),
            reactions,
            gap_item(Space::S6, metrics),
            input,
        ],
    );
    ui.data_mut(|data| data.insert_temp(id, draft));
    if let Some(sent) = sent.take() {
        *request = Some(sent);
    }
}

fn chat_message(
    ui: &mut crate::gfx::Ui,
    message: &WatchChatView,
    first: bool,
    width: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
) {
    let size = 30.0;
    let gap = 10.0;
    let galley = ui.painter().layout_job(crate::emoji::job(
        &message.text,
        FontId::proportional(metrics.text.body),
        text_primary(),
        width - size - gap,
    ));
    if first {
        let name = if message.you {
            t("watch.you")
        } else {
            message.name.clone()
        };
        let label = ui.painter().layout_no_wrap(
            name,
            FontId::proportional(metrics.text.label),
            if message.you {
                accent()
            } else {
                metrics.text_secondary
            },
        );
        let height = (label.size().y + 2.0 + galley.size().y).max(size);
        let block = crate::kit::item(Vec2::new(width, height), |ui, rect| {
            components::avatar(
                ui.painter(),
                assets,
                Pos2::new(rect.left() + size * 0.5, rect.top() + size * 0.5),
                size * 0.5,
                &message.name,
                message.avatar.as_deref(),
                metrics,
            );
            let left = rect.left() + size + gap;
            let label_height = label.size().y;
            ui.painter()
                .galley(Pos2::new(left, rect.top()), label, text_primary());
            crate::emoji::paint(
                ui.painter(),
                Pos2::new(left, rect.top() + label_height + 2.0),
                galley,
                assets,
            );
        });
        lay(ui, vec![gap_item(Space::S6, metrics), block]);
    } else {
        let block = crate::kit::item(Vec2::new(width, galley.size().y + 2.0), |ui, rect| {
            crate::emoji::paint(
                ui.painter(),
                Pos2::new(rect.left() + size + gap, rect.top() + 2.0),
                galley,
                assets,
            );
        });
        lay(ui, vec![block]);
    }
}

fn side_seats(
    ui: &mut crate::gfx::Ui,
    model: &WatchTogetherModel,
    width: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
) {
    let seats = components::seat_count(model.members.len(), model.max_members);
    if seats == 0 {
        return;
    }
    let total = model.max_members.max(model.members.len() as u32);
    let radius = 10.0;
    let (row, _) = ui.allocate_exact_size(Vec2::new(width, radius * 2.0 + 6.0), Sense::hover());
    let painter = ui.painter().clone();
    painter.text(
        Pos2::new(row.left() + 2.0, row.center().y),
        Align2::LEFT_CENTER,
        t("watch.seats")
            .replace("{taken}", &model.members.len().to_string())
            .replace("{total}", &total.to_string()),
        FontId::proportional(metrics.text.meta),
        text_primary(),
    );
    let span = components::seat_row_width(seats, radius, metrics);
    components::seat_row(
        &painter,
        assets,
        &model.members,
        seats,
        Pos2::new(row.right() - span - 3.0, row.center().y),
        radius,
        metrics,
    );
}

fn toggle_row(
    ui: &mut crate::gfx::Ui,
    id: &str,
    labels: (&str, &str),
    on: bool,
    width: f32,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 66.0), Sense::click());
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, 12.0, crate::stylesheet::color("raised"));
    }
    painter.text(
        Pos2::new(rect.left() + 12.0, rect.top() + 22.0),
        Align2::LEFT_CENTER,
        t(labels.0),
        FontId::proportional(metrics.text.meta),
        text_primary(),
    );
    let galley = painter.layout(
        t(labels.1),
        FontId::proportional(metrics.text.label),
        metrics.text_muted,
        width - 24.0 - metrics.settings_toggle_width - 16.0,
    );
    painter.galley(
        Pos2::new(rect.left() + 12.0, rect.top() + 38.0),
        galley,
        metrics.text_muted,
    );
    let toggle_size = Vec2::new(metrics.settings_toggle_width, metrics.settings_toggle_height);
    let track = crate::flex::anchored(rect, Align2::RIGHT_TOP, toggle_size, 0.0)
        .translate(Vec2::new(-8.0, 22.0 - toggle_size.y * 0.5));
    components::toggle(ui.ctx(), painter, Id::new(id), track, on, metrics);
    response.clicked()
}

fn side_member(
    ui: &mut crate::gfx::Ui,
    member: &WatchMemberView,
    can_remove: bool,
    width: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
) -> bool {
    let radius = metrics.member_row_avatar_radius;
    let ring = radius + metrics.member_row_ring_gap;
    let icon = metrics.member_row_icon;
    let removable = can_remove && !member.you && !member.host;
    let role = if member.you {
        format!("{} · {}", t("watch.you"), status_text(member, t))
    } else {
        status_text(member, t)
    };
    let painter = ui.painter().clone();
    let mut scene = crate::kit::Scene::default();
    let avatar = scene.add(
        &painter,
        crate::kit::Part::avatar(radius, &member.name, member.avatar.as_deref()),
    );
    let name = scene.add(
        &painter,
        crate::kit::Part::text(
            clipped(
                &member.name,
                ((width - ring * 2.0) / metrics.text.meta * 1.8) as usize,
            ),
            FontId::proportional(metrics.text.meta),
            text_primary(),
        ),
    );
    let status = scene.add(
        &painter,
        crate::kit::Part::text(
            role,
            FontId::proportional(metrics.text.label),
            status_color(member, metrics),
        ),
    );
    let crown = member
        .host
        .then(|| scene.add(&painter, crate::kit::Part::icon("Crown", icon, accent())));
    let remove_id = 900;
    let hover = ui.input(|input| input.pointer().hover_pos());
    let mut children = vec![
        crate::flex::stack(Align2::CENTER_CENTER, [avatar.id(1)])
            .padding(crate::flex::Insets::all(metrics.member_row_ring_gap)),
        crate::flex::column([name, status])
            .align(crate::flex::Align::Start)
            .gap(Space::S2.px(&metrics))
            .grow(1.0),
    ];
    children.extend(crown);
    if removable {
        children.push(crate::flex::leaf(Vec2::splat(icon)).id(remove_id));
    }
    let row = crate::flex::row(children)
        .align(crate::flex::Align::Center)
        .gap(Space::S12.px(&metrics))
        .padding(crate::flex::Insets::all(Space::S4.px(&metrics)));
    let height = ring * 2.0 + Space::S8.px(&metrics);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let placed = row.solve(rect);
    scene.paint(&placed, &painter, assets, metrics);
    let center = placed.find(1).map_or(rect.center(), |rect| rect.center());
    if member.buffering {
        let start = ui.input(|input| input.time()) as f32 * 4.0;
        let points = (0..=14)
            .map(|step| center + Vec2::angled(start + step as f32 / 14.0 * 2.2) * ring)
            .collect::<Vec<_>>();
        painter.add(crate::gfx::Shape::line(
            points,
            crate::gfx::Stroke::new(2.5, accent()),
        ));
        ui.ctx().request_repaint();
    } else if member.ready {
        painter.circle_stroke(center, ring, crate::gfx::Stroke::new(2.5, accent()));
    } else {
        painter.circle_stroke(
            center,
            ring,
            crate::gfx::Stroke::new(1.2, metrics.text_muted.gamma_multiply(0.5)),
        );
    }
    let remove_rect = placed.find(remove_id);
    let remove_hover = removable
        && remove_rect
            .zip(hover)
            .is_some_and(|(rect, pos)| rect.expand(Space::S4.px(&metrics)).contains(pos));
    if let Some(rect) = remove_rect {
        glyph(
            &painter,
            assets,
            "Close",
            rect.center(),
            icon,
            if remove_hover {
                text_primary()
            } else {
                metrics.text_muted
            },
        );
    }
    removable && remove_hover && response.clicked()
}
