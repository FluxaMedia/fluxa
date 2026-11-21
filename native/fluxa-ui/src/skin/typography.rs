use super::*;

pub(super) fn text_style(style: Option<&str>) -> (Type, Role) {
    let mut kind = Type::Body;
    let mut role = Role::Primary;
    for word in style.unwrap_or_default().split_whitespace() {
        match word {
            "headline" => kind = Type::Headline,
            "title" => kind = Type::Title,
            "subtitle" => kind = Type::Subtitle,
            "body" => kind = Type::Body,
            "meta" => kind = Type::Meta,
            "label" => kind = Type::Label,
            "secondary" => role = Role::Secondary,
            "muted" => role = Role::Muted,
            _ => {}
        }
    }
    (kind, role)
}

pub(super) fn text_rule(
    ui: &Ui,
    node: &Resolved,
    chain: &[std::rc::Rc<Ancestor>],
    position: Option<(usize, usize)>,
) -> Option<crate::stylesheet::Style> {
    let (class, modifiers) = class_parts(node, position)?;
    let follow = prop(node, "follow").into_owned();
    let hovered = !follow.is_empty() && group_hovered(ui.ctx(), &follow);
    let ancestors = ancestor_states(ui.ctx(), chain);
    Some(crate::stylesheet::within(&ancestors, || {
        crate::stylesheet::resolve_with(
            ui.ctx(),
            &class,
            Id::new(("skin-text", &node.key)),
            crate::stylesheet::State {
                hovered,
                pressed: false,
                focused: false,
                enabled: true,
                selected: false,
            },
            &modifiers,
        )
    }))
}

pub(super) fn hex_color(text: &str) -> Option<Color32> {
    let digits = text.strip_prefix('#')?;
    let number = u32::from_str_radix(digits, 16).ok()?;
    if digits.len() == 8 {
        return Some(Color32::from_rgba_unmultiplied(
            (number >> 24) as u8,
            (number >> 16) as u8,
            (number >> 8) as u8,
            number as u8,
        ));
    }
    Some(Color32::from_rgb(
        (number >> 16) as u8,
        (number >> 8) as u8,
        number as u8,
    ))
}

pub(super) fn text<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let content = node.text.clone().unwrap_or_default();
    let (kind, role) = text_style(node.style.as_deref());
    let section = node
        .style
        .as_deref()
        .is_some_and(|style| style.split_whitespace().any(|word| word == "section"));
    let section_size = crate::vars::px("screen-section-title-size");
    if !crate::emoji::contains(&content) {
        let style = text_rule(
            ui,
            node,
            &env.ancestors.borrow(),
            env.positions.get(&node.key).copied(),
        );
        let width = style.as_ref().and_then(|style| style.max_width);
        let opacity = style.as_ref().map_or(1.0, |style| style.opacity);
        let color = hex_color(
            node.props
                .get("color")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        )
        .or_else(|| {
            style
                .as_ref()
                .filter(|style| style.text_declared)
                .map(|style| style.text)
        });
        let font_size = style.as_ref().and_then(|style| style.font_size);
        let bold = style
            .as_ref()
            .and_then(|style| style.weight)
            .is_some_and(|weight| weight >= 600.0);
        if let Some(style) = style.as_ref().filter(|style| shaped(style)) {
            let style = *style;
            let mut label = kit::text(content.clone(), kind).role(role).inert();
            if bold {
                label = label.bold();
            }
            if let Some(color) = color {
                label = label.color(color);
            }
            if let Some(size) = font_size {
                label = label.size(size);
            }
            let (galley, tint) = label.galley(ui, f32::INFINITY);
            let inset = style.padding_box.map_or([style.padding; 4], |sides| sides);
            let size = galley.size() + Vec2::new(inset[1] + inset[3], inset[0] + inset[2]);
            return kit::direct(size, move |painter, rect| {
                paint_shape(painter, rect, &style);
                let anchor = Pos2::new(
                    rect.center().x + (inset[3] - inset[1]) * 0.5,
                    rect.center().y + (inset[0] - inset[2]) * 0.5,
                );
                let shape =
                    crate::gfx::epaint::TextShape::new(anchor - galley.size() * 0.5, galley, tint)
                        .with_angle_and_anchor(
                            style.rotate.to_radians(),
                            crate::gfx::Align2::CENTER_CENTER,
                        );
                painter.add(shape);
            });
        }
        let lines = node
            .props
            .get("lines")
            .and_then(|value| value.as_u64())
            .filter(|lines| *lines > 0)
            .map(|lines| lines as usize);
        if let Some(rows) = lines {
            let size = font_size.unwrap_or(crate::stylesheet::hero_synopsis_size());
            let color = color.unwrap_or(crate::metrics::theme().text_primary);
            let grown = match EXPAND.get() {
                Some((ease, Some(base))) if ease > 0.0 => Some(expanded_width(base as f32, ease)),
                _ => None,
            };
            let centered = prop(node, "center") == "true";
            let inset = prop(node, "inset").parse::<f32>().unwrap_or(0.0);
            let explicit = node.width.or(grown.map(|width| width - inset * 2.0));
            let inherited = env.inner.borrow().last().copied().flatten();
            let font = match style.as_ref().and_then(|style| style.weight) {
                Some(weight) if weight >= 600.0 => crate::fonts::bold(size),
                Some(weight) if weight >= 500.0 => crate::fonts::medium(size),
                _ => crate::fonts::regular(size),
            };
            if opacity >= 1.0 {
                let wrap_width = explicit.or(inherited).unwrap_or(560.0);
                let mut job = crate::gfx::text::LayoutJob::simple(
                    content.clone(),
                    font.clone(),
                    color,
                    wrap_width,
                );
                job.wrap.max_rows = rows;
                job.wrap.overflow_character = Some('…');
                if centered {
                    job.halign = crate::gfx::Align::Center;
                }
                let galley = ui.ctx().fonts_mut(|fonts| fonts.layout_job(job));
                let size = if centered {
                    Vec2::new(wrap_width, galley.size().y)
                } else {
                    galley.size()
                };
                let ctx = ui.ctx().clone();
                let fit_rows = galley.rows.len().max(1);
                return kit::direct(size, move |painter, rect| {
                    let galley = if rect.width() + 0.5 < size.x {
                        let mut job = crate::gfx::text::LayoutJob::simple(
                            content.clone(),
                            font.clone(),
                            color,
                            rect.width(),
                        );
                        job.wrap.max_rows = fit_rows;
                        job.wrap.overflow_character = Some('…');
                        if centered {
                            job.halign = crate::gfx::Align::Center;
                        }
                        ctx.fonts_mut(|fonts| fonts.layout_job(job))
                    } else {
                        galley
                    };
                    let origin = if centered {
                        crate::gfx::Pos2::new(rect.center().x, rect.min.y)
                    } else {
                        rect.min
                    };
                    painter.galley_with_override_text_color(origin, galley, color);
                })
                .min_width(0.0)
                .shrink(1.0);
            }
            return kit::widget(ui, move |ui| {
                if opacity < 1.0 {
                    ui.multiply_opacity(opacity);
                }
                let wrap_width = explicit.or(inherited).unwrap_or(560.0);
                let mut job = crate::gfx::text::LayoutJob::simple(
                    content.clone(),
                    font.clone(),
                    color,
                    wrap_width,
                );
                job.wrap.max_rows = rows;
                job.wrap.overflow_character = Some('…');
                if centered {
                    job.halign = crate::gfx::Align::Center;
                }
                let galley = ui.ctx().fonts_mut(|fonts| fonts.layout_job(job));
                let size = if centered {
                    Vec2::new(wrap_width, galley.size().y)
                } else {
                    galley.size()
                };
                let (rect, _) = ui.allocate_exact_size(size, crate::gfx::Sense::hover());
                let origin = if centered {
                    crate::gfx::Pos2::new(rect.center().x, rect.min.y)
                } else {
                    rect.min
                };
                ui.painter()
                    .galley_with_override_text_color(origin, galley, color);
            });
        }
        if opacity >= 1.0 {
            let mut text = kit::text(content, kind).role(role).inert();
            if bold {
                text = text.bold();
            }
            if let Some(color) = color {
                text = text.color(color);
            }
            if let Some(size) = font_size {
                text = text.size(size);
            } else if section {
                text = text.size(section_size);
            }
            let wrap = ui.available_rect_before_wrap().width();
            let (galley, tint) = text.galley(ui, wrap);
            let size = galley.size();
            let ctx = ui.ctx().clone();
            let item = kit::direct(size, move |painter, rect| {
                let galley = if rect.width() + 0.5 < size.x {
                    let ui_ctx = &ctx;
                    let mut job = crate::gfx::text::LayoutJob::simple(
                        galley.job.text.clone(),
                        galley.job.sections[0].format.font_id.clone(),
                        tint,
                        rect.width(),
                    );
                    job.wrap.max_width = rect.width();
                    ui_ctx.fonts_mut(|fonts| fonts.layout_job(job))
                } else {
                    galley
                };
                match width {
                    Some(width) => {
                        let clip =
                            crate::flex::width_clip(rect, width).intersect(painter.clip_rect());
                        painter.with_clip_rect(clip).galley(rect.min, galley, tint);
                    }
                    None => painter.galley(rect.min, galley, tint),
                }
            });
            return match width {
                Some(width) => item.max_width(width),
                None => item,
            };
        }
        let item = kit::widget(ui, move |ui| {
            if opacity < 1.0 {
                ui.multiply_opacity(opacity);
            }
            if let Some(width) = width {
                let clip = crate::flex::width_clip(ui.max_rect(), width);
                ui.set_clip_rect(clip.intersect(ui.clip_rect()));
            }
            let mut text = kit::text(content.clone(), kind).role(role).inert();
            if bold {
                text = text.bold();
            }
            if let Some(color) = color {
                text = text.color(color);
            }
            if let Some(size) = font_size {
                text = text.size(size);
            }
            if section && font_size.is_none() {
                text.size(section_size).show(ui);
            } else {
                text.show(ui);
            }
        });
        return match width {
            Some(width) => item.max_width(width),
            None => item,
        };
    }
    let assets = env.assets;
    kit::widget(ui, move |ui| {
        crate::emoji::label(
            ui,
            &mut **assets.borrow_mut(),
            &content,
            if section { section_size } else { kind.size() },
            role.color(),
            crate::emoji::Label::default(),
        );
    })
}
