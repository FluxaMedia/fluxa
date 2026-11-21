use super::*;
use crate::components::{
    ListRow, Trailing, bottom_sheet, chip, labeled_button, list_row, primary_button,
    secondary_button, segmented, side_drawer, stepper, swatch, switch,
};
use crate::flex::{Insets, Justify, column, leaf, row, text};

mod posters;

use crate::kit::{self, layout_items};
use std::cell::RefCell;

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

fn heading<'a>(ui: &mut crate::gfx::Ui, text: &'a str) -> kit::Item<'a> {
    let size = crate::vars::px("text-subtitle");
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
    if let Some(variant) = scene.strip_prefix("ticket") {
        draw_ticket_scene(context, viewport, variant, assets);
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
                    let options: Vec<(String, String)> = ["English", "Turkish", "Japanese"]
                        .iter()
                        .map(|label| (label.to_lowercase(), (*label).to_owned()))
                        .collect();
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
                    items.push(heading(ui, "Dropdown"));
                    items.push(kit::widget(ui, |ui| {
                        crate::components::dropdown(
                            ui,
                            "gallery-dropdown",
                            "english",
                            &options,
                            Space::S48.px() * 5.0,
                            false,
                        );
                    }));
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
                    items.push(heading(ui, "Overlays"));
                    items.push(
                        layout_items::row(vec![
                            kit::widget(ui, |ui| {
                                if primary_button(ui, "Bottom sheet", 160.0, true).clicked() {
                                    state.borrow_mut().sheet = true;
                                }
                            }),
                            kit::widget(ui, |ui| {
                                if secondary_button(ui, "Side drawer", 160.0).clicked() {
                                    state.borrow_mut().drawer = true;
                                }
                            }),
                        ])
                        .gap(gap),
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
    let mut state = state.into_inner();
    if bottom_sheet(
        context,
        "fluxa-gallery-sheet",
        state.sheet,
        viewport,
        |ui| {
            let title = RichText::new("Playback")
                .size(crate::vars::px("text-title"))
                .strong()
                .color(text_primary());
            let buttons = labels
                .iter()
                .enumerate()
                .map(|(index, label)| {
                    kit::widget(ui, move |ui| {
                        labeled_button(ui, icons[index], label, index == 1);
                    })
                })
                .collect();
            let items = vec![
                kit::widget(ui, |ui| {
                    ui.label(title.clone());
                }),
                layout_items::space(ui, Space::S12),
                layout_items::row(buttons).gap(gap).wrap(),
            ];
            kit::lay(ui, items);
        },
    ) {
        state.sheet = false;
    }
    if side_drawer(
        context,
        "fluxa-gallery-drawer",
        state.drawer,
        viewport,
        |ui| {
            ui.label(
                RichText::new("Details")
                    .size(crate::vars::px("text-title"))
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

fn draw_ticket_scene(
    context: &crate::gfx::Context,
    viewport: Viewport,
    variant: &str,
    assets: &mut impl HomeAssets,
) {
    let member = |name: &str, host: bool, you: bool| crate::WatchMemberView {
        id: name.to_owned(),
        name: name.to_owned(),
        host,
        you,
        ..Default::default()
    };
    let joined = variant.contains("joined")
        || (variant.contains("tear") && context.input(|input| input.time()) > 1.0);
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
        crate::components::watch_ticket(ui, &ticket, &mut code, width, "en", assets);
    });
}
