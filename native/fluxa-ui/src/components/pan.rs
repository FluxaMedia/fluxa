use crate::gfx::{ScrollArea, Ui, UiBuilder};
use crate::gfx::{ScrollBarVisibility, ScrollOutput, ScrollSource};
use std::hash::Hash;

pub fn pan_area<R>(
    ui: &mut Ui,
    id: impl Hash + std::fmt::Debug,
    configure: impl FnOnce(ScrollArea) -> ScrollArea,
    add: impl FnOnce(&mut Ui) -> R,
) -> ScrollOutput<R> {
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

pub fn pan_row<R>(
    ui: &mut Ui,
    id: impl Hash + std::fmt::Debug,
    configure: impl FnOnce(ScrollArea) -> ScrollArea,
    add: impl FnOnce(&mut Ui) -> R,
) -> ScrollOutput<R> {
    let mut bounds = ui.available_rect_before_wrap();
    bounds.max.y = f32::INFINITY;
    let mut output = ui
        .scope_builder(UiBuilder::new().max_rect(bounds), |ui| {
            pan_area(ui, id, configure, add)
        })
        .inner;
    output.inner_rect.max.y = output
        .inner_rect
        .max
        .y
        .min(output.inner_rect.min.y + output.content_size.y);
    output
}

pub fn pan_strip<R>(
    ui: &mut Ui,
    name: &'static str,
    add: impl FnOnce(&mut Ui) -> R,
) -> ScrollOutput<R> {
    let offset = crate::scroll_region::strip_offset(ui.ctx(), name);
    let output = pan_row(
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
