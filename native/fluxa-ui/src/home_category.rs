use std::sync::atomic::{AtomicU8, Ordering};

use crate::model::HomeModel;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HomeCategory {
    #[default]
    Home,
    Movies,
    Series,
    Anime,
}

static ACTIVE: AtomicU8 = AtomicU8::new(0);

impl HomeCategory {
    pub const ALL: [HomeCategory; 4] = [
        HomeCategory::Home,
        HomeCategory::Movies,
        HomeCategory::Series,
        HomeCategory::Anime,
    ];

    pub fn catalog_type(self) -> Option<&'static str> {
        match self {
            HomeCategory::Home => None,
            HomeCategory::Movies => Some("movie"),
            HomeCategory::Series => Some("series"),
            HomeCategory::Anime => Some("anime"),
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            HomeCategory::Home => "home.category.home",
            HomeCategory::Movies => "home.category.movies",
            HomeCategory::Series => "home.category.series",
            HomeCategory::Anime => "home.category.anime",
        }
    }

    pub fn node(self) -> u64 {
        crate::NODE_HOME_CATEGORY_BASE + self as u64
    }

    pub fn from_node(node: u64) -> Option<HomeCategory> {
        node.checked_sub(crate::NODE_HOME_CATEGORY_BASE)
            .and_then(|index| Self::ALL.get(index as usize).copied())
    }
}

pub fn home_category() -> HomeCategory {
    HomeCategory::ALL
        .get(ACTIVE.load(Ordering::Relaxed) as usize)
        .copied()
        .unwrap_or_default()
}

pub fn set_home_category(category: HomeCategory) {
    ACTIVE.store(category as u8, Ordering::Relaxed);
}

fn row_type(row: &crate::HomeRow) -> Option<&str> {
    row.catalog_page
        .as_ref()
        .and_then(|page| page.get("type"))
        .and_then(serde_json::Value::as_str)
}

pub(crate) fn apply(model: &mut HomeModel, enabled: bool) {
    model.category_bar = enabled;
    model.categories.clear();
    model.active_category = HomeCategory::Home;
    if !enabled {
        return;
    }
    model.categories.push(HomeCategory::Home);
    for category in &HomeCategory::ALL[1..] {
        if model
            .rows
            .iter()
            .any(|row| row_type(row) == category.catalog_type())
        {
            model.categories.push(*category);
        }
    }
    let active = home_category();
    if active == HomeCategory::Home || !model.categories.contains(&active) {
        return;
    }
    model.active_category = active;
    model
        .rows
        .retain(|row| row_type(row) == active.catalog_type());
    model.cards.clear();
    model.hero_slides.clear();
    model.show_hero_section = false;
}
