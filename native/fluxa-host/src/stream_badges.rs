use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

use egui::Color32;
use fluxa_core::player::stream_badges::{self as engine, BadgeKind, Filter, Options, Rules, Theme};
use fluxa_core::player::stream_layout::{self as layouts, Layout};
use fluxa_core::player::stream_formatter::{Formatter, PRESETS, Rule as FormatterRule};
use fluxa_ui::{
    BadgeCustomRow, BadgePackRow, FormatterPresetRow, FormatterRow, NODE_SETTINGS_BADGE_CLEAR, NODE_SETTINGS_BADGE_EDIT_BASE,
    NODE_SETTINGS_BADGE_IMPORT, NODE_SETTINGS_BADGE_PACK_REMOVE_BASE,
    NODE_SETTINGS_BADGE_PACK_TOGGLE_BASE, NODE_SETTINGS_BADGE_REMOVE_BASE,
    NODE_SETTINGS_BADGE_SAVE, NODE_SETTINGS_BADGE_STYLE, NODE_SETTINGS_BADGE_TOGGLE_BASE,
    NODE_SETTINGS_FORMATTER_CLEAR, NODE_SETTINGS_FORMATTER_EDIT_BASE,
    NODE_SETTINGS_LAYOUT_SAVE, NODE_SETTINGS_LAYOUT_USE_CURRENT,
    NODE_SETTINGS_FORMATTER_PRESET_BASE,
    NODE_SETTINGS_FORMATTER_REMOVE_BASE, NODE_SETTINGS_FORMATTER_SAVE,
    NODE_SETTINGS_FORMATTER_TOGGLE_BASE,
    SourceBadge, localized,
};
use serde_json::Value;

use crate::{NativeAction, RendererState};

const RULES_KEY: &str = "streamBadgeRules";
const FORMATTER_KEY: &str = "streamFormatterRules";
const FORM_FIELDS: usize = 7;
const LAYOUT_FIELDS: usize = 9;

#[derive(Default)]
pub(crate) struct State {
    import: Option<(String, Receiver<Option<String>>)>,
    editing: Option<String>,
    editing_rule: Option<String>,
}

struct Config {
    enabled: bool,
    top: bool,
    options: Options,
    raw: Value,
    rules: Rules,
    cache: HashMap<(String, Option<i64>), Vec<SourceBadge>>,
    formatter_enabled: bool,
    formatter_raw: Value,
    formatter: Formatter,
    formatted: HashMap<String, String>,
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

#[derive(Default)]
struct LayoutConfig {
    key: (String, String, String),
    layout: Option<Layout>,
    cache: HashMap<(String, Option<i64>), (String, String)>,
}

static LAYOUT: Mutex<Option<LayoutConfig>> = Mutex::new(None);

fn active_layout(mode: &str, name: &str, detail: &str) -> Option<Layout> {
    if mode == "custom" {
        if name.trim().is_empty() && detail.trim().is_empty() {
            return layouts::preset("detailed");
        }
        return Some(Layout {
            name: name.to_owned(),
            detail: detail.to_owned(),
        });
    }
    layouts::preset(mode)
}

fn escaped(template: &str) -> String {
    template.replace('\n', "\\n")
}

fn unescaped(template: &str) -> String {
    template.replace("\\n", "\n")
}

pub(crate) fn lay_out(text: &str, video_size: Option<i64>) -> Option<(String, String)> {
    let mut guard = LAYOUT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = guard.as_mut()?;
    let layout = config.layout.clone()?;
    let key = (text.to_owned(), video_size);
    if let Some(found) = config.cache.get(&key) {
        return Some(found.clone());
    }
    let fields = layouts::extract(text, video_size);
    let found = (
        layouts::render(&layout.name, &fields),
        layouts::render(&layout.detail, &fields),
    );
    if config.cache.len() > 2000 {
        config.cache.clear();
    }
    config.cache.insert(key, found.clone());
    Some(found)
}

fn sync_layout(state: &RendererState) {
    let settings = &state.settings;
    let key = (
        settings.str_value("streamLayout").unwrap_or("off").to_owned(),
        settings.str_value("streamLayoutName").unwrap_or_default().to_owned(),
        settings.str_value("streamLayoutDetail").unwrap_or_default().to_owned(),
    );
    let mut guard = LAYOUT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let config = guard.get_or_insert_with(|| LayoutConfig {
        key: (String::from("\0"), String::new(), String::new()),
        ..Default::default()
    });
    if config.key != key {
        config.layout = active_layout(&key.0, &unescaped(&key.1), &unescaped(&key.2));
        config.cache.clear();
        config.key = key;
    }
}

const SAMPLE: &str = "Movie.2160p.UHD.BluRay.REMUX.HEVC.DV.HDR10.Atmos.TrueHD.7.1-GRP 🇬🇧 Italian";
const SAMPLE_BYTES: i64 = 67_645_734_912;

fn layout_preview(state: &RendererState) -> Vec<String> {
    let settings = &state.settings;
    let mode = settings.str_value("streamLayout").unwrap_or("off");
    let fields = &settings.badge_fields;
    let drafting = !fields[LAYOUT_FIELDS].trim().is_empty() || !fields[LAYOUT_FIELDS + 1].trim().is_empty();
    let layout = if drafting {
        Some(Layout {
            name: unescaped(&fields[LAYOUT_FIELDS]),
            detail: unescaped(&fields[LAYOUT_FIELDS + 1]),
        })
    } else {
        active_layout(
            mode,
            &unescaped(settings.str_value("streamLayoutName").unwrap_or_default()),
            &unescaped(settings.str_value("streamLayoutDetail").unwrap_or_default()),
        )
    };
    let Some(layout) = layout else {
        return Vec::new();
    };
    let extracted = layouts::extract(SAMPLE, Some(SAMPLE_BYTES));
    let mut lines = vec![layouts::render(&layout.name, &extracted)];
    lines.extend(layouts::render(&layout.detail, &extracted).lines().map(str::to_owned));
    lines.retain(|line| !line.is_empty());
    lines
}

fn set_setting(state: &mut RendererState, key: &str, value: Value) {
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert(key.to_owned(), value.clone());
    }
    state
        .pending_native_actions
        .push(NativeAction::SettingsChange {
            key: key.to_owned(),
            value,
        });
}

fn save_layout(state: &mut RendererState) {
    let name = state.settings.badge_fields[LAYOUT_FIELDS].trim().to_owned();
    let detail = state.settings.badge_fields[LAYOUT_FIELDS + 1].trim().to_owned();
    set_setting(state, "streamLayoutName", Value::String(name));
    set_setting(state, "streamLayoutDetail", Value::String(detail));
    set_setting(state, "streamLayout", Value::String("custom".to_owned()));
    for field in state.settings.badge_fields[LAYOUT_FIELDS..].iter_mut() {
        field.clear();
    }
    sync_layout(state);
}

fn use_current_layout(state: &mut RendererState) {
    let mode = state.settings.str_value("streamLayout").unwrap_or("off").to_owned();
    let name = state.settings.str_value("streamLayoutName").unwrap_or_default().to_owned();
    let detail = state.settings.str_value("streamLayoutDetail").unwrap_or_default().to_owned();
    let layout = active_layout(&mode, &unescaped(&name), &unescaped(&detail))
        .or_else(|| layouts::preset("detailed"));
    if let Some(layout) = layout {
        state.settings.badge_fields[LAYOUT_FIELDS] = escaped(&layout.name);
        state.settings.badge_fields[LAYOUT_FIELDS + 1] = escaped(&layout.detail);
    }
}

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

pub(crate) fn format_text(text: &str) -> String {
    let mut guard = CONFIG
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(config) = guard.as_mut().filter(|config| config.formatter_enabled) else {
        return text.to_owned();
    };
    if let Some(found) = config.formatted.get(text) {
        return found.clone();
    }
    let found = config.formatter.apply(text);
    if config.formatted.len() > 2000 {
        config.formatted.clear();
    }
    config.formatted.insert(text.to_owned(), found.clone());
    found
}

fn current_formatter(state: &RendererState) -> Formatter {
    Formatter::from_value(
        state
            .settings
            .values
            .get(FORMATTER_KEY)
            .unwrap_or(&Value::Null),
    )
}

fn current_rules(state: &RendererState) -> Rules {
    Rules::from_value(state.settings.values.get(RULES_KEY).unwrap_or(&Value::Null))
}

pub(crate) fn poll(state: &mut RendererState) {
    receive_import(state);
    sync_config(state);
    sync_layout(state);
    if state.route == crate::Route::Settings {
        state.settings.layout_preview = layout_preview(state);
        state.settings.badge_preview = preview(state);
    }
}

fn sync_config(state: &mut RendererState) {
    let settings = &state.settings;
    let raw = settings.values.get(RULES_KEY).unwrap_or(&Value::Null);
    let formatter_raw = settings.values.get(FORMATTER_KEY).unwrap_or(&Value::Null);
    let formatter_enabled = settings.bool_value("streamFormatterEnabled");
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
        && config.formatter_raw == *formatter_raw
    {
        if config.formatter_enabled != formatter_enabled {
            config.formatter_enabled = formatter_enabled;
            config.formatted.clear();
        }
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
    let formatter = Formatter::from_value(formatter_raw);
    let formatter_raw = formatter_raw.clone();
    *guard = Some(Config {
        enabled,
        top,
        options,
        raw,
        rules: rules.clone(),
        cache: HashMap::new(),
        formatter_enabled,
        formatter_raw,
        formatter: formatter.clone(),
        formatted: HashMap::new(),
    });
    drop(guard);
    let language = state.settings.language().to_owned();
    state.settings.formatter_presets = PRESETS
        .iter()
        .map(|preset| FormatterPresetRow {
            name: localized(&format!("settings.formatter_preset.{}", preset.id), &language),
            description: localized(
                &format!("settings.formatter_preset.{}.description", preset.id),
                &language,
            ),
            enabled: formatter.presets.iter().any(|id| id == preset.id),
        })
        .collect();
    state.settings.formatter_rules = formatter
        .rules
        .iter()
        .map(|rule| FormatterRow {
            pattern: rule.pattern.clone(),
            replacement: rule.replacement.clone(),
            enabled: rule.is_enabled,
        })
        .collect();
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
    for field in state.settings.badge_fields[1..FORM_FIELDS].iter_mut() {
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

fn store_formatter(state: &mut RendererState, formatter: &Formatter) {
    let value = serde_json::to_value(formatter).unwrap_or(Value::Null);
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert(FORMATTER_KEY.to_owned(), value.clone());
    }
    state
        .pending_native_actions
        .push(NativeAction::SettingsChange {
            key: FORMATTER_KEY.to_owned(),
            value,
        });
    sync_config(state);
}

fn clear_formatter_form(state: &mut RendererState) {
    for field in state.settings.badge_fields[FORM_FIELDS..].iter_mut() {
        field.clear();
    }
    state.stream_badges.editing_rule = None;
}

fn save_formatter(state: &mut RendererState) {
    let fields = &state.settings.badge_fields;
    let rule = FormatterRule {
        id: state.stream_badges.editing_rule.clone().unwrap_or_default(),
        pattern: fields[FORM_FIELDS].clone(),
        replacement: fields[FORM_FIELDS + 1].clone(),
        is_enabled: true,
    };
    let mut formatter = current_formatter(state);
    if formatter.upsert(rule) {
        store_formatter(state, &formatter);
        clear_formatter_form(state);
        state.settings.formatter_status = None;
    } else {
        let language = state.settings.language().to_owned();
        state.settings.formatter_status = Some(localized("settings.formatter_invalid", &language));
    }
}

fn edit_formatter(state: &mut RendererState, index: usize) {
    let Some(rule) = current_formatter(state).rules.get(index).cloned() else {
        return;
    };
    state.settings.badge_fields[FORM_FIELDS] = rule.pattern;
    state.settings.badge_fields[FORM_FIELDS + 1] = rule.replacement;
    state.stream_badges.editing_rule = Some(rule.id);
}

fn activate_formatter(state: &mut RendererState, node: u64) -> bool {
    let index = |base: u64, count: usize| {
        let offset = node.checked_sub(base)? as usize;
        (offset < count).then_some(offset)
    };
    let count = state.settings.formatter_rules.len();
    if node == NODE_SETTINGS_LAYOUT_SAVE {
        save_layout(state);
    } else if node == NODE_SETTINGS_LAYOUT_USE_CURRENT {
        use_current_layout(state);
    } else if let Some(i) = index(NODE_SETTINGS_FORMATTER_PRESET_BASE, PRESETS.len()) {
        let mut formatter = current_formatter(state);
        formatter.toggle_preset(PRESETS[i].id);
        store_formatter(state, &formatter);
    } else if node == NODE_SETTINGS_FORMATTER_SAVE {
        save_formatter(state);
    } else if node == NODE_SETTINGS_FORMATTER_CLEAR {
        clear_formatter_form(state);
        state.settings.formatter_status = None;
    } else if let Some(i) = index(NODE_SETTINGS_FORMATTER_EDIT_BASE, count) {
        edit_formatter(state, i);
    } else if let Some(i) = index(NODE_SETTINGS_FORMATTER_TOGGLE_BASE, count) {
        let mut formatter = current_formatter(state);
        if let Some(id) = formatter.rules.get(i).map(|rule| rule.id.clone()) {
            formatter.toggle(&id);
            store_formatter(state, &formatter);
        }
    } else if let Some(i) = index(NODE_SETTINGS_FORMATTER_REMOVE_BASE, count) {
        let mut formatter = current_formatter(state);
        if let Some(id) = formatter.rules.get(i).map(|rule| rule.id.clone()) {
            formatter.remove(&id);
            store_formatter(state, &formatter);
        }
    } else {
        return false;
    }
    true
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    let index = |base: u64, count: usize| {
        let offset = node.checked_sub(base)? as usize;
        (offset < count).then_some(offset)
    };
    if activate_formatter(state, node) {
        return true;
    }
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
