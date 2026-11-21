use super::*;

pub(crate) fn mobile_scroll_reserve(viewport: Viewport) -> f32 {
    if viewport.is_compact() {
        crate::navigation::nav_reserve()
    } else {
        0.0
    }
}

const SCREEN_SCROLL_PENDING: &str = "fluxa-screen-scroll-pending";
const SCROLL_DIRECT: &str = "fluxa-scroll-direct";

pub fn set_scroll_direct(context: &crate::gfx::Context, direct: bool) {
    context.data_mut(|data| data.insert_temp(Id::new(SCROLL_DIRECT), direct));
}

pub fn scroll_blocked(context: &crate::gfx::Context) -> bool {
    crate::gfx::Popup::is_any_open(context) || crate::components::screen_covered(context)
}

pub fn pointer_over_mini_player(context: &crate::gfx::Context, pos: Pos2) -> bool {
    crate::components::over_mini_player(context, pos)
}

pub fn screen_scroll_pending(context: &crate::gfx::Context) -> f32 {
    context
        .data(|data| data.get_temp::<f32>(Id::new(SCREEN_SCROLL_PENDING)))
        .unwrap_or(0.0)
}

fn row_pending_id(row: usize) -> Id {
    Id::new(("fluxa-home-row-scroll", row)).with("pending")
}

pub fn row_scroll_pending(context: &crate::gfx::Context, row: usize) -> f32 {
    context
        .data(|data| data.get_temp::<f32>(row_pending_id(row)))
        .unwrap_or(0.0)
}

pub fn scroll_pending(context: &crate::gfx::Context, id: crate::ScrollId) -> f32 {
    match id {
        crate::ScrollId::Row(row) => row_scroll_pending(context, row),
        crate::ScrollId::Strip(name) => context
            .data(|data| data.get_temp::<f32>(strip_pending_id(name)))
            .unwrap_or(0.0),
        _ => 0.0,
    }
}

fn strip_pending_id(name: &'static str) -> Id {
    Id::new(("fluxa-strip-scroll", name)).with("pending")
}

pub(crate) fn resolve_scroll(
    context: &crate::gfx::Context,
    id: crate::ScrollId,
    target: f32,
    eased: bool,
) -> f32 {
    match id {
        crate::ScrollId::Row(row) => resolve_row_scroll(context, row, target, eased),
        crate::ScrollId::Strip(name) => {
            if !eased {
                return target;
            }
            let shown = scroll_display(context, Id::new(("fluxa-strip-scroll", name)), target);
            context.data_mut(|data| data.insert_temp(strip_pending_id(name), target - shown));
            shown
        }
        _ => target,
    }
}

pub(crate) fn scroll_display(context: &crate::gfx::Context, id: Id, target: f32) -> f32 {
    let direct = context
        .data(|data| data.get_temp::<bool>(Id::new(SCROLL_DIRECT)))
        .unwrap_or(false);
    if direct {
        return target;
    }
    ease_scroll(context, id, target)
}

pub(crate) fn resolve_row_scroll(
    context: &crate::gfx::Context,
    row: usize,
    target: f32,
    eased: bool,
) -> f32 {
    if !eased {
        return target;
    }
    let shown = scroll_display(context, Id::new(("fluxa-home-row-scroll", row)), target);
    context.data_mut(|data| data.insert_temp(row_pending_id(row), target - shown));
    shown
}

pub(crate) fn ease_scroll(context: &crate::gfx::Context, id: Id, target: f32) -> f32 {
    let frame = context.cumulative_pass_nr();
    let (shown, seen) = context
        .data(|data| data.get_temp::<(f32, u64)>(id))
        .unwrap_or((target, frame));
    let shown = if frame.saturating_sub(seen) > 3 {
        target
    } else {
        let dt = context.input(|input| input.stable_dt()).clamp(0.001, 0.05);
        let step = (target - shown) * (1.0 - (-dt * 14.0).exp());
        if (target - shown - step).abs() < 0.5 {
            target
        } else {
            shown + step
        }
    };
    context.data_mut(|data| data.insert_temp(id, (shown, frame)));
    if shown != target {
        context.request_repaint();
    }
    shown
}

pub(crate) fn resolve_screen_scroll(
    context: &crate::gfx::Context,
    viewport: Viewport,
    id: Id,
    max_offset: f32,
) -> f32 {
    crate::report_scroll_region(
        context,
        crate::ScrollRegion {
            id: crate::ScrollId::Screen,
            axis: crate::ScrollAxis::Vertical,
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height)),
            max: max_offset,
        },
    );
    let target = viewport.scroll_y.clamp(0.0, max_offset);
    if viewport.form_factor == UiFormFactor::Mobile {
        return target;
    }
    let shown = scroll_display(context, id, target);
    context.data_mut(|data| data.insert_temp(Id::new(SCREEN_SCROLL_PENDING), target - shown));
    shown
}

pub fn backdrop_target_size(width: f32, pixels_per_point: f32) -> [u32; 2] {
    artwork_target_size(Vec2::new(width, width * 9.0 / 16.0), pixels_per_point)
}

pub(crate) fn artwork_target_size(size: Vec2, pixels_per_point: f32) -> [u32; 2] {
    let scale = if pixels_per_point.is_finite() {
        pixels_per_point.max(0.25)
    } else {
        1.0
    };
    fluxa_artwork::bounded_target([
        (size.x.max(1.0) * scale).ceil() as u32,
        (size.y.max(1.0) * scale).ceil() as u32,
    ])
}
