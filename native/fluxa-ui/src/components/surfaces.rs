use super::*;
use crate::flex::Space;
use crate::kit;

pub(crate) fn heading(
    ui: &mut crate::gfx::Ui,
    eyebrow: &str,
    title: &str,
    subtitle: &str,
    _metrics: UiMetrics,
) {
    let block = kit::widget(ui, |ui| {
        kit::view().column().gap(Space::S8).show(ui, |ui| {
            kit::text(eyebrow.to_uppercase(), kit::Type::Meta)
                .role(kit::Role::Muted)
                .show(ui);
            kit::text(title, kit::Type::Headline).show(ui);
            kit::text(subtitle, kit::Type::Body)
                .role(kit::Role::Muted)
                .show(ui);
        });
    });
    kit::view()
        .column()
        .items(ui, vec![block, kit::layout_items::space(ui, Space::S24)]);
}

pub(crate) fn field_label(ui: &mut crate::gfx::Ui, text: &str, _metrics: UiMetrics) {
    let label = kit::widget(ui, |ui| {
        kit::text(text.to_uppercase(), kit::Type::Meta)
            .role(kit::Role::Muted)
            .show(ui);
    });
    kit::view()
        .column()
        .items(ui, vec![label, kit::layout_items::space(ui, Space::S6)]);
}

pub(crate) fn note(ui: &mut crate::gfx::Ui, text: &str, _metrics: UiMetrics) {
    let line = kit::widget(ui, |ui| {
        kit::text(text, kit::Type::Meta)
            .role(kit::Role::Secondary)
            .show(ui);
    });
    kit::view().column().items(
        ui,
        vec![
            kit::layout_items::space(ui, Space::S16),
            line,
            kit::layout_items::space(ui, Space::S10),
        ],
    );
}

pub(crate) fn secondary_button(
    ui: &mut crate::gfx::Ui,
    label: &str,
    width: f32,
) -> crate::gfx::Response {
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
    ui: &mut crate::gfx::Ui,
    label: &str,
    width: f32,
    enabled: bool,
) -> crate::gfx::Response {
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
    ui: &mut crate::gfx::Ui,
    label: &str,
    width: f32,
    metrics: UiMetrics,
) -> crate::gfx::Response {
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

pub(crate) fn panel(
    ui: &mut crate::gfx::Ui,
    add: impl FnOnce(&mut crate::gfx::Ui),
    metrics: UiMetrics,
) {
    crate::gfx::Frame::NONE
        .fill(metrics.surface)
        .stroke(crate::gfx::Stroke::new(stroke::hairline(), metrics.border))
        .corner_radius(metrics.dialog_radius)
        .inner_margin(crate::gfx::Margin::same(PANEL_PADDING))
        .show(ui, add);
}

pub(crate) fn modal(
    context: &crate::gfx::Context,
    screen: Rect,
    id: &str,
    add: impl FnOnce(&mut crate::gfx::Ui),
    metrics: UiMetrics,
) {
    modal_width(context, screen, id, 340.0, add, metrics);
}

pub(crate) fn modal_width(
    context: &crate::gfx::Context,
    screen: Rect,
    id: &str,
    width: f32,
    add: impl FnOnce(&mut crate::gfx::Ui),
    metrics: UiMetrics,
) {
    context
        .layer_painter(crate::gfx::LayerId::new(
            crate::gfx::Order::Middle,
            Id::new((id, "scrim")),
        ))
        .rect_filled(screen, 0.0, shade(Shade::Scrim));
    crate::gfx::Area::new(Id::new(id))
        .order(crate::gfx::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(context, |ui| {
            crate::gfx::Frame::NONE
                .fill(metrics.surface)
                .stroke(crate::gfx::Stroke::new(stroke::hairline(), metrics.border))
                .corner_radius(metrics.dialog_radius)
                .inner_margin(crate::gfx::Margin::same(26))
                .show(ui, |ui| {
                    ui.set_width(width);
                    add(ui);
                });
        });
}

pub(crate) fn text_button(ui: &mut Ui, label: &str, size: f32, alpha: u8) -> Response {
    ui.add(crate::gfx::Button::new(RichText::new(label).size(size).color(ink(alpha))).frame(false))
}

pub(crate) fn surface(
    ui: &mut Ui,
    fill: Color32,
    radius: f32,
    margin: crate::gfx::Margin,
    add: impl FnOnce(&mut Ui),
    metrics: UiMetrics,
) {
    crate::gfx::Frame::NONE
        .fill(fill)
        .stroke(crate::gfx::Stroke::new(stroke::hairline(), metrics.border))
        .corner_radius(radius)
        .inner_margin(margin)
        .show(ui, add);
}

pub(crate) fn screen_bar(
    ui: &mut crate::gfx::Ui,
    assets: &impl HomeAssets,
    title: &str,
    back_label: &str,
    metrics: UiMetrics,
) -> bool {
    let mut back = false;
    let arrow = assets.icon("ArrowLeft");
    let bar = kit::widget(ui, |ui| {
        let button = kit::widget(ui, |ui| {
            back = back_button(ui, arrow, metrics)
                .on_hover_text(back_label)
                .clicked();
        });
        let heading = kit::widget(ui, |ui| {
            kit::text(title, kit::Type::Title).show(ui);
        });
        kit::view().row().gap(Space::S4).hug().items(
            ui,
            vec![kit::layout_items::space(ui, Space::S12), button, heading],
        );
    });
    kit::view().column().items(
        ui,
        vec![
            kit::layout_items::gap(metrics.section_gap),
            bar,
            kit::layout_items::gap(metrics.section_gap),
        ],
    );
    back
}

pub(crate) fn close_button(
    ui: &mut crate::gfx::Ui,
    assets: &impl HomeAssets,
    label: &str,
    metrics: UiMetrics,
) -> bool {
    let mut clicked = false;
    let button = kit::widget(ui, |ui| {
        let icon = kit::widget(ui, |ui| {
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
        let inset = kit::layout_items::space(ui, Space::S12);
        kit::view().row().hug().items(ui, vec![inset, icon]);
    });
    kit::view().column().items(
        ui,
        vec![
            kit::layout_items::gap(metrics.section_gap),
            button,
            kit::layout_items::gap(metrics.section_gap),
        ],
    );
    clicked
}

pub(crate) fn section(
    ui: &mut crate::gfx::Ui,
    flat: bool,
    width: f32,
    metrics: UiMetrics,
    add: impl FnOnce(&mut crate::gfx::Ui),
) {
    if flat {
        ui.scope_builder(
            crate::gfx::UiBuilder::new()
                .layout(crate::gfx::Layout::top_down(crate::gfx::Align::Min)),
            |ui| {
                ui.set_width(width);
                add(ui);
            },
        );
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
    sense: Sense,
    assets: &mut impl HomeAssets,
) -> Vec<Response> {
    ratings
        .iter()
        .map(|rating| rating_item(ui, rating, size, color, muted, sense, assets))
        .collect()
}

fn rating_item(
    ui: &mut Ui,
    (source, score): &(String, String),
    size: f32,
    color: Color32,
    muted: Color32,
    sense: Sense,
    assets: &mut impl HomeAssets,
) -> Response {
    let line = ui
        .painter()
        .layout_no_wrap(score.clone(), crate::gfx::FontId::proportional(size), color)
        .size()
        .y;
    let logo = crate::rating_logo(source, score).and_then(|name| assets.logo(name));
    let shown = kit::view().row().hug().gap(Space::S4).show(ui, |ui| {
        if let Some((texture, aspect)) = logo {
            ui.add(crate::gfx::Image::new((texture, aspect * line)))
                .on_hover_text(source.as_str());
        } else {
            ui.label(RichText::new(source).size(size).color(muted));
        }
        ui.label(RichText::new(score).size(size).strong().color(color));
    });
    ui.interact(
        shown.response.rect,
        Id::new(("fluxa-rating", source)),
        sense,
    )
}

pub(crate) fn meta_line(
    ui: &mut Ui,
    facts: &[String],
    ratings: &[(String, String)],
    size: f32,
    color: Color32,
    muted: Color32,
    sense: Sense,
    assets: &mut impl HomeAssets,
) -> Vec<Response> {
    let mut hits = Vec::new();
    kit::view().row().wrap().gap(Space::S8).show(ui, |ui| {
        let mut first = true;
        let mut separate = |ui: &mut Ui| {
            if !first {
                ui.label(RichText::new("·").size(size).color(muted));
            }
            first = false;
        };
        for fact in facts {
            separate(ui);
            ui.label(RichText::new(fact).size(size).strong().color(color));
        }
        for rating in ratings {
            separate(ui);
            hits.push(rating_item(ui, rating, size, color, muted, sense, assets));
        }
    });
    hits
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
        crate::gfx::Stroke::new(look.stroke, look.border),
        crate::gfx::StrokeKind::Inside,
    );
}
