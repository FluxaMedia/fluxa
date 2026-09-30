use super::*;

pub(super) fn request_projection(state: &mut RendererState) {
    let Some(snapshot) = state.core_snapshot.clone() else {
        return;
    };
    let revision = state.core_snapshot_revision;
    if state.last_snapshot_revision == Some(revision) {
        return;
    }
    state.last_snapshot_revision = Some(revision);
    state.projector.submit(projection::Request {
        revision,
        snapshot,
        form_factor: state.home.form_factor,
        library_tab: state.library_tab,
        library_query: state.library_query.clone(),
        library_sort: state.library_sort.clone(),
        library_type: state.library_type.clone(),
        library_list: state.library_list,
        library_downloads: state.library_downloads,
        hero_trailers: state.trailers.inputs(),
    });
}

pub(super) fn apply_projection(state: &mut RendererState, projection: projection::Projection) {
    state
        .projector
        .settle(projection.revision, state.core_snapshot_revision);
    if projection.route != state.route {
        state.active_scroll = None;
        state.scroll_velocity = 0.0;
    }
    state.route = projection.route;
    let scroll_offset = state.home.scroll_offset;
    let row_scroll_offsets = std::mem::take(&mut state.home.row_scroll_offsets);
    let platform = state.home.platform;
    state.home = projection.home;
    state.home.platform = platform;
    state.home.scroll_offset = scroll_offset;
    state.home.row_scroll_offsets = row_scroll_offsets;
    if HOME_SYNC_LOGS.fetch_add(1, Ordering::Relaxed) % 120 == 0 {
        let titles = state
            .home
            .rows
            .iter()
            .take(5)
            .map(|row| row.title.as_str())
            .collect::<Vec<_>>();
        host_log(format!(
            "Home snapshot: hero={}, title={:?}, artwork={}, continue_cards={}, rows={:?}",
            state.home.show_hero_section,
            state.home.title,
            state.home.background_url.is_some(),
            state.home.cards.len(),
            titles,
        ));
    }
    state.library = projection.library;
    let query = std::mem::take(&mut state.discover.query);
    state.discover = projection.discover;
    state.discover.query = query;
    let selected_day = state.calendar.selected_day;
    let previous_month = (state.calendar.year, state.calendar.month);
    state.calendar = projection.calendar;
    if previous_month == (state.calendar.year, state.calendar.month) {
        state.calendar.selected_day = selected_day;
    }
    let row_scroll_offsets = state.detail.row_scroll_offsets;
    let selected_season = state.detail.selected_season;
    let same_detail = state.detail.id == projection.detail.id;
    state.detail = projection.detail;
    if same_detail {
        state.detail.row_scroll_offsets = row_scroll_offsets;
        state.detail.selected_season = selected_season;
    }
    state.detail.resume = state.home.resume_for(&state.detail.id).cloned();
    state.trailers.set_targets(projection.trailer_targets);
    let settings_section = state.settings.active_section;
    let section_open = state.settings.section_open;
    let page_open = state.settings.page_open;
    let addon_url = std::mem::take(&mut state.settings.addon_url);
    let plugin_url = std::mem::take(&mut state.settings.plugin_url);
    let search = std::mem::take(&mut state.settings.search);
    let poster_fields = std::mem::take(&mut state.settings.poster_fields);
    let server_fields = std::mem::take(&mut state.settings.server_fields);
    state.settings = projection.settings;
    state.settings.active_section = settings_section.min(fluxa_ui::SETTINGS_SECTIONS.len() - 1);
    state.settings.section_open = section_open;
    state.settings.page_open = page_open;
    state.settings.addon_url = addon_url;
    state.settings.plugin_url = plugin_url;
    state.settings.search = search;
    state.settings.server_fields = server_fields;
    shortcuts::refresh(state);
    accounts::refresh_servers(state);
    for (field, typed) in state.settings.poster_fields.iter_mut().zip(poster_fields) {
        if !typed.is_empty() {
            *field = typed;
        }
    }
    reset_ui(state);
    request_discover_background_page(state);
}

pub(super) const DISCOVER_BACKGROUND_LIMIT: usize = 400;

pub(super) fn request_discover_background_page(state: &mut RendererState) {
    let discover = &state.discover;
    if state.route == Route::Discover
        || discover.is_loading
        || !discover.query.is_empty()
        || discover.results.len() >= DISCOVER_BACKGROUND_LIMIT
    {
        return;
    }
    let Some(request) = discover.next_page.as_ref() else {
        return;
    };
    let skip = request.get("skip").and_then(Value::as_i64);
    if skip.is_none() || skip == state.discover_background_skip {
        return;
    }
    state.discover_background_skip = skip;
    state
        .pending_native_actions
        .push(NativeAction::CoreCommand {
            command: request.clone(),
        });
}

pub(super) fn sync_home_from_core_snapshot(state: &mut RendererState) {
    request_projection(state);
    if let Some(projection) = state.projector.take() {
        apply_projection(state, projection);
    }
}
