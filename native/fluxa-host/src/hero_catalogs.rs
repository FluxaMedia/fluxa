use serde_json::{Value, json};

use crate::{RendererState, Route};

const DEFAULT_FEEDS: usize = 2;

fn selected(state: &RendererState) -> Vec<String> {
    state
        .settings
        .values
        .get("heroFeedToggles")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn feeds(state: &RendererState) -> Vec<(String, String)> {
    let options = fluxa_core::ambient::catalog_options(&state.settings.addons);
    state
        .home
        .rows
        .iter()
        .filter(|row| {
            matches!(row.kind, fluxa_ui::HomeRowKind::Poster)
                && row.id.as_deref() != Some("continue_watching")
        })
        .filter_map(|row| {
            let id = row.id.clone()?;
            let name = options
                .iter()
                .find(|option| option.key == id)
                .map_or_else(|| row.title.clone(), |option| option.name.clone());
            Some((id, name))
        })
        .take(fluxa_ui::HERO_CATALOG_MAX)
        .collect()
}

fn effective(state: &RendererState) -> Vec<String> {
    let chosen = selected(state);
    if !chosen.is_empty() {
        return chosen;
    }
    feeds(state)
        .into_iter()
        .take(DEFAULT_FEEDS)
        .map(|(id, _)| id)
        .collect()
}

pub(crate) fn sync_settings(state: &mut RendererState) {
    if state.route != Route::Settings {
        return;
    }
    let chosen = effective(state);
    state.settings.hero_catalogs = feeds(state)
        .into_iter()
        .map(|(id, name)| fluxa_ui::AmbientCatalogRow {
            selected: chosen.contains(&id),
            name,
        })
        .collect();
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    let Some(index) = node
        .checked_sub(fluxa_ui::NODE_SETTINGS_HERO_CATALOG_BASE)
        .map(|offset| offset as usize)
        .filter(|index| *index < fluxa_ui::HERO_CATALOG_MAX)
    else {
        return false;
    };
    let Some((id, _)) = feeds(state).into_iter().nth(index) else {
        return true;
    };
    let mut chosen = effective(state);
    match chosen.iter().position(|existing| *existing == id) {
        Some(position) => {
            chosen.remove(position);
        }
        None => chosen.push(id),
    }
    crate::stream_badges::set_setting(state, "heroFeedToggles", json!(chosen));
    sync_settings(state);
    true
}
