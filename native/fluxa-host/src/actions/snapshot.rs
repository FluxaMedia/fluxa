use super::*;

pub(crate) fn take_native_actions(state: &mut RendererState) -> String {
    if state.pending_native_actions.is_empty() {
        return "[]".to_owned();
    }
    serde_json::to_string(&std::mem::take(&mut state.pending_native_actions))
        .unwrap_or_else(|_| "[]".to_owned())
}

pub(crate) fn profile_language(profile: &Value) -> String {
    profile
        .get("language")
        .and_then(Value::as_str)
        .filter(|language| !language.is_empty())
        .unwrap_or("en")
        .to_owned()
}

pub(crate) fn pull_session_snapshot(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let revision = session.revision();
    if state.session_revision == Some(revision) {
        return;
    }
    state.session_revision = Some(revision);
    let live = session.snapshot();
    let snapshot = crate::preview::fixture_snapshot(&live).unwrap_or(live);
    let unchanged = state
        .core_snapshot
        .as_deref()
        .is_some_and(|old| !projection_inputs_changed(old, &snapshot));
    if let Some(old) = state.core_snapshot.replace(snapshot) {
        retire(old);
    }
    if unchanged {
        return;
    }
    state.core_snapshot_revision = state.core_snapshot_revision.wrapping_add(1);
    sync_home_from_core_snapshot(state);
}

pub(super) const UNPROJECTED_SLICES: [&str; 5] =
    ["trailer", "lookup", "auth", "watchTogether", "_runtime"];

pub(super) fn projection_inputs_changed(old: &Value, new: &Value) -> bool {
    let (Some(old), Some(new)) = (old.as_object(), new.as_object()) else {
        return true;
    };
    let relevant = |key: &&String| !UNPROJECTED_SLICES.contains(&key.as_str());
    old.keys().filter(relevant).count() != new.keys().filter(relevant).count()
        || new
            .iter()
            .filter(|(key, _)| relevant(key))
            .any(|(key, value)| old.get(key) != Some(value))
}

pub(crate) fn discover_command(
    profile: &Value,
    content_type: &str,
    catalog_key: &str,
    extra_name: &str,
    extra_value: &str,
    load_catalog_filters: bool,
) -> Value {
    let mut extra = serde_json::Map::new();
    if !extra_name.is_empty() && !extra_value.is_empty() {
        extra.insert(extra_name.to_owned(), json!(extra_value));
    }
    json!({
        "type": "discoverRequested",
        "loadCatalogFilters": load_catalog_filters,
        "contentType": content_type,
        "filters": {
            "catalogKey": (!catalog_key.is_empty()).then_some(catalog_key),
            "extra": extra,
        },
        "profile": profile,
        "language": profile_language(profile),
    })
}

pub(crate) fn folder_commands(
    snapshot: Option<&Value>,
    folder_id: &str,
    profile: &Value,
) -> Vec<Value> {
    let mut commands = vec![json!({
        "type": "navigationRequested",
        "route": Route::Folder.as_str(),
        "params": {"folderId": folder_id},
    })];
    let categories = snapshot
        .and_then(|snapshot| snapshot.pointer("/home/categories"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for category in categories {
        let is_source = category.get("type").and_then(Value::as_str)
            == Some("collection_folder_source")
            && category.get("folderId").and_then(Value::as_str) == Some(folder_id);
        let loaded = category
            .get("items")
            .and_then(Value::as_array)
            .is_some_and(|items| !items.is_empty());
        if !is_source || loaded {
            continue;
        }
        commands.push(json!({
            "type": "catalogPageRequested",
            "categoryId": category.get("id"),
            "transportUrl": category.get("transportUrl"),
            "contentType": category.get("contentType"),
            "catalogId": category.get("catalogId"),
            "skip": 0,
            "genre": category.get("genre"),
            "remoteSource": category.get("remoteSource"),
            "profile": profile,
        }));
    }
    commands
}

pub(crate) fn navigation(route: Route) -> Value {
    json!({"type": "navigationRequested", "route": route.as_str(), "params": {}})
}
