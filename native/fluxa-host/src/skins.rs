use crate::route::{NativeAction, Route};
use fluxa_skin::{FormFactor, Resolved, ResolvedAction, Skin};
use fluxa_ui::{HomeModel, UiFormFactorJson};
use serde_json::{Value, json};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

static ACTIVE: RwLock<Option<Arc<Skin>>> = RwLock::new(None);
static TARGETS: Mutex<Vec<(u64, ResolvedAction)>> = Mutex::new(Vec::new());
static FAILED: Mutex<Option<String>> = Mutex::new(None);

pub(crate) fn load(data_dir: &Path) {
    let (skins, problems) = fluxa_skin::load_skins(&data_dir.join("skins"));
    for problem in problems {
        crate::host_log(format!("skins/{problem}"));
    }
    let wanted = crate::preview::skin_override()
        .or_else(|| std::fs::read_to_string(data_dir.join("active-skin")).ok())
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty());
    let active = wanted.and_then(|id| {
        let folder = Path::new(&id);
        if cfg!(debug_assertions) && folder.join("skin.json").is_file() {
            return match fluxa_skin::load_skin(folder) {
                Ok(skin) => Some(Arc::new(skin)),
                Err(problems) => {
                    problems
                        .iter()
                        .for_each(|problem| crate::host_log(format!("skin {id}: {problem}")));
                    None
                }
            };
        }
        skins
            .into_iter()
            .find(|skin| skin.manifest.id == id)
            .map(Arc::new)
    });
    if let Ok(mut slot) = ACTIVE.write() {
        *slot = active;
    }
}

pub(crate) fn tree(route: Route, home: &HomeModel) -> Option<Vec<Resolved>> {
    let skin = ACTIVE.read().ok()?.clone()?;
    let screen = route.as_str();
    if !skin.replaces(screen) {
        return None;
    }
    let data = json!({ "home": serde_json::to_value(home).ok()? });
    let form = match home.form_factor {
        UiFormFactorJson::Mobile => FormFactor::Mobile,
        UiFormFactorJson::Desktop => FormFactor::Desktop,
        UiFormFactorJson::Tv => FormFactor::Tv,
    };
    match skin.resolve(screen, &data, form)? {
        Ok(tree) => Some(tree),
        Err(error) => {
            let message = format!("skin {}: {screen}: {error}", skin.manifest.id);
            if let Ok(mut last) = FAILED.lock()
                && last.as_deref() != Some(&message)
            {
                crate::host_log(&message);
                *last = Some(message);
            }
            None
        }
    }
}

pub(crate) fn remember(targets: Vec<(u64, ResolvedAction)>) {
    if let Ok(mut slot) = TARGETS.lock() {
        *slot = targets;
    }
}

pub(crate) fn action_for_node(node: u64) -> Option<NativeAction> {
    let targets = TARGETS.lock().ok()?;
    let (_, action) = targets.iter().find(|(id, _)| *id == node)?;
    native_action(action)
}

fn text(action: &ResolvedAction, key: &str) -> Option<String> {
    match action.args.get(key)? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

pub(crate) fn native_action(action: &ResolvedAction) -> Option<NativeAction> {
    let value = match action.name.as_str() {
        "play" => {
            json!({"type": "play", "id": text(action, "id")?, "itemType": text(action, "type")?})
        }
        "open_detail" => {
            json!({"type": "detail", "id": text(action, "id")?, "itemType": text(action, "type")?})
        }
        "navigate" => json!({"type": "navigate", "destination": text(action, "screen")?}),
        "back" => json!({"type": "back"}),
        "search" => json!({"type": "search", "query": text(action, "query")?}),
        "load_more" => json!({"type": "loadMore", "rowId": text(action, "row")?}),
        "set_setting" => {
            json!({"type": "settingsChange", "key": text(action, "key")?, "value": action.args.get("value")?})
        }
        "open_settings" => match text(action, "page").and_then(|page| page.parse::<usize>().ok()) {
            Some(index) => json!({"type": "settingsSection", "index": index}),
            None => json!({"type": "navigate", "destination": "settings"}),
        },
        "watchlist_toggle" => {
            json!({"type": "toggleWatchlist", "item": {"id": text(action, "id")?, "type": text(action, "type")?}})
        }
        _ => return None,
    };
    serde_json::from_value(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn action(name: &str, args: &[(&str, Value)]) -> ResolvedAction {
        ResolvedAction {
            name: name.to_owned(),
            args: args
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn open_detail_carries_id_and_type() {
        let native = native_action(&action(
            "open_detail",
            &[("id", json!("tt1")), ("type", json!("movie"))],
        ));
        assert!(
            matches!(native, Some(NativeAction::Detail { id, item_type, .. }) if id == "tt1" && item_type == "movie")
        );
    }

    #[test]
    fn navigate_rejects_unknown_screens() {
        assert!(native_action(&action("navigate", &[("screen", json!("library"))])).is_some());
        assert!(native_action(&action("navigate", &[("screen", json!("nowhere"))])).is_none());
    }

    #[test]
    fn missing_arguments_produce_no_action() {
        assert!(native_action(&action("play", &[("id", json!("tt1"))])).is_none());
    }

    #[test]
    fn setting_values_keep_their_type() {
        let native = native_action(&action(
            "set_setting",
            &[("key", json!("autoplay")), ("value", json!(true))],
        ));
        assert!(
            matches!(native, Some(NativeAction::SettingsChange { value, .. }) if value == json!(true))
        );
    }

    #[test]
    fn extension_actions_are_not_native() {
        assert!(native_action(&action("ext.example.sync", &[])).is_none());
    }
}
