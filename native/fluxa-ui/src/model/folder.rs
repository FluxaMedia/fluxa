use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FolderViewMode {
    #[default]
    TabbedGrid,
    Rows,
    FollowLayout,
}

impl FolderViewMode {
    fn parse(value: Option<&str>) -> Self {
        match value.map(str::to_ascii_lowercase).as_deref() {
            Some("rows") => Self::Rows,
            Some("follow_layout") => Self::FollowLayout,
            _ => Self::TabbedGrid,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FolderTab {
    pub category_id: String,
    pub label: String,
    pub cards: Vec<HomeCard>,
    pub loading: bool,
    pub error: Option<String>,
    pub next_page: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default)]
pub struct FolderModel {
    pub language: String,
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub emoji: Option<String>,
    pub shape: Option<String>,
    pub view_mode: FolderViewMode,
    pub tabs: Vec<FolderTab>,
    pub has_all_tab: bool,
}

impl FolderModel {
    pub fn tab(&self, index: usize) -> Option<&FolderTab> {
        self.tabs.get(index)
    }

    pub fn cards(&self, index: usize) -> &[HomeCard] {
        self.tabs
            .get(index)
            .map(|tab| tab.cards.as_slice())
            .unwrap_or_default()
    }

    pub fn card_at(&self, tab: usize, index: usize) -> Option<&HomeCard> {
        if self.view_mode == FolderViewMode::TabbedGrid {
            return self.cards(tab).get(index);
        }
        self.source_tabs()
            .flat_map(|(_, tab)| tab.cards.iter())
            .nth(index)
    }

    pub fn home_rows(&self) -> Vec<HomeRow> {
        self.source_tabs()
            .filter(|(_, tab)| !tab.cards.is_empty())
            .map(|(_, tab)| {
                let mut page = tab.next_page.clone();
                if let Some(fields) = page.as_mut().and_then(serde_json::Value::as_object_mut) {
                    fields.remove("skip");
                }
                HomeRow {
                    id: Some(tab.category_id.clone()),
                    title: tab.label.clone(),
                    type_label: None,
                    cards: tab.cards.clone(),
                    kind: HomeRowKind::Poster,
                    can_load_more: tab.next_page.is_some(),
                    catalog_page: page,
                }
            })
            .collect()
    }

    pub fn source_tabs(&self) -> impl Iterator<Item = (usize, &FolderTab)> {
        let skip = usize::from(self.has_all_tab);
        self.tabs.iter().enumerate().skip(skip)
    }
}

pub fn folder_model_from_core_snapshot(snapshot: &serde_json::Value) -> FolderModel {
    let language = snapshot_language(snapshot);
    let Some(folder_id) = snapshot
        .pointer("/navigation/params/folderId")
        .and_then(serde_json::Value::as_str)
    else {
        return FolderModel {
            language,
            ..FolderModel::default()
        };
    };
    let categories = snapshot
        .pointer("/home/categories")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let definition = categories.iter().find(|category| {
        value_string(category, "id").as_deref() == Some(folder_id)
            && value_string(category, "type").as_deref() == Some("collection_folder")
    });
    let tile = categories
        .iter()
        .filter(|category| value_string(category, "type").as_deref() == Some("collection"))
        .filter_map(|category| category.get("items").and_then(serde_json::Value::as_array))
        .flatten()
        .find(|item| value_string(item, "id").as_deref() == Some(folder_id));
    let mut tabs: Vec<FolderTab> = categories
        .iter()
        .filter(|category| {
            value_string(category, "type").as_deref() == Some("collection_folder_source")
                && value_string(category, "folderId").as_deref() == Some(folder_id)
        })
        .map(|category| {
            let items = category
                .get("items")
                .and_then(serde_json::Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let can_more = category
                .get("canLoadMore")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let category_id = value_string(category, "id").unwrap_or_default();
            let failed = category.get("error").is_some_and(|error| !error.is_null());
            FolderTab {
                label: value_string(category, "name").unwrap_or_default(),
                cards: items
                    .iter()
                    .map(|item| core_home_card_for_kind(item, HomeRowKind::Poster))
                    .collect(),
                loading: items.is_empty() && can_more && !failed,
                error: failed.then(|| localized("collections.folder_error", &language)),
                next_page: (can_more && !items.is_empty()).then(|| {
                    serde_json::json!({
                        "type": "catalogPageRequested",
                        "categoryId": category_id,
                        "transportUrl": category.get("transportUrl"),
                        "contentType": category.get("contentType"),
                        "catalogId": category.get("catalogId"),
                        "genre": category.get("genre"),
                        "remoteSource": category.get("remoteSource"),
                        "skip": items.len(),
                        "profile": serde_json::Value::Null,
                    })
                }),
                category_id,
            }
        })
        .collect();
    tabs.sort_by_key(|tab| {
        tab.category_id
            .rsplit('#')
            .next()
            .and_then(|index| index.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    let show_all = definition
        .and_then(|folder| folder.get("showAllTab"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let has_all_tab = show_all && tabs.len() > 1;
    if has_all_tab {
        let mut seen = std::collections::HashSet::new();
        let merged: Vec<HomeCard> = tabs
            .iter()
            .flat_map(|tab| tab.cards.iter())
            .filter(|card| seen.insert((card.id.clone(), card.item_type.clone())))
            .cloned()
            .collect();
        tabs.insert(
            0,
            FolderTab {
                category_id: String::new(),
                label: localized("collections.tab_all", &language),
                loading: tabs.iter().all(|tab| tab.loading),
                error: None,
                next_page: None,
                cards: merged,
            },
        );
    }
    FolderModel {
        id: folder_id.to_owned(),
        title: definition
            .and_then(|folder| value_string(folder, "name"))
            .or_else(|| tile.and_then(|tile| value_string(tile, "name")))
            .unwrap_or_default(),
        cover_url: tile.and_then(|tile| value_string(tile, "poster")),
        emoji: tile.and_then(|tile| value_string(tile, "coverEmoji")),
        shape: tile.and_then(|tile| value_string(tile, "reason")),
        view_mode: FolderViewMode::parse(
            definition
                .and_then(|folder| folder.get("viewMode"))
                .and_then(serde_json::Value::as_str),
        ),
        tabs,
        has_all_tab,
        language,
    }
}
