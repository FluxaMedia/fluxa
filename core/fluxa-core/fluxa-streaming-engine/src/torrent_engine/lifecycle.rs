use super::*;

#[derive(Default)]
pub(super) struct TorrentFileFocus {
    pub(super) primary_video: Option<usize>,
    pub(super) auxiliary_files: HashSet<usize>,
}

#[derive(Clone, Copy)]
pub(super) struct TorrentLifecycle {
    pub(super) last_accessed: Instant,
    pub(super) prewarmed: bool,
    pub(super) active: bool,
    pub(super) estimated_cache_bytes: u64,
}

#[derive(Default)]
pub(super) struct TorrentRuntimeState {
    pub(super) known_links: HashMap<String, usize>,
    pub(super) prioritized_files: HashMap<usize, TorrentFileFocus>,
    pub(super) playback_windows: HashMap<(usize, usize), PlaybackWindow>,
    pub(super) playback_sessions: HashMap<(usize, usize), PlaybackSession>,
    pub(super) torrent_cancellations: HashMap<usize, CancellationToken>,
    pub(super) lifecycle: HashMap<usize, TorrentLifecycle>,
    pub(super) active_torrent: Option<usize>,
}

pub(super) fn touch_torrent_lifecycle(state: &EngineState, torrent_id: usize, active: bool) {
    if let Ok(mut runtime) = state.runtime.lock() {
        let entry = runtime
            .lifecycle
            .entry(torrent_id)
            .or_insert(TorrentLifecycle {
                last_accessed: Instant::now(),
                prewarmed: !active,
                active,
                estimated_cache_bytes: 0,
            });
        entry.last_accessed = Instant::now();
        entry.active |= active;
        if active {
            entry.prewarmed = false;
        }
    }
}

pub(super) fn should_deactivate_prewarm(
    lifecycle: &HashMap<usize, TorrentLifecycle>,
    torrent_id: usize,
) -> bool {
    lifecycle
        .get(&torrent_id)
        .is_some_and(|entry| entry.prewarmed && !entry.active)
}

pub(super) async fn activate_torrent(state: &EngineState, torrent_id: usize) {
    let previous = state
        .runtime
        .lock()
        .map(|mut runtime| runtime.active_torrent.replace(torrent_id))
        .ok()
        .flatten();
    if let Some(previous) = previous.filter(|previous| *previous != torrent_id) {
        deactivate_torrent(state, previous).await;
    }
    if let Ok(mut runtime) = state.runtime.lock() {
        let entry = runtime
            .lifecycle
            .entry(torrent_id)
            .or_insert(TorrentLifecycle {
                last_accessed: Instant::now(),
                prewarmed: false,
                active: true,
                estimated_cache_bytes: 0,
            });
        entry.active = true;
        entry.prewarmed = false;
        entry.last_accessed = Instant::now();
    }
}

pub(super) async fn deactivate_torrent(state: &EngineState, torrent_id: usize) {
    cancel_torrent_root(state, torrent_id);
    clear_playback_telemetry(state, torrent_id);
    let files = state
        .runtime
        .lock()
        .map(|mut runtime| {
            // The focus cache short-circuits a repeat request for the same
            // file, so leaving it behind means the next play never re-applies
            // the file selection or the streaming window this call clears.
            runtime.prioritized_files.remove(&torrent_id);
            let windows = &mut runtime.playback_windows;
            let files = windows
                .keys()
                .filter(|(id, _)| *id == torrent_id)
                .map(|(_, file_id)| *file_id)
                .collect::<Vec<_>>();
            windows.retain(|(id, _), _| *id != torrent_id);
            files
        })
        .unwrap_or_default();
    for file_id in files {
        let _ = state
            .api
            .api_clear_streaming_window(TorrentIdOrHash::Id(torrent_id), file_id);
    }
    if let Ok(mut runtime) = state.runtime.lock() {
        for session in runtime
            .playback_sessions
            .extract_if(|(id, _), _| *id == torrent_id)
            .map(|(_, session)| session)
        {
            session.cancel.cancel();
        }
    }
    if let Ok(mut runtime) = state.runtime.lock()
        && let Some(entry) = runtime.lifecycle.get_mut(&torrent_id)
    {
        entry.active = false;
        entry.last_accessed = Instant::now();
    }
    if let Ok(mut runtime) = state.runtime.lock()
        && runtime.active_torrent == Some(torrent_id)
    {
        runtime.active_torrent = None;
    }
    let _ = state
        .api
        .api_torrent_action_pause(TorrentIdOrHash::Id(torrent_id))
        .await;
}

pub(super) fn clear_playback_telemetry(state: &EngineState, torrent_id: usize) {
    if let Ok(mut telemetry) = state.telemetry.lock() {
        clear_telemetry_for_torrent(&mut telemetry, torrent_id);
    }
}

pub(super) fn clear_telemetry_for_torrent(telemetry: &mut TelemetryState, torrent_id: usize) {
    telemetry.records.retain(|(id, _), _| *id != torrent_id);
    telemetry.active_sessions.remove(&torrent_id);
}

pub(super) fn torrent_worker_threads() -> usize {
    let available = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(2);
    let platform_default = if cfg!(target_os = "android") {
        available.clamp(2, 4)
    } else {
        available.clamp(2, 16)
    };
    std::env::var("FLUXA_TORRENT_WORKERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| (1..=32).contains(value))
        .unwrap_or(platform_default)
}

/// Prewarming resolves metadata and discovers peers but must not keep an idle
/// torrent transferring indefinitely. Pausing retains the files and
/// fast-resume/session records; a real stream request resumes it above.
pub(super) async fn prewarm_reaper(state: EngineState) {
    const PREWARM_IDLE_TTL: Duration = Duration::from_secs(15 * 60);
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let expired = state
            .runtime
            .lock()
            .map(|mut runtime| {
                runtime
                    .lifecycle
                    .iter_mut()
                    .filter_map(|(&torrent_id, entry)| {
                        if entry.prewarmed
                            && !entry.active
                            && entry.last_accessed.elapsed() >= PREWARM_IDLE_TTL
                        {
                            entry.prewarmed = false;
                            Some(torrent_id)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for torrent_id in expired {
            let _ = state
                .api
                .api_torrent_action_pause(TorrentIdOrHash::Id(torrent_id))
                .await;
            debug_log(format!(
                "[TorrServer] paused idle prewarm torrent={torrent_id}"
            ));
        }
        enforce_cache_limit(&state).await;
    }
}

pub(super) async fn enforce_cache_limit(state: &EngineState) {
    let Some(limit) = state.cache_limit_bytes.lock().ok().and_then(|limit| *limit) else {
        return;
    };
    let snapshots = state
        .api
        .api_torrent_list_ext(ApiTorrentListOpts { with_stats: true });
    let now = Instant::now();
    let stale_access = now
        .checked_sub(Duration::from_secs(365 * 24 * 60 * 60))
        .unwrap_or(now);
    let mut entries = state
        .runtime
        .lock()
        .map(|runtime| {
            snapshots
                .torrents
                .iter()
                .filter_map(|torrent| {
                    let id = torrent.id?;
                    let lifecycle = runtime.lifecycle.get(&id);
                    Some((
                        id,
                        runtime.active_torrent == Some(id)
                            || lifecycle.is_some_and(|entry| entry.active)
                            || torrent.stats.as_ref().is_some_and(|stats| {
                                matches!(&stats.state, TorrentStatsState::Live)
                            }),
                        lifecycle
                            .map(|entry| entry.last_accessed)
                            .unwrap_or(stale_access),
                        torrent
                            .stats
                            .as_ref()
                            .map(|stats| stats.progress_bytes)
                            .unwrap_or(0),
                    ))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut used = entries.iter().map(|entry| entry.3).sum::<u64>();
    if used <= limit {
        return;
    }
    entries.sort_by_key(|entry| entry.2);
    for (torrent_id, active, _, bytes) in entries {
        if active || used <= limit {
            continue;
        }
        if state
            .api
            .api_torrent_action_delete(TorrentIdOrHash::Id(torrent_id))
            .await
            .is_ok()
        {
            used = used.saturating_sub(bytes);
            if let Ok(mut runtime) = state.runtime.lock() {
                runtime
                    .known_links
                    .retain(|_, known_id| *known_id != torrent_id);
                runtime.lifecycle.remove(&torrent_id);
                runtime.prioritized_files.remove(&torrent_id);
                runtime
                    .playback_windows
                    .retain(|(id, _), _| *id != torrent_id);
                for session in runtime
                    .playback_sessions
                    .extract_if(|(id, _), _| *id == torrent_id)
                    .map(|(_, session)| session)
                {
                    session.cancel.cancel();
                }
            }
            cancel_torrent_root(state, torrent_id);
            clear_playback_telemetry(state, torrent_id);
            debug_log(format!(
                "[TorrServer] evicted inactive torrent={torrent_id} for cache limit"
            ));
        }
    }
}

pub(super) fn playback_window_for(
    state: &EngineState,
    torrent_id: usize,
    file_id: usize,
) -> Option<PlaybackWindow> {
    state
        .runtime
        .lock()
        .ok()?
        .playback_windows
        .get(&(torrent_id, file_id))
        .copied()
}

pub(super) fn playback_session_for(
    state: &EngineState,
    torrent_id: usize,
    file_id: usize,
    offset: u64,
) -> CancellationToken {
    let key = (torrent_id, file_id);
    let seek_threshold = playback_window_for(state, torrent_id, file_id)
        .map(|window| (window.warm_ahead_bytes / 4).max(1))
        .unwrap_or(1);
    let previous_offset = state.runtime.lock().ok().and_then(|runtime| {
        runtime
            .playback_windows
            .get(&key)
            .map(|window| window.playback_offset)
    });
    // Resolved before the lock: torrent_cancellation_token takes the same
    // non-reentrant runtime mutex, and reaching for it while holding the guard
    // deadlocks the thread against itself and wedges the whole HTTP server.
    let parent = torrent_cancellation_token(state, torrent_id);
    if let Ok(mut runtime) = state.runtime.lock() {
        let sessions = &mut runtime.playback_sessions;
        let seek =
            previous_offset.is_some_and(|previous| offset.abs_diff(previous) > seek_threshold);
        if seek && let Some(previous) = sessions.get(&key) {
            previous.cancel.cancel();
        }
        let generation = sessions
            .get(&key)
            .map(|session| session.generation)
            .unwrap_or(0)
            + u64::from(seek || !sessions.contains_key(&key));
        let session = sessions.entry(key).or_insert_with(|| PlaybackSession {
            generation,
            cancel: parent.child_token(),
        });
        if seek {
            *session = PlaybackSession {
                generation,
                cancel: parent.child_token(),
            };
        }
        return session.cancel.clone();
    }
    parent.child_token()
}

pub(super) fn torrent_cancellation_token(
    state: &EngineState,
    torrent_id: usize,
) -> CancellationToken {
    state
        .runtime
        .lock()
        .map(|mut runtime| {
            runtime
                .torrent_cancellations
                .entry(torrent_id)
                .or_insert_with(CancellationToken::new)
                .clone()
        })
        // A poisoned bookkeeping lock must not break media serving. The
        // standalone token still keeps the reader's local cancellation valid.
        .unwrap_or_else(|_| CancellationToken::new())
}

pub(super) fn cancel_torrent_root(state: &EngineState, torrent_id: usize) {
    if let Ok(mut runtime) = state.runtime.lock()
        && let Some(token) = runtime.torrent_cancellations.remove(&torrent_id)
    {
        token.cancel();
    }
}

/// MPV/FFmpeg may issue a tiny distant cue/index read without seeking the
/// primary playback stream. Keep the live scheduler window in that case.
pub(super) fn is_probe_range(
    state: &EngineState,
    torrent_id: usize,
    file_id: usize,
    offset: u64,
    length: u64,
) -> bool {
    const MAX_PROBE_BYTES: u64 = 2 * 1024 * 1024;
    let Some(window) = playback_window_for(state, torrent_id, file_id) else {
        return false;
    };
    is_probe_for_window(window, offset, length, MAX_PROBE_BYTES)
}

pub(super) fn is_probe_for_window(
    window: PlaybackWindow,
    offset: u64,
    length: u64,
    max_probe_bytes: u64,
) -> bool {
    length <= max_probe_bytes
        && offset.abs_diff(window.playback_offset)
            > (window.warm_ahead_bytes / 4).max(max_probe_bytes)
}

pub(super) fn store_playback_window(state: &EngineState, window: PlaybackWindow) {
    if let Ok(mut runtime) = state.runtime.lock() {
        runtime
            .playback_windows
            .insert((window.torrent_id, window.file_id), window);
    }
}

pub(super) fn playback_phase(
    stats: Option<&librqbit::TorrentStats>,
    buffered: u64,
    target: u64,
    window: Option<PlaybackWindow>,
) -> &'static str {
    match stats.map(|stats| stats.state) {
        None | Some(TorrentStatsState::Initializing) => "resolving_metadata",
        Some(TorrentStatsState::Error) => "error",
        Some(TorrentStatsState::Paused) => "stalled",
        Some(TorrentStatsState::Live)
            if window.is_some_and(|window| {
                window
                    .seek_started_at
                    .is_some_and(|started| started.elapsed() < Duration::from_secs(2))
            }) =>
        {
            "seeking"
        }
        Some(TorrentStatsState::Live) if buffered >= target && target > 0 => "streaming",
        Some(TorrentStatsState::Live) if window.is_some_and(|window| window.was_ready) => {
            "rebuffering"
        }
        Some(TorrentStatsState::Live) if buffered > 0 => "buffering_startup",
        Some(TorrentStatsState::Live) => "connecting_peers",
    }
}

pub(super) fn set_streaming_window(
    state: &EngineState,
    torrent_id: usize,
    file_id: usize,
    offset: u64,
) {
    // The forked picker serves urgent pieces first, then the warm window, then
    // normal selected-file ordering. A new offset replaces the old window.
    let (urgent, warm) = playback_window_for(state, torrent_id, file_id)
        .map(|window| (window.urgent_ahead_bytes, window.warm_ahead_bytes))
        .unwrap_or_else(|| {
            state
                .preload_size
                .lock()
                .map(|value| (*value, value.saturating_mul(2).max(32 * 1024 * 1024)))
                .unwrap_or((10 * 1024 * 1024, 32 * 1024 * 1024))
        });
    let _ = state.api.api_set_streaming_window_with_priority(
        TorrentIdOrHash::Id(torrent_id),
        file_id,
        offset,
        urgent,
        warm,
    );
}
