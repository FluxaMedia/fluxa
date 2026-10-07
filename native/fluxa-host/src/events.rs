use super::*;

impl FluxaHost {
    pub fn scroll(&self, delta_y: f32) {
        self.with_state(|state| {
            let viewport = logical_viewport(state);
            if active_route(state) == Route::Home {
                let max = fluxa_ui::home_scroll_max(viewport, &state.home);
                state.home.scroll_offset = (state.home.scroll_offset + delta_y).clamp(0.0, max);
            } else {
                update_screen_scroll(state, delta_y, viewport);
            }
        });
    }

    pub fn gamepad_scroll(&self, delta_y: f32) {
        self.wheel(0.0, delta_y);
        self.with_state(|state| {
            input::hide_pointer(state);
            layout::follow_focus(state);
        });
    }

    pub fn mouse_moved(&self, x: f32, y: f32) {
        self.with_state(|state| {
            let position = state.mirror_position(Pos2::new(x, y) * (state.density / state.scale()));
            state.mouse_position = Some(position);
            state.keyboard_focus_visible = false;
            if let Some(player) = state.player.as_mut() {
                player.touch();
            }
            state.egui_events.push(egui::Event::PointerMoved(position));
        });
    }

    pub fn mouse_button(&self, button: MouseButton, pressed: bool, x: f32, y: f32) {
        self.with_state(|state| {
            let position = state.mirror_position(Pos2::new(x, y) * (state.density / state.scale()));
            state.mouse_position = Some(position);
            state.keyboard_focus_visible = false;
            if pressed
                && matches!(button, MouseButton::Secondary)
                && state.card_menu.is_none()
                && card_menu::open_at(state, position)
            {
                return;
            }
            state.egui_events.push(egui::Event::PointerButton {
                pos: position,
                button: match button {
                    MouseButton::Primary => egui::PointerButton::Primary,
                    MouseButton::Secondary => egui::PointerButton::Secondary,
                    MouseButton::Middle => egui::PointerButton::Middle,
                },
                pressed,
                modifiers: state.modifiers,
            });
        });
    }

    pub fn mouse_left(&self) {
        self.with_state(|state| {
            state.mouse_position = None;
            state.egui_events.push(egui::Event::PointerGone);
        });
    }

    pub fn wheel(&self, delta_x: f32, delta_y: f32) {
        self.with_state(|state| {
            let delta_x = if state.mirrored() { -delta_x } else { delta_x };
            let (delta_x, delta_y) = if state.modifiers.shift && delta_x == 0.0 {
                (delta_y, 0.0)
            } else {
                (delta_x, delta_y)
            };
            state.egui_events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(-delta_x, -delta_y),
                phase: egui::TouchPhase::Move,
                modifiers: state.modifiers,
            });
            let viewport = logical_viewport(state);
            if state.route == Route::Home {
                if delta_x != 0.0
                    && let Some(row) = state.mouse_position.and_then(|position| {
                        fluxa_ui::home_row_at_y(viewport, &state.home, position.y)
                    })
                {
                    if state.home.row_scroll_offsets.len() <= row {
                        state.home.row_scroll_offsets.resize(row + 1, 0.0);
                    }
                    let max = fluxa_ui::home_row_scroll_max(viewport, &state.home, row);
                    let offset = &mut state.home.row_scroll_offsets[row];
                    *offset = (*offset + delta_x).clamp(0.0, max);
                    request_home_row_load_more(state, row, viewport);
                }
                let max = fluxa_ui::home_scroll_max(viewport, &state.home);
                state.home.scroll_offset = (state.home.scroll_offset + delta_y).clamp(0.0, max);
            } else if delta_y != 0.0 {
                update_screen_scroll(state, delta_y, viewport);
            }
        });
    }

    pub fn set_modifiers(&self, modifiers: egui::Modifiers) {
        self.with_state(|state| state.modifiers = modifiers);
    }

    pub fn egui_key(&self, key: egui::Key, pressed: bool) {
        self.with_state(|state| {
            state.egui_events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: state.modifiers,
            });
        });
    }

    pub fn egui_text(&self, text: &str) {
        self.with_state(|state| state.egui_events.push(egui::Event::Text(text.to_owned())));
    }

    pub fn egui_paste(&self, text: &str) {
        self.with_state(|state| state.egui_events.push(egui::Event::Paste(text.to_owned())));
    }

    pub fn take_copied(&self) -> Option<String> {
        self.with_state(|state| state.copied.take()).flatten()
    }

    pub fn egui_event(&self, event: egui::Event) {
        self.with_state(|state| {
            let event = if state.mirrored() {
                match event {
                    egui::Event::PointerMoved(pos) => {
                        egui::Event::PointerMoved(state.mirror_position(pos))
                    }
                    egui::Event::PointerButton {
                        pos,
                        button,
                        pressed,
                        modifiers,
                    } => egui::Event::PointerButton {
                        pos: state.mirror_position(pos),
                        button,
                        pressed,
                        modifiers,
                    },
                    other => other,
                }
            } else {
                event
            };
            state.egui_events.push(event)
        });
    }

    pub fn wants_keyboard(&self) -> bool {
        self.with_state(|state| state.wants_keyboard)
            .unwrap_or(false)
    }

    pub fn cursor(&self) -> egui::CursorIcon {
        self.with_state(|state| state.cursor).unwrap_or_default()
    }

    pub fn pointer(&self, phase: PointerPhase, x: f32, y: f32) {
        self.with_state(|state| {
            let ratio = state.density / state.scale();
            pointer_event(state, phase, [state.mirror_x(x * ratio), y * ratio])
        });
    }

    pub fn key_down(&self, input: KeyInput) {
        self.with_state(|state| {
            let input = if state.mirrored() {
                match input {
                    KeyInput::Key(Key::Left) => KeyInput::Key(Key::Right),
                    KeyInput::Key(Key::Right) => KeyInput::Key(Key::Left),
                    KeyInput::Gamepad(GamepadButton::DPadLeft) => {
                        KeyInput::Gamepad(GamepadButton::DPadRight)
                    }
                    KeyInput::Gamepad(GamepadButton::DPadRight) => {
                        KeyInput::Gamepad(GamepadButton::DPadLeft)
                    }
                    other => other,
                }
            } else {
                input
            };
            key_down(state, input)
        });
    }

    pub fn shortcut(&self, chord: &str, repeat: bool) -> bool {
        self.with_state(|state| {
            if shortcuts::recording(state) {
                shortcuts::record(state, chord);
                return true;
            }
            shortcuts::run(state, chord, repeat)
        })
        .unwrap_or(false)
    }

    pub fn shortcut_recording(&self) -> bool {
        self.with_state(|state| shortcuts::recording(state))
            .unwrap_or(false)
    }

    pub fn back(&self) -> bool {
        self.with_state(|state| {
            if state.player.is_none() && state.profiles.is_none() && state.route == Route::Home {
                return false;
            }
            key_down(state, KeyInput::Key(Key::Back));
            true
        })
        .unwrap_or(false)
    }

    pub fn focused_node(&self) -> Option<u64> {
        self.with_state(|state| state.ui.focused()).flatten()
    }

    pub fn text_input_focused(&self) -> bool {
        self.with_state(|state| {
            state.access.field.is_some()
                || state
                    .ui
                    .focused()
                    .and_then(|node| state.ui.node(node))
                    .is_some_and(|node| node.kind == UiNodeKind::Input)
        })
        .unwrap_or(false)
    }

    pub fn caret_bounds(&self) -> Option<[f32; 4]> {
        self.with_state(|state| state.access.caret).flatten()
    }

    pub fn focused_bounds(&self) -> Option<[f32; 4]> {
        self.with_state(|state| {
            if let Some(field) = &state.access.field {
                return Some([
                    field.rect.min.x,
                    field.rect.min.y,
                    field.rect.width(),
                    field.rect.height(),
                ]);
            }
            let node = state.ui.node(state.ui.focused()?)?;
            Some([
                node.bounds.x,
                node.bounds.y,
                node.bounds.width,
                node.bounds.height,
            ])
        })
        .flatten()
    }

    pub fn blur_text_input(&self) {
        self.with_state(|state| {
            state.focus_hint = None;
            if state
                .ui
                .focused()
                .and_then(|node| state.ui.node(node))
                .is_some_and(|node| node.kind == UiNodeKind::Input)
            {
                let actions = state.ui.set_focus(None).into_iter().collect();
                remember_actions(state, actions);
            }
        });
    }

    pub fn focus_node(&self, node: u64) {
        self.with_state(|state| {
            state.keyboard_focus_visible = true;
            rebuild_current_ui(state);
            let actions = state.ui.dispatch(UiEvent::FocusRequest(node));
            remember_actions(state, actions);
            ensure_focused_visible(state);
        });
    }

    pub fn activate_node(&self, node: u64) {
        self.with_state(|state| {
            rebuild_current_ui(state);
            remember_actions(state, vec![UiAction::Activated(node)]);
        });
    }

    pub fn focused_text(&self) -> Option<String> {
        self.with_state(|state| match &state.access.field {
            Some(field) => Some(field.text.clone()),
            None => actions::focused_text(state),
        })
        .flatten()
    }

    pub fn set_focused_text(&self, text: &str) {
        self.with_state(|state| {
            if state.access.field.is_some() {
                let ctrl = egui::Modifiers::COMMAND;
                state.egui_events.push(egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: ctrl,
                });
                state.egui_events.push(if text.is_empty() {
                    egui::Event::Key {
                        key: egui::Key::Backspace,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }
                } else {
                    egui::Event::Text(text.to_owned())
                });
            } else if let Some(node) = state.ui.focused() {
                actions::edit_text(state, node, |value| *value = text.to_owned());
            }
        });
    }

    pub fn text_input(&self, text: &str) {
        self.with_state(|state| {
            let actions = state.ui.dispatch(UiEvent::TextInput(text.to_owned()));
            remember_actions(state, actions);
        });
    }
}
