use super::*;
use crate::flex::{Align, Insets, Weight, column, fill, leaf, paint_text, row, text};
use crate::{IconSize, Radius, Space};

fn pill_fill(selected: bool, hover: f32, metrics: &UiMetrics) -> Color32 {
    if selected {
        metrics.ink(tone::strong())
    } else {
        metrics.ink((tone::faint() as f32 + hover * 20.0) as u8)
    }
}

fn label_color(selected: bool, hovered: bool, metrics: &UiMetrics) -> Color32 {
    metrics.ink(if selected || hovered {
        tone::full()
    } else {
        tone::body()
    })
}

pub(crate) fn chip(ui: &mut Ui, label: &str, selected: bool, metrics: UiMetrics) -> Response {
    let font = Weight::Medium.font(metrics.text.label);
    let tree = row([text(ui.painter(), label, &font).id(0)])
        .align(Align::Center)
        .padding(Insets::xy(Space::S12.px(&metrics), Space::S6.px(&metrics)))
        .min_height(metrics.icon_button_size * 0.8);
    let (rect, response) = ui.allocate_exact_size(tree.outer_size(), Sense::click());
    let slot = tree.solve(rect).find(0).unwrap_or(rect);
    let hover = crate::motion::hover(ui.ctx(), &response);
    crate::motion::press_scale(ui.painter(), rect, || {
        let painter = ui.painter();
        painter.rect_filled(
            rect,
            rect.height() * 0.5,
            pill_fill(selected, hover, &metrics),
        );
        paint_text(
            painter,
            slot,
            label,
            &font,
            label_color(selected, response.hovered(), &metrics),
            Align2::LEFT_CENTER,
        );
    });
    response
}

pub(crate) fn segmented_width(painter: &Painter, labels: &[String], metrics: UiMetrics) -> f32 {
    let font = Weight::Medium.font(metrics.text.label);
    let widest = labels
        .iter()
        .map(|label| crate::flex::text_size(painter, label, &font).x)
        .fold(0.0, f32::max);
    labels.len() as f32 * (widest + 28.0) + Space::S2.px(&metrics) * 2.0
}

pub(crate) fn segmented(
    ui: &mut Ui,
    id: Id,
    labels: &[&str],
    selected: usize,
    metrics: UiMetrics,
) -> Option<usize> {
    let font = Weight::Medium.font(metrics.text.label);
    let height = metrics.screen_control_height;
    let width = ui.available_width().min(labels.len() as f32 * 120.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let pad = Space::S2.px(&metrics);
    let tree = row(labels.iter().enumerate().map(|(i, _)| fill().id(i)))
        .padding(Insets::all(pad))
        .align(Align::Stretch);
    let placed = tree.solve(rect);
    let painter = ui.painter();
    painter.rect_filled(rect, rect.height() * 0.5, metrics.ink(tone::faint()));
    let position = crate::motion::ease(
        ui.ctx(),
        id.with("indicator"),
        selected as f32,
        crate::motion::normal(),
        crate::motion::Ease::Token,
    );
    let first = placed.find(0).unwrap_or(rect);
    let slot = first.width();
    let indicator = Rect::from_min_size(
        Pos2::new(first.left() + slot * position, first.top()),
        first.size(),
    );
    painter.rect_filled(
        indicator,
        indicator.height() * 0.5,
        metrics.ink(tone::dim()),
    );
    let mut chosen = None;
    for (index, label) in labels.iter().enumerate() {
        let cell = placed.find(index).unwrap_or(rect);
        let response = ui.interact(cell, id.with(index), Sense::click());
        paint_text(
            ui.painter(),
            cell,
            label,
            &font,
            label_color(index == selected, response.hovered(), &metrics),
            Align2::CENTER_CENTER,
        );
        if response.clicked() && index != selected {
            chosen = Some(index);
        }
    }
    chosen
}

pub(crate) fn stepper(
    ui: &mut Ui,
    id: Id,
    value_label: &str,
    can_decrease: bool,
    can_increase: bool,
    metrics: UiMetrics,
) -> Option<i32> {
    let font = Weight::Bold.font(metrics.text.body);
    let button = metrics.screen_control_height;
    let value_width = crate::flex::text_size(ui.painter(), value_label, &font)
        .x
        .max(button);
    let tree = row([
        leaf(Vec2::splat(button)).id(0),
        leaf(Vec2::new(value_width, button)).id(1),
        leaf(Vec2::splat(button)).id(2),
    ])
    .gap(Space::S4.px(&metrics))
    .align(Align::Center);
    let (rect, _) = ui.allocate_exact_size(tree.outer_size(), Sense::hover());
    let placed = tree.solve(rect);
    let mut delta = None;
    for (slot_id, sign, enabled) in [(0, -1, can_decrease), (2, 1, can_increase)] {
        let slot = placed.find(slot_id).unwrap_or(rect);
        let response = ui.interact(slot, id.with(slot_id), Sense::click());
        let alpha = match (enabled, response.hovered()) {
            (false, _) => tone::faint(),
            (true, true) => tone::strong(),
            (true, false) => tone::soft(),
        };
        let painter = ui.painter();
        painter.rect_filled(slot, slot.height() * 0.5, metrics.ink(alpha));
        let arm = IconSize::Sm.px(&metrics) * 0.4;
        let color = metrics.ink(if enabled { tone::full() } else { tone::muted() });
        let line = egui::Stroke::new(stroke::emphasis(), color);
        let center = slot.center();
        painter.line_segment(
            [center - Vec2::new(arm, 0.0), center + Vec2::new(arm, 0.0)],
            line,
        );
        if sign > 0 {
            painter.line_segment(
                [center - Vec2::new(0.0, arm), center + Vec2::new(0.0, arm)],
                line,
            );
        }
        if enabled && response.clicked() {
            delta = Some(sign);
        }
    }
    paint_text(
        ui.painter(),
        placed.find(1).unwrap_or(rect),
        value_label,
        &font,
        metrics.text_primary,
        Align2::CENTER_CENTER,
    );
    delta
}

pub(crate) fn switch(ui: &mut Ui, id: Id, on: bool, metrics: UiMetrics) -> Response {
    let size = Vec2::new(
        metrics.settings_toggle_width,
        metrics.settings_toggle_height,
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    toggle(ui.ctx(), ui.painter(), id, rect, on, metrics);
    response
}

pub(crate) enum Trailing<'a> {
    None,
    Value(&'a str),
    Chevron,
    Switch(bool),
}

pub(crate) struct ListRow<'a> {
    pub icon: Option<egui::TextureId>,
    pub title: &'a str,
    pub subtitle: Option<&'a str>,
    pub trailing: Trailing<'a>,
}

pub(crate) fn list_row(ui: &mut Ui, row_data: ListRow, metrics: UiMetrics) -> Response {
    let title_font = Weight::Medium.font(metrics.text.body);
    let sub_font = Weight::Regular.font(metrics.text.label);
    let value_font = Weight::Regular.font(metrics.text.body);
    let icon = IconSize::Md.px(&metrics);
    let trailing_size = match &row_data.trailing {
        Trailing::None => Vec2::ZERO,
        Trailing::Value(value) => {
            let width = crate::flex::text_size(ui.painter(), value, &value_font).x;
            Vec2::new(
                width + IconSize::Sm.px(&metrics) + Space::S4.px(&metrics),
                20.0,
            )
        }
        Trailing::Chevron => Vec2::splat(IconSize::Sm.px(&metrics)),
        Trailing::Switch(_) => Vec2::new(
            metrics.settings_toggle_width,
            metrics.settings_toggle_height,
        ),
    };
    let mut lines = vec![leaf(Vec2::new(0.0, metrics.text.body * 1.3)).id(1)];
    if row_data.subtitle.is_some() {
        lines.push(leaf(Vec2::new(0.0, metrics.text.label * 1.3)).id(2));
    }
    let tree = row([
        leaf(if row_data.icon.is_some() {
            Vec2::splat(icon)
        } else {
            Vec2::ZERO
        })
        .id(0),
        column(lines)
            .grow(1.0)
            .align(Align::Stretch)
            .min_width(0.0)
            .gap(Space::S2.px(&metrics)),
        leaf(trailing_size).id(3),
    ])
    .gap(Space::S12.px(&metrics))
    .align(Align::Center)
    .padding(Insets::xy(Space::S16.px(&metrics), Space::S8.px(&metrics)))
    .min_height(metrics.settings_row_height);
    let size = Vec2::new(ui.available_width(), tree.outer_size().y);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let placed = tree.solve(rect);
    let hover = crate::motion::hover(ui.ctx(), &response);
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        Radius::Md.px(&metrics),
        metrics.ink((tone::faint() as f32 * hover) as u8),
    );
    if let (Some(texture), Some(slot)) = (row_data.icon, placed.find(0)) {
        painter.image(texture, slot, full_uv(), metrics.ink(tone::body()));
    }
    if let Some(slot) = placed.find(1) {
        paint_text(
            painter,
            slot,
            row_data.title,
            &title_font,
            metrics.text_primary,
            Align2::LEFT_CENTER,
        );
    }
    if let (Some(subtitle), Some(slot)) = (row_data.subtitle, placed.find(2)) {
        paint_text(
            painter,
            slot,
            subtitle,
            &sub_font,
            metrics.text_secondary,
            Align2::LEFT_CENTER,
        );
    }
    if let Some(slot) = placed.find(3) {
        match row_data.trailing {
            Trailing::None => {}
            Trailing::Value(value) => {
                let chevron = Rect::from_center_size(
                    Pos2::new(
                        slot.right() - IconSize::Sm.px(&metrics) * 0.5,
                        slot.center().y,
                    ),
                    Vec2::splat(IconSize::Sm.px(&metrics)),
                );
                paint_text(
                    painter,
                    Rect::from_min_max(
                        slot.min,
                        Pos2::new(chevron.left() - Space::S4.px(&metrics), slot.max.y),
                    ),
                    value,
                    &value_font,
                    metrics.text_muted,
                    Align2::RIGHT_CENTER,
                );
                paint_chevron(painter, chevron, &metrics);
            }
            Trailing::Chevron => paint_chevron(painter, slot, &metrics),
            Trailing::Switch(on) => {
                toggle(
                    ui.ctx(),
                    ui.painter(),
                    response.id.with("switch"),
                    slot,
                    on,
                    metrics,
                );
            }
        }
    }
    response
}

fn paint_chevron(painter: &Painter, rect: Rect, metrics: &UiMetrics) {
    let arm = rect.width() * 0.25;
    let center = rect.center();
    let stroke = egui::Stroke::new(stroke::icon(), metrics.ink(tone::muted()));
    painter.line_segment(
        [
            center + Vec2::new(-arm * 0.5, -arm),
            center + Vec2::new(arm * 0.5, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + Vec2::new(arm * 0.5, 0.0),
            center + Vec2::new(-arm * 0.5, arm),
        ],
        stroke,
    );
}

pub(crate) fn swatch(ui: &mut Ui, color: Color32, selected: bool, metrics: UiMetrics) -> Response {
    let size = IconSize::Xl.px(&metrics);
    let outer = size + Space::S8.px(&metrics);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(outer), Sense::click());
    let hover = crate::motion::hover(ui.ctx(), &response);
    crate::motion::press_scale(ui.painter(), rect, || {
        let painter = ui.painter();
        let radius = size * 0.5 * (1.0 + 0.06 * hover);
        painter.circle_filled(rect.center(), radius, color);
        painter.circle_stroke(
            rect.center(),
            radius,
            egui::Stroke::new(stroke::hairline(), metrics.border),
        );
        if selected {
            painter.circle_stroke(
                rect.center(),
                outer * 0.5 - 1.0,
                egui::Stroke::new(stroke::emphasis(), metrics.text_primary),
            );
        }
    });
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiFormFactor, Viewport};

    fn click_at(context: &egui::Context, point: Pos2, mut run: impl FnMut(&mut Ui)) {
        context.set_fonts(crate::fonts::definitions());
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let frames = [
            vec![egui::Event::PointerMoved(point)],
            vec![egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            }],
            vec![egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        ];
        for (tick, events) in frames.into_iter().enumerate() {
            let input = egui::RawInput {
                time: Some(tick as f64 / 60.0),
                predicted_dt: 1.0 / 60.0,
                screen_rect: Some(screen),
                events,
                ..Default::default()
            };
            context
                .run_ui(input, |root| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(root, |ui| run(ui));
                })
                .drop_without_applying_deltas();
        }
    }

    fn metrics() -> UiMetrics {
        UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Desktop))
    }

    #[test]
    fn segmented_reports_the_clicked_segment() {
        let context = egui::Context::default();
        let metrics = metrics();
        let mut chosen = None;
        click_at(&context, Pos2::new(350.0, 12.0), |ui| {
            chosen = chosen.or(segmented(ui, Id::new("seg"), &["a", "b", "c"], 0, metrics));
        });
        assert_eq!(chosen, Some(2));
    }

    #[test]
    fn stepper_ignores_a_disabled_direction() {
        let context = egui::Context::default();
        let metrics = metrics();
        let mut delta = None;
        click_at(&context, Pos2::new(10.0, 12.0), |ui| {
            delta = delta.or(stepper(ui, Id::new("step"), "3", false, true, metrics));
        });
        assert_eq!(delta, None);
    }

    #[test]
    fn stepper_increases_from_the_plus_button() {
        let context = egui::Context::default();
        let metrics = metrics();
        let mut delta = None;
        let plus =
            metrics.screen_control_height * 2.0 + 4.0 * 2.0 + metrics.screen_control_height * 0.5;
        click_at(&context, Pos2::new(plus + 8.0, 12.0), |ui| {
            delta = delta.or(stepper(ui, Id::new("step"), "3", true, true, metrics));
        });
        assert_eq!(delta, Some(1));
    }
}
