use std::time::Duration;
use web_time::Instant;

use crate::{RendererState, UiFormFactorJson, semantics};

const DELAY: Duration = Duration::from_millis(450);

fn shortcut_id(node: u64) -> Option<&'static str> {
    Some(match node {
        fluxa_ui::NODE_PLAYER_TOGGLE => "player_play_pause",
        fluxa_ui::NODE_PLAYER_MUTE => "player_mute",
        fluxa_ui::NODE_PLAYER_FULLSCREEN => "player_fullscreen",
        fluxa_ui::NODE_PLAYER_REWIND => "player_seek_back",
        fluxa_ui::NODE_PLAYER_FORWARD => "player_seek_forward",
        fluxa_ui::NODE_PLAYER_NEXT => "player_next_episode",
        fluxa_ui::NODE_PLAYER_PREVIOUS => "player_previous_episode",
        fluxa_ui::NODE_PLAYER_SKIP => "player_skip_active",
        fluxa_ui::NODE_PLAYER_SUBMIT => "player_mark_segment",
        fluxa_ui::NODE_PLAYER_INFO => "player_toggle_stats",
        fluxa_ui::NODE_PLAYER_ANIME4K => "player_anime4k_toggle",
        fluxa_ui::NODE_PLAYER_BOOST => "player_boost_toggle",
        fluxa_ui::NODE_PLAYER_CLOSE | fluxa_ui::NODE_DETAIL_BACK | fluxa_ui::NODE_SETTINGS_BACK => {
            "go_back"
        }
        fluxa_ui::NODE_HOME => "nav_home",
        fluxa_ui::NODE_LIBRARY => "nav_library",
        fluxa_ui::NODE_DISCOVER => "nav_discover",
        fluxa_ui::NODE_CALENDAR => "nav_calendar",
        fluxa_ui::NODE_PROFILE => "nav_settings",
        fluxa_ui::NODE_WINDOW_FULLSCREEN => "toggle_window_fullscreen",
        _ => return None,
    })
}

fn text(state: &RendererState, node: u64) -> Option<String> {
    let label = semantics::tooltip_label(state, node)?;
    let chord = shortcut_id(node).and_then(|id| {
        state
            .settings
            .shortcuts
            .iter()
            .find(|row| row.id == id)
            .map(|row| row.label.as_str())
            .filter(|label| !label.is_empty())
    });
    Some(match chord {
        Some(chord) => format!("{label} ({chord})"),
        None => label,
    })
}

pub(crate) fn update(state: &mut RendererState, hovered: Option<(u64, egui::Rect)>) {
    let node = hovered
        .map(|(node, _)| node)
        .filter(|_| state.home.form_factor == UiFormFactorJson::Desktop);
    if state.hover.map(|(held, _)| held) != node {
        state.hover = node.map(|node| (node, Instant::now()));
        state.home.hover_tip = None;
    }
    let (Some((node, rect)), Some((_, since))) = (hovered.filter(|_| node.is_some()), state.hover)
    else {
        return;
    };
    if state.home.hover_tip.is_some() {
        return;
    }
    let due = since + DELAY;
    if Instant::now() < due {
        state.redraw_at = Some(state.redraw_at.map_or(due, |at| at.min(due)));
        return;
    }
    state.home.hover_tip = text(state, node).map(|text| fluxa_ui::HoverTip { rect, text });
}
