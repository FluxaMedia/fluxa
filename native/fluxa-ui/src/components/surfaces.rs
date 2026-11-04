use super::*;

pub(crate) fn heading(
    ui: &mut egui::Ui,
    eyebrow: &str,
    title: &str,
    subtitle: &str,
    metrics: UiMetrics,
) {
    ui.label(
        RichText::new(eyebrow.to_uppercase())
            .size(metrics.text.label)
            .color(metrics.text_muted),
    );
    ui.add_space(10.0);
    ui.label(
        RichText::new(title)
            .size(36.0)
            .strong()
            .color(metrics.text_primary),
    );
    ui.add_space(8.0);
    ui.label(
        RichText::new(subtitle)
            .size(metrics.text.body)
            .color(metrics.text_muted),
    );
    ui.add_space(28.0);
}

pub(crate) fn field_label(ui: &mut egui::Ui, text: &str, metrics: UiMetrics) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(metrics.text.label)
            .color(metrics.text_muted),
    );
    ui.add_space(6.0);
}

pub(crate) fn note(ui: &mut egui::Ui, text: &str, metrics: UiMetrics) {
    ui.add_space(16.0);
    ui.label(
        RichText::new(text)
            .size(metrics.text.label)
            .color(metrics.text_secondary),
    );
    ui.add_space(10.0);
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

pub(crate) fn panel(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui), metrics: UiMetrics) {
    egui::Frame::NONE
        .fill(metrics.surface)
        .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
        .corner_radius(metrics.dialog_radius)
        .inner_margin(egui::Margin::same(22))
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
    ui.horizontal(|ui| {
        ui.add_space(metrics.control_gap);
        back = back_button(ui, assets.icon("ArrowLeft"), metrics)
            .on_hover_text(back_label)
            .clicked();
        ui.add_space(crate::Space::S4.px(&metrics));
        ui.label(
            RichText::new(title)
                .size(metrics.text.title)
                .strong()
                .color(text_primary()),
        );
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
    ui.horizontal(|ui| {
        ui.add_space(metrics.control_gap);
        clicked = icon_button(
            ui,
            assets.icon("Close"),
            metrics.icon_button_size,
            ink(tone::body()),
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
    metrics: UiMetrics,
    add: impl FnOnce(&mut egui::Ui),
) {
    if flat {
        ui.vertical(add);
    } else {
        panel(ui, add, metrics);
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
        if let Some((texture, aspect)) = logo {
            ui.add(egui::Image::new((texture, aspect * size * 1.15)))
                .on_hover_text(source.as_str());
        } else {
            ui.label(RichText::new(source).size(size).color(muted));
        }
        ui.label(RichText::new(score).size(size).strong().color(color));
        ui.add_space(size * 0.4);
    }
}

pub fn raised_card(painter: &Painter, card: Rect, metrics: UiMetrics) {
    painter.rect_filled(card, metrics.card_radius, metrics.surface_raised);
    painter.rect_stroke(
        card,
        metrics.card_radius,
        egui::Stroke::new(stroke::hairline(), ink(34)),
        egui::StrokeKind::Inside,
    );
}
