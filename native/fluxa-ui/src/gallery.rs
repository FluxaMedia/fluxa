use super::*;
use crate::components::{
    DrawerSide, bottom_sheet, labeled_button, primary_button, secondary_button, side_drawer,
};
use crate::flex::{Insets, Justify, column, leaf, row, text};

#[derive(Clone, Default)]
struct State {
    sheet: bool,
    drawer: bool,
    selected: usize,
    started: bool,
}

pub fn draw_gallery(
    context: &egui::Context,
    viewport: Viewport,
    scene: &str,
    assets: &mut impl HomeAssets,
) {
    let metrics = metrics_for_assets(viewport, assets);
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    context
        .layer_painter(egui::LayerId::background())
        .rect_filled(screen, 0.0, background());
    let state_id = Id::new("fluxa-gallery");
    let mut state = context
        .data(|data| data.get_temp::<State>(state_id))
        .unwrap_or_default();
    if !state.started {
        state.started = true;
        state.sheet = scene == "sheet";
        state.drawer = scene == "drawer";
    }
    let icons = ["Settings", "Plus", "Lock", "Edit"].map(|name| assets.icon(name));
    let labels = ["Audio", "Subtitles", "Speed", "A very long label"];
    egui::Area::new(Id::new("fluxa-gallery-body"))
        .fixed_pos(Pos2::ZERO)
        .show(context, |ui| {
            ui.set_min_size(screen.size());
            ui.add_space(Space::S48.px(&metrics));
            ui.horizontal(|ui| {
                ui.add_space(Space::S24.px(&metrics));
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(Space::S12.px(&metrics));
                    ui.label(
                        RichText::new("LabeledButton")
                            .size(metrics.text.title)
                            .strong()
                            .color(text_primary()),
                    );
                    ui.horizontal_wrapped(|ui| {
                        for (index, label) in labels.iter().enumerate() {
                            if labeled_button(
                                ui,
                                icons[index],
                                label,
                                state.selected == index,
                                metrics,
                            )
                            .clicked()
                            {
                                state.selected = index;
                            }
                        }
                    });
                    ui.label(
                        RichText::new("Overlays")
                            .size(metrics.text.title)
                            .strong()
                            .color(text_primary()),
                    );
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Bottom sheet", 160.0, true).clicked() {
                            state.sheet = true;
                        }
                        if secondary_button(ui, "Side drawer", 160.0).clicked() {
                            state.drawer = true;
                        }
                    });
                    ui.label(
                        RichText::new("Layout tree")
                            .size(metrics.text.title)
                            .strong()
                            .color(text_primary()),
                    );
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(360.0, 120.0), Sense::hover());
                    paint_layout_sample(ui.painter(), rect, metrics);
                });
            });
        });
    if bottom_sheet(
        context,
        "fluxa-gallery-sheet",
        state.sheet,
        viewport,
        metrics,
        |ui| {
            ui.label(
                RichText::new("Playback")
                    .size(metrics.text.title)
                    .strong()
                    .color(text_primary()),
            );
            ui.add_space(Space::S12.px(&metrics));
            ui.horizontal_wrapped(|ui| {
                for (index, label) in labels.iter().enumerate() {
                    labeled_button(ui, icons[index], label, index == 1, metrics);
                }
            });
        },
    ) {
        state.sheet = false;
    }
    if side_drawer(
        context,
        "fluxa-gallery-drawer",
        state.drawer,
        DrawerSide::Right,
        viewport,
        metrics,
        |ui| {
            ui.label(
                RichText::new("Details")
                    .size(metrics.text.title)
                    .strong()
                    .color(text_primary()),
            );
        },
    ) {
        state.drawer = false;
    }
    context.data_mut(|data| data.insert_temp(state_id, state));
}

fn paint_layout_sample(painter: &egui::Painter, rect: Rect, metrics: UiMetrics) {
    let font = FontId::proportional(metrics.labeled_button_text);
    let tree = column([
        row([
            leaf(Vec2::splat(32.0)).id(0),
            text(painter, "Title row", &font).id(1),
            leaf(Vec2::ZERO).grow(1.0),
            leaf(Vec2::new(48.0, 24.0)).id(2),
        ])
        .gap(Space::S8.px(&metrics))
        .align(Align::Center),
        row((3..8).map(|i| leaf(Vec2::new(54.0, 28.0)).id(i)))
            .gap(Space::S6.px(&metrics))
            .line_gap(Space::S6.px(&metrics))
            .wrap(),
    ])
    .gap(Space::S10.px(&metrics))
    .align(Align::Stretch)
    .padding(Insets::all(Space::S12.px(&metrics)))
    .justify(Justify::Start);
    painter.rect_stroke(
        rect,
        radius::control(),
        egui::Stroke::new(stroke::HAIRLINE, metrics.border),
        egui::StrokeKind::Inside,
    );
    let placed = tree.solve(rect);
    for (id, slot) in placed.tagged() {
        if id == 1 {
            draw_text(
                painter,
                slot.left_center(),
                Align2::LEFT_CENTER,
                "Title row",
                font.clone(),
                text_primary(),
                None,
            );
        } else {
            painter.rect_filled(slot, radius::chip(), ink(tone::RAISED));
        }
    }
}
