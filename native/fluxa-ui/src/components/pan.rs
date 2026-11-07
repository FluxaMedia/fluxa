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

pub fn pan_strip<R>(
    ui: &mut Ui,
    name: &'static str,
    add: impl FnOnce(&mut Ui) -> R,
) -> ScrollAreaOutput<R> {
    let offset = crate::scroll_region::strip_offset(ui.ctx(), name);
    let output = pan_area(
        ui,
        name,
        |area| {
            area.horizontal_scroll_offset(offset)
                .scroll_source(ScrollSource::NONE)
        },
        add,
    );
    crate::report_scroll_region(
        ui.ctx(),
        crate::ScrollRegion {
            id: crate::ScrollId::Strip(name),
            axis: crate::ScrollAxis::Horizontal,
            rect: output.inner_rect,
            max: (output.content_size.x - output.inner_rect.width()).max(0.0),
        },
    );
    output
}
