use super::*;

pub(super) fn lens(ctx: &crate::gfx::Context, keys: &[String], rect: Rect) -> Rect {
    for key in keys {
        let found = ctx.data(|data| data.get_temp::<(f32, f32, f32)>(Id::new(("skin-lens", key))));
        if let Some((x, reach, magnify)) = found {
            let t = 1.0 - ((rect.center().x - x).abs() / reach.max(1.0)).clamp(0.0, 1.0);
            let scale = 1.0 + magnify * t * t * (3.0 - 2.0 * t);
            return crate::flex::scaled(rect, scale);
        }
    }
    rect
}

pub(super) fn ancestor_keys<A: HomeAssets>(env: &Env<'_, A>) -> Vec<String> {
    env.ancestors
        .borrow()
        .iter()
        .map(|ancestor| ancestor.key.clone())
        .collect()
}

pub(super) fn indicator_width(rects: &[Rect], x: f32) -> f32 {
    let first = rects[0];
    if x <= first.center().x {
        return first.width();
    }
    for pair in rects.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if x <= b.center().x {
            let t = (x - a.center().x) / (b.center().x - a.center().x).max(1.0);
            return a.width() + (b.width() - a.width()) * t;
        }
    }
    rects[rects.len() - 1].width()
}

pub(super) struct IndicatorSlot {
    pub(super) rect: Rect,
    pub(super) selected: bool,
    pub(super) node: Option<u64>,
}

pub(super) fn paint_indicator(
    ui: &Ui,
    key: &str,
    class: &str,
    slots: &[IndicatorSlot],
    dragged_x: Option<f32>,
) {
    let Some(first) = slots.first() else {
        return;
    };
    let rects: Vec<Rect> = slots.iter().map(|slot| slot.rect).collect();
    let inner = rects.iter().fold(first.rect, |acc, rect| acc.union(*rect));
    let first_x = first.rect.center().x;
    let last_x = slots[slots.len() - 1].rect.center().x;
    let active = slots
        .iter()
        .find(|slot| slot.selected)
        .map(|slot| slot.rect.center().x);
    let Some(target) = dragged_x.map(|x| x.clamp(first_x, last_x)).or(active) else {
        return;
    };
    let x = crate::motion::ease(
        ui.ctx(),
        Id::new(("skin-indicator-x", key)),
        target,
        crate::motion::normal(),
        crate::motion::Ease::Linear,
    );
    let style = crate::stylesheet::resolve_with(
        ui.ctx(),
        class,
        Id::new(("skin-indicator", key)),
        crate::stylesheet::State {
            hovered: false,
            pressed: false,
            focused: false,
            enabled: true,
            selected: false,
            passive: true,
        },
        &[],
    );
    let width = indicator_width(&rects, x);
    let reach = width * style.stretch;
    let stretch = (target - x).abs().min(reach);
    let squash = if reach > 0.0 {
        (stretch / reach).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let pill = crate::flex::centered_at_x(
        inner,
        x,
        Vec2::new(
            width + stretch,
            inner.height() * (1.0 - style.squash * squash),
        ),
    )
    .intersect(inner);
    if style.magnify > 0.0 {
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                Id::new(("skin-lens", key)),
                (x, width * style.reach, style.magnify),
            );
        });
    }
    paint_box(ui.painter(), pill, style.radius.of(pill.height()), &style);
    if dragged_x.is_some() || stretch > 0.5 {
        ui.ctx().request_repaint();
    }
}
