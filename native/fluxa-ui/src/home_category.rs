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
    let slides: Vec<_> = model
        .rows
        .iter()
        .filter(|row| row.kind == crate::HomeRowKind::Poster)
        .flat_map(|row| row.cards.iter())
        .map(|card| crate::model::core_home_hero(&card.raw, &model.language))
        .filter(|hero| hero.background_url.is_some() && hero.item_id.is_some())
        .take(HERO_SLIDES)
        .collect();
    if let Some(first) = slides.first() {
        model.title = first.title.clone();
        model.eyebrow = first.eyebrow.clone();
        model.description = first.description.clone();
        model.background_url = first.background_url.clone();
        model.logo_url = first.logo_url.clone();
        model.item_id = first.item_id.clone();
        model.item_type = first.item_type.clone();
    }
    model.hero_slides = slides;
}

const HERO_SLIDES: usize = 5;
