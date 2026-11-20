use super::*;

#[derive(Clone, Debug)]
pub struct VideoRequest {
    pub item: String,
    pub urls: Vec<String>,
    pub delay: f64,
}

#[derive(Clone, Debug)]
pub struct PlayingVideo {
    pub item: String,
    pub texture: crate::gfx::TextureId,
    pub subtitle: Option<String>,
}

pub fn take_video_request(context: &crate::gfx::Context) -> Option<VideoRequest> {
    context.data_mut(|data| {
        let request = data.get_temp::<VideoRequest>(Id::new("skin-video-request"));
        data.remove::<VideoRequest>(Id::new("skin-video-request"));
        request
    })
}

pub fn set_playing_video(context: &crate::gfx::Context, playing: Option<PlayingVideo>) {
    context.data_mut(|data| match playing {
        Some(playing) => data.insert_temp(Id::new("skin-video-playing"), playing),
        None => data.remove::<PlayingVideo>(Id::new("skin-video-playing")),
    });
}

pub(super) fn video<'a>(node: &Resolved) -> kit::Item<'a> {
    let item = prop(node, "item").into_owned();
    let delay = node
        .props
        .get("delay")
        .and_then(|value| match value {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.parse().ok(),
            _ => None,
        })
        .unwrap_or(2.0)
        .max(0.0);
    let urls: Vec<String> = node
        .props
        .get("urls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|url| url.as_str().map(str::to_owned))
        .collect();
    let subtitles = node.props.get("subtitles").is_some_and(truthy);
    if prop(node, "source") == "player" {
        let radius = node
            .props
            .get("radius")
            .and_then(Value::as_f64)
            .unwrap_or(0.0) as f32;
        return kit::item(Vec2::ZERO, move |ui, rect| {
            if kit::probing() {
                return;
            }
            let texture = ui
                .ctx()
                .data(|data| data.get_temp::<crate::gfx::TextureId>(Id::new("skin-player-video")));
            ui.painter().rect_filled(rect, radius, Color32::BLACK);
            if let Some(texture) = texture {
                crate::components::rounded_image(
                    ui.painter(),
                    texture,
                    rect,
                    radius,
                    crate::paint::full_uv(),
                    Color32::WHITE,
                );
            }
        })
        .grow(1.0)
        .align_self(FlexAlign::Stretch);
    }
    kit::item(Vec2::ZERO, move |ui, rect| {
        if kit::probing() || item.is_empty() {
            return;
        }
        let context = ui.ctx().clone();
        context.data_mut(|data| {
            data.insert_temp(
                Id::new("skin-video-request"),
                VideoRequest {
                    item: item.to_string(),
                    urls: urls.clone(),
                    delay,
                },
            );
        });
        let playing = context
            .data(|data| data.get_temp::<PlayingVideo>(Id::new("skin-video-playing")))
            .filter(|playing| playing.item == item);
        if let Some(playing) = playing {
            crate::paint::paint_trailer(
                &context,
                ui.painter(),
                playing.texture,
                rect,
                Some(&item),
                255,
            );
            if subtitles && let Some(text) = &playing.subtitle {
                crate::paint::paint_trailer_subtitle(&context, rect, text);
            }
        }
    })
    .grow(1.0)
    .align_self(FlexAlign::Stretch)
}

pub(super) fn image<'a, A: HomeAssets>(
    _ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let source = node.src.clone();
    let assets = env.assets;
    if prop(node, "fit") == "cover" {
        let focus_y = node
            .props
            .get("focus")
            .and_then(|value| value.as_f64())
            .map_or(0.5, |value| value.clamp(0.0, 1.0) as f32);
        let view_width = env.view.get().x;
        let item = kit::item(Vec2::ZERO, move |ui, rect| {
            let mut assets = assets.borrow_mut();
            let target = crate::backdrop_target_size(view_width, ui.ctx().pixels_per_point());
            let texture = assets
                .texture_for(source.as_deref(), target, ArtworkPriority::Hero)
                .or_else(|| assets.cached_texture(source.as_deref()));
            if let Some(texture) = texture {
                let size = assets.texture_size(source.as_deref()).unwrap_or(target);
                let profile = assets.artwork_focus(source.as_deref());
                let mut focus = crate::paint::framed_focus(size, rect, profile.as_deref());
                if profile.is_none() {
                    focus[1] = focus_y;
                }
                ui.painter().with_clip_rect(rect).image(
                    texture,
                    rect,
                    crate::paint::cover_uv_focus(size, rect, focus),
                    Color32::from_white_alpha(210),
                );
            }
        });
        return match node.width.zip(node.height) {
            Some((width, height)) => item.width(width).height(height).shrink(0.0),
            None => item.grow(1.0).align_self(FlexAlign::Stretch),
        };
    }
    let share = prop(node, "share").parse::<f32>().ok();
    let (width, share_final) = match (share, EXPAND.get()) {
        (Some(share), Some((ease, Some(base)))) => (
            expanded_width(base as f32, ease, env.metrics) * share,
            Some(expanded_width(base as f32, 1.0, env.metrics) * share),
        ),
        _ => (node.width.unwrap_or(env.metrics.poster_card_width), None),
    };
    let size = if node.fill {
        Vec2::ZERO
    } else {
        Vec2::new(width, node.height.unwrap_or(width * 1.5))
    };
    let contain = prop(node, "fit") == "contain";
    let autoplay = prop(node, "autoplay") != "false";
    let start = prop(node, "position") == "start";
    let reveal = prop(node, "reveal").parse::<f32>().ok();
    let motion = prop(node, "motion").into_owned();
    let metrics = env.metrics;
    let keys = ancestor_keys(env);
    let overlay_card = prop(node, "overlayCard").into_owned();
    let logo_of = prop(node, "logoOf").into_owned();
    let fallback = prop(node, "fallback").into_owned();
    let number = |name: &str| prop(node, name).parse::<f32>().ok();
    let final_size = number("expandedW")
        .zip(number("expandedH"))
        .map(|(w, h)| Vec2::new(w, h))
        .or_else(|| share_final.map(|w| Vec2::new(w, node.height.unwrap_or(w))));
    let poster_size = number("posterW").map(|w| Vec2::new(w, w * 1.5));
    let rank = prop(node, "rank").parse::<usize>().ok();
    let styled = class_parts(node, env.positions.get(&node.key).copied())
        .map(|(rule, modifiers)| crate::stylesheet::base_with(&rule, &modifiers));
    kit::shared(size, move |ui, rect| {
        let rect = lens(ui.ctx(), &keys, rect);
        let source = if logo_of.is_empty() {
            source.clone()
        } else {
            crate::card_registry::card(&logo_of).and_then(|card| {
                crate::card_logos::want(&card);
                crate::card_logos::logo_of(&card)
            })
        };
        let source = match card_art(ui, &overlay_card) {
            Some(url) => Some(url),
            None => source,
        };
        let radius = styled
            .as_ref()
            .map_or(metrics.card_radius, |style| style.radius.of(rect.height()))
            .min(rect.width().min(rect.height()) * 0.5);
        let fill = styled
            .as_ref()
            .map_or(Color32::TRANSPARENT, |style| style.fill);
        let tint = styled.as_ref().map_or(Color32::WHITE, |style| {
            Color32::from_white_alpha((style.opacity * 255.0) as u8)
        });
        let mut assets = assets.borrow_mut();
        if fill.a() > 0 {
            ui.painter().rect_filled(rect, radius, fill);
        }
        let ppp = ui.ctx().pixels_per_point();
        let target = crate::artwork_target_size(final_size.unwrap_or(rect.size()), ppp);
        let mut source = source;
        let texture = match assets.texture_for(source.as_deref(), target, ArtworkPriority::Visible)
        {
            Some(texture) => texture,
            None => {
                let held = (!fallback.is_empty())
                    .then(|| {
                        let size = poster_size.unwrap_or(rect.size());
                        assets.texture_for(
                            Some(&*fallback),
                            crate::artwork_target_size(size, ppp),
                            ArtworkPriority::Visible,
                        )
                    })
                    .flatten();
                let Some(held) = held else {
                    return;
                };
                source = Some(fallback.to_string());
                held
            }
        };
        let (dest, uv) = match (contain, assets.texture_size(source.as_deref())) {
            (true, Some(size)) => {
                let mut dest = kit::fit_inside(rect, size);
                if start {
                    dest = dest.translate(Vec2::new(rect.left() - dest.left(), 0.0));
                }
                (dest, crate::paint::full_uv())
            }
            (false, Some(size)) => (rect, crate::paint::cover_uv(size, rect)),
            _ => (rect, crate::paint::full_uv()),
        };
        let radius = if contain { 0.0 } else { radius };
        match reveal {
            Some(share) => {
                let share = share.clamp(0.0, 1.0);
                let ghost = Color32::from_white_alpha(tint.a() / 4);
                crate::components::rounded_image(ui.painter(), texture, dest, radius, uv, ghost);
                let mut clip = dest;
                clip.set_right(dest.left() + dest.width() * share);
                let painter = ui.painter().with_clip_rect(clip);
                crate::components::rounded_image(&painter, texture, dest, radius, uv, tint);
            }
            None => crate::components::rounded_image(ui.painter(), texture, dest, radius, uv, tint),
        }
        if !motion.is_empty() && !kit::probing() {
            motion_overlay(ui, rect, &motion, autoplay, &mut **assets);
        }
        if kit::probing() {
            return;
        }
        if let Some(card) = (!overlay_card.is_empty())
            .then(|| crate::card_registry::card(&overlay_card))
            .flatten()
        {
            let custom = crate::poster_overlay::custom_url(ui.painter().ctx(), &card).is_some();
            if card.is_landscape() {
                if !custom {
                    crate::components::landscape_logo(ui.painter(), rect, &card, &mut **assets);
                }
                crate::poster_overlay::paint_landscape_band(ui.painter(), rect, &card);
            } else {
                if let Some(logo) = crate::poster_overlay::textless_art(ui.painter().ctx(), &card)
                    .and_then(|art| art.logo)
                {
                    crate::components::textless_logo(ui.painter(), rect, &logo, &mut **assets);
                }
                let tones = assets.artwork_tones(card.artwork_url.as_deref());
                crate::poster_overlay::paint(ui.painter(), rect, &card, radius, tones);
            }
        }
        if let Some(rank) = rank {
            crate::poster_overlay::paint_beside(
                ui.painter(),
                rect,
                rank,
                metrics.accent,
                metrics.background,
            );
            crate::poster_overlay::paint_overlay(
                ui.painter(),
                rect,
                rank,
                metrics.accent,
                metrics.background,
            );
        }
    })
}

pub(super) fn card_art(ui: &Ui, id: &str) -> Option<String> {
    let card = crate::card_registry::card(id)?;
    crate::poster_overlay::custom_url(ui.painter().ctx(), &card).or_else(|| {
        card.is_landscape()
            .then(|| card.poster_art().map(str::to_owned))
            .flatten()
    })
}

pub(super) type PagerState = (usize, f64, Option<usize>, f64);

pub(super) fn pager_step(state: PagerState, count: usize, delta: isize, now: f64) -> PagerState {
    let (index, ..) = state;
    let next = (index as isize + delta).rem_euclid(count as isize) as usize;
    (next, now, Some(index), now)
}

pub(super) fn faded<'a>(item: kit::Item<'a>, alpha: f32, height: Option<f32>) -> kit::Item<'a> {
    let wrapper = kit::item(Vec2::ZERO, move |ui, rect| {
        ui.set_opacity(alpha);
        kit::lay_in(ui, rect, item);
    })
    .grow(1.0)
    .align_self(FlexAlign::Stretch);
    match height {
        Some(height) => wrapper.height(height),
        None => wrapper,
    }
}

pub(super) fn pager<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let count = node.children.len();
    if count == 0 {
        return layout_items::column(Vec::new());
    }
    let interval = node
        .props
        .get("interval")
        .and_then(|value| value.as_f64())
        .unwrap_or(8.0)
        .max(1.0);
    let id = Id::new(("skin-pager", &node.key));
    let drag_id = Id::new(("skin-pager-drag", &node.key));
    let now = ui.ctx().input(|input| input.time());
    let mut state = ui
        .ctx()
        .data(|data| data.get_temp::<PagerState>(id))
        .unwrap_or((0, now, None, f64::NEG_INFINITY));
    if state.0 >= count {
        state = (0, now, None, f64::NEG_INFINITY);
    }
    if count > 1 && now - state.1 >= interval {
        state = pager_step(state, count, 1, now);
    }
    ui.ctx().data_mut(|data| data.insert_temp(id, state));
    if count > 1 {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(interval));
    }
    let linear = ((now - state.3) / 0.6).clamp(0.0, 1.0) as f32;
    let eased = 1.0 - (1.0 - linear).powi(3);
    let (index, previous) = (state.0, state.2.filter(|previous| *previous < count));
    let height_of = |child: &Resolved| {
        child
            .height
            .or(child.height_vh.map(|share| share * env.view.get().y))
    };
    let mut layers = Vec::new();
    if let Some(previous) = previous.filter(|_| linear < 1.0) {
        ui.ctx().request_repaint();
        layers.push(faded(
            build(ui, &node.children[previous], env),
            1.0,
            height_of(&node.children[previous]),
        ));
        layers.push(faded(
            build(ui, &node.children[index], env),
            eased,
            height_of(&node.children[index]),
        ));
    } else {
        layers.push(
            build(ui, &node.children[index], env)
                .grow(1.0)
                .align_self(FlexAlign::Stretch),
        );
    }
    if count > 1 {
        layers.push(
            kit::item(Vec2::ZERO, move |ui, rect| {
                let context = ui.ctx().clone();
                let pointer = context.input(|input| input.pointer().interact_pos());
                if context.input(|input| input.pointer().primary_pressed())
                    && let Some(position) = pointer.filter(|position| rect.contains(*position))
                {
                    context.data_mut(|data| data.insert_temp(drag_id, position));
                }
                if !context.input(|input| input.pointer().primary_released()) {
                    return;
                }
                let Some(start) =
                    context.data_mut(|data| data.remove_temp::<crate::gfx::Pos2>(drag_id))
                else {
                    return;
                };
                let Some(end) = pointer else {
                    return;
                };
                let (dx, dy) = (end.x - start.x, end.y - start.y);
                if dx.abs() < 60.0 || dy.abs() > dx.abs() {
                    return;
                }
                let now = context.input(|input| input.time());
                let current = context
                    .data(|data| data.get_temp::<PagerState>(id))
                    .unwrap_or((0, now, None, f64::NEG_INFINITY));
                let delta = if dx < 0.0 { 1 } else { -1 };
                context
                    .data_mut(|data| data.insert_temp(id, pager_step(current, count, delta, now)));
                context.request_repaint();
            })
            .grow(1.0)
            .align_self(FlexAlign::Stretch),
        );
    }
    if layers.len() == 1 {
        return layers.remove(0);
    }
    layout_items::stack(layers).align_self(FlexAlign::Stretch)
}

pub(super) fn qr<'a>(node: &Resolved) -> kit::Item<'a> {
    let modules: Vec<bool> = prop(node, "modules").chars().map(|c| c == '1').collect();
    let count = prop(node, "count").parse::<usize>().unwrap_or(0);
    let side = node
        .props
        .get("size")
        .and_then(|value| value.as_f64())
        .map_or(240.0, |size| size as f32);
    kit::item(Vec2::splat(side), move |ui, rect| {
        let painter = ui.painter();
        painter.rect_filled(rect, 16.0, Color32::WHITE);
        if count == 0 || modules.len() < count * count {
            return;
        }
        let inner = rect.shrink(side * 0.06);
        let cell = inner.width() / count as f32;
        for (index, dark) in modules.iter().enumerate().take(count * count) {
            if !*dark {
                continue;
            }
            let min = inner.min + Vec2::new((index % count) as f32, (index / count) as f32) * cell;
            painter.rect_filled(
                Rect::from_min_max(min, min + Vec2::splat(cell.ceil())),
                0.0,
                Color32::BLACK,
            );
        }
    })
}

pub fn set_player_video(context: &crate::gfx::Context, texture: Option<crate::gfx::TextureId>) {
    context.data_mut(|data| match texture {
        Some(texture) => data.insert_temp(Id::new("skin-player-video"), texture),
        None => data.remove::<crate::gfx::TextureId>(Id::new("skin-player-video")),
    });
}
