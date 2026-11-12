use super::*;

pub(crate) fn remember_actions(state: &mut RendererState, actions: Vec<UiAction>) {
    save_left_fields(state);
    recent_searches::save_left_query(state);
    if actions.is_empty() {
        return;
    }
    for action in &actions {
        if let UiAction::TextInput { node, value } = action {
            edit_text(state, *node, |text| text.push_str(value));
            continue;
        }
        let node = match action {
            UiAction::Activated(node) | UiAction::PointerReleased(node) => Some(*node),
            _ => None,
        };
        if let Some(node) = node {
            host_log(format!(
                "ui activate node={node} route={}{}",
                state.route.as_str(),
                if state.player.is_some() {
                    " player"
                } else {
                    ""
                }
            ));
        } else if matches!(action, UiAction::Back) {
            host_log(format!("ui back route={}", state.route.as_str()));
        }
        let minimized = state.player.as_ref().is_some_and(|player| player.minimized);
        if state.player.is_some() && (!minimized || node.is_some_and(player::is_mini_node)) {
            if let Some(node) = node {
                player::activate(state, node);
            } else if matches!(action, UiAction::Back) {
                player::close(state);
            }
            continue;
        }
        if let Some(node) = node {
            let episode = state.route == Route::Detail
                && (fluxa_ui::NODE_DETAIL_EPISODE_BASE..fluxa_ui::NODE_DETAIL_EPISODE_BASE + 1000)
                    .contains(&node);
            if (episode
                || matches!(
                    node,
                    fluxa_ui::NODE_PLAY | fluxa_ui::NODE_DETAIL_PLAY | fluxa_ui::NODE_SHORTS_PLAY
                ))
                && playback_blocked(state)
            {
                show_notice(state, "playback.no_sources");
                continue;
            }
            if activate_welcome(state, node)
                || profiles::activate_node(state, node)
                || shortcuts::activate_node(state, node)
                || updates::activate_node(state, node)
                || stream_badges::activate_node(state, node)
                || collections::activate_node(state, node)
                || watch_together::activate_node(state, node)
                || ambient::activate_node(state, node)
                || hero_catalogs::activate_node(state, node)
                || detail_sections::activate_node(state, node)
                || downloads::activate_node(state, node)
            {
                continue;
            }
            let request = state
                .rendered_layout
                .as_ref()
                .filter(|(route, _)| *route == state.route)
                .and_then(|(_, layout)| layout.choices.iter().find(|(id, _)| *id == node))
                .map(|(_, request)| request.clone());
            if let Some(request) = request {
                card_menu::open_choice(state, request);
                continue;
            }
            if state.route == Route::Home
                && let Some(category) = fluxa_ui::HomeCategory::from_node(node)
            {
                fluxa_ui::set_home_category(category);
                state.last_snapshot_revision = None;
                state.scroll_reset(ScrollKey::Screen(Route::Home));
                reset_ui(state);
                continue;
            }
            if state.route == Route::Home && node == fluxa_ui::NODE_HOME_ADD_ADDONS {
                state.settings.active_section = fluxa_ui::addons_page();
                state.settings.section_open = true;
                state.settings.page_open = true;
                state.settings.search.clear();
                state.scroll_reset(ScrollKey::Screen(Route::Settings));
                state.pending_native_actions.push(NativeAction::Navigate {
                    destination: Route::Settings,
                });
                reset_ui(state);
                continue;
            }
            if node == fluxa_ui::NODE_LIBRARY {
                state.library_downloads = false;
                state.library_download_group = None;
                refresh_library_view(state);
            }
            if node == fluxa_ui::NODE_SIDEBAR_DOWNLOADS {
                state.library_downloads = true;
                state.library_download_group = None;
                refresh_library_view(state);
            }
            if (fluxa_ui::NODE_LIBRARY_SECTION_BASE..fluxa_ui::NODE_LIBRARY_SECTION_BASE + 3)
                .contains(&node)
            {
                let section = node - fluxa_ui::NODE_LIBRARY_SECTION_BASE;
                state.library_downloads = section == 2;
                state.library_download_group = None;
                refresh_library_view(state);
                let destination = if section == 1 {
                    Route::Calendar
                } else {
                    Route::Library
                };
                if destination != state.route {
                    state
                        .pending_native_actions
                        .push(NativeAction::Navigate { destination });
                }
                state.scroll_reset(ScrollKey::Screen(Route::Library));
                reset_ui(state);
                continue;
            }
            if state.route == Route::Library && node == fluxa_ui::NODE_LIBRARY_VIEW {
                state.library_list = !state.library_list;
                refresh_library_view(state);
                state.scroll_reset(ScrollKey::Screen(Route::Library));
                reset_ui(state);
                continue;
            }
            if state.route == Route::Library && node == fluxa_ui::NODE_LIBRARY_SORT {
                state.library_sort = match state.library_sort.as_str() {
                    "recent" => "title",
                    "title" => "rating",
                    _ => "recent",
                }
                .to_owned();
                refresh_library_view(state);
                continue;
            }
            if state.route == Route::Folder
                && node >= fluxa_ui::NODE_FOLDER_TAB_BASE
                && let index = (node - fluxa_ui::NODE_FOLDER_TAB_BASE) as usize
                && index < state.folder.tabs.len()
            {
                state.folder_tab = index;
                state.scroll_reset(ScrollKey::Screen(Route::Folder));
                reset_ui(state);
                continue;
            }
            if state.route == Route::Calendar {
                if node == fluxa_ui::NODE_CALENDAR_CLOSE_DAY {
                    state.calendar.selected_day = None;
                    reset_ui(state);
                    continue;
                }
                if (fluxa_ui::NODE_CALENDAR_DAY_BASE..fluxa_ui::NODE_CALENDAR_DAY_BASE + 32)
                    .contains(&node)
                {
                    state.calendar.selected_day =
                        Some((node - fluxa_ui::NODE_CALENDAR_DAY_BASE) as u32);
                    reset_ui(state);
                    continue;
                }
            }
            if state.route == Route::Settings
                && let Some(color) = node
                    .checked_sub(fluxa_ui::NODE_SETTINGS_SWATCH_BASE)
                    .and_then(|slot| fluxa_ui::accent_swatches().get(slot as usize))
            {
                state
                    .pending_native_actions
                    .push(NativeAction::SettingsChange {
                        key: fluxa_ui::ACCENT_COLOR_KEY.to_owned(),
                        value: Value::String((*color).to_owned()),
                    });
                continue;
            }
            if state.route == Route::Settings
                && let Some(row) = node
                    .checked_sub(fluxa_ui::NODE_SETTINGS_ROW_BASE)
                    .and_then(|index| fluxa_ui::settings_row_by_index(index as usize))
                && (row.key == fluxa_ui::APP_ICON_KEY
                    || logical_viewport(state).is_compact()
                    || logical_viewport(state).is_tv())
                && !row.choices().is_empty()
                && row.key != fluxa_ui::ACCENT_COLOR_KEY
            {
                card_menu::open_setting(state, row);
                continue;
            }
            if state.route == Route::Settings
                && (logical_viewport(state).is_compact() || logical_viewport(state).is_tv())
                && let Some(index) = node.checked_sub(fluxa_ui::NODE_SETTINGS_ACCOUNT_SOURCE_BASE)
                && (index < 2 || (3..=5).contains(&index))
            {
                card_menu::open_source(state, index as usize);
                continue;
            }
            if state.route == Route::Settings
                && (fluxa_ui::NODE_SETTINGS_PLUGIN_CONFIG_BASE
                    ..fluxa_ui::NODE_SETTINGS_PLUGIN_CONFIG_BASE + 30)
                    .contains(&node)
            {
                let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_CONFIG_BASE) as usize;
                open_plugin_config(state, index);
                reset_ui(state);
                continue;
            }
            if state.route == Route::Settings
                && let Some((index, 5)) = fluxa_ui::addon_action(node)
                && state.settings.addon_edit.is_none()
            {
                state.settings.addon_edit_name = state
                    .settings
                    .addons
                    .get(index)
                    .and_then(|addon| addon.get("displayName"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                state.settings.addon_edit = Some(index);
                reset_ui(state);
                continue;
            }
            if state.route == Route::Settings && state.settings.addon_edit.is_some() {
                match node {
                    fluxa_ui::NODE_ADDON_EDIT_SAVE => {
                        if let Some(index) = state.settings.addon_edit
                            && let Some(action) = settings_action_json(
                                fluxa_ui::addon_action_node(index, 5),
                                &state.settings,
                            )
                        {
                            state
                                .pending_native_actions
                                .push(NativeAction::CoreCommand { command: action });
                        }
                        state.settings.addon_edit = None;
                        state.settings.addon_edit_name.clear();
                    }
                    fluxa_ui::NODE_ADDON_EDIT_CANCEL => {
                        state.settings.addon_edit = None;
                        state.settings.addon_edit_name.clear();
                    }
                    _ => {}
                }
                reset_ui(state);
                continue;
            }
            if state.route == Route::Settings && state.settings.report_open {
                match node {
                    fluxa_ui::NODE_REPORT_SEND => crash::submit_report(state),
                    fluxa_ui::NODE_REPORT_CANCEL => crash::close_report(state),
                    _ => {}
                }
                continue;
            }
            if state.route == Route::Settings && node == fluxa_ui::NODE_SETTINGS_REPORT_PROBLEM {
                crash::open_report(state);
                continue;
            }
            if state.route == Route::Settings
                && let Some(url) = fluxa_ui::settings_link_url(node)
            {
                state.open_url = Some(url.to_owned());
                continue;
            }
            if state.route == Route::Settings
                && node == fluxa_ui::NODE_SETTINGS_BACK
                && (state.settings.section_open || state.settings.page_open)
            {
                close_settings_section(state);
                continue;
            }
            if state.route == Route::Settings
                && let Some((index, 3)) = fluxa_ui::addon_action(node)
                && let Some(section) = state
                    .settings
                    .addons
                    .get(index)
                    .and_then(fluxa_ui::kit_section)
            {
                state.settings.active_section = section;
                state.settings.section_open = true;
                state.settings.page_open = true;
                state.settings.search.clear();
                state.scroll_reset(ScrollKey::Screen(Route::Settings));
                reset_ui(state);
                continue;
            }
            if state.route == Route::Settings
                && let Some(index) = fluxa_ui::settings_page_for_node(node)
            {
                state.settings.active_section = index;
                state.settings.section_open = true;
                state.settings.page_open = node < fluxa_ui::NODE_SETTINGS_SECTION_BASE;
                state.settings.search.clear();
                state.scroll_reset(ScrollKey::Screen(Route::Settings));
                reset_ui(state);
                continue;
            }
            if state.route == Route::Settings
                && let Some(action_json) = settings_action_json(node, &state.settings)
            {
                if node == fluxa_ui::NODE_SETTINGS_ADDON_INSTALL {
                    state.addon_install = Some(state.settings.addon_url.trim().to_owned());
                    state.settings.addon_url.clear();
                } else if node == fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL {
                    state.settings.plugin_url.clear();
                } else if node == fluxa_ui::NODE_SETTINGS_STREAMKIT_ADDON_ADD {
                    state.settings.streamkit_addon_url.clear();
                }
                state
                    .pending_native_actions
                    .push(NativeAction::CoreCommand {
                        command: action_json,
                    });
                reset_ui(state);
                continue;
            }
            if state.route == Route::Detail
                && (fluxa_ui::NODE_DETAIL_SEASON_BASE..fluxa_ui::NODE_DETAIL_EPISODE_BASE)
                    .contains(&node)
            {
                state.detail.selected_season =
                    Some((node - fluxa_ui::NODE_DETAIL_SEASON_BASE) as i64);
                state.scroll_reset(ScrollKey::Row(Route::Detail, 1));
                reset_ui(state);
                continue;
            }
            if state.route == Route::Detail && node == fluxa_ui::NODE_DETAIL_SHUFFLE {
                if let Some(item) = player::start_shuffle(state) {
                    state
                        .pending_native_actions
                        .push(NativeAction::StartPlayback { item });
                }
                continue;
            }
            let (node, hero) = if state.route == Route::Shorts {
                let node = match node {
                    fluxa_ui::NODE_SHORTS_PLAY => NODE_PLAY,
                    fluxa_ui::NODE_SHORTS_WATCHLIST => fluxa_ui::NODE_HERO_WATCHLIST,
                    fluxa_ui::NODE_SHORTS_INFO => NODE_MORE_INFO,
                    other => other,
                };
                (node, shorts::current(state))
            } else {
                (
                    node,
                    state.gpu.as_ref().and_then(|gpu| {
                        fluxa_ui::active_home_hero(
                            fluxa_ui::gfx::Context::from_backend(&gpu.egui_context),
                            &state.home,
                        )
                    }),
                )
            };
            if let Some(native_action) = native_action_for_node(
                node,
                &state.home,
                &state.library,
                state.library_tab,
                &state.discover,
                &state.folder,
                state.folder_tab,
                &state.calendar,
                &state.detail,
                &state.settings,
                state.route,
                hero,
            ) {
                let native_action = match native_action {
                    NativeAction::Detail {
                        id,
                        item_type,
                        preview,
                    } if state.route != Route::Detail
                        && state.home.form_factor == fluxa_ui::UiFormFactorJson::Desktop =>
                    {
                        NativeAction::DetailModal {
                            id,
                            item_type,
                            preview,
                        }
                    }
                    action => action,
                };
                if let NativeAction::Detail { preview, .. }
                | NativeAction::DetailModal { preview, .. } = &native_action
                {
                    state.backdrop_prefetch = ["background", "poster"]
                        .into_iter()
                        .find_map(|key| preview.get(key).and_then(Value::as_str))
                        .map(ToOwned::to_owned);
                }
                state.pending_native_actions.push(native_action);
            }
        }
        if matches!(action, UiAction::Back) {
            if state.profiles.is_some() {
                profiles::back(state);
            } else if state.route == Route::Calendar && state.calendar.selected_day.is_some() {
                state.calendar.selected_day = None;
                reset_ui(state);
            } else if state.route == Route::Settings
                && (state.settings.section_open || state.settings.page_open)
            {
                close_settings_section(state);
            } else {
                state.pending_native_actions.push(NativeAction::Back);
            }
        }
    }
    state.last_actions.extend(actions);
    // Keep the bridge bounded while the real scene/action dispatcher is being
    // connected. The queue is diagnostic for now, not the source of truth.
    const MAX_RETAINED_ACTIONS: usize = 32;
    if state.last_actions.len() > MAX_RETAINED_ACTIONS {
        let drain_count = state.last_actions.len() - MAX_RETAINED_ACTIONS;
        state.last_actions.drain(..drain_count);
    }
    const MAX_PENDING_NATIVE_ACTIONS: usize = 16;
    if state.pending_native_actions.len() > MAX_PENDING_NATIVE_ACTIONS {
        let drain_count = state.pending_native_actions.len() - MAX_PENDING_NATIVE_ACTIONS;
        state.pending_native_actions.drain(..drain_count);
    }
}

fn activate_welcome(state: &mut RendererState, node: u64) -> bool {
    if node != fluxa_ui::NODE_WELCOME_OFFLINE || state.welcome.is_none() {
        return false;
    }
    welcome::handle(state, fluxa_ui::WelcomeRequest::Offline);
    true
}
