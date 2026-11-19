use super::*;

pub(super) fn next_redraw(state: &mut RendererState) -> Option<Instant> {
    let now = Instant::now();
    let (all, cause) = wake(state, true)?;
    if all > now {
        state.wake_cause = cause;
        return Some(all);
    }
    if wake(state, false).is_some_and(|(ui, _)| ui <= now) {
        state.wake_cause = cause;
        return Some(all);
    }
    state.last_artwork_poll = now;
    let arrived = match state.gpu.as_mut() {
        Some(gpu) => gpu.artwork.ingest(&gpu.egui_context.clone()),
        None => false,
    };
    if arrived || state.projector.ready() {
        state.wake_cause = "artwork-arrived";
        return Some(now);
    }
    let (at, cause) = wake(state, true)?;
    state.wake_cause = cause;
    Some(at)
}

fn wake(state: &RendererState, artwork_polling: bool) -> Option<(Instant, &'static str)> {
    let now = Instant::now();
    if state.gpu.is_none() {
        return Some((now + Duration::from_millis(8), "no-gpu"));
    }
    let busy = [
        (state.player.is_some(), "player"),
        (state.touch_start.is_some(), "touch"),
        (state.scroll_motion.is_active(), "scroll-motion"),
        (state.pack_job.is_some(), "pack-job"),
        (state.catalog_job.is_some(), "catalog-job"),
        (state.pin_job.is_some(), "pin-job"),
        (state.pin_change_job.is_some(), "pin-change-job"),
        (!state.pending_native_actions.is_empty(), "native-actions"),
        (state.pending_resize.is_some(), "resize"),
    ]
    .into_iter()
    .find_map(|(active, name)| active.then_some(name));
    if let Some(name) = busy {
        return Some((now, name));
    }
    let since = state.last_frame;
    let revision = state.session.as_ref().map(SessionHandle::revision);
    if revision.is_some() && revision != state.session_revision {
        return Some((now, "session-revision"));
    }
    if artwork_polling
        && state
            .session
            .as_ref()
            .is_some_and(SessionHandle::has_queued_dispatches)
    {
        return Some((
            since.max(state.last_artwork_poll) + Duration::from_millis(4),
            "queued-dispatches",
        ));
    }
    let effects = state
        .session
        .as_ref()
        .is_some_and(|session| session.has_outstanding_effects())
        .then(|| since + Duration::from_millis(50));
    let artwork = (artwork_polling
        && state
            .gpu
            .as_ref()
            .is_some_and(|gpu| gpu.artwork.fetcher.has_pending() || gpu.artwork.converting()))
    .then(|| since.max(state.last_artwork_poll) + Duration::from_millis(50));
    let animation = state
        .gpu
        .as_ref()
        .and_then(|gpu| gpu.artwork.next_animation_frame());
    let updating = updates::busy(state).then(|| since + Duration::from_millis(120));
    let notice = (state.notice.is_some())
        .then(|| since + Duration::from_millis(16))
        .or_else(preview::next_check);
    let projecting = if state.projector.ready() {
        Some(now)
    } else {
        (artwork_polling && state.projector.pending())
            .then(|| since.max(state.last_artwork_poll) + Duration::from_millis(8))
    };
    let settling = (state.busy_until > now).then_some(state.busy_until);
    let home_refresh = (state.session.is_some()
        && state.window_focused
        && state.route == Route::Home
        && state.player.is_none())
    .then(|| state.home_refreshed_at + HOME_REFRESH_INTERVAL);
    let search = state.search_due;
    let trailer = trailer::next_redraw(state, now);
    let ambient = ambient::next_redraw(state);
    let caret = state
        .ui
        .focused()
        .and_then(|node| state.ui.node(node))
        .is_some_and(|node| node.kind == UiNodeKind::Input)
        .then(|| since + Duration::from_millis(265));
    [
        (artwork, "artwork-pending"),
        (projecting, "projector"),
        (animation, "artwork-animation"),
        (effects, "outstanding-effects"),
        (trailer, "trailer"),
        (search, "search"),
        (caret, "caret"),
        (home_refresh, "home-refresh"),
        (ambient, "ambient"),
        (skins::next_watch(), "skin-watch"),
        (updating, "updates"),
        (notice, "notice"),
        (settling, "settling"),
        (state.redraw_at, "egui"),
    ]
    .into_iter()
    .filter_map(|(at, name)| Some((at?, name)))
    .min_by_key(|(at, _)| *at)
}

pub(super) struct FrameTimer {
    started: Instant,
    last: Instant,
    phases: Vec<(&'static str, Duration)>,
}

impl FrameTimer {
    fn start() -> Self {
        let now = Instant::now();
        Self {
            started: now,
            last: now,
            phases: Vec::new(),
        }
    }

    pub(super) fn mark(&mut self, phase: &'static str) {
        let now = Instant::now();
        self.phases.push((phase, now - self.last));
        self.last = now;
    }

    fn report(self, label: &str) {
        let total = self.started.elapsed();
        if stats_enabled() {
            record_stats(&self, total);
        }
        if total < SLOW_FRAME {
            return;
        }
        let phases = self
            .phases
            .iter()
            .filter(|(_, spent)| *spent >= Duration::from_millis(2))
            .map(|(phase, spent)| format!("{phase}={}ms", spent.as_millis()))
            .collect::<Vec<_>>()
            .join(" ");
        host_log(format!("Slow {label}: {}ms {phases}", total.as_millis()));
    }
}

const SETTLE_DELAY: Duration = Duration::from_millis(400);

pub(super) const SLOW_FRAME: Duration = Duration::from_millis(24);

const STATS_WINDOW: Duration = Duration::from_secs(5);

static EXTRA_PASSES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub(super) fn note_passes(passes: usize) {
    EXTRA_PASSES.fetch_add(
        passes.saturating_sub(1),
        std::sync::atomic::Ordering::Relaxed,
    );
}

struct FrameStats {
    since: Instant,
    totals: Vec<f32>,
    works: Vec<f32>,
    phases: Vec<(&'static str, Vec<f32>)>,
    presented: usize,
    hitches: Vec<(&'static str, u32)>,
}

thread_local! {
    static CAUSES: std::cell::RefCell<Vec<(&'static str, u32)>> = const { std::cell::RefCell::new(Vec::new()) };
    static STATS: std::cell::RefCell<Option<FrameStats>> = const { std::cell::RefCell::new(None) };
}

pub(super) fn stats_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("FLUXA_FRAME_STATS").is_ok_and(|value| value != "0")
            || (cfg!(target_os = "android")
                && std::path::Path::new("/data/local/tmp/fluxa-frame-stats").exists())
    })
}

fn count_cause(cause: &'static str) {
    CAUSES.with(|causes| {
        let mut causes = causes.borrow_mut();
        match causes.iter_mut().find(|(name, _)| *name == cause) {
            Some((_, count)) => *count += 1,
            None => causes.push((cause, 1)),
        }
    });
}

fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    sorted[((sorted.len() - 1) as f32 * fraction).round() as usize]
}

fn record_stats(timer: &FrameTimer, total: Duration) {
    STATS.with(|cell| {
        let mut cell = cell.borrow_mut();
        let stats = cell.get_or_insert_with(|| FrameStats {
            since: Instant::now(),
            totals: Vec::new(),
            works: Vec::new(),
            phases: Vec::new(),
            presented: 0,
            hitches: Vec::new(),
        });
        if total >= Duration::from_millis(20)
            && let Some((phase, _)) = timer.phases.iter().max_by_key(|(_, spent)| *spent)
        {
            match stats.hitches.iter_mut().find(|(name, _)| name == phase) {
                Some((_, count)) => *count += 1,
                None => stats.hitches.push((phase, 1)),
            }
        }
        stats.totals.push(total.as_secs_f32() * 1000.0);
        let waited = timer
            .phases
            .iter()
            .find(|(phase, _)| *phase == "present")
            .map_or(Duration::ZERO, |(_, spent)| *spent);
        stats
            .works
            .push(total.saturating_sub(waited).as_secs_f32() * 1000.0);
        for (phase, spent) in &timer.phases {
            if *phase == "present" {
                stats.presented += 1;
            }
            let samples = match stats.phases.iter().position(|(name, _)| name == phase) {
                Some(index) => &mut stats.phases[index].1,
                None => {
                    stats.phases.push((phase, Vec::new()));
                    &mut stats.phases.last_mut().unwrap().1
                }
            };
            samples.push(spent.as_secs_f32() * 1000.0);
        }
        if stats.since.elapsed() < STATS_WINDOW {
            return;
        }
        let mut stats = cell.take().unwrap();
        stats.totals.sort_by(f32::total_cmp);
        stats.works.sort_by(f32::total_cmp);
        let phases = stats
            .phases
            .iter_mut()
            .map(|(phase, samples)| {
                samples.sort_by(f32::total_cmp);
                (*phase, percentile(samples, 0.95), samples.len(), percentile(samples, 0.5))
            })
            .filter(|(_, p95, _, _)| *p95 >= 0.5)
            .map(|(phase, p95, count, p50)| format!("{phase}={p50:.1}/{p95:.1}ms({count})"))
            .collect::<Vec<_>>()
            .join(" ");
        let causes = CAUSES.with(|causes| {
            causes
                .take()
                .iter()
                .map(|(name, count)| format!("{name}={count}"))
                .collect::<Vec<_>>()
                .join(" ")
        });
        host_log(format!(
            "Frame wake causes: {causes}; built {} nodes/frame, card cache hits={} misses={}",
            fluxa_ui::BUILT_NODES.swap(0, std::sync::atomic::Ordering::Relaxed) / stats.totals.len().max(1) as u64,
            fluxa_ui::CARD_HITS.swap(0, std::sync::atomic::Ordering::Relaxed),
            fluxa_ui::CARD_MISSES.swap(0, std::sync::atomic::Ordering::Relaxed),
        ));
        let mut gaps = crate::presenter::drain_intervals();
        gaps.retain(|gap| *gap < 250.0);
        let mut stages = crate::presenter::drain_stages();
        if !stages.is_empty() {
            let mut line = Vec::new();
            for (index, name) in ["acquire", "encode", "submit", "present"].into_iter().enumerate() {
                let mut column: Vec<f32> = stages.iter().map(|stage| stage[index]).collect();
                column.sort_by(f32::total_cmp);
                line.push(format!("{name} p50={:.1} p95={:.1}", percentile(&column, 0.5), percentile(&column, 0.95)));
            }
            stages.clear();
            host_log(format!("Presenter stages: {}", line.join(" | ")));
        }
        if !gaps.is_empty() {
            gaps.sort_by(f32::total_cmp);
            let over = |limit: f32| gaps.iter().filter(|gap| **gap > limit).count();
            host_log(format!(
                "Present gaps: {} presents, p50={:.1}ms p95={:.1}ms max={:.1}ms, >20ms={}, >33ms={}",
                gaps.len(),
                percentile(&gaps, 0.5),
                percentile(&gaps, 0.95),
                percentile(&gaps, 1.0),
                over(20.0),
                over(33.0),
            ));
        }
        if !stats.hitches.is_empty() {
            stats.hitches.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            let culprits = stats
                .hitches
                .iter()
                .map(|(phase, count)| format!("{phase}={count}"))
                .collect::<Vec<_>>()
                .join(" ");
            host_log(format!("Hitch culprits (frames >20ms, dominant phase): {culprits}"));
        }
        host_log(format!(
            "Frame stats: {} frames, {} extra passes, tree {:.1}ms/frame, {} presented, total p50={:.1}ms p95={:.1}ms max={:.1}ms | work p50={:.1}ms p95={:.1}ms | p95 {phases}",
            stats.totals.len(),
            EXTRA_PASSES.swap(0, std::sync::atomic::Ordering::Relaxed),
            crate::skins::TREE_NANOS.swap(0, std::sync::atomic::Ordering::Relaxed) as f64
                / 1e6
                / stats.totals.len() as f64,
            stats.presented,
            percentile(&stats.totals, 0.5),
            percentile(&stats.totals, 0.95),
            percentile(&stats.totals, 1.0),
            percentile(&stats.works, 0.5),
            percentile(&stats.works, 0.95),
        ));
    });
}

pub(super) fn render_frame(state: &mut RendererState) {
    state.last_frame = Instant::now();
    count_cause(state.wake_cause);
    fluxa_effects::flow(|| format!("frame start (woke by {})", state.wake_cause));
    let mut timer = FrameTimer::start();
    preview::reload_tokens();
    route_actions_to_session(state);
    timer.mark("actions");
    let revision_before = state.session_revision;
    pull_session_snapshot(state, &mut timer);
    timer.mark("snap-idle");
    updates::poll(state);
    timer.mark("snap-updates");
    crash::poll(state);
    timer.mark("snap-crash");
    profiles::poll(state);
    accounts::poll(state);
    if state.route == Route::Settings {
        accounts::poll_mdblist_lists(state);
        accounts::poll_tracker_profiles(state);
    }
    stream_badges::poll(state);
    collections::poll(state);
    player::pump(state);
    watch_together::tick(state);
    downloads::tick(state);
    stream_menu::tick(state);
    player::upload_frame(state);
    timer.mark("player");
    if state.gpu.is_none() {
        let count = GPU_WAIT_LOGS.fetch_add(1, Ordering::Relaxed);
        if count % 120 == 0 {
            host_log(format!(
                "Waiting for GPU initialization ({}x{})",
                state.size[0], state.size[1]
            ));
        }
    }
    if let Some(gpu) = state.gpu.as_mut()
        && let Some(size) = state.pending_resize.take()
    {
        gpu.resize(size);
    }
    welcome::poll(state);
    sync_home_from_core_snapshot(state);
    apply_notice(state);
    guide::tick(state);
    if let Some(gpu) = state.gpu.as_ref()
        && fluxa_ui::take_hero_wrap(fluxa_ui::gfx::Context::from_backend(&gpu.egui_context))
    {
        state.hero_page += 1;
        state.last_snapshot_revision = None;
    }
    input::tick_discover_search(state);
    ambient::tick(state);
    ambient::sync_settings(state);
    hero_catalogs::sync_settings(state);
    hero_badges::tick(state);
    prefetch::tick(state);
    detail_sections::sync_settings(state);
    trailer::tick(state);
    layout::sync_expanded(state);
    state.scroll_sync_models();
    poster_data::tick(state);
    episode_details::tick(state);
    card_logos::tick(state);
    timer.mark("sync");
    if state.route == Route::Home || matches!(state.active_scroll, Some(_)) {
        advance_home_inertia(state);
    }
    state.scroll_push_strips();
    let direct = state.touch_scrolled || state.scroll_motion.is_active();
    if let Some(gpu) = state.gpu.as_ref() {
        fluxa_ui::set_scroll_direct(
            fluxa_ui::gfx::Context::from_backend(&gpu.egui_context),
            direct,
        );
    }
    card_menu::tick(state);
    detail_modal::tick(state);
    apply_nav_layout(state);
    rebuild_current_ui(state);
    timer.mark("rebuild");
    let route = active_route(state);
    let mode = player::upscaling(&state.settings).to_owned();
    let fit = player::video_fit(&state.settings);
    if let Some(player) = state.player.as_mut() {
        player.fit = fit;
        player.compact = state.home.form_factor == UiFormFactorJson::Mobile;
        player.anime4k = mode != "off";
        player.upscaling = mode;
    }
    let watch_side = state.watch_panel
        && (!state.window_fullscreen || state.home.form_factor == UiFormFactorJson::Tv);
    let player_model = state.player.as_ref().map(|player| {
        let mut model = fluxa_ui::PlayerModel {
            watch_button: true,
            minimizable: state.home.form_factor == UiFormFactorJson::Desktop
                && player.can_minimize(),
            watch: watch_side.then(|| state.watch_view.clone()).flatten(),
            upscaling: player::upscaling(&state.settings).to_owned(),
            ..player.model()
        };
        model.video_hole = state.gpu.as_ref().is_some_and(|gpu| gpu.hdr_active());
        model.pip = state.pip;
        player::apply_options(&mut model, &state.settings);
        crate::preview::decorate_player(&mut model);
        model
    });
    let library_tab = state.library_tab;
    if state.home.form_factor == UiFormFactorJson::Tv {
        state.keyboard_focus_visible = true;
    }
    let focused = state
        .keyboard_focus_visible
        .then(|| state.ui.focused())
        .flatten();
    fluxa_ui::set_nav_focus(focused);
    let safe_bottom = state.safe_bottom;
    let logical_size = logical_surface_size(state);
    if route != Route::Home && route != Route::Player {
        let max_offset = screen_scroll_max(state);
        let key = ScrollKey::Screen(route);
        let offset = state.scroll_get(key);
        if offset > max_offset {
            state.scroll_set(key, max_offset);
        }
    }
    let scroll_y = state.scroll_get(ScrollKey::Screen(route));
    let scale = state.scale();
    if let (Some(url), Some(gpu)) = (state.backdrop_prefetch.take(), state.gpu.as_mut()) {
        let width = (gpu.config.width as f32 / scale).round().max(1.0);
        gpu.artwork.texture_for_priority(
            Some(&url),
            fluxa_ui::backdrop_target_size(width, scale),
            ArtworkFetchPriority::Hero,
        );
    }
    presence::update(current_presence(state));
    timer.mark("prepare");
    let menu = card_menu::view(state);
    let ambient_view = ambient::view(state);
    let caret = state
        .ui
        .focused()
        .and_then(|node| state.ui.node(node))
        .filter(|node| node.kind == UiNodeKind::Input)
        .map(|node| {
            egui::Rect::from_min_size(
                egui::pos2(node.bounds.x, node.bounds.y),
                egui::vec2(node.bounds.width, node.bounds.height),
            )
        });
    if let Some(gpu) = state.gpu.as_ref() {
        fluxa_ui::set_input_caret(
            fluxa_ui::gfx::Context::from_backend(&gpu.egui_context),
            caret,
        );
    }
    let focus_driven = state.keyboard_focus_visible;
    if state.home.focus_driven != focus_driven {
        state.home.focus_driven = focus_driven;
        state.scroll_reset(ScrollKey::Screen(Route::Home));
    }
    let mirrored = state.mirrored();
    state.settings.unreachable_addon_hosts = fluxa_effects::unreachable_addon_hosts();
    let watch_modal_hidden = state.player.is_some();
    let loading = state.projector.pending()
        || state
            .session
            .as_ref()
            .is_some_and(SessionHandle::has_queued_dispatches)
        || state.gpu.as_ref().is_some_and(|gpu| {
            gpu.artwork.fetcher.has_foreground_pending() || gpu.artwork.converting()
        });
    if loading {
        state.busy_until = Instant::now() + SETTLE_DELAY;
    }
    let settled = state.busy_until <= Instant::now();
    if let Some(gpu) = state.gpu.as_ref() {
        fluxa_ui::set_ui_settled(
            fluxa_ui::gfx::Context::from_backend(&gpu.egui_context),
            settled,
        );
    }
    if let Some(gpu) = state.gpu.as_ref() {
        fluxa_ui::set_detail_modal_open(
            fluxa_ui::gfx::Context::from_backend(&gpu.egui_context),
            state.detail_modal,
        );
        fluxa_ui::set_screen_covered(
            fluxa_ui::gfx::Context::from_backend(&gpu.egui_context),
            input_layer(state) != InputLayer::Screen,
        );
    }
    skins::poll_dev();
    skins::begin_frame(
        matches!(
            state.wake_cause,
            "egui" | "artwork-pending" | "artwork-arrived" | "artwork-animation" | "skin-watch"
        ) && state.egui_events.is_empty()
            && state.session_revision == revision_before,
    );
    let render_result = {
        let RendererState {
            gpu,
            welcome,
            home,
            library,
            discover,
            folder,
            folder_tab,
            calendar,
            guide,
            detail,
            settings,
            profiles,
            watch_view,
            picker_background,
            egui_events,
            field_text,
            modifiers,
            pre_present,
            window_fullscreen,
            keyboard_inset,
            font_scale,
            ..
        } = state;
        let events = std::mem::take(egui_events);
        let modifiers = *modifiers;
        gpu.as_mut().map(|gpu| {
            gpu.density = scale;
            gpu.keyboard = *keyboard_inset;
            gpu.apply_font_scale(*font_scale);
            gpu.apply_reduced_motion();
            gpu.mirror = mirrored;
            gpu.window_fullscreen = *window_fullscreen;
            fluxa_ui::set_pending_field(
                fluxa_ui::gfx::Context::from_backend(&gpu.egui_context),
                field_text.take(),
            );
            gpu.render(
                route,
                state.route,
                home,
                library,
                library_tab,
                discover,
                folder,
                *folder_tab,
                calendar,
                guide,
                detail,
                settings,
                profiles.as_mut(),
                welcome.as_mut(),
                picker_background.as_deref(),
                player_model.as_ref(),
                focused,
                safe_bottom,
                scroll_y,
                events,
                modifiers,
                pre_present.as_ref(),
                menu.as_ref(),
                ambient_view.as_ref(),
                watch_view.as_mut().filter(|_| !watch_modal_hidden),
                &mut timer,
            )
        })
    };
    if let Some(render_result) = render_result {
        match render_result {
            Ok(frame) => {
                state.cursor = frame.cursor;
                if frame.copied.is_some() {
                    state.copied = frame.copied;
                }
                state.wants_keyboard = frame.wants_keyboard;
                state.access = AccessFrame {
                    runs: frame.text_runs,
                    clicks: frame.click_targets,
                    field: frame.field,
                    caret: frame.caret,
                };
                state.redraw_at = Instant::now().checked_add(frame.repaint_delay);
                ime::sync(state);
                events::notify_input_focus(state);
                let mut layout = frame.layout;
                hover_tip::update(state, layout.hovered);
                detail_modal::apply(state);
                if let Some(request) = layout.profiles.take() {
                    profiles::handle(state, request);
                }
                if let Some(request) = layout.welcome.take() {
                    welcome::handle(state, request);
                }
                welcome::schedule(state);
                if let Some(position) = layout.seek_to {
                    player::command(state, VideoCommand::SeekTo(position));
                }
                if let Some(gesture) = layout.player_gesture {
                    player::gesture(state, gesture);
                }
                if route == Route::Player {
                    player::speed_hold(state, layout.player_speed_hold);
                }
                player::want_filmstrip(state, layout.filmstrip.take());
                player::hover_seek(state, layout.seek_hover);
                if layout.activated == Some(fluxa_ui::NODE_WINDOW_FULLSCREEN) {
                    layout.activated = None;
                    state.fullscreen_toggle = true;
                }
                apply_pointer_results(state, route, &layout);
                if matches!(route, Route::Discover | Route::Folder) {
                    for request in &layout.load_more {
                        state
                            .pending_native_actions
                            .push(NativeAction::CoreCommand {
                                command: request.clone(),
                            });
                    }
                }
                if route == Route::Home && state.home.form_factor == UiFormFactorJson::Desktop {
                    let viewport = logical_viewport(state);
                    let rows = state.home.content_rows_with_kind().count();
                    for row in 0..rows {
                        let offset = state.scroll_get(ScrollKey::Row(Route::Home, row));
                        request_home_row_load_more_at(state, row, viewport, offset);
                    }
                }
                if let Some((key, value)) = layout.filter_change.as_ref() {
                    apply_choice(state, route, key, value.clone());
                }
                if let Some(gpu) = state.gpu.as_ref() {
                    state.scroll_regions = (
                        route,
                        fluxa_ui::take_scroll_regions(fluxa_ui::gfx::Context::from_backend(
                            &gpu.egui_context,
                        )),
                    );
                }
                rebuild_ui_from_layout(state, &layout, logical_size);
                state.rendered_layout = Some((route, layout));
            }
            Err(error) => host_log(format!("Frame skipped: {error}")),
        }
    }
    timer.mark("layout");
    if state.flow_episodes != state.detail.episodes.len() {
        state.flow_episodes = state.detail.episodes.len();
        fluxa_effects::flow(|| {
            format!(
                "frame drawn with {} episodes (woke by {})",
                state.flow_episodes, state.wake_cause
            )
        });
    }
    timer.report(&format!(
        "frame on {} (woke by {})",
        state.route.as_str(),
        state.wake_cause
    ));
}

fn apply_nav_layout(state: &RendererState) {
    let setting = |key: &str, fallback: &'static str| {
        state
            .settings
            .value(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or(fallback)
            .to_owned()
    };
    let hosts_nav = !state.player_full() && state.profiles.is_none() && state.welcome.is_none();
    let layout = if hosts_nav && !state.mirrored() {
        setting("navLayout", "topbar")
    } else {
        "topbar".to_owned()
    };
    fluxa_ui::set_nav_sidebar(&layout, &setting("navSidebarMode", "hover"));
    fluxa_ui::set_nav_search(&state.discover.query);
}
