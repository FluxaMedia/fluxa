use super::*;

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
pub struct WatchTicket {
    pub room: Option<String>,
    pub connecting: bool,
    pub error: Option<String>,
    pub title: Option<String>,
    pub logo: Option<String>,
    pub backdrop: Option<String>,
    pub episode: Option<String>,
    pub people: usize,
}

#[derive(Clone, Debug, Default)]
pub struct WatchTogetherModel {
    pub language: String,
    pub code_input: String,
    pub room_code: Option<String>,
    pub members: Vec<WatchMemberView>,
    pub host: bool,
    pub locked: bool,
    pub connecting: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchTogetherRequest {
    Create,
    Join(String),
    Leave,
    Close,
    CopyCode,
    Kick(String),
    SetLocked(bool),
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
    painter: &egui::Painter,
    assets: &impl HomeAssets,
    name: &str,
    center: Pos2,
    size: f32,
    tint: Color32,
) {
    if let Some(texture) = assets.icon(name) {
        painter.image(
            texture,
            Rect::from_center_size(center, Vec2::splat(size)),
            full_uv(),
            tint,
        );
    }
}

const SLOTS: usize = 6;
const AVATAR: f32 = 30.0;

pub fn draw_watch_together(
    context: &egui::Context,
    viewport: Viewport,
    model: &mut WatchTogetherModel,
    assets: &mut impl HomeAssets,
) -> Option<WatchTogetherRequest> {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let metrics = metrics_for_assets(viewport, assets);
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let width = (viewport.width - 64.0).clamp(300.0, 500.0);
    let close = assets.icon("Close");
    components::modal_width(
        context,
        screen,
        "fluxa-watch-together",
        width,
        |ui| {
            let bar = Vec2::new(ui.available_width(), metrics.icon_button_size);
            ui.allocate_ui_with_layout(
                bar,
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(
                        RichText::new(t("watch.title"))
                            .size(22.0)
                            .strong()
                            .color(text_primary()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let closed = components::icon_button(
                            ui,
                            close,
                            metrics.icon_button_size,
                            ink(tone::BODY),
                            true,
                            false,
                            true,
                        )
                        .clicked();
                        if closed || ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                            request = Some(WatchTogetherRequest::Close);
                        }
                    });
                },
            );
            ui.add_space(18.0);
            match model.room_code.clone() {
                Some(code) => draw_room(ui, model, &code, assets, metrics, &t, &mut request),
                None => draw_entry(ui, model, metrics, &t, &mut request),
            }
            if let Some(error) = model.error.as_ref() {
                components::note(ui, &t(error), metrics);
            }
        },
        metrics,
    );
    request
}

fn draw_entry(
    ui: &mut egui::Ui,
    model: &mut WatchTogetherModel,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let width = ui.available_width();
    if model.connecting {
        components::note(ui, &t("watch.connecting"), metrics);
        return;
    }
    if components::primary_button(ui, &t("watch.start"), width, true).clicked() {
        *request = Some(WatchTogetherRequest::Create);
    }
    ui.add_space(20.0);
    components::note(ui, &t("watch.join_hint"), metrics);
    ui.add_space(8.0);
    if components::code_input(ui, &mut model.code_input, 6, width, metrics).submitted {
        *request = Some(WatchTogetherRequest::Join(model.code_input.clone()));
    }
}

fn draw_room(
    ui: &mut egui::Ui,
    model: &WatchTogetherModel,
    code: &str,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let width = ui.available_width();
    draw_code(ui, code, width, assets, metrics, t, request);
    ui.add_space(22.0);
    let gap = 6.0;
    let slot = (width - gap * (SLOTS as f32 - 1.0)) / SLOTS as f32;
    let (row, _) = ui.allocate_exact_size(Vec2::new(width, AVATAR * 2.0 + 62.0), Sense::hover());
    for index in 0..SLOTS {
        let left = row.left() + index as f32 * (slot + gap);
        let center = Pos2::new(left + slot * 0.5, row.top() + AVATAR + 6.0);
        match model.members.get(index) {
            Some(member) => draw_member(ui, member, center, slot, assets, metrics, t),
            None => draw_empty(ui, assets, center, metrics),
        }
    }
    let ready = model.members.iter().filter(|member| member.ready).count();
    let summary = match waiting_member(&model.members) {
        Some(member) => t("watch.waiting_for").replace("{name}", &member.name),
        None => t("watch.ready_count")
            .replace("{ready}", &ready.to_string())
            .replace("{total}", &model.members.len().to_string()),
    };
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(summary)
                .size(metrics.text.subtitle * 0.8)
                .strong()
                .color(text_primary()),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(t("watch.hint"))
                .size(metrics.text.label)
                .color(metrics.text_secondary),
        );
    });
    ui.add_space(18.0);
    if components::secondary_button(ui, &t("watch.leave"), width).clicked() {
        *request = Some(WatchTogetherRequest::Leave);
    }
}

fn draw_code(
    ui: &mut egui::Ui,
    code: &str,
    width: f32,
    assets: &impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 58.0), Sense::click());
    ui.painter().rect_filled(rect, 14.0, ink(tone::RAISED));
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
    ui: &mut egui::Ui,
    member: &WatchMemberView,
    center: Pos2,
    slot: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
) {
    let painter = ui.painter();
    components::avatar(
        painter,
        assets,
        center,
        AVATAR,
        &member.name,
        member.avatar.as_deref(),
        metrics,
    );
    if member.buffering {
        let time = ui.input(|input| input.time) as f32;
        let start = time * 4.0;
        let points = (0..=18)
            .map(|step| {
                let angle = start + step as f32 / 18.0 * 2.2;
                center + Vec2::angled(angle) * (AVATAR + 5.0)
            })
            .collect::<Vec<_>>();
        painter.add(egui::Shape::line(points, egui::Stroke::new(3.0, accent())));
        ui.ctx().request_repaint();
    } else if member.ready {
        painter.circle_stroke(center, AVATAR + 5.0, egui::Stroke::new(3.0, accent()));
        let badge = center + Vec2::new(AVATAR * 0.72, AVATAR * 0.72);
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
            AVATAR + 5.0,
            egui::Stroke::new(1.5, metrics.text_muted.gamma_multiply(0.5)),
        );
    }
    let limit = ((slot / 7.5) as usize).max(4);
    painter.text(
        Pos2::new(center.x, center.y + AVATAR + 18.0),
        Align2::CENTER_CENTER,
        clipped(&member.name, limit),
        FontId::proportional(metrics.text.meta),
        text_primary(),
    );
    if member.host {
        glyph(
            painter,
            assets,
            "Crown",
            center + Vec2::new(0.0, -AVATAR - 12.0),
            18.0,
            accent(),
        );
    }
    painter.text(
        Pos2::new(center.x, center.y + AVATAR + 34.0),
        Align2::CENTER_CENTER,
        status_text(member, t),
        FontId::proportional(metrics.text.label),
        status_color(member, metrics),
    );
}

fn draw_empty(ui: &mut egui::Ui, assets: &impl HomeAssets, center: Pos2, metrics: UiMetrics) {
    let painter = ui.painter();
    painter.circle_filled(center, AVATAR, ink(tone::RAISED).gamma_multiply(0.45));
    glyph(
        painter,
        assets,
        "Plus",
        center,
        AVATAR * 0.7,
        metrics.text_muted.gamma_multiply(0.6),
    );
}

pub fn draw_watch_side_panel(
    context: &egui::Context,
    rect: Rect,
    model: &WatchTogetherModel,
    assets: &mut impl HomeAssets,
) -> Option<WatchTogetherRequest> {
    let metrics = UiMetrics::for_viewport(Viewport::new(
        rect.width().max(1.0) as u32,
        rect.height().max(1.0) as u32,
        UiFormFactor::Desktop,
    ));
    let language = model.language.clone();
    let t = |key: &str| localized(key, &language);
    let mut request = None;
    let close = assets.icon("Close");
    egui::Area::new(Id::new("fluxa-watch-side"))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .show(context, |ui| {
            ui.set_min_size(rect.size());
            ui.set_max_size(rect.size());
            ui.painter().rect_filled(rect, 0.0, metrics.surface);
            ui.painter().line_segment(
                [rect.left_top(), rect.left_bottom()],
                egui::Stroke::new(stroke::HAIRLINE, metrics.border),
            );
            let inner = rect.shrink2(Vec2::new(18.0, 18.0));
            let mut ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(inner)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            let ui = &mut ui;
            let bar = Vec2::new(ui.available_width(), metrics.icon_button_size);
            ui.allocate_ui_with_layout(
                bar,
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(
                        RichText::new(t("watch.title"))
                            .size(22.0)
                            .strong()
                            .color(text_primary()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let closed = components::icon_button(
                            ui,
                            close,
                            metrics.icon_button_size,
                            ink(tone::BODY),
                            true,
                            false,
                            true,
                        )
                        .clicked();
                        if closed {
                            request = Some(WatchTogetherRequest::Close);
                        }
                    });
                },
            );
            ui.add_space(12.0);
            match model.room_code.as_deref() {
                Some(code) => side_room(ui, model, code, assets, metrics, &t, &mut request),
                None => side_entry(ui, model, metrics, &t, &mut request),
            }
            if let Some(error) = model.error.as_ref() {
                components::note(ui, &t(error), metrics);
            }
        });
    request
}

fn side_entry(
    ui: &mut egui::Ui,
    model: &WatchTogetherModel,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let width = ui.available_width();
    if model.connecting {
        components::note(ui, &t("watch.connecting"), metrics);
        return;
    }
    if components::primary_button(ui, &t("watch.start"), width, true).clicked() {
        *request = Some(WatchTogetherRequest::Create);
    }
    ui.add_space(16.0);
    components::note(ui, &t("watch.join_hint"), metrics);
    ui.add_space(12.0);
    let id = Id::new("fluxa-watch-side-code");
    let mut code = ui.data_mut(|data| data.get_temp::<String>(id).unwrap_or_default());
    if components::code_input(ui, &mut code, 6, width, metrics).submitted {
        *request = Some(WatchTogetherRequest::Join(code.clone()));
    }
    ui.data_mut(|data| data.insert_temp(id, code));
}

fn side_room(
    ui: &mut egui::Ui,
    model: &WatchTogetherModel,
    code: &str,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let width = ui.available_width();
    draw_code(ui, code, width, assets, metrics, t, request);
    ui.add_space(10.0);
    let ready = model.members.iter().filter(|member| member.ready).count();
    let summary = match waiting_member(&model.members) {
        Some(member) => t("watch.waiting_for").replace("{name}", &member.name),
        None if model.members.len() < 2 => t("watch.hint_alone"),
        None => t("watch.ready_count")
            .replace("{ready}", &ready.to_string())
            .replace("{total}", &model.members.len().to_string()),
    };
    ui.label(
        RichText::new(summary)
            .size(metrics.text.meta)
            .color(metrics.text_secondary),
    );
    ui.add_space(12.0);
    if model.host {
        lock_row(ui, model.locked, width, assets, metrics, t, request);
        ui.add_space(6.0);
    }
    for member in &model.members {
        if side_member(ui, member, model.host, width, assets, metrics, t) {
            *request = Some(WatchTogetherRequest::Kick(member.id.clone()));
        }
    }
    let spare = ui.available_height() - button_height() - 4.0;
    ui.add_space(spare.max(14.0));
    if components::secondary_button(ui, &t("watch.leave"), width).clicked() {
        *request = Some(WatchTogetherRequest::Leave);
    }
}

fn lock_row(
    ui: &mut egui::Ui,
    locked: bool,
    width: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
    request: &mut Option<WatchTogetherRequest>,
) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 40.0), Sense::click());
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, 12.0, ink(tone::RAISED));
    }
    glyph(
        painter,
        assets,
        if locked { "Lock" } else { "Unlock" },
        Pos2::new(rect.left() + 20.0, rect.center().y),
        18.0,
        if locked {
            accent()
        } else {
            metrics.text_secondary
        },
    );
    painter.text(
        Pos2::new(rect.left() + 40.0, rect.center().y),
        Align2::LEFT_CENTER,
        if locked {
            t("watch.unlock")
        } else {
            t("watch.lock")
        },
        FontId::proportional(metrics.text.meta),
        text_primary(),
    );
    if response.clicked() {
        *request = Some(WatchTogetherRequest::SetLocked(!locked));
    }
}

fn side_member(
    ui: &mut egui::Ui,
    member: &WatchMemberView,
    can_remove: bool,
    width: f32,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    t: &dyn Fn(&str) -> String,
) -> bool {
    let radius = 20.0;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, radius * 2.0 + 12.0), Sense::click());
    let center = Pos2::new(rect.left() + radius + 3.0, rect.center().y);
    let removable = can_remove && !member.you && !member.host;
    let remove_center = Pos2::new(rect.right() - 14.0, rect.center().y);
    let remove_hover = removable
        && ui
            .input(|input| input.pointer.hover_pos())
            .is_some_and(|pos| pos.distance(remove_center) < 16.0);
    let painter = ui.painter();
    components::avatar(
        painter,
        assets,
        center,
        radius,
        &member.name,
        member.avatar.as_deref(),
        metrics,
    );
    if member.buffering {
        let start = ui.input(|input| input.time) as f32 * 4.0;
        let points = (0..=14)
            .map(|step| center + Vec2::angled(start + step as f32 / 14.0 * 2.2) * (radius + 3.5))
            .collect::<Vec<_>>();
        painter.add(egui::Shape::line(points, egui::Stroke::new(2.5, accent())));
        ui.ctx().request_repaint();
    } else if member.ready {
        painter.circle_stroke(center, radius + 3.5, egui::Stroke::new(2.5, accent()));
    } else {
        painter.circle_stroke(
            center,
            radius + 3.5,
            egui::Stroke::new(1.2, metrics.text_muted.gamma_multiply(0.5)),
        );
    }
    let role = if member.you {
        format!("{} · {}", t("watch.you"), status_text(member, t))
    } else {
        status_text(member, t)
    };
    if member.host {
        glyph(
            painter,
            assets,
            "Crown",
            Pos2::new(rect.right() - 10.0, rect.center().y),
            16.0,
            accent(),
        );
    }
    let text_x = center.x + radius + 14.0;
    painter.text(
        Pos2::new(text_x, rect.center().y - 8.0),
        Align2::LEFT_CENTER,
        clipped(&member.name, ((width - 70.0) / 8.0) as usize),
        FontId::proportional(metrics.text.meta),
        text_primary(),
    );
    painter.text(
        Pos2::new(text_x, rect.center().y + 9.0),
        Align2::LEFT_CENTER,
        role,
        FontId::proportional(metrics.text.label),
        status_color(member, metrics),
    );
    if removable {
        glyph(
            painter,
            assets,
            "Close",
            remove_center,
            16.0,
            if remove_hover {
                text_primary()
            } else {
                metrics.text_muted
            },
        );
    }
    removable && remove_hover && response.clicked()
}
