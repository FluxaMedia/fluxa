use crate::gfx::{Pos2, Rect, ScrollArea, Ui, UiBuilder, Vec2};
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

pub(crate) fn pan_scrolled<R>(
    ui: &mut Ui,
    id: crate::ScrollId,
    salt: impl Hash + std::fmt::Debug,
    target: f32,
    eased: bool,
    bleed: bool,
    add: impl FnOnce(&mut Ui) -> R,
) -> ScrollOutput<R> {
    let shown = crate::layout::resolve_scroll(ui.ctx(), id, target, eased);
    let column = ui.available_rect_before_wrap();
    let screen = ui.ctx().content_rect();
    let left = if bleed {
        (column.min.x - screen.min.x).max(0.0)
    } else {
        0.0
    };
    let right = if bleed {
        (screen.max.x - column.max.x).max(0.0)
    } else {
        0.0
    };
    let bleed = Rect::from_min_max(
        Pos2::new(column.min.x - left, column.min.y),
        Pos2::new(column.max.x + right, f32::INFINITY),
    );
    let mut row = ui.new_child(UiBuilder::new().max_rect(bleed).layout(*ui.layout()));
    let mut output = pan_row(
        &mut row,
        salt,
        |area| {
            area.horizontal_scroll_offset(shown)
                .scroll_source(ScrollSource::NONE)
        },
        |ui| {
            let body = Rect::from_min_max(
                Pos2::new(ui.max_rect().min.x + left, ui.max_rect().min.y),
                ui.max_rect().max,
            );
            let inner = ui.scope_builder(UiBuilder::new().max_rect(body), add).inner;
            ui.expand_to_include_rect(crate::flex::trailing_point(ui.min_rect(), right));
            inner
        },
    );
    ui.allocate_rect(
        crate::flex::with_height(column, output.inner_rect.height()),
        crate::gfx::Sense::hover(),
    );
    output.inner_rect = Rect::from_min_max(
        Pos2::new(output.inner_rect.min.x + left, output.inner_rect.min.y),
        Pos2::new(output.inner_rect.max.x - right, output.inner_rect.max.y),
    );
    crate::report_scroll_region(
        ui.ctx(),
        crate::ScrollRegion {
            id,
            axis: crate::ScrollAxis::Horizontal,
            rect: output.inner_rect,
            max: (output.content_size.x - (output.inner_rect.width() + left + right)).max(0.0),
        },
    );
    output
}

pub fn pan_strip<R>(
    ui: &mut Ui,
    name: &'static str,
    add: impl FnOnce(&mut Ui) -> R,
) -> ScrollOutput<R> {
    let offset = crate::scroll_region::strip_offset(ui.ctx(), name);
    pan_scrolled(
        ui,
        crate::ScrollId::Strip(name),
        name,
        offset,
        true,
        true,
        add,
    )
}

pub fn pan_strip_within<R>(
    ui: &mut Ui,
    name: &'static str,
    add: impl FnOnce(&mut Ui) -> R,
) -> ScrollOutput<R> {
    let offset = crate::scroll_region::strip_offset(ui.ctx(), name);
    pan_scrolled(
        ui,
        crate::ScrollId::Strip(name),
        name,
        offset,
        true,
        false,
        add,
    )
}
