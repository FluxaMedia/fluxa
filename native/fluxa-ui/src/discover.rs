use super::*;

fn all_extra_label(name: &str, language: &str) -> String {
    if name.eq_ignore_ascii_case("genre") {
        return localized("search.all_genres", language);
    }
    let readable_name = name
        .split(['_', '-', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ");
    if language.starts_with("tr") {
        format!("Tüm {readable_name}")
    } else {
        format!("All {readable_name}")
    }
}

pub fn draw_discover(
    context: &egui::Context,
    viewport: Viewport,
    discover: &DiscoverModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let scroll_max = discover_scroll_max(viewport, discover);
    let scroll_y = resolve_screen_scroll(
        context,
        viewport,
        Id::new("fluxa-screen-scroll-discover").with((
            &discover.content_type,
            &discover.selected_catalog_key,
            &discover.selected_extra_value,
            &discover.query,
        )),
        scroll_max,
    );
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    painter.rect_filled(screen, 0.0, metrics.background);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 2, assets);
    let compact = viewport.is_compact();
    let page = PageLayout::new(viewport, metrics, false);
    let margin = page.margin;
    let filter_row_width = page.width;
    let has_catalogs_for_type = discover
        .catalogs
        .iter()
        .any(|catalog| catalog.content_type == discover.content_type);
    egui::Area::new(Id::new("fluxa-shared-discover-title"))
        .fixed_pos(Pos2::new(margin, page.top))
        .show(context, |ui| {
            ui.set_min_height(page.title_height);
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new(localized("nav.discover", &discover.language))
                        .size(if compact {
                            metrics.screen_title_size_mobile
                        } else if viewport.is_tv() {
                            metrics.screen_title_size_tv
                        } else {
                            metrics.screen_title_size
                        })
                        .strong()
                        .color(Color32::WHITE),
                );
            });
        });
    egui::Area::new(Id::new("fluxa-shared-discover-search"))
        .order(egui::Order::Foreground)
        .fixed_pos(page.search.min)
        .show(context, |ui| {
            let mut query = discover.query.clone();
            let search = components::search_field(
                ui,
                &mut query,
                &localized("search.placeholder_expanded", &discover.language),
                page.search.width(),
                page.search.height(),
                metrics,
            );
            layout.focusable.push((NODE_DISCOVER_SEARCH, search.rect));
            if search.changed() {
                layout.text_input = Some(query);
                layout.text_input_node = Some(NODE_DISCOVER_SEARCH);
            }
        });
    egui::Area::new(Id::new("fluxa-shared-discover-header"))
        .order(egui::Order::Foreground)
        .fixed_pos(Pos2::new(margin, page.filters_top))
        .show(context, |ui| {
            ui.set_min_width(filter_row_width);
            ui.set_max_width(filter_row_width);
            let catalogs = discover
                .catalogs
                .iter()
                .filter(|catalog| catalog.content_type == discover.content_type)
                .collect::<Vec<_>>();
            let selected_catalog = catalogs
                .iter()
                .find(|catalog| catalog.key == discover.selected_catalog_key)
                .copied()
                .or_else(|| catalogs.first().copied());
            let catalog_options = catalogs
                .iter()
                .map(|catalog| (catalog.key.clone(), catalog.label.clone()))
                .collect::<Vec<_>>();
            let mut content_types = if discover.content_types.is_empty() {
                discover
                    .catalogs
                    .iter()
                    .map(|catalog| catalog.content_type.as_str())
                    .collect::<Vec<_>>()
            } else {
                discover
                    .content_types
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            };
            content_types.sort_unstable();
            content_types.dedup();
            if content_types.is_empty() {
                content_types.extend(["movie", "series"]);
            }
            if !content_types.contains(&discover.content_type.as_str()) {
                content_types.push(discover.content_type.as_str());
            }
            let type_options = content_types
                .iter()
                .map(|content_type| {
                    let label = match *content_type {
                        "movie" => localized("auto.movies", &discover.language),
                        "series" => localized("auto.series", &discover.language),
                        other => {
                            let mut label = other.to_owned();
                            if let Some(first) = label.get_mut(0..1) {
                                first.make_ascii_uppercase();
                            }
                            label
                        }
                    };
                    ((*content_type).to_owned(), label)
                })
                .collect::<Vec<_>>();
            let available = filter_row_width;
            let selected_extra = selected_catalog.and_then(|catalog| {
                catalog
                    .extras
                    .iter()
                    .find(|(name, _)| name == &discover.selected_extra_name)
                    .or_else(|| catalog.extras.first())
            });
            let selected_extra_required = selected_extra.is_some_and(|(name, _)| {
                selected_catalog.is_some_and(|catalog| catalog.required_extras.contains(name))
            });
            let extra_options = selected_extra
                .map(|(name, values)| {
                    let all_label = all_extra_label(name, &discover.language);
                    let mut options =
                        Vec::with_capacity(values.len() + usize::from(!selected_extra_required));
                    if !selected_extra_required {
                        options.push((String::new(), all_label));
                    }
                    options.extend(values.iter().cloned().map(|value| (value.clone(), value)));
                    options
                })
                .unwrap_or_default();
            let catalog_placeholder = localized("discover.catalog", &discover.language);
            let genre_placeholder = if selected_extra.is_some() {
                localized("search.all_genres", &discover.language)
            } else {
                localized("discover.filters_unavailable", &discover.language)
            };
            let selected_type_label = type_options
                .iter()
                .find(|(value, _)| value == &discover.content_type)
                .map(|(_, label)| label.as_str())
                .unwrap_or("Movies");
            let selected_catalog_label = selected_catalog
                .map(|catalog| catalog.label.as_str())
                .unwrap_or(catalog_placeholder.as_str());
            let selected_extra_label = extra_options
                .iter()
                .find(|(value, _)| value == &discover.selected_extra_value)
                .map(|(_, label)| label.as_str())
                .unwrap_or_else(|| {
                    if selected_extra_required {
                        extra_options
                            .first()
                            .map(|(_, label)| label.as_str())
                            .unwrap_or(genre_placeholder.as_str())
                    } else {
                        genre_placeholder.as_str()
                    }
                });
            let (type_width, catalog_width, genre_width) = if compact {
                let width = ((available - metrics.control_gap * 2.0) / 3.0).max(72.0);
                (width, width, width)
            } else {
                let gap = metrics.control_gap * 2.0;
                let target = (available - gap).max(1.0);
                let preferred = [
                    components::dropdown_width_for_label(
                        ui,
                        selected_type_label,
                        metrics,
                        80.0,
                        220.0,
                    ),
                    components::dropdown_width_for_label(
                        ui,
                        selected_catalog_label,
                        metrics,
                        88.0,
                        (available * 0.52).min(460.0),
                    ),
                    components::dropdown_width_for_label(
                        ui,
                        selected_extra_label,
                        metrics,
                        80.0,
                        280.0,
                    ),
                ];
                // Text determines each control's natural width; only compress
                // when the complete filter row cannot fit the current window.
                let scale = (target / preferred.iter().sum::<f32>()).min(1.0);
                (
                    preferred[0] * scale,
                    preferred[1] * scale,
                    preferred[2] * scale,
                )
            };
            let mut draw_filter_controls = |ui: &mut egui::Ui| {
                let (type_response, type_change) = components::dropdown(
                    ui,
                    "discover-content-type",
                    discover.content_type.as_str(),
                    &type_options,
                    type_width,
                    metrics,
                );
                layout
                    .focusable
                    .push((NODE_DISCOVER_TYPE_BASE, type_response.rect));
                if let Some(value) = type_change {
                    layout.filter_change = Some(("discover:contentType".to_owned(), value));
                }

                if catalog_options.is_empty() {
                    let response = components::dropdown_placeholder(
                        ui,
                        "discover-catalog-empty",
                        &catalog_placeholder,
                        catalog_width,
                        metrics,
                    );
                    layout
                        .focusable
                        .push((NODE_DISCOVER_CATALOG_BASE, response.rect));
                } else {
                    let (response, changed) = components::dropdown(
                        ui,
                        "discover-catalog",
                        selected_catalog
                            .map(|catalog| catalog.key.as_str())
                            .unwrap_or(""),
                        &catalog_options,
                        catalog_width,
                        metrics,
                    );
                    layout
                        .focusable
                        .push((NODE_DISCOVER_CATALOG_BASE, response.rect));
                    if let Some(value) = changed {
                        layout.filter_change = Some(("discover:catalog".to_owned(), value));
                    }
                }

                if selected_extra.is_some() && !extra_options.is_empty() {
                    let (response, changed) = components::dropdown(
                        ui,
                        "discover-extra",
                        discover.selected_extra_value.as_str(),
                        &extra_options,
                        genre_width,
                        metrics,
                    );
                    layout.focusable.push((NODE_DISCOVER_EXTRA, response.rect));
                    if let Some(value) = changed {
                        layout.filter_change = Some(("discover:extra".to_owned(), value));
                    }
                } else {
                    let response = components::dropdown_placeholder(
                        ui,
                        "discover-extra-empty",
                        &genre_placeholder,
                        genre_width,
                        metrics,
                    );
                    layout.focusable.push((NODE_DISCOVER_EXTRA, response.rect));
                }
            };
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = metrics.control_gap;
                draw_filter_controls(ui);
            });
        });

    let results_width = page.width;
    let grid = PosterGrid::new(results_width, metrics);
    let card_width = grid.card_width;
    let poster_height = grid.poster_height;
    let card_height = grid.card_height;
    let results_view_top = page.content_top;
    painter.hline(
        margin..=margin + page.width,
        page.content_top - metrics.section_gap * 0.75,
        egui::Stroke::new(1.0, Color32::from_white_alpha(18)),
    );
    let results_clip = Rect::from_min_max(
        Pos2::new(margin, results_view_top),
        Pos2::new(
            (viewport.width - margin).max(margin),
            (viewport.height - mobile_scroll_reserve(viewport)).max(results_view_top),
        ),
    );
    let row_top = results_view_top - scroll_y;
    egui::Area::new(Id::new("fluxa-shared-discover-results"))
        .fixed_pos(Pos2::new(margin, row_top))
        .show(context, |ui| {
            // An Area otherwise sizes itself to its contents. Explicitly bind
            // its width to the viewport so horizontal_wrapped can choose the
            // correct number of columns for phones, tablets, and desktops.
            ui.set_min_width(results_width);
            ui.set_max_width(results_width);
            // Keep the filters visible while results scroll below them. Clip
            // both paint and hit targets so cards cannot bleed into controls.
            ui.set_clip_rect(ui.clip_rect().intersect(results_clip));
            let grid_top = row_top;
            let columns = grid.columns;
            let row_stride = card_height + grid.row_gap;
            let total_rows = discover.results.len().div_ceil(columns);
            let first_row = ((results_clip.top() - grid_top) / row_stride)
                .floor()
                .max(0.0) as usize;
            let end_row = ((results_clip.bottom() - grid_top) / row_stride)
                .ceil()
                .max(0.0) as usize;
            let visible_rows = first_row.min(total_rows)..end_row.min(total_rows);
            let grid_height = if total_rows == 0 {
                0.0
            } else {
                grid.height(discover.results.len())
            };

            // The document uses an explicit scroll offset, so ordinary wrapped
            // layout would still allocate and measure every result on each
            // frame. Place only rows that intersect the viewport; this keeps
            // dropdown interaction cheap even when a catalog has thousands of
            // entries.
            for row in visible_rows {
                for column in 0..columns {
                    let index = row * columns + column;
                    let Some(card) = discover.results.get(index) else {
                        break;
                    };
                    let rect = Rect::from_min_size(
                        Pos2::new(
                            margin + column as f32 * (card_width + grid.gap),
                            grid_top + row as f32 * row_stride,
                        ),
                        Vec2::new(card_width, card_height),
                    );
                    let node_id = NODE_CARD_BASE + index as u64;
                    let response =
                        ui.interact(rect, Id::new(("discover-card", index)), Sense::click());
                    let poster_rect =
                        Rect::from_min_size(rect.min, Vec2::new(card_width, poster_height));
                    let visible_rect = poster_rect.intersect(results_clip);
                    if visible_rect.is_positive() {
                        layout.focusable.push((node_id, visible_rect));
                        components::poster_card(
                            ui.painter(),
                            poster_rect,
                            card,
                            column,
                            viewport,
                            metrics,
                            assets,
                            false,
                        );
                    }
                    if response.clicked() {
                        layout.activated = Some(node_id);
                    }
                }
            }
            // Warm artwork for rows just below the viewport so fast desktop
            // scrolling doesn't make each newly revealed row wait for network
            // fetch + decode. Keep this incremental and low-priority; visible
            // artwork always wins in the shared fetcher.
            let prefetch_end_row = (end_row + 3).min(total_rows);
            let prefetch_cursor_id = Id::new("fluxa-discover-artwork-prefetch").with((
                discover.generation,
                &discover.content_type,
                &discover.selected_catalog_key,
                &discover.query,
            ));
            let prefetch_start_row = context.data_mut(|data| {
                let previous_end = data
                    .get_temp::<usize>(prefetch_cursor_id)
                    .unwrap_or(end_row.min(total_rows));
                data.insert_temp(prefetch_cursor_id, prefetch_end_row.max(previous_end));
                previous_end.max(end_row.min(total_rows))
            });
            for row in prefetch_start_row..prefetch_end_row {
                for column in 0..columns {
                    let index = row * columns + column;
                    let Some(card) = discover.results.get(index) else {
                        break;
                    };
                    assets.prefetch_for(
                        card.artwork_url.as_deref(),
                        artwork_target_size(
                            Vec2::new(card_width, poster_height),
                            context.pixels_per_point(),
                        ),
                        ArtworkPriority::Prefetch,
                    );
                }
            }
            // Keep the Area's full content extent even though only visible
            // rows were materialized above; scroll bounds and clipping must
            // still include every row in the catalog.
            ui.add_space(metrics.control_gap);
            ui.allocate_space(Vec2::new(results_width, grid_height));
        });
    if discover.results.is_empty() {
        let title = if discover.is_loading || discover.catalogs_loading {
            localized("common.loading", &discover.language)
        } else if discover.error.is_some() {
            localized("common.error", &discover.language)
        } else if !has_catalogs_for_type {
            localized("discover.no_content", &discover.language)
        } else {
            localized("auto.no_results_yet", &discover.language)
        };
        let description = discover
            .error
            .as_deref()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                if !has_catalogs_for_type {
                    localized("discover.install_addons_hint", &discover.language)
                } else {
                    localized(
                        "auto.try_different_filters_to_discover_something_",
                        &discover.language,
                    )
                }
            });
        let state_rect = results_clip;
        {
            components::empty_state(
                &painter,
                state_rect,
                if discover.catalogs.is_empty() {
                    assets.icon("Discover")
                } else {
                    assets.icon("Movie")
                },
                &title,
                &description,
                metrics,
            );
        }
    }
    // Keep at least one results viewport buffered ahead. This starts fetching
    // subsequent catalog pages before the user reaches the tail, while poster
    // artwork remains viewport/lazy loaded by the artwork scheduler.
    let prefetch_distance = (results_clip.height() * 2.5).max(card_height * 2.0);
    if let Some(request) = discover.next_page.as_ref()
        && scroll_y >= (scroll_max - prefetch_distance).max(0.0)
        && let Some(skip) = request.get("skip").and_then(serde_json::Value::as_i64)
    {
        let request_id = Id::new("fluxa-discover-load-more").with((
            discover.generation,
            &discover.content_type,
            &discover.selected_catalog_key,
        ));
        let already_requested = context.data_mut(|data| {
            if data
                .get_temp::<(i64, f32)>(request_id)
                .is_some_and(|(previous_skip, _)| previous_skip == skip)
            {
                true
            } else {
                // De-duplicate this page token only. Requiring additional
                // scroll after a response can deadlock pagination when the
                // content-height estimate changes with newly projected cards.
                data.insert_temp(request_id, (skip, scroll_y));
                false
            }
        });
        if !already_requested {
            layout.load_more.push(request.clone());
        }
    }
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    if let Some((_, rect)) = layout.focusable.iter().find(|(id, _)| Some(*id) == focused) {
        painter.rect_stroke(
            rect.expand(metrics.focus_ring_expand),
            metrics.focus_ring_radius,
            egui::Stroke::new(metrics.focus_ring_width, Color32::WHITE),
            egui::StrokeKind::Outside,
        );
    }
    layout
}
