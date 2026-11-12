use super::*;
use crate::flex::Insets;
use crate::gfx::Ui;

#[derive(Clone, Debug)]
pub struct ExpandedPoster {
    pub node: u64,
    pub id: String,
    pub raw: serde_json::Value,
    pub trailer: bool,
}

pub fn expanded_poster(context: &crate::gfx::Context) -> Option<ExpandedPoster> {
    context.data(|data| data.get_temp(Id::new("fluxa-expanded-poster")))
}

pub fn set_episode_details(
    context: &crate::gfx::Context,
    details: std::sync::Arc<std::collections::HashMap<String, serde_json::Value>>,
) {
    context.data_mut(|data| data.insert_temp(Id::new("fluxa-episode-details"), details));
}

fn with_episode_details(context: &crate::gfx::Context, card: &HomeCard) -> Option<HomeCard> {
    let video = card.raw.get("lastVideoId")?.as_str()?;
    let details = context.data(|data| {
        data.get_temp::<std::sync::Arc<std::collections::HashMap<String, serde_json::Value>>>(
            Id::new("fluxa-episode-details"),
        )
    })?;
    let found = details.get(video)?;
    let mut card = card.clone();
    let raw = card.raw.to_mut().as_object_mut()?;
    for (target, source) in [
        ("lastEpisodeOverview", "overview"),
        ("lastEpisodeThumbnail", "thumbnail"),
    ] {
        let present = raw
            .get(target)
            .and_then(|value| value.as_str())
            .is_some_and(|text| !text.trim().is_empty());
        if let Some(text) = found
            .get(source)
            .filter(|text| text.is_string())
            .filter(|_| !present)
        {
            raw.insert(target.into(), text.clone());
        }
    }
    Some(card)
}

pub fn previewed_poster(context: &crate::gfx::Context) -> Option<ExpandedPoster> {
    context.data(|data| data.get_temp(Id::new("fluxa-previewed-poster")))
}

#[derive(Clone, Default)]
struct ExpandState {
    focus: Option<(u64, f64)>,
    progress: Vec<(u64, f32)>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_home_rows(
    ui: &mut Ui,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
    metrics: UiMetrics,
    tail: f32,
    prefetch_home_artwork: bool,
    margin: f32,
    screen: Rect,
    layout: &mut HomeLayout,
    activated: &mut Option<u64>,
) {
    let context = &ui.ctx().clone();
    let mut flat_index = 0usize;
    let preview_id = Id::new("fluxa-hover-preview-state");
    let preview_state = context.data(|data| data.get_temp::<(u64, f64, Rect)>(preview_id));
    let hover_pos = (viewport.form_factor == UiFormFactor::Desktop
        && !components::screen_covered(context))
    .then(|| context.input(|input| input.pointer().hover_pos()))
    .flatten()
    .filter(|pos| !components::over_mini_player(context, *pos) && !over_top_bar(context, *pos));
    let pointer = hover_pos.filter(|_| home.poster_hover_preview && !home.expanded_posters);
    let expand_hover_id = Id::new("fluxa-expand-hover");
    let hover_node = home
        .expanded_posters
        .then(|| context.data(|data| data.get_temp::<u64>(expand_hover_id)))
        .flatten();
    let mut next_hover_node: Option<u64> = None;
    let mut direct_hover: Option<(u64, Rect, &HomeCard)> = None;
    let mut popup_hover: Option<(u64, Rect, &HomeCard)> = None;
    let now = context.input(|input| input.time());
    let expand_id = Id::new("fluxa-expand-state");
    let mut expand = context
        .data(|data| data.get_temp::<ExpandState>(expand_id))
        .unwrap_or_default();
    let focus_node = hover_node.or_else(|| {
        focused.filter(|node| {
            home.expanded_posters && (NODE_CARD_BASE..NODE_PREVIEW_BASE).contains(node)
        })
    });
    if expand.focus.map(|(node, _)| node) != focus_node {
        expand.focus = focus_node.map(|node| (node, now));
    }
    let expanded_node = expand
        .focus
        .filter(|(_, since)| now - since >= home.expanded_posters_delay)
        .map(|(node, _)| node);
    let snap = home.expanded_posters_delay <= 0.0;
    let step =
        context.input(|input| input.stable_dt()).min(0.05) / metrics.screen_poster_expand_seconds;
    if let Some(node) = expanded_node
        && !expand.progress.iter().any(|(id, _)| *id == node)
    {
        expand.progress.push((node, 0.0));
    }
    for (node, progress) in &mut expand.progress {
        let open = Some(*node) == expanded_node;
        *progress = match (open, snap) {
            (true, true) => 1.0,
            (false, true) => 0.0,
            (true, false) => (*progress + step).min(1.0),
            (false, false) => (*progress - step).max(0.0),
        };
    }
    expand
        .progress
        .retain(|(node, progress)| *progress > 0.0 || Some(*node) == expanded_node);
    let moving = expand.progress.iter().any(|(node, progress)| {
        if Some(*node) == expanded_node {
            *progress < 1.0
        } else {
            *progress > 0.0
        }
    });
    if moving {
        context.request_repaint();
    } else if let Some((_, since)) = expand.focus
        && expanded_node.is_none()
    {
        let wait = (home.expanded_posters_delay - (now - since)).max(0.0);
        context.request_repaint_after(std::time::Duration::from_secs_f64(wait));
    }
    let expand_ease = |node: u64| {
        expand
            .progress
            .iter()
            .find(|(id, _)| *id == node)
            .map_or(0.0, |(_, progress)| {
                crate::gfx::emath::easing::cubic_out(*progress)
            })
    };
    context.data_mut(|data| data.insert_temp(expand_id, expand.clone()));
    let mut expanded_card: Option<ExpandedPoster> = None;
    let mut expanded_panels: Vec<(u64, Rect, &HomeCard, f32)> = Vec::new();
    let shelf_count = home_shelves(metrics, home, 0.0).take(64).count();
    for shelf in home_shelves(metrics, home, 0.0).take(64) {
        let (row_index, title, cards, kind) = (shelf.index, shelf.title, shelf.cards, shelf.kind);
        let last_shelf = row_index + 1 == shelf_count;
        let gap = if last_shelf { 0.0 } else { shelf_gap(metrics) };
        let slot = ui
            .allocate_space(Vec2::new(viewport.width, shelf.heading + shelf.body + gap))
            .1;
        let row_y = slot.min.y;
        let (card_width, card_height, _) = home_row_dimensions(metrics, kind);
        let body_height = shelf.body;
        let expand_extra = if kind == HomeRowKind::Poster {
            let wide = card_height * metrics.screen_poster_expand_aspect;
            (0..cards.len())
                .map(|index| {
                    let ease = expand_ease(NODE_CARD_BASE + (flat_index + index) as u64);
                    (wide - card_width).max(0.0) * ease
                })
                .sum::<f32>()
        } else {
            0.0
        };
        let row_scroll_max =
            home_row_scroll_max_for_cards(viewport, metrics, title, cards, kind) + expand_extra;
        let row_height = home_row_heading_height(metrics) + body_height;
        let visible_y = row_y;
        let row_is_visible = visible_y + row_height >= 0.0 && visible_y <= viewport.height;

        // Prefetch the complete home document, not just the visible row. This
        // is still lazy from the renderer's perspective: requests are async,
        // bounded by the host fetcher, decoded off-thread, and uploaded only
        // as completed images arrive. It prevents the web/Compose behaviour
        // difference where scrolling is required to start loading artwork.
        if prefetch_home_artwork {
            for card in cards {
                assets.prefetch_for(
                    card.poster_art(),
                    artwork_target_size(
                        Vec2::from(home_card_dimensions(metrics, card, kind)),
                        context.pixels_per_point(),
                    ),
                    ArtworkPriority::Prefetch,
                );
                if home.gif_autoplay_enabled
                    && !viewport.is_tv()
                    && kind == HomeRowKind::Collection
                    && card.motion_enabled
                    && !row_is_visible
                    && let Some(url) = card.motion_url.as_deref()
                {
                    let (width, height) = home_collection_card_dimensions(metrics, card);
                    assets.prefetch_animated_for(
                        Some(url),
                        fluxa_artwork::animation_target_size(artwork_target_size(
                            Vec2::new(width, height),
                            context.pixels_per_point(),
                        )),
                        ArtworkPriority::Prefetch,
                    );
                }
            }
        }
        let above_reach = if viewport.is_tv() { row_height } else { 0.0 };
        if visible_y + row_height + above_reach < 0.0 || visible_y > viewport.height + row_height {
            flat_index += cards.len();
            continue;
        }
        let heading_height = home_row_heading_height(metrics);
        let type_label = row_index
            .checked_sub(usize::from(home.has_continue_row()))
            .and_then(|index| home.rows.get(index))
            .and_then(|row| row.type_label.as_deref());
        {
            let mut ui = ui.new_child(crate::gfx::UiBuilder::new().max_rect(crate::home::lane(
                Rect::from_x_y_ranges(0.0..=viewport.width, visible_y..=visible_y + heading_height),
                Insets::xy(margin, 0.0),
            )));
            {
                ui.set_clip_rect(ui.clip_rect().intersect(screen));
                let ui = &mut ui;
                let title_size = metrics.row_heading_size;
                let title = match type_label {
                    Some(label) => format!("{title} - {label}"),
                    None => title.to_owned(),
                };
                crate::emoji::label(
                    ui,
                    assets,
                    &title,
                    title_size,
                    ui.strong_text_color(),
                    crate::emoji::Label::default(),
                );
            }
        }
        let row_clip = Rect::from_min_max(
            Pos2::new(0.0, visible_y + heading_height),
            Pos2::new(
                viewport.width,
                (visible_y + heading_height + body_height).min(viewport.height),
            ),
        );
        crate::report_scroll_region(
            context,
            crate::ScrollRegion {
                id: crate::ScrollId::Row(row_index),
                axis: crate::ScrollAxis::Horizontal,
                rect: row_clip,
                max: row_scroll_max,
            },
        );
        let row_scroll_offset = crate::layout::resolve_row_scroll(
            context,
            row_index,
            home.row_scroll_offsets
                .get(row_index)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, row_scroll_max),
            viewport.form_factor == UiFormFactor::Desktop,
        );
        if viewport.form_factor == UiFormFactor::Desktop
            && let Some(row) = home
                .rows
                .get(row_index.saturating_sub(usize::from(home.has_continue_row())))
            && row.can_load_more
            && !cards.is_empty()
            && let Some(row_id) = row.id.as_deref().filter(|value| !value.is_empty())
        {
            let max_offset = row_scroll_max;
            let card_pitch = (card_width + metrics.horizontal_spacing).max(1.0);
            let prefetch = (viewport.width * 2.0).max(card_pitch * 6.0);
            if row_scroll_offset >= (max_offset - prefetch).max(0.0) {
                let request_id = Id::new(("fluxa-home-load-more", row_id));
                let already_requested = context.data_mut(|data| {
                    let previous = data.get_temp::<usize>(request_id).unwrap_or(0);
                    if previous == cards.len() {
                        true
                    } else {
                        data.insert_temp(request_id, cards.len());
                        false
                    }
                });
                if !already_requested && let Some(category) = row.catalog_page.as_ref() {
                    let content_type = category
                        .get("contentType")
                        .or_else(|| category.get("type"))
                        .and_then(serde_json::Value::as_str)
                        .filter(|value| matches!(*value, "movie" | "series"))
                        .unwrap_or("movie");
                    layout.load_more.push(serde_json::json!({
                        "type": "catalogPageRequested",
                        "categoryId": row_id,
                        "transportUrl": category.get("addonTransportUrl").or_else(|| category.get("transportUrl")),
                        "contentType": content_type,
                        "catalogId": category.get("catalogId").or_else(|| category.get("id")),
                        "skip": category.get("skip").and_then(serde_json::Value::as_i64).unwrap_or(0) + cards.len() as i64,
                        "genre": category.get("addonGenre").or_else(|| category.get("genre")),
                        "remoteSource": category.get("remoteSources").or_else(|| category.get("remoteSource")),
                    }));
                }
            }
        }
        if cards.is_empty() && kind == HomeRowKind::Continue {
            {
                {
                    let strip = Rect::from_x_y_ranges(
                        0.0..=viewport.width,
                        visible_y + heading_height..=visible_y + heading_height + card_height,
                    );
                    let mut ui = ui.new_child(
                        crate::gfx::UiBuilder::new()
                            .max_rect(crate::home::lane(strip, Insets::xy(margin, 0.0))),
                    );
                    let ui = &mut ui;
                    ui.set_clip_rect(ui.clip_rect().intersect(row_clip).intersect(screen));
                    let pitch = card_width + metrics.horizontal_spacing;
                    let count = ((viewport.width - margin) / pitch).ceil().max(0.0) as usize;
                    let cards = (0..count)
                        .map(|_| {
                            crate::kit::item(Vec2::new(card_width, card_height), |ui, rect| {
                                components::skeleton_card(ui.painter(), rect, metrics);
                            })
                            .shrink(0.0)
                        })
                        .collect();
                    crate::kit::lay_in(
                        ui,
                        strip,
                        crate::kit::layout_items::row(cards)
                            .gap(metrics.horizontal_spacing)
                            .padding(Insets::xy(margin, 0.0)),
                    );
                }
            }
        }
        {
            let mut ui = ui.new_child(crate::gfx::UiBuilder::new().max_rect(crate::gfx::rect_at(
                Pos2::new(margin - row_scroll_offset, visible_y + heading_height),
                Vec2::new(row_scroll_max + viewport.width, body_height),
            )));
            {
                let ui = &mut ui;
                let lift_room = if home.poster_lift {
                    metrics.card_lift
                } else {
                    0.0
                };
                let cards_clip =
                    Rect::from_min_max(row_clip.min - Vec2::new(0.0, lift_room), row_clip.max);
                ui.set_clip_rect(ui.clip_rect().intersect(cards_clip).intersect(screen));
                let card_rects = shelf_card_rects(
                    metrics,
                    title,
                    kind,
                    cards,
                    ui.max_rect().min,
                    |index, width, height| {
                        let node_id = NODE_CARD_BASE + (flat_index + index) as u64;
                        let is_poster = !matches!(kind, HomeRowKind::Continue);
                        if is_poster && kind != HomeRowKind::Collection {
                            width
                                + (height * metrics.screen_poster_expand_aspect - width).max(0.0)
                                    * expand_ease(node_id)
                        } else {
                            width
                        }
                    },
                );
                let slot_height = if matches!(kind, HomeRowKind::Continue) {
                    card_height
                } else {
                    body_height
                };
                let slot_specs: Vec<crate::kit::Slot> = cards
                    .iter()
                    .enumerate()
                    .map(|(index, card)| {
                        let (_, item_height) = home_card_dimensions(metrics, card, kind);
                        let lead = home_card_lead(title, kind, index, item_height);
                        crate::kit::Slot::sized(lead + card_rects[index].width(), slot_height)
                    })
                    .collect();
                let slots = crate::kit::solve(
                    crate::kit::Flow::Row,
                    metrics.horizontal_spacing,
                    ui.max_rect().min,
                    ui.max_rect().size(),
                    &slot_specs,
                );
                {
                    for (index, card) in cards.iter().enumerate() {
                        let is_poster = !matches!(kind, HomeRowKind::Continue);
                        let (_, item_height) = home_card_dimensions(metrics, card, kind);
                        let node_id = NODE_CARD_BASE + flat_index as u64;
                        let ease = if is_poster && kind != HomeRowKind::Collection {
                            expand_ease(node_id)
                        } else {
                            0.0
                        };
                        let slot_rect = slots.get(index).copied().unwrap_or(Rect::NOTHING);
                        let widget_id = Id::new(("fluxa-home-card-slot", row_index, index));
                        let rect = card_rects[index];
                        let artwork_size = if viewport.is_tv() && kind != HomeRowKind::Collection {
                            rect.size() * metrics.focused_scale
                        } else {
                            rect.size()
                        };
                        let rect = if viewport.is_tv()
                            && focused == Some(node_id)
                            && kind != HomeRowKind::Collection
                        {
                            rect.scale_from_center(metrics.focused_scale)
                        } else {
                            rect
                        };
                        layout.focusable.push((node_id, rect));
                        let card_visible = rect.intersects(screen) && rect.intersects(row_clip);
                        if card_visible
                            && home.expanded_posters
                            && is_poster
                            && kind != HomeRowKind::Collection
                            && hover_pos.is_some_and(|pos| rect.intersect(row_clip).contains(pos))
                        {
                            next_hover_node = Some(node_id);
                        }
                        if card_visible
                            && kind != HomeRowKind::Collection
                            && let Some(pos) = pointer
                        {
                            if rect.intersect(row_clip).contains(pos) {
                                direct_hover = Some((node_id, rect, card));
                            } else if preview_state.is_some_and(|(id, since, area)| {
                                id == node_id
                                    && now - since >= home.hover_preview_delay
                                    && area.contains(pos)
                            }) {
                                popup_hover = Some((node_id, rect, card));
                            }
                        }
                        let response = card_visible
                            .then(|| ui.interact(slot_rect, widget_id, crate::gfx::Sense::click()));
                        if response.as_ref().is_some_and(|response| response.clicked()) {
                            *activated = Some(node_id);
                        }
                        if card_visible
                            && home.expanded_posters
                            && is_poster
                            && kind != HomeRowKind::Collection
                        {
                            let near = focus_node.is_some_and(|focus| node_id.abs_diff(focus) <= 4);
                            let priority = if near {
                                ArtworkPriority::Visible
                            } else {
                                ArtworkPriority::Prefetch
                            };
                            components::prefetch_expanded(
                                context,
                                card,
                                item_height,
                                priority,
                                assets,
                            );
                        }
                        if card_visible {
                            let rank = poster_overlay::rank_for(title, kind, index);
                            if let Some(rank) = rank {
                                poster_overlay::paint_beside(
                                    ui.painter(),
                                    rect,
                                    rank,
                                    metrics.accent,
                                    metrics.background,
                                );
                            }
                            if is_poster && ease > 0.0 {
                                if expanded_node == Some(node_id)
                                    && let Some(id) = card.id.clone()
                                {
                                    expanded_card = Some(ExpandedPoster {
                                        node: node_id,
                                        id,
                                        raw: card.raw.cloned(),
                                        trailer: true,
                                    });
                                }
                                let trailer = home
                                    .poster_trailer
                                    .as_ref()
                                    .filter(|trailer| {
                                        card.id.as_deref() == Some(trailer.item_id.as_str())
                                    })
                                    .and_then(|trailer| trailer.texture);
                                expanded_panels.push((node_id, rect, card, ease));
                                components::expanded_poster_card(
                                    ui.painter(),
                                    rect,
                                    card,
                                    ease,
                                    trailer,
                                    metrics,
                                    assets,
                                );
                            } else if is_poster {
                                let raised = home.poster_lift
                                    && (focused == Some(node_id)
                                        || response
                                            .as_ref()
                                            .is_some_and(|response| response.hovered()));
                                let lift = crate::motion::toggle(
                                    context,
                                    Id::new(("fluxa-poster-lift", node_id)),
                                    raised,
                                    crate::motion::fast(),
                                ) * metrics.card_lift;
                                components::poster_card(
                                    ui.painter(),
                                    rect.translate(Vec2::new(0.0, -lift)),
                                    card,
                                    metrics,
                                    assets,
                                    card.motion_enabled
                                        && card.motion_url.is_some()
                                        && ((home.gif_autoplay_enabled
                                            && !viewport.is_tv()
                                            && mostly_visible(rect, screen.intersect(row_clip)))
                                            || focused == Some(node_id)
                                            || response
                                                .as_ref()
                                                .is_some_and(|response| response.hovered())),
                                );
                            } else {
                                components::continue_card(
                                    ui.painter(),
                                    rect,
                                    artwork_size,
                                    card,
                                    metrics,
                                    assets,
                                );
                            }
                            if let Some(rank) = rank {
                                poster_overlay::paint_overlay(
                                    ui.painter(),
                                    rect,
                                    rank,
                                    metrics.accent,
                                    metrics.background,
                                );
                            }
                        }
                        flat_index += 1;
                    }
                }
            }
        }
    }
    crate::kit::lay(
        ui,
        vec![crate::kit::item(Vec2::new(0.0, tail), |_, _| {}).shrink(0.0)],
    );
    context.data_mut(|data| match next_hover_node {
        Some(node) => {
            data.insert_temp(expand_hover_id, node);
        }
        None => data.remove::<u64>(expand_hover_id),
    });
    for (node, rect, card, ease) in expanded_panels {
        let synopsis =
            first_value_string(&card.raw, &["description", "overview"]).unwrap_or_default();
        let meta = hero_meta_line(&card.raw, &home.language);
        components::expanded_info_panel(
            context, node, rect, screen, &meta, &synopsis, ease, metrics,
        );
    }
    context.data_mut(|data| match expanded_card {
        Some(card) => {
            data.insert_temp(Id::new("fluxa-expanded-poster"), card);
        }
        None => data.remove::<ExpandedPoster>(Id::new("fluxa-expanded-poster")),
    });
    let hovered = popup_hover.or(direct_hover);
    let resume = hovered
        .and_then(|(_, _, card)| card.id.as_deref())
        .and_then(|id| home.resume_for(id));
    draw_poster_hover_preview(
        context,
        hovered,
        PosterHover {
            screen,
            language: &home.language,
            delay: home.hover_preview_delay,
            resume,
            trailer: home.poster_trailer.as_ref(),
            metrics,
        },
        assets,
        activated,
    );
}

pub(crate) fn over_top_bar(context: &crate::gfx::Context, pos: Pos2) -> bool {
    context
        .layer_id_at(pos)
        .is_some_and(|layer| layer.id == Id::new("fluxa-shared-top-bar"))
}

pub(crate) struct PosterHover<'a> {
    pub screen: Rect,
    pub language: &'a str,
    pub delay: f64,
    pub resume: Option<&'a HomeCard>,
    pub trailer: Option<&'a HeroTrailer>,
    pub metrics: UiMetrics,
}

pub(crate) fn draw_poster_hover_preview(
    context: &crate::gfx::Context,
    hovered: Option<(u64, Rect, &HomeCard)>,
    hover: PosterHover,
    assets: &mut impl HomeAssets,
    activated: &mut Option<u64>,
) {
    let PosterHover {
        screen,
        language,
        delay,
        resume,
        trailer,
        metrics,
    } = hover;
    let preview_id = Id::new("fluxa-hover-preview-state");
    let preview_state = context.data(|data| data.get_temp::<(u64, f64, Rect)>(preview_id));
    let now = context.input(|input| input.time());
    let previewed_id = Id::new("fluxa-previewed-poster");
    let Some((node_id, anchor, card)) = hovered else {
        components::hover_preview_reset(context, crate::gfx::Order::Tooltip);
        context.data_mut(|data| data.remove::<ExpandedPoster>(previewed_id));
        context.data_mut(|data| data.remove::<(u64, f64, Rect)>(preview_id));
        return;
    };
    let enriched = with_episode_details(context, card);
    let card = enriched.as_ref().unwrap_or(card);
    let since = preview_state
        .filter(|(id, _, _)| *id == node_id)
        .map_or(now, |(_, since, _)| since);
    let (meta, synopsis) = card_summary(card, language);
    let (area, synopsis) = components::hover_preview_rect(
        context,
        anchor,
        Rect::from_min_max(Pos2::ZERO, screen.max),
        &synopsis,
        components::preview_is_compact(card),
        metrics,
    );
    context.data_mut(|data| data.insert_temp(preview_id, (node_id, since, area)));
    let remaining = delay - (now - since);
    if remaining > 0.0 {
        components::hover_preview_reset(context, crate::gfx::Order::Tooltip);
        context.data_mut(|data| data.remove::<ExpandedPoster>(previewed_id));
        context.request_repaint_after(std::time::Duration::from_secs_f64(remaining));
        return;
    }
    let series = matches!(card.item_type.as_deref(), Some("series" | "tv" | "show"));
    let first = series
        .then(|| first_episode(&card.raw))
        .flatten()
        .unwrap_or((1, 1, String::new()));
    let play_label = play_label(
        language,
        resume,
        series.then_some((first.0, first.1, Some(first.2.as_str()))),
    );
    let progress = resume
        .map(|resume| resume.progress)
        .filter(|progress| *progress > 0.0);
    if let Some(id) = card.id.clone() {
        context.data_mut(|data| {
            data.insert_temp(
                previewed_id,
                ExpandedPoster {
                    node: node_id,
                    id,
                    raw: card.raw.cloned(),
                    trailer: !components::preview_is_compact(card),
                },
            )
        });
    }
    let trailer = trailer
        .filter(|trailer| card.id.as_deref() == Some(trailer.item_id.as_str()))
        .and_then(|trailer| trailer.texture);
    let (response, action) = components::hover_preview(
        context,
        area,
        card,
        trailer,
        &meta,
        synopsis,
        &play_label,
        progress,
        Some(language),
        crate::gfx::Order::Tooltip,
        metrics,
        assets,
    );
    let slot = (node_id - NODE_CARD_BASE) * PREVIEW_ACTIONS;
    let offset = match action {
        Some(components::PreviewAction::Play) => Some(0),
        Some(components::PreviewAction::Watchlist) => Some(1),
        Some(components::PreviewAction::Details) => Some(2),
        Some(components::PreviewAction::Watched) => Some(3),
        Some(components::PreviewAction::ClearProgress) => Some(4),
        None => None,
    };
    match offset {
        Some(offset) => *activated = Some(NODE_PREVIEW_BASE + slot + offset),
        None if response.clicked() => *activated = Some(node_id),
        None => {}
    }
}
