use super::*;

pub(super) fn next_redraw(state: &mut RendererState) -> Option<Instant> {
    let now = Instant::now();
    let busy = state.gpu.is_none()
        || state.player.is_some()
        || state.touch_start.is_some()
        || state.scroll_velocity != 0.0
        || state.pack_job.is_some()
        || !state.pending_native_actions.is_empty()
        || state.pending_resize.is_some();
    if busy {
        return Some(now);
    }
    let revision = state.session.as_ref().map(SessionHandle::revision);
    if revision.is_some() && revision != state.session_revision {
        return Some(now);
    }
    if state
        .session
        .as_ref()
        .is_some_and(SessionHandle::has_queued_dispatches)
    {
        return Some(now + Duration::from_millis(4));
    }
    let effects = state
        .session
        .as_ref()
        .is_some_and(|session| session.has_outstanding_effects())
        .then(|| now + Duration::from_millis(50));
    let artwork = state
        .gpu
        .as_ref()
        .is_some_and(|gpu| gpu.artwork.fetcher.has_pending())
        .then(|| now + Duration::from_millis(50));
    let animation = state
        .gpu
        .as_ref()
        .and_then(|gpu| gpu.artwork.next_animation_frame());
    let projecting = state
        .projector
        .pending()
        .then(|| now + Duration::from_millis(8));
    let trailer = trailer::next_redraw(state, now);
    let caret = state
        .ui
        .focused()
        .and_then(|node| state.ui.node(node))
        .is_some_and(|node| node.kind == UiNodeKind::Input)
        .then(|| now + Duration::from_millis(265));
    let artwork = [artwork, projecting, animation, effects, trailer, caret]
        .into_iter()
        .flatten()
        .min();
    match (state.redraw_at, artwork) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
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

pub(super) const SLOW_FRAME: Duration = Duration::from_millis(24);

pub(super) fn render_frame(state: &mut RendererState) {
    let mut timer = FrameTimer::start();
    route_actions_to_session(state);
    timer.mark("actions");
    pull_session_snapshot(state);
    timer.mark("snapshot");
    profiles::poll(state);
    accounts::poll(state);
    stream_badges::poll(state);
    collections::poll(state);
    player::pump(state);
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
    if let Some(size) = state.pending_resize.take()
        && let Some(gpu) = state.gpu.as_mut()
    {
        gpu.resize(size);
    }
    sync_home_from_core_snapshot(state);
    shorts::tick(state);
    trailer::tick(state);
    poster_data::tick(state);
    timer.mark("sync");
    if state.route == Route::Home
        || matches!(
            state.active_scroll,
            Some(
                HomeScrollTarget::ScreenVertical
                    | HomeScrollTarget::DetailRow(_)
                    | HomeScrollTarget::FolderRow(_)
            )
        )
    {
        advance_home_inertia(state);
    }
    card_menu::tick(state);
    rebuild_current_ui(state);
    timer.mark("rebuild");
    let route = active_route(state);
    let mode = player::upscaling(&state.settings).to_owned();
    if let Some(player) = state.player.as_mut() {
        player.anime4k = mode != "off";
        player.upscaling = mode;
    }
    let player_model = state.player.as_ref().map(|player| {
        let mut model = fluxa_ui::PlayerModel {
            upscaling: player::upscaling(&state.settings).to_owned(),
            ..player.model()
        };
        player::apply_options(&mut model, &state.settings);
        model
    });
    let library_tab = state.library_tab;
    let focused = state
        .keyboard_focus_visible
        .then(|| state.ui.focused())
        .flatten();
    let safe_bottom = state.safe_bottom;
    let logical_size = logical_surface_size(state);
    if route != Route::Home && route != Route::Player {
        let viewport = Viewport::new(
            logical_size[0],
            logical_size[1],
            state.home.form_factor.into(),
        )
        .with_platform(state.home.platform)
        .with_safe_bottom(safe_bottom);
        let max_offset = screen_scroll_max(state, viewport);
        if let Some(offset) = state.screen_scroll_offsets.get_mut(&route) {
            *offset = offset.clamp(0.0, max_offset);
        }
    }
    let scroll_y = state
        .screen_scroll_offsets
        .get(&route)
        .copied()
        .unwrap_or(0.0);
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
        fluxa_ui::set_input_caret(&gpu.egui_context, caret);
    }
    let render_result = {
        let RendererState {
            gpu,
            home,
            library,
            discover,
            folder,
            folder_tab,
            calendar,
            shorts,
            detail,
            settings,
            profiles,
            picker_background,
            egui_events,
            modifiers,
            pre_present,
            ..
        } = state;
        let events = std::mem::take(egui_events);
        let modifiers = *modifiers;
        gpu.as_mut().map(|gpu| {
            gpu.density = scale;
            gpu.render(
                route,
                home,
                library,
                library_tab,
                discover,
                folder,
                *folder_tab,
                calendar,
                shorts,
                detail,
                settings,
                profiles.as_mut(),
                picker_background.as_deref(),
                player_model.as_ref(),
                focused,
                safe_bottom,
                scroll_y,
                events,
                modifiers,
                pre_present.as_ref(),
                menu.as_ref(),
                &mut timer,
            )
        })
    };
    if let Some(render_result) = render_result {
        match render_result {
            Ok(frame) => {
                state.cursor = frame.cursor;
                state.wants_keyboard = frame.wants_keyboard;
                state.redraw_at = Instant::now().checked_add(frame.repaint_delay);
                let mut layout = frame.layout;
                if let Some(outcome) = frame.menu_outcome {
                    card_menu::apply(state, outcome);
                }
                if let Some(request) = layout.profiles.take() {
                    profiles::handle(state, request);
                }
                if let Some(position) = layout.seek_to {
                    player::command(state, VideoCommand::SeekTo(position));
                }
                if let Some(gesture) = layout.player_gesture {
                    player::gesture(state, gesture);
                }
                if route == Route::Player {
                    player::speed_hold(state, layout.player_speed_hold);
                }
                player::hover_seek(state, layout.seek_hover);
                if let (Some(max), Some(player)) = (layout.scroll_max, state.player.as_mut()) {
                    player.sources_scroll_max = max;
                }
                if let Some(delta) = layout.short_step {
                    shorts::step(state, delta);
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
                if let Some((key, value)) = layout.filter_change.as_ref() {
                    apply_choice(state, route, key, value.clone());
                }
                rebuild_ui_from_layout(state, &layout, logical_size);
                state.rendered_layout = Some((route, layout));
            }
            Err(error) => host_log(format!("Frame skipped: {error}")),
        }
    }
    timer.mark("layout");
    timer.report(&format!("frame on {}", state.route.as_str()));
}
