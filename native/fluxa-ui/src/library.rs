use super::*;

pub fn draw_library(
    context: &egui::Context,
    viewport: Viewport,
    library: &LibraryModel,
    tab: LibraryTab,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let scroll_y = resolve_screen_scroll(
        context,
        viewport,
        Id::new("fluxa-screen-scroll-library"),
        library_scroll_max(viewport, library, tab),
    );
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    painter.rect_filled(screen, 0.0, metrics.background);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 1, assets);

    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let margin = if compact {
        metrics.page_padding
    } else if tv {
        metrics.screen_padding.max(32.0)
    } else {
        metrics.screen_padding
    };
    let top = if compact {
        metrics.content_header_top_mobile
    } else {
        metrics.content_header_top
    };
    let mut header_bottom = top;
    egui::Area::new(Id::new("fluxa-shared-library-header"))
        .fixed_pos(Pos2::new(margin, top))
        .show(context, |ui| {
            ui.set_max_width((viewport.width - margin * 2.0).max(1.0));
            ui.label(
                RichText::new(localized("nav.library", &library.language))
                    .size(if compact {
                        metrics.screen_title_size_mobile
                    } else if tv {
                        metrics.screen_title_size_tv
                    } else {
                        metrics.screen_title_size
                    })
                    .strong()
                    .color(metrics.text_primary),
            );
            ui.add_space(metrics.control_gap);
            ui.label(
                RichText::new(localized("native.library.description", &library.language))
                    .size(if tv {
                        metrics.screen_body_size_tv
                    } else {
                        metrics.screen_body_size
                    })
                    .color(metrics.text_secondary),
            );
            ui.add_space(metrics.section_gap);
            if compact {
                let tabs_viewport = egui::ScrollArea::horizontal()
                    .id_salt("fluxa-library-tabs-compact")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = metrics.control_gap;
                            for candidate in LibraryTab::ALL {
                                let tab_index = LibraryTab::ALL
                                    .iter()
                                    .position(|value| *value == candidate)
                                    .unwrap_or_default();
                                let selected = candidate == tab;
                                let response = components::button_auto_width(
                                    ui,
                                    &localized(candidate.translation_key(), &library.language),
                                    if selected {
                                        components::ButtonKind::Accent
                                    } else {
                                        components::ButtonKind::Subtle
                                    },
                                    metrics.nav_label_size,
                                    metrics,
                                );
                                layout.focusable.push((
                                    NODE_LIBRARY_TAB_BASE + tab_index as u64,
                                    response.rect,
                                ));
                                if response.clicked() {
                                    layout.activated =
                                        Some(NODE_LIBRARY_TAB_BASE + tab_index as u64);
                                }
                            }
                        });
                    });
                for (id, rect) in &mut layout.focusable {
                    if (NODE_LIBRARY_TAB_BASE..NODE_LIBRARY_TAB_BASE + LibraryTab::ALL.len() as u64)
                        .contains(id)
                    {
                        *rect = rect.intersect(tabs_viewport.inner_rect);
                    }
                }
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = metrics.control_gap;
                    for candidate in LibraryTab::ALL {
                        let tab_index = LibraryTab::ALL
                            .iter()
                            .position(|value| *value == candidate)
                            .unwrap_or_default();
                        let selected = candidate == tab;
                        let response = components::button_auto_width(
                            ui,
                            &localized(candidate.translation_key(), &library.language),
                            if selected {
                                components::ButtonKind::Accent
                            } else {
                                components::ButtonKind::Subtle
                            },
                            metrics.nav_label_size,
                            metrics,
                        );
                        layout
                            .focusable
                            .push((NODE_LIBRARY_TAB_BASE + tab_index as u64, response.rect));
                        if response.clicked() {
                            layout.activated = Some(NODE_LIBRARY_TAB_BASE + tab_index as u64);
                        }
                    }
                });
            }
            ui.add_space(metrics.control_gap);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = metrics.control_gap;
                let mut query = library.query.clone();
                let available = (viewport.width - margin * 2.0 - 20.0).max(1.0);
                let search_width = available.min(metrics.library_search_max_width).max(1.0);
                let search = ui
                    .horizontal(|ui| {
                        ui.add_sized(
                            [search_width, metrics.screen_control_height],
                            egui::TextEdit::singleline(&mut query).hint_text(localized(
                                "library.filter_placeholder",
                                &library.language,
                            )),
                        )
                    })
                    .inner;
                layout.focusable.push((NODE_LIBRARY_SEARCH, search.rect));
                if search.changed() {
                    layout.text_input = Some(query);
                    layout.text_input_node = Some(NODE_LIBRARY_SEARCH);
                }
                let dropdown_space = (available - metrics.control_gap).max(1.0);
                let sort_options = [
                    (
                        "recent".to_owned(),
                        localized("library.sort_recent", &library.language),
                    ),
                    (
                        "title".to_owned(),
                        localized("library.sort_title", &library.language),
                    ),
                    (
                        "rating".to_owned(),
                        localized("library.sort_rating", &library.language),
                    ),
                ];
                let sort_width = if compact {
                    let label = sort_options
                        .iter()
                        .find(|(key, _)| key == &library.sort_by)
                        .map(|(_, label)| label.as_str())
                        .unwrap_or("Recent");
                    let natural = ui.fonts_mut(|fonts| {
                        fonts
                            .layout_no_wrap(
                                label.to_owned(),
                                egui::FontId::proportional(metrics.nav_label_size),
                                Color32::WHITE,
                            )
                            .size()
                            .x
                    }) + metrics.control_gap * 2.0
                        + 34.0;
                    natural.clamp(104.0, dropdown_space * 0.55)
                } else {
                    (dropdown_space * 0.45).min(230.0).max(1.0)
                };
                let source_width = if compact {
                    let label = library
                        .source_options
                        .iter()
                        .find(|(key, _)| key == &library.source)
                        .map(|(_, label)| label.as_str())
                        .unwrap_or("Library");
                    let natural = ui.fonts_mut(|fonts| {
                        fonts
                            .layout_no_wrap(
                                label.to_owned(),
                                egui::FontId::proportional(metrics.nav_label_size),
                                Color32::WHITE,
                            )
                            .size()
                            .x
                    }) + metrics.control_gap * 2.0
                        + 34.0;
                    natural.clamp(104.0, (dropdown_space - sort_width).max(1.0))
                } else {
                    (dropdown_space - sort_width).min(280.0).max(1.0)
                };
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = metrics.control_gap;
                    let (sort, sort_change) = components::dropdown(
                        ui,
                        "library-sort",
                        &library.sort_by,
                        &sort_options,
                        sort_width,
                        metrics,
                    );
                    layout.focusable.push((NODE_LIBRARY_SORT, sort.rect));
                    if let Some(value) = sort_change {
                        layout.filter_change = Some(("librarySort".to_owned(), value));
                    }
                    let (source, source_change) = components::dropdown(
                        ui,
                        "library-source",
                        &library.source,
                        &library.source_options,
                        source_width,
                        metrics,
                    );
                    layout.focusable.push((NODE_LIBRARY_SOURCE, source.rect));
                    if let Some(value) = source_change {
                        layout.setting_change = Some((
                            "integrationLibrarySource".to_owned(),
                            serde_json::Value::String(value),
                        ));
                    }
                });
            });
            ui.add_space(metrics.control_gap);
            ui.label(
                RichText::new(format!(
                    "{} {}",
                    library.cards(tab).len(),
                    localized("native.library.titles", &library.language)
                ))
                .size(metrics.screen_card_subtitle_size)
                .color(metrics.text_muted),
            );
            header_bottom = ui.min_rect().bottom();
        });

    let cards = library.cards(tab);
    let card_width = metrics
        .poster_card_width
        .min((viewport.width - margin * 2.0).max(metrics.screen_card_min_width));
    let card_height = metrics.poster_card_height;
    let row_top = header_bottom + metrics.vertical_spacing - scroll_y;
    let mut flat_index = 0usize;
    egui::Area::new(Id::new("fluxa-shared-library-grid"))
        .fixed_pos(Pos2::new(margin, row_top))
        .show(context, |ui| {
            ui.set_max_width((viewport.width - margin * 2.0).max(1.0));
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing =
                    Vec2::new(metrics.horizontal_spacing, metrics.vertical_spacing);
                for card in cards.iter().take(64) {
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(
                            card_width,
                            card_height
                                + metrics.screen_card_title_size
                                + metrics.screen_card_subtitle_size
                                + metrics.control_gap * 2.0,
                        ),
                        Sense::click(),
                    );
                    let node_id = NODE_CARD_BASE + flat_index as u64;
                    layout.focusable.push((node_id, rect));
                    if response.clicked() {
                        layout.activated = Some(node_id);
                    }
                    let poster = Rect::from_min_size(rect.min, Vec2::new(card_width, card_height));
                    components::poster_card(
                        ui.painter(),
                        poster,
                        card,
                        flat_index,
                        viewport,
                        metrics,
                        assets,
                        false,
                    );
                    if card.progress > 0.0 {
                        let bar =
                            poster.left_bottom() + Vec2::new(0.0, -metrics.card_progress_height);
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                bar,
                                Vec2::new(
                                    poster.width() * card.progress.clamp(0.0, 1.0),
                                    metrics.card_progress_height,
                                ),
                            ),
                            2.0,
                            metrics.accent,
                        );
                    }
                    flat_index += 1;
                }
            });
        });
    if cards.is_empty() {
        let state_width = (viewport.width - margin * 2.0).min(680.0).max(1.0);
        let state_height = 72.0;
        let content_top = header_bottom + metrics.vertical_spacing;
        let state_top = content_top - scroll_y;
        let state_rect = Rect::from_min_size(
            Pos2::new(margin, state_top),
            Vec2::new(state_width, state_height),
        );
        let title = if library.is_loading {
            localized("native.library.loading", &library.language)
        } else if library.error.is_some() {
            localized("native.library.error", &library.language)
        } else {
            localized("library.empty", &library.language)
        };
        let description = if library.is_loading || library.error.is_some() {
            library.error.clone().unwrap_or_default()
        } else {
            localized("library.add_titles_hint", &library.language)
        };
        components::empty_state(
            &painter,
            state_rect,
            assets.icon("Library"),
            &title,
            &description,
            metrics,
        );
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
