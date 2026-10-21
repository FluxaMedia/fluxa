use super::*;

pub(super) fn advance_home_inertia(state: &mut RendererState) {
    let now = Instant::now();
    let elapsed = now
        .duration_since(state.scroll_animation_at)
        .min(Duration::from_millis(50));
    state.scroll_animation_at = now;
    if state.touch_start.is_some() || elapsed.is_zero() {
        return;
    }
    let Some(target) = state.active_scroll else {
        state.scroll_motion.stop();
        return;
    };
    if let Some(value) = state.scroll_motion.step(elapsed) {
        let viewport = logical_viewport(state);
        scroll_write(state, target, value, viewport);
    }
    if !state.scroll_motion.is_active() {
        state.active_scroll = None;
    }
}

fn scroll_axis(target: HomeScrollTarget) -> usize {
    match target {
        HomeScrollTarget::Vertical | HomeScrollTarget::ScreenVertical => 1,
        _ => 0,
    }
}

fn scroll_extent(target: HomeScrollTarget, viewport: Viewport) -> f32 {
    if scroll_axis(target) == 1 {
        viewport.height
    } else {
        viewport.width
    }
}

fn scroll_read(state: &RendererState, target: HomeScrollTarget, viewport: Viewport) -> (f32, f32) {
    match target {
        HomeScrollTarget::Vertical => (
            state.home.scroll_offset,
            fluxa_ui::home_scroll_max(viewport, &state.home),
        ),
        HomeScrollTarget::Horizontal(row) => (
            state
                .home
                .row_scroll_offsets
                .get(row)
                .copied()
                .unwrap_or(0.0),
            fluxa_ui::home_row_scroll_max(viewport, &state.home, row),
        ),
        HomeScrollTarget::ScreenVertical => (
            state
                .screen_scroll_offsets
                .get(&active_route(state))
                .copied()
                .unwrap_or(0.0),
            screen_scroll_max(state, viewport),
        ),
        HomeScrollTarget::FolderRow(row) => (
            state
                .folder
                .row_scroll_offsets
                .get(row)
                .copied()
                .unwrap_or(0.0),
            fluxa_ui::folder_row_scroll_max(viewport, &state.folder, row),
        ),
        HomeScrollTarget::DetailRow(row) => (
            state
                .detail
                .row_scroll_offsets
                .get(row)
                .copied()
                .unwrap_or(0.0),
            fluxa_ui::detail_row_scroll_max(row),
        ),
    }
}

fn scroll_write(
    state: &mut RendererState,
    target: HomeScrollTarget,
    value: f32,
    viewport: Viewport,
) {
    match target {
        HomeScrollTarget::Vertical => state.home.scroll_offset = value,
        HomeScrollTarget::Horizontal(row) => {
            if state.home.row_scroll_offsets.len() <= row {
                state.home.row_scroll_offsets.resize(row + 1, 0.0);
            }
            state.home.row_scroll_offsets[row] = value;
            request_home_row_load_more(state, row, viewport);
        }
        HomeScrollTarget::ScreenVertical => {
            let route = active_route(state);
            state.screen_scroll_offsets.insert(route, value);
        }
        HomeScrollTarget::FolderRow(row) => {
            let offsets = &mut state.folder.row_scroll_offsets;
            if offsets.len() <= row {
                offsets.resize(row + 1, 0.0);
            }
            offsets[row] = value;
        }
        HomeScrollTarget::DetailRow(row) => {
            if let Some(offset) = state.detail.row_scroll_offsets.get_mut(row) {
                *offset = value;
            }
        }
    }
}

fn track_touch(state: &mut RendererState, now: Instant, position: [f32; 2]) {
    let seconds = now.duration_since(state.touch_epoch).as_secs_f32();
    state.touch_tracker.add(seconds, position);
}

fn scroll_feel(state: &RendererState) -> fluxa_ui::ScrollFeel {
    fluxa_ui::ScrollFeel::for_platform(state.home.platform)
}

fn scroll_drag(state: &mut RendererState, target: HomeScrollTarget, delta: f32) {
    let viewport = logical_viewport(state);
    let (offset, max) = scroll_read(state, target, viewport);
    let next = fluxa_ui::scroll_drag(
        scroll_feel(state),
        offset,
        delta,
        max,
        scroll_extent(target, viewport),
    );
    scroll_write(state, target, next, viewport);
}

fn scroll_release(state: &mut RendererState, position: [f32; 2]) {
    let now = Instant::now();
    track_touch(state, now, position);
    let Some(target) = state.active_scroll else {
        return;
    };
    let viewport = logical_viewport(state);
    let (offset, max) = scroll_read(state, target, viewport);
    let velocity = -state.touch_tracker.fling_velocity(scroll_axis(target));
    let feel = scroll_feel(state);
    let density = state.density;
    state
        .scroll_motion
        .release(feel, offset, velocity, max, density);
    state.scroll_animation_at = now;
    if !state.scroll_motion.is_active() {
        state.active_scroll = None;
    }
}

pub(super) fn request_home_row_load_more(
    state: &mut RendererState,
    content_row_index: usize,
    viewport: Viewport,
) {
    let offset = state
        .home
        .row_scroll_offsets
        .get(content_row_index)
        .copied()
        .unwrap_or(0.0);
    request_home_row_load_more_at(state, content_row_index, viewport, offset);
}

pub(super) fn request_home_row_load_more_at(
    state: &mut RendererState,
    content_row_index: usize,
    viewport: Viewport,
    offset: f32,
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
    match active_route(state) {
        Route::Player => state
            .player
            .as_ref()
            .map_or(0.0, |player| player.sources_scroll_max),
        Route::Library => fluxa_ui::library_scroll_max(viewport, &state.library, state.library_tab),
        Route::Discover => fluxa_ui::discover_scroll_max(viewport, &state.discover),
        Route::Folder => fluxa_ui::folder_scroll_max(viewport, &state.folder, state.folder_tab),
        Route::Calendar => fluxa_ui::calendar_scroll_max(viewport, &state.calendar),
        Route::Detail => fluxa_ui::detail_scroll_max(viewport, &state.detail),
        Route::Settings => fluxa_ui::settings_scroll_max(viewport, &state.settings),
        _ => 0.0,
    }
}

pub(super) fn update_screen_scroll(state: &mut RendererState, delta: f32, viewport: Viewport) {
    let max_offset = screen_scroll_max(state, viewport);
    let offset = state
        .screen_scroll_offsets
        .entry(active_route(state))
        .or_default();
    *offset = (*offset + delta).clamp(0.0, max_offset);
}

pub(super) fn logical_viewport(state: &RendererState) -> Viewport {
    Viewport::new(
        (state.size[0] as f32 / state.scale()).round().max(1.0) as u32,
        (state.size[1] as f32 / state.scale()).round().max(1.0) as u32,
        state.home.form_factor.into(),
    )
    .with_platform(state.home.platform)
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
    if state.profiles.is_some()
        || state.welcome.is_some()
        || state.player_full()
        || state.detail_modal
    {
        if state
            .player
            .as_ref()
            .is_some_and(|player| player.sources_scroll_max > 0.0)
        {
            drag_sources(state, phase, position);
        }
        let pos = Pos2::new(position[0], position[1]);
        let button = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: state.modifiers,
        };
        match phase {
            PointerPhase::Move => state.egui_events.push(egui::Event::PointerMoved(pos)),
            PointerPhase::Down => {
                state.egui_events.push(egui::Event::PointerMoved(pos));
                state.egui_events.push(button(true));
            }
            PointerPhase::Up => {
                state.egui_events.push(button(false));
                state.egui_events.push(egui::Event::PointerGone);
            }
        }
        return;
    }
    let viewport = logical_viewport(state);
    if phase == PointerPhase::Down
        && !state.player_full()
        && state.profiles.is_none()
        && state.welcome.is_none()
        && viewport.is_compact()
        && fluxa_ui::mobile_nav_rect(viewport).contains(Pos2::new(position[0], position[1]))
    {
        state.keyboard_focus_visible = false;
        state.nav_dragging = true;
        fluxa_ui::set_mobile_nav_drag(Some(fluxa_ui::mobile_nav_slot_center(
            viewport,
            position[0],
        )));
        state.touch_start = Some(position);
        return;
    }
    if state.nav_dragging {
        match phase {
            PointerPhase::Move => {
                if state
                    .touch_start
                    .is_some_and(|start| (position[0] - start[0]).abs() > 12.0)
                {
                    state.touch_start = None;
                }
                if state.touch_start.is_none() {
                    fluxa_ui::set_mobile_nav_drag(Some(position[0]));
                }
            }
            _ => {
                state.nav_dragging = false;
                state.touch_start = None;
                fluxa_ui::set_mobile_nav_pending(fluxa_ui::mobile_nav_slot_center(
                    viewport,
                    position[0],
                ));
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
        let caught = state.scroll_motion.is_active() && state.active_scroll.is_some();
        state.touch_start = Some(position);
        state.touch_down_at = Some(now);
        state.touch_last = Some(position);
        state.touch_last_at = Some(now);
        state.touch_epoch = now;
        state.touch_tracker.clear();
        track_touch(state, now, position);
        state.touch_scrolled = caught;
        state.scroll_motion.stop();
        state.scroll_animation_at = now;
        if caught {
            return;
        }
        state.active_scroll = None;
    } else if phase == PointerPhase::Move {
        let now = Instant::now();
        if let (Some(start), Some(last)) = (state.touch_start, state.touch_last) {
            let total_x = start[0] - position[0];
            let total_y = start[1] - position[1];
            if total_x.hypot(total_y) > scroll_feel(state).touch_slop() {
                state.touch_scrolled = true;
            }
            if state.active_scroll.is_none() && state.touch_scrolled {
                let viewport = logical_viewport(state);
                if active_route(state) == Route::Home {
                    if total_x.abs() > total_y.abs() * 1.15 {
                        state.active_scroll =
                            fluxa_ui::home_row_at_y(viewport, &state.home, start[1])
                                .map(HomeScrollTarget::Horizontal);
                    }
                    if state.active_scroll.is_none() {
                        state.active_scroll = Some(HomeScrollTarget::Vertical);
                    }
                } else {
                    if state.route == Route::Folder && total_x.abs() > total_y.abs() * 1.15 {
                        let scroll_y = state
                            .screen_scroll_offsets
                            .get(&Route::Folder)
                            .copied()
                            .unwrap_or(0.0);
                        state.active_scroll =
                            fluxa_ui::folder_row_at_y(viewport, &state.folder, scroll_y, start[1])
                                .map(HomeScrollTarget::FolderRow);
                    }
                    if !state.player_full()
                        && state.route == Route::Detail
                        && total_x.abs() > total_y.abs() * 1.15
                    {
                        state.active_scroll = fluxa_ui::detail_row_at_y(
                            viewport.with_scroll_y(
                                state
                                    .screen_scroll_offsets
                                    .get(&Route::Detail)
                                    .copied()
                                    .unwrap_or(0.0),
                            ),
                            &state.detail,
                            start[1],
                        )
                        .map(HomeScrollTarget::DetailRow);
                    }
                    if state.active_scroll.is_none() {
                        state.active_scroll = Some(HomeScrollTarget::ScreenVertical);
                    }
                }
            }
            if let Some(target) = state.active_scroll {
                let axis = scroll_axis(target);
                let delta = last[axis] - position[axis];
                if delta != 0.0 {
                    scroll_drag(state, target, delta);
                }
            }
            track_touch(state, now, position);
        }
        state.touch_last = Some(position);
        state.touch_last_at = Some(now);
    }
    if phase == PointerPhase::Up {
        let was_scroll = state.touch_scrolled;
        if was_scroll {
            scroll_release(state, position);
        }
        state.touch_down_at = None;
        state.touch_start = None;
        state.touch_last = None;
        state.touch_last_at = None;
        state.touch_scrolled = false;
        state.touch_tracker.clear();
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

pub(super) fn hide_pointer(state: &mut RendererState) {
    if !state.keyboard_focus_visible {
        state.egui_events.push(egui::Event::PointerGone);
    }
    state.keyboard_focus_visible = true;
}

pub(super) fn key_down(state: &mut RendererState, input: KeyInput) {
    if state.player_full() && matches!(player::key(state, input), player::KeyOutcome::Handled) {
        return;
    }
    if card_menu::key(state, input) || detail_modal::key(state, input) {
        return;
    }
    if matches!(input, KeyInput::Key(Key::Back | Key::Escape)) && downloads::close_group(state) {
        return;
    }
    if matches!(input, KeyInput::Gamepad(GamepadButton::West)) {
        hide_pointer(state);
        card_menu::open_from_focus(state);
        return;
    }
    if !matches!(input, KeyInput::Key(Key::Back | Key::Escape)) {
        hide_pointer(state);
    }
    if state.route == Route::Shorts
        && !state.player_full()
        && state.profiles.is_none()
        && state.welcome.is_none()
    {
        let delta = match input {
            KeyInput::Key(Key::Up) | KeyInput::Gamepad(GamepadButton::DPadUp) => Some(-1),
            KeyInput::Key(Key::Down) | KeyInput::Gamepad(GamepadButton::DPadDown) => Some(1),
            _ => None,
        };
        if let Some(delta) = delta {
            shorts::step(state, delta);
            return;
        }
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
    let vertical = matches!(
        input,
        KeyInput::Key(Key::Up | Key::Down)
            | KeyInput::Gamepad(GamepadButton::DPadUp | GamepadButton::DPadDown)
    );
    let previous = state.ui.focused();
    let up = matches!(
        input,
        KeyInput::Key(Key::Up) | KeyInput::Gamepad(GamepadButton::DPadUp)
    );
    if up
        && state.route == Route::Home
        && previous.is_some_and(|id| id >= NODE_CARD_BASE)
        && let Some(action) = state.ui.move_up_where(|id| id >= NODE_CARD_BASE)
    {
        remember_actions(state, vec![action]);
        focus_row_start(state, previous);
        ensure_focused_visible(state);
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
    if vertical {
        focus_row_start(state, previous);
    }
    ensure_focused_visible(state);
}

fn focus_row_start(state: &mut RendererState, previous: Option<u64>) {
    if state.route != Route::Home {
        return;
    }
    let Some(focused) = state.ui.focused().filter(|id| *id >= NODE_CARD_BASE) else {
        return;
    };
    let card = (focused - NODE_CARD_BASE) as usize;
    let Some(row) = fluxa_ui::home_row_of_card(&state.home, card) else {
        return;
    };
    let same_row = previous
        .filter(|id| *id >= NODE_CARD_BASE)
        .and_then(|id| fluxa_ui::home_row_of_card(&state.home, (id - NODE_CARD_BASE) as usize))
        == Some(row);
    if same_row {
        return;
    }
    let mut first = card;
    while first > 0 && fluxa_ui::home_row_of_card(&state.home, first - 1) == Some(row) {
        first -= 1;
    }
    if first != card
        && let Some(action) = state.ui.set_focus(Some(NODE_CARD_BASE + first as u64))
    {
        remember_actions(state, vec![action]);
    }
}

pub(super) fn active_route(state: &RendererState) -> Route {
    if state.profiles.is_some() || state.welcome.is_some() {
        Route::Profiles
    } else if state.player_full() {
        Route::Player
    } else {
        state.route
    }
}

pub(super) fn apply_pointer_results(state: &mut RendererState, route: Route, layout: &HomeLayout) {
    if let (Some(node), Some(value)) = (layout.text_input_node, layout.text_input.as_ref()) {
        set_text_value(state, node, value);
    }
    if let Some(value) = fluxa_ui::take_nav_search() {
        set_text_value(state, fluxa_ui::NODE_SIDEBAR_SEARCH, &value);
    }
    if let Some((key, value)) = layout.filter_change.as_ref()
        && key.starts_with("library")
        && route == Route::Library
    {
        apply_choice(state, route, key, value.clone());
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

pub(super) fn apply_choice(state: &mut RendererState, route: Route, key: &str, value: String) {
    match key {
        "downloadDir" => {
            state.storage_polled = None;
            state
                .pending_native_actions
                .push(NativeAction::SettingsChange {
                    key: key.to_owned(),
                    value: Value::String(value),
                });
        }
        "libraryStatus" if route == Route::Library => {
            if let Some(tab) = LibraryTab::ALL
                .into_iter()
                .find(|tab| tab.core_tab_key() == value)
            {
                state.library_tab = tab;
                state.screen_scroll_offsets.remove(&Route::Library);
                refresh_library_view(state);
            }
        }
        "libraryType" if route == Route::Library => {
            state.library_type = value;
            refresh_library_view(state);
        }
        "librarySort" if route == Route::Library => {
            state.library_sort = value;
            refresh_library_view(state);
        }
        "discover:contentType" if route == Route::Discover => {
            state.discover.content_type = value.clone();
            state
                .pending_native_actions
                .push(NativeAction::DiscoverType {
                    content_type: value,
                });
        }
        "detail:season" if route == Route::Detail => {
            state.detail.selected_season = value.parse().ok();
            state.detail.row_scroll_offsets[1] = 0.0;
            reset_ui(state);
        }
        "discover:catalog" | "discover:extra" if route == Route::Discover => {
            if key == "discover:catalog" {
                state.discover.selected_catalog_key = value;
            } else {
                state.discover.selected_extra_value = value;
            }
            let action = NativeAction::DiscoverFilters {
                content_type: state.discover.content_type.clone(),
                catalog_key: state.discover.selected_catalog_key.clone(),
                extra_name: state.discover.selected_extra_name.clone(),
                extra_value: state.discover.selected_extra_value.clone(),
                query: state.discover.query.clone(),
            };
            state.pending_native_actions.push(action);
        }
        _ => {}
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
            schedule_discover_search(state);
        }
        fluxa_ui::NODE_SIDEBAR_SEARCH if state.discover.query != value => {
            state.discover.query = value.to_owned();
            search_from_sidebar(state);
        }
        fluxa_ui::NODE_SETTINGS_ADDON_URL => state.settings.addon_url = value.to_owned(),
        fluxa_ui::NODE_SETTINGS_SEARCH => state.settings.search = value.to_owned(),
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL => state.settings.plugin_url = value.to_owned(),
        fluxa_ui::NODE_SETTINGS_WATCH_CODE => {
            state.settings.watch_code = value
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .take(6)
                .collect::<String>()
                .to_uppercase();
        }
        node => {
            if let Some(index) = fluxa_ui::poster_field(node) {
                state.settings.poster_fields[index] = value.to_owned();
            }
            if let Some(index) = fluxa_ui::server_input(node) {
                state.settings.server_fields[index] = value.to_owned();
            }
            if let Some(index) = fluxa_ui::collection_input(node) {
                state.settings.collection_fields[index] = value.to_owned();
            }
            if let Some(index) = fluxa_ui::badge_input(node) {
                state.settings.badge_fields[index] = value.to_owned();
            }
        }
    }
}

const SEARCH_DEBOUNCE: Duration = Duration::from_millis(350);

pub(super) fn schedule_discover_search(state: &mut RendererState) {
    state.search_due = Some(Instant::now() + SEARCH_DEBOUNCE);
}

pub(super) fn tick_discover_search(state: &mut RendererState) {
    if state.search_due.is_some_and(|due| due <= Instant::now()) {
        state.search_due = None;
        request_discover_search(state);
    }
}

pub(super) fn search_from_sidebar(state: &mut RendererState) {
    schedule_discover_search(state);
    if state.route != Route::Discover {
        state.pending_native_actions.push(NativeAction::Navigate {
            destination: Route::Discover,
        });
    }
}

pub(super) fn request_discover_search(state: &mut RendererState) {
    recent_searches::refresh(state);
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

fn drag_sources(state: &mut RendererState, phase: PointerPhase, position: [f32; 2]) {
    let now = Instant::now();
    match phase {
        PointerPhase::Down => {
            let caught = state.scroll_motion.is_active();
            state.touch_start = Some(position);
            state.touch_last = Some(position);
            state.touch_last_at = Some(now);
            state.touch_scrolled = caught;
            state.touch_epoch = now;
            state.touch_tracker.clear();
            track_touch(state, now, position);
            state.scroll_motion.stop();
            state.active_scroll = Some(HomeScrollTarget::ScreenVertical);
            state.scroll_animation_at = now;
        }
        PointerPhase::Move => {
            let (Some(start), Some(last)) = (state.touch_start, state.touch_last) else {
                return;
            };
            let moved = (start[0] - position[0]).hypot(start[1] - position[1]);
            if moved > scroll_feel(state).touch_slop() {
                state.touch_scrolled = true;
            }
            if !state.touch_scrolled {
                return;
            }
            scroll_drag(
                state,
                HomeScrollTarget::ScreenVertical,
                last[1] - position[1],
            );
            track_touch(state, now, position);
            state.touch_last = Some(position);
            state.touch_last_at = Some(now);
        }
        PointerPhase::Up => {
            if state.touch_scrolled {
                scroll_release(state, position);
            }
            state.touch_start = None;
            state.touch_last = None;
            state.touch_last_at = None;
            state.touch_scrolled = false;
            state.touch_tracker.clear();
        }
    }
}
