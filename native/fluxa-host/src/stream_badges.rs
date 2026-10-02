use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

use egui::Color32;
use fluxa_core::player::stream_badges::{self as engine, BadgeKind, Filter, Options, Rules, Theme};
use fluxa_ui::{
    BadgeCustomRow, BadgePackRow, NODE_SETTINGS_BADGE_CLEAR, NODE_SETTINGS_BADGE_EDIT_BASE,
    NODE_SETTINGS_BADGE_IMPORT, NODE_SETTINGS_BADGE_PACK_REMOVE_BASE,
    NODE_SETTINGS_BADGE_PACK_TOGGLE_BASE, NODE_SETTINGS_BADGE_REMOVE_BASE,
    NODE_SETTINGS_BADGE_SAVE, NODE_SETTINGS_BADGE_STYLE, NODE_SETTINGS_BADGE_TOGGLE_BASE,
    SourceBadge, localized,
};
use serde_json::Value;

use crate::{NativeAction, RendererState};

const RULES_KEY: &str = "streamBadgeRules";

#[derive(Default)]
pub(crate) struct State {
    import: Option<(String, Receiver<Option<String>>)>,
    editing: Option<String>,
}

struct Config {
    enabled: bool,
    top: bool,
    options: Options,
    raw: Value,
    rules: Rules,
    cache: HashMap<(String, Option<i64>), Vec<SourceBadge>>,
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

fn color(value: u32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    )
}

fn convert(badge: engine::StreamBadge) -> SourceBadge {
    SourceBadge {
        fill: badge.look.fill.map(color),
        text: badge.look.text.map(color),
        border: badge.look.border.map(color),
        outline: badge.look.outline,
        image: badge.image_url,
        label: badge.label,
    }
}

pub(crate) fn badges_for(text: &str, video_size: Option<i64>) -> (Vec<SourceBadge>, bool) {
    let mut guard = CONFIG
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(config) = guard.as_mut().filter(|config| config.enabled) else {
        return (Vec::new(), false);
    };
    let top = config.top;
    let key = (text.to_owned(), video_size);
    if let Some(found) = config.cache.get(&key) {
        return (found.clone(), top);
    }
    let found: Vec<SourceBadge> = engine::parse(text, video_size, config.options, &config.rules)
        .into_iter()
        .map(convert)
        .collect();
    if config.cache.len() > 2000 {
        config.cache.clear();
    }
    config.cache.insert(key, found.clone());
    (found, top)
}

fn current_rules(state: &RendererState) -> Rules {
    Rules::from_value(state.settings.values.get(RULES_KEY).unwrap_or(&Value::Null))
}

pub(crate) fn poll(state: &mut RendererState) {
    receive_import(state);
    sync_config(state);
    if state.route == crate::Route::Settings {
        state.settings.badge_preview = preview(state);
    }
}

fn sync_config(state: &mut RendererState) {
    let settings = &state.settings;
    let raw = settings.values.get(RULES_KEY).unwrap_or(&Value::Null);
    let options = Options {
        built_in: settings.bool_value("streamBadgeBuiltIn"),
        file_size: settings.bool_value("streamBadgeFileSize"),
        theme: Theme::from_name(settings.str_value("streamBadgeTheme").unwrap_or_default()),
    };
    let enabled = settings.bool_value("streamBadgesEnabled");
    let top = settings.str_value("streamBadgePlacement") == Some("top");
    let mut guard = CONFIG
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(config) = guard.as_mut()
        && config.raw == *raw
    {
        if config.enabled != enabled || config.top != top || config.options != options {
            config.enabled = enabled;
            config.top = top;
            config.options = options;
            config.cache.clear();
        }
        return;
    }
    let rules = Rules::from_value(raw);
    let raw = raw.clone();
    *guard = Some(Config {
        enabled,
        top,
        options,
        raw,
        rules: rules.clone(),
        cache: HashMap::new(),
    });
    drop(guard);
    state.settings.badge_packs = rules
        .packs
        .iter()
        .map(|pack| BadgePackRow {
            name: pack.name.clone(),
            count: pack.filters.len(),
            active: pack.is_active,
        })
        .collect();
    state.settings.badge_custom = rules
        .custom
        .iter()
        .map(|filter| BadgeCustomRow {
            badge: convert(engine::StreamBadge {
                kind: BadgeKind::Custom,
                label: filter.name.clone(),
                look: filter.look(),
                image_url: filter
                    .image_url
                    .clone()
                    .filter(|url| !url.trim().is_empty()),
            }),
            enabled: filter.is_enabled,
        })
        .collect();
}

fn preview(state: &RendererState) -> SourceBadge {
    let fields = &state.settings.badge_fields;
    let parse = |text: &str| engine::parse_color(text).map(color);
    let name = fields[1].trim();
    SourceBadge {
        label: if name.is_empty() {
            "Badge".to_owned()
        } else {
            name.to_owned()
        },
        fill: parse(&fields[3]),
        text: parse(&fields[4]),
        border: parse(&fields[5]),
        outline: state.settings.badge_outline,
        image: Some(fields[6].trim().to_owned()).filter(|url| !url.is_empty()),
    }
}

fn store(state: &mut RendererState, rules: &Rules) {
    let value = serde_json::to_value(rules).unwrap_or(Value::Null);
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert(RULES_KEY.to_owned(), value.clone());
    }
    state
        .pending_native_actions
        .push(NativeAction::SettingsChange {
            key: RULES_KEY.to_owned(),
            value,
        });
    sync_config(state);
}

fn say(state: &mut RendererState, key: &str) {
    let language = state.settings.language().to_owned();
    state.settings.badge_status = Some(localized(key, &language));
}

fn import_text(state: &mut RendererState, text: &str, source: Option<&str>) {
    let Ok(pack) = engine::parse_pack(text, source) else {
        say(state, "settings.badges_status_failed");
        return;
    };
    let mut rules = current_rules(state);
    if rules.add_pack(pack) {
        store(state, &rules);
        state.settings.badge_fields[0].clear();
        say(state, "settings.badges_status_imported");
    } else {
        say(state, "settings.badges_status_limit");
    }
}

fn receive_import(state: &mut RendererState) {
    let Some((url, receiver)) = state.stream_badges.import.as_ref() else {
        return;
    };
    let Ok(text) = receiver.try_recv() else {
        return;
    };
    let url = url.clone();
    state.stream_badges.import = None;
    match text {
        Some(text) => import_text(state, &text, Some(&url)),
        None => say(state, "settings.badges_status_failed"),
    }
}

fn start_import(state: &mut RendererState) {
    let input = state.settings.badge_fields[0].trim().to_owned();
    if input.starts_with('{') || input.starts_with('[') {
        import_text(state, &input, None);
        return;
    }
    if !(input.starts_with("http://") || input.starts_with("https://")) {
        say(state, "settings.badges_status_failed");
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let receiver = fluxa_effects::fetch_text(&input);
        state.stream_badges.import = Some((input, receiver));
        say(state, "settings.badges_status_loading");
    }
    #[cfg(target_arch = "wasm32")]
    say(state, "settings.badges_status_paste_only");
}

fn clear_form(state: &mut RendererState) {
    for field in state.settings.badge_fields.iter_mut().skip(1) {
        field.clear();
    }
    state.settings.badge_outline = false;
    state.stream_badges.editing = None;
}

fn save_custom(state: &mut RendererState) {
    let fields = &state.settings.badge_fields;
    let optional = |text: &str| Some(text.trim().to_owned()).filter(|text| !text.is_empty());
    let filter = Filter {
        id: state.stream_badges.editing.clone().unwrap_or_default(),
        name: fields[1].clone(),
        pattern: fields[2].trim().to_owned(),
        is_enabled: true,
        tag_color: optional(&fields[3]),
        text_color: optional(&fields[4]),
        border_color: optional(&fields[5]),
        image_url: optional(&fields[6]),
        tag_style: state.settings.badge_outline.then(|| "OUTLINE".to_owned()),
        ..Default::default()
    };
    let mut rules = current_rules(state);
    if rules.upsert_custom(filter) {
        store(state, &rules);
        clear_form(state);
        state.settings.badge_status = None;
    } else {
        say(state, "settings.badges_status_invalid");
    }
}

fn edit_custom(state: &mut RendererState, index: usize) {
    let Some(filter) = current_rules(state).custom.get(index).cloned() else {
        return;
    };
    let fields = &mut state.settings.badge_fields;
    fields[1] = filter.name;
    fields[2] = filter.pattern;
    fields[3] = filter.tag_color.unwrap_or_default();
    fields[4] = filter.text_color.unwrap_or_default();
    fields[5] = filter.border_color.unwrap_or_default();
    fields[6] = filter.image_url.unwrap_or_default();
    state.settings.badge_outline = filter
        .tag_style
        .is_some_and(|style| style.eq_ignore_ascii_case("outline"));
    state.stream_badges.editing = Some(filter.id);
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    let index = |base: u64, count: usize| {
        let offset = node.checked_sub(base)? as usize;
        (offset < count).then_some(offset)
    };
    let packs = state.settings.badge_packs.len();
    let customs = state.settings.badge_custom.len();
    if node == NODE_SETTINGS_BADGE_IMPORT {
        start_import(state);
    } else if node == NODE_SETTINGS_BADGE_STYLE {
        state.settings.badge_outline = !state.settings.badge_outline;
    } else if node == NODE_SETTINGS_BADGE_SAVE {
        save_custom(state);
    } else if node == NODE_SETTINGS_BADGE_CLEAR {
        clear_form(state);
    } else if let Some(i) = index(NODE_SETTINGS_BADGE_PACK_TOGGLE_BASE, packs) {
        let mut rules = current_rules(state);
        if let Some(id) = rules.packs.get(i).map(|pack| pack.id.clone()) {
            rules.toggle_pack(&id);
            store(state, &rules);
        }
    } else if let Some(i) = index(NODE_SETTINGS_BADGE_PACK_REMOVE_BASE, packs) {
        let mut rules = current_rules(state);
        if let Some(id) = rules.packs.get(i).map(|pack| pack.id.clone()) {
            rules.remove_pack(&id);
            store(state, &rules);
        }
    } else if let Some(i) = index(NODE_SETTINGS_BADGE_EDIT_BASE, customs) {
        edit_custom(state, i);
    } else if let Some(i) = index(NODE_SETTINGS_BADGE_TOGGLE_BASE, customs) {
        let mut rules = current_rules(state);
        if let Some(id) = rules.custom.get(i).map(|filter| filter.id.clone()) {
            rules.toggle_custom(&id);
            store(state, &rules);
        }
    } else if let Some(i) = index(NODE_SETTINGS_BADGE_REMOVE_BASE, customs) {
        let mut rules = current_rules(state);
        if let Some(id) = rules.custom.get(i).map(|filter| filter.id.clone()) {
            rules.remove_custom(&id);
            store(state, &rules);
        }
    } else {
        return false;
    }
    true
}
