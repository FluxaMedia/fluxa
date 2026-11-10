use super::*;

const WHEEL_GAIN: f32 = 1.65;

impl FluxaHost {
    pub fn scroll(&self, delta_y: f32) {
        self.with_state(|state| {
            if ambient::input(state)
                || input_layer(state) == InputLayer::DetailModal
                || card_menu::scroll_by(state, delta_y)
            {
                return;
            }
            let viewport = logical_viewport(state);
            if active_route(state) == Route::Home {
                let max = region_max(state, fluxa_ui::ScrollId::Screen);
                let offset =
                    (state.scroll_get(ScrollKey::Screen(Route::Home)) + delta_y).clamp(0.0, max);
                state.scroll_set(ScrollKey::Screen(Route::Home), offset);
            } else {
                update_screen_scroll(state, delta_y);
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
            if ambient::input(state) {
                return;
            }
            let position = state.mirror_position(Pos2::new(x, y) * (state.density / state.scale()));
            state.mouse_position = Some(position);
            if mouse_pan(state, PointerPhase::Move, position) {
                return;
            }
            let regained = state
                .focus_gained_at
                .is_some_and(|at| at.elapsed() < Duration::from_millis(400));
            if !regained {
                state.keyboard_focus_visible = false;
            }
            let over_skip = state
                .ui
                .node(fluxa_ui::NODE_PLAYER_SKIP)
                .is_some_and(|node| {
                    let bounds = node.bounds;
                    position.x >= bounds.x
                        && position.x <= bounds.x + bounds.width
                        && position.y >= bounds.y
                        && position.y <= bounds.y + bounds.height
                });
            if !over_skip && let Some(player) = state.player.as_mut() {
                player.touch();
            }
            state.egui_events.push(egui::Event::PointerMoved(position));
        });
    }

    pub fn mouse_button(&self, button: MouseButton, pressed: bool, x: f32, y: f32) {
        self.with_state(|state| {
            if ambient::pointer(
                state,
                if pressed {
                    PointerPhase::Down
                } else {
                    PointerPhase::Up
                },
            ) {
                return;
            }
            let position = state.mirror_position(Pos2::new(x, y) * (state.density / state.scale()));
            state.mouse_position = Some(position);
            state.keyboard_focus_visible = false;
            if matches!(button, MouseButton::Primary) {
                let phase = if pressed {
                    PointerPhase::Down
                } else {
                    PointerPhase::Up
                };
                if mouse_pan(state, phase, position) {
                    return;
                }
            }
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
            if ambient::input(state) {
                return;
            }
            let delta_x = if state.mirrored() { -delta_x } else { delta_x };
            let (delta_x, delta_y) = if state.modifiers.shift && delta_x == 0.0 {
                (delta_y, 0.0)
            } else {
                (delta_x, delta_y)
            };
            if card_menu::scroll_by(state, delta_y) {
                return;
            }
            state.egui_events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::ZERO,
                phase: egui::TouchPhase::Start,
                source: egui::MouseWheelSource::Unknown,
                modifiers: state.modifiers,
            });
            state.egui_events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(-delta_x, -delta_y),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Unknown,
                modifiers: state.modifiers,
            });
            if input_layer(state) == InputLayer::DetailModal {
                return;
            }
            let gain = if state.home.form_factor == UiFormFactorJson::Desktop {
                WHEEL_GAIN
            } else {
                1.0
            };
            let (delta_x, delta_y) = (delta_x * gain, delta_y * gain);
            if delta_x != 0.0
                && let Some(target) = state
                    .mouse_position
                    .and_then(|position| row_target_at(state, [position.x, position.y]))
            {
                let max = region_max(state, target.id());
                let offset = (state.scroll_get(target) + delta_x).clamp(0.0, max);
                scroll_write(state, target, offset, logical_viewport(state));
            }
            if delta_y != 0.0 {
                update_screen_scroll(state, delta_y);
            }
        });
    }

    pub fn set_modifiers(&self, modifiers: egui::Modifiers) {
        self.with_state(|state| state.modifiers = modifiers);
    }

    pub fn egui_key(&self, key: egui::Key, pressed: bool) {
        self.with_state(|state| {
            if ambient::input(state) {
                return;
            }
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
        self.peek(|state| state.copied.take()).flatten()
    }

    pub fn egui_event(&self, event: egui::Event) {
        self.with_state(|state| {
            if ambient::input(state) {
                return;
            }
            if ambient::input(state) {
                return;
            }
            if ambient::input(state) {
                return;
            }
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

    pub fn egui_navigates(&self) -> bool {
        self.peek(|state| state.welcome.is_some()).unwrap_or(false)
    }

    pub fn wants_keyboard(&self) -> bool {
        self.peek(|state| state.wants_keyboard).unwrap_or(false)
    }

    pub fn cursor(&self) -> egui::CursorIcon {
        self.peek(|state| state.cursor).unwrap_or_default()
    }

    pub fn pointer_at(&self, phase: PointerPhase, x: f32, y: f32, event_ms: u64) {
        self.with_state(|state| {
            state.pointer_event_ms = Some(event_ms);
            if phase == PointerPhase::Down {
                state.touch_anchor_ms = Some(event_ms);
            }
        });
        self.pointer(phase, x, y);
        self.with_state(|state| state.pointer_event_ms = None);
    }

    pub fn pointer(&self, phase: PointerPhase, x: f32, y: f32) {
        self.with_state(|state| {
            if ambient::pointer(state, phase) {
                return;
            }
            let ratio = state.density / state.scale();
            pointer_event(state, phase, [state.mirror_x(x * ratio), y * ratio])
        });
    }

    pub fn key_down(&self, input: KeyInput) {
        self.with_state(|state| {
            if ambient::input(state) {
                return;
            }
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
            if ambient::input(state) {
                return true;
            }
            if shortcuts::recording(state) {
                shortcuts::record(state, chord);
                return true;
            }
            shortcuts::run(state, chord, repeat)
        })
        .unwrap_or(false)
    }

    pub fn shortcut_recording(&self) -> bool {
        self.peek(|state| shortcuts::recording(state))
            .unwrap_or(false)
    }

    pub fn can_go_back(&self) -> bool {
        self.peek(|state| {
            state.player_full()
                || state.profiles.is_some()
                || state.welcome.is_some()
                || state.route != Route::Home
        })
        .unwrap_or(false)
    }

    pub fn back(&self) -> bool {
        self.with_state(|state| {
            if ambient::input(state) {
                return true;
            }
            if state
                .ui
                .focused()
                .and_then(|node| state.ui.node(node))
                .is_some_and(|node| node.kind == UiNodeKind::Input)
            {
                state.focus_hint = None;
                let actions = state.ui.set_focus(None).into_iter().collect();
                remember_actions(state, actions);
                return true;
            }
            if !state.player_full()
                && state.profiles.is_none()
                && state.welcome.is_none()
                && state.route == Route::Home
            {
                return false;
            }
            key_down(state, KeyInput::Key(Key::Back));
            true
        })
        .unwrap_or(false)
    }

    pub fn focused_node(&self) -> Option<u64> {
        self.peek(|state| state.ui.focused()).flatten()
    }

    pub fn text_input_focused(&self) -> bool {
        self.peek(|state| {
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
        self.peek(|state| {
            let inset = logical_viewport(state).side_inset;
            state.access.caret.map(|[x, y, w, h]| [x + inset, y, w, h])
        })
        .flatten()
    }

    pub fn focused_bounds(&self) -> Option<[f32; 4]> {
        self.peek(|state| {
            let inset = logical_viewport(state).side_inset;
            if let Some(field) = &state.access.field {
                return Some([
                    field.rect.min.x + inset,
                    field.rect.min.y,
                    field.rect.width(),
                    field.rect.height(),
                ]);
            }
            let node = state.ui.node(state.ui.focused()?)?;
            Some([
                node.bounds.x + inset,
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
            if card_menu::activate(state, node) || semantics::activate_click(state, node) {
                return;
            }
            rebuild_current_ui(state);
            remember_actions(state, vec![UiAction::Activated(node)]);
        });
    }

    pub fn focused_text(&self) -> Option<String> {
        self.peek(|state| match &state.access.field {
            Some(field) => Some(
                state
                    .field_text
                    .as_ref()
                    .filter(|pending| pending.id == field.id)
                    .map_or_else(|| field.text.clone(), |pending| pending.text.clone()),
            ),
            None => actions::focused_text(state),
        })
        .flatten()
    }

    pub fn focused_field_id(&self) -> u64 {
        self.peek(|state| field_id(state)).unwrap_or(0)
    }

    pub fn set_focused_text(&self, text: &str) {
        let id = self.focused_field_id();
        self.set_field_text(id, text);
    }

    pub fn set_field_text(&self, id: u64, text: &str) {
        self.with_state(|state| {
            if let Some(field) = &state.access.field {
                if field.id == id && field.text != text {
                    let end = text.chars().count();
                    state.field_text = Some(fluxa_ui::PendingField {
                        id,
                        text: text.to_owned(),
                        selection: [end, end],
                        composing: None,
                    });
                }
            } else if id != 0 && state.ui.focused() == Some(id) {
                actions::edit_text(state, id, |value| *value = text.to_owned());
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

pub(super) fn field_id(state: &RendererState) -> u64 {
    if let Some(field) = &state.access.field {
        return field.id;
    }
    state
        .ui
        .focused()
        .filter(|node| {
            state
                .ui
                .node(*node)
                .is_some_and(|node| node.kind == UiNodeKind::Input)
        })
        .unwrap_or(0)
}

pub(super) fn notify_input_focus(state: &mut RendererState) {
    let id = field_id(state);
    let edited = ime::changed(state);
    if id == state.notified_field && !edited {
        return;
    }
    state.notified_field = id;
    if let Some(listener) = super::INPUT_FOCUS.get() {
        listener();
    }
}
