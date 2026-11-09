use super::*;
use crate::dp::Dp;
use egui::Ui;

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
    localized("discover.all_filter_values", language).replacen("%s", &readable_name, 1)
}

pub fn draw_discover(
    context: &egui::Context,
    viewport: Viewport,
    discover: &DiscoverModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 1, assets);
    let preview_id = Id::new("fluxa-hover-preview-state");
    let preview_state = context.data(|data| data.get_temp::<(u64, f64, Rect)>(preview_id));
    let pointer = (viewport.form_factor == UiFormFactor::Desktop
        && discover.poster_hover_preview
        && !components::screen_covered(context))
    .then(|| context.input(|input| input.pointer.hover_pos()))
    .flatten()
    .filter(|pos| !components::over_mini_player(context, *pos) && !over_top_bar(context, *pos));
    let mut direct_hover: Option<(u64, Rect, &HomeCard)> = None;
    let mut popup_hover: Option<(u64, Rect, &HomeCard)> = None;
    let now = context.input(|input| input.time);
    let compact = viewport.is_compact();
    let sizes = PageLayout::new(viewport, metrics, false);
    let has_catalogs_for_type = discover
        .catalogs
        .iter()
        .any(|catalog| catalog.content_type == discover.content_type);
    let mut search_rect = sizes.search;
    let scroll_key = (
        &discover.content_type,
        &discover.selected_catalog_key,
        &discover.selected_extra_value,
        &discover.query,
    );

    kit::page(context, viewport, "discover", |ui| {
        let width = ui.available_width();
        kit::view()
            .class("screen-header")
            .row()
            .height(sizes.title_height)
            .show(ui, |ui| {
                let title = localized("nav.discover", &discover.language);
                let mut search = None;
                kit::flex("discover-header")
                    .justify(kit::Justify::Between)
                    .center()
                    .child(|ui| {
                        ui.label(
                            RichText::new(title)
                                .size(metrics.screen_title_size)
                                .strong()
                                .color(text_primary()),
                        );
                    })
                    .child(|ui| {
                        if !compact {
                            search =
                                Some(discover_search(ui, &mut layout, discover, &sizes, metrics));
                        }
                    })
                    .show(ui);
                if let Some(rect) = search {
                    search_rect = rect;
                }
            });
        ui.add_space(metrics.screen_stack_gap);
        if compact {
            search_rect = discover_search(ui, &mut layout, discover, &sizes, metrics);
            kit::space(ui, Space::S12);
        }
        discover_filters(ui, &mut layout, discover, metrics, compact);
        kit::space(ui, Space::S16);
        let line = ui.cursor().min.y;
        painter.hline(
            ui.min_rect().left()..=ui.min_rect().left() + width,
            line,
            egui::Stroke::new(stroke::hairline(), metrics.border),
        );
        kit::space(ui, Space::S16);

        kit::scroll_body_keyed(ui, viewport, "discover", scroll_key, |ui| {
            let clip = ui.clip_rect();
            let origin = ui.cursor().min;
            let grid = PosterGrid::new(width, metrics);
            let card_width = grid.card_width;
            let poster_height = grid.poster_height;
            let card_height = grid.card_height;
            let columns = grid.columns;
            let blocks = discover_blocks(&grid, metrics, discover);
            let grid_height = blocks.last().map_or(0.0, |block| block.top + block.height);
            let mut end_index = 0;
            for block in &blocks {
                let block_top = origin.y + block.top;
                if block_top > clip.bottom() {
                    break;
                }
                if block_top + block.height < clip.top() {
                    continue;
                }
                if block.pending {
                    skeleton_section(
                        ui.painter(),
                        Pos2::new(origin.x, block_top),
                        &grid,
                        clip.right(),
                        block.header,
                        metrics,
                    );
                    continue;
                }
                if let Some(title) = block.title {
                    draw_text(
                        ui.painter(),
                        Rect::from_min_size(
                            Pos2::new(origin.x, block_top),
                            Vec2::new(width, block.header),
                        )
                        .left_center(),
                        egui::Align2::LEFT_CENTER,
                        title,
                        crate::fonts::bold(metrics.screen_section_title_size),
                        text_primary(),
                        None,
                    );
                }
                let grid_top = block_top + block.header;
                let (columns, x_offset) = if block.title.is_some() {
                    let row_rect = Rect::from_min_max(
                        Pos2::new(origin.x, grid_top),
                        Pos2::new(clip.right(), grid_top + card_height),
                    );
                    let scroll_id = Id::new(("discover-section-scroll", block.start, block.title));
                    let max_offset = (kit::Lattice {
                        origin: Pos2::ZERO,
                        cell: Vec2::new(card_width, card_height),
                        gap: grid.gap,
                        row_gap: 0.0,
                        columns: block.len.max(1),
                    }
                    .width(block.len)
                        - width)
                        .max(0.0);
                    let mut offset = context
                        .data(|data| data.get_temp::<f32>(scroll_id))
                        .unwrap_or(0.0);
                    if !components::screen_covered(context) && ui.rect_contains_pointer(row_rect) {
                        let delta = ui.input(|input| input.smooth_scroll_delta.x);
                        offset = (offset - delta).clamp(0.0, max_offset);
                    }
                    context.data_mut(|data| data.insert_temp(scroll_id, offset));
                    (block.len.max(1), offset)
                } else {
                    (columns, 0.0)
                };
                let lattice = kit::Lattice {
                    origin: Pos2::new(origin.x - x_offset, grid_top),
                    cell: Vec2::new(card_width, card_height),
                    gap: grid.gap,
                    row_gap: grid.row_gap,
                    columns,
                };
                for row in lattice.visible_rows(block.len, clip.top(), clip.bottom()) {
                    for column in 0..columns {
                        let local = row * columns + column;
                        if local >= block.len {
                            break;
                        }
                        let index = block.start + local;
                        let Some(card) = discover.results.get(index) else {
                            break;
                        };
                        end_index = end_index.max(index + 1);
                        let rect = lattice.rect(local);
                        if rect.right() < clip.left() {
                            continue;
                        }
                        if rect.left() > clip.right() {
                            break;
                        }
                        let node_id = NODE_CARD_BASE + index as u64;
                        let response =
                            ui.interact(rect, Id::new(("discover-card", index)), Sense::click());
                        let poster_rect =
                            Rect::from_min_size(rect.min, Vec2::new(card_width, poster_height));
                        let visible_rect = poster_rect.intersect(clip);
                        if visible_rect.is_positive() {
                            layout.focusable.push((node_id, visible_rect));
                            if let Some(pos) = pointer {
                                if visible_rect.contains(pos) {
                                    direct_hover = Some((node_id, poster_rect, card));
                                } else if preview_state.is_some_and(|(id, since, area)| {
                                    id == node_id
                                        && now - since >= discover.hover_preview_delay
                                        && area.contains(pos)
                                }) {
                                    popup_hover = Some((node_id, poster_rect, card));
                                }
                            }
                            components::poster_card(
                                ui.painter(),
                                poster_rect,
                                card,
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
            }
            let prefetch_end = (end_index + columns * 3).min(discover.results.len());
            let prefetch_cursor_id = Id::new("fluxa-discover-artwork-prefetch").with((
                discover.generation,
                &discover.content_type,
                &discover.selected_catalog_key,
                &discover.query,
            ));
            let prefetch_start = context.data_mut(|data| {
                let previous_end = data
                    .get_temp::<usize>(prefetch_cursor_id)
                    .unwrap_or(end_index);
                data.insert_temp(prefetch_cursor_id, prefetch_end.max(previous_end));
                previous_end.max(end_index)
            });
            for card in discover
                .results
                .get(prefetch_start..prefetch_end)
                .into_iter()
                .flatten()
            {
                assets.prefetch_for(
                    card.poster_art(),
                    artwork_target_size(
                        Vec2::new(card_width, poster_height),
                        context.pixels_per_point(),
                    ),
                    ArtworkPriority::Prefetch,
                );
            }
            let loading = discover.is_loading || discover.catalogs_loading;
            if discover.results.is_empty() && loading && discover.error.is_none() {
                let painter = ui.painter().clone();
                let mut top = origin.y;
                while top < clip.bottom() {
                    skeleton_section(
                        &painter,
                        Pos2::new(origin.x, top),
                        &grid,
                        clip.right(),
                        metrics.screen_section_title_size + metrics.control_gap * 2.0,
                        metrics,
                    );
                    top += metrics.screen_section_title_size
                        + metrics.control_gap * 2.0
                        + card_height
                        + metrics.section_gap;
                }
            } else if discover.results.is_empty() {
                let title = if discover.error.is_some() {
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
                let state_rect = Rect::from_min_max(
                    clip.min,
                    Pos2::new(
                        clip.right(),
                        (clip.bottom() - mobile_scroll_reserve(viewport)).max(clip.top()),
                    ),
                );
                components::empty_state(
                    ui.painter(),
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
            kit::space(ui, Space::S12);
            ui.allocate_space(Vec2::new(width, grid_height));
        });
    });

    if components::recent_searches_open(focused, &discover.recent_searches) {
        components::recent_searches(
            context,
            search_rect,
            &localized("search.recent_searches", &discover.language),
            &discover.recent_searches,
            &mut layout,
            metrics,
        );
    }
    draw_poster_hover_preview(
        context,
        popup_hover.or(direct_hover),
        PosterHover {
            screen,
            language: &discover.language,
            delay: discover.hover_preview_delay,
            resume: None,
            trailer: None,
            metrics,
        },
        assets,
        &mut layout.activated,
    );
    let measured = kit::scroll_state(context, "discover");
    let grid = PosterGrid::new(sizes.width, metrics);
    let prefetch_distance = (viewport.height * 6.0).max(grid.card_height * 12.0);
    if let Some(request) = discover.next_page.as_ref()
        && let Some((scroll_y, scroll_max)) = measured
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
    layout
}

fn discover_search(
    ui: &mut Ui,
    layout: &mut HomeLayout,
    discover: &DiscoverModel,
    sizes: &PageLayout,
    metrics: UiMetrics,
) -> Rect {
    let mut query = discover.query.clone();
    let search = components::search_field(
        ui,
        &mut query,
        &localized("search.placeholder_expanded", &discover.language),
        sizes.search.width().min(ui.available_width()),
        sizes.search.height(),
        metrics,
    );
    layout.focusable.push((NODE_DISCOVER_SEARCH, search.rect));
    if search.changed() {
        layout.text_input = Some(query);
        layout.text_input_node = Some(NODE_DISCOVER_SEARCH);
    }
    search.rect
}

fn discover_filters(
    ui: &mut Ui,
    layout: &mut HomeLayout,
    discover: &DiscoverModel,
    metrics: UiMetrics,
    compact: bool,
) {
    let filter_row_width = ui.available_width();
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
    let (type_width, catalog_width, genre_width) = {
        let gap = metrics.control_gap * 2.0;
        let target = (available - gap).max(1.0);
        let preferred = [
            components::dropdown_width_for_label(
                ui,
                selected_type_label,
                metrics,
                Dp::D80.px(),
                metrics.screen_filter_type_max,
            ),
            components::dropdown_width_for_label(
                ui,
                selected_catalog_label,
                metrics,
                Dp::D80.px(),
                if compact {
                    available
                } else {
                    available * metrics.screen_filter_catalog_ratio
                }
                .min(metrics.screen_filter_catalog_max),
            ),
            components::dropdown_width_for_label(
                ui,
                selected_extra_label,
                metrics,
                Dp::D80.px(),
                metrics.screen_filter_genre_max,
            ),
        ];
        let scale = if compact {
            1.0
        } else {
            (target / preferred.iter().sum::<f32>()).min(1.0)
        };
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
            compact,
            metrics,
        );
        if compact
            && let Some(request) = components::sheet_choice(
                NODE_DISCOVER_TYPE_BASE,
                "discover:contentType",
                localized("auto.type", &discover.language),
                &type_options,
                discover.content_type.as_str(),
            )
        {
            layout.choices.push(request);
        }
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
                compact,
                metrics,
            );
            if compact
                && let Some(request) = components::sheet_choice(
                    NODE_DISCOVER_CATALOG_BASE,
                    "discover:catalog",
                    catalog_placeholder.clone(),
                    &catalog_options,
                    selected_catalog
                        .map(|catalog| catalog.key.as_str())
                        .unwrap_or(""),
                )
            {
                layout.choices.push(request);
            }
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
                compact,
                metrics,
            );
            if compact
                && let Some(request) = components::sheet_choice(
                    NODE_DISCOVER_EXTRA,
                    "discover:extra",
                    genre_placeholder.clone(),
                    &extra_options,
                    discover.selected_extra_value.as_str(),
                )
            {
                layout.choices.push(request);
            }
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
    crate::components::pan_strip(ui, "discover-filters", |ui| {
        kit::view()
            .class("discover-filters")
            .row()
            .hug()
            .show(ui, |ui| {
                draw_filter_controls(ui);
            });
    });
}

fn skeleton_section(
    painter: &egui::Painter,
    origin: Pos2,
    grid: &PosterGrid,
    right: f32,
    header: f32,
    metrics: UiMetrics,
) {
    let band = Rect::from_min_size(origin, Vec2::new(right - origin.x, header));
    let title = crate::flex::column([
        crate::flex::fill(),
        crate::flex::leaf(Vec2::new(
            grid.card_width * metrics.screen_skeleton_title_cards,
            0.0,
        ))
        .grow(2.0)
        .id(0),
        crate::flex::fill(),
    ])
    .solve(band)
    .find(0);
    if let Some(title) = title {
        components::skeleton_card(painter, title, metrics);
    }
    let pitch = grid.card_width + grid.gap;
    let count = ((right - origin.x) / pitch).ceil().max(0.0) as usize;
    let strip = crate::flex::row((0..count).map(|index| {
        crate::flex::leaf(Vec2::new(grid.card_width, grid.poster_height))
            .width(grid.card_width)
            .height(grid.poster_height)
            .id(index)
    }))
    .gap(grid.gap)
    .solve(Rect::from_min_size(
        Pos2::new(origin.x, origin.y + header),
        Vec2::new(pitch * count as f32, grid.poster_height),
    ));
    for (_, rect) in strip.tagged() {
        components::skeleton_card(painter, rect, metrics);
    }
}
