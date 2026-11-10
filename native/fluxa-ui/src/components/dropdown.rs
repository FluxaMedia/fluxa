use super::*;
use crate::Space;

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
    (text_width + metrics.dropdown_dropdown_chrome + 2.0).clamp(min_width, max_width.max(min_width))
}

pub(crate) fn dropdown_popup_width(
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
    (widest_label + 64.0).max(minimum).min(600.0)
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

const CHECK_WIDTH: f32 = 20.0;

fn paint_chevron(painter: &Painter, center: Pos2, open: f32, color: Color32) {
    let flip = 1.0 - 2.0 * open;
    let stroke = crate::gfx::Stroke::new(stroke::icon(), color);
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
    let open = enabled && crate::gfx::Popup::is_id_open(ui.ctx(), popup_id);
    let turn = crate::motion::toggle(ui.ctx(), id.with("chevron"), open, crate::motion::fast());
    let radius = rect.height() * 0.5;
    let field_state = crate::stylesheet::State {
        hovered: response.hovered(),
        selected: open,
        enabled,
        ..Default::default()
    };
    let look = crate::stylesheet::resolve(ui.ctx(), "dropdown", response.id, field_state, None);
    let painter = ui.painter();
    painter.rect(
        rect,
        radius,
        look.fill,
        crate::gfx::Stroke::new(look.stroke, look.border),
        crate::gfx::StrokeKind::Inside,
    );
    let font = FontId::proportional(text_size);
    let text_color = look.text;
    draw_text(
        painter,
        rect.left_center() + Vec2::new(14.0, 0.0),
        Align2::LEFT_CENTER,
        truncate_to_width(
            painter,
            current,
            &font,
            (rect.width() - metrics.dropdown_dropdown_chrome).max(1.0),
        ),
        font.clone(),
        text_color,
        None,
    );
    paint_chevron(
        painter,
        rect.right_center() - Vec2::new(18.0, 0.0),
        turn,
        crate::stylesheet::resolve(
            ui.ctx(),
            "dropdown.chevron",
            response.id.with("chevron-look"),
            crate::stylesheet::State {
                enabled,
                ..Default::default()
            },
            None,
        )
        .text,
    );
    if !enabled || sheet {
        return response;
    }
    let row_height = (text_size + 18.0).max(34.0);
    let popup_look = crate::stylesheet::look(ui.ctx(), "popup");
    crate::gfx::Popup::from_toggle_button_response(&response)
        .id(popup_id)
        .gap(6.0)
        .width(popup_width.max(rect.width()))
        .close_behavior(crate::gfx::PopupCloseBehavior::CloseOnClickOutside)
        .frame(
            crate::gfx::Frame::new()
                .fill(if liquid_glass() {
                    Color32::TRANSPARENT
                } else {
                    popup_look.fill
                })
                .stroke(crate::gfx::Stroke::new(popup_look.stroke, popup_look.border))
                .corner_radius(metrics.dialog_radius)
                .inner_margin(crate::gfx::Margin::same(5))
                .shadow(crate::gfx::epaint::Shadow {
                    offset: [0, 10],
                    blur: 28,
                    spread: 0,
                    color: shade(Shade::Veil),
                }),
        )
        .show(|ui| {
            let backdrop = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            let slot = liquid_glass().then(|| backdrop.add(crate::gfx::Shape::Noop));
            super::pan_area(
                ui,
                "fluxa-dropdown-scroll",
                |area| area.max_height(300.0),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for (value, label) in choices {
                        let active = value == selected;
                        let text = std::cell::Cell::new(Rect::NOTHING);
                        let check = std::cell::Cell::new(Rect::NOTHING);
                        let shown = crate::kit::view()
                            .row()
                            .height(row_height)
                            .clickable()
                            .class("dropdown-row")
                            .items(
                                ui,
                                vec![
                                    crate::kit::layout_items::space(ui, Space::S12),
                                    crate::kit::item(Vec2::new(0.0, row_height), |_, rect| {
                                        text.set(rect)
                                    })
                                    .grow(1.0)
                                    .min_width(0.0),
                                    crate::kit::item(Vec2::new(CHECK_WIDTH, row_height), |_, rect| {
                                        check.set(rect)
                                    })
                                    .shrink(0.0),
                                    crate::kit::layout_items::space(ui, Space::S8),
                                ],
                            );
                        let (text, check) = (text.get(), check.get());
                        let row_response = shown.response;
                        let option = crate::stylesheet::resolve(
                            ui.ctx(),
                            "dropdown.option",
                            row_response.id,
                            crate::stylesheet::State {
                                hovered: row_response.hovered(),
                                selected: active,
                                enabled: true,
                                ..Default::default()
                            },
                            None,
                        );
                        ui.painter()
                            .rect_filled(row_response.rect, radius::control(), option.fill);
                        draw_text(
                            ui.painter(),
                            text.left_center(),
                            Align2::LEFT_CENTER,
                            truncate_to_width(ui.painter(), label, &font, text.width().max(1.0)),
                            font.clone(),
                            option.text,
                            None,
                        );
                        if active {
                            let c = check.center();
                            let stroke = crate::gfx::Stroke::new(stroke::icon(), metrics.text_primary);
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
                            crate::gfx::Popup::close_id(ui.ctx(), popup_id);
                        }
                    }
                },
            );
            if let Some(slot) = slot {
                backdrop.set(
                    slot,
                    glass_shape(ui.min_rect().expand(5.0), 12.0, metrics.surface_raised),
                );
            }
        });
    response
}
