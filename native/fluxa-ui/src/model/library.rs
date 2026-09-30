use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LibraryTab {
    #[default]
    Watchlist,
    Watching,
    Completed,
    Dropped,
    OnHold,
    Liked,
}

impl LibraryTab {
    pub const ALL: [Self; 6] = [
        Self::Watchlist,
        Self::Watching,
        Self::Completed,
        Self::Dropped,
        Self::OnHold,
        Self::Liked,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Watchlist => "Watchlist",
            Self::Watching => "Watching",
            Self::Completed => "Completed",
            Self::Dropped => "Dropped",
            Self::OnHold => "On Hold",
            Self::Liked => "Favorites",
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::Watchlist => "library.watchlist",
            Self::Watching => "library.watching",
            Self::Completed => "library.completed",
            Self::Dropped => "library.dropped",
            Self::OnHold => "library.on_hold",
            Self::Liked => "library.favorites",
        }
    }

    pub fn core_tab_key(self) -> &'static str {
        match self {
            Self::Watchlist => "watchlist",
            Self::Watching => "watching",
            Self::Completed => "completed",
            Self::Dropped => "dropped",
            Self::OnHold => "hold",
            Self::Liked => "favorites",
        }
    }

    pub(super) fn state_key(self) -> &'static str {
        match self {
            Self::Watchlist => "watchlist",
            Self::Watching => "continueWatching",
            Self::Completed => "completed",
            Self::Dropped => "dropped",
            Self::OnHold => "onHold",
            Self::Liked => "liked",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LibraryModel {
    pub language: String,
    pub query: String,
    pub sort_by: String,
    pub content_type: String,
    pub list_view: bool,
    pub downloads_open: bool,
    pub types: Vec<String>,
    pub statuses: Vec<String>,
    pub sorts: Vec<String>,
    pub source: String,
    pub source_options: Vec<(String, String)>,
    pub view_tab: Option<LibraryTab>,
    pub view_cards: Vec<HomeCard>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub watchlist: Vec<HomeCard>,
    pub watching: Vec<HomeCard>,
    pub completed: Vec<HomeCard>,
    pub dropped: Vec<HomeCard>,
    pub on_hold: Vec<HomeCard>,
    pub liked: Vec<HomeCard>,
    pub personal: std::sync::Arc<PersonalIndex>,
}

impl LibraryModel {
    pub fn cards(&self, tab: LibraryTab) -> &[HomeCard] {
        if self.view_tab == Some(tab) {
            return &self.view_cards;
        }
        match tab {
            LibraryTab::Watchlist => &self.watchlist,
            LibraryTab::Watching => &self.watching,
            LibraryTab::Completed => &self.completed,
            LibraryTab::Dropped => &self.dropped,
            LibraryTab::OnHold => &self.on_hold,
            LibraryTab::Liked => &self.liked,
        }
    }

    pub fn apply_core_plan(&mut self, plan: &serde_json::Value, active_tab: LibraryTab) {
        self.view_cards = plan
            .get("items")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().take(64).map(core_home_card).collect())
            .unwrap_or_default();
        let strings = |key| -> Vec<String> {
            plan.get(key)
                .and_then(serde_json::Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                        .collect()
                })
                .unwrap_or_default()
        };
        self.types = strings("types");
        self.statuses = strings("statuses");
        self.sorts = strings("sorts");
        self.view_tab = Some(active_tab);
    }
}

/// Projects the Core library state into the same card model used by Home.
/// The renderer owns the tab/layout decision; Core only owns the data and
/// loading/error state.
pub fn library_model_from_core_snapshot(snapshot: &serde_json::Value) -> LibraryModel {
    let library = snapshot.get("library").unwrap_or(&serde_json::Value::Null);
    let language = snapshot_language(snapshot);
    let cards = |tab: LibraryTab| {
        library
            .get(tab.state_key())
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().take(64).map(core_home_card).collect())
            .unwrap_or_default()
    };
    LibraryModel {
        personal: std::sync::Arc::new(poster_overlay::personal_index(library)),
        language: language.clone(),
        query: String::new(),
        sort_by: "recent".to_owned(),
        content_type: "all".to_owned(),
        list_view: false,
        downloads_open: false,
        types: Vec::new(),
        statuses: Vec::new(),
        sorts: Vec::new(),
        source: snapshot
            .pointer("/settings/values/integrationLibrarySource")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("local")
            .to_owned(),
        source_options: {
            let profile = snapshot
                .pointer("/profile/active")
                .or_else(|| snapshot.pointer("/home/activeProfile"))
                .unwrap_or(&serde_json::Value::Null);
            let mut options = vec![(
                "local".to_owned(),
                localized("library.source_local", &language),
            )];
            for (source, token_key) in crate::settings::SOURCE_TOKENS {
                if profile
                    .get(token_key)
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|token| !token.is_empty())
                {
                    options.push((
                        source.to_owned(),
                        localized(&format!("library.source_{source}"), &language),
                    ));
                }
            }
            options
        },
        view_tab: None,
        view_cards: Vec::new(),
        is_loading: library
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        error: library
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        watchlist: cards(LibraryTab::Watchlist),
        watching: cards(LibraryTab::Watching),
        completed: cards(LibraryTab::Completed),
        dropped: cards(LibraryTab::Dropped),
        on_hold: cards(LibraryTab::OnHold),
        liked: cards(LibraryTab::Liked),
    }
}
