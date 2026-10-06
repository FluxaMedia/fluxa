use super::*;

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
    home.content_rows_with_kind()
        .map(|(_, cards, kind)| {
            home_row_heading_height(metrics)
                + home_row_body_height(metrics, cards, kind)
                + metrics.section_gap
                + metrics.vertical_spacing
        })
        .sum()
}

const FOLDER_CARD_LIMIT: usize = 600;

pub fn folder_row_at_y(
    viewport: Viewport,
    folder: &FolderModel,
    scroll_y: f32,
    y: f32,
) -> Option<usize> {
    if folder.view_mode == FolderViewMode::TabbedGrid {
        return None;
    }
    let metrics = UiMetrics::for_viewport(viewport);
    let mut row_y = folder_geometry(viewport, metrics, folder).content_top - scroll_y;
    for (index, (_, cards, kind)) in rows_home(folder).content_rows_with_kind().enumerate() {
        let heading = home_row_heading_height(metrics);
        let body = home_row_body_height(metrics, cards, kind);
        if y >= row_y + heading && y <= row_y + heading + body {
            return Some(index);
        }
        row_y += heading + body + metrics.section_gap + metrics.vertical_spacing;
    }
    None
}

pub fn folder_row_scroll_max(viewport: Viewport, folder: &FolderModel, row: usize) -> f32 {
    home_row_scroll_max(viewport, &rows_home(folder), row)
}

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
    let scroll_y = resolve_screen_scroll(
        context,
        viewport,
        Id::new("fluxa-screen-scroll-folder").with((&folder.id, tab)),
        folder_scroll_max(viewport, folder, tab),
    );
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    paint_ambient(&painter, screen, assets);
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 0, assets);

    let back_rect = Rect::from_min_size(
        Pos2::new(
            page.margin,
            page.top + (geometry.header_height - metrics.icon_button_size) * 0.5 - scroll_y,
        ),
        Vec2::splat(metrics.icon_button_size),
    );
    let sticky_on = scroll_y > geometry.content_top * 0.6;
    if !sticky_on {
        components::placed_front(context, Id::new("fluxa-folder-back"), back_rect.min, |ui| {
            let response = components::back_button(ui, assets.icon("ArrowLeft"), metrics);
            if response.clicked() {
                layout.activated = Some(NODE_FOLDER_BACK);
            }
        });
        layout.focusable.push((NODE_FOLDER_BACK, back_rect));
    }
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

    let art = Rect::from_min_size(
        Pos2::new(back_rect.right() + metrics.control_gap * 1.5, page.top),
        Vec2::splat(HEADER_ART),
    );
    let text_left = if folder.cover_url.is_some() || folder.emoji.is_some() {
        art.right() + metrics.control_gap * 1.5
    } else {
        art.left()
    };
    if folder.cover_url.is_some() {
        components::rounded_artwork(
            &painter,
            art,
            metrics.card_radius,
            folder.cover_url.as_deref(),
            [160, 160],
            ArtworkPriority::Visible,
            Color32::WHITE,
            assets,
        );
    } else if let Some(emoji) = &folder.emoji {
        painter.rect_filled(art, metrics.card_radius, metrics.surface);
        let galley = painter.layout_job(crate::emoji::job(
            emoji,
            FontId::proportional(HEADER_ART * 0.55),
            text_primary(),
            art.width(),
        ));
        let pos = art.center() - galley.size() * 0.5;
        crate::emoji::paint(&painter, pos, galley, assets);
    }
    components::placed(
        context,
        Id::new("fluxa-folder-title"),
        Pos2::new(text_left, page.top),
        |ui| {
            ui.set_height(geometry.header_height);
            ui.set_max_width((viewport.width - text_left - page.margin).max(1.0));
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(&folder.title)
                        .size(metrics.screen_title_size)
                        .strong()
                        .color(metrics.text_primary),
                );
            });
        },
    );

    if let Some(tabs_top) = geometry.tabs_top {
        let labels: Vec<String> = folder.tabs.iter().map(FolderTab::full_label).collect();
        components::placed(
            context,
            Id::new("fluxa-folder-tabs"),
            Pos2::new(page.margin, tabs_top),
            |ui| {
                ui.set_width(page.width);
                egui::ScrollArea::horizontal()
                    .id_salt("fluxa-folder-tabs-scroll")
                    .show(ui, |ui| {
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
            },
        );
    }

    let grid = PosterGrid::new(page.width, metrics);
    let origin = Pos2::new(page.margin, geometry.content_top - scroll_y);
    let clip = Rect::from_min_max(
        Pos2::new(0.0, geometry.content_top - metrics.section_gap * 0.5),
        Pos2::new(
            viewport.width,
            viewport.height - mobile_scroll_reserve(viewport),
        ),
    );
    let mut any_cards = false;
    components::placed(context, Id::new("fluxa-folder-grid"), origin, |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(clip));
        if folder.view_mode == FolderViewMode::TabbedGrid {
            let cards = folder.cards(tab);
            any_cards = !cards.is_empty();
            for (index, card) in cards.iter().take(FOLDER_CARD_LIMIT).enumerate() {
                let rect = grid.cell(origin, index);
                let visible = rect.intersect(clip);
                if !visible.is_positive() {
                    let poster = Rect::from_min_size(
                        rect.min,
                        Vec2::new(grid.card_width, grid.poster_height),
                    );
                    components::poster_card(ui.painter(), poster, card, metrics, assets, false);
                    continue;
                }
                let node = NODE_CARD_BASE + index as u64;
                let response = ui.interact(rect, Id::new(("folder-card", node)), Sense::click());
                layout.focusable.push((node, visible));
                if response.clicked() {
                    layout.activated = Some(node);
                }
                let poster =
                    Rect::from_min_size(rect.min, Vec2::new(grid.card_width, grid.poster_height));
                components::poster_card(ui.painter(), poster, card, metrics, assets, false);
            }
        }
    });
    if folder.view_mode != FolderViewMode::TabbedGrid {
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
            scroll_y,
            geometry.content_top,
            false,
            page.margin,
            screen,
            &mut layout,
            &mut activated,
        );
        layout.activated = layout.activated.or(activated);
    }

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
        let bottom = viewport.height - mobile_scroll_reserve(viewport);
        components::empty_state(
            &painter,
            Rect::from_min_max(
                Pos2::new(page.margin, geometry.content_top),
                Pos2::new(
                    page.margin + page.width,
                    bottom.max(geometry.content_top + 200.0),
                ),
            ),
            assets.icon("Library"),
            &title,
            &description,
            metrics,
        );
    }
    if folder.view_mode == FolderViewMode::TabbedGrid
        && scroll_y + viewport.height
            >= folder_scroll_max(viewport, folder, tab) + viewport.height * 0.5
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
    components::focus_ring(&painter, &layout.focusable, focused, viewport, metrics);
    layout
}
