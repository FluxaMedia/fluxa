use super::*;

pub fn skin_group_rect(context: &crate::gfx::Context, group: &str) -> Option<Rect> {
    context.data(|data| data.get_temp::<Rect>(Id::new(("skin-group", group))))
}

pub(super) fn group_hovered(context: &crate::gfx::Context, group: &str) -> bool {
    let rect = context.data(|data| data.get_temp::<Rect>(Id::new(("skin-group", group))));
    rect.zip(hover_pos(context))
        .is_some_and(|(rect, pos)| rect.expand(6.0).contains(pos))
}

pub(super) fn boxed<'a, A: HomeAssets>(
    item: kit::Item<'a>,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let container = matches!(node.kind.as_str(), "row" | "column" | "stack");
    let class = class_parts(node, env.positions.get(&node.key).copied());
    let chain = if class.is_some() {
        env.ancestors.borrow().clone()
    } else {
        Vec::new()
    };
    let target = if container {
        env.claim(
            &node.key,
            node.action.as_ref(),
            node.props.get("disabled").is_some_and(truthy),
        )
    } else {
        None
    };
    let selected = node.props.get("selected").is_some_and(truthy);
    let disabled = node.props.get("disabled").is_some_and(truthy);
    let span = node
        .props
        .get("span")
        .map(|value| match value {
            Value::String(text) => text.trim().parse::<f64>().unwrap_or(1.0),
            other => other.as_f64().unwrap_or(1.0),
        })
        .map_or(1.0, |share| share.clamp(0.0, 1.0) as f32);
    let start = node
        .props
        .get("start")
        .map(|value| match value {
            Value::String(text) => text.trim().parse::<f64>().unwrap_or(0.0),
            other => other.as_f64().unwrap_or(0.0),
        })
        .map_or(0.0, |share| share.clamp(0.0, 1.0) as f32);
    let group = node
        .props
        .get("group")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let drag = prop(node, "drag").into_owned();
    let indicator = prop(node, "indicator").into_owned();
    let tint = hex_color(&prop(node, "tint"));
    let outline = hex_color(&prop(node, "border"));
    let slots: Vec<(String, Option<u64>)> = if drag.is_empty() && indicator.is_empty() {
        Vec::new()
    } else {
        node.children
            .iter()
            .filter(|child| !is_float(child))
            .map(|child| {
                (
                    child.key.clone(),
                    env.claimed.borrow().get(&child.key).copied(),
                )
            })
            .collect()
    };
    let drag_max = node
        .props
        .get("max")
        .and_then(|value| match value {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.parse().ok(),
            _ => None,
        })
        .unwrap_or(0.0);
    let layout = env.layout;
    let popover = prop(node, "popover").into_owned();
    let hover_delay: Option<f64> = prop(node, "hover").parse().ok();
    let compact = node.props.get("hoverCompact").is_some_and(truthy);
    let floats: Vec<Resolved> = node
        .children
        .iter()
        .filter(|child| is_float(child))
        .cloned()
        .collect();
    if !container
        || (class.is_none()
            && target.is_none()
            && group.is_none()
            && drag.is_empty()
            && indicator.is_empty()
            && popover.is_empty()
            && hover_delay.is_none())
    {
        return item;
    }
    let class = class;
    let key = node.key.clone();
    item.painted_shared(move |ui, rect| {
        if kit::probing() {
            return;
        }
        if let Some(group) = &group
            && !kit::probing()
        {
            ui.ctx()
                .data_mut(|data| data.insert_temp(Id::new(("skin-group", group)), rect));
            if group == "navbar" {
                crate::navigation::set_nav_bottom(rect.bottom());
                crate::navigation::set_nav_reserve(
                    (ui.ctx().content_rect().bottom() - rect.top()).max(0.0),
                );
            }
        }
        let sense = if !drag.is_empty() || !indicator.is_empty() {
            Sense::click_and_drag()
        } else if !popover.is_empty() {
            Sense::click()
        } else if target.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui.interact(rect, Id::new(("skin-box", &key)), sense);
        if !drag.is_empty() && response.hovered() {
            ui.ctx()
                .set_cursor_icon(crate::gfx::CursorIcon::PointingHand);
        }
        if drag == "seek" && drag_max > 0.0 {
            let hover = response.hover_pos().map(|pointer| {
                let fraction = ((pointer.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);
                (rect, pointer.x, drag_max * fraction as f64)
            });
            layout.borrow_mut().seek_hover = hover.map(|(_, _, time)| time);
            ui.ctx()
                .data_mut(|data| data.insert_temp(Id::new("player-seek-hover"), hover));
        }
        if !drag.is_empty()
            && drag_max > 0.0
            && !kit::probing()
            && (response.clicked() || response.dragged())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let fraction = ((pointer.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);
            if drag == "seek" {
                layout.borrow_mut().seek_to = Some(drag_max * fraction as f64);
            }
        }
        let selectable = (!drag.is_empty() || !indicator.is_empty()) && !kit::probing();
        let mut dragged_x = None;
        let found: Vec<IndicatorSlot> = if selectable {
            slots
                .iter()
                .filter_map(|(child, node)| {
                    let (rect, flags) = ui
                        .ctx()
                        .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", child))))?;
                    Some(IndicatorSlot {
                        rect,
                        selected: flags & crate::stylesheet::SELECTED != 0,
                        node: *node,
                    })
                })
                .collect()
        } else {
            Vec::new()
        };
        if selectable && drag == "select" {
            if response.dragged() {
                dragged_x = response.interact_pointer_pos().map(|pointer| pointer.x);
            }
            if response.drag_stopped()
                && let Some(pointer) = response.interact_pointer_pos()
                && let Some(slot) =
                    found
                        .iter()
                        .filter(|slot| slot.node.is_some())
                        .min_by(|a, b| {
                            let da = (a.rect.center().x - pointer.x).abs();
                            let db = (b.rect.center().x - pointer.x).abs();
                            da.total_cmp(&db)
                        })
                && let Some(node) = slot.node
            {
                crate::interact::activate(ui, node, slot.rect);
            }
        }
        let has_focus = response.has_focus()
            || target.is_some_and(|node| crate::interact::is_focused(ui.ctx(), node));
        if !kit::probing() {
            let flags = u8::from(has_focus) * crate::stylesheet::FOCUSED
                | u8::from(response.is_pointer_button_down_on()) * crate::stylesheet::PRESSED
                | u8::from(selected) * crate::stylesheet::SELECTED;
            ui.ctx().data_mut(|data| {
                data.insert_temp(Id::new(("skin-state", &key)), (rect, flags));
            });
            crate::capture::note_state(&key, rect, flags);
        }
        if let Some((class, modifiers)) = &class {
            let ancestors = ancestor_states(ui.ctx(), &chain);
            let style = crate::stylesheet::within(&ancestors, || {
                crate::stylesheet::resolve_with(
                    ui.ctx(),
                    class,
                    response.id,
                    crate::stylesheet::State {
                        hovered: response.hovered(),
                        pressed: response.is_pointer_button_down_on(),
                        focused: has_focus,
                        enabled: !disabled,
                        selected,
                    },
                    &modifiers,
                )
            });
            let mut style = style;
            if let Some(tint) = tint {
                style.fill = tint;
                style.gradient = None;
            }
            if let Some(outline) = outline {
                style.border = outline;
                style.stroke = style.stroke.max(1.0);
            }
            let radius = style.radius.of(rect.height());
            let painter = ui.painter();
            let mut rect = rect;
            let width = rect.width();
            rect.min.x += width * start;
            rect.max.x = rect.min.x + width * span;
            paint_box(painter, rect, radius, &style);
            if style.stroke > 0.0 && style.border.a() > 0 {
                painter.rect_stroke(
                    rect,
                    radius,
                    crate::gfx::Stroke::new(style.stroke, style.border),
                    crate::gfx::StrokeKind::Inside,
                );
            }
        }
        if !indicator.is_empty() && selectable {
            paint_indicator(ui, &key, &indicator, &found, dragged_x);
        }
        if !popover.is_empty() && !floats.is_empty() && !kit::probing() {
            crate::gfx::Popup::from_toggle_button_response(&response)
                .id(Id::new(("skin-popover", &key)))
                .gap(6.0)
                .width(rect.width().max(160.0))
                .close_behavior(crate::gfx::PopupCloseBehavior::CloseOnClick)
                .frame(crate::gfx::Frame::NONE)
                .show(|ui| {
                    for float in &floats {
                        let content = build(ui, float, env);
                        kit::lay_hug(ui, content);
                    }
                });
        }
        if let Some(delay) = hover_delay
            && !floats.is_empty()
            && !kit::probing()
        {
            hover_float(
                ui,
                &key,
                rect,
                response.hovered(),
                delay,
                compact,
                &floats,
                env,
            );
        }
        if let Some(node) = target.filter(|_| !kit::probing()) {
            response.focus_node(ui, node);
        }
    })
}

pub(super) fn hover_float<'a, A: HomeAssets>(
    ui: &mut Ui,
    key: &str,
    rect: Rect,
    hovered: bool,
    delay: f64,
    compact: bool,
    floats: &[Resolved],
    env: &'a Env<'a, A>,
) {
    let ctx = ui.ctx().clone();
    let since_id = Id::new(("skin-hover", key));
    let shown_id = Id::new(("skin-hover-rect", key));
    let now = ctx.input(|input| input.time());
    let shown: Option<Rect> = ctx.data(|data| data.get_temp(shown_id));
    if shown.is_some() {
        crate::capture::mark_dirty();
    }
    let inside = shown
        .zip(hover_pos(&ctx))
        .is_some_and(|(shown, pointer)| shown.contains(pointer));
    if !hovered && !inside {
        ctx.data_mut(|data| {
            data.remove::<f64>(since_id);
            data.remove::<Rect>(shown_id);
        });
        return;
    }
    let since = ctx.data_mut(|data| match data.get_temp::<f64>(since_id) {
        Some(since) => since,
        None => {
            data.insert_temp(since_id, now);
            now
        }
    });
    let wait = since + delay - now;
    if wait > 0.0 {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(wait));
        return;
    }
    let popover = if compact {
        "card-popover-min"
    } else {
        "card-popover-max"
    };
    let width = crate::vars::scaled(popover, rect.width());
    let area = crate::gfx::Area::new(Id::new(("skin-hover-area", key)))
        .order(crate::gfx::Order::Tooltip)
        .fixed_pos(rect.center())
        .pivot(crate::gfx::Align2::CENTER_CENTER)
        .constrain(true)
        .show(&ctx, |ui| {
            ui.set_max_width(width);
            for float in floats {
                let mut sized = float.clone();
                sized.width = Some(width);
                let content = build(ui, &sized, env);
                kit::lay_hug(ui, content);
            }
        });
    let used = area.response.rect;
    ctx.data_mut(|data| data.insert_temp(shown_id, used));
}
