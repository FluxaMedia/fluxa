use super::*;

pub(crate) fn activate(state: &mut RendererState, node: u64) {
    match node {
        fluxa_ui::NODE_PLAYER_CLOSE => close(state),
        fluxa_ui::NODE_PLAYER_TOGGLE => command(state, VideoCommand::TogglePause),
        fluxa_ui::NODE_PLAYER_REWIND => seek(state, -1.0),
        fluxa_ui::NODE_PLAYER_FORWARD => seek(state, 1.0),
        fluxa_ui::NODE_PLAYER_MUTE => command(state, VideoCommand::ToggleMute),
        fluxa_ui::NODE_PLAYER_BOOST => toggle_boost(state),
        fluxa_ui::NODE_PLAYER_FULLSCREEN => state.fullscreen_toggle = true,
        fluxa_ui::NODE_PLAYER_CONTROLS => {
            if let Some(player) = state.player.as_mut() {
                player.touch();
            }
        }
        fluxa_ui::NODE_PLAYER_HIDE_CONTROLS => {
            if let Some(player) = state.player.as_mut() {
                player.hide();
            }
        }
        fluxa_ui::NODE_PLAYER_SKIP => {
            let target = state.player.as_mut().and_then(|player| {
                let target = player.overlay.skip_target();
                player.toast_skip();
                player.overlay.dismiss_skip();
                target
            });
            if let Some(target) = target {
                command(state, VideoCommand::SeekTo(target));
            }
        }
        fluxa_ui::NODE_PLAYER_SUBMIT => {
            if let Some(player) = state.player.as_mut() {
                submit::open(player);
            }
        }
        fluxa_ui::NODE_PLAYER_CAST => {
            if let Some(player) = state.player.as_mut() {
                casting::open(player);
            }
        }
        fluxa_ui::NODE_PLAYER_TRACKS => open_menu(state, menu::Menu::Tracks),
        fluxa_ui::NODE_PLAYER_SPEED => open_menu(state, menu::Menu::Speed),
        fluxa_ui::NODE_PLAYER_EPISODES => open_menu(state, menu::Menu::Episodes),
        fluxa_ui::NODE_PLAYER_SETTINGS => open_menu(state, menu::Menu::Settings),
        fluxa_ui::NODE_PLAYER_NEXT => start_next(state, true),
        fluxa_ui::NODE_PLAYER_PREVIOUS => menu::previous(state),
        fluxa_ui::NODE_PLAYER_PANEL_CLOSE => {
            if let Some(player) = state.player.as_mut() {
                player.panel = None;
            }
        }
        node if (fluxa_ui::NODE_PLAYER_PANEL_ROW_BASE
            ..fluxa_ui::NODE_PLAYER_PANEL_ROW_BASE + fluxa_ui::PLAYER_PANEL_ROW_LIMIT as u64)
            .contains(&node) =>
        {
            let row = (node - fluxa_ui::NODE_PLAYER_PANEL_ROW_BASE) as usize;
            match state
                .player
                .as_ref()
                .and_then(|player| player.panel.as_ref())
            {
                Some(Panel::Cast) => casting::activate_row(state, row),
                Some(Panel::Menu(kind)) => menu::activate_row(state, *kind, row),
                _ => submit::activate_row(state, row),
            }
        }
        fluxa_ui::NODE_PLAYER_NEXT_PLAY => play_next(state),
        fluxa_ui::NODE_PLAYER_NEXT_DISMISS => {
            if let Some(player) = state.player.as_mut() {
                player.overlay.dismiss_next();
            }
        }
        fluxa_ui::NODE_PLAYER_RECOMMENDATIONS_CLOSE => dismiss_recommendations(state),
        fluxa_ui::NODE_PLAYER_RECOMMENDATION_PLAY => open_recommendation(state, true),
        fluxa_ui::NODE_PLAYER_RECOMMENDATION_DETAILS => open_recommendation(state, false),
        fluxa_ui::NODE_PLAYER_SOURCES_RETRY => {
            if let Some(item) = state.player.as_ref().map(|player| player.meta.clone()) {
                close(state);
                state
                    .pending_native_actions
                    .push(crate::NativeAction::StartPlayback { item });
            }
        }
        node if node >= fluxa_ui::NODE_PLAYER_SOURCE_BASE => {
            if let Some(player) = state.player.as_mut() {
                player.chosen = Some((node - fluxa_ui::NODE_PLAYER_SOURCE_BASE) as usize);
                player.sources = None;
            }
        }
        node if node >= fluxa_ui::NODE_PLAYER_SOURCE_FILTER_BASE => {
            if let Some(player) = state.player.as_mut() {
                let index = (node - fluxa_ui::NODE_PLAYER_SOURCE_FILTER_BASE) as usize;
                let filter = index.checked_sub(1).and_then(|index| {
                    player
                        .model()
                        .addons()
                        .get(index)
                        .map(|addon| addon.to_string())
                });
                player.source_filter = filter;
            }
            state.screen_scroll_offsets.remove(&Route::Player);
        }
        node if (fluxa_ui::NODE_PLAYER_RECOMMENDATION_BASE
            ..fluxa_ui::NODE_PLAYER_RECOMMENDATION_BASE
                + fluxa_ui::PLAYER_RECOMMENDATION_LIMIT as u64)
            .contains(&node) =>
        {
            if let Some(player) = state.player.as_mut() {
                player.recommendation_index =
                    (node - fluxa_ui::NODE_PLAYER_RECOMMENDATION_BASE) as usize;
            }
        }
        _ => {}
    }
}

pub(crate) fn gesture(state: &mut RendererState, gesture: fluxa_ui::PlayerGesture) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    match gesture {
        fluxa_ui::PlayerGesture::Volume(delta) => {
            let volume = player.status.volume + f64::from(delta) * 100.0;
            set_volume(state, volume);
        }
        fluxa_ui::PlayerGesture::VolumeSet(percent) => set_volume(state, f64::from(percent)),
        fluxa_ui::PlayerGesture::Brightness(delta) => {
            player.brightness = (player.brightness + delta).clamp(MIN_BRIGHTNESS, 1.0);
            let level = f64::from(player.brightness);
            player.toast("brightness", level);
        }
    }
}

pub(crate) fn speed_hold(state: &mut RendererState, held: bool) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    if player.speed_held == held {
        return;
    }
    player.speed_held = held;
    let rate = if held { FAST_SPEED } else { player.speed };
    command(state, VideoCommand::SetSpeed(rate));
}

pub(crate) fn step_speed(state: &mut RendererState, delta: f64) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    player.speed = (player.speed + delta).clamp(0.25, 4.0);
    let rate = player.speed;
    command(state, VideoCommand::SetSpeed(rate));
}

pub(super) fn dismiss_recommendations(state: &mut RendererState) {
    if let Some(player) = state.player.as_mut() {
        player.recommendations.clear();
        player.touch();
    }
}

pub(super) fn open_recommendation(state: &mut RendererState, play: bool) {
    let Some(item) = state
        .player
        .as_ref()
        .and_then(|player| player.recommendations.get(player.recommendation_index))
    else {
        return;
    };
    let (Some(id), Some(item_type)) = (
        item.get("id").and_then(Value::as_str),
        item.get("type").and_then(Value::as_str),
    ) else {
        return;
    };
    let action = if play {
        crate::NativeAction::StartPlayback { item: item.clone() }
    } else {
        crate::NativeAction::Detail {
            id: id.to_owned(),
            item_type: item_type.to_owned(),
            preview: item.clone(),
        }
    };
    close(state);
    state.pending_native_actions.push(action);
}

pub(crate) enum KeyOutcome {
    Handled,
    Focus,
}

pub(crate) fn key(state: &mut RendererState, input: crate::KeyInput) -> KeyOutcome {
    use crate::KeyInput;
    use fluxa_renderer::ui::{GamepadButton, Key};
    let closes = matches!(
        input,
        KeyInput::Key(Key::Back | Key::Escape) | KeyInput::Gamepad(GamepadButton::East)
    );
    let tv = state.home.form_factor == fluxa_ui::UiFormFactorJson::Tv;
    let recommending = state
        .player
        .as_ref()
        .is_some_and(|player| !player.recommendations.is_empty());
    if closes {
        let tv_hides = tv
            && state
                .player
                .as_ref()
                .is_some_and(|player| player.status.has_frame && player.controls_visible());
        let paneled = state
            .player
            .as_ref()
            .is_some_and(|player| player.panel.is_some());
        if paneled {
            if let Some(player) = state.player.as_mut()
                && !menu::back(player)
            {
                player.panel = None;
            }
        } else if recommending {
            dismiss_recommendations(state);
        } else if tv_hides {
            if let Some(player) = state.player.as_mut() {
                player.hide();
            }
        } else {
            close(state);
        }
        return KeyOutcome::Handled;
    }
    if recommending {
        return KeyOutcome::Focus;
    }
    if tv {
        return tv_key(state, input);
    }
    let Some(player) = state.player.as_mut() else {
        return KeyOutcome::Focus;
    };
    if player.controls_visible() {
        player.touch();
        return KeyOutcome::Focus;
    }
    player.touch();
    match input {
        KeyInput::Key(Key::Left) | KeyInput::Gamepad(GamepadButton::DPadLeft) => seek(state, -1.0),
        KeyInput::Key(Key::Right) | KeyInput::Gamepad(GamepadButton::DPadRight) => seek(state, 1.0),
        KeyInput::Key(Key::Enter) | KeyInput::Gamepad(GamepadButton::South) => {
            command(state, VideoCommand::TogglePause)
        }
        _ => {}
    }
    KeyOutcome::Handled
}

pub(super) fn tv_key(state: &mut RendererState, input: crate::KeyInput) -> KeyOutcome {
    use crate::KeyInput;
    use fluxa_renderer::ui::{GamepadButton, Key};
    let on_seek = state.ui.focused() == Some(fluxa_ui::NODE_PLAYER_SEEK);
    let Some(player) = state.player.as_mut() else {
        return KeyOutcome::Focus;
    };
    let direction = match input {
        KeyInput::Key(Key::Left) | KeyInput::Gamepad(GamepadButton::DPadLeft) => -1.0,
        KeyInput::Key(Key::Right) | KeyInput::Gamepad(GamepadButton::DPadRight) => 1.0,
        _ => 0.0,
    };
    let ok = matches!(
        input,
        KeyInput::Key(Key::Enter) | KeyInput::Gamepad(GamepadButton::South)
    );
    let visible = player.controls_visible();
    if ok && !visible {
        if let Some(node) = player.overlay.card_node() {
            activate(state, node);
            return KeyOutcome::Handled;
        }
    }
    if visible && !on_seek {
        player.touch();
        return KeyOutcome::Focus;
    }
    if direction != 0.0 {
        player.scrub(direction);
    } else if ok && visible {
        match player.scrub.take() {
            Some((time, _)) => command(state, VideoCommand::SeekTo(time)),
            None => command(state, VideoCommand::TogglePause),
        }
    } else if visible {
        player.touch();
        return KeyOutcome::Focus;
    } else {
        player.touch();
    }
    state.ui.set_focus(Some(fluxa_ui::NODE_PLAYER_SEEK));
    KeyOutcome::Handled
}

fn open_menu(state: &mut RendererState, kind: menu::Menu) {
    menu::open(state, kind);
}

fn volume_max(state: &RendererState) -> f64 {
    if state.settings.bool_value("playerAudioBoost") {
        200.0
    } else {
        100.0
    }
}

fn set_volume(state: &mut RendererState, volume: f64) {
    let max = volume_max(state);
    let volume = volume.clamp(0.0, max);
    let Some(player) = state.player.as_mut() else {
        return;
    };
    player.status.volume = volume;
    player.toast("volume", volume);
    if let Some(video) = state.video.as_mut() {
        video.command(VideoCommand::Mpv(vec![
            "set".to_owned(),
            "volume-max".to_owned(),
            max.to_string(),
        ]));
        video.command(VideoCommand::SetVolume(volume));
    }
}

pub(crate) fn toggle_boost(state: &mut RendererState) {
    let on = !state.settings.bool_value("playerAudioBoost");
    let value = json!(on);
    if let Some(values) = state.settings.values.as_object_mut() {
        values.insert("playerAudioBoost".to_owned(), value.clone());
    }
    state
        .pending_native_actions
        .push(crate::NativeAction::SettingsChange {
            key: "playerAudioBoost".to_owned(),
            value,
        });
    let current = state
        .player
        .as_ref()
        .map_or(100.0, |player| player.status.volume);
    set_volume(state, if on { 200.0 } else { current });
}

pub(crate) fn toggle_stats(state: &mut RendererState) {
    if let Some(player) = state.player.as_mut() {
        player.stats_visible = !player.stats_visible;
        player.stats_at = None;
    }
}
