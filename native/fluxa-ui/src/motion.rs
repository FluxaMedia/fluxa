use egui::{Id, LayerId, Vec2, emath::TSTransform, epaint::shape_transform::adjust_colors};

const PAGE_SECONDS: f32 = 0.26;

pub(crate) fn fast() -> f32 {
    crate::metrics::motion_seconds(0)
}

pub(crate) fn normal() -> f32 {
    crate::metrics::motion_seconds(1)
}

pub(crate) fn slow() -> f32 {
    crate::metrics::motion_seconds(2)
}

#[derive(Clone, Copy)]
pub(crate) enum Ease {
    Linear,
    Out,
}

#[derive(Clone, Copy)]
struct Tween {
    from: f32,
    to: f32,
    t: f32,
    frame: u64,
}

impl Tween {
    fn value(&self, ease: Ease) -> f32 {
        let t = match ease {
            Ease::Linear => self.t,
            Ease::Out => egui::emath::easing::cubic_out(self.t),
        };
        self.from + (self.to - self.from) * t
    }
}

pub(crate) fn ease(context: &egui::Context, id: Id, target: f32, seconds: f32, ease: Ease) -> f32 {
    let frame = context.cumulative_frame_nr();
    let mut tween = context
        .data(|data| data.get_temp::<Tween>(id))
        .unwrap_or(Tween {
            from: target,
            to: target,
            t: 1.0,
            frame,
        });
    if tween.to != target {
        tween.from = tween.value(ease);
        tween.to = target;
        tween.t = 0.0;
        tween.frame = frame;
    } else if tween.t < 1.0 && tween.frame != frame {
        let dt = context.input(|input| input.stable_dt).min(1.0 / 30.0);
        tween.t = (tween.t + dt / seconds.max(0.001)).min(1.0);
        tween.frame = frame;
    }
    if tween.t < 1.0 {
        context.request_repaint();
    }
    context.data_mut(|data| data.insert_temp(id, tween));
    tween.value(ease)
}

pub(crate) fn appear(context: &egui::Context, id: Id, seconds: f32, curve: Ease) -> f32 {
    if context.data(|data| data.get_temp::<Tween>(id)).is_none() {
        let tween = Tween {
            from: 0.0,
            to: 1.0,
            t: 0.0,
            frame: context.cumulative_frame_nr(),
        };
        context.data_mut(|data| data.insert_temp(id, tween));
    }
    ease(context, id, 1.0, seconds, curve)
}

pub(crate) fn reset(context: &egui::Context, id: Id) {
    context.data_mut(|data| data.remove::<Tween>(id));
}

pub(crate) fn toggle(context: &egui::Context, id: Id, on: bool, seconds: f32) -> f32 {
    ease(
        context,
        id,
        if on { 1.0 } else { 0.0 },
        seconds,
        Ease::Linear,
    )
}

pub(crate) fn presence(context: &egui::Context, id: Id, shown: bool, seconds: f32) -> f32 {
    ease(
        context,
        id,
        if shown { 1.0 } else { 0.0 },
        seconds,
        Ease::Out,
    )
}

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
    let t = presence(context, id, held, fast());
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
