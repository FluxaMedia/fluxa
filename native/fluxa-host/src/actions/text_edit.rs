use super::*;

pub(crate) fn edit_text(
    state: &mut RendererState,
    node: u64,
    edit: impl FnOnce(&mut String),
) -> bool {
    let route = state.route;
    let text = match node {
        fluxa_ui::NODE_LIBRARY_SEARCH if route == Route::Library => &mut state.library_query,
        fluxa_ui::NODE_DISCOVER_SEARCH if route == Route::Discover => &mut state.discover.query,
        fluxa_ui::NODE_SIDEBAR_SEARCH => &mut state.discover.query,
        fluxa_ui::NODE_SETTINGS_SEARCH if route == Route::Settings => &mut state.settings.search,
        fluxa_ui::NODE_SETTINGS_ADDON_URL if route == Route::Settings => {
            &mut state.settings.addon_url
        }
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL if route == Route::Settings => {
            &mut state.settings.plugin_url
        }
        fluxa_ui::NODE_SETTINGS_STREAMKIT_ADDON_URL if route == Route::Settings => {
            &mut state.settings.streamkit_addon_url
        }
        fluxa_ui::NODE_SETTINGS_WATCH_CODE if route == Route::Settings => {
            &mut state.settings.watch_code
        }
        fluxa_ui::NODE_ADDON_EDIT_NAME if route == Route::Settings => {
            &mut state.settings.addon_edit_name
        }
        fluxa_ui::NODE_REPORT_TITLE if route == Route::Settings => &mut state.settings.report_title,
        fluxa_ui::NODE_REPORT_BODY if route == Route::Settings => &mut state.settings.report_body,
        _ if route == Route::Settings && fluxa_ui::server_input(node).is_some() => {
            &mut state.settings.server_fields[fluxa_ui::server_input(node).unwrap_or_default()]
        }
        _ if route == Route::Settings && fluxa_ui::collection_input(node).is_some() => {
            &mut state.settings.collection_fields
                [fluxa_ui::collection_input(node).unwrap_or_default()]
        }
        _ if route == Route::Settings && fluxa_ui::badge_input(node).is_some() => {
            &mut state.settings.badge_fields[fluxa_ui::badge_input(node).unwrap_or_default()]
        }
        _ if route == Route::Settings && fluxa_ui::plugin_config_input(node).is_some() => {
            &mut state.settings.plugin_config_fields
                [fluxa_ui::plugin_config_input(node).unwrap_or_default()]
        }
        _ if route == Route::Settings && fluxa_ui::poster_field(node).is_some() => {
            &mut state.settings.poster_fields[fluxa_ui::poster_field(node).unwrap_or_default()]
        }
        _ => return false,
    };
    let before = text.clone();
    edit(text);
    if *text == before {
        return true;
    }
    match node {
        fluxa_ui::NODE_LIBRARY_SEARCH => refresh_library_view(state),
        fluxa_ui::NODE_DISCOVER_SEARCH => schedule_discover_search(state),
        fluxa_ui::NODE_SIDEBAR_SEARCH => search_from_sidebar(state),
        fluxa_ui::NODE_SETTINGS_WATCH_CODE => {
            let code = &mut state.settings.watch_code;
            *code = code
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .take(6)
                .collect::<String>()
                .to_uppercase();
        }
        _ => {}
    }
    true
}

pub(crate) fn focused_text(state: &RendererState) -> Option<String> {
    node_text(state, state.ui.focused()?)
}

pub(crate) fn node_text(state: &RendererState, node: u64) -> Option<String> {
    let route = state.route;
    Some(match node {
        fluxa_ui::NODE_LIBRARY_SEARCH if route == Route::Library => state.library_query.clone(),
        fluxa_ui::NODE_DISCOVER_SEARCH if route == Route::Discover => state.discover.query.clone(),
        fluxa_ui::NODE_SIDEBAR_SEARCH => state.discover.query.clone(),
        fluxa_ui::NODE_SETTINGS_SEARCH if route == Route::Settings => state.settings.search.clone(),
        fluxa_ui::NODE_SETTINGS_ADDON_URL if route == Route::Settings => {
            state.settings.addon_url.clone()
        }
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL if route == Route::Settings => {
            state.settings.plugin_url.clone()
        }
        fluxa_ui::NODE_SETTINGS_STREAMKIT_ADDON_URL if route == Route::Settings => {
            state.settings.streamkit_addon_url.clone()
        }
        fluxa_ui::NODE_SETTINGS_WATCH_CODE if route == Route::Settings => {
            state.settings.watch_code.clone()
        }
        fluxa_ui::NODE_ADDON_EDIT_NAME if route == Route::Settings => {
            state.settings.addon_edit_name.clone()
        }
        fluxa_ui::NODE_REPORT_TITLE if route == Route::Settings => {
            state.settings.report_title.clone()
        }
        fluxa_ui::NODE_REPORT_BODY if route == Route::Settings => {
            state.settings.report_body.clone()
        }
        _ if route == Route::Settings && fluxa_ui::server_input(node).is_some() => {
            state.settings.server_fields[fluxa_ui::server_input(node)?].clone()
        }
        _ if route == Route::Settings && fluxa_ui::collection_input(node).is_some() => {
            state.settings.collection_fields[fluxa_ui::collection_input(node)?].clone()
        }
        _ if route == Route::Settings && fluxa_ui::badge_input(node).is_some() => {
            state.settings.badge_fields[fluxa_ui::badge_input(node)?].clone()
        }
        _ if route == Route::Settings && fluxa_ui::plugin_config_input(node).is_some() => {
            state.settings.plugin_config_fields[fluxa_ui::plugin_config_input(node)?].clone()
        }
        _ if route == Route::Settings => {
            state.settings.poster_fields[fluxa_ui::poster_field(node)?].clone()
        }
        _ => return None,
    })
}

pub(super) fn save_left_fields(state: &mut RendererState) {
    let focused = state.ui.focused();
    for (index, field) in fluxa_ui::POSTER_FIELDS.iter().enumerate() {
        let value = state.settings.poster_fields[index].trim().to_owned();
        match field.kind {
            fluxa_ui::FieldKind::Action => continue,
            fluxa_ui::FieldKind::Setup => {
                let current = fluxa_ui::setup_export(field.key, &state.settings.values);
                if focused != Some(field.input) && !value.is_empty() && value != current {
                    import_setup(state, field.key, &value);
                }
                continue;
            }
            fluxa_ui::FieldKind::Text => {}
        }
        if focused == Some(field.input)
            || state.settings.str_value(field.key).unwrap_or("") == value
        {
            continue;
        }
        if let Some(values) = state.settings.values.as_object_mut() {
            values.insert(field.key.to_owned(), Value::String(value.clone()));
        }
        state
            .pending_native_actions
            .push(NativeAction::CoreCommand {
                command: json!({"type":"settingsChanged", "key":field.key, "value":value}),
            });
    }
}

pub(super) fn import_setup(state: &mut RendererState, field_key: &str, text: &str) {
    let Ok(Value::Object(entries)) = serde_json::from_str::<Value>(text) else {
        return;
    };
    for (pref, value) in entries {
        if !fluxa_ui::setup_accepts(field_key, &pref) || value.is_array() || value.is_object() {
            continue;
        }
        if let Some(values) = state.settings.values.as_object_mut() {
            values.insert(pref.clone(), value.clone());
        }
        state
            .pending_native_actions
            .push(NativeAction::CoreCommand {
                command: json!({"type":"settingsChanged", "key":pref, "value":value}),
            });
    }
}
