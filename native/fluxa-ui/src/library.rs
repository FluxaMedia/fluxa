use super::*;
use crate::dp::Dp;
use crate::gfx::Ui;
use crate::kit;

pub fn draw_library(
    context: &crate::gfx::Context,
    viewport: Viewport,
    library: &LibraryModel,
    tab: LibraryTab,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(
        context,
        viewport,
        if library.downloads_open { 6 } else { 2 },
        assets,
    );
    if library.downloads_open {
        draw_library_downloads(
            viewport,
            library,
            context,
            assets,
            metrics,
            &mut layout,
            &painter,
        );
    } else {
        draw_library_main(
            context,
            viewport,
            library,
            tab,
            assets,
            metrics,
            &mut layout,
            &painter,
        );
    }
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    layout
}

fn draw_library_downloads(
    viewport: Viewport,
    library: &LibraryModel,
    context: &crate::gfx::Context,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
    painter: &crate::gfx::Painter,
) {
    let compact = viewport.is_compact();
    let sizes = PageLayout::new(viewport, metrics, false);
    let cards = library.cards(LibraryTab::Watchlist);
    let open = library.open_download_group();
    kit::page(context, viewport, "library-downloads", |ui| {
        let width = ui.available_width();
        kit::view()
            .class("screen-header")
            .row()
            .height(sizes.title_height)
            .show(ui, |ui| {
                library_title(ui, &library.language, cards.len(), metrics);
            });
        ui.add_space(metrics.screen_stack_gap);
        let sections_origin = ui.cursor().min;
        library_sections_row(ui, layout, 2, None, &library.language, assets, metrics);
        kit::space(ui, Space::S12);
        if !compact && open.is_none() {
            draw_download_status(ui, sections_origin, width, library, metrics, assets, layout);
        }
        kit::scroll_body_keyed(
            ui,
            viewport,
            "library-downloads",
            open.map(|(position, _)| position),
            |ui| {
                if let Some((position, group)) = open {
                    let first_index = library.downloads[..position]
                        .iter()
                        .map(|group| group.rows.len())
                        .sum();
                    kit::view()
                        .class("screen-header")
                        .row()
                        .height(metrics.icon_button_size)
                        .show(ui, |ui| {
                            let mut back = None;
                            let mut toggle = None;
                            kit::flex("library-group-header")
                                .justify(kit::Justify::Between)
                                .center()
                                .child(|ui| {
                                    kit::view().row().gap(Space::S8).show(ui, |ui| {
                                        back = Some(components::back_button(
                                            ui,
                                            assets.icon("ArrowLeft"),
                                            metrics,
                                        ));
                                        ui.label(
                                            RichText::new(&group.title)
                                                .size(metrics.screen_title_size)
                                                .strong()
                                                .color(metrics.text_primary),
                                        );
                                    });
                                })
                                .child(|ui| {
                                    toggle = Some(components::icon_button(
                                        ui,
                                        assets.icon(if !library.list_view {
                                            "LayoutGrid"
                                        } else {
                                            "List"
                                        }),
                                        metrics.screen_control_height,
                                        text_primary(),
                                        true,
                                        false,
                                        true,
                                    ));
                                })
                                .show(ui);
                            if let Some(response) = back {
                                layout.focusable.push((NODE_DOWNLOAD_BACK, response.rect));
                                if response.clicked() {
                                    layout.activated = Some(NODE_DOWNLOAD_BACK);
                                }
                            }
                            if let Some(response) = toggle {
                                components::focus_action(layout, NODE_LIBRARY_VIEW, &response);
                            }
                        });
                    kit::space(ui, Space::S20);
                    let origin = ui.cursor().min;
                    components::download_group(
                        ui,
                        group,
                        first_index,
                        origin,
                        width,
                        !library.list_view,
                        &library.language,
                        metrics,
                        assets,
                        layout,
                    );
                    ui.allocate_space(Vec2::new(
                        width,
                        components::download_group_height(
                            group,
                            width,
                            metrics,
                            !library.list_view,
                        ),
                    ));
                    return;
                }
                if compact {
                    draw_download_status(
                        ui,
                        ui.cursor().min,
                        width,
                        library,
                        metrics,
                        assets,
                        layout,
                    );
                }
                ui.add_space(components::download_status_height(metrics));
                if library.downloads.is_empty() {
                    let rect = ui.available_rect_before_wrap();
                    ui.allocate_rect(rect, crate::gfx::Sense::hover());
                    components::empty_state(
                        painter,
                        Rect::from_min_size(
                            rect.min,
                            Vec2::new(width, rect.height().max(metrics.screen_empty_min_height)),
                        ),
                        assets.icon("Downloads"),
                        &localized("library.downloads_empty", &library.language),
                        &localized("library.downloads_hint", &library.language),
                        metrics,
                    );
                } else {
                    let origin = ui.cursor().min;
                    components::download_sections(
                        ui,
                        &library.downloads,
                        origin,
                        components::download_sections_width(width, metrics),
                        &library.language,
                        metrics,
                        assets,
                        layout,
                    );
                    ui.allocate_space(Vec2::new(
                        width,
                        components::download_sections_height(
                            &library.downloads,
                            components::download_sections_width(width, metrics),
                            metrics,
                        ),
                    ));
                }
            },
        );
    });
}

fn draw_download_status(
    ui: &mut Ui,
    origin: Pos2,
    width: f32,
    library: &LibraryModel,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let compact = metrics.compact;
    let rect = Rect::from_min_size(
        origin,
        Vec2::new(width, components::download_status_height(metrics).max(1.0)),
    );
    let mut child = ui.new_child(crate::gfx::UiBuilder::new().max_rect(rect));
    components::download_status(
        &mut child,
        &library.downloads_summary,
        Pos2::new(origin.x + width, origin.y),
        if compact {
            width
        } else {
            width - metrics.library_downloads_status_reserve
        },
        &library.language,
        metrics,
        assets,
        layout,
    );
}

fn draw_library_main(
    context: &crate::gfx::Context,
    viewport: Viewport,
    library: &LibraryModel,
    tab: LibraryTab,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
    painter: &crate::gfx::Painter,
) {
    let compact = viewport.is_compact();
    let sizes = PageLayout::new(viewport, metrics, false);
    let cards = library.cards(tab);
    let language = library.language.as_str();
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
                localized(candidate.translation_key(), language),
            )
        })
        .collect::<Vec<_>>();
    let mut type_options = vec![("all".to_owned(), localized("auto.all", language))];
    for (key, label) in [
        ("movie", "auto.movies"),
        ("series", "auto.series"),
        ("anime", "auto.anime"),
    ] {
        if library.types.iter().any(|kind| kind == key) {
            type_options.push((key.to_owned(), localized(label, language)));
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
            localized(&format!("library.sort_{sort}"), language),
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
    let has_status = !status_options.is_empty();
    let min_dropdown = metrics.screen_control_height * 2.0;

    kit::page(context, viewport, "library", |ui| {
        let width = ui.available_width();
        kit::view()
            .class("screen-header")
            .row()
            .height(sizes.title_height)
            .show(ui, |ui| {
                kit::flex("library-header")
                    .justify(kit::Justify::Between)
                    .center()
                    .child(|ui| library_title(ui, language, cards.len(), metrics))
                    .child(|ui| {
                        if !compact {
                            library_search(ui, layout, library, &sizes, metrics);
                        }
                    })
                    .show(ui);
            });
        ui.add_space(metrics.screen_stack_gap);
        library_tabs(ui, layout, library, assets, metrics);
        if compact {
            kit::space(ui, Space::S12);
            library_search(ui, layout, library, &sizes, metrics);
        }
        kit::space(ui, Space::S12);
        kit::view()
            .class("library-controls")
            .row()
            .height(metrics.screen_control_height)
            .show(ui, |ui| {
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
                let mut remaining = width;
                for (index, (id, node, key, title, selected, options)) in
                    controls.into_iter().enumerate()
                {
                    let label = options
                        .iter()
                        .find(|(option, _)| option == selected)
                        .map_or(selected, |(_, label)| label.as_str());
                    let control_width = components::dropdown_width_for_label(
                        ui,
                        label,
                        metrics,
                        min_dropdown,
                        remaining
                            - (count - index - 1) as f32 * (min_dropdown + metrics.control_gap),
                    );
                    remaining -= control_width + metrics.control_gap;
                    let (response, change) = components::dropdown(
                        ui,
                        id,
                        selected,
                        options,
                        control_width,
                        compact,
                        metrics,
                    );
                    if compact
                        && let Some(request) = components::sheet_choice(
                            node,
                            key,
                            localized(title, language),
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
        kit::space(ui, Space::S16);
        let line = ui.cursor().min.y;
        painter.hline(
            ui.min_rect().left()..=ui.min_rect().left() + width,
            line,
            crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
        );
        kit::space(ui, Space::S16);

        kit::scroll_body(ui, viewport, "library", |ui| {
            if cards.is_empty() {
                library_empty_state(ui, painter, library, assets, metrics);
                return;
            }
            let grid = PosterGrid::new(width, metrics);
            let clip = ui.clip_rect();
            let shown = cards.len();
            let layout_grid = if library.list_view {
                kit::grid(shown)
                    .columns(1, 0.0)
                    .cell_height(library_row_height(viewport, metrics))
                    .gap_px(metrics.control_gap)
            } else {
                kit::grid(shown)
                    .cell(Vec2::new(grid.card_width, grid.card_height))
                    .per_row(grid.columns)
                    .gap_px(grid.gap)
                    .row_gap_px(grid.row_gap)
                    .start()
            };
            layout_grid.show(ui, |ui, index| {
                let card = &cards[index];
                let rect = ui.max_rect();
                let poster =
                    Rect::from_min_size(rect.min, Vec2::new(grid.card_width, grid.poster_height));
                let visible = rect.intersect(clip);
                if !visible.is_positive() {
                    if !library.list_view {
                        components::poster_card(ui.painter(), poster, card, metrics, assets, false);
                    }
                    return;
                }
                let node = NODE_CARD_BASE + index as u64;
                let response = ui.interact(rect, Id::new(("library-card", index)), Sense::click());
                layout.focusable.push((node, visible));
                if response.clicked() {
                    layout.activated = Some(node);
                }
                if library.list_view {
                    components::library_row(ui.painter(), rect, card, metrics, assets);
                    return;
                }
                components::poster_card(ui.painter(), poster, card, metrics, assets, false);
                if card.progress > 0.0 {
                    ui.painter().rect_filled(
                        crate::flex::anchored(
                            poster,
                            Align2::LEFT_BOTTOM,
                            Vec2::new(
                                poster.width() * card.progress.clamp(0.0, 1.0),
                                metrics.card_progress_height,
                            ),
                            0.0,
                        ),
                        Dp::D2.px(),
                        metrics.accent,
                    );
                }
            });
        });
    });
}

fn library_title(ui: &mut Ui, language: &str, count: usize, metrics: UiMetrics) {
    ui.label(
        RichText::new(localized("nav.library", language))
            .size(metrics.screen_title_size)
            .strong()
            .color(metrics.text_primary),
    );
    if count > 0 {
        ui.label(
            RichText::new(count.to_string())
                .size(metrics.screen_body_size + 2.0)
                .color(metrics.text_muted),
        );
    }
}

fn library_search(
    ui: &mut Ui,
    layout: &mut HomeLayout,
    library: &LibraryModel,
    sizes: &PageLayout,
    metrics: UiMetrics,
) {
    let mut query = library.query.clone();
    let search = components::search_field(
        ui,
        &mut query,
        &localized("library.filter_placeholder", &library.language),
        if sizes.search.width() > ui.available_width() {
            ui.available_width()
        } else {
            sizes.search.width()
        },
        sizes.search.height(),
        metrics,
    );
    layout.focusable.push((NODE_LIBRARY_SEARCH, search.rect));
    if search.changed() {
        layout.text_input = Some(query);
        layout.text_input_node = Some(NODE_LIBRARY_SEARCH);
    }
}

fn library_tabs(
    ui: &mut Ui,
    layout: &mut HomeLayout,
    library: &LibraryModel,
    assets: &impl HomeAssets,
    metrics: UiMetrics,
) {
    library_sections_row(
        ui,
        layout,
        0,
        Some(library.list_view),
        &library.language,
        assets,
        metrics,
    );
}

fn library_empty_state(
    ui: &mut Ui,
    painter: &crate::gfx::Painter,
    library: &LibraryModel,
    assets: &impl HomeAssets,
    metrics: UiMetrics,
) {
    let language = library.language.as_str();
    let title = if library.is_loading {
        localized("library.loading", language)
    } else if library.error.is_some() {
        localized("library.error", language)
    } else {
        localized("library.empty", language)
    };
    let description = if library.is_loading || library.error.is_some() {
        library.error.clone().unwrap_or_default()
    } else {
        localized("library.add_titles_hint", language)
    };
    let rect = ui.available_rect_before_wrap();
    ui.allocate_rect(rect, Sense::hover());
    components::empty_state(
        painter,
        rect,
        assets.icon("Library"),
        &title,
        &description,
        metrics,
    );
}

pub(crate) fn draw_library_sections(
    context: &crate::gfx::Context,
    layout: &mut HomeLayout,
    pos: Pos2,
    width: f32,
    active: usize,
    list_view: Option<bool>,
    language: &str,
    assets: &impl HomeAssets,
    metrics: UiMetrics,
) {
    components::placed(
        context,
        Id::new("fluxa-shared-library-sections"),
        pos,
        |ui| {
            ui.set_width(width);
            library_sections_row(ui, layout, active, list_view, language, assets, metrics);
        },
    );
}

pub(crate) fn library_sections_row(
    ui: &mut Ui,
    layout: &mut HomeLayout,
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
    kit::view()
        .class("library-tabs")
        .row()
        .height(metrics.screen_control_height)
        .show(ui, |ui| {
            let mut tabs = Vec::new();
            let mut toggle = None;
            kit::flex(("library-sections", active))
                .justify(kit::Justify::Between)
                .center()
                .child(|ui| {
                    tabs = components::text_tabs(
                        ui,
                        &labels,
                        active,
                        metrics.screen_control_height,
                        metrics,
                    );
                })
                .child(|ui| {
                    toggle = list_view.map(|list| {
                        components::icon_button(
                            ui,
                            assets.icon(if list { "LayoutGrid" } else { "List" }),
                            metrics.screen_control_height,
                            text_primary(),
                            true,
                            false,
                            true,
                        )
                    });
                })
                .show(ui);
            for (index, response) in tabs.into_iter().enumerate() {
                let node = NODE_LIBRARY_SECTION_BASE + index as u64;
                components::focus_action(layout, node, &response);
            }
            if let Some(response) = toggle {
                components::focus_action(layout, NODE_LIBRARY_VIEW, &response);
            }
        });
}
