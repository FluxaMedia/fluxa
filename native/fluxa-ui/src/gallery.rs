use super::*;
use crate::components::{
    ListRow, Trailing, bottom_sheet, chip, labeled_button, list_row, primary_button,
    secondary_button, segmented, side_drawer, stepper, swatch, switch,
};
use crate::flex::{Insets, Justify, column, leaf, row, text};

mod posters;

#[derive(Clone)]
struct State {
    sheet: bool,
    drawer: bool,
    labeled: usize,
    chip: usize,
    segment: usize,
    steps: i32,
    switches: [bool; 2],
    swatch: usize,
    focused: Option<u64>,
    started: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            sheet: false,
            drawer: false,
            labeled: 0,
            chip: 1,
            segment: 1,
            steps: 3,
            switches: [true, false],
            swatch: 2,
            focused: None,
            started: false,
        }
    }
}

const SWATCHES: [Color32; 6] = [
    Color32::from_rgb(255, 255, 255),
    Color32::from_rgb(255, 214, 10),
    Color32::from_rgb(255, 107, 107),
    Color32::from_rgb(10, 132, 255),
    Color32::from_rgb(48, 209, 88),
    Color32::from_rgb(191, 90, 242),
];

fn heading(ui: &mut egui::Ui, text: &str, metrics: &UiMetrics) {
    ui.add_space(Space::S8.px(metrics));
    ui.label(
        RichText::new(text)
            .size(metrics.text.subtitle)
            .strong()
            .color(metrics.text_muted),
    );
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
    if scene == "posters" {
        posters::draw(context, viewport, metrics);
        return;
    }
    if let Some(variant) = scene.strip_prefix("ticket") {
        draw_ticket_scene(context, viewport, variant, assets, metrics);
        return;
    }
    let state_id = Id::new("fluxa-gallery");
    let mut state = context
        .data(|data| data.get_temp::<State>(state_id))
        .unwrap_or_default();
    if !state.started {
        state.started = true;
        state.sheet = scene == "sheet";
        state.drawer = scene == "drawer";
        state.focused = (scene == "focus").then_some(11);
    }
    let icons = ["Settings", "Plus", "Lock", "Edit"].map(|name| assets.icon(name));
    let labels = ["Audio", "Subtitles", "Speed", "A very long label"];
    let safe = viewport.safe_area();
    let gap = Space::S12.px(&metrics);
    crate::begin_focus_frame(context, state.focused);
    egui::Area::new(Id::new("fluxa-gallery-body"))
        .fixed_pos(Pos2::new(safe.left + Space::S24.px(&metrics), safe.top))
        .show(context, |ui| {
            let size = Vec2::new(
                viewport.width - safe.left - safe.right - Space::S24.px(&metrics) * 2.0,
                viewport.height - safe.top - safe.bottom,
            );
            ui.set_max_size(size);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(size.x);
                    ui.spacing_mut().item_spacing = Vec2::splat(gap);
                    ui.add_space(Space::S24.px(&metrics));
                    heading(ui, "LabeledButton", &metrics);
                    ui.horizontal_wrapped(|ui| {
                        for (index, label) in labels.iter().enumerate() {
                            let response = labeled_button(
                                ui,
                                icons[index],
                                label,
                                state.labeled == index,
                                metrics,
                            )
                            .focus_node(ui, 10 + index as u64, &metrics);
                            if response.clicked() {
                                state.labeled = index;
                            }
                        }
                    });
                    heading(ui, "Chip", &metrics);
                    ui.horizontal_wrapped(|ui| {
                        for (index, label) in ["All", "Movies", "Series", "Anime", "Live"]
                            .iter()
                            .enumerate()
                        {
                            if chip(ui, label, state.chip == index, metrics).clicked() {
                                state.chip = index;
                            }
                        }
                    });
                    heading(ui, "Segmented", &metrics);
                    if let Some(next) = segmented(
                        ui,
                        Id::new("gallery-segmented"),
                        &["Off", "Auto", "On"],
                        state.segment,
                        metrics,
                    ) {
                        state.segment = next;
                    }
                    heading(ui, "Stepper", &metrics);
                    let label = format!("{}", state.steps);
                    if let Some(delta) = stepper(
                        ui,
                        Id::new("gallery-stepper"),
                        &label,
                        state.steps > 0,
                        state.steps < 9,
                        metrics,
                    ) {
                        state.steps += delta;
                    }
                    heading(ui, "Switch and ListRow", &metrics);
                    for (index, title) in ["Autoplay next", "Skip intros"].iter().enumerate() {
                        let response = list_row(
                            ui,
                            ListRow {
                                icon: icons[index],
                                title,
                                subtitle: Some("Starts the next episode without asking"),
                                trailing: Trailing::Switch(state.switches[index]),
                            },
                            metrics,
                        );
                        if response.clicked() {
                            state.switches[index] = !state.switches[index];
                        }
                    }
                    list_row(
                        ui,
                        ListRow {
                            icon: icons[0],
                            title: "Subtitle language",
                            subtitle: None,
                            trailing: Trailing::Value("Türkçe"),
                        },
                        metrics,
                    );
                    list_row(
                        ui,
                        ListRow {
                            icon: None,
                            title: "A row title long enough to be cut off before it reaches the trailing chevron",
                            subtitle: None,
                            trailing: Trailing::Chevron,
                        },
                        metrics,
                    );
                    ui.horizontal(|ui| {
                        switch(ui, Id::new("gallery-switch-a"), true, metrics);
                        switch(ui, Id::new("gallery-switch-b"), false, metrics);
                    });
                    heading(ui, "Swatch", &metrics);
                    ui.horizontal_wrapped(|ui| {
                        for (index, color) in SWATCHES.iter().enumerate() {
                            if swatch(ui, *color, state.swatch == index, metrics).clicked() {
                                state.swatch = index;
                            }
                        }
                    });
                    heading(ui, "Overlays", &metrics);
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Bottom sheet", 160.0, true).clicked() {
                            state.sheet = true;
                        }
                        if secondary_button(ui, "Side drawer", 160.0).clicked() {
                            state.drawer = true;
                        }
                    });
                    heading(ui, "Layout tree", &metrics);
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(360.0, 120.0), Sense::hover());
                    paint_layout_sample(ui.painter(), rect, metrics);
                    ui.add_space(Space::S48.px(&metrics) + viewport.safe_bottom);
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
    let _ = crate::take_focus_targets(context);
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
        egui::Stroke::new(stroke::hairline(), metrics.border),
        egui::StrokeKind::Inside,
    );
    let placed = tree.solve(rect);
    for (id, slot) in placed.tagged() {
        if id == 1 {
            crate::flex::paint_text(
                painter,
                slot,
                "Title row",
                &font,
                text_primary(),
                Align2::LEFT_CENTER,
            );
        } else {
            painter.rect_filled(slot, radius::chip(), ink(tone::raised()));
        }
    }
}

fn draw_ticket_scene(
    context: &egui::Context,
    viewport: Viewport,
    variant: &str,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
) {
    let member = |name: &str, host: bool, you: bool| crate::WatchMemberView {
        id: name.to_owned(),
        name: name.to_owned(),
        host,
        you,
        ..Default::default()
    };
    let joined = variant.contains("joined")
        || (variant.contains("tear") && context.input(|input| input.time) > 1.0);
    if variant.contains("tear") {
        context.request_repaint();
    }
    let ticket = crate::WatchTicket {
        room: joined.then(|| "K7Q4M2".to_owned()),
        title: joined.then(|| "Andor".to_owned()),
        episode: joined.then(|| "S1 · E4 · Aldhani".to_owned()),
        backdrop: variant
            .contains("poster")
            .then(|| "fixture-backdrop".to_owned()),
        host: joined.then(|| member("Emma", true, false)),
        members: if joined {
            vec![
                member("Emma", true, false),
                member("Marcus", false, false),
                member("Zoe", false, false),
                member("You", false, true),
            ]
        } else {
            Vec::new()
        },
        max_members: 6,
        ..Default::default()
    };
    let width = (viewport.width - 48.0).min(560.0);
    let origin = Pos2::new(
        (viewport.width - width) * 0.5,
        (viewport.height - crate::components::ticket_height(width)) * 0.5,
    );
    let mut code = if joined {
        String::new()
    } else {
        "K7Q".to_owned()
    };
    crate::components::placed_front(context, Id::new("fluxa-gallery-ticket"), origin, |ui| {
        crate::components::watch_ticket(ui, &ticket, &mut code, width, "en", assets, metrics);
    });
}
