use super::*;

pub(super) fn rebuild_home_ui(
    ui: &mut UiTree,
    [width, height]: [u32; 2],
    home: &HomeModel,
    safe_bottom: f32,
) {
    if ui.node(NODE_PLAY).is_some() {
        return;
    }
    *ui = UiTree::default();
    let root = ui.root();
    if let Some(node) = ui.node_mut(root) {
        node.bounds = fluxa_renderer::Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        };
    }
    let layout = fluxa_ui::home_layout(
        Viewport::new(width, height, home.form_factor.into())
            .with_platform(home.platform)
            .with_safe_bottom(safe_bottom),
        home,
    );
    for (id, rect) in layout.focusable {
        let label = match id {
            NODE_HOME => "Home",
            NODE_LIBRARY => "Library",
            NODE_DISCOVER => "Discover",
            NODE_CALENDAR => "Calendar",
            fluxa_ui::NODE_SHORTS => "Shorts",
            NODE_PROFILE => "Profile",
            NODE_PLAY => "Play",
            NODE_MORE_INFO => "More info",
            _ => home
                .card_at((id - NODE_CARD_BASE) as usize)
                .map(|card| card.title.as_str())
                .unwrap_or("Title"),
        };
        let _ = ui.add(
            root,
            UiNode::new(
                id,
                UiNodeKind::Button,
                fluxa_renderer::Rect {
                    x: rect.left(),
                    y: rect.top(),
                    width: rect.width(),
                    height: rect.height(),
                },
            )
            .focusable()
            .label(label),
        );
    }
    // Compose opens Home with the primary action focused only when a real
    // billboard exists. A loading/empty state must not draw a focus ring over
    // an unrelated row placeholder.
    let _ = ui.set_focus(if home.form_factor == UiFormFactorJson::Mobile {
        None
    } else if home.item_id.is_some() {
        Some(NODE_PLAY)
    } else {
        Some(NODE_HOME)
    });
}

pub(super) fn logical_surface_size(state: &RendererState) -> [u32; 2] {
    [
        (state.size[0] as f32 / state.scale()).round().max(1.0) as u32,
        (state.size[1] as f32 / state.scale()).round().max(1.0) as u32,
    ]
}

pub(super) fn rebuild_current_ui(state: &mut RendererState) {
    if state
        .ui
        .node(state.ui.root())
        .is_some_and(|root| !root.children.is_empty())
    {
        return;
    }
    let size = logical_surface_size(state);
    let cached_layout = state
        .rendered_layout
        .as_ref()
        .filter(|(route, _)| *route == active_route(state))
        .map(|(_, layout)| layout.clone());
    if let Some(layout) = cached_layout {
        // This path only runs when an input event has invalidated the
        // accessibility tree; the normal frame path borrows the layout.
        rebuild_ui_from_layout(state, &layout, size);
        return;
    }
    if state.route != Route::Home || state.player.is_some() || state.profiles.is_some() {
        return;
    }
    rebuild_home_ui(&mut state.ui, size, &state.home, state.safe_bottom);
}

pub(super) fn label_for_node(state: &RendererState, node: u64) -> String {
    match node {
        fluxa_ui::NODE_HOME => "Home".to_owned(),
        fluxa_ui::NODE_LIBRARY => "Library".to_owned(),
        fluxa_ui::NODE_DISCOVER => "Discover".to_owned(),
        fluxa_ui::NODE_CALENDAR => "Calendar".to_owned(),
        fluxa_ui::NODE_SHORTS => "Shorts".to_owned(),
        fluxa_ui::NODE_SHORTS_PLAY => "Play".to_owned(),
        fluxa_ui::NODE_SHORTS_WATCHLIST => "Watchlist".to_owned(),
        fluxa_ui::NODE_SHORTS_INFO => "More info".to_owned(),
        fluxa_ui::NODE_PROFILE => "Profile and settings".to_owned(),
        fluxa_ui::NODE_LIBRARY_SEARCH => {
            fluxa_ui::localized("library.filter_placeholder", &state.library.language)
        }
        fluxa_ui::NODE_SETTINGS_ADDON_URL => fluxa_ui::localized("settings.addon_url", "en"),
        fluxa_ui::NODE_SETTINGS_SEARCH => fluxa_ui::localized("settings.search_placeholder", "en"),
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL => fluxa_ui::localized("settings.plugin_url", "en"),
        fluxa_ui::NODE_SETTINGS_SERVER_ADDRESS => {
            fluxa_ui::localized("settings.server_address", "en")
        }
        fluxa_ui::NODE_SETTINGS_SERVER_USERNAME => {
            fluxa_ui::localized("settings.server_username", "en")
        }
        fluxa_ui::NODE_SETTINGS_SERVER_PASSWORD => {
            fluxa_ui::localized("settings.server_password", "en")
        }
        fluxa_ui::NODE_SETTINGS_SERVER_JELLYFIN => "Jellyfin".to_owned(),
        fluxa_ui::NODE_SETTINGS_SERVER_EMBY => "Emby".to_owned(),
        fluxa_ui::NODE_SETTINGS_SERVER_PLEX => "Plex".to_owned(),
        id if fluxa_ui::poster_field(id).is_some() => fluxa_ui::poster_field(id)
            .map(|index| fluxa_ui::localized(fluxa_ui::POSTER_FIELDS[index].label, "en"))
            .unwrap_or_default(),
        fluxa_ui::NODE_SETTINGS_ADDON_INSTALL => {
            fluxa_ui::localized("settings.addon_install", "en")
        }
        fluxa_ui::NODE_SETTINGS_ADDON_REFRESH => {
            fluxa_ui::localized("settings.addon_refresh", "en")
        }
        fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL => fluxa_ui::localized("settings.plugin_add", "en"),
        id if fluxa_ui::addon_action(id).is_some() => {
            let key = match fluxa_ui::addon_action(id).map_or(0, |(_, action)| action) {
                0 => "settings.addon_move_up",
                1 => "settings.addon_move_down",
                2 => "settings.addon_refresh_one",
                _ => "settings.addon_remove",
            };
            fluxa_ui::localized(key, "en")
        }
        id if (fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE
            ..fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + 20)
            .contains(&id) =>
        {
            fluxa_ui::localized("settings.plugin_remove", "en")
        }
        id if (fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE
            ..fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE + 20)
            .contains(&id) =>
        {
            fluxa_ui::localized("settings.plugin_refresh", "en")
        }
        id if (fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE
            ..fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE + 30)
            .contains(&id) =>
        {
            let index = (id - fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE) as usize;
            let enabled = state
                .settings
                .plugins
                .get("scrapers")
                .and_then(Value::as_array)
                .and_then(|items| items.get(index))
                .and_then(|scraper| scraper.get("enabled"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            fluxa_ui::localized(
                if enabled {
                    "settings.plugin_enabled"
                } else {
                    "settings.plugin_disabled"
                },
                "en",
            )
        }
        fluxa_ui::NODE_LIBRARY_SORT => fluxa_ui::localized(
            match state.library_sort.as_str() {
                "title" => "library.sort_title",
                "rating" => "library.sort_rating",
                _ => "library.sort_recent",
            },
            &state.library.language,
        ),
        fluxa_ui::NODE_CALENDAR_PREV => "Previous month".to_owned(),
        fluxa_ui::NODE_CALENDAR_NEXT => "Next month".to_owned(),
        fluxa_ui::NODE_DETAIL_BACK | fluxa_ui::NODE_SETTINGS_BACK => "Back".to_owned(),
        fluxa_ui::NODE_SETTINGS_SWITCH_PROFILE => "Switch profiles".to_owned(),
        fluxa_ui::NODE_DETAIL_PLAY => "Play".to_owned(),
        fluxa_ui::NODE_DETAIL_WATCHLIST => {
            if state.detail.in_watchlist {
                "Remove from watchlist".to_owned()
            } else {
                "Add to watchlist".to_owned()
            }
        }
        fluxa_ui::NODE_LIBRARY_STATUS => state.library_tab.label().to_owned(),
        fluxa_ui::NODE_LIBRARY_TYPE => "Type".to_owned(),
        id if id == fluxa_ui::NODE_DISCOVER_TYPE_BASE => {
            if id == fluxa_ui::NODE_DISCOVER_TYPE_BASE {
                "Movies".to_owned()
            } else {
                "Series".to_owned()
            }
        }
        id if id == fluxa_ui::NODE_DISCOVER_CATALOG_BASE => state
            .discover
            .catalogs
            .iter()
            .filter(|catalog| catalog.content_type == state.discover.content_type)
            .nth((id - fluxa_ui::NODE_DISCOVER_CATALOG_BASE) as usize)
            .map(|catalog| catalog.label.clone())
            .unwrap_or_else(|| "Catalog".to_owned()),
        id if (fluxa_ui::NODE_CALENDAR_DAY_BASE..fluxa_ui::NODE_CALENDAR_DAY_BASE + 32)
            .contains(&id) =>
        {
            format!("Day {}", id - fluxa_ui::NODE_CALENDAR_DAY_BASE)
        }
        fluxa_ui::NODE_CALENDAR_CLOSE_DAY => "Close day".to_owned(),
        id if (fluxa_ui::NODE_CALENDAR_EVENT_BASE..fluxa_ui::NODE_CALENDAR_EVENT_BASE + 10)
            .contains(&id) =>
        {
            state
                .calendar
                .selected_day
                .and_then(|day| {
                    state
                        .calendar
                        .entries_for_day(day)
                        .nth((id - fluxa_ui::NODE_CALENDAR_EVENT_BASE) as usize)
                })
                .map(|entry| entry.card.title.clone())
                .unwrap_or_else(|| "Release".to_owned())
        }
        id if (fluxa_ui::NODE_DETAIL_SIMILAR_BASE..fluxa_ui::NODE_DETAIL_SIMILAR_BASE + 16)
            .contains(&id) =>
        {
            state
                .detail
                .similar
                .get((id - fluxa_ui::NODE_DETAIL_SIMILAR_BASE) as usize)
                .map(|card| card.title.clone())
                .unwrap_or_else(|| "Recommended title".to_owned())
        }
        id if (fluxa_ui::NODE_SETTINGS_ROW_BASE
            ..fluxa_ui::NODE_SETTINGS_ROW_BASE
                + fluxa_ui::SETTINGS_SECTIONS
                    .iter()
                    .map(|section| section.rows.len())
                    .sum::<usize>() as u64)
            .contains(&id) =>
        {
            fluxa_ui::settings_row_by_index((id - fluxa_ui::NODE_SETTINGS_ROW_BASE) as usize)
                .map(|row| row.label.to_owned())
                .unwrap_or_else(|| "Setting".to_owned())
        }
        id if fluxa_ui::settings_page_for_node(id).is_some() => {
            fluxa_ui::settings_page_for_node(id)
                .and_then(|index| fluxa_ui::SETTINGS_SECTIONS.get(index))
                .map(|section| section.title.to_owned())
                .unwrap_or_else(|| "Settings".to_owned())
        }
        id if id >= NODE_CARD_BASE => {
            let index = (id - NODE_CARD_BASE) as usize;
            let card = match state.route {
                Route::Library => state.library.cards(state.library_tab).get(index),
                Route::Discover => state.discover.results.get(index),
                _ => state.home.card_at(index),
            };
            card.map(|card| card.title.clone())
                .unwrap_or_else(|| "Media".to_owned())
        }
        _ => "Button".to_owned(),
    }
}

pub(super) fn reset_ui(state: &mut RendererState) {
    if let (Some((route, _)), Some(node)) = (state.rendered_layout.as_ref(), state.ui.focused()) {
        state.focus_hint = Some((*route, node));
    }
    state.ui = UiTree::default();
}

pub(super) fn rebuild_ui_from_layout(
    state: &mut RendererState,
    layout: &HomeLayout,
    [width, height]: [u32; 2],
) {
    let target_count = layout
        .focusable
        .iter()
        .filter(|(_, bounds)| {
            // Keep off-screen targets in the focus graph. Pointer hit testing
            // still ignores them because their bounds do not contain an
            // on-screen position, while D-pad navigation can select a card
            // and ask the scroll controller to reveal it.
            !bounds.is_negative() && bounds.width() >= 1.0 && bounds.height() >= 1.0
        })
        .count();
    let root = state.ui.root();
    let same_targets = state
        .ui
        .node(root)
        .is_some_and(|root_node| root_node.children.len() == target_count)
        && layout
            .focusable
            .iter()
            .filter(|(_, bounds)| {
                !bounds.is_negative() && bounds.width() >= 1.0 && bounds.height() >= 1.0
            })
            .zip(
                state
                    .ui
                    .node(root)
                    .into_iter()
                    .flat_map(|root_node| root_node.children.iter()),
            )
            .all(|((id, _), child_id)| id == child_id && state.ui.node(*id).is_some());
    if same_targets {
        // Scrolling changes focus-target coordinates continuously, but not
        // the focus graph. Update those bounds in place instead of allocating
        // a Vec and rebuilding the whole UiTree on every scroll frame.
        for (id, bounds) in layout.focusable.iter().filter(|(_, bounds)| {
            !bounds.is_negative() && bounds.width() >= 1.0 && bounds.height() >= 1.0
        }) {
            if let Some(node) = state.ui.node_mut(*id) {
                node.bounds = fluxa_renderer::Rect {
                    x: bounds.left(),
                    y: bounds.top(),
                    width: bounds.width(),
                    height: bounds.height(),
                };
            }
        }
        if let Some(root_node) = state.ui.node_mut(root) {
            root_node.bounds.width = width as f32;
            root_node.bounds.height = height as f32;
        }
        return;
    }

    let targets = layout
        .focusable
        .iter()
        .filter_map(|(id, bounds)| {
            (!bounds.is_negative() && bounds.width() >= 1.0 && bounds.height() >= 1.0)
                .then_some((*id, *bounds))
        })
        .collect::<Vec<_>>();
    let route = active_route(state);
    let hint = state.focus_hint.take().filter(|(built, _)| *built == route);
    let previous_focus = state.ui.focused().or(hint.map(|(_, node)| node));
    let retained_focus = targets
        .iter()
        .any(|(id, _)| Some(*id) == previous_focus)
        .then_some(previous_focus)
        .flatten();
    let first_target = targets
        .iter()
        .map(|(id, _)| *id)
        .find(|id| !fluxa_ui::is_text_node(*id));
    let mut ui = UiTree::default();
    let root = ui.root();
    if let Some(node) = ui.node_mut(root) {
        node.bounds = fluxa_renderer::Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        };
    }
    for (id, bounds) in targets {
        let Some(node_id) = ui.add(
            root,
            UiNode::new(
                id,
                if id == fluxa_ui::NODE_LIBRARY_SEARCH
                    || id == fluxa_ui::NODE_DISCOVER_SEARCH
                    || id == fluxa_ui::NODE_SETTINGS_ADDON_URL
                    || id == fluxa_ui::NODE_SETTINGS_SEARCH
                    || id == fluxa_ui::NODE_SETTINGS_PLUGIN_URL
                    || fluxa_ui::poster_field(id).is_some()
                    || fluxa_ui::server_input(id).is_some()
                {
                    UiNodeKind::Input
                } else {
                    UiNodeKind::Button
                },
                fluxa_renderer::Rect {
                    x: bounds.left(),
                    y: bounds.top(),
                    width: bounds.width(),
                    height: bounds.height(),
                },
            )
            .focusable()
            .label(label_for_node(state, id)),
        ) else {
            continue;
        };
        if matches!(
            node_id,
            fluxa_ui::NODE_HOME
                | fluxa_ui::NODE_LIBRARY
                | fluxa_ui::NODE_DISCOVER
                | fluxa_ui::NODE_CALENDAR
                | fluxa_ui::NODE_SHORTS
                | fluxa_ui::NODE_PROFILE
        ) {
            if let Some(node) = ui.node_mut(node_id) {
                node.order = 10_000;
            }
        }
    }
    let fallback = match state.route {
        Route::Library => state
            .library
            .cards(state.library_tab)
            .first()
            .map(|_| NODE_CARD_BASE)
            .unwrap_or(fluxa_ui::NODE_LIBRARY_STATUS),
        Route::Discover => state
            .discover
            .results
            .first()
            .map(|_| NODE_CARD_BASE)
            .unwrap_or(fluxa_ui::NODE_DISCOVER_TYPE_BASE),
        Route::Calendar => {
            if state.calendar.selected_day.is_some() {
                fluxa_ui::NODE_CALENDAR_CLOSE_DAY
            } else {
                fluxa_ui::NODE_CALENDAR_PREV
            }
        }
        Route::Shorts => fluxa_ui::NODE_SHORTS_PLAY,
        Route::Detail => fluxa_ui::NODE_DETAIL_PLAY,
        Route::Settings => fluxa_ui::NODE_SETTINGS_BACK,
        _ if state.home.item_id.is_some() => fluxa_ui::NODE_PLAY,
        _ => fluxa_ui::NODE_HOME,
    };
    let fallback = if ui.node(fallback).is_some() {
        fallback
    } else {
        first_target.unwrap_or(fallback)
    };
    ui.set_focus(Some(retained_focus.unwrap_or(fallback)));
    state.ui = ui;
}

pub(super) fn ensure_focused_visible(state: &mut RendererState) {
    let Some(focused) = state.ui.focused() else {
        return;
    };
    let Some(node) = state.ui.node(focused) else {
        return;
    };
    let is_navigation = matches!(
        focused,
        fluxa_ui::NODE_HOME
            | fluxa_ui::NODE_LIBRARY
            | fluxa_ui::NODE_DISCOVER
            | fluxa_ui::NODE_CALENDAR
            | fluxa_ui::NODE_SHORTS
            | fluxa_ui::NODE_PROFILE
    );
    if is_navigation {
        return;
    }
    let viewport = Viewport::new(
        (state.size[0] as f32 / state.scale()).round().max(1.0) as u32,
        (state.size[1] as f32 / state.scale()).round().max(1.0) as u32,
        state.home.form_factor.into(),
    )
    .with_platform(state.home.platform)
    .with_safe_bottom(state.safe_bottom);
    let top_inset = 12.0;
    let bottom_inset = 12.0 + state.safe_bottom;
    let visible_top = top_inset;
    let visible_bottom = (viewport.height - bottom_inset).max(visible_top + 1.0);
    let node_top = node.bounds.y;
    let node_bottom = node.bounds.y + node.bounds.height;
    let delta = if node_top < visible_top {
        node_top - visible_top
    } else if node_bottom > visible_bottom {
        node_bottom - visible_bottom
    } else {
        0.0
    };
    if delta.abs() < 0.5 {
        return;
    }
    if state.route == Route::Home {
        let max_offset = fluxa_ui::home_scroll_max(viewport, &state.home);
        state.home.scroll_offset = (state.home.scroll_offset + delta).clamp(0.0, max_offset);
    } else {
        let route = state.route;
        let max_offset = screen_scroll_max(state, viewport);
        let offset = state.screen_scroll_offsets.entry(route).or_default();
        *offset = (*offset + delta).clamp(0.0, max_offset);
    }
    // The next frame will draw the newly revealed target at its scrolled
    // position and rebuild the same UiTree with the updated geometry.
}
