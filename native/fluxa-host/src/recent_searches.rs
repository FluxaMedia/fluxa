use super::*;
use fluxa_core::recent_searches as policy;

const SAVE_KEY: &str = "saveRecentSearches";
const HISTORY_KEY: &str = "recentSearches";

fn history(state: &RendererState) -> Vec<String> {
    state
        .settings
        .values
        .get(HISTORY_KEY)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn store(state: &mut RendererState, entries: &[String]) {
    let value = json!(entries);
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert(HISTORY_KEY.to_owned(), value.clone());
    }
    state
        .pending_native_actions
        .push(NativeAction::SettingsChange {
            key: HISTORY_KEY.to_owned(),
            value,
        });
}

pub(super) fn refresh(state: &mut RendererState) {
    let entries = history(state);
    if !state.settings.bool_value(SAVE_KEY) {
        if !entries.is_empty() {
            store(state, &[]);
        }
        state.discover.recent_searches.clear();
        return;
    }
    state.discover.recent_searches = policy::matching(&entries, &state.discover.query)
        .into_iter()
        .map(str::to_owned)
        .collect();
}

pub(super) fn save_left_query(state: &mut RendererState) {
    if state.route != Route::Discover || !state.settings.bool_value(SAVE_KEY) {
        return;
    }
    let searching = state.ui.focused().is_some_and(|node| {
        node == fluxa_ui::NODE_DISCOVER_SEARCH || fluxa_ui::discover_recent(node).is_some()
    });
    if searching {
        return;
    }
    if let Some(next) = policy::record(&history(state), &state.discover.query) {
        store(state, &next);
        refresh(state);
    }
}
