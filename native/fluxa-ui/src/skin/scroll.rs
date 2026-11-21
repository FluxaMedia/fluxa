use super::*;

pub(super) fn activate(ui: &mut Ui, rect: Rect, target: Option<u64>) {
    if let Some(node) = target.filter(|_| !kit::probing()) {
        let response = ui.interact(rect, Id::new(("skin-target", node)), Sense::click());
        response.focus_node(ui, node);
    }
}

pub(super) fn scroll_y<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let key = node.key.clone();
    let gap = gap_of(node, env).unwrap_or(0.0);
    let mut items = Vec::new();
    for (index, item) in children(ui, node, env).into_iter().enumerate() {
        if index > 0 && gap > 0.0 {
            items.push(layout_items::gap(gap));
        }
        items.push(item);
    }
    kit::item(Vec2::ZERO, move |ui, rect| {
        ui.set_clip_rect(rect);
        crate::components::pan_area(
            ui,
            ("skin-scroll-y", &key),
            |area| area.max_height(rect.height()).auto_shrink([false, false]),
            |ui| {
                kit::lay(ui, items);
            },
        );
    })
    .grow(1.0)
}

pub(super) fn scroll<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    if prop(node, "axis") == "y" {
        return scroll_y(ui, node, env);
    }
    let name = Some(strip_name(&node.key));
    let gap = gap_of(node, env).unwrap_or(0.0);
    let node = node.clone();
    let share = node.grow.is_some() || node.fill;
    let within = prop(&node, "within") == "true";
    let item = kit::widget(ui, move |ui| {
        let offset = name.map_or(0.0, |name| {
            crate::scroll_region::strip_offset(ui.ctx(), name)
        });
        let reach = ui.ctx().content_rect().width();
        let height_id = Id::new(("skin-strip-height", &node.key, reach as i32));
        let measured = ui.ctx().data(|data| data.get_temp::<f32>(height_id));
        let probing = kit::probing();
        let rect = ui.max_rect();
        let clip = ui.clip_rect();
        let off_screen = measured.is_some_and(|height| {
            probing
                || rect.top() > clip.bottom() + clip.height() * 0.2
                || rect.top() + height < clip.top() - clip.height() * 0.2
        });
        let strip_slot = name
            .filter(|_| !probing && !off_screen && measured.is_some())
            .map(|name| {
                let shown = crate::components::resolve_strip(ui, name);
                let sig = StripSig {
                    card: card_sig(
                        ui,
                        Vec2::new(ui.max_rect().width(), measured.unwrap_or(0.0)),
                    ),
                    shown: shown.to_bits(),
                    reach: reach.to_bits(),
                };
                (name, shown, sig)
            });
        if let Some((_, _, sig)) = &strip_slot
            && cached_strip(ui, &node.children, sig, env)
        {
            return;
        }
        if strip_slot.is_some() {
            crate::capture::begin();
        }
        let mut x = 0.0;
        let mut near: Option<(usize, usize)> = None;
        let items = node
            .children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let Some(width) = card_width(child, env) else {
                    return build(ui, child, env).shrink(0.0);
                };
                let start = x;
                x += width + gap;
                let far = start + width < offset - reach * 0.1 || start > offset + reach * 1.1;
                if start + width >= offset - reach && start <= offset + reach * 2.0 {
                    near = Some(near.map_or((index, index), |(first, _)| (first, index)));
                }
                if child.kind == "far" || far || off_screen {
                    far_card(child, width, measured.filter(|_| off_screen), env).shrink(0.0)
                } else if let Some(height) =
                    measured.filter(|_| !probing && !expanding_card(ui, child))
                {
                    cached_card(
                        ui,
                        &node.children,
                        index,
                        child,
                        Vec2::new(width, height),
                        env,
                    )
                    .shrink(0.0)
                } else {
                    build(ui, child, env).shrink(0.0)
                }
            })
            .collect();
        if !probing
            && let Some((first, last)) = near
            && let Ok(mut windows) = GRID_WINDOWS.lock()
        {
            let window = (first / 4 * 4, (last + 1).div_ceil(4) * 4);
            if windows.get(&node.key) != Some(&window) {
                windows.insert(node.key.clone(), window);
                ui.ctx().request_repaint();
            }
        }
        let strip = layout_items::row(items).gap(gap);
        let ctx = ui.ctx().clone();
        let layer = ui.layer_id();
        let origin = ui.max_rect().min;
        let clip = ui.clip_rect();
        let start = ctx.layer_cursor(layer);
        match (name, &strip_slot) {
            (Some(name), Some((_, shown, _))) => {
                crate::components::pan_strip_shown(ui, name, *shown, !within, |ui| {
                    kit::lay_overflow(ui, strip);
                });
            }
            (Some(name), None) if within => {
                crate::components::pan_strip_within(ui, name, |ui| {
                    kit::lay_overflow(ui, strip);
                });
            }
            (Some(name), None) => {
                crate::components::pan_strip(ui, name, |ui| {
                    kit::lay_overflow(ui, strip);
                });
            }
            (None, _) => {
                kit::lay_hug(ui, strip);
            }
        }
        if let Some((_, _, sig)) = strip_slot {
            let log = crate::capture::end();
            let used = ui.min_rect().size();
            if !log.dirty && !card_busy(&ctx, &log.states) && fully_in_view(clip, origin.y, used.y)
            {
                let entry = StripEntry {
                    parent: node.children.clone(),
                    sig,
                    origin,
                    clip,
                    used,
                    shapes: ctx.layer_shapes_since(layer, start),
                    registers: log.registers,
                    states: log.states,
                    claims: log.claims,
                    touches: log.touches,
                    regions: log.regions,
                };
                STRIPS.with(|strips| {
                    let mut strips = strips.borrow_mut();
                    if strips.len() >= CARD_CACHE_LIMIT {
                        strips.clear();
                    }
                    strips.insert(
                        Arc::as_ptr(&node.children) as usize,
                        std::rc::Rc::new(entry),
                    );
                });
            }
        }
        if !probing && !off_screen {
            let height = ui.min_rect().height();
            ui.ctx()
                .data_mut(|data| data.insert_temp(height_id, height));
        }
    });
    if share {
        item.basis(0.0).min_width(0.0)
    } else {
        item
    }
}

pub(super) fn motion_overlay(
    ui: &mut Ui,
    rect: Rect,
    url: &str,
    autoplay: bool,
    assets: &mut impl HomeAssets,
) {
    let clip = ui.clip_rect();
    let visible = rect.intersect(clip);
    let on_screen = visible.is_positive() && visible.area() >= rect.area() * 0.6;
    let hovered = ui.rect_contains_pointer(rect);
    let target = fluxa_artwork::animation_target_size(crate::artwork_target_size(
        rect.size(),
        ui.ctx().pixels_per_point(),
    ));
    if !(autoplay && on_screen || hovered) {
        return;
    }
    let Some(animated) = assets.animated_texture_for(Some(url), target, ArtworkPriority::Visible)
    else {
        return;
    };
    let dest = kit::fit_inside(rect, animated.image_size);
    ui.painter()
        .rect_filled(rect, 0.0, crate::stylesheet::color("well"));
    texture_image(
        ui.painter(),
        animated.texture,
        dest,
        animated.uv,
        Color32::WHITE,
    );
    ui.ctx().request_repaint();
}
