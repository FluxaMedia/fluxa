use super::*;

pub(super) const EXPAND_PATIENCE: f64 = 1.5;

thread_local! {
    pub(super) static EXPAND: Cell<Option<(f32, Option<f64>)>> = const { Cell::new(None) };
}

pub(super) fn expanding_card(ui: &Ui, node: &Resolved) -> bool {
    if prop(node, "expand").is_empty() || !prop(node, "expandOff").is_empty() {
        return false;
    }
    let ctx = ui.ctx();
    let moving = ctx
        .data(|data| data.get_temp::<(Option<f64>, f32)>(Id::new(("skin-expand", &node.key))))
        .is_some_and(|(since, progress)| since.is_some() || progress > 0.0);
    moving
        || ctx
            .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &node.key))))
            .is_some_and(|(rect, flags)| {
                flags & crate::stylesheet::FOCUSED != 0
                    || hover_pos(ctx).is_some_and(|pos| rect.contains(pos))
            })
}

pub(super) fn expand_ease(
    ui: &Ui,
    node: &Resolved,
    delay: f64,
    seconds: f32,
    ready: impl FnOnce() -> bool,
) -> f32 {
    let ctx = ui.ctx();
    let id = Id::new(("skin-expand", &node.key));
    let (mut since, mut progress): (Option<f64>, f32) =
        ctx.data(|data| data.get_temp(id)).unwrap_or((None, 0.0));
    if progress > 0.0 || since.is_some() {
        crate::capture::mark_dirty();
    }
    if kit::probing() {
        return progress;
    }
    let now = ctx.input(|input| input.time());
    let open = ctx
        .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &node.key))))
        .is_some_and(|(rect, flags)| {
            flags & crate::stylesheet::FOCUSED != 0
                || hover_pos(ctx).is_some_and(|pos| rect.contains(pos))
        });
    if open {
        since.get_or_insert(now);
    } else {
        since = None;
    }
    let prepared = open && ready();
    let shown = since.is_some_and(|since| {
        let waited = now - since - delay;
        waited >= 0.0 && (progress > 0.0 || prepared || waited >= EXPAND_PATIENCE)
    });
    let step = ctx.input(|input| input.stable_dt()).min(0.05) / seconds.max(0.01);
    progress = match (shown, delay <= 0.0) {
        (true, true) => 1.0,
        (false, true) => 0.0,
        (true, false) => (progress + step).min(1.0),
        (false, false) => (progress - step).max(0.0),
    };
    ctx.data_mut(|data| data.insert_temp(id, (since, progress)));
    if progress > 0.0 || since.is_some() {
        crate::capture::mark_dirty();
    }
    if let Some(since) = since.filter(|_| !shown) {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(
            (delay - (now - since)).max(0.0),
        ));
    } else if (shown && progress < 1.0) || (!shown && progress > 0.0) {
        ctx.request_repaint();
    }
    crate::gfx::emath::easing::cubic_out(progress)
}

pub(super) fn expanded_targets(
    node: &Resolved,
    base: f32,
    art: &mut Vec<(String, [u32; 2])>,
    logos: &mut Vec<(String, [u32; 2])>,
) {
    for child in node.children.iter() {
        let logo = prop(child, "logoOf");
        if child.kind == "image" && !logo.is_empty() {
            let share = prop(child, "share").parse::<f32>().unwrap_or(1.0);
            let size = Vec2::new(
                child
                    .width
                    .unwrap_or_else(|| expanded_width(base, 1.0) * share),
                child.height.unwrap_or(base * 1.5),
            );
            logos.push((logo.into_owned(), crate::artwork_target_size(size, 1.0)));
        } else if child.kind == "image" && child.props.get("expandedOnly").is_some_and(truthy) {
            if let Some(url) = child.src.clone().filter(|url| !url.is_empty()) {
                let size = Vec2::new(
                    expanded_width(base, 1.0),
                    child.height.unwrap_or(base * 1.5),
                );
                art.push((url, crate::artwork_target_size(size, 1.0)));
            }
        }
        expanded_targets(child, base, art, logos);
    }
}

pub(super) fn expand_ready<A: HomeAssets>(node: &Resolved, env: &Env<'_, A>) -> bool {
    let Some(base) = card_width_base(node, env) else {
        return true;
    };
    let mut art = Vec::new();
    let mut logos = Vec::new();
    expanded_targets(node, base, &mut art, &mut logos);
    let mut assets = env.assets.borrow_mut();
    let mut ready = true;
    for (url, target) in &art {
        ready &= assets
            .texture_for(Some(url), *target, ArtworkPriority::Visible)
            .is_some();
    }
    for (id, target) in &logos {
        let Some(card) = crate::card_registry::card(id) else {
            continue;
        };
        crate::card_logos::urgent(&card);
        if crate::card_logos::awaiting(&card) {
            ready = false;
        } else if let Some(url) = crate::card_logos::logo_of(&card) {
            ready &= assets
                .texture_for(Some(&url), *target, ArtworkPriority::Visible)
                .is_some();
        }
    }
    ready
}

pub(super) fn state_scale<A: HomeAssets>(ui: &Ui, node: &Resolved, env: &Env<'_, A>) -> f32 {
    let Some((rule, modifiers)) = class_parts(node, env.positions.get(&node.key).copied()) else {
        return 1.0;
    };
    let ctx = ui.ctx();
    let (hovered, focused) = ctx
        .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &node.key))))
        .map_or((false, false), |(rect, flags)| {
            (
                hover_pos(ctx).is_some_and(|pos| rect.contains(pos)),
                flags & crate::stylesheet::FOCUSED != 0,
            )
        });
    crate::stylesheet::resolve_with(
        ctx,
        &rule,
        Id::new(("skin-pop", &node.key)),
        crate::stylesheet::State {
            hovered,
            focused,
            enabled: true,
            ..Default::default()
        },
        &modifiers,
    )
    .scale
}

pub(super) fn lift_ease(ui: &Ui, node: &Resolved) -> f32 {
    let ctx = ui.ctx();
    let raised = ctx
        .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &node.key))))
        .is_some_and(|(rect, flags)| {
            flags & crate::stylesheet::FOCUSED != 0
                || hover_pos(ctx).is_some_and(|pos| rect.contains(pos))
        });
    crate::motion::toggle(
        ctx,
        Id::new(("skin-lift", &node.key)),
        raised,
        crate::motion::fast(),
    )
}
