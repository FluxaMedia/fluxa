use super::*;
use crate::gfx::{ScrollSource, UiBuilder};

pub(crate) fn screen_body<R>(
    context: &crate::gfx::Context,
    viewport: Viewport,
    name: &'static str,
    key: impl std::hash::Hash + std::fmt::Debug,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    screen_body_below(context, viewport, name, key, 0.0, add)
}

pub(crate) fn screen_body_below<R>(
    context: &crate::gfx::Context,
    viewport: Viewport,
    name: &'static str,
    key: impl std::hash::Hash + std::fmt::Debug,
    top: f32,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    body_at(context, viewport, name, key, top, true, add)
}

pub(crate) fn overlay_body<R>(
    context: &crate::gfx::Context,
    viewport: Viewport,
    name: &'static str,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    body_at(context, viewport, name, (), 0.0, false, add)
}

fn body_at<R>(
    context: &crate::gfx::Context,
    viewport: Viewport,
    name: &'static str,
    key: impl std::hash::Hash + std::fmt::Debug,
    top: f32,
    scrolls: bool,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    crate::metrics::enter(viewport);
    let rect = Rect::from_min_max(
        Pos2::new(0.0, top),
        Pos2::new(viewport.width, viewport.height),
    );
    crate::gfx::Area::new(Id::new(("screen", name)))
        .constrain(false)
        .fixed_pos(rect.min)
        .order(crate::gfx::Order::Background)
        .show(context, |ui| {
            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.set_width(rect.width());
                ui.set_item_spacing(Vec2::ZERO);
                scroll_body_inner(ui, viewport, name, key, scrolls, add)
            })
            .inner
        })
        .inner
}

fn scroll_body_inner<R>(
    ui: &mut Ui,
    viewport: Viewport,
    name: &'static str,
    key: impl std::hash::Hash + std::fmt::Debug,
    scrolls: bool,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let context = ui.ctx().clone();
    let key = Id::new(("page-scroll", name)).with(key);
    let max_id = key.with("max");
    let last_max = context
        .data(|data| data.get_temp::<f32>(max_id))
        .unwrap_or(0.0);
    let offset = if scrolls {
        crate::layout::resolve_screen_scroll(&context, viewport, key, last_max)
    } else {
        0.0
    };
    let width = ui.available_width();
    let output = crate::components::pan_area(
        ui,
        name,
        |area| {
            area.scroll_source(ScrollSource::NONE)
                .auto_shrink([false, false])
                .vertical_scroll_offset(offset)
        },
        |ui| {
            ui.set_width(width);
            let value = add(ui);
            let reserve =
                (crate::layout::mobile_scroll_reserve(viewport) - viewport.safe_bottom).max(0.0);
            lay(ui, vec![item(Vec2::new(0.0, reserve), |_, _| {})]);
            value
        },
    );
    if scrolls {
        let max = (output.content_size.y - output.inner_rect.height()).max(0.0);
        context.data_mut(|data| {
            data.insert_temp(max_id, max);
            data.insert_temp(Id::new(("page-scroll-max", name)), max);
            data.insert_temp(Id::new(("page-scroll-offset", name)), offset);
        });
    }
    output.inner
}

pub(crate) fn scroll_state(
    context: &crate::gfx::Context,
    name: &'static str,
) -> Option<(f32, f32)> {
    context.data(|data| {
        Some((
            data.get_temp::<f32>(Id::new(("page-scroll-offset", name)))?,
            data.get_temp::<f32>(Id::new(("page-scroll-max", name)))?,
        ))
    })
}
