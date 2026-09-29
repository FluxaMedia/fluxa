use egui::{Id, LayerId, Vec2, emath::TSTransform, epaint::shape_transform::adjust_colors};

const PAGE_SECONDS: f32 = 0.26;

pub fn page_transition(context: &egui::Context, key: u64, keep: &[Id]) -> f32 {
    let state = Id::new("fluxa-page-transition");
    let dt = context.input(|input| input.stable_dt).min(1.0 / 30.0);
    let (last, progress) = context
        .data(|data| data.get_temp::<(u64, f32)>(state))
        .unwrap_or((key, 1.0));
    let linear = if last == key {
        (progress + dt / PAGE_SECONDS).min(1.0)
    } else {
        0.0
    };
    context.data_mut(|data| data.insert_temp(state, (key, linear)));
    if linear >= 1.0 {
        return linear;
    }
    let t = egui::emath::easing::cubic_out(linear);
    let alpha = t;
    let shift = TSTransform::from_translation(Vec2::new(0.0, (1.0 - t) * 14.0));
    let layers: Vec<LayerId> = context.memory(|memory| memory.layer_ids().collect());
    context.graphics_mut(|graphics| {
        for layer in layers {
            if keep.contains(&layer.id) {
                continue;
            }
            let Some(list) = graphics.get_mut(layer) else {
                continue;
            };
            let end = list.next_idx();
            list.transform_range(egui::layers::ShapeIdx(0), end, shift);
            for index in 0..end.0 {
                list.mutate_shape(egui::layers::ShapeIdx(index), |clipped| {
                    let alpha = alpha;
                    adjust_colors(&mut clipped.shape, move |color| {
                        *color = color.gamma_multiply(alpha)
                    });
                });
            }
        }
    });
    linear
}

pub(crate) fn press_scale(painter: &egui::Painter, rect: egui::Rect, draw: impl FnOnce()) {
    let context = painter.ctx();
    let layer = painter.layer_id();
    let start = context.graphics_mut(|graphics| graphics.entry(layer).next_idx());
    draw();
    let held = context.input(|input| {
        input.pointer.primary_down()
            && input.pointer.press_origin().is_some_and(|origin| {
                rect.contains(origin)
                    && input
                        .pointer
                        .interact_pos()
                        .is_some_and(|pos| pos.distance(origin) < 12.0)
            })
    });
    let id = Id::new((
        "fluxa-press",
        rect.min.x as i32,
        rect.min.y as i32,
        layer.id,
    ));
    let t =
        context.animate_bool_with_time_and_easing(id, held, 0.14, egui::emath::easing::cubic_out);
    if t <= 0.0 {
        return;
    }
    let center = rect.center().to_vec2();
    let scale = TSTransform::from_translation(center)
        * TSTransform::from_scaling(1.0 - 0.045 * t)
        * TSTransform::from_translation(-center);
    context.graphics_mut(|graphics| {
        let list = graphics.entry(layer);
        let end = list.next_idx();
        list.transform_range(start, end, scale);
    });
}
