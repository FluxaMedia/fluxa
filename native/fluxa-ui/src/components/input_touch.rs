use super::*;
use crate::dp::Dp;
use crate::gfx::{Area, Order, Shape};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

const STATE: &str = "fluxa-touch-edit";
const KEEP: &str = "fluxa-touch-keep";
const GRIPS: &str = "fluxa-touch-grips";
const LONG_PRESS: f64 = 0.5;
const SLOP: f32 = 10.0;
const ROOT_HALF: f32 = std::f32::consts::FRAC_1_SQRT_2;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Cursor,
    Selection,
}

#[derive(Clone, Copy, PartialEq)]
enum Grip {
    Start,
    End,
    Caret,
}

#[derive(Clone, Copy)]
struct Drag {
    grip: Grip,
    grab: Vec2,
}

#[derive(Clone)]
struct Touch {
    field: Id,
    text: u64,
    mode: Option<Mode>,
    toolbar: bool,
    ends: [usize; 2],
    drag: Option<Drag>,
    pressed_at: Option<f64>,
    fired: bool,
    held: Option<[usize; 2]>,
}

impl Touch {
    fn new(field: Id, text: u64) -> Self {
        Self {
            field,
            text,
            mode: None,
            toolbar: false,
            ends: [0, 0],
            drag: None,
            pressed_at: None,
            fired: false,
            held: None,
        }
    }
}

fn digest(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn word_around(text: &str, at: usize) -> Option<[usize; 2]> {
    let chars: Vec<char> = text.chars().collect();
    let at = at.min(chars.len());
    let on = |i: usize| chars.get(i).copied().is_some_and(word_char);
    let seed = if on(at) {
        at
    } else if at > 0 && on(at - 1) {
        at - 1
    } else {
        return None;
    };
    let mut start = seed;
    while start > 0 && on(start - 1) {
        start -= 1;
    }
    let mut end = seed;
    while on(end) {
        end += 1;
    }
    Some([start, end])
}

fn apply(ctx: &crate::gfx::Context, id: Id, [from, to]: [usize; 2]) {
    if from == to {
        crate::gfx::set_cursor(ctx, id, from);
    } else {
        crate::gfx::set_selection(ctx, id, from.min(to), from.max(to));
    }
}

fn load(ctx: &crate::gfx::Context) -> Option<Touch> {
    ctx.data(|data| data.get_temp::<Touch>(Id::new(STATE)))
}

fn store(ctx: &crate::gfx::Context, touch: Touch) {
    ctx.data_mut(|data| data.insert_temp(Id::new(STATE), touch));
}

pub fn dismiss_text_toolbar(context: &crate::gfx::Context) {
    if let Some(mut touch) = load(context) {
        touch.toolbar = false;
        store(context, touch);
    }
}

pub fn text_grip_hit(context: &crate::gfx::Context, pos: Pos2) -> bool {
    context.data(|data| {
        data.get_temp::<Vec<Rect>>(Id::new(GRIPS))
            .is_some_and(|grips| grips.iter().any(|grip| grip.contains(pos)))
    })
}

pub(super) fn keeps_focus(context: &crate::gfx::Context) -> bool {
    context
        .data_mut(|data| data.remove_temp::<bool>(Id::new(KEEP)))
        .unwrap_or(false)
}

pub(super) fn clear_grips(context: &crate::gfx::Context) {
    context.data_mut(|data| data.remove::<Vec<Rect>>(Id::new(GRIPS)));
}

fn register(ctx: &crate::gfx::Context, hit: Rect) {
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<Vec<Rect>>(Id::new(GRIPS))
            .push(hit)
    });
}

fn paint_grip(ui: &Ui, grip: Grip, at: Pos2, leading: bool, radius: f32, color: Color32) {
    let painter = ui.painter();
    match grip {
        Grip::Caret => {
            let reach = radius * 2.0_f32.sqrt();
            let shoulder = radius * ROOT_HALF;
            painter.add(Shape::convex_polygon(
                vec![
                    at,
                    Pos2::new(at.x - shoulder, at.y + shoulder),
                    Pos2::new(at.x + shoulder, at.y + shoulder),
                ],
                color,
                crate::gfx::Stroke::NONE,
            ));
            painter.circle_filled(Pos2::new(at.x, at.y + reach), radius, color);
        }
        _ => {
            let side = if leading { -1.0 } else { 1.0 };
            let corner = Rect::from_two_pos(at, Pos2::new(at.x + side * radius, at.y + radius));
            painter.rect_filled(corner, 0.0, color);
            painter.circle_filled(
                Pos2::new(at.x + side * radius, at.y + radius),
                radius,
                color,
            );
        }
    }
}

struct GripEvents {
    clicked: bool,
    started: bool,
    dragging: bool,
    stopped: bool,
    pointer: Option<Pos2>,
}

fn show_grip(
    ctx: &crate::gfx::Context,
    grip: Grip,
    at: Pos2,
    leading: bool,
    size: f32,
) -> GripEvents {
    let radius = size * 0.5;
    let reach = if grip == Grip::Caret {
        radius * 2.0_f32.sqrt()
    } else {
        radius
    };
    let touch = Dp::D48.px();
    let hit = Rect::from_center_size(Pos2::new(at.x, at.y + reach), Vec2::splat(touch));
    register(ctx, hit);
    let color = crate::metrics::accent();
    ctx.keep_focus_on_click(true);
    let shown = Area::new(Id::new((STATE, grip as u8)))
        .order(Order::Foreground)
        .fixed_pos(hit.min)
        .constrain(false)
        .show(ctx, |ui| {
            let response = ui.allocate_rect(hit, Sense::click_and_drag());
            paint_grip(ui, grip, at, leading, radius, color);
            GripEvents {
                clicked: response.clicked(),
                started: response.drag_started(),
                dragging: response.dragged(),
                stopped: response.drag_stopped(),
                pointer: response.interact_pointer_pos(),
            }
        });
    ctx.keep_focus_on_click(false);
    shown.inner
}

pub(super) fn drive(
    ui: &Ui,
    output: &crate::gfx::TextEditOutput,
    field: Rect,
    value: &str,
) -> Option<Rect> {
    let size = crate::metrics::active().touch_handle;
    if size <= 0.0 {
        return None;
    }
    let ctx = ui.ctx();
    let id = output.response.id;
    let hash = digest(value);
    let mut touch = load(ctx)
        .filter(|touch| touch.field == id && !output.response.gained_focus())
        .unwrap_or_else(|| {
            let mut fresh = Touch::new(id, hash);
            fresh.mode = Some(Mode::Cursor);
            fresh
        });
    if touch.text != hash {
        touch = Touch::new(id, hash);
    }
    let range = output.cursor_range.unwrap_or_default();
    let (lo, hi) = (
        range.primary.min(range.secondary),
        range.primary.max(range.secondary),
    );
    if touch.drag.is_none() && touch.held.is_none() {
        touch.ends = [lo, hi];
        match (touch.mode, lo != hi) {
            (Some(Mode::Cursor), true) => touch.mode = Some(Mode::Selection),
            (Some(Mode::Selection), false) => {
                touch.mode = Some(Mode::Cursor);
                touch.toolbar = false;
            }
            _ => {}
        }
    }

    let (down, latest, origin, started, now) = ctx.input(|input| {
        (
            input.pointer().primary_down(),
            input.pointer().latest_pos(),
            input.pointer().press_origin(),
            input.pointer().press_start_time(),
            input.time(),
        )
    });
    let tapped = output.response.clicked() && !touch.fired;
    if started.is_some() && started != touch.pressed_at {
        touch.pressed_at = started;
        touch.fired = false;
    }
    if touch.drag.is_none()
        && down
        && !touch.fired
        && let (Some(origin), Some(latest), Some(started)) = (origin, latest, started)
        && field.expand(4.0).contains(origin)
    {
        if origin.distance(latest) > SLOP {
            touch.fired = true;
        } else if now - started >= LONG_PRESS {
            touch.fired = true;
            let at = output.index_at(origin);
            let ends = word_around(value, at).unwrap_or([at, at]);
            touch.mode = Some(if ends[0] == ends[1] {
                Mode::Cursor
            } else {
                Mode::Selection
            });
            touch.ends = ends;
            touch.held = Some(ends);
            touch.toolbar = true;
            crate::haptics::buzz(crate::haptics::Haptic::LongPress);
        } else {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
    if let Some(ends) = touch.held {
        apply(ctx, id, ends);
        if !down {
            touch.held = None;
        }
    } else if output.response.double_clicked() {
        touch.mode = Some(if lo != hi {
            Mode::Selection
        } else {
            Mode::Cursor
        });
        touch.toolbar = lo != hi;
    } else if tapped {
        touch.mode = Some(if lo != hi {
            Mode::Selection
        } else {
            Mode::Cursor
        });
        touch.toolbar = false;
    }

    let row = |index: usize| output.rect_at(index);
    let anchor = |index: usize| {
        let rect = row(index);
        (field.x_range().contains(rect.min.x) && field.y_range().contains(rect.center().y))
            .then(|| Pos2::new(rect.min.x, rect.max.y))
    };
    let grips: Vec<(Grip, usize)> = match touch.mode {
        Some(Mode::Cursor) => vec![(Grip::Caret, touch.ends[0])],
        Some(Mode::Selection) => vec![(Grip::Start, touch.ends[0]), (Grip::End, touch.ends[1])],
        None => Vec::new(),
    };
    for (grip, index) in grips {
        let (own, other) = if grip == Grip::End {
            (touch.ends[1], touch.ends[0])
        } else {
            (touch.ends[0], touch.ends[1])
        };
        let leading = own < other || (own == other && grip == Grip::Start);
        let shown = anchor(index).unwrap_or(Pos2::new(f32::MAX, f32::MAX));
        if shown.x == f32::MAX && touch.drag.is_none_or(|drag| drag.grip != grip) {
            continue;
        }
        let events = show_grip(ctx, grip, shown, leading, size);
        if events.clicked {
            touch.toolbar = !touch.toolbar;
        }
        if events.started
            && let Some(pointer) = events.pointer
        {
            touch.drag = Some(Drag {
                grip,
                grab: pointer - shown,
            });
            touch.toolbar = false;
        }
        if events.dragging
            && let (Some(drag), Some(pointer)) = (touch.drag, events.pointer)
            && drag.grip == grip
        {
            let line = row(index).height() * 0.5;
            let probe = pointer - drag.grab - Vec2::new(0.0, line);
            let moved = output.index_at(probe);
            match grip {
                Grip::Caret => touch.ends = [moved, moved],
                Grip::Start => touch.ends[0] = moved,
                Grip::End => touch.ends[1] = moved,
            }
            apply(ctx, id, touch.ends);
        }
        if events.stopped && touch.drag.is_some_and(|drag| drag.grip == grip) {
            touch.drag = None;
            touch.toolbar = grip != Grip::Caret;
        }
    }

    if touch.mode.is_some() {
        ctx.data_mut(|data| data.insert_temp(Id::new(KEEP), true));
    }

    let toolbar = (touch.toolbar && touch.drag.is_none())
        .then(|| {
            row(touch.ends[0])
                .union(row(touch.ends[1]))
                .intersect(field)
        })
        .filter(|rect| rect.height() > 0.0);
    store(ctx, touch);
    toolbar
}
