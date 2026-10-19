use egui::{Id, LayerId, Vec2, emath::TSTransform, epaint::shape_transform::adjust_colors};

const PAGE_SECONDS: f32 = 0.26;

static CURVE: [std::sync::atomic::AtomicU32; 4] = [
    std::sync::atomic::AtomicU32::new(0x3E4C_CCCD),
    std::sync::atomic::AtomicU32::new(0),
    std::sync::atomic::AtomicU32::new(0),
    std::sync::atomic::AtomicU32::new(0x3F80_0000),
];

pub(crate) fn set_curve(points: [f32; 4]) {
    for (cell, value) in CURVE.iter().zip(points) {
        cell.store(value.to_bits(), std::sync::atomic::Ordering::Relaxed);
    }
}

fn curve() -> [f32; 4] {
    std::array::from_fn(|i| f32::from_bits(CURVE[i].load(std::sync::atomic::Ordering::Relaxed)))
}

fn bezier(t: f32) -> f32 {
    let [x1, y1, x2, y2] = curve();
    let at = |a: f32, b: f32, u: f32| {
        3.0 * a * u * (1.0 - u).powi(2) + 3.0 * b * u.powi(2) * (1.0 - u) + u.powi(3)
    };
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    for _ in 0..24 {
        let mid = (low + high) * 0.5;
        if at(x1, x2, mid) < t {
            low = mid;
        } else {
            high = mid;
        }
    }
    at(y1, y2, (low + high) * 0.5)
}

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
    Token,
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
            Ease::Token => bezier(self.t),
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

pub(crate) fn disappear(context: &egui::Context, id: Id, seconds: f32, curve: Ease) -> f32 {
    if context.data(|data| data.get_temp::<Tween>(id)).is_none() {
        let tween = Tween {
            from: 1.0,
            to: 0.0,
            t: 0.0,
            frame: context.cumulative_frame_nr(),
        };
        context.data_mut(|data| data.insert_temp(id, tween));
    }
    ease(context, id, 0.0, seconds, curve)
}

pub(crate) fn reset(context: &egui::Context, id: Id) {
    context.data_mut(|data| data.remove::<Tween>(id));
}

pub(crate) fn hover(context: &egui::Context, response: &egui::Response) -> f32 {
    let target = if response.is_pointer_button_down_on() {
        1.0
    } else if response.hovered() {
        0.6
    } else {
        0.0
    };
    ease(
        context,
        response.id.with("hover"),
        target,
        fast(),
        Ease::Token,
    )
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
        Ease::Token,
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
        dt / PAGE_SECONDS
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
            if layer.order == egui::Order::Background || keep.contains(&layer.id) {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    None,
    Fade,
    SlideUp,
    SlideDown,
    SlideLeft,
    SlideRight,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Transition {
    pub enter: Effect,
    pub exit: Effect,
}

impl Transition {
    pub const fn same(effect: Effect) -> Self {
        Self {
            enter: effect,
            exit: effect,
        }
    }

    pub const fn new(enter: Effect, exit: Effect) -> Self {
        Self { enter, exit }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Frame {
    pub shown: f32,
    pub closing: bool,
    pub offset: Vec2,
    pub alpha: f32,
}

impl Frame {
    fn new(shown: f32, closing: bool, effect: Effect, extent: Vec2) -> Self {
        let hidden = 1.0 - shown;
        let (offset, alpha) = match effect {
            Effect::None => (Vec2::ZERO, 1.0),
            Effect::Fade => (Vec2::ZERO, shown),
            Effect::SlideUp => (Vec2::new(0.0, hidden * extent.y), 1.0),
            Effect::SlideDown => (Vec2::new(0.0, -hidden * extent.y), 1.0),
            Effect::SlideLeft => (Vec2::new(hidden * extent.x, 0.0), 1.0),
            Effect::SlideRight => (Vec2::new(-hidden * extent.x, 0.0), 1.0),
        };
        Self {
            shown,
            closing,
            offset,
            alpha,
        }
    }
}

pub(crate) fn mount(
    context: &egui::Context,
    id: Id,
    open: bool,
    transition: Transition,
    extent: Vec2,
) -> Option<Frame> {
    let live = context.data(|data| data.get_temp::<Tween>(id)).is_some();
    if !live {
        if !open {
            return None;
        }
        let tween = Tween {
            from: 0.0,
            to: 1.0,
            t: 0.0,
            frame: context.cumulative_frame_nr(),
        };
        context.data_mut(|data| data.insert_temp(id, tween));
    }
    let shown = ease(
        context,
        id,
        if open { 1.0 } else { 0.0 },
        normal(),
        Ease::Token,
    );
    if !open && shown <= 0.001 {
        reset(context, id);
        return None;
    }
    let effect = if open {
        transition.enter
    } else {
        transition.exit
    };
    Some(Frame::new(shown, !open, effect, extent))
}

#[cfg(test)]
mod mount_tests {
    use super::*;

    fn frame(context: &egui::Context, tick: usize, run: impl FnOnce(&egui::Context)) {
        let input = egui::RawInput {
            time: Some(tick as f64 / 60.0),
            predicted_dt: 1.0 / 60.0,
            ..Default::default()
        };
        let mut run = Some(run);
        let _ = context.run(input, |context| (run.take().unwrap())(context));
    }

    #[test]
    fn panel_stays_mounted_until_the_exit_finishes() {
        let context = egui::Context::default();
        let id = Id::new("panel");
        let transition = Transition::same(Effect::SlideUp);
        let extent = Vec2::new(300.0, 200.0);
        let mut tick = 0;
        let mut shown = Vec::new();
        for open in std::iter::repeat_n(true, 40).chain(std::iter::repeat_n(false, 60)) {
            tick += 1;
            frame(&context, tick, |context| {
                shown.push((open, mount(context, id, open, transition, extent)));
            });
        }
        let (_, first) = shown[0];
        assert!(first.unwrap().shown < 0.2);
        assert!(shown[39].1.unwrap().shown > 0.99);
        let closing = shown[41].1.expect("still drawn right after close");
        assert!(closing.closing && closing.offset.y > 0.0);
        assert!(shown.last().unwrap().1.is_none());
    }

    #[test]
    fn token_curve_runs_from_zero_to_one_and_decelerates() {
        super::set_curve([0.2, 0.0, 0.0, 1.0]);
        assert!(super::bezier(0.0).abs() < 0.01);
        assert!((super::bezier(1.0) - 1.0).abs() < 0.01);
        assert!(super::bezier(0.5) > 0.5);
    }

    #[test]
    fn closed_panel_never_mounts() {
        let context = egui::Context::default();
        frame(&context, 1, |context| {
            assert!(
                mount(
                    context,
                    Id::new("never"),
                    false,
                    Transition::same(Effect::Fade),
                    Vec2::ZERO
                )
                .is_none()
            );
        });
    }

    #[test]
    fn exit_can_differ_from_enter() {
        let transition = Transition::new(Effect::SlideLeft, Effect::Fade);
        let enter = Frame::new(0.5, false, transition.enter, Vec2::new(100.0, 0.0));
        let exit = Frame::new(0.5, true, transition.exit, Vec2::new(100.0, 0.0));
        assert_eq!(enter.offset.x, 50.0);
        assert_eq!(enter.alpha, 1.0);
        assert_eq!(exit.offset.x, 0.0);
        assert_eq!(exit.alpha, 0.5);
    }
}
