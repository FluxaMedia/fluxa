use crate::route::{NativeAction, Route};
use fluxa_skin::{FormFactor, Resolved, ResolvedAction, Skin};
use fluxa_ui::{HomeLayout, HomeModel, SettingsModel, SkinFrame, SkinTarget, UiFormFactorJson};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

static DEFAULT: RwLock<Option<Arc<Skin>>> = RwLock::new(None);
static ACTIVE: RwLock<Option<Arc<Skin>>> = RwLock::new(None);
static TARGETS: Mutex<Vec<(u64, SkinTarget)>> = Mutex::new(Vec::new());
static CACHE: Mutex<Option<(u64, Arc<Vec<Resolved>>)>> = Mutex::new(None);
static FIELDS: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
static FAILED: Mutex<Option<String>> = Mutex::new(None);

const DEFAULT_FILES: &[(&str, &str)] = &[
    (
        "skin.json",
        include_str!("../../../shared/skins/default/skin.json"),
    ),
    (
        "includes.json",
        include_str!("../../../shared/skins/default/includes.json"),
    ),
    (
        "screens/home.json",
        include_str!("../../../shared/skins/default/screens/home.json"),
    ),
    (
        "style.css",
        include_str!("../../../shared/skins/default/style.css"),
    ),
];

fn load_default() {
    match fluxa_skin::load_embedded(DEFAULT_FILES) {
        Ok(skin) => {
            if let Ok(mut slot) = DEFAULT.write() {
                *slot = Some(Arc::new(skin));
            }
        }
        Err(problems) => problems
            .iter()
            .for_each(|problem| crate::host_log(format!("default skin: {problem}"))),
    }
}

const DEFAULT_READY: &[&str] = &[];

fn skin_for(screen: &str) -> Option<Arc<Skin>> {
    let active = ACTIVE.read().ok()?.clone();
    let default = DEFAULT
        .read()
        .ok()?
        .clone()
        .filter(|_| DEFAULT_READY.contains(&screen));
    [active, default]
        .into_iter()
        .flatten()
        .find(|skin| skin.replaces(screen))
}

fn localize(tree: &mut [Resolved], language: &str) {
    for node in tree {
        if let Some(key) = &node.i18n {
            node.text = Some(fluxa_ui::localized(key, language));
        }
        localize(&mut node.children, language);
    }
}

pub(crate) fn load(data_dir: &Path) {
    load_default();
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
    apply_theme(active.as_deref());
    if let Ok(mut slot) = ACTIVE.write() {
        *slot = active;
    }
}

fn apply_theme(active: Option<&Skin>) {
    let default = DEFAULT.read().ok().and_then(|slot| slot.clone());
    let css = [default.as_deref(), active]
        .into_iter()
        .flatten()
        .filter_map(Skin::theme)
        .collect::<Vec<_>>()
        .join("\n");
    if css.is_empty() {
        return;
    }
    if let Err(error) = fluxa_ui::reload_styles_with(&css) {
        crate::host_log(format!("skin theme: {error}"));
    }
}

const FINGERPRINT_CARDS: usize = 60;

fn fingerprint(
    screen: &str,
    skin: &Skin,
    home: &HomeModel,
    settings: &SettingsModel,
    fields: &HashMap<String, String>,
    nav: &Value,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (screen, &skin.manifest.id, &skin.manifest.version).hash(&mut hasher);
    (
        &home.title,
        &home.eyebrow,
        &home.description,
        &home.background_url,
        &home.logo_url,
        &home.item_id,
        &home.profile_name,
        &home.profile_avatar_url,
        home.is_loading,
        home.show_hero_section,
    )
        .hash(&mut hasher);
    std::mem::discriminant(&home.form_factor).hash(&mut hasher);
    for slide in &home.hero_slides {
        (&slide.item_id, &slide.title, &slide.background_url).hash(&mut hasher);
    }
    for row in &home.rows {
        (&row.id, &row.title, row.cards.len()).hash(&mut hasher);
        for card in row.cards.iter().take(FINGERPRINT_CARDS) {
            (
                &card.id,
                &card.title,
                &card.subtitle,
                &card.artwork_url,
                card.progress.to_bits(),
            )
                .hash(&mut hasher);
        }
    }
    settings.values.to_string().hash(&mut hasher);
    nav.to_string().hash(&mut hasher);
    let mut names: Vec<_> = fields.iter().collect();
    names.sort();
    names.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn tree(
    route: Route,
    home: &HomeModel,
    settings: &SettingsModel,
) -> Option<Arc<Vec<Resolved>>> {
    let screen = route.as_str();
    let skin = skin_for(screen)?;
    let fields = FIELDS
        .lock()
        .ok()
        .and_then(|fields| fields.clone())
        .unwrap_or_default();
    let nav = fluxa_ui::nav_data(screen);
    let key = fingerprint(screen, &skin, home, settings, &fields, &nav);
    if let Ok(cache) = CACHE.lock()
        && let Some((cached, tree)) = cache.as_ref()
        && *cached == key
    {
        return Some(tree.clone());
    }
    let data = json!({
        "home": serde_json::to_value(home).ok()?,
        "settings": settings.values,
        "fields": fields,
        "nav": nav,
    });
    let form = match home.form_factor {
        UiFormFactorJson::Mobile => FormFactor::Mobile,
        UiFormFactorJson::Desktop => FormFactor::Desktop,
        UiFormFactorJson::Tv => FormFactor::Tv,
    };
    let language = settings
        .values
        .get("language")
        .and_then(Value::as_str)
        .unwrap_or("en")
        .to_owned();
    match skin.resolve(screen, &data, form)? {
        Ok(mut tree) => {
            localize(&mut tree, &language);
            let tree = Arc::new(tree);
            if let Ok(mut cache) = CACHE.lock() {
                *cache = Some((key, tree.clone()));
            }
            Some(tree)
        }
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

pub(crate) fn remember(frame: SkinFrame) -> HomeLayout {
    if let Ok(mut slot) = TARGETS.lock() {
        *slot = frame.targets;
    }
    frame.layout
}

pub(crate) fn action_for_node(node: u64) -> Option<NativeAction> {
    let targets = TARGETS.lock().ok()?;
    let (_, target) = targets.iter().find(|(id, _)| *id == node)?;
    match target {
        SkinTarget::Action(action) => native_action(action),
        SkinTarget::Field(_) | SkinTarget::Control => None,
    }
}

fn field_name(node: u64) -> Option<String> {
    let targets = TARGETS.lock().ok()?;
    targets.iter().find_map(|(id, target)| match target {
        SkinTarget::Field(name) if *id == node => Some(name.clone()),
        _ => None,
    })
}

pub(crate) fn is_field(node: u64) -> bool {
    field_name(node).is_some()
}

pub(crate) fn field_text(node: u64) -> Option<String> {
    let name = field_name(node)?;
    let fields = FIELDS.lock().ok()?;
    Some(
        fields
            .as_ref()
            .and_then(|fields| fields.get(&name).cloned())
            .unwrap_or_default(),
    )
}

pub(crate) fn edit_field(node: u64, edit: impl FnOnce(&mut String)) -> bool {
    let Some(name) = field_name(node) else {
        return false;
    };
    let Ok(mut fields) = FIELDS.lock() else {
        return false;
    };
    edit(fields.get_or_insert_default().entry(name).or_default());
    true
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

    #[test]
    fn field_edits_land_in_the_named_field() {
        remember(SkinFrame {
            targets: vec![(40_000_001, SkinTarget::Field("query".to_owned()))],
            layout: HomeLayout::default(),
        });
        assert!(is_field(40_000_001));
        assert!(edit_field(40_000_001, |text| text.push_str("dune")));
        assert_eq!(field_text(40_000_001).as_deref(), Some("dune"));
        assert!(!edit_field(40_000_002, |_| {}));
    }

    #[test]
    fn bundled_default_skin_loads() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        assert_eq!(skin.manifest.id, "default");
    }
}
