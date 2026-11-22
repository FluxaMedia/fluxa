use super::*;
use crate::components::{
    ListRow, Trailing, chip, labeled_button, list_row, segmented, stepper, swatch, switch,
};
use crate::flex::{Insets, Justify, column, leaf, row, text};

mod posters;

use crate::kit::{self, layout_items};
use std::cell::RefCell;

#[derive(Clone)]
struct State {
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

fn heading<'a>(ui: &mut crate::gfx::Ui, text: &'a str) -> kit::Item<'a> {
    let size = crate::vars::px("text-lg");
    let color = crate::metrics::theme().text_muted;
    layout_items::column(vec![
        layout_items::gap(Space::S8.px()),
        kit::widget(ui, move |ui| {
            ui.label(RichText::new(text).size(size).strong().color(color));
        }),
    ])
}

pub fn draw_gallery(
    context: &crate::gfx::Context,
    viewport: Viewport,
    scene: &str,
    assets: &mut impl HomeAssets,
) {
    crate::metrics::enter(viewport);
    let screen = Rect::from_min_max(Pos2::ZERO, Pos2::new(viewport.width, viewport.height));
    context
        .layer_painter(crate::gfx::LayerId::background())
        .rect_filled(screen, 0.0, background());
    if scene == "posters" {
        posters::draw(context, viewport);
        return;
    }
    let state_id = Id::new("fluxa-gallery");
    let mut state = context
        .data(|data| data.get_temp::<State>(state_id))
        .unwrap_or_default();
    if !state.started {
        state.started = true;
        state.focused = (scene == "focus").then_some(11);
    }
    let icons = ["Settings", "Plus", "Lock", "Edit"].map(|name| assets.icon(name));
    let labels = ["Audio", "Subtitles", "Speed", "A very long label"];
    let safe = viewport.safe_area();
    let gap = Space::S12.px();
    crate::begin_focus_frame(context, state.focused);
    let state = RefCell::new(state);
    crate::gfx::Area::new(Id::new("fluxa-gallery-body"))
        .fixed_pos(Pos2::new(safe.left + Space::S24.px(), safe.top))
        .show(context, |ui| {
            let size = Vec2::new(
                viewport.width - safe.left - safe.right - Space::S24.px() * 2.0,
                viewport.height - safe.top - safe.bottom,
            );
            ui.set_max_size(size);
            crate::components::pan_area(
                ui,
                "fluxa-gallery-scroll",
                |area| area.auto_shrink([false, false]),
                |ui| {
                    let state = &state;
                    ui.set_width(size.x);
                    ui.set_item_spacing(Vec2::splat(gap));
                    let mut items = vec![layout_items::gap(Space::S24.px())];
                    items.push(heading(ui, "LabeledButton"));
                    items.push(
                        layout_items::row(
                            labels
                                .iter()
                                .enumerate()
                                .map(|(index, label)| {
                                    kit::widget(ui, move |ui| {
                                        let selected = state.borrow().labeled == index;
                                        let response = labeled_button(
                                            ui,
                                            icons[index],
                                            label,
                                            selected,
                                        );
                                        if !crate::kit::probing() {
                                            response.clone().focus_node(ui, 10 + index as u64);
                                        }
                                        if response.clicked() {
                                            state.borrow_mut().labeled = index;
                                        }
                                    })
                                })
                                .collect(),
                        )
                        .gap(gap)
                        .wrap(),
                    );
                    items.push(heading(ui, "Chip"));
                    items.push(
                        layout_items::row(
                            ["All", "Movies", "Series", "Anime", "Live"]
                                .iter()
                                .enumerate()
                                .map(|(index, label)| {
                                    kit::widget(ui, move |ui| {
                                        let selected = state.borrow().chip == index;
                                        if chip(ui, label, selected).clicked() {
                                            state.borrow_mut().chip = index;
                                        }
                                    })
                                })
                                .collect(),
                        )
                        .gap(gap)
                        .wrap(),
                    );
                    items.push(heading(ui, "Segmented"));
                    items.push(kit::widget(ui, |ui| {
                        let current = state.borrow().segment;
                        if let Some(next) = segmented(
                            ui,
                            Id::new("gallery-segmented"),
                            &["Off", "Auto", "On"],
                            current,
                        ) {
                            state.borrow_mut().segment = next;
                        }
                    }));
                    items.push(heading(ui, "Stepper"));
                    items.push(kit::widget(ui, |ui| {
                        let steps = state.borrow().steps;
                        let label = format!("{steps}");
                        if let Some(delta) = stepper(
                            ui,
                            Id::new("gallery-stepper"),
                            &label,
                            steps > 0,
                            steps < 9,
                        ) {
                            state.borrow_mut().steps += delta;
                        }
                    }));
                    items.push(heading(ui, "Switch and ListRow"));
                    for (index, title) in ["Autoplay next", "Skip intros"].iter().enumerate() {
                        items.push(kit::widget(ui, move |ui| {
                            let on = state.borrow().switches[index];
                            let response = list_row(
                                ui,
                                ListRow {
                                    icon: icons[index],
                                    title,
                                    subtitle: Some("Starts the next episode without asking"),
                                    trailing: Trailing::Switch(on),
                                },
                            );
                            if response.clicked() {
                                state.borrow_mut().switches[index] = !on;
                            }
                        }));
                    }
                    items.push(kit::widget(ui, |ui| {
                        list_row(
                            ui,
                            ListRow {
                                icon: icons[0],
                                title: "Subtitle language",
                                subtitle: None,
                                trailing: Trailing::Value("Türkçe"),
                            },
                        );
                    }));
                    items.push(kit::widget(ui, |ui| {
                        list_row(
                            ui,
                            ListRow {
                                icon: None,
                                title: "A row title long enough to be cut off before it reaches the trailing chevron",
                                subtitle: None,
                                trailing: Trailing::Chevron,
                            },
                        );
                    }));
                    items.push(
                        layout_items::row(vec![
                            kit::widget(ui, |ui| {
                                switch(ui, Id::new("gallery-switch-a"), true);
                            }),
                            kit::widget(ui, |ui| {
                                switch(ui, Id::new("gallery-switch-b"), false);
                            }),
                        ])
                        .gap(gap),
                    );
                    items.push(heading(ui, "Swatch"));
                    items.push(
                        layout_items::row(
                            SWATCHES
                                .iter()
                                .enumerate()
                                .map(|(index, color)| {
                                    kit::widget(ui, move |ui| {
                                        let selected = state.borrow().swatch == index;
                                        if swatch(ui, *color, selected).clicked() {
                                            state.borrow_mut().swatch = index;
                                        }
                                    })
                                })
                                .collect(),
                        )
                        .gap(gap)
                        .wrap(),
                    );
                    items.push(heading(ui, "Layout tree"));
                    items.push(kit::item(Vec2::new(360.0, 120.0), |ui, rect| {
                        paint_layout_sample(ui.painter(), rect);
                    }));
                    items.push(layout_items::gap(Space::S48.px() + viewport.safe_bottom));
                    kit::lay(ui, items);
                },
            );
        });
    let state = state.into_inner();
    let _ = crate::take_focus_targets(context);
    context.data_mut(|data| data.insert_temp(state_id, state));
}

fn paint_layout_sample(painter: &crate::gfx::Painter, rect: Rect) {
    let font = FontId::proportional(crate::vars::px("labeled-button-text"));
    let tree = column([
        row([
            leaf(Vec2::splat(32.0)).id(0),
            text(painter, "Title row", &font).id(1),
            leaf(Vec2::ZERO).grow(1.0),
            leaf(Vec2::new(48.0, 24.0)).id(2),
        ])
        .gap(Space::S8.px())
        .align(Align::Center),
        row((3..8).map(|i| leaf(Vec2::new(54.0, 28.0)).id(i)))
            .gap(Space::S6.px())
            .line_gap(Space::S6.px())
            .wrap(),
    ])
    .gap(Space::S10.px())
    .align(Align::Stretch)
    .padding(Insets::all(Space::S12.px()))
    .justify(Justify::Start);
    painter.rect_stroke(
        rect,
        radius::control(),
        crate::gfx::Stroke::new(stroke::hairline(), crate::metrics::theme().border),
        crate::gfx::StrokeKind::Inside,
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
            painter.rect_filled(slot, radius::chip(), crate::stylesheet::color("raised"));
        }
    }
}
