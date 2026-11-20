use super::*;

pub(super) fn grid<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let metrics = env.metrics;
    let own = node
        .width
        .or(node.width_vw.map(|share| share * env.view.get().x));
    let margin = if own.is_some() { 0.0 } else { metrics.gutter };
    let inner = own.unwrap_or(env.view.get().x - metrics.gutter * 2.0);
    let columns = node
        .props
        .get("columns")
        .and_then(|value| match value {
            Value::String(text) => text.trim().parse::<u64>().ok(),
            other => other.as_u64(),
        })
        .filter(|columns| *columns > 0);
    let (cell, gap) = match columns {
        Some(columns) => {
            let gap = node.gap.unwrap_or(metrics.control_gap);
            let cell = ((inner - gap * (columns - 1) as f32) / columns as f32)
                .floor()
                .max(1.0);
            (cell, gap)
        }
        None => {
            let sizes = crate::layout::PosterGrid::new(inner, metrics);
            (sizes.card_width, sizes.gap)
        }
    };
    request_more(ui.ctx(), node, row_hint(&node.children, ui.ctx()), env);
    let top_id = Id::new(("skin-grid-top", &node.key));
    let row_id = Id::new(("skin-grid-row", &node.key));
    let ctx = ui.ctx().clone();
    let top = ctx.data(|data| data.get_temp::<f32>(top_id));
    let row = node
        .children
        .iter()
        .find_map(|child| {
            ctx.data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &child.key))))
        })
        .map(|(rect, _)| rect.height())
        .filter(|height| *height > 1.0)
        .or_else(|| ctx.data(|data| data.get_temp::<f32>(row_id)));
    if let Some(row) = row {
        ctx.data_mut(|data| data.insert_temp(row_id, row));
    }
    let known = top.zip(row);
    let clip = ui.clip_rect();
    let columns = (((inner + gap) / (cell + gap)).floor() as usize).max(1);
    if let Some((top, row)) = known {
        let pitch = (row + gap).max(1.0);
        let screens = (clip.height() / pitch).ceil() as usize + 1;
        let first = ((clip.top() - top) / pitch).floor().max(0.0) as usize;
        let last = ((clip.bottom() - top) / pitch).floor().max(0.0) as usize;
        let chunk = columns * 4;
        let start = first.saturating_sub(screens * 2) * columns / chunk * chunk;
        let end = ((last + screens * 2 + 1) * columns).div_ceil(chunk) * chunk;
        if let Ok(mut windows) = GRID_WINDOWS.lock()
            && windows.get(&node.key) != Some(&(start, end))
        {
            windows.insert(node.key.clone(), (start, end));
            ctx.request_repaint();
        }
    }
    let items = node
        .children
        .iter()
        .enumerate()
        .map(|(index, child)| {
            let far = child.kind == "far"
                || known.is_some_and(|(top, row)| {
                    let y = top + (index / columns) as f32 * (row + gap);
                    y + row < clip.top() - clip.height() * 0.5
                        || y > clip.bottom() + clip.height() * 0.5
                });
            if far {
                far_card(child, cell, known.map(|(_, row)| row), env)
            } else {
                let mut child = child.clone();
                if child.width.is_none() {
                    child.width = Some(cell);
                }
                build(ui, &child, env)
            }
        })
        .collect();
    layout_items::row(items)
        .wrap()
        .gap(gap)
        .padding(Insets::xy(margin, 0.0))
        .align_self(FlexAlign::Stretch)
        .painted(move |ui, rect| {
            if !kit::probing() {
                ui.ctx()
                    .data_mut(|data| data.insert_temp(top_id, rect.top()));
            }
        })
}

pub(super) fn row_hint(cards: &[Resolved], ctx: &crate::gfx::Context) -> f32 {
    cards
        .iter()
        .find_map(|child| {
            ctx.data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &child.key))))
        })
        .map_or(300.0, |(rect, _)| rect.height())
}

pub(super) fn request_more<A: HomeAssets>(
    ctx: &crate::gfx::Context,
    node: &Resolved,
    row: f32,
    env: &Env<'_, A>,
) {
    let Some(request) = node.props.get("more").filter(|more| more.is_object()) else {
        return;
    };
    let Some(skip) = request.get("skip").and_then(Value::as_i64) else {
        return;
    };
    let Some((offset, max)) = kit::scroll_state(ctx, "fluxa-skin") else {
        return;
    };
    if offset < (max - env.view.get().y * 8.0 - row * 12.0).max(0.0) {
        return;
    }
    let id = Id::new(("skin-grid-more", &node.key));
    let now = ctx.input(|input| input.time());
    let count = node.children.len();
    let retry = std::time::Duration::from_secs(2);
    let due = ctx.data_mut(|data| {
        let last = data.get_temp::<(i64, usize, f64)>(id);
        let fresh = last.is_none_or(|(prev, prev_count, at)| {
            prev != skip || prev_count != count || now - at >= retry.as_secs_f64()
        });
        if fresh {
            data.insert_temp(id, (skip, count, now));
        }
        fresh
    });
    if due {
        env.layout.borrow_mut().load_more.push(request.clone());
    }
    ctx.request_repaint_after(retry);
}
