use super::*;
use egui::{UiBuilder, scroll_area::ScrollSource};

pub(crate) fn page<R>(
    context: &egui::Context,
    viewport: Viewport,
    name: &'static str,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let metrics = UiMetrics::for_viewport(viewport);
    provide(context, metrics);
    let margin = crate::layout::screen_margin(metrics);
    let rect = Rect::from_min_max(
        Pos2::new(margin, metrics.content_header_top),
        Pos2::new(viewport.width - margin, viewport.height),
    );
    egui::Area::new(Id::new(("page", name)))
        .constrain(false)
        .fixed_pos(rect.min)
        .show(context, |ui| {
            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.set_width(rect.width());
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                add(ui)
            })
            .inner
        })
        .inner
}

pub(crate) fn scroll_body<R>(
    ui: &mut Ui,
    viewport: Viewport,
    name: &'static str,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    scroll_body_keyed(ui, viewport, name, (), add)
}

pub(crate) fn scroll_body_keyed<R>(
    ui: &mut Ui,
    viewport: Viewport,
    name: &'static str,
    key: impl std::hash::Hash + std::fmt::Debug,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let context = ui.ctx().clone();
    let key = Id::new(("page-scroll", name)).with(key);
    let max_id = key.with("max");
    let last_max = context
        .data(|data| data.get_temp::<f32>(max_id))
        .unwrap_or(0.0);
    let offset = crate::layout::resolve_screen_scroll(&context, viewport, key, last_max);
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
            ui.add_space(crate::layout::mobile_scroll_reserve(viewport));
            value
        },
    );
    let max = (output.content_size.y - output.inner_rect.height()).max(0.0);
    context.data_mut(|data| {
        data.insert_temp(max_id, max);
        data.insert_temp(Id::new(("page-scroll-max", name)), max);
        data.insert_temp(Id::new(("page-scroll-offset", name)), offset);
    });
    output.inner
}

pub(crate) fn scroll_state(context: &egui::Context, name: &'static str) -> Option<(f32, f32)> {
    context.data(|data| {
        Some((
            data.get_temp::<f32>(Id::new(("page-scroll-offset", name)))?,
            data.get_temp::<f32>(Id::new(("page-scroll-max", name)))?,
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(context: &egui::Context, scroll_y: f32) -> (f32, f32) {
        let mut viewport = Viewport::new(400, 400, UiFormFactor::Mobile);
        viewport.scroll_y = scroll_y;
        let mut top = 0.0;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            ..Default::default()
        };
        context
            .run_ui(input, |_| {
                page(context, viewport, "test", |ui| {
                    scroll_body(ui, viewport, "test", |ui| {
                        top = ui.allocate_space(Vec2::new(50.0, 1000.0)).1.top();
                    });
                });
            })
            .drop_without_applying_deltas();
        let max = context
            .data(|data| data.get_temp::<f32>(Id::new(("page-scroll-max", "test"))))
            .unwrap_or(0.0);
        (top, max)
    }

    #[test]
    fn body_scrolls_by_the_shell_offset_and_measures_its_own_max() {
        let context = egui::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let (rest, max) = frame(&context, 0.0);
        assert!(max > 500.0);
        let (_, max) = frame(&context, 0.0);
        let (scrolled, _) = frame(&context, 100.0);
        assert!((rest - scrolled - 100.0).abs() < 1.0);
        assert!(max > 0.0);
    }
}
