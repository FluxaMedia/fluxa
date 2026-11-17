use super::*;
use crate::dp::Dp;
use crate::draw_text;
use crate::flex::{self, Align, Insets};


const FORM_FIELDS: usize = 7;
const LAYOUT_FIELDS: usize = 9;
const FIELD_LABELS: [&str; BADGE_INPUT_COUNT] = [
    "settings.badges_import_hint",
    "settings.badges_field_name",
    "settings.badges_field_pattern",
    "settings.badges_field_color",
    "settings.badges_field_text_color",
    "settings.badges_field_border_color",
    "settings.badges_field_image",
    "settings.formatter_field_pattern",
    "settings.formatter_field_replacement",
    "settings.layout_field_name",
    "settings.layout_field_detail",
];

pub fn badge_input(node: u64) -> Option<usize> {
    if let Some(offset) = node.checked_sub(NODE_SETTINGS_LAYOUT_INPUT_BASE) {
        let index = LAYOUT_FIELDS + offset as usize;
        return (index < BADGE_INPUT_COUNT).then_some(index);
    }
    let index = node.checked_sub(NODE_SETTINGS_BADGE_INPUT_BASE)? as usize;
    (index < LAYOUT_FIELDS).then_some(index)
}

fn input_node(index: usize) -> u64 {
    if index >= LAYOUT_FIELDS {
        NODE_SETTINGS_LAYOUT_INPUT_BASE + (index - LAYOUT_FIELDS) as u64
    } else {
        NODE_SETTINGS_BADGE_INPUT_BASE + index as u64
    }
}

pub fn badge_input_label(index: usize) -> &'static str {
    FIELD_LABELS[index]
}












pub(super) fn badges_blocks(settings: &SettingsModel, language: &str) -> Vec<serde_json::Value> {
    use serde_json::{Value, json};
    let fill = (0..BADGE_INPUT_COUNT)
        .map(|index| format!("{}=badge{index}", input_node(index)))
        .collect::<Vec<_>>()
        .join(";");
    let pill = |node: u64, key: &str| {
        json!({"node": node.to_string(), "label": localized(key, language), "fill": fill})
    };
    let toggle_key = |on: bool| {
        if on {
            "settings.badges_disable"
        } else {
            "settings.badges_enable"
        }
    };
    let field = |index: usize, hint: &str| {
        json!({
            "kind": "field",
            "title": localized(FIELD_LABELS[index], language),
            "key": format!("badge{index}"),
            "value": settings.badge_fields[index],
            "placeholder": hint,
            "inputKind": "text",
        })
    };
    let item = |title: &str, hint: &str, pills: Vec<Value>| {
        json!({"kind": "item", "title": title, "hint": hint, "pills": pills})
    };
    let group = |key: &str, rows: Vec<Value>| {
        json!({"heading": localized(key, language), "rows": rows})
    };
    let note = |text: String| json!({"kind": "note", "title": text, "danger": false});

    let mut groups = Vec::new();

    let mut import = vec![json!({
        "kind": "item",
        "title": "",
        "hint": "",
        "key": "badge0",
        "value": settings.badge_fields[0],
        "placeholder": localized(FIELD_LABELS[0], language),
        "pills": [pill(NODE_SETTINGS_BADGE_IMPORT, "settings.badges_import")],
    })];
    if let Some(status) = &settings.badge_status {
        import.push(note(status.clone()));
    }
    groups.push(group("settings.group.badges_import", import));

    let packs = if settings.badge_packs.is_empty() {
        vec![note(localized("settings.badges_no_packs", language))]
    } else {
        settings
            .badge_packs
            .iter()
            .enumerate()
            .map(|(index, pack)| {
                item(
                    &pack.name,
                    &format!("{} {}", pack.count, localized("settings.badges_count_unit", language)),
                    vec![
                        pill(NODE_SETTINGS_BADGE_PACK_TOGGLE_BASE + index as u64, toggle_key(pack.active)),
                        pill(NODE_SETTINGS_BADGE_PACK_REMOVE_BASE + index as u64, "settings.badges_remove"),
                    ],
                )
            })
            .collect()
    };
    groups.push(group("settings.group.badges_packs", packs));

    let player = settings
        .player_badge_choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let key = if choice.selected {
                "settings.badges_player_selected"
            } else {
                "settings.badges_player_use"
            };
            item(&choice.name, "", vec![pill(NODE_SETTINGS_BADGE_PLAYER_BASE + index as u64, key)])
        })
        .collect();
    groups.push(group("settings.group.badges_player", player));

    let mut custom: Vec<Value> = (1..FORM_FIELDS)
        .map(|index| field(index, if (3..=5).contains(&index) { "#RRGGBB" } else { "" }))
        .collect();
    let style_key = if settings.badge_outline {
        "settings.option.outline"
    } else {
        "settings.option.filled"
    };
    custom.push(item(
        &localized("settings.badges_field_style", language),
        "",
        vec![pill(NODE_SETTINGS_BADGE_STYLE, style_key)],
    ));
    custom.push(item(
        &settings.badge_preview.label,
        "",
        vec![
            pill(NODE_SETTINGS_BADGE_CLEAR, "settings.badges_clear"),
            pill(NODE_SETTINGS_BADGE_SAVE, "settings.badges_save"),
        ],
    ));
    groups.push(group("settings.group.badges_custom", custom));

    if !settings.badge_custom.is_empty() {
        let rows = settings
            .badge_custom
            .iter()
            .enumerate()
            .map(|(index, custom)| {
                item(
                    &custom.badge.label,
                    "",
                    vec![
                        pill(NODE_SETTINGS_BADGE_EDIT_BASE + index as u64, "settings.badges_edit"),
                        pill(NODE_SETTINGS_BADGE_TOGGLE_BASE + index as u64, toggle_key(custom.enabled)),
                        pill(NODE_SETTINGS_BADGE_REMOVE_BASE + index as u64, "settings.badges_remove"),
                    ],
                )
            })
            .collect();
        groups.push(group("settings.group.badges_yours", rows));
    }

    let mut layout = Vec::new();
    if settings.layout_preview.is_empty() {
        layout.push(note(localized("settings.layout_original", language)));
    }
    layout.extend(settings.layout_preview.iter().map(|line| note(line.clone())));
    layout.extend((LAYOUT_FIELDS..BADGE_INPUT_COUNT).map(|index| field(index, "{resolution}")));
    layout.push(item(
        "",
        "",
        vec![
            pill(NODE_SETTINGS_LAYOUT_USE_CURRENT, "settings.layout_use_current"),
            pill(NODE_SETTINGS_LAYOUT_SAVE, "settings.layout_save"),
        ],
    ));
    groups.push(group("settings.group.layout", layout));

    let presets = settings
        .formatter_presets
        .iter()
        .enumerate()
        .map(|(index, preset)| {
            item(
                &preset.name,
                &preset.description,
                vec![pill(NODE_SETTINGS_FORMATTER_PRESET_BASE + index as u64, toggle_key(preset.enabled))],
            )
        })
        .collect();
    groups.push(group("settings.group.formatter_presets", presets));

    let mut formatter: Vec<Value> = (FORM_FIELDS..LAYOUT_FIELDS).map(|index| field(index, "")).collect();
    formatter.push(item(
        settings.formatter_status.as_deref().unwrap_or(""),
        "",
        vec![
            pill(NODE_SETTINGS_FORMATTER_CLEAR, "settings.badges_clear"),
            pill(NODE_SETTINGS_FORMATTER_SAVE, "settings.formatter_save"),
        ],
    ));
    groups.push(group("settings.group.formatter", formatter));

    if !settings.formatter_rules.is_empty() {
        let rows = settings
            .formatter_rules
            .iter()
            .enumerate()
            .map(|(index, rule)| {
                let replacement = if rule.replacement.is_empty() {
                    localized("settings.formatter_removed", language)
                } else {
                    rule.replacement.clone()
                };
                item(
                    &format!("{}  →  {}", rule.pattern, replacement),
                    "",
                    vec![
                        pill(NODE_SETTINGS_FORMATTER_EDIT_BASE + index as u64, "settings.badges_edit"),
                        pill(NODE_SETTINGS_FORMATTER_TOGGLE_BASE + index as u64, toggle_key(rule.enabled)),
                        pill(NODE_SETTINGS_FORMATTER_REMOVE_BASE + index as u64, "settings.badges_remove"),
                    ],
                )
            })
            .collect();
        groups.push(group("settings.group.formatter_rules", rows));
    }
    groups
}
