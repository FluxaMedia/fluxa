use fluxa_renderer::ui::Key;
use fluxa_ui::{
    NODE_CALENDAR, NODE_DISCOVER, NODE_DISCOVER_SEARCH, NODE_HOME, NODE_LIBRARY,
    NODE_LIBRARY_SEARCH, NODE_PROFILE, NODE_SETTINGS_SEARCH, NODE_SETTINGS_SHORTCUT_BASE,
    NODE_SETTINGS_SHORTCUT_RESET_ALL, NODE_SETTINGS_SHORTCUT_RESET_BASE, ShortcutRow,
};
use serde_json::{Value, json};

use crate::player::{self, VideoCommand};
use crate::{
    KeyInput, NativeAction, RendererState, Route, UiAction, UiNodeKind, core_value, key_down,
    rebuild_current_ui, remember_actions,
};

const SETTING_KEY: &str = "keyboardShortcuts";
const SEEK_SMALL: f64 = 10.0;
const SEEK_BIG: f64 = 60.0;
const VOLUME_STEP: f32 = 0.05;
const SPEED_STEP: f64 = 0.25;

fn overrides(state: &RendererState) -> Value {
    state
        .settings
        .values
        .get(SETTING_KEY)
        .cloned()
        .unwrap_or(Value::Null)
}

pub(crate) fn refresh(state: &mut RendererState) {
    let Some(plan) = core_value("shortcutBindings", json!({"overrides": overrides(state)})) else {
        return;
    };
    state.settings.shortcuts = plan["actions"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|action| ShortcutRow {
            id: action["id"].as_str().unwrap_or_default().to_owned(),
            player: action["scope"] == "player",
            label: action["label"].as_str().unwrap_or_default().to_owned(),
            custom: action["custom"].as_bool().unwrap_or(false),
        })
        .collect();
    state.settings.shortcut_recording = state.shortcut_recording.clone();
}

fn store(state: &mut RendererState, overrides: Value) {
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert(SETTING_KEY.to_owned(), overrides.clone());
    }
    state
        .pending_native_actions
        .push(NativeAction::SettingsChange {
            key: SETTING_KEY.to_owned(),
            value: overrides,
        });
    refresh(state);
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    let row_id = |base: u64| {
        let index = node.checked_sub(base)? as usize;
        state
            .settings
            .shortcuts
            .get(index)
            .map(|row| row.id.clone())
    };
    if node == NODE_SETTINGS_SHORTCUT_RESET_ALL {
        state.shortcut_recording = None;
        store(state, json!({}));
    } else if let Some(id) = row_id(NODE_SETTINGS_SHORTCUT_RESET_BASE)
        && node < NODE_SETTINGS_SHORTCUT_RESET_ALL
    {
        assign(state, &id, json!({"reset": true}));
    } else if let Some(id) = row_id(NODE_SETTINGS_SHORTCUT_BASE)
        && node < NODE_SETTINGS_SHORTCUT_RESET_BASE
    {
        state.shortcut_recording = Some(id);
        refresh(state);
    } else {
        return false;
    }
    true
}

fn assign(state: &mut RendererState, action: &str, change: Value) -> bool {
    let mut args = json!({"overrides": overrides(state), "action": action});
    if let (Some(args), Some(change)) = (args.as_object_mut(), change.as_object()) {
        args.extend(change.clone());
    }
    let Some(plan) = core_value("shortcutAssign", args) else {
        return false;
    };
    if plan["ok"] != true {
        return false;
    }
    store(state, plan["overrides"].clone());
    true
}

pub(crate) fn recording(state: &RendererState) -> bool {
    state.shortcut_recording.is_some()
}

pub(crate) fn cancel_recording(state: &mut RendererState) {
    state.shortcut_recording = None;
    refresh(state);
}

pub(crate) fn record(state: &mut RendererState, chord: &str) {
    let Some(action) = state.shortcut_recording.clone() else {
        return;
    };
    match chord {
        "escape" => {}
        "backspace" | "delete" => {
            assign(state, &action, json!({"chord": null}));
        }
        _ => {
            if !assign(state, &action, json!({"chord": chord})) {
                return;
            }
        }
    }
    state.shortcut_recording = None;
    refresh(state);
}

fn typing(state: &RendererState) -> bool {
    state.wants_keyboard
        || state
            .ui
            .focused()
            .and_then(|node| state.ui.node(node))
            .is_some_and(|node| node.kind == UiNodeKind::Input)
}

pub(crate) fn run(state: &mut RendererState, chord: &str, repeat: bool) -> bool {
    if state.card_menu.is_some() {
        return false;
    }
    let playing = state.player.is_some();
    let Some(plan) = core_value(
        "shortcutResolve",
        json!({
            "chord": chord,
            "overrides": overrides(state),
            "player": playing,
            "typing": typing(state),
        }),
    ) else {
        return false;
    };
    let Some(action) = plan["action"].as_str() else {
        return false;
    };
    if state.profiles.is_some() && action != "toggle_window_fullscreen" {
        return false;
    }
    if repeat
        && !matches!(
            action,
            "player_seek_back"
                | "player_seek_forward"
                | "player_seek_big_back"
                | "player_seek_big_forward"
                | "player_volume_up"
                | "player_volume_down"
                | "player_speed_decrease"
                | "player_speed_increase"
                | "player_frame_step"
                | "player_frame_back"
                | "player_subtitle_delay_decrease"
                | "player_subtitle_delay_increase"
        )
    {
        return true;
    }
    perform(state, action);
    true
}

fn open(state: &mut RendererState, node: u64) {
    rebuild_current_ui(state);
    remember_actions(state, vec![UiAction::Activated(node)]);
}

fn perform(state: &mut RendererState, action: &str) {
    match action {
        "nav_home" => open(state, NODE_HOME),
        "nav_library" => open(state, NODE_LIBRARY),
        "nav_discover" => open(state, NODE_DISCOVER),
        "nav_calendar" => open(state, NODE_CALENDAR),
        "nav_settings" => open(state, NODE_PROFILE),
        "focus_search" => focus_search(state),
        "go_back" => key_down(state, KeyInput::Key(Key::Back)),
        "toggle_window_fullscreen" | "player_fullscreen" => state.fullscreen_toggle = true,
        _ => player_action(state, action),
    }
}

fn focus_search(state: &mut RendererState) {
    let node = match state.route {
        Route::Library => NODE_LIBRARY_SEARCH,
        Route::Discover => NODE_DISCOVER_SEARCH,
        Route::Settings => NODE_SETTINGS_SEARCH,
        _ => {
            open(state, NODE_DISCOVER);
            return;
        }
    };
    state.keyboard_focus_visible = true;
    rebuild_current_ui(state);
    let actions = state
        .ui
        .dispatch(fluxa_renderer::ui::UiEvent::FocusRequest(node));
    remember_actions(state, actions);
}

fn player_action(state: &mut RendererState, action: &str) {
    let Some(session) = state.player.as_mut() else {
        return;
    };
    session.touch();
    let duration = session.status.duration;
    match action {
        "player_play_pause" => player::command(state, VideoCommand::TogglePause),
        "player_seek_back" => player::command(state, VideoCommand::Seek(-SEEK_SMALL)),
        "player_seek_forward" => player::command(state, VideoCommand::Seek(SEEK_SMALL)),
        "player_seek_big_back" => player::command(state, VideoCommand::Seek(-SEEK_BIG)),
        "player_seek_big_forward" => player::command(state, VideoCommand::Seek(SEEK_BIG)),
        "player_seek_start" => player::command(state, VideoCommand::SeekTo(0.0)),
        "player_seek_end" => player::command(state, VideoCommand::SeekTo(duration)),
        "player_frame_step" => mpv(state, &["frame-step"]),
        "player_frame_back" => mpv(state, &["frame-back-step"]),
        "player_cycle_audio" => mpv(state, &["cycle", "audio"]),
        "player_cycle_subtitles" => mpv(state, &["cycle", "sub"]),
        "player_subtitle_delay_decrease" => mpv(state, &["add", "sub-delay", "-0.1"]),
        "player_subtitle_delay_increase" => mpv(state, &["add", "sub-delay", "0.1"]),
        "player_mute" => player::command(state, VideoCommand::ToggleMute),
        id if id.starts_with("player_seek_percent_") => {
            if let Some(tenth) = id.chars().last().and_then(|c| c.to_digit(10)) {
                player::command(
                    state,
                    VideoCommand::SeekTo(duration * f64::from(tenth) / 10.0),
                );
            }
        }
        "player_volume_up" => {
            player::gesture(state, fluxa_ui::PlayerGesture::Volume(VOLUME_STEP));
        }
        "player_volume_down" => {
            player::gesture(state, fluxa_ui::PlayerGesture::Volume(-VOLUME_STEP));
        }
        "player_speed_decrease" => player::step_speed(state, -SPEED_STEP),
        "player_speed_increase" => player::step_speed(state, SPEED_STEP),
        "player_skip_active" => player::activate(state, fluxa_ui::NODE_PLAYER_SKIP),
        "player_next_episode" => player::media_command(state, "next", 0.0),
        _ => {}
    }
}

fn mpv(state: &mut RendererState, args: &[&str]) {
    let args = args.iter().map(|arg| (*arg).to_owned()).collect();
    player::command(state, VideoCommand::Mpv(args));
}
