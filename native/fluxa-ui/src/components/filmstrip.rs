use super::*;
use crate::player::format_time;

const SELECTED_SCALE: f32 = 1.08;
const MIN_GAP: f32 = 0.06;
const WHEEL_STEP: f32 = 40.0;

pub(crate) struct FilmstripLayout {
    pub count: usize,
    pub width: f32,
    pub gap: f32,
}

pub(crate) fn filmstrip_layout(
    track_width: f32,
    native_width: Option<f32>,
    metrics: UiMetrics,
) -> FilmstripLayout {
    let native = native_width.unwrap_or(metrics.filmstrip_frame_width);
    let gap = (native * MIN_GAP).max(6.0);
    let fit = ((track_width + gap) / (native + gap)).ceil();
    let count = fit.max(metrics.filmstrip_min_frames) as usize;
    let width = (track_width - (count - 1) as f32 * gap) / count as f32;
    FilmstripLayout { count, width, gap }
}

fn frame_layout(
    context: &egui::Context,
    track: Rect,
    frames: &[(f64, egui::TextureId, Vec2)],
    metrics: UiMetrics,
) -> FilmstripLayout {
    let native = frames
        .first()
        .map(|(_, _, size)| size.x / context.pixels_per_point());
    filmstrip_layout(track.width(), native, metrics)
}

pub(crate) fn filmstrip_pitch(
    context: &egui::Context,
    track: Rect,
    frames: &[(f64, egui::TextureId, Vec2)],
    metrics: UiMetrics,
) -> f32 {
    let layout = frame_layout(context, track, frames, metrics);
    layout.width + layout.gap
}

pub(crate) struct FilmstripOutput {
    pub slots: Vec<f64>,
    pub seek: Option<f64>,
}

fn area_id() -> Id {
    Id::new("fluxa-filmstrip-area")
}

fn fine_id() -> Id {
    Id::new("fluxa-filmstrip-fine")
}

fn wheel_id() -> Id {
    Id::new("fluxa-filmstrip-wheel")
}

pub(crate) fn filmstrip_area(context: &egui::Context) -> Option<Rect> {
    context
        .data(|data| data.get_temp::<Option<Rect>>(area_id()))
        .flatten()
}

pub(crate) fn filmstrip_close(context: &egui::Context) {
    context.data_mut(|data| {
        data.insert_temp(area_id(), None::<Rect>);
        data.remove_temp::<f32>(wheel_id());
        data.insert_temp(fine_id(), None::<f64>);
    });
}

pub(crate) fn seek_filmstrip(
    context: &egui::Context,
    track: Rect,
    bar_time: Option<f64>,
    duration: f64,
    step: f64,
    forced: Option<f64>,
    lift: f32,
    frames: &[(f64, egui::TextureId, Vec2)],
    metrics: UiMetrics,
) -> FilmstripOutput {
    let FilmstripLayout { count, width, gap } = frame_layout(context, track, frames, metrics);
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-seek-filmstrip"),
    ));
    let height = width * 9.0 / 16.0;
    let bottom = track.top() - 18.0 - lift;
    let bases: Vec<Rect> = (0..count)
        .map(|index| {
            Rect::from_min_size(
                Pos2::new(track.left() + index as f32 * (width + gap), bottom - height),
                Vec2::new(width, height),
            )
        })
        .collect();
    context.data_mut(|data| {
        data.insert_temp(
            area_id(),
            Some(Rect::from_min_max(
                Pos2::new(track.left(), bottom - height * SELECTED_SCALE - 6.0),
                Pos2::new(track.right(), track.top()),
            )),
        )
    });

    let pointer = context.input(|input| input.pointer.hover_pos());
    let moved = context.input(|input| input.pointer.delta() != Vec2::ZERO);
    let over = pointer.and_then(|pointer| {
        bases
            .iter()
            .position(|base| base.expand2(Vec2::new(gap * 0.5, 0.0)).contains(pointer))
    });
    let step = step.max(1.0);
    let last = (duration / step).floor() as i64;
    let window = |center: f64| {
        let index = ((center / step).round() as i64).clamp(0, last);
        let start = (index - count as i64 / 2).clamp(0, (last + 1 - count as i64).max(0));
        let slots: Vec<f64> = (0..count as i64)
            .map(|offset| ((start + offset) as f64 * step).min(duration))
            .collect();
        (slots, (index - start) as usize)
    };
    let mut center = context
        .data(|data| data.get_temp::<Option<f64>>(fine_id()))
        .flatten();
    if moved && over.is_none() {
        center = bar_time.or(center);
    }
    let scroll = context.input(|input| input.smooth_scroll_delta.x + input.smooth_scroll_delta.y);
    let wheel = context
        .data(|data| data.get_temp::<f32>(wheel_id()))
        .unwrap_or(0.0)
        + scroll;
    let steps = (wheel / WHEEL_STEP).trunc();
    context.data_mut(|data| data.insert_temp(wheel_id(), wheel - steps * WHEEL_STEP));
    let center = match forced {
        Some(time) => time.clamp(0.0, duration),
        None => (center.or(bar_time).unwrap_or(0.0) + steps as f64 * step).clamp(0.0, duration),
    };
    let (slots, middle) = window(center);
    let selected = over.filter(|_| forced.is_none()).unwrap_or(middle);
    let fine = Some(center);
    context.data_mut(|data| {
        data.insert_temp(fine_id(), fine);
    });

    let seek = over
        .filter(|_| forced.is_none())
        .filter(|_| context.input(|input| input.pointer.primary_clicked()))
        .map(|index| slots[index]);

    let font = FontId::proportional(metrics.text.meta);
    for (index, slot) in slots.iter().enumerate() {
        let base = bases[index];
        let chosen = index == selected;
        let frame = if chosen {
            Rect::from_center_size(
                Pos2::new(
                    base.center().x,
                    base.bottom() - height * SELECTED_SCALE * 0.5,
                ),
                base.size() * SELECTED_SCALE,
            )
        } else {
            base
        };
        let tint = if chosen {
            Color32::WHITE
        } else {
            Color32::from_gray(170)
        };
        painter.rect_filled(frame, radius::chip(), ink(tone::raised()));
        if let Some((_, texture, _)) = frames.iter().find(|(at, _, _)| (at - slot).abs() < 0.01) {
            rounded_image(&painter, *texture, frame, radius::chip(), full_uv(), tint);
        }
        paint_vertical_gradient(
            &painter,
            Rect::from_min_max(Pos2::new(frame.left(), frame.bottom() - 26.0), frame.max),
            Color32::TRANSPARENT,
            Color32::from_black_alpha(190),
        );
        draw_text(
            &painter,
            frame.left_bottom() + Vec2::new(6.0, -5.0),
            Align2::LEFT_BOTTOM,
            format_time(*slot),
            font.clone(),
            text_primary(),
            Some(frame.width() - 8.0),
        );
        if chosen {
            painter.rect_stroke(
                frame,
                radius::chip(),
                egui::Stroke::new(2.0, Color32::WHITE),
                egui::StrokeKind::Inside,
            );
        }
    }
    FilmstripOutput { slots, seek }
}
