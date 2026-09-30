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
    paint_ambient(&painter, screen, assets);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 2, assets);

    let compact = viewport.is_compact();
    let page = PageLayout::new(viewport, metrics, false).with_sections(viewport, metrics);
    let cards = library.cards(tab);

    egui::Area::new(Id::new("fluxa-shared-library-title"))
        .constrain(false)
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

    draw_library_sections(
        context,
        &mut layout,
        Pos2::new(page.margin, page.sections_top),
        page.width,
        if library.downloads_open { 2 } else { 0 },
        (!library.downloads_open).then_some(library.list_view),
        &library.language,
        assets,
        metrics,
    );
    if library.downloads_open {
        let bottom = viewport.height - mobile_scroll_reserve(viewport);
        components::empty_state(
            &painter,
            Rect::from_min_max(
                Pos2::new(page.margin, page.filters_top),
                Pos2::new(
                    page.margin + page.width,
                    bottom.max(page.filters_top + 200.0),
                ),
            ),
            assets.icon("Downloads"),
            &localized("library.downloads_empty", &library.language),
            &localized("library.downloads_hint", &library.language),
            metrics,
        );
        layout
            .focusable
            .extend(navigation_focus_rects(viewport, metrics));
        components::focus_ring(&painter, &layout.focusable, focused, viewport, metrics);
        return layout;
    }

    egui::Area::new(Id::new("fluxa-shared-library-search"))
        .constrain(false)
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

    let status_options = LibraryTab::ALL
        .iter()
        .filter(|candidate| {
            library
                .statuses
                .iter()
                .any(|status| status == candidate.core_tab_key())
        })
        .map(|candidate| {
            (
                candidate.core_tab_key().to_owned(),
                localized(candidate.translation_key(), &library.language),
            )
        })
        .collect::<Vec<_>>();
    let mut type_options = vec![("all".to_owned(), localized("auto.all", &library.language))];
    for (key, label) in [
        ("movie", "auto.movies"),
        ("series", "auto.series"),
        ("anime", "auto.anime"),
    ] {
        if library.types.iter().any(|kind| kind == key) {
            type_options.push((key.to_owned(), localized(label, &library.language)));
        }
    }
    let sort_options = [
        "tracker",
        "recent",
        "oldest",
        "title",
        "title_desc",
        "rating",
    ]
    .into_iter()
    .filter(|sort| library.sorts.is_empty() || library.sorts.iter().any(|item| item == sort))
    .map(|sort| {
        (
            sort.to_owned(),
            localized(&format!("library.sort_{sort}"), &library.language),
        )
    })
    .collect::<Vec<_>>();
    let selected_sort = if sort_options.iter().any(|(key, _)| *key == library.sort_by) {
        library.sort_by.as_str()
    } else {
        sort_options.first().map_or("", |(key, _)| key.as_str())
    };
    let selected_type = if library.content_type.is_empty() {
        "all"
    } else {
        library.content_type.as_str()
    };
    let gap = metrics.control_gap;
    let has_status = !status_options.is_empty();
    egui::Area::new(Id::new("fluxa-shared-library-controls"))
        .constrain(false)
        .fixed_pos(Pos2::new(page.margin, page.filters_top))
        .show(context, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                let mut controls = vec![
                    (
                        "library-status",
                        NODE_LIBRARY_STATUS,
                        "libraryStatus",
                        "library.status",
                        tab.core_tab_key(),
                        &status_options,
                    ),
                    (
                        "library-type",
                        NODE_LIBRARY_TYPE,
                        "libraryType",
                        "auto.type",
                        selected_type,
                        &type_options,
                    ),
                    (
                        "library-sort",
                        NODE_LIBRARY_SORT,
                        "librarySort",
                        "library.sort_by",
                        selected_sort,
                        &sort_options,
                    ),
                ];
                if !has_status {
                    controls.remove(0);
                }
                let count = controls.len();
                let mut remaining = page.width;
                for (index, (id, node, key, title, selected, options)) in
                    controls.into_iter().enumerate()
                {
                    let label = options
                        .iter()
                        .find(|(option, _)| option == selected)
                        .map_or(selected, |(_, label)| label.as_str());
                    let width = components::dropdown_width_for_label(
                        ui,
                        label,
                        metrics,
                        72.0,
                        remaining - (count - index - 1) as f32 * (72.0 + gap),
                    );
                    remaining -= width + gap;
                    let (response, change) =
                        components::dropdown(ui, id, selected, options, width, compact, metrics);
                    if compact
                        && let Some(request) = components::sheet_choice(
                            node,
                            key,
                            localized(title, &library.language),
                            options,
                            selected,
                        )
                    {
                        layout.choices.push(request);
                    }
                    layout.focusable.push((node, response.rect));
                    if let Some(value) = change {
                        layout.filter_change = Some((key.to_owned(), value));
                    }
                }
            });
        });
    painter.hline(
        page.margin..=page.margin + page.width,
        page.content_top - metrics.section_gap * 0.75,
        egui::Stroke::new(1.0, metrics.border),
    );

    let grid = PosterGrid::new(page.width, metrics);
    let row_height = library_row_height(viewport, metrics);
    let origin = Pos2::new(page.margin, page.content_top - scroll_y);
    let clip = Rect::from_min_max(
        Pos2::new(0.0, page.content_top - metrics.section_gap * 0.5),
        Pos2::new(
            viewport.width,
            viewport.height - mobile_scroll_reserve(viewport),
        ),
    );
    egui::Area::new(Id::new("fluxa-shared-library-grid"))
        .constrain(false)
        .fixed_pos(origin)
        .show(context, |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(clip));
            for (index, card) in cards.iter().take(LIBRARY_CARD_LIMIT).enumerate() {
                let rect = if library.list_view {
                    Rect::from_min_size(
                        origin + Vec2::new(0.0, index as f32 * (row_height + metrics.control_gap)),
                        Vec2::new(page.width, row_height),
                    )
                } else {
                    grid.cell(origin, index)
                };
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
                if library.list_view {
                    components::library_row(ui.painter(), rect, card, viewport, metrics, assets);
                    continue;
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
            localized("library.loading", &library.language)
        } else if library.error.is_some() {
            localized("library.error", &library.language)
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
    components::focus_ring(&painter, &layout.focusable, focused, viewport, metrics);
    layout
}

pub(crate) fn draw_library_sections(
    context: &egui::Context,
    layout: &mut HomeLayout,
    pos: Pos2,
    width: f32,
    active: usize,
    list_view: Option<bool>,
    language: &str,
    assets: &impl HomeAssets,
    metrics: UiMetrics,
) {
    let labels = [
        localized("nav.library", language),
        localized("nav.calendar", language),
        localized("library.downloads", language),
    ];
    egui::Area::new(Id::new("fluxa-shared-library-sections"))
        .constrain(false)
        .fixed_pos(pos)
        .show(context, |ui| {
            ui.set_width(width);
            ui.horizontal(|ui| {
                let tabs = components::text_tabs(
                    ui,
                    &labels,
                    active,
                    metrics.screen_control_height,
                    metrics,
                );
                for (index, response) in tabs.into_iter().enumerate() {
                    let node = NODE_LIBRARY_SECTION_BASE + index as u64;
                    layout.focusable.push((node, response.rect));
                    if response.clicked() {
                        layout.activated = Some(node);
                    }
                }
                if let Some(list) = list_view {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let response = components::icon_button(
                            ui,
                            assets.icon(if list { "LayoutGrid" } else { "List" }),
                            metrics.screen_control_height,
                            Color32::WHITE,
                            true,
                            false,
                            true,
                        );
                        layout.focusable.push((NODE_LIBRARY_VIEW, response.rect));
                        if response.clicked() {
                            layout.activated = Some(NODE_LIBRARY_VIEW);
                        }
                    });
                }
            });
        });
}
