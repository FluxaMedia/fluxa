use super::*;

const DROPDOWN_CHROME: f32 = 44.0;

pub(crate) fn dropdown_width_for_label(
    ui: &Ui,
    label: &str,
    metrics: UiMetrics,
    min_width: f32,
    max_width: f32,
) -> f32 {
    let font = FontId::proportional(metrics.nav_label_size + 1.0);
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::WHITE)
        .size()
        .x;
    (text_width + DROPDOWN_CHROME + 2.0).clamp(min_width, max_width.max(min_width))
}

fn dropdown_popup_width(
    ui: &Ui,
    options: &[(String, String)],
    minimum: f32,
    metrics: UiMetrics,
) -> f32 {
    let font = FontId::proportional(metrics.nav_label_size + 1.0);
    let widest_label = options
        .iter()
        .map(|(_, label)| {
            ui.painter()
                .layout_no_wrap(label.clone(), font.clone(), Color32::WHITE)
                .size()
                .x
        })
        .fold(0.0_f32, f32::max);
    (widest_label + 64.0)
        .max(minimum)
        .min(600.0)
}

pub(crate) fn sheet_choice(
    node: u64,
    key: &'static str,
    title: String,
    options: &[(String, String)],
    selected: &str,
) -> Option<(u64, crate::ChoiceRequest)> {
    (!options.is_empty()).then(|| {
        (
            node,
            crate::ChoiceRequest {
                key,
                title,
                options: options.to_vec(),
                selected: selected.to_owned(),
            },
        )
    })
}

pub(crate) fn dropdown(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    selected: &str,
    options: &[(String, String)],
    width: f32,
    sheet: bool,
    metrics: UiMetrics,
) -> (Response, Option<String>) {
    let selected_label = options
        .iter()
        .find(|(option, _)| option == selected)
        .map(|(_, label)| label.as_str())
        .unwrap_or(selected);
    let popup_width = dropdown_popup_width(ui, options, width, metrics);
    dropdown_inner(
        ui,
        id,
        selected,
        selected_label,
        options,
        width,
        popup_width,
        !options.is_empty(),
        sheet,
        metrics,
    )
}

pub(crate) fn dropdown_placeholder(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    placeholder: &str,
    width: f32,
    metrics: UiMetrics,
) -> Response {
    dropdown_inner(
        ui,
        id,
        "",
        placeholder,
        &[],
        width,
        width,
        false,
        false,
        metrics,
    )
    .0
}

fn dropdown_inner(
    ui: &mut Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    selected: &str,
    selected_label: &str,
    options: &[(String, String)],
    width: f32,
    popup_width: f32,
    enabled: bool,
    sheet: bool,
    metrics: UiMetrics,
) -> (Response, Option<String>) {
    let height = metrics.screen_control_height.max(38.0);
    let mut value = selected.to_owned();
    let response = choice_field(
        ui,
        Id::new(("fluxa-shared-dropdown", id)),
        Vec2::new(width.max(1.0), height),
        metrics.nav_label_size + 1.0,
        selected_label,
        options,
        &mut value,
        popup_width,
        enabled,
        sheet,
        metrics,
    );
    let changed = value != selected;
    (response, changed.then_some(value))
}

fn paint_chevron(painter: &Painter, center: Pos2, open: f32, color: Color32) {
    let flip = 1.0 - 2.0 * open;
    let stroke = egui::Stroke::new(stroke::icon(), color);
    let tip = center + Vec2::new(0.0, 2.5 * flip);
    painter.line_segment([center + Vec2::new(-4.5, -2.0 * flip), tip], stroke);
    painter.line_segment([tip, center + Vec2::new(4.5, -2.0 * flip)], stroke);
}

pub(crate) fn choice_field(
    ui: &mut Ui,
    id: Id,
    size: Vec2,
    text_size: f32,
    current: &str,
    choices: &[(String, String)],
    selected: &mut String,
    popup_width: f32,
    enabled: bool,
    sheet: bool,
    metrics: UiMetrics,
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let popup_id = id.with("popup");
    let open = enabled && egui::Popup::is_id_open(ui.ctx(), popup_id);
    let turn = crate::motion::toggle(ui.ctx(), id.with("chevron"), open, crate::motion::fast());
    let radius = (rect.height() * 0.5).min(12.0);
    let fill = if !enabled {
        metrics.ink(tone::faint())
    } else if open {
        metrics.ink(tone::soft())
    } else if response.hovered() {
        metrics.ink(tone::soft())
    } else {
        metrics.ink(tone::faint())
    };
    let painter = ui.painter();
    painter.rect(
        rect,
        radius,
        fill,
        egui::Stroke::new(stroke::hairline(), metrics.ink(if open { 70 } else { 26 })),
        egui::StrokeKind::Inside,
    );
    let font = FontId::proportional(text_size);
    let text_color = if enabled {
        metrics.text_primary
    } else {
        metrics.ink(tone::muted())
    };
    draw_text(
        painter,
        rect.left_center() + Vec2::new(14.0, 0.0),
        Align2::LEFT_CENTER,
        truncate_to_width(
            painter,
            current,
            &font,
            (rect.width() - DROPDOWN_CHROME).max(1.0),
        ),
        font.clone(),
        text_color,
        None,
    );
    paint_chevron(
        painter,
        rect.right_center() - Vec2::new(18.0, 0.0),
        turn,
        metrics.ink(if enabled { 190 } else { 80 }),
    );
    if !enabled || sheet {
        return response;
    }
    let row_height = (text_size + 18.0).max(34.0);
    egui::Popup::from_toggle_button_response(&response)
        .id(popup_id)
        .gap(6.0)
        .width(popup_width.max(rect.width()))
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .frame(
            egui::Frame::new()
                .fill(if liquid_glass() {
                    Color32::TRANSPARENT
                } else {
                    metrics.surface_raised
                })
                .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
                .corner_radius(metrics.dialog_radius)
                .inner_margin(egui::Margin::same(5))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 10],
                    blur: 28,
                    spread: 0,
                    color: shade(Shade::Veil),
                }),
        )
        .show(|ui| {
            let backdrop = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            let slot = liquid_glass().then(|| backdrop.add(egui::Shape::Noop));
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for (value, label) in choices {
                        let (row, row_response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), row_height),
                            Sense::click(),
                        );
                        let active = value == selected;
                        if row_response.hovered() || active {
                            ui.painter().rect_filled(
                                row,
                                radius::control(),
                                metrics.ink(if row_response.hovered() { 18 } else { 9 }),
                            );
                        }
                        draw_text(
                            ui.painter(),
                            row.left_center() + Vec2::new(12.0, 0.0),
                            Align2::LEFT_CENTER,
                            truncate_to_width(
                                ui.painter(),
                                label,
                                &font,
                                (row.width() - 48.0).max(1.0),
                            ),
                            font.clone(),
                            if active {
                                metrics.text_primary
                            } else {
                                metrics.ink(tone::body())
                            },
                            None,
                        );
                        if active {
                            let c = row.right_center() - Vec2::new(18.0, 0.0);
                            let stroke = egui::Stroke::new(stroke::icon(), metrics.text_primary);
                            ui.painter().line_segment(
                                [c + Vec2::new(-5.0, 0.0), c + Vec2::new(-1.5, 3.5)],
                                stroke,
                            );
                            ui.painter().line_segment(
                                [c + Vec2::new(-1.5, 3.5), c + Vec2::new(5.0, -3.5)],
                                stroke,
                            );
                        }
                        if row_response.clicked() {
                            selected.clone_from(value);
                            egui::Popup::close_id(ui.ctx(), popup_id);
                        }
                    }
                });
            if let Some(slot) = slot {
                backdrop.set(
                    slot,
                    glass_shape(ui.min_rect().expand(5.0), 12.0, metrics.surface_raised),
                );
            }
        });
    response
}
