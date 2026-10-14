use fluxa_core::detail_sections as sections;
use fluxa_ui::{
    DetailSection, DetailSectionRow, NODE_SETTINGS_DETAIL_SECTION_BASE,
    NODE_SETTINGS_DETAIL_SECTION_DOWN_BASE, NODE_SETTINGS_DETAIL_SECTION_UP_BASE,
};
use serde_json::{Value, json};

use super::*;

pub(crate) fn sync_settings(state: &mut RendererState) {
    if state.route != Route::Settings {
        return;
    }
    state.settings.detail_sections = sections::rows(&state.settings.values)
        .into_iter()
        .filter_map(|(key, enabled)| {
            Some(DetailSectionRow {
                section: DetailSection::from_key(key)?,
                enabled,
            })
        })
        .collect();
}

fn section_at(base: u64, node: u64) -> Option<&'static str> {
    let index = node.checked_sub(base)? as usize;
    sections::SECTIONS
        .iter()
        .copied()
        .find(|key| DetailSection::from_key(key).is_some_and(|section| section as usize == index))
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    if let Some(section) = section_at(NODE_SETTINGS_DETAIL_SECTION_BASE, node)
        .filter(|_| node < NODE_SETTINGS_DETAIL_SECTION_UP_BASE)
    {
        let Some(key) = sections::toggle_key(section) else {
            return true;
        };
        let on = sections::enabled(&state.settings.values, section);
        crate::stream_badges::set_setting(state, key, json!(!on));
    } else if let Some(section) = section_at(NODE_SETTINGS_DETAIL_SECTION_UP_BASE, node)
        .filter(|_| node < NODE_SETTINGS_DETAIL_SECTION_DOWN_BASE)
    {
        move_section(state, section, -1);
    } else if let Some(section) = section_at(NODE_SETTINGS_DETAIL_SECTION_DOWN_BASE, node)
        .filter(|_| node < NODE_SETTINGS_DETAIL_SECTION_DOWN_BASE + 10)
    {
        move_section(state, section, 1);
    } else {
        return false;
    }
    sync_settings(state);
    true
}

fn move_section(state: &mut RendererState, section: &str, delta: isize) {
    if let Some(order) = sections::moved(&state.settings.values, section, delta) {
        crate::stream_badges::set_setting(state, sections::ORDER_KEY, Value::String(order));
    }
}
