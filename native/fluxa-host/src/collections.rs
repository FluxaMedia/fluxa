use std::sync::mpsc::Receiver;

use fluxa_ui::{
    CollectionEdit, CollectionListRow, FolderEdit, NODE_SETTINGS_COLLECTION_ADD,
    NODE_SETTINGS_COLLECTION_CLOSE, NODE_SETTINGS_COLLECTION_DELETE_BASE,
    NODE_SETTINGS_COLLECTION_EDIT_BASE, NODE_SETTINGS_COLLECTION_EXPORT,
    NODE_SETTINGS_COLLECTION_GLOW, NODE_SETTINGS_COLLECTION_IMPORT, NODE_SETTINGS_COLLECTION_PIN,
    NODE_SETTINGS_COLLECTION_RENAME, NODE_SETTINGS_COLLECTION_SHOW_ALL,
    NODE_SETTINGS_COLLECTION_SHOW_HOME, NODE_SETTINGS_COLLECTION_VIEW_MODE,
    NODE_SETTINGS_FOLDER_CANCEL, NODE_SETTINGS_FOLDER_DELETE_BASE, NODE_SETTINGS_FOLDER_EDIT_BASE,
    NODE_SETTINGS_FOLDER_HIDE_TITLE, NODE_SETTINGS_FOLDER_NEW, NODE_SETTINGS_FOLDER_SAVE,
    NODE_SETTINGS_FOLDER_SHAPE, NODE_SETTINGS_SOURCE_ADD, NODE_SETTINGS_SOURCE_PROVIDER,
    NODE_SETTINGS_SOURCE_REMOVE_BASE, localized,
};
use serde_json::{Value, json};

use crate::{RendererState, Route};

const PROVIDERS: [&str; 3] = ["addon", "tmdb", "trakt"];
const SHAPES: [&str; 3] = ["poster", "wide", "square"];

#[derive(Default)]
pub(crate) struct State {
    list: Option<Vec<Value>>,
    loading: Option<Receiver<Value>>,
    saving: Option<Receiver<bool>>,
    importing: Option<Receiver<Option<String>>>,
    dirty: bool,
    reload: bool,
    edit: Option<usize>,
    draft: Option<Draft>,
}

struct Draft {
    folder: Value,
    provider: usize,
}

fn open(state: &RendererState) -> bool {
    state.route == Route::Settings
        && fluxa_ui::SETTINGS_SECTIONS
            .get(state.settings.active_section)
            .is_some_and(|section| section.title == "Collections")
}

fn say(state: &mut RendererState, key: &str) {
    let language = state.settings.language().to_owned();
    state.settings.collection_status = Some(localized(key, &language));
}

fn clock() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default()
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn flag(value: &Value, key: &str, default: bool) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn folders(collection: &Value) -> Vec<Value> {
    collection
        .get("folders")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn source_count(folder: &Value) -> usize {
    let count = |key: &str| {
        folder
            .get(key)
            .and_then(Value::as_array)
            .map_or(0, Vec::len)
    };
    count("sources").max(count("catalogSources"))
}

fn source_label(source: &Value) -> String {
    match text(source, "provider").as_str() {
        "tmdb" => format!(
            "TMDB {} {}",
            text(source, "tmdbSourceType"),
            source
                .get("tmdbId")
                .map(|id| id.to_string())
                .unwrap_or_default()
        ),
        "trakt" => format!(
            "Trakt {}",
            source
                .get("traktListId")
                .map(|id| id.to_string())
                .unwrap_or_default()
        ),
        _ => format!("{} · {}", text(source, "catalogId"), text(source, "type")),
    }
}

fn folder_sources(folder: &Value) -> Vec<Value> {
    let sources = folder
        .get("sources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !sources.is_empty() {
        return sources;
    }
    folder
        .get("catalogSources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|source| {
            json!({
                "provider": "addon",
                "addonId": source.get("addonId"),
                "type": source.get("type").cloned().unwrap_or(json!("movie")),
                "catalogId": source.get("catalogId"),
                "genre": source.get("genre"),
            })
        })
        .collect()
}

fn current(state: &RendererState) -> Vec<Value> {
    state.collections.list.clone().unwrap_or_default()
}

fn mutate(state: &mut RendererState, command: Value) {
    let args = json!({"collections": current(state), "command": command});
    let next = fluxa_core::ffi::call("collectionMutationPlan", &args)
        .ok()
        .and_then(|plan| plan.get("collections").and_then(Value::as_array).cloned());
    match next {
        Some(list) => commit(state, list),
        None => say(state, "collections.status_invalid"),
    }
}

fn commit(state: &mut RendererState, list: Vec<Value>) {
    state.collections.list = Some(list);
    state.collections.dirty = true;
}

fn edit_collection(state: &mut RendererState, change: impl FnOnce(&mut Value)) {
    let Some(index) = state.collections.edit else {
        return;
    };
    let Some(mut collection) = current(state).get(index).cloned() else {
        return;
    };
    change(&mut collection);
    mutate(state, json!({"type": "upsert", "collection": collection}));
}

fn field(state: &RendererState, index: usize) -> String {
    state.settings.collection_fields[index].trim().to_owned()
}

fn clear_folder_form(state: &mut RendererState) {
    for text in state.settings.collection_fields.iter_mut().skip(3) {
        text.clear();
    }
}

fn open_folder(state: &mut RendererState, index: Option<usize>) {
    let Some(collection) = state
        .collections
        .edit
        .and_then(|edit| current(state).get(edit).cloned())
    else {
        return;
    };
    let mut folder = index
        .and_then(|index| folders(&collection).get(index).cloned())
        .unwrap_or_else(|| {
            json!({
                "id": format!("folder_{}", clock()),
                "shape": "poster",
                "hideTitle": false,
                "focusGifEnabled": true,
            })
        });
    let sources = folder_sources(&folder);
    folder["sources"] = Value::Array(sources);
    if let Some(fields) = folder.as_object_mut() {
        fields.remove("catalogSources");
    }
    let fields = &mut state.settings.collection_fields;
    fields[3] = text(&folder, "title");
    fields[4] = text(&folder, "coverImageUrl");
    fields[5] = text(&folder, "coverEmoji");
    fields[6] = text(&folder, "focusGifUrl");
    for text in fields.iter_mut().skip(7) {
        text.clear();
    }
    state.collections.draft = Some(Draft {
        folder,
        provider: 0,
    });
}

fn save_folder(state: &mut RendererState) {
    let Some(draft) = state.collections.draft.take() else {
        return;
    };
    let Some(collection_id) = state.collections.edit.and_then(|edit| {
        current(state)
            .get(edit)
            .map(|collection| text(collection, "id"))
    }) else {
        return;
    };
    let title = field(state, 3);
    if title.is_empty() {
        state.collections.draft = Some(draft);
        say(state, "collections.status_invalid");
        return;
    }
    let optional = |text: String| (!text.is_empty()).then_some(text);
    let cover = optional(field(state, 4));
    let gif = optional(field(state, 6));
    let mut folder = draft.folder;
    let shape = text(&folder, "shape");
    folder["title"] = json!(title);
    folder["coverImageUrl"] = json!(cover);
    folder["imageUrl"] = json!(cover);
    folder["coverEmoji"] = json!(optional(field(state, 5)));
    folder["focusGifUrl"] = json!(gif);
    folder["tileShape"] = json!(match shape.as_str() {
        "wide" => "LANDSCAPE",
        "square" => "SQUARE",
        _ => "POSTER",
    });
    mutate(
        state,
        json!({"type": "saveFolder", "collectionId": collection_id, "folder": folder}),
    );
    clear_folder_form(state);
}

fn add_source(state: &mut RendererState) {
    let Some(provider) = state
        .collections
        .draft
        .as_ref()
        .map(|draft| PROVIDERS[draft.provider])
    else {
        return;
    };
    let (first, second, third) = (field(state, 7), field(state, 8), field(state, 9));
    let media = |value: &str| match value.to_ascii_lowercase().as_str() {
        "tv" | "series" | "show" => "TV",
        _ => "MOVIE",
    };
    let source = match provider {
        "tmdb" => {
            let kind = if third.is_empty() {
                "LIST".to_owned()
            } else {
                third.to_ascii_uppercase()
            };
            let id = first.parse::<i64>().ok();
            if id.is_none() && kind != "DISCOVER" {
                return say(state, "collections.status_invalid");
            }
            let mut source = json!({
                "provider": "tmdb",
                "mediaType": media(&second),
                "tmdbSourceType": kind,
                "tmdbId": id,
            });
            source["title"] = json!(source_label(&source));
            source
        }
        "trakt" => {
            let Ok(id) = first.parse::<i64>() else {
                return say(state, "collections.status_invalid");
            };
            let mut source = json!({
                "provider": "trakt",
                "mediaType": media(&second),
                "traktListId": id,
                "sortBy": "rank",
                "sortHow": "asc",
            });
            source["title"] = json!(source_label(&source));
            source
        }
        _ => {
            if first.is_empty() || third.is_empty() {
                return say(state, "collections.status_invalid");
            }
            json!({
                "provider": "addon",
                "addonId": first,
                "type": if second.is_empty() { "movie".to_owned() } else { second.to_ascii_lowercase() },
                "catalogId": third,
            })
        }
    };
    if let Some(draft) = state.collections.draft.as_mut()
        && let Some(sources) = draft.folder["sources"].as_array_mut()
    {
        sources.push(source);
    }
    for text in state.settings.collection_fields.iter_mut().skip(7) {
        text.clear();
    }
    state.settings.collection_status = None;
}

fn import_text(state: &mut RendererState, raw: &str) {
    let Ok(parsed) = serde_json::from_str::<Value>(raw) else {
        return say(state, "collections.status_unreadable");
    };
    let imported = fluxa_core::ffi::call("importCollections", &parsed)
        .ok()
        .and_then(|value| value.as_array().cloned())
        .filter(|list| !list.is_empty());
    let Some(imported) = imported else {
        return say(state, "collections.status_unreadable");
    };
    let ids: Vec<String> = imported.iter().map(|item| text(item, "id")).collect();
    let mut list: Vec<Value> = current(state)
        .into_iter()
        .filter(|item| !ids.contains(&text(item, "id")))
        .collect();
    list.extend(imported);
    commit(state, list);
    state.settings.collection_fields[0].clear();
    say(state, "collections.status_imported");
}

fn start_import(state: &mut RendererState) {
    let input = field(state, 0);
    if input.starts_with('{') || input.starts_with('[') {
        return import_text(state, &input);
    }
    if !(input.starts_with("http://") || input.starts_with("https://")) {
        return say(state, "collections.status_unreadable");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        state.collections.importing = Some(fluxa_effects::fetch_text(&input));
        say(state, "collections.status_loading");
    }
    #[cfg(target_arch = "wasm32")]
    say(state, "collections.status_paste_only");
}

fn export(state: &mut RendererState) {
    let exported = fluxa_core::ffi::call("exportCollections", &Value::Array(current(state))).ok();
    match exported {
        Some(value) => {
            state.settings.collection_fields[0] = value.to_string();
            say(state, "collections.status_exported");
        }
        None => say(state, "collections.status_invalid"),
    }
}

fn add_collection(state: &mut RendererState) {
    let title = field(state, 1);
    if title.is_empty() {
        return say(state, "collections.status_invalid");
    }
    let collection = json!({
        "id": format!("collection_{}", clock()),
        "title": title,
        "folders": [],
        "showOnHome": true,
        "viewMode": "TABBED_GRID",
        "showAllTab": true,
        "pinToTop": false,
        "focusGlowEnabled": true,
    });
    mutate(state, json!({"type": "create", "collection": collection}));
    state.settings.collection_fields[1].clear();
    state.settings.collection_status = None;
}

fn open_collection(state: &mut RendererState, index: usize) {
    let Some(collection) = current(state).get(index).cloned() else {
        return;
    };
    state.settings.collection_fields[2] = text(&collection, "title");
    state.collections.edit = Some(index);
    state.collections.draft = None;
    clear_folder_form(state);
}

fn toggle_flag(state: &mut RendererState, key: &'static str, default: bool) {
    edit_collection(state, |collection| {
        collection[key] = json!(!flag(collection, key, default));
    });
}

fn start_save(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let list = Value::Array(current(state));
    state.collections.saving = Some(session.executor().save_collections(list));
    state.collections.dirty = false;
}

pub(crate) fn poll(state: &mut RendererState) {
    if let Some(receiver) = state.collections.saving.as_ref()
        && let Ok(saved) = receiver.try_recv()
    {
        state.collections.saving = None;
        state.collections.reload = saved;
        if !saved {
            say(state, "collections.status_save_failed");
        }
    }
    if state.collections.saving.is_none() && state.collections.dirty {
        start_save(state);
    }
    if state.collections.saving.is_none()
        && !state.collections.dirty
        && std::mem::take(&mut state.collections.reload)
        && let Some(session) = state.session.as_ref()
    {
        let profile = session.active_profile();
        let command = json!({
            "type": "homeLoadRequested",
            "profile": profile,
            "language": crate::profile_language(&profile),
            "force": true,
        });
        if let Err(error) = session.dispatch(command) {
            crate::host_log(format!("collections home reload failed: {error}"));
        }
    }
    if let Some(receiver) = state.collections.importing.as_ref()
        && let Ok(body) = receiver.try_recv()
    {
        state.collections.importing = None;
        match body {
            Some(body) => import_text(state, &body),
            None => say(state, "collections.status_unreadable"),
        }
    }
    if !open(state) {
        return;
    }
    if state.collections.list.is_none() && state.collections.loading.is_none() {
        if let Some(session) = state.session.as_ref() {
            state.collections.loading = Some(session.executor().read_collections());
        }
    }
    if let Some(receiver) = state.collections.loading.as_ref()
        && let Ok(list) = receiver.try_recv()
    {
        state.collections.loading = None;
        if state.collections.list.is_none() {
            state.collections.list = Some(list.as_array().cloned().unwrap_or_default());
        }
    }
    project(state);
}

fn project(state: &mut RendererState) {
    let list = current(state);
    state.settings.collections = list
        .iter()
        .map(|collection| CollectionListRow {
            title: text(collection, "title"),
            folders: folders(collection).len(),
        })
        .collect();
    let selected = state
        .collections
        .edit
        .and_then(|index| list.get(index).map(|collection| (index, collection)));
    let Some((_, collection)) = selected else {
        state.collections.edit = None;
        state.settings.collection_edit = None;
        return;
    };
    let folder = state.collections.draft.as_ref().map(|draft| FolderEdit {
        shape: text(&draft.folder, "shape"),
        hide_title: flag(&draft.folder, "hideTitle", false),
        provider: PROVIDERS[draft.provider].to_owned(),
        sources: draft.folder["sources"]
            .as_array()
            .map(|sources| sources.iter().map(source_label).collect())
            .unwrap_or_default(),
    });
    state.settings.collection_edit = Some(CollectionEdit {
        view_mode: collection
            .get("viewMode")
            .and_then(Value::as_str)
            .unwrap_or("TABBED_GRID")
            .to_owned(),
        show_all_tab: flag(collection, "showAllTab", true),
        pin_to_top: flag(collection, "pinToTop", false),
        focus_glow: flag(collection, "focusGlowEnabled", true),
        show_on_home: flag(collection, "showOnHome", true),
        folders: folders(collection)
            .iter()
            .map(|folder| CollectionListRow {
                title: text(folder, "title"),
                folders: source_count(folder),
            })
            .collect(),
        folder,
    });
}

fn cycle(options: &[&str], current: &str) -> String {
    let position = options
        .iter()
        .position(|option| option.eq_ignore_ascii_case(current))
        .unwrap_or(options.len() - 1);
    options[(position + 1) % options.len()].to_owned()
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    if state.route != Route::Settings || !open(state) {
        return false;
    }
    let slot = |base: u64, count: usize| {
        let offset = node.checked_sub(base)? as usize;
        (offset < count).then_some(offset)
    };
    let collections = state.settings.collections.len();
    let folder_count = state
        .settings
        .collection_edit
        .as_ref()
        .map_or(0, |edit| edit.folders.len());
    let sources = state
        .settings
        .collection_edit
        .as_ref()
        .and_then(|edit| edit.folder.as_ref())
        .map_or(0, |folder| folder.sources.len());
    if node == NODE_SETTINGS_COLLECTION_IMPORT {
        start_import(state);
    } else if node == NODE_SETTINGS_COLLECTION_EXPORT {
        export(state);
    } else if node == NODE_SETTINGS_COLLECTION_ADD {
        add_collection(state);
    } else if node == NODE_SETTINGS_COLLECTION_RENAME {
        let title = field(state, 2);
        if title.is_empty() {
            say(state, "collections.status_invalid");
        } else {
            edit_collection(state, |collection| collection["title"] = json!(title));
        }
    } else if node == NODE_SETTINGS_COLLECTION_VIEW_MODE {
        edit_collection(state, |collection| {
            let next = cycle(
                fluxa_ui::collection_view_modes(),
                &text(collection, "viewMode"),
            );
            collection["viewMode"] = json!(next);
        });
    } else if node == NODE_SETTINGS_COLLECTION_SHOW_ALL {
        toggle_flag(state, "showAllTab", true);
    } else if node == NODE_SETTINGS_COLLECTION_PIN {
        toggle_flag(state, "pinToTop", false);
    } else if node == NODE_SETTINGS_COLLECTION_GLOW {
        toggle_flag(state, "focusGlowEnabled", true);
    } else if node == NODE_SETTINGS_COLLECTION_SHOW_HOME {
        toggle_flag(state, "showOnHome", true);
    } else if node == NODE_SETTINGS_COLLECTION_CLOSE {
        state.collections.edit = None;
        state.collections.draft = None;
    } else if node == NODE_SETTINGS_FOLDER_NEW {
        open_folder(state, None);
    } else if node == NODE_SETTINGS_FOLDER_SAVE {
        save_folder(state);
    } else if node == NODE_SETTINGS_FOLDER_CANCEL {
        state.collections.draft = None;
        clear_folder_form(state);
    } else if node == NODE_SETTINGS_FOLDER_SHAPE {
        if let Some(draft) = state.collections.draft.as_mut() {
            let next = cycle(&SHAPES, &text(&draft.folder, "shape"));
            draft.folder["shape"] = json!(next);
        }
    } else if node == NODE_SETTINGS_FOLDER_HIDE_TITLE {
        if let Some(draft) = state.collections.draft.as_mut() {
            let hidden = flag(&draft.folder, "hideTitle", false);
            draft.folder["hideTitle"] = json!(!hidden);
        }
    } else if node == NODE_SETTINGS_SOURCE_PROVIDER {
        if let Some(draft) = state.collections.draft.as_mut() {
            draft.provider = (draft.provider + 1) % PROVIDERS.len();
        }
    } else if node == NODE_SETTINGS_SOURCE_ADD {
        add_source(state);
    } else if let Some(i) = slot(NODE_SETTINGS_COLLECTION_EDIT_BASE, collections) {
        open_collection(state, i);
    } else if let Some(i) = slot(NODE_SETTINGS_COLLECTION_DELETE_BASE, collections) {
        if let Some(id) = current(state)
            .get(i)
            .map(|collection| text(collection, "id"))
        {
            state.collections.edit = None;
            state.collections.draft = None;
            mutate(state, json!({"type": "delete", "id": id}));
        }
    } else if let Some(i) = slot(NODE_SETTINGS_FOLDER_EDIT_BASE, folder_count) {
        open_folder(state, Some(i));
    } else if let Some(i) = slot(NODE_SETTINGS_FOLDER_DELETE_BASE, folder_count) {
        let target = state.collections.edit.and_then(|edit| {
            let collection = current(state).get(edit).cloned()?;
            let folder = folders(&collection)
                .get(i)
                .map(|folder| text(folder, "id"))?;
            Some((text(&collection, "id"), folder))
        });
        if let Some((collection_id, folder_id)) = target {
            state.collections.draft = None;
            mutate(
                state,
                json!({"type": "deleteFolder", "collectionId": collection_id, "folderId": folder_id}),
            );
        }
    } else if let Some(i) = slot(NODE_SETTINGS_SOURCE_REMOVE_BASE, sources) {
        if let Some(draft) = state.collections.draft.as_mut()
            && let Some(list) = draft.folder["sources"].as_array_mut()
        {
            list.remove(i);
        }
    } else {
        return false;
    }
    true
}
