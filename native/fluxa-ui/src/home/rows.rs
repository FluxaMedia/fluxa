use super::*;

const HOVER_PREVIEW_DELAY: f64 = 0.5;

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_home_rows(
    context: &egui::Context,
    viewport: Viewport,
    home: &HomeModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
    metrics: UiMetrics,
    scroll_offset: f32,
    row_start: f32,
    prefetch_home_artwork: bool,
    margin: f32,
    screen: Rect,
    layout: &mut HomeLayout,
    activated: &mut Option<u64>,
) {
    let mut flat_index = 0usize;
    let mut row_y = row_start;
    let preview_id = Id::new("fluxa-hover-preview-state");
    let preview_state = context.data(|data| data.get_temp::<(u64, f64, Rect)>(preview_id));
    let pointer = (viewport.form_factor == UiFormFactor::Desktop)
        .then(|| context.input(|input| input.pointer.hover_pos()))
        .flatten();
    let mut hovered: Option<(u64, Rect, &HomeCard)> = None;
    for (row_index, (title, cards, kind)) in home
        .content_rows_with_kind()
        .into_iter()
        .enumerate()
        .take(64)
    {
        let (card_width, card_height, _) = home_row_dimensions(metrics, kind);
        let body_height = home_row_body_height(metrics, cards, kind);
        let row_scroll_max = home_row_scroll_max_for_cards(viewport, metrics, title, cards, kind);
        let row_height = home_row_heading_height(metrics) + body_height;
        let visible_y = row_y - scroll_offset;
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
        if visible_y + row_height < 0.0 || visible_y > viewport.height + row_height {
            row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
            flat_index += cards.len();
            continue;
        }
        let heading_height = home_row_heading_height(metrics);
        let type_label = row_index
            .checked_sub(usize::from(!home.cards.is_empty()))
            .and_then(|index| home.rows.get(index))
            .and_then(|row| row.type_label.as_deref());
        egui::Area::new(Id::new(format!("fluxa-shared-row-heading-{row_index}")))
            .fixed_pos(Pos2::new(margin, visible_y))
            // Catalog shelves are part of a scrolling document, not floating
            // dialogs. egui's default constraint pulls an off-screen shelf up
            // to fit the viewport, which makes it paint over the prior shelf.
            .constrain(false)
            .show(context, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(screen));
                let title_size = metrics.row_heading_size;
                let title = match type_label {
                    Some(label) => format!("{title} - {label}"),
                    None => title.to_owned(),
                };
                ui.add(egui::Label::new(RichText::new(title).size(title_size).strong()).extend());
                ui.add_space(metrics.control_gap);
            });
        let row_clip = Rect::from_min_max(
            Pos2::new(0.0, visible_y + heading_height),
            Pos2::new(
                viewport.width,
                (visible_y + heading_height + body_height).min(viewport.height),
            ),
        );
        let row_scroll_offset = if viewport.form_factor == UiFormFactor::Desktop {
            resolve_desktop_horizontal_scroll(
                context,
                Id::new(("fluxa-home-row-scroll", row_index)),
                row_clip,
                home.row_scroll_offsets
                    .get(row_index)
                    .copied()
                    .unwrap_or(0.0),
                row_scroll_max,
            )
        } else {
            home.row_scroll_offsets
                .get(row_index)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, row_scroll_max)
        };
        if viewport.form_factor == UiFormFactor::Desktop
            && let Some(row) = home
                .rows
                .get(row_index.saturating_sub(usize::from(!home.cards.is_empty())))
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
        egui::Area::new(Id::new(format!("fluxa-shared-row-cards-{row_index}")))
            .fixed_pos(Pos2::new(
                margin - row_scroll_offset,
                visible_y + heading_height,
            ))
            .constrain(false)
            .show(context, |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(row_clip));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = metrics.horizontal_spacing;
                    for (index, card) in cards.iter().enumerate() {
                        let is_poster = !matches!(kind, HomeRowKind::Continue);
                        let (item_width, item_height) = home_card_dimensions(metrics, card, kind);
                        let slot_height = if is_poster { body_height } else { card_height };
                        let lead = home_card_lead(title, kind, index, item_height);
                        let (widget_id, slot_rect) =
                            ui.allocate_space(Vec2::new(lead + item_width, slot_height));
                        let rect = Rect::from_min_size(
                            slot_rect.min + Vec2::new(lead, 0.0),
                            Vec2::new(item_width, item_height),
                        );
                        let node_id = NODE_CARD_BASE + flat_index as u64;
                        let rect = if viewport.is_tv() && focused == Some(node_id) {
                            Rect::from_center_size(
                                rect.center(),
                                rect.size() * metrics.focused_scale,
                            )
                        } else {
                            rect
                        };
                        layout.focusable.push((node_id, rect));
                        let card_visible = rect.intersects(screen) && rect.intersects(row_clip);
                        if card_visible
                            && is_poster
                            && kind != HomeRowKind::Collection
                            && let Some(pos) = pointer
                            && (rect.intersect(row_clip).contains(pos)
                                || preview_state.is_some_and(|(id, _, area)| {
                                    id == node_id && area.contains(pos)
                                }))
                        {
                            hovered = Some((node_id, rect, card));
                        }
                        let response = card_visible
                            .then(|| ui.interact(slot_rect, widget_id, egui::Sense::click()));
                        if response.as_ref().is_some_and(|response| response.clicked()) {
                            *activated = Some(node_id);
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
                            if is_poster {
                                components::poster_card(
                                    ui.painter(),
                                    rect,
                                    card,
                                    metrics,
                                    assets,
                                    card.motion_enabled
                                        && card.motion_url.is_some()
                                        && ((home.gif_autoplay_enabled
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
                });
            });
        row_y += row_height + metrics.section_gap + metrics.vertical_spacing;
    }
    let Some((node_id, anchor, card)) = hovered else {
        components::hover_preview_reset(context);
        context.data_mut(|data| data.remove::<(u64, f64, Rect)>(preview_id));
        return;
    };
    let now = context.input(|input| input.time);
    let since = preview_state
        .filter(|(id, _, _)| *id == node_id)
        .map_or(now, |(_, since, _)| since);
    let synopsis = first_value_string(&card.raw, &["description", "overview"]).unwrap_or_default();
    let (area, synopsis) =
        components::hover_preview_rect(context, anchor, screen, &synopsis, metrics);
    context.data_mut(|data| data.insert_temp(preview_id, (node_id, since, area)));
    let remaining = HOVER_PREVIEW_DELAY - (now - since);
    if remaining > 0.0 {
        components::hover_preview_reset(context);
        context.request_repaint_after(std::time::Duration::from_secs_f64(remaining));
        return;
    }
    let meta = hero_meta_line(&card.raw, &home.language);
    let resume = card.id.as_deref().and_then(|id| home.resume_for(id));
    let series = matches!(card.item_type.as_deref(), Some("series" | "tv" | "show"));
    let play_label = play_label(&home.language, resume, series.then_some((1, 1, None)));
    let progress = resume
        .map(|resume| resume.progress)
        .filter(|progress| *progress > 0.0);
    let (response, action) = components::hover_preview(
        context,
        area,
        card,
        &meta,
        synopsis,
        &play_label,
        progress,
        metrics,
        assets,
    );
    let slot = (node_id - NODE_CARD_BASE) * 2;
    match action {
        Some(components::PreviewAction::Play) => *activated = Some(NODE_PREVIEW_BASE + slot),
        Some(components::PreviewAction::Watchlist) => {
            *activated = Some(NODE_PREVIEW_BASE + slot + 1)
        }
        None if response.clicked() => *activated = Some(node_id),
        None => {}
    }
}
