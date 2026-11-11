use super::*;

pub(crate) fn route_actions_to_session(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        start_playback_without_session(state);
        return;
    };
    if home_refresh_due(state) {
        let profile = session.active_profile();
        let command = json!({
            "type": "refreshContinueWatchingRequested",
            "profile": profile,
            "language": profile_language(&profile),
            "source": "poll",
        });
        state.home_refreshed_at = Instant::now();
        if let Err(error) = session.dispatch(command) {
            host_log(format!("core dispatch failed: {error}"));
        }
    }
    if state.pending_native_actions.is_empty() {
        return;
    }
    let profile = session.active_profile();
    let mut unhandled = Vec::new();
    let mut detail_loads = Vec::new();
    let mut open_profiles = false;
    let mut log_out = false;
    let mut toggles = Vec::new();
    let mut servers = Vec::new();
    let mut callbacks = Vec::new();
    let mut files = Vec::new();
    let mut media = Vec::new();
    let mut searched = false;
    for action in std::mem::take(&mut state.pending_native_actions) {
        let action = match action {
            NativeAction::OpenLink { url } => {
                match core_value("deepLinkAction", json!({"url": url}))
                    .and_then(|action| serde_json::from_value(action).ok())
                {
                    Some(action) => action,
                    None => continue,
                }
            }
            NativeAction::Search { query } => {
                let command = json!({
                    "type": "searchRequested",
                    "query": query.trim(),
                    "language": state.discover.language,
                });
                state.discover.query = query;
                searched = true;
                state
                    .pending_native_actions
                    .push(NativeAction::CoreCommand { command });
                NativeAction::Navigate {
                    destination: Route::Discover,
                }
            }
            action => action,
        };
        if let NativeAction::MediaCommand { command, value } = action {
            media.push((command, value));
            continue;
        }
        if let NativeAction::AccountToggle { provider } = action {
            toggles.push(provider);
            continue;
        }
        if matches!(action, NativeAction::LogOut) {
            log_out = true;
            continue;
        }
        if let NativeAction::MediaServer { node } = action {
            servers.push(node);
            continue;
        }
        if let NativeAction::OauthCallback { url } = action {
            callbacks.push(url);
            continue;
        }
        if matches!(&action, NativeAction::Navigate { destination } if *destination == Route::Profiles)
        {
            open_profiles = true;
            continue;
        }
        if let NativeAction::PlayTrailer { item, urls } = &action {
            if let Some(video) = state.video.as_mut() {
                video.stop();
            }
            state.player = Some(player::PlayerSession::trailer(item.clone(), urls));
            state.scroll.reset(ScrollKey::Screen(Route::Player));
            state.ui = UiTree::default();
            continue;
        }
        if let NativeAction::Folder { id } = &action {
            let commands = folder_commands(state.core_snapshot.as_deref(), id, &profile);
            state.folder_tab = 0;
            state.scroll.reset(ScrollKey::Screen(Route::Folder));
            for command in commands {
                if let Err(error) = session.dispatch(command) {
                    host_log(format!("core dispatch failed: {error}"));
                }
            }
            continue;
        }
        if let NativeAction::DetailModal {
            id,
            item_type,
            preview,
        } = &action
        {
            let load = NativeAction::Detail {
                id: id.clone(),
                item_type: item_type.clone(),
                preview: preview.clone(),
            };
            state.detail_modal = true;
            detail_loads.push((id.clone(), item_type.clone(), preview.clone()));
            for command in session_commands(&load, &profile)
                .unwrap_or_default()
                .into_iter()
                .skip(1)
            {
                if let Err(error) = session.dispatch(command) {
                    host_log(format!("core dispatch failed: {error}"));
                }
            }
            continue;
        }
        if let NativeAction::Detail {
            id,
            item_type,
            preview,
        } = &action
        {
            detail_loads.push((id.clone(), item_type.clone(), preview.clone()));
        }
        if let NativeAction::Play { id, item_type } = &action {
            detail_loads.push((id.clone(), item_type.clone(), Value::Null));
        }
        if matches!(action, NativeAction::OpenDetailPage) {
            state.detail_modal = false;
            if let Err(error) = session.dispatch(navigation(Route::Detail)) {
                host_log(format!("core dispatch failed: {error}"));
            }
            continue;
        }
        if let NativeAction::OpenFile { uri, title } = action {
            files.push((uri, title));
            continue;
        }
        if let NativeAction::StartPlayback { item } = &action
            && state
                .player
                .as_ref()
                .is_some_and(|player| player.minimized && player.is_playing(item))
        {
            if let Some(player) = state.player.as_mut() {
                player.minimized = false;
                player.touch();
            }
            state.ui = UiTree::default();
            continue;
        }
        if let NativeAction::StartPlayback { item } = &action {
            host_log(format!(
                "start playback id={} type={}",
                item.get("id").and_then(Value::as_str).unwrap_or("?"),
                item.get("type").and_then(Value::as_str).unwrap_or("?")
            ));
            player::keep_shuffle_for(&mut state.shuffle, item);
            let mut player =
                player::PlayerSession::new(item.clone()).with_logo_hint(known_logo(state, item));
            player.pick_manually(state.core_snapshot.as_deref());
            player.cast_target = state.cast_next.take();
            player.stale = session.snapshot().get("player").cloned();
            if let Some(video) = state.video.as_mut() {
                video.stop();
            }
            state.player = Some(player);
            state.scroll.reset(ScrollKey::Screen(Route::Player));
            state.ui = UiTree::default();
        }
        if matches!(&action, NativeAction::Navigate { destination } if *destination == Route::Discover)
            && !state.discover.catalogs.is_empty()
        {
            if let Err(error) = session.dispatch(navigation(Route::Discover)) {
                host_log(format!("core dispatch failed: {error}"));
            }
            continue;
        }
        match session_commands(&action, &profile) {
            Some(commands) => {
                for command in commands {
                    host_log(format!(
                        "core command {}",
                        command.get("type").and_then(Value::as_str).unwrap_or("?")
                    ));
                    if let Err(error) = session.dispatch(command) {
                        host_log(format!("core dispatch failed: {error}"));
                    }
                }
            }
            None => unhandled.push(action),
        }
    }
    state.pending_native_actions = unhandled;
    for (id, item_type, preview) in detail_loads {
        crate::detail_modal::begin_load(state, &id, &item_type, &preview);
    }
    if searched {
        recent_searches::refresh(state);
        reset_ui(state);
        state.focus_hint = None;
    }
    for (command, value) in media {
        player::media_command(state, &command, value);
    }
    for provider in toggles {
        accounts::toggle(state, &provider);
    }
    for node in servers {
        accounts::media_server(state, node);
    }
    for url in callbacks {
        accounts::finish_redirect(state, &url);
    }
    for (uri, title) in files {
        let command = player::open_file(state, &uri, &title);
        if let Some(session) = state.session.as_ref()
            && let Err(error) = session.dispatch(command)
        {
            host_log(format!("core dispatch failed: {error}"));
        }
    }
    if open_profiles {
        reset_ui(state);
        profiles::open(state);
    }
    if log_out {
        reset_ui(state);
        welcome::log_out(state);
    }
}

pub(crate) fn start_playback_without_session(state: &mut RendererState) {
    let profile = state
        .core_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.pointer("/profile/active").cloned())
        .unwrap_or(Value::Null);
    for index in 0..state.pending_native_actions.len() {
        let NativeAction::StartPlayback { item } = &state.pending_native_actions[index] else {
            continue;
        };
        player::keep_shuffle_for(&mut state.shuffle, item);
        let command = player::direct_playback_command(item, &profile);
        let mut player =
            player::PlayerSession::new(item.clone()).with_logo_hint(known_logo(state, item));
        player.pick_manually(state.core_snapshot.as_deref());
        player.cast_target = state.cast_next.take();
        player.stale = state
            .core_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.get("player"))
            .cloned();
        state.player = Some(player);
        state.scroll.reset(ScrollKey::Screen(Route::Player));
        reset_ui(state);
        state.pending_native_actions[index] = NativeAction::CoreCommand { command };
    }
}

pub(super) fn close_settings_section(state: &mut RendererState) {
    if state.settings.page_open {
        state.settings.page_open = false;
        state.settings.active_section = fluxa_ui::category_pages(state.settings.active_section)
            .next()
            .unwrap_or_default();
    } else {
        state.settings.section_open = false;
    }
    state.scroll.reset(ScrollKey::Screen(Route::Settings));
    reset_ui(state);
}

pub(crate) fn home_refresh_due(state: &RendererState) -> bool {
    state.window_focused
        && state.route == Route::Home
        && state.player.is_none()
        && state.home_refreshed_at.elapsed() >= HOME_REFRESH_INTERVAL
}
