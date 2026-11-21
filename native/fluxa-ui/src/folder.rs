use super::*;

const FOLDER_CARD_LIMIT: usize = 600;

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
