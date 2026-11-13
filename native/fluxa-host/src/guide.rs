use super::*;
use fluxa_ui::{NODE_GUIDE_CELL_BASE, NODE_GUIDE_CHANNEL_BASE};
use std::sync::mpsc::{Receiver, TryRecvError};
use web_time::{SystemTime, UNIX_EPOCH};

const HOUR_MS: i64 = 3_600_000;
const LOOKBACK_HOURS: i64 = 2;
const WINDOW_HOURS: i64 = 26;

#[derive(Default)]
pub(crate) struct GuideState {
    pending: Option<Receiver<Option<Value>>>,
    next_refresh: Option<Instant>,
    started: bool,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as i64)
}

fn utc_offset_ms() -> i64 {
    i64::from(chrono::Local::now().offset().local_minus_utc()) * 1000
}

pub(crate) fn tick(state: &mut RendererState) {
    if state.route != Route::Live {
        if state.guide_state.started {
            state.guide_state = GuideState::default();
            state.guide = Default::default();
        }
        return;
    }
    let fixture = crate::preview::guide_fixture();
    let now = fixture.as_ref().map_or_else(now_ms, |(now, _)| *now);
    state.guide.now_ms = now;
    state.guide.utc_offset_ms = if fixture.is_some() {
        0
    } else {
        utc_offset_ms()
    };
    state.guide.language = state.home.language.clone();
    if !state.guide_state.started {
        state.guide_state.started = true;
        state.guide.from_ms = now.div_euclid(HOUR_MS) * HOUR_MS - LOOKBACK_HOURS * HOUR_MS;
        state.guide.to_ms = state.guide.from_ms + WINDOW_HOURS * HOUR_MS;
        state.guide.loading = true;
        if let Some((_, guide)) = fixture {
            state.guide.apply(&guide);
            state.guide.loading = false;
            state.guide_state.next_refresh = Some(Instant::now() + Duration::from_secs(86_400));
            fluxa_ui::set_nav_live(true);
        }
    }
    if let Some(pending) = state.guide_state.pending.as_ref() {
        let result = match pending.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => None,
        };
        state.guide_state.pending = None;
        settle(state, result, now);
        return;
    }
    let due = state
        .guide_state
        .next_refresh
        .is_none_or(|at| Instant::now() >= at);
    if due && let Some(session) = state.session.as_ref() {
        state.guide_state.pending =
            Some(session.fetch_epg_guide(state.guide.from_ms, state.guide.to_ms, now));
    }
}

fn settle(state: &mut RendererState, result: Option<Value>, now: i64) {
    state.guide.loading = false;
    let failed = match result.as_ref() {
        Some(guide) => {
            state.guide.apply(guide);
            state.guide.failed
        }
        None => {
            state.guide.failed = true;
            true
        }
    };
    let delay = core_value(
        "epgRefreshDelayMs",
        json!({"lastEndMs": state.guide.last_end_ms(), "nowMs": now, "failed": failed}),
    )
    .and_then(|value| value.as_u64())
    .unwrap_or(60_000);
    state.guide_state.next_refresh = Some(Instant::now() + Duration::from_millis(delay));
}

pub(crate) fn activate_node(state: &mut RendererState, node: u64) -> bool {
    if state.route != Route::Live
        || !(NODE_GUIDE_CHANNEL_BASE..NODE_GUIDE_CELL_BASE + fluxa_ui::NODE_GUIDE_CELL_SPAN)
            .contains(&node)
    {
        return false;
    }
    let Some(channel) = state.guide.channel_for_node(node) else {
        return true;
    };
    let item = crate::actions::playback_item(&channel.raw, None, None);
    if playback_blocked(state) {
        show_notice(state, "playback.no_sources");
        return true;
    }
    state
        .pending_native_actions
        .push(NativeAction::StartPlayback { item });
    true
}
