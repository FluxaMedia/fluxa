use super::*;
use fluxa_ui::localized;

const NOTICE_SECONDS: f32 = 2.5;

pub(super) fn request_projection(state: &mut RendererState) {
    let Some(snapshot) = state.core_snapshot.clone() else {
        return;
    };
    let revision = state.core_snapshot_revision;
    if state.last_snapshot_revision == Some(revision) {
        return;
    }
    state.last_snapshot_revision = Some(revision);
    state.stream_providers = has_stream_providers(&snapshot);
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
        hero_logos: state.logos.inputs(),
    });
}

fn has_stream_providers(snapshot: &Value) -> bool {
    let addons = snapshot
        .pointer("/addons/installed")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let plugin_names = snapshot
        .pointer("/plugins/scrapers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|scraper| scraper.get("enabled").and_then(Value::as_bool) != Some(false))
        .filter_map(|scraper| scraper.get("name").cloned())
        .collect::<Vec<_>>();
    core_value(
        "providerAvailabilityPlan",
        json!({"addons": addons, "pluginNames": plugin_names}),
    )
    .and_then(|plan| plan.get("hasStreamProviders").and_then(Value::as_bool))
    .unwrap_or(true)
}

pub(super) fn playback_blocked(state: &RendererState) -> bool {
    !state.stream_providers && state.settings.media_servers.is_empty()
}

pub(super) fn show_notice(state: &mut RendererState, key: &str) {
    let language = state.settings.language().to_owned();
    state.notice = Some((localized(key, &language), Instant::now()));
}

pub(super) fn apply_notice(state: &mut RendererState) {
    let language = state.settings.language().to_owned();
    state.home.playback_unavailable =
        playback_blocked(state).then(|| localized("playback.unavailable", &language));
    state.home.notice = None;
    let Some((text, shown)) = state.notice.as_ref() else {
        return;
    };
    let elapsed = shown.elapsed().as_secs_f32();
    if elapsed > NOTICE_SECONDS {
        state.notice = None;
        return;
    }
    let opacity = ((NOTICE_SECONDS - elapsed) / 0.4).clamp(0.0, 1.0);
    state.home.notice = Some((text.clone(), opacity));
}

pub(super) fn retire<T: Send + 'static>(value: T) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::{OnceLock, mpsc};
        type Junk = Box<dyn Send>;
        static REAPER: OnceLock<mpsc::Sender<Junk>> = OnceLock::new();
        let tx = REAPER.get_or_init(|| {
            let (tx, rx) = mpsc::channel::<Junk>();
            let _ = std::thread::Builder::new()
                .name("fluxa-retire".into())
                .spawn(move || {
                    fluxa_artwork::lower_thread_priority();
                    for junk in rx {
                        drop(junk);
                    }
                });
            tx
        });
        let _ = tx.send(Box::new(value));
    }
    #[cfg(target_arch = "wasm32")]
    drop(value);
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
    let focus_driven = state.home.focus_driven;
    retire(std::mem::replace(&mut state.home, projection.home));
    state.home.platform = platform;
    state.home.focus_driven = focus_driven;
    state.home.scroll_offset = scroll_offset;
    state.home.row_scroll_offsets = row_scroll_offsets;
    apply_notice(state);
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
    retire(std::mem::replace(&mut state.library, projection.library));
    let previous_folder = state.folder.id.clone();
    if state.folder.id != projection.folder.id {
        state.folder_tab = 0;
    }
    let folder_offsets = std::mem::take(&mut state.folder.row_scroll_offsets);
    retire(std::mem::replace(&mut state.folder, projection.folder));
    if state.folder.id == previous_folder {
        state.folder.row_scroll_offsets = folder_offsets;
    }
    let query = std::mem::take(&mut state.discover.query);
    retire(std::mem::replace(&mut state.discover, projection.discover));
    state.discover.query = query;
    let selected_day = state.calendar.selected_day;
    let previous_month = (state.calendar.year, state.calendar.month);
    retire(std::mem::replace(&mut state.calendar, projection.calendar));
    if previous_month == (state.calendar.year, state.calendar.month) {
        state.calendar.selected_day = selected_day;
    }
    let row_scroll_offsets = std::mem::take(&mut state.detail.row_scroll_offsets);
    let selected_season = state.detail.selected_season;
    let same_detail = state.detail.id == projection.detail.id;
    retire(std::mem::replace(&mut state.detail, projection.detail));
    if same_detail {
        state.detail.row_scroll_offsets = row_scroll_offsets;
        state.detail.selected_season = selected_season;
    }
    state.detail.resume = state.home.resume_for(&state.detail.id).cloned();
    state.trailers.set_targets(projection.trailer_targets);
    state.logos.set_targets(projection.logo_targets);
    state.home.apply_logos(state.logos.fetched());
    let settings_section = state.settings.active_section;
    let section_open = state.settings.section_open;
    let page_open = state.settings.page_open;
    let addon_url = std::mem::take(&mut state.settings.addon_url);
    let plugin_url = std::mem::take(&mut state.settings.plugin_url);
    let search = std::mem::take(&mut state.settings.search);
    let poster_fields = std::mem::take(&mut state.settings.poster_fields);
    let server_fields = std::mem::take(&mut state.settings.server_fields);
    let server_kind = state.settings.server_kind;
    let badge_fields = std::mem::take(&mut state.settings.badge_fields);
    let collection_fields = std::mem::take(&mut state.settings.collection_fields);
    let collections = std::mem::take(&mut state.settings.collections);
    let collection_edit = state.settings.collection_edit.take();
    let collection_status = state.settings.collection_status.take();
    let badge_outline = state.settings.badge_outline;
    let badge_status = state.settings.badge_status.take();
    let formatter_status = state.settings.formatter_status.take();
    let playback_summary = std::mem::take(&mut state.settings.playback_summary);
    retire(std::mem::replace(&mut state.settings, projection.settings));
    state.settings.playback_summary = playback_summary;
    state.settings.active_section = settings_section.min(fluxa_ui::SETTINGS_SECTIONS.len() - 1);
    state.settings.section_open = section_open;
    state.settings.page_open = page_open;
    state.settings.addon_url = addon_url;
    state.settings.plugin_url = plugin_url;
    state.settings.search = search;
    state.settings.server_fields = server_fields;
    state.settings.server_kind = server_kind;
    state.settings.badge_fields = badge_fields;
    state.settings.collection_fields = collection_fields;
    state.settings.collections = collections;
    state.settings.collection_edit = collection_edit;
    state.settings.collection_status = collection_status;
    state.settings.badge_outline = badge_outline;
    state.settings.badge_status = badge_status;
    state.settings.formatter_status = formatter_status;
    shortcuts::refresh(state);
    recent_searches::refresh(state);
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
