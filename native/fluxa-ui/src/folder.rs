use super::*;
use egui::Sense;

const HEADER_ART: f32 = 56.0;

struct FolderGeometry {
    page: PageLayout,
    header_height: f32,
    tabs_top: Option<f32>,
    content_top: f32,
}

fn folder_geometry(viewport: Viewport, metrics: UiMetrics, folder: &FolderModel) -> FolderGeometry {
    let page = PageLayout::new(viewport, metrics, false);
    let header_height = HEADER_ART;
    let show_tabs = folder.view_mode == FolderViewMode::TabbedGrid && folder.tabs.len() > 1;
    let mut y = page.top + header_height + metrics.section_gap;
    let tabs_top = show_tabs.then_some(y);
    if show_tabs {
        y += metrics.screen_control_height + metrics.section_gap;
    }
    FolderGeometry {
        page,
        header_height,
        tabs_top,
        content_top: y,
    }
}

fn rows_home(folder: &FolderModel) -> HomeModel {
    HomeModel {
        rows: folder.home_rows(),
        show_hero_section: false,
        cards: Vec::new(),
        row_scroll_offsets: folder.row_scroll_offsets.clone(),
        language: folder.language.clone(),
        ..HomeModel::default()
    }
}

fn folder_content_height(metrics: UiMetrics, folder: &FolderModel, tab: usize, width: f32) -> f32 {
    if folder.view_mode == FolderViewMode::TabbedGrid {
        let grid = PosterGrid::new(width, metrics);
        return grid.height(folder.cards(tab).len().min(FOLDER_CARD_LIMIT));
    }
    let home = rows_home(folder);
    home_shelves(metrics, &home, 0.0)
        .last()
        .map_or(0.0, |shelf| shelf.top + shelf.height() + shelf_gap(metrics))
}

const FOLDER_CARD_LIMIT: usize = 600;

pub fn folder_scroll_max(viewport: Viewport, folder: &FolderModel, tab: usize) -> f32 {
    let metrics = UiMetrics::for_viewport(viewport);
    let geometry = folder_geometry(viewport, metrics, folder);
    let content = folder_content_height(metrics, folder, tab, geometry.page.width);
    (geometry.content_top + content + metrics.section_gap
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}

pub fn draw_folder(
    context: &egui::Context,
    viewport: Viewport,
    folder: &FolderModel,
    tab: usize,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let metrics = metrics_for_assets(viewport, assets);
    let geometry = folder_geometry(viewport, metrics, folder);
    let page = &geometry.page;
    let tab = tab.min(folder.tabs.len().saturating_sub(1));
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 0, assets);
    let measured = kit::scroll_state(context, "folder");
    let scroll_y = measured.map_or(0.0, |(offset, _)| offset);
    let sticky_on = scroll_y > 8.0;
    if let Some(sticky) = components::sticky_bar(
        context,
        "fluxa-folder",
        viewport,
        page.margin,
        sticky_on,
        components::StickyBar {
            back_node: NODE_FOLDER_BACK,
            back_icon: assets.icon("ArrowLeft"),
            logo: None,
            title: folder.title.as_str(),
            actions: Vec::new(),
        },
        metrics,
    ) {
        layout.focusable.extend(sticky.rects);
        layout.activated = sticky.clicked.or(layout.activated);
    }

    let mut any_cards = false;
    kit::page(context, viewport, "folder", |ui| {
        let width = ui.available_width();
        kit::scroll_body_keyed(ui, viewport, "folder", (&folder.id, tab), |ui| {
            let clip = ui.clip_rect();
            let has_art = folder.cover_url.is_some() || folder.emoji.is_some();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = metrics.control_gap * 1.5;
                ui.set_height(geometry.header_height);
                let back = components::back_button(ui, assets.icon("ArrowLeft"), metrics);
                if !sticky_on {
                    layout.focusable.push((NODE_FOLDER_BACK, back.rect));
                }
                if back.clicked() {
                    layout.activated = Some(NODE_FOLDER_BACK);
                }
                if has_art {
                    let (art, _) = ui.allocate_exact_size(Vec2::splat(HEADER_ART), Sense::hover());
                    if folder.cover_url.is_some() {
                        components::rounded_artwork(
                            ui.painter(),
                            art,
                            metrics.card_radius,
                            folder.cover_url.as_deref(),
                            [160, 160],
                            ArtworkPriority::Visible,
                            Color32::WHITE,
                            assets,
                        );
                    } else if let Some(emoji) = &folder.emoji {
                        ui.painter()
                            .rect_filled(art, metrics.card_radius, metrics.surface);
                        let galley = ui.painter().layout_job(crate::emoji::job(
                            emoji,
                            FontId::proportional(HEADER_ART * 0.55),
                            text_primary(),
                            art.width(),
                        ));
                        let pos = art.center() - galley.size() * 0.5;
                        crate::emoji::paint(ui.painter(), pos, galley, assets);
                    }
                }
                crate::emoji::label(
                    ui,
                    assets,
                    &folder.title,
                    metrics.screen_title_size,
                    metrics.text_primary,
                    crate::emoji::Label::default(),
                );
            });
            ui.add_space(metrics.section_gap);

            if geometry.tabs_top.is_some() {
                let labels: Vec<String> = folder.tabs.iter().map(FolderTab::full_label).collect();
                crate::components::pan_strip(ui, "fluxa-folder-tabs-scroll", |ui| {
                    let responses = components::text_tabs(
                        ui,
                        &labels,
                        tab,
                        metrics.screen_control_height,
                        metrics,
                    );
                    for (index, response) in responses.into_iter().enumerate() {
                        let node = NODE_FOLDER_TAB_BASE + index as u64;
                        components::focus_action(&mut layout, node, &response);
                    }
                });
                ui.add_space(metrics.section_gap);
            }

            let content_top = ui.cursor().min.y;
            if folder.view_mode == FolderViewMode::TabbedGrid {
                let cards = folder.cards(tab);
                any_cards = !cards.is_empty();
                let shown = cards.len().min(FOLDER_CARD_LIMIT);
                let poster_grid = PosterGrid::new(width, metrics);
                kit::grid(shown)
                    .cell(Vec2::new(poster_grid.card_width, poster_grid.card_height))
                    .per_row(poster_grid.columns)
                    .gap_px(poster_grid.gap)
                    .row_gap_px(poster_grid.row_gap)
                    .start()
                    .show(ui, |ui, index| {
                        let rect = ui.max_rect();
                        let poster = Rect::from_min_size(
                            rect.min,
                            Vec2::new(poster_grid.card_width, poster_grid.poster_height),
                        );
                        let visible = rect.intersect(clip);
                        if visible.is_positive() {
                            let node = NODE_CARD_BASE + index as u64;
                            let response =
                                ui.interact(rect, Id::new(("folder-card", node)), Sense::click());
                            layout.focusable.push((node, visible));
                            if response.clicked() {
                                layout.activated = Some(node);
                            }
                        }
                        components::poster_card(
                            ui.painter(),
                            poster,
                            &cards[index],
                            metrics,
                            assets,
                            false,
                        );
                    });
            } else {
                let home = rows_home(folder);
                any_cards = !home.rows.is_empty();
                let mut activated = None;
                draw_home_rows(
                    context,
                    viewport,
                    &home,
                    assets,
                    focused,
                    metrics,
                    0.0,
                    content_top,
                    false,
                    page.margin,
                    screen,
                    &mut layout,
                    &mut activated,
                );
                layout.activated = layout.activated.or(activated);
                let height = folder_content_height(metrics, folder, tab, width);
                ui.allocate_space(Vec2::new(width, height));
            }
            ui.add_space(metrics.section_gap);

            if !any_cards {
                let active = folder.tab(tab);
                let loading = active.is_none_or(|tab| tab.loading)
                    || folder
                        .tabs
                        .iter()
                        .any(|tab| tab.loading && tab.cards.is_empty());
                let error = active.and_then(|tab| tab.error.clone());
                let (title, description) = if let Some(error) = error {
                    (
                        localized("collections.folder_error", &folder.language),
                        error,
                    )
                } else if loading {
                    (
                        localized("library.loading", &folder.language),
                        String::new(),
                    )
                } else {
                    (
                        localized("collections.folder_empty", &folder.language),
                        String::new(),
                    )
                };
                let bottom =
                    (clip.bottom() - mobile_scroll_reserve(viewport)).max(content_top + 200.0);
                components::empty_state(
                    ui.painter(),
                    Rect::from_min_max(
                        Pos2::new(page.margin, content_top),
                        Pos2::new(page.margin + page.width, bottom),
                    ),
                    assets.icon("Library"),
                    &title,
                    &description,
                    metrics,
                );
            }
        });
    });

    if folder.view_mode == FolderViewMode::TabbedGrid
        && let Some((offset, max)) = measured
        && offset >= max - viewport.height * 0.5
    {
        let sources: Vec<&FolderTab> = if folder.has_all_tab && tab == 0 {
            folder.source_tabs().map(|(_, source)| source).collect()
        } else {
            folder.tab(tab).into_iter().collect()
        };
        for request in sources
            .into_iter()
            .filter_map(|source| source.next_page.as_ref())
        {
            let key = Id::new(("folder-page", &folder.id)).with(request.to_string());
            let seen = context.data_mut(|data| data.get_temp::<bool>(key).is_some());
            if !seen {
                context.data_mut(|data| data.insert_temp(key, true));
                layout.load_more.push(request.clone());
            }
        }
    }
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    layout
}
