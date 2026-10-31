use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use crate::model::HomeCard;

#[derive(Default)]
struct Logos {
    enabled: bool,
    found: HashMap<String, String>,
    queued: HashSet<String>,
    wanted: Vec<(String, String)>,
}

fn logos() -> std::sync::MutexGuard<'static, Logos> {
    static LOGOS: OnceLock<Mutex<Logos>> = OnceLock::new();
    LOGOS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn own_logo(card: &HomeCard) -> Option<&str> {
    card.logo_url
        .as_deref()
        .filter(|url| !url.trim().is_empty())
}

pub(crate) fn logo_of(card: &HomeCard) -> Option<String> {
    if let Some(url) = own_logo(card) {
        return Some(url.to_owned());
    }
    logos()
        .found
        .get(card.id.as_deref()?)
        .filter(|url| !url.is_empty())
        .cloned()
}

pub(crate) fn want(card: &HomeCard) {
    if own_logo(card).is_some() {
        return;
    }
    let (Some(id), Some(kind)) = (card.id.as_deref(), card.item_type.as_deref()) else {
        return;
    };
    let mut logos = logos();
    if !logos.enabled || logos.found.contains_key(id) || logos.queued.contains(id) {
        return;
    }
    logos.queued.insert(id.to_owned());
    logos.wanted.push((id.to_owned(), kind.to_owned()));
}

pub(crate) fn awaiting(card: &HomeCard) -> bool {
    if own_logo(card).is_some() {
        return false;
    }
    let Some(id) = card.id.as_deref() else {
        return false;
    };
    let logos = logos();
    logos.enabled && !logos.found.contains_key(id)
}

pub fn enable_card_logos(known: HashMap<String, String>) {
    let mut logos = logos();
    logos.enabled = true;
    logos.found.extend(known);
}

pub fn take_wanted_logos(max: usize) -> Vec<(String, String)> {
    let mut logos = logos();
    let keep = logos.wanted.len().saturating_sub(max);
    logos.wanted.split_off(keep)
}

pub fn resolve_card_logos(entries: Vec<(String, String)>) -> HashMap<String, String> {
    let mut logos = logos();
    for (id, url) in entries {
        logos.queued.remove(&id);
        logos.found.insert(id, url);
    }
    logos
        .found
        .iter()
        .filter(|(_, url)| !url.is_empty())
        .map(|(id, url)| (id.clone(), url.clone()))
        .collect()
}
