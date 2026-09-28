use super::*;

pub(super) fn advance_home_inertia(state: &mut RendererState) {
    let now = Instant::now();
    let elapsed = now
        .duration_since(state.scroll_animation_at)
        .as_secs_f32()
        .clamp(0.0, 0.05);
    state.scroll_animation_at = now;
    if state.touch_start.is_some() || elapsed == 0.0 || state.scroll_velocity.abs() < 14.0 {
        if state.scroll_velocity.abs() < 14.0 {
            state.scroll_velocity = 0.0;
            state.active_scroll = None;
        }
        return;
    }
    let viewport = Viewport::new(
        (state.size[0] as f32 / state.scale()).round().max(1.0) as u32,
        (state.size[1] as f32 / state.scale()).round().max(1.0) as u32,
        state.home.form_factor.into(),
    )
    .with_safe_bottom(state.safe_bottom);
    let movement = state.scroll_velocity * elapsed;
    match state.active_scroll {
        Some(HomeScrollTarget::Vertical) => {
            let max_offset = fluxa_ui::home_scroll_max(viewport, &state.home);
            let next = (state.home.scroll_offset + movement).clamp(0.0, max_offset);
            if (next - state.home.scroll_offset).abs() < 0.01 {
                state.scroll_velocity = 0.0;
            } else {
                state.home.scroll_offset = next;
            }
        }
        Some(HomeScrollTarget::Horizontal(row_index)) => {
            if state.home.row_scroll_offsets.len() <= row_index {
                state.home.row_scroll_offsets.resize(row_index + 1, 0.0);
            }
            let max_offset = fluxa_ui::home_row_scroll_max(viewport, &state.home, row_index);
            let offset = &mut state.home.row_scroll_offsets[row_index];
            let next = (*offset + movement).clamp(0.0, max_offset);
            if (next - *offset).abs() < 0.01 {
                state.scroll_velocity = 0.0;
            } else {
                *offset = next;
            }
            request_home_row_load_more(state, row_index, viewport);
        }
        Some(HomeScrollTarget::ScreenVertical) => {
            let max_offset = screen_scroll_max(state, viewport);
            let offset = state.screen_scroll_offsets.entry(state.route).or_default();
            let next = (*offset + movement).clamp(0.0, max_offset);
            if (next - *offset).abs() < 0.01 {
                state.scroll_velocity = 0.0;
            } else {
                *offset = next;
            }
        }
        None => state.scroll_velocity = 0.0,
    }
    // Exponential friction produces a predictable inertial tail independent
    // of the render loop's exact frame cadence.
    // Android's native scroller carries substantially more momentum than a
    // web-style smooth-scroll tail. Keep a long, quick decay so a firm flick
    // can traverse multiple viewports before coming to rest.
    state.scroll_velocity *= (-1.6 * elapsed).exp();
}

pub(super) fn request_home_row_load_more(
    state: &mut RendererState,
    content_row_index: usize,
    viewport: Viewport,
) {
    if state.route != Route::Home {
        return;
    }
    let has_continue_row = !state.home.cards.is_empty();
    let Some(row_index) = content_row_index.checked_sub(usize::from(has_continue_row)) else {
        return;
    };
    let Some(row) = state.home.rows.get(row_index) else {
        return;
    };
    let Some(row_id) = row.id.as_deref().map(str::trim).filter(|id| !id.is_empty()) else {
        return;
    };
    let card_count = row.cards.len();
    if !row.can_load_more
        || card_count == 0
        || state.load_more_requested_counts.get(row_id) == Some(&card_count)
    {
        return;
    }
    let scroll_index = row_index + usize::from(has_continue_row);
    let max_offset = fluxa_ui::home_row_scroll_max(viewport, &state.home, scroll_index);
    let visible_width = (viewport.width * 0.9).max(1.0);
    let card_pitch = (max_offset + visible_width) / card_count as f32;
    let threshold = (visible_width * 2.0).max(card_pitch * 6.0).min(max_offset);
    let offset = state
        .home
        .row_scroll_offsets
        .get(scroll_index)
        .copied()
        .unwrap_or(0.0);
    if offset < (max_offset - threshold).max(0.0) {
        return;
    }
    state
        .load_more_requested_counts
        .insert(row_id.to_owned(), card_count);
    if let Some(category) = row.catalog_page.as_ref() {
        let content_type = category
            .get("contentType")
            .or_else(|| category.get("type"))
            .and_then(Value::as_str)
            .filter(|value| matches!(*value, "movie" | "series"))
            .unwrap_or("movie");
        let transport_url = category
            .get("addonTransportUrl")
            .or_else(|| category.get("transportUrl"))
            .cloned()
            .unwrap_or(Value::Null);
        let catalog_id = category
            .get("catalogId")
            .or_else(|| category.get("id"))
            .cloned()
            .unwrap_or(Value::Null);
        let skip =
            json!(category.get("skip").and_then(Value::as_i64).unwrap_or(0) + card_count as i64);
        let remote_sources = category
            .get("remoteSources")
            .or_else(|| category.get("remoteSource"))
            .cloned()
            .unwrap_or(Value::Null);
        let command = json!({
            "type": "catalogPageRequested",
            "categoryId": row_id,
            "transportUrl": transport_url,
            "contentType": content_type,
            "catalogId": catalog_id,
            "skip": skip,
            "genre": category.get("addonGenre").or_else(|| category.get("genre")),
            "remoteSource": remote_sources,
            "profile": Value::Null,
        });
        state
            .pending_native_actions
            .push(NativeAction::CoreCommand { command });
    } else {
        state.pending_native_actions.push(NativeAction::LoadMore {
            row_id: row_id.to_owned(),
        });
    }
}

pub(super) fn screen_scroll_max(state: &RendererState, viewport: Viewport) -> f32 {
    match state.route {
        Route::Library => fluxa_ui::library_scroll_max(viewport, &state.library, state.library_tab),
        Route::Discover => fluxa_ui::discover_scroll_max(viewport, &state.discover),
        Route::Calendar => fluxa_ui::calendar_scroll_max(viewport, &state.calendar),
        Route::Detail => fluxa_ui::detail_scroll_max(viewport, &state.detail),
        Route::Settings => fluxa_ui::settings_scroll_max(viewport, &state.settings),
        _ => 0.0,
    }
}

pub(super) fn update_screen_scroll(state: &mut RendererState, delta: f32, viewport: Viewport) {
    let max_offset = screen_scroll_max(state, viewport);
    let offset = state.screen_scroll_offsets.entry(state.route).or_default();
    *offset = (*offset + delta).clamp(0.0, max_offset);
}

pub(super) fn logical_viewport(state: &RendererState) -> Viewport {
    Viewport::new(
        (state.size[0] as f32 / state.scale()).round().max(1.0) as u32,
        (state.size[1] as f32 / state.scale()).round().max(1.0) as u32,
        state.home.form_factor.into(),
    )
    .with_safe_bottom(state.safe_bottom)
}

pub(super) fn pointer_event(state: &mut RendererState, phase: PointerPhase, position: [f32; 2]) {
    if card_menu::pointer(state, phase, position) {
        if phase == PointerPhase::Up {
            state.touch_start = None;
            state.touch_last = None;
            state.touch_down_at = None;
            state.touch_scrolled = false;
        }
        return;
    }
    let viewport = logical_viewport(state);
    if phase == PointerPhase::Down
        && state.player.is_none()
        && state.profiles.is_none()
        && viewport.is_compact()
        && fluxa_ui::mobile_nav_rect(viewport).contains(Pos2::new(position[0], position[1]))
    {
        state.keyboard_focus_visible = false;
        state.nav_dragging = true;
        let bar = fluxa_ui::mobile_nav_rect(viewport);
        let slot = bar.width() / 5.0;
        let center = bar.left() + slot * (((position[0] - bar.left()) / slot).floor().clamp(0.0, 4.0) + 0.5);
        fluxa_ui::set_mobile_nav_drag(Some(center));
        state.touch_start = Some(position);
        return;
    }
    if state.nav_dragging {
        match phase {
            PointerPhase::Move => {
                if state.touch_start.is_some_and(|start| (position[0] - start[0]).abs() > 12.0) {
                    state.touch_start = None;
                }
                if state.touch_start.is_none() {
                    fluxa_ui::set_mobile_nav_drag(Some(position[0]));
                }
            }
            _ => {
                state.nav_dragging = false;
                state.touch_start = None;
                let bar = fluxa_ui::mobile_nav_rect(viewport);
                let slot = bar.width() / 5.0;
                fluxa_ui::set_mobile_nav_pending(
                    bar.left() + slot * (((position[0] - bar.left()) / slot).floor().clamp(0.0, 4.0) + 0.5),
                );
                let node = fluxa_ui::mobile_nav_node_at(viewport, position[0]);
                rebuild_current_ui(state);
                remember_actions(state, vec![UiAction::Activated(node)]);
            }
        }
        return;
    }
    if phase == PointerPhase::Down {
        state.keyboard_focus_visible = false;
        let now = Instant::now();
        state.touch_start = Some(position);
        state.touch_down_at = Some(now);
        state.touch_last = Some(position);
        state.touch_last_at = Some(now);
        state.touch_velocity_samples.clear();
        state.touch_velocity_samples.push_back((now, position));
        state.touch_scrolled = false;
        state.active_scroll = None;
        state.scroll_velocity = 0.0;
        state.scroll_animation_at = now;
    } else if phase == PointerPhase::Move {
        let now = Instant::now();
        if let (Some(start), Some(last)) = (state.touch_start, state.touch_last) {
            let total_x = start[0] - position[0];
            let total_y = start[1] - position[1];
            if total_x.hypot(total_y) > 8.0 {
                state.touch_scrolled = true;
            }
            if state.active_scroll.is_none() && state.touch_scrolled {
                let viewport = logical_viewport(state);
                if state.route == Route::Home {
                    if total_x.abs() > total_y.abs() * 1.15 {
                        state.active_scroll =
                            fluxa_ui::home_row_at_y(viewport, &state.home, start[1])
                                .map(HomeScrollTarget::Horizontal);
                    }
                    if state.active_scroll.is_none() {
                        state.active_scroll = Some(HomeScrollTarget::Vertical);
                    }
                } else {
                    state.active_scroll = Some(HomeScrollTarget::ScreenVertical);
                }
            }
            let sample_seconds = state
                .touch_last_at
                .map(|last_at| now.duration_since(last_at).as_secs_f32())
                .unwrap_or(1.0 / 60.0)
                .clamp(1.0 / 240.0, 0.08);
            let delta = match state.active_scroll {
                Some(HomeScrollTarget::Vertical) => last[1] - position[1],
                Some(HomeScrollTarget::Horizontal(_)) => last[0] - position[0],
                Some(HomeScrollTarget::ScreenVertical) => last[1] - position[1],
                None => 0.0,
            };
            if delta.abs() > 0.25 {
                let viewport = logical_viewport(state);
                match state.active_scroll {
                    Some(HomeScrollTarget::Vertical) => {
                        let max_offset = fluxa_ui::home_scroll_max(viewport, &state.home);
                        state.home.scroll_offset =
                            (state.home.scroll_offset + delta).clamp(0.0, max_offset);
                    }
                    Some(HomeScrollTarget::Horizontal(row_index)) => {
                        if state.home.row_scroll_offsets.len() <= row_index {
                            state.home.row_scroll_offsets.resize(row_index + 1, 0.0);
                        }
                        let max_offset =
                            fluxa_ui::home_row_scroll_max(viewport, &state.home, row_index);
                        let offset = &mut state.home.row_scroll_offsets[row_index];
                        *offset = (*offset + delta).clamp(0.0, max_offset);
                    }
                    Some(HomeScrollTarget::ScreenVertical) => {
                        update_screen_scroll(state, delta, viewport);
                    }
                    None => {}
                }
                if let Some(HomeScrollTarget::Horizontal(row_index)) = state.active_scroll {
                    request_home_row_load_more(state, row_index, viewport);
                }
                state.touch_velocity_samples.push_back((now, position));
                while state
                    .touch_velocity_samples
                    .front()
                    .is_some_and(|(sample_at, _)| {
                        now.duration_since(*sample_at).as_secs_f32() > 0.12
                    })
                    && state.touch_velocity_samples.len() > 2
                {
                    state.touch_velocity_samples.pop_front();
                }
                let velocity = state
                    .touch_velocity_samples
                    .front()
                    .zip(state.touch_velocity_samples.back())
                    .map(|((first_at, first), (last_at, last))| {
                        let seconds = last_at.duration_since(*first_at).as_secs_f32();
                        if seconds > 0.001 {
                            match state.active_scroll {
                                Some(HomeScrollTarget::Horizontal(_)) => {
                                    (first[0] - last[0]) / seconds
                                }
                                Some(
                                    HomeScrollTarget::Vertical | HomeScrollTarget::ScreenVertical,
                                ) => (first[1] - last[1]) / seconds,
                                None => 0.0,
                            }
                        } else {
                            delta / sample_seconds
                        }
                    })
                    .unwrap_or(delta / sample_seconds);
                // Keep the recent-window velocity (Compose VelocityTracker
                // style) so a firm fling remains materially faster than a
                // slow drag even when the last MotionEvent is noisy.
                // Recent-window pointer velocity underestimates the release
                // velocity on touchscreens because MOVE events are sampled
                // sparsely. Calibrate toward Android OverScroller's spline
                // distance while retaining a hard bound for noisy events.
                state.scroll_velocity = (velocity * 2.5).clamp(-14_000.0, 14_000.0);
                state.scroll_animation_at = now;
            }
        }
        state.touch_last = Some(position);
        state.touch_last_at = Some(now);
    }
    if phase == PointerPhase::Up {
        let was_scroll = state.touch_scrolled;
        state.touch_down_at = None;
        state.touch_start = None;
        state.touch_last = None;
        state.touch_last_at = None;
        state.touch_scrolled = false;
        state.touch_velocity_samples.clear();
        if was_scroll {
            return;
        }
    }
    let event = match phase {
        PointerPhase::Move => UiEvent::PointerMove { position },
        PointerPhase::Down => UiEvent::PointerDown {
            position,
            button: PointerButton::Primary,
        },
        PointerPhase::Up => UiEvent::PointerUp {
            position,
            button: PointerButton::Primary,
        },
    };
    let actions = state.ui.dispatch(event);
    remember_actions(state, actions);
}

pub(super) fn key_down(state: &mut RendererState, input: KeyInput) {
    if state.player.is_some() && matches!(player::key(state, input), player::KeyOutcome::Handled) {
        return;
    }
    if card_menu::key(state, input) {
        return;
    }
    if matches!(input, KeyInput::Gamepad(GamepadButton::West)) {
        state.keyboard_focus_visible = true;
        card_menu::open_from_focus(state);
        return;
    }
    if !matches!(input, KeyInput::Key(Key::Back | Key::Escape)) {
        state.keyboard_focus_visible = true;
    }
    rebuild_current_ui(state);
    if matches!(input, KeyInput::Backspace)
        && let Some(node) = state.ui.focused()
        && edit_text(state, node, |text| {
            text.pop();
        })
    {
        return;
    }
    let actions = match input {
        KeyInput::Gamepad(button) => state.ui.dispatch(UiEvent::GamepadButton {
            button,
            pressed: true,
        }),
        KeyInput::Key(key) => state.ui.dispatch(UiEvent::KeyDown(key)),
        KeyInput::Backspace => return,
    };
    remember_actions(state, actions);
    ensure_focused_visible(state);
}

pub(super) fn active_route(state: &RendererState) -> Route {
    if state.profiles.is_some() {
        Route::Profiles
    } else if state.player.is_some() {
        Route::Player
    } else {
        state.route
    }
}

pub(super) fn apply_pointer_results(state: &mut RendererState, route: Route, layout: &HomeLayout) {
    if let (Some(node), Some(value)) = (layout.text_input_node, layout.text_input.as_ref()) {
        set_text_value(state, node, value);
    }
    if let Some((key, value)) = layout.filter_change.as_ref()
        && key == "librarySort"
        && route == Route::Library
    {
        state.library_sort = value.clone();
        refresh_library_view(state);
    }
    if let Some((key, value)) = layout.setting_change.as_ref() {
        state
            .pending_native_actions
            .push(NativeAction::SettingsChange {
                key: key.clone(),
                value: value.clone(),
            });
    }
    if let Some(node) = layout.activated
        && state.card_menu.is_none()
    {
        rebuild_current_ui(state);
        remember_actions(state, vec![UiAction::Activated(node)]);
    }
}

pub(super) fn set_text_value(state: &mut RendererState, node: u64, value: &str) {
    match node {
        fluxa_ui::NODE_LIBRARY_SEARCH if state.library_query != value => {
            state.library_query = value.to_owned();
            refresh_library_view(state);
        }
        fluxa_ui::NODE_DISCOVER_SEARCH if state.discover.query != value => {
            state.discover.query = value.to_owned();
            request_discover_search(state);
        }
        fluxa_ui::NODE_SETTINGS_ADDON_URL => state.settings.addon_url = value.to_owned(),
        fluxa_ui::NODE_SETTINGS_SEARCH => state.settings.search = value.to_owned(),
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL => state.settings.plugin_url = value.to_owned(),
        node => {
            if let Some(index) = fluxa_ui::poster_field(node) {
                state.settings.poster_fields[index] = value.to_owned();
            }
        }
    }
}

pub(super) fn request_discover_search(state: &mut RendererState) {
    let query = state.discover.query.trim();
    if query.chars().count() == 1 {
        return;
    }
    let command = json!({
        "type": "searchRequested",
        "query": query,
        "language": state.discover.language,
    });
    state
        .pending_native_actions
        .push(NativeAction::CoreCommand { command });
}
