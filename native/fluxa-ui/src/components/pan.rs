use egui::scroll_area::{ScrollAreaOutput, ScrollBarVisibility, ScrollSource};
use egui::{ScrollArea, Ui};
use std::hash::Hash;

pub fn pan_area<R>(
    ui: &mut Ui,
    id: impl Hash + std::fmt::Debug,
    configure: impl FnOnce(ScrollArea) -> ScrollArea,
    add: impl FnOnce(&mut Ui) -> R,
) -> ScrollAreaOutput<R> {
    let width = ui.available_width();
    let area = ScrollArea::both()
        .id_salt(id)
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .scroll_source(ScrollSource::ALL);
    configure(area).show(ui, |ui| {
        ui.set_max_width(width);
        add(ui)
    })
}
