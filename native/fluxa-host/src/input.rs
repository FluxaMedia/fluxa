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

fn scroll_extent(key: ScrollKey, viewport: Viewport) -> f32 {
    if key.axis() == 1 {
        viewport.height
    } else {
        viewport.width
    }
}

fn scroll_key(state: &RendererState, id: fluxa_ui::ScrollId) -> ScrollKey {
    match id {
        fluxa_ui::ScrollId::Screen => ScrollKey::Screen(active_route(state)),
        fluxa_ui::ScrollId::Row(row) => ScrollKey::Row(state.route, row),
        fluxa_ui::ScrollId::Chips => ScrollKey::Chips,
        fluxa_ui::ScrollId::Strip(name) => ScrollKey::Strip(name),
    }
}

pub(super) fn region_max(state: &RendererState, id: fluxa_ui::ScrollId) -> f32 {
    let (route, regions) = &state.scroll_regions;
    if *route != active_route(state) && *route != state.route {
        return 0.0;
    }
    regions
        .iter()
        .find(|region| region.id == id)
        .map_or(0.0, |region| region.max)
}

fn scroll_read(state: &RendererState, key: ScrollKey) -> (f32, f32) {
    (state.scroll_get(key), region_max(state, key.id()))
}

pub(super) fn scroll_write(
    state: &mut RendererState,
    key: ScrollKey,
    value: f32,
    viewport: Viewport,
) {
    state.scroll_set(key, value);
    if let ScrollKey::Row(Route::Home, row) = key {
        request_home_row_load_more(state, row, viewport);
    }
}

fn track_touch(state: &mut RendererState, now: Instant, position: [f32; 2]) {
    let seconds = match (state.pointer_event_ms, state.touch_anchor_ms) {
        (Some(at), Some(anchor)) => at.saturating_sub(anchor) as f32 / 1000.0,
        _ => now.duration_since(state.touch_epoch).as_secs_f32(),
    };
    state.touch_tracker.add(seconds, position);
}

fn scroll_feel(state: &RendererState) -> fluxa_ui::ScrollFeel {
    fluxa_ui::ScrollFeel::for_platform(state.home.platform)
}

fn scroll_drag(state: &mut RendererState, target: ScrollKey, delta: f32) {
    let viewport = logical_viewport(state);
    let (offset, max) = scroll_read(state, target);
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
    let (offset, max) = scroll_read(state, target);
    let velocity = -state.touch_tracker.fling_velocity(target.axis());
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
    let has_continue_row = state.home.has_continue_row();
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
    let max_offset = region_max(state, fluxa_ui::ScrollId::Row(scroll_index));
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

pub(super) fn screen_scroll_max(state: &RendererState) -> f32 {
    region_max(state, fluxa_ui::ScrollId::Screen)
}

pub(super) fn update_screen_scroll(state: &mut RendererState, delta: f32) {
    let max_offset = screen_scroll_max(state);
    let key = ScrollKey::Screen(active_route(state));
    let offset = (state.scroll_get(key) + delta).clamp(0.0, max_offset);
    state.scroll_set(key, offset);
}

pub(super) fn row_target_at(state: &RendererState, position: [f32; 2]) -> Option<ScrollKey> {
    let (route, regions) = &state.scroll_regions;
    if *route != active_route(state) && *route != state.route {
        return None;
    }
    let point = Pos2::new(position[0], position[1]);
    regions
        .iter()
        .filter(|region| region.axis == fluxa_ui::ScrollAxis::Horizontal)
        .find(|region| region.rect.contains(point))
        .map(|region| scroll_key(state, region.id))
}

fn pan_target(state: &RendererState, start: [f32; 2], total_x: f32, total_y: f32) -> ScrollKey {
    if total_x.abs() > total_y.abs() * 1.15
        && let Some(target) = row_target_at(state, start)
        && scroll_read(state, target).1 > 0.0
    {
        return target;
    }
    ScrollKey::Screen(active_route(state))
}

fn strip_at(state: &RendererState, at: [f32; 2]) -> bool {
    row_target_at(state, at).is_some_and(|key| scroll_read(state, key).1 > 0.0)
}

fn pan_enabled(state: &RendererState, at: [f32; 2]) -> bool {
    match input_layer(state) {
        InputLayer::Screen => !state.nav_dragging && state.card_menu.is_none(),
        InputLayer::Player => screen_scroll_max(state) > 0.0 || strip_at(state, at),
        _ => false,
    }
}

pub(super) fn pan(state: &mut RendererState, phase: PointerPhase, position: [f32; 2]) -> bool {
    let origin = state.touch_start.unwrap_or(position);
    if phase != PointerPhase::Up && !pan_enabled(state, origin) {
        return false;
    }
    let now = Instant::now();
    match phase {
        PointerPhase::Down => {
            let caught = state.scroll_motion.is_active() && state.active_scroll.is_some();
            state.touch_start = Some(position);
            state.touch_last = Some(position);
            state.touch_last_at = Some(now);
            state.touch_epoch = now;
            state.touch_tracker.clear();
            track_touch(state, now, position);
            state.touch_scrolled = caught;
            state.scroll_motion.stop();
            state.scroll_animation_at = now;
            if !caught {
                state.active_scroll = None;
            }
            caught
        }
        PointerPhase::Move => {
            let (Some(start), Some(last)) = (state.touch_start, state.touch_last) else {
                return false;
            };
            if !state.touch_scrolled {
                let total_x = start[0] - position[0];
                let total_y = start[1] - position[1];
                if total_x.hypot(total_y) <= scroll_feel(state).touch_slop() {
                    return false;
                }
                state.touch_scrolled = true;
                state.active_scroll = Some(pan_target(state, start, total_x, total_y));
            }
            if let Some(target) = state.active_scroll {
                let axis = target.axis();
                let delta = last[axis] - position[axis];
                if delta != 0.0 {
                    scroll_drag(state, target, delta);
                }
            }
            track_touch(state, now, position);
            state.touch_last = Some(position);
            state.touch_last_at = Some(now);
            true
        }
        PointerPhase::Up => {
            let was_scroll = state.touch_scrolled;
            if was_scroll {
                scroll_release(state, position);
            }
            state.touch_start = None;
            state.touch_last = None;
            state.touch_last_at = None;
            state.touch_scrolled = false;
            state.touch_tracker.clear();
            was_scroll
        }
    }
}

pub(super) fn mouse_pan(state: &mut RendererState, phase: PointerPhase, position: Pos2) -> bool {
    let at = [position.x, position.y];
    match phase {
        PointerPhase::Down => {
            let on_strip = input_layer(state) == InputLayer::Player && strip_at(state, at);
            let blocked = !on_strip
                && state.gpu.as_ref().is_some_and(|gpu| {
                    fluxa_ui::scroll_blocked(fluxa_ui::gfx::Context::from_backend(
                        &gpu.egui_context,
                    )) || fluxa_ui::pointer_over_mini_player(
                        fluxa_ui::gfx::Context::from_backend(&gpu.egui_context),
                        position,
                    )
                });
            if blocked || !pan_enabled(state, at) {
                return false;
            }
            state.mouse_pan = true;
            pan(state, phase, at);
            false
        }
        PointerPhase::Move => {
            if !state.mouse_pan {
                return false;
            }
            let dragging = state
                .gpu
                .as_ref()
                .is_some_and(|gpu| gpu.egui_context.dragged_id().is_some());
            if dragging && !state.touch_scrolled {
                state.mouse_pan = false;
                pan(state, PointerPhase::Up, at);
                return false;
            }
            let was_scrolling = state.touch_scrolled;
            let scrolling = pan(state, phase, at);
            if scrolling && !was_scrolling {
                state.egui_events.push(egui::Event::PointerGone);
            }
            scrolling
        }
        PointerPhase::Up => {
            if !std::mem::take(&mut state.mouse_pan) {
                return false;
            }
            pan(state, phase, at)
        }
    }
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
    if input_layer(state) != InputLayer::Screen {
        pan(state, phase, position);
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
    if nav_drag(state, phase, position) {
        return;
    }
    match phase {
        PointerPhase::Down => {
            state.keyboard_focus_visible = false;
            state.touch_down_at = Some(Instant::now());
            if pan(state, phase, position) {
                return;
            }
        }
        PointerPhase::Move => {
            pan(state, phase, position);
        }
        PointerPhase::Up => {
            state.touch_down_at = None;
            if pan(state, phase, position) {
                return;
            }
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

fn nav_drag(state: &mut RendererState, phase: PointerPhase, position: [f32; 2]) -> bool {
    let viewport = logical_viewport(state);
    let at = Pos2::new(position[0], position[1]);
    if phase == PointerPhase::Down
        && !state.player_full()
        && state.profiles.is_none()
        && state.welcome.is_none()
        && fluxa_ui::nav_bar_rect(viewport).is_some_and(|bar| bar.contains(at))
    {
        state.keyboard_focus_visible = false;
        state.nav_dragging = true;
        fluxa_ui::set_mobile_nav_drag(Some(fluxa_ui::nav_slot_center(viewport, position[0])));
        state.touch_start = Some(position);
        return true;
    }
    if !state.nav_dragging {
        return false;
    }
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
            fluxa_ui::set_mobile_nav_pending(fluxa_ui::nav_slot_center(viewport, position[0]));
            let node = fluxa_ui::nav_node_at(viewport, at);
            rebuild_current_ui(state);
            remember_actions(state, vec![UiAction::Activated(node)]);
        }
    }
    true
}

pub(super) fn hide_pointer(state: &mut RendererState) {
    if !state.keyboard_focus_visible {
        state.egui_events.push(egui::Event::PointerGone);
    }
    state.keyboard_focus_visible = true;
}

pub(super) fn key_down(state: &mut RendererState, input: KeyInput) {
    state.settings.gamepad_active = matches!(input, KeyInput::Gamepad(_));
    if let KeyInput::Gamepad(button) = input
        && shortcuts::pad_input(state, button)
    {
        return;
    }
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
    if matches!(
        input,
        KeyInput::Key(Key::Up | Key::Down | Key::Left | Key::Right | Key::Back | Key::Escape)
    ) {
        state.ime_engaged = None;
    }
    if matches!(input, KeyInput::Key(Key::Enter))
        && let Some(node) = state.ui.focused()
        && state
            .ui
            .node(node)
            .is_some_and(|node| node.kind == UiNodeKind::Input)
    {
        state.ime_engaged = Some(node);
    }
    if matches!(input, KeyInput::Key(Key::Enter)) && open_search_input(state) {
        return;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum InputLayer {
    DetailModal,
    Profiles,
    Player,
    Screen,
}

pub(super) fn input_layer(state: &RendererState) -> InputLayer {
    if state.detail_modal {
        InputLayer::DetailModal
    } else if state.profiles.is_some() || state.welcome.is_some() {
        InputLayer::Profiles
    } else if state.player_full() {
        InputLayer::Player
    } else {
        InputLayer::Screen
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
                state.scroll_reset(ScrollKey::Screen(Route::Library));
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
            state.scroll_reset(ScrollKey::Row(Route::Detail, 1));
            reset_ui(state);
        }
        "detail:trailerKind" if route == Route::Detail => {
            state.detail.selected_trailer_kind = Some(value);
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
        fluxa_ui::NODE_ADDON_EDIT_NAME => {
            state.settings.addon_edit_name =
                value.chars().take(fluxa_ui::ADDON_NAME_LIMIT).collect();
        }
        fluxa_ui::NODE_REPORT_TITLE => {
            state.settings.report_title =
                value.chars().take(fluxa_ui::REPORT_TITLE_LIMIT).collect();
        }
        fluxa_ui::NODE_REPORT_BODY => {
            state.settings.report_body = value.chars().take(fluxa_ui::REPORT_BODY_LIMIT).collect();
        }
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL => state.settings.plugin_url = value.to_owned(),
        fluxa_ui::NODE_SETTINGS_STREAMKIT_ADDON_URL => {
            state.settings.streamkit_addon_url = value.to_owned()
        }
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
            if let Some(index) = fluxa_ui::plugin_config_input(node) {
                state.settings.plugin_config_fields[index] = value.to_owned();
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

fn open_search_input(state: &mut RendererState) -> bool {
    let searching = state.ui.focused().is_some_and(|node| {
        matches!(
            node,
            fluxa_ui::NODE_DISCOVER_SEARCH | fluxa_ui::NODE_SIDEBAR_SEARCH
        )
    });
    if !searching {
        return false;
    }
    let Some(action) = core_value("searchInputAction", json!({"query": state.discover.query}))
        .and_then(|action| serde_json::from_value::<NativeAction>(action).ok())
    else {
        return false;
    };
    state.pending_native_actions.push(action);
    true
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
