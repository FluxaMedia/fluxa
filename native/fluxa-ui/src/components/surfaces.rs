use super::*;
use crate::flex::Space;
use crate::kit;

pub(crate) fn heading(
    ui: &mut egui::Ui,
    eyebrow: &str,
    title: &str,
    subtitle: &str,
    _metrics: UiMetrics,
) {
    kit::view().column().gap(Space::S8).show(ui, |ui| {
        kit::text(eyebrow.to_uppercase(), kit::Type::Meta)
            .role(kit::Role::Muted)
            .show(ui);
        kit::text(title, kit::Type::Headline).show(ui);
        kit::text(subtitle, kit::Type::Body)
            .role(kit::Role::Muted)
            .show(ui);
    });
    kit::space(ui, Space::S24);
}

pub(crate) fn field_label(ui: &mut egui::Ui, text: &str, _metrics: UiMetrics) {
    kit::text(text.to_uppercase(), kit::Type::Meta)
        .role(kit::Role::Muted)
        .show(ui);
    kit::space(ui, Space::S6);
}

pub(crate) fn note(ui: &mut egui::Ui, text: &str, _metrics: UiMetrics) {
    kit::space(ui, Space::S16);
    kit::text(text, kit::Type::Meta)
        .role(kit::Role::Secondary)
        .show(ui);
    kit::space(ui, Space::S10);
}

pub(crate) fn secondary_button(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    button_core(
        ui,
        label,
        width,
        button_height(),
        ButtonKind::Secondary,
        size::BUTTON_TEXT,
        true,
        None,
    )
}

pub(crate) fn primary_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    enabled: bool,
) -> egui::Response {
    button_core(
        ui,
        label,
        width,
        button_height(),
        ButtonKind::Primary,
        size::BUTTON_TEXT,
        enabled,
        None,
    )
}

pub(crate) fn danger_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    metrics: UiMetrics,
) -> egui::Response {
    button_core(
        ui,
        label,
        width,
        button_height(),
        ButtonKind::Danger,
        size::BUTTON_TEXT,
        true,
        Some(metrics),
    )
}

const PANEL_PADDING: i8 = 22;

pub(crate) fn panel(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui), metrics: UiMetrics) {
    egui::Frame::NONE
        .fill(metrics.surface)
        .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
        .corner_radius(metrics.dialog_radius)
        .inner_margin(egui::Margin::same(PANEL_PADDING))
        .show(ui, add);
}

pub(crate) fn modal(
    context: &egui::Context,
    screen: Rect,
    id: &str,
    add: impl FnOnce(&mut egui::Ui),
    metrics: UiMetrics,
) {
    modal_width(context, screen, id, 340.0, add, metrics);
}

pub(crate) fn modal_width(
    context: &egui::Context,
    screen: Rect,
    id: &str,
    width: f32,
    add: impl FnOnce(&mut egui::Ui),
    metrics: UiMetrics,
) {
    context
        .layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            Id::new((id, "scrim")),
        ))
        .rect_filled(screen, 0.0, shade(Shade::Scrim));
    egui::Area::new(Id::new(id))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(context, |ui| {
            egui::Frame::NONE
                .fill(metrics.surface)
                .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
                .corner_radius(metrics.dialog_radius)
                .inner_margin(egui::Margin::same(26))
                .show(ui, |ui| {
                    ui.set_width(width);
                    add(ui);
                });
        });
}

pub(crate) fn text_button(ui: &mut Ui, label: &str, size: f32, alpha: u8) -> Response {
    ui.add(egui::Button::new(RichText::new(label).size(size).color(ink(alpha))).frame(false))
}

pub(crate) fn surface(
    ui: &mut Ui,
    fill: Color32,
    radius: f32,
    margin: egui::Margin,
    add: impl FnOnce(&mut Ui),
    metrics: UiMetrics,
) {
    egui::Frame::NONE
        .fill(fill)
        .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
        .corner_radius(radius)
        .inner_margin(margin)
        .show(ui, add);
}

pub(crate) fn screen_bar(
    ui: &mut egui::Ui,
    assets: &impl HomeAssets,
    title: &str,
    back_label: &str,
    metrics: UiMetrics,
) -> bool {
    ui.add_space(metrics.section_gap);
    let mut back = false;
    kit::view().row().gap(Space::S4).hug().show(ui, |ui| {
        kit::space(ui, Space::S12);
        back = back_button(ui, assets.icon("ArrowLeft"), metrics)
            .on_hover_text(back_label)
            .clicked();
        kit::text(title, kit::Type::Title).show(ui);
    });
    ui.add_space(metrics.section_gap);
    back
}

pub(crate) fn close_button(
    ui: &mut egui::Ui,
    assets: &impl HomeAssets,
    label: &str,
    metrics: UiMetrics,
) -> bool {
    ui.add_space(metrics.section_gap);
    let mut clicked = false;
    kit::view().row().hug().show(ui, |ui| {
        kit::space(ui, Space::S12);
        clicked = icon_button(
            ui,
            assets.icon("Close"),
            metrics.icon_button_size,
            crate::stylesheet::color("text.body"),
            true,
            false,
            true,
        )
        .on_hover_text(label)
        .clicked();
    });
    ui.add_space(metrics.section_gap);
    clicked
}

pub(crate) fn section(
    ui: &mut egui::Ui,
    flat: bool,
    width: f32,
    metrics: UiMetrics,
    add: impl FnOnce(&mut egui::Ui),
) {
    if flat {
        ui.vertical(|ui| {
            ui.set_width(width);
            add(ui);
        });
    } else {
        panel(
            ui,
            |ui| {
                ui.set_width(width - f32::from(PANEL_PADDING) * 2.0);
                add(ui);
            },
            metrics,
        );
    }
}

pub(crate) fn rating_row(
    ui: &mut Ui,
    ratings: &[(String, String)],
    size: f32,
    color: Color32,
    muted: Color32,
    assets: &mut impl HomeAssets,
) {
    for (source, score) in ratings {
        let logo = crate::rating_logo(source, score).and_then(|name| assets.logo(name));
        kit::view().row().hug().gap(Space::S4).show(ui, |ui| {
            if let Some((texture, aspect)) = logo {
                ui.add(egui::Image::new((texture, aspect * size * 1.15)))
                    .on_hover_text(source.as_str());
            } else {
                ui.label(RichText::new(source).size(size).color(muted));
            }
            ui.label(RichText::new(score).size(size).strong().color(color));
        });
    }
}

pub fn raised_card(painter: &Painter, card: Rect, metrics: UiMetrics) {
    let look = crate::stylesheet::resolve(
        painter.ctx(),
        "card",
        Id::new("fluxa-raised-card"),
        crate::stylesheet::State {
            enabled: true,
            ..Default::default()
        },
        Some(&metrics),
    );
    let radius = look.radius.of(card.height());
    painter.rect_filled(card, radius, look.fill);
    painter.rect_stroke(
        card,
        radius,
        egui::Stroke::new(look.stroke, look.border),
        egui::StrokeKind::Inside,
    );
}
