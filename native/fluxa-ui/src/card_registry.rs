use crate::model::HomeCard;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

static CARDS: RwLock<Option<Arc<HashMap<String, HomeCard>>>> = RwLock::new(None);

pub fn register_cards<'a>(cards: impl IntoIterator<Item = &'a HomeCard>) {
    let known: HashMap<String, HomeCard> = cards
        .into_iter()
        .filter_map(|card| Some((card.id.clone()?, card.clone())))
        .collect();
    if let Ok(mut slot) = CARDS.write() {
        *slot = Some(Arc::new(known));
    }
}

pub(crate) fn card(id: &str) -> Option<HomeCard> {
    CARDS.read().ok()?.as_ref()?.get(id).cloned()
}

pub fn card_raw(id: &str) -> Option<serde_json::Value> {
    card(id).map(|card| card.raw.cloned())
}
