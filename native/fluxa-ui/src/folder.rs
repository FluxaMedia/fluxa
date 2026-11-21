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
    let mut y = page.top + header_height + crate::vars::px("section-gap");
    let tabs_top = show_tabs.then_some(y);
    if show_tabs {
        y += crate::vars::px("screen-control-height") + crate::vars::px("section-gap");
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
        let grid = PosterGrid::new(width, &metrics);
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
    (geometry.content_top + content + crate::vars::px("section-gap")
        - (viewport.height - mobile_scroll_reserve(viewport)))
    .max(0.0)
}

pub fn folder_data(folder: &FolderModel, tab: usize) -> serde_json::Value {
    let tab = tab.min(folder.tabs.len().saturating_sub(1));
    let grid = folder.view_mode == FolderViewMode::TabbedGrid;
    let current = folder.tab(tab);
    let tabs: Vec<_> = folder
        .tabs
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            serde_json::json!({
                "index": index.to_string(),
                "label": entry.full_label(),
                "selected": index == tab,
            })
        })
        .collect();
    let cards: Vec<_> = folder.cards(tab).iter().take(FOLDER_CARD_LIMIT).collect();
    let rows = folder.home_rows();
    serde_json::json!({
        "title": folder.title,
        "cover": folder.cover_url,
        "emoji": folder.emoji,
        "tabs": tabs,
        "showTabs": grid && folder.tabs.len() > 1,
        "grid": grid,
        "cards": if grid { serde_json::to_value(&cards).unwrap_or_default() } else { serde_json::Value::Null },
        "rows": if grid { Vec::new() } else { rows.iter().map(|row| serde_json::to_value(row).unwrap_or_default()).collect() },
        "loading": current.is_some_and(|tab| tab.loading),
        "error": current.and_then(|tab| tab.error.clone()),
        "empty": if grid { cards.is_empty() } else { rows.is_empty() },
    })
}
