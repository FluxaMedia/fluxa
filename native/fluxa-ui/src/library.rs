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
    let split = library_needs_second_row(viewport, metrics);
    let page = PageLayout::new(viewport, metrics, split);
    let cards = library.cards(tab);

    egui::Area::new(Id::new("fluxa-shared-library-title"))
        .fixed_pos(Pos2::new(page.margin, page.top))
        .show(context, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(page.title_height);
                ui.spacing_mut().item_spacing.x = metrics.control_gap * 1.5;
                ui.label(
                    RichText::new(localized("nav.library", &library.language))
                        .size(if compact {
                            metrics.screen_title_size_mobile
                        } else if viewport.is_tv() {
                            metrics.screen_title_size_tv
                        } else {
                            metrics.screen_title_size
                        })
                        .strong()
                        .color(metrics.text_primary),
                );
                if !cards.is_empty() {
                    ui.label(
                        RichText::new(cards.len().to_string())
                            .size(metrics.screen_body_size + 2.0)
                            .color(metrics.text_muted),
                    );
                }
            });
        });

    egui::Area::new(Id::new("fluxa-shared-library-search"))
        .fixed_pos(page.search.min)
        .show(context, |ui| {
            let mut query = library.query.clone();
            let search = components::search_field(
                ui,
                &mut query,
                &localized("library.filter_placeholder", &library.language),
                page.search.width(),
                page.search.height(),
                metrics,
            );
            layout.focusable.push((NODE_LIBRARY_SEARCH, search.rect));
            if search.changed() {
                layout.text_input = Some(query);
                layout.text_input_node = Some(NODE_LIBRARY_SEARCH);
            }
        });

    let labels = LibraryTab::ALL
        .iter()
        .map(|candidate| localized(candidate.translation_key(), &library.language))
        .collect::<Vec<_>>();
    let selected = LibraryTab::ALL
        .iter()
        .position(|candidate| *candidate == tab)
        .unwrap_or_default();
    egui::Area::new(Id::new("fluxa-shared-library-tabs"))
        .fixed_pos(Pos2::new(page.margin, page.filters_top))
        .show(context, |ui| {
            let tabs_width = if split {
                page.width
            } else {
                page.width - 440.0 - metrics.section_gap * 2.0
            };
            ui.set_width(tabs_width);
            let scroll = egui::ScrollArea::horizontal()
                .id_salt("fluxa-library-tabs")
                .auto_shrink([false, true])
                .max_width(tabs_width)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    components::text_tabs(
                        ui,
                        &labels,
                        selected,
                        metrics.screen_control_height,
                        metrics,
                    )
                });
            for (index, response) in scroll.inner.into_iter().enumerate() {
                let node = NODE_LIBRARY_TAB_BASE + index as u64;
                layout
                    .focusable
                    .push((node, response.rect.intersect(scroll.inner_rect)));
                if response.clicked() {
                    layout.activated = Some(node);
                }
            }
        });

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
    let controls_width = if split { page.width.min(440.0) } else { 440.0 };
    let sort_width = (controls_width - metrics.control_gap) * 0.42;
    let source_width = controls_width - metrics.control_gap - sort_width;
    let controls_pos = match page.second_row_top {
        Some(row) => Pos2::new(page.margin, row),
        None => Pos2::new(page.margin + page.width - controls_width, page.filters_top),
    };
    egui::Area::new(Id::new("fluxa-shared-library-controls"))
        .fixed_pos(controls_pos)
        .show(context, |ui| {
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
    painter.hline(
        page.margin..=page.margin + page.width,
        page.content_top - metrics.section_gap * 0.75,
        egui::Stroke::new(1.0, Color32::from_white_alpha(18)),
    );

    let grid = PosterGrid::new(page.width, metrics);
    let origin = Pos2::new(page.margin, page.content_top - scroll_y);
    let clip = Rect::from_min_max(
        Pos2::new(0.0, page.content_top - metrics.section_gap * 0.5),
        Pos2::new(
            viewport.width,
            viewport.height - mobile_scroll_reserve(viewport),
        ),
    );
    egui::Area::new(Id::new("fluxa-shared-library-grid"))
        .fixed_pos(origin)
        .show(context, |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(clip));
            for (index, card) in cards.iter().take(LIBRARY_CARD_LIMIT).enumerate() {
                let rect = grid.cell(origin, index);
                let visible = rect.intersect(clip);
                if !visible.is_positive() {
                    continue;
                }
                let node = NODE_CARD_BASE + index as u64;
                let response = ui.interact(rect, Id::new(("library-card", index)), Sense::click());
                layout.focusable.push((node, visible));
                if response.clicked() {
                    layout.activated = Some(node);
                }
                let poster =
                    Rect::from_min_size(rect.min, Vec2::new(grid.card_width, grid.poster_height));
                components::poster_card(
                    ui.painter(),
                    poster,
                    card,
                    index,
                    viewport,
                    metrics,
                    assets,
                    false,
                );
                if card.progress > 0.0 {
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            poster.left_bottom() - Vec2::new(0.0, metrics.card_progress_height),
                            Vec2::new(
                                poster.width() * card.progress.clamp(0.0, 1.0),
                                metrics.card_progress_height,
                            ),
                        ),
                        2.0,
                        metrics.accent,
                    );
                }
            }
        });
    if cards.is_empty() {
        let bottom = viewport.height - mobile_scroll_reserve(viewport);
        let state_rect = Rect::from_min_max(
            Pos2::new(page.margin, page.content_top - scroll_y),
            Pos2::new(
                page.margin + page.width,
                bottom.max(page.content_top + 200.0),
            ),
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
