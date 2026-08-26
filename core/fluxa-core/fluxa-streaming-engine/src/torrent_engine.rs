use axum::body::Body;
use axum::extract::{Query, State, connect_info::ConnectInfo};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use librqbit::api::{ApiTorrentListOpts, TorrentDetailsResponse, TorrentIdOrHash};
use librqbit::dht::PersistentDhtConfig;
use librqbit::{
    AddTorrent, AddTorrentOptions, Api, PeerConnectionOptions, Session, SessionOptions,
    SessionPersistenceConfig, TorrentStatsState,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::io::SeekFrom;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::oneshot;
use tokio_util::io::ReaderStream;
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};

#[derive(Deserialize)]
struct TorrRequest {
    action: String,
    link: Option<String>,
    hash: Option<String>,
    title: Option<String>,
    #[serde(default)]
    save_to_db: bool,
    // Optional file index to focus on right after add — prevents rqbit
    // from spreading peer slots across every file in the torrent.
    file_id: Option<usize>,
    #[serde(default)]
    role: FileRole,
    /// Metadata/peer discovery requested before playback. Prewarmed torrents
    /// are paused after their idle TTL; their on-disk fast-resume data stays.
    #[serde(default)]
    prewarm: bool,
}

#[derive(Deserialize)]
struct TorrSettings {
    #[serde(rename = "PreloadSize")]
    preload_size: Option<u64>,
    /// Zero means unlimited. A positive value enables LRU eviction of
    /// inactive torrents only; active playback is never an eviction target.
    #[serde(rename = "CacheLimitMb")]
    cache_limit_mb: Option<u64>,
    #[serde(rename = "StreamBufferBytes")]
    stream_buffer_bytes: Option<u64>,
    #[serde(alias = "deviceBudget", alias = "DeviceBudget")]
    device_budget: Option<DeviceBudgetSettings>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceBudgetSettings {
    torrent_preload_mb: Option<u64>,
    torrent_cache_mb: Option<u64>,
    stream_reader_buffer_bytes: Option<u64>,
}

#[derive(Deserialize)]
struct StreamQuery {
    link: String,
    title: Option<String>,
    index: Option<usize>,
    stat: Option<String>,
    access_token: Option<String>,
    #[serde(default)]
    role: FileRole,
    #[serde(alias = "durationMs")]
    duration_ms: Option<u64>,
}

#[derive(Clone, Copy)]
struct PlaybackWindow {
    torrent_id: usize,
    file_id: usize,
    playback_offset: u64,
    requested_end: u64,
    contiguous_ready_bytes: u64,
    estimated_bitrate_bps: u64,
    urgent_ahead_bytes: u64,
    warm_ahead_bytes: u64,
    smoothed_download_bps: f64,
    seek_generation: u64,
    was_ready: bool,
    seek_started_at: Option<Instant>,
    updated_at: Instant,
}

#[derive(Clone)]
struct PlaybackSession {
    generation: u64,
    cancel: CancellationToken,
}

#[derive(Default, Clone, Copy)]
struct PlaybackTelemetry {
    first_frame_ms: Option<u64>,
    stall_count: u64,
    stall_duration_ms: u64,
}

#[derive(Clone)]
struct ActiveTelemetrySession {
    id: String,
    generation: u64,
}

#[derive(Default)]
struct TelemetryState {
    active_sessions: HashMap<usize, ActiveTelemetrySession>,
    records: HashMap<(usize, String), PlaybackTelemetry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TelemetryEvent {
    link: String,
    session_id: String,
    session_generation: u64,
    event: String,
    elapsed_ms: Option<u64>,
}

struct CancellableReader<R> {
    inner: R,
    // Keep the cancellation future alive while the underlying rqbit reader is
    // pending so cancelling its token wakes this reader immediately.
    cancellation: Pin<Box<WaitForCancellationFutureOwned>>,
}

impl<R> CancellableReader<R> {
    fn new(inner: R, cancel: CancellationToken) -> Self {
        Self {
            inner,
            cancellation: Box::pin(cancel.cancelled_owned()),
        }
    }
}

impl<R: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for CancellableReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.cancellation.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "playback session cancelled",
            )));
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum FileRole {
    #[default]
    Video,
    Subtitle,
    Auxiliary,
}

#[derive(Default)]
struct TorrentFileFocus {
    primary_video: Option<usize>,
    auxiliary_files: HashSet<usize>,
}

#[derive(Clone, Copy)]
struct TorrentLifecycle {
    last_accessed: Instant,
    prewarmed: bool,
    active: bool,
    estimated_cache_bytes: u64,
}

#[derive(Default)]
struct TorrentRuntimeState {
    known_links: HashMap<String, usize>,
    prioritized_files: HashMap<usize, TorrentFileFocus>,
    playback_windows: HashMap<(usize, usize), PlaybackWindow>,
    playback_sessions: HashMap<(usize, usize), PlaybackSession>,
    torrent_cancellations: HashMap<usize, CancellationToken>,
    lifecycle: HashMap<usize, TorrentLifecycle>,
    active_torrent: Option<usize>,
}

#[derive(Clone)]
struct EngineState {
    api: Api,
    output_dir: PathBuf,
    preload_size: Arc<Mutex<u64>>,
    stream_buffer_bytes: Arc<Mutex<usize>>,
    runtime: Arc<Mutex<TorrentRuntimeState>>,
    // Session ownership and its measurements must be changed atomically. Keeping
    // them together also prevents lock-order inversions between telemetry writes
    // and torrent teardown.
    telemetry: Arc<Mutex<TelemetryState>>,
    cache_limit_bytes: Arc<Mutex<Option<u64>>>,
    // A per-link lock serializes retries for one magnet while allowing
    // unrelated metadata lookups to progress independently.
    in_flight_adds: Arc<AsyncMutex<HashMap<String, Arc<AsyncMutex<()>>>>>,
    pending_adds: Arc<Mutex<HashSet<String>>>,
    access_token: Arc<String>,
}

struct TorrentServerHandle {
    generation: u64,
    stop: Option<oneshot::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

static TORRENT_SERVER: OnceLock<Mutex<Option<TorrentServerHandle>>> = OnceLock::new();
static TORRENT_GENERATION: AtomicU64 = AtomicU64::new(0);

fn torrent_server_handle() -> &'static Mutex<Option<TorrentServerHandle>> {
    TORRENT_SERVER.get_or_init(|| Mutex::new(None))
}

fn debug_log(message: impl AsRef<str>) {
    if std::env::var_os("FLUXA_TORRENT_DEBUG").is_some() {
        eprintln!("{}", message.as_ref());
    }
}

fn join_with_timeout(thread: thread::JoinHandle<()>, timeout: Duration) {
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    thread::spawn(move || {
        let _ = thread.join();
        let _ = done_tx.send(());
    });
    if done_rx.recv_timeout(timeout).is_err() {
        debug_log(format!(
            "[TorrServer] teardown did not finish within {timeout:?}, continuing without waiting"
        ));
    }
}

pub fn start_torrent_server(
    cache_dir: &str,
    preferred_port: i32,
    access_token: &str,
) -> Option<String> {
    let mut guard = torrent_server_handle().lock().ok()?;
    if let Some(mut handle) = guard.take() {
        let teardown_start = std::time::Instant::now();
        if let Some(stop) = handle.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = handle.thread.take() {
            join_with_timeout(thread, Duration::from_secs(5));
        }
        debug_log(format!(
            "[TorrServer] previous server torn down in {:?}",
            teardown_start.elapsed()
        ));
    }
    let bootstrap_start = std::time::Instant::now();

    let cache_dir = PathBuf::from(cache_dir);
    std::fs::create_dir_all(&cache_dir).ok()?;
    let dht_config = PersistentDhtConfig {
        dump_interval: Some(Duration::from_secs(60)),
        config_filename: Some(cache_dir.parent()?.join("torrent-dht.json")),
    };
    let bind_port = preferred_port.clamp(0, u16::MAX as i32) as u16;
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<u16, String>>();
    let thread_cache_dir = cache_dir.clone();
    let thread_dht_config = dht_config;
    let thread_access_token = access_token.trim().to_string();

    let thread = thread::spawn(move || {
        let worker_threads = torrent_worker_threads();
        let concurrent_init_limit = if cfg!(target_os = "android") { 2 } else { 4 };
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(worker_threads)
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = ready_tx.send(Err(error.to_string()));
                return;
            }
        };

        runtime.block_on(async move {
            let options = SessionOptions {
                disable_dht_persistence: false,
                dht_config: Some(thread_dht_config),
                fastresume: true,
                persistence: Some(SessionPersistenceConfig::Json {
                    folder: Some(thread_cache_dir.join("session")),
                }),
                defer_writes_up_to: Some(64),
                listen_port_range: Some(49152..65535),
                enable_upnp_port_forwarding: true,
                disable_upload: true,
                concurrent_init_limit: Some(concurrent_init_limit),
                trackers: [
                    "udp://tracker.opentrackr.org:1337/announce",
                    "udp://open.demonii.com:1337/announce",
                    "udp://tracker.openbittorrent.com:80/announce",
                    "udp://exodus.desync.com:6969/announce",
                    "udp://open.stealth.si:80/announce",
                    "udp://tracker.torrent.eu.org:451/announce",
                    "udp://tracker.tiny-vps.com:6969/announce",
                ]
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect(),
                ..Default::default()
            };

            let session = match tokio::time::timeout(
                Duration::from_secs(18),
                Session::new_with_opts(thread_cache_dir.clone(), options),
            )
            .await
            {
                Ok(Ok(session)) => session,
                Ok(Err(error)) => {
                    let _ = ready_tx.send(Err(format!("{error:#}")));
                    return;
                }
                Err(_) => {
                    let _ = ready_tx.send(Err("torrent session init timed out".to_string()));
                    return;
                }
            };

            let listener = match TcpListener::bind(("0.0.0.0", bind_port)).await {
                Ok(listener) => listener,
                Err(error) => {
                    let _ = ready_tx.send(Err(error.to_string()));
                    return;
                }
            };
            let port = match listener.local_addr() {
                Ok(addr) => addr.port(),
                Err(error) => {
                    let _ = ready_tx.send(Err(error.to_string()));
                    return;
                }
            };

            let state = EngineState {
                api: Api::new(session, None),
                output_dir: thread_cache_dir,
                preload_size: Arc::new(Mutex::new(10 * 1024 * 1024)),
                stream_buffer_bytes: Arc::new(Mutex::new(64 * 1024)),
                runtime: Arc::new(Mutex::new(TorrentRuntimeState::default())),
                telemetry: Arc::new(Mutex::new(TelemetryState::default())),
                cache_limit_bytes: Arc::new(Mutex::new(None)),
                in_flight_adds: Arc::new(AsyncMutex::new(HashMap::new())),
                pending_adds: Arc::new(Mutex::new(HashSet::new())),
                access_token: Arc::new(thread_access_token),
            };
            tokio::spawn(peer_stats_logger(state.clone()));
            tokio::spawn(prewarm_reaper(state.clone()));
            let app = Router::new()
                .route("/", get(root))
                .route("/health", get(health))
                .route("/settings", post(update_settings))
                .route("/torrents", post(torrents))
                .route("/telemetry", post(record_telemetry))
                .route("/stream/fname", get(stream_fname))
                .with_state(state);

            let server = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            );
            let _ = ready_tx.send(Ok(port));
            tokio::select! {
                _ = server => {}
                _ = stop_rx => {}
            }
        });
        runtime.shutdown_timeout(Duration::from_secs(2));
    });

    match ready_rx.recv_timeout(Duration::from_secs(20)) {
        Ok(Ok(port)) => {
            debug_log(format!(
                "[TorrServer] new server bootstrapped in {:?}",
                bootstrap_start.elapsed()
            ));
            let generation = TORRENT_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
            *guard = Some(TorrentServerHandle {
                generation,
                stop: Some(stop_tx),
                thread: Some(thread),
            });
            serde_json::to_string(&json!({
                "url": format!("http://127.0.0.1:{port}"),
                "port": port,
                "cacheDir": cache_dir.to_string_lossy(),
                "generation": generation
            }))
            .ok()
        }
        Ok(Err(error)) => {
            debug_log(format!("[TorrServer] startup failed: {error}"));
            let _ = stop_tx.send(());
            join_with_timeout(thread, Duration::from_secs(5));
            None
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            debug_log("[TorrServer] startup timed out");
            let _ = stop_tx.send(());
            None
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            join_with_timeout(thread, Duration::from_secs(5));
            None
        }
    }
}

/// Stops the running torrent server. If `expected_generation` is given, the
/// stop is a no-op when a newer server has already replaced the one the
/// caller meant to stop (e.g. a stale stop racing a fast replay's start) —
/// otherwise a stop issued for an old session could tear down the session
/// that superseded it.
pub fn stop_torrent_server(expected_generation: Option<u64>) -> bool {
    let mut guard = match torrent_server_handle().lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };
    if let Some(expected) = expected_generation
        && guard.as_ref().map(|h| h.generation) != Some(expected)
    {
        return false;
    }
    let Some(mut handle) = guard.take() else {
        return false;
    };
    if let Some(stop) = handle.stop.take() {
        let _ = stop.send(());
    }
    if let Some(thread) = handle.thread.take() {
        join_with_timeout(thread, Duration::from_secs(5));
    }
    true
}

// Independent of UI stat polling, so the timeline is complete even if the
// frontend isn't actively hitting /stream/fname?stat. One line per known
// torrent every 2s, gated behind FLUXA_TORRENT_DEBUG like everything else
// here — meant to be diffed against Stremio/TorrServer runs to see whether
// peer discovery plateaus (queued/live flat) or throughput per peer is the
// bottleneck (live steady, download_speed low).
async fn peer_stats_logger(state: EngineState) {
    if std::env::var_os("FLUXA_TORRENT_DEBUG").is_none() {
        return;
    }
    loop {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let ids: HashSet<usize> = state
            .runtime
            .lock()
            .map(|runtime| runtime.known_links.values().copied().collect())
            .unwrap_or_default();
        for id in ids {
            let Ok(stats) = state.api.api_stats_v1(TorrentIdOrHash::Id(id)) else {
                continue;
            };
            let peers = stats.live.as_ref().map(|live| &live.snapshot.peer_stats);
            let download_bps = stats
                .live
                .as_ref()
                .map(|live| live.download_speed.mbps * 1024.0 * 1024.0)
                .unwrap_or(0.0);
            debug_log(format!(
                "[TorrServer][peers] torrent={id} state={:?} queued={} connecting={} live={} seen={} dead={} steals={} down={download_bps:.0}B/s progress={}/{} uploaded={}",
                stats.state,
                peers.map(|p| p.queued).unwrap_or(0),
                peers.map(|p| p.connecting).unwrap_or(0),
                peers.map(|p| p.live).unwrap_or(0),
                peers.map(|p| p.seen).unwrap_or(0),
                peers.map(|p| p.dead).unwrap_or(0),
                peers.map(|p| p.steals).unwrap_or(0),
                stats.progress_bytes,
                stats.total_bytes,
                stats.uploaded_bytes,
            ));
        }
    }
}

async fn root() -> impl IntoResponse {
    "Fluxa Rust Torrent Engine"
}

async fn health() -> impl IntoResponse {
    "ok"
}

async fn update_settings(
    State(state): State<EngineState>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    Json(settings): Json<TorrSettings>,
) -> impl IntoResponse {
    if !request_authorized(&state, remote_addr, None) {
        return error_response(StatusCode::UNAUTHORIZED, "unauthorized");
    }
    let preload_mb = settings
        .preload_size
        .or_else(|| settings.device_budget.as_ref()?.torrent_preload_mb);
    if let Some(preload_mb) = preload_mb
        && let Ok(mut preload_size) = state.preload_size.lock()
    {
        *preload_size = preload_mb.saturating_mul(1024 * 1024);
    }
    let cache_limit_mb = settings
        .cache_limit_mb
        .or_else(|| settings.device_budget.as_ref()?.torrent_cache_mb);
    if let Some(limit_mb) = cache_limit_mb
        && let Ok(mut cache_limit) = state.cache_limit_bytes.lock()
    {
        *cache_limit = (limit_mb > 0).then(|| limit_mb.saturating_mul(1024 * 1024));
    }
    let buffer_bytes = settings
        .stream_buffer_bytes
        .or_else(|| settings.device_budget.as_ref()?.stream_reader_buffer_bytes);
    if let Some(buffer_bytes) = buffer_bytes
        && let Ok(mut buffer) = state.stream_buffer_bytes.lock()
    {
        *buffer = buffer_bytes.clamp(32 * 1024, 1024 * 1024) as usize;
    }
    (StatusCode::OK, Json(json!({}))).into_response()
}

fn stream_buffer_size(state: &EngineState) -> usize {
    state
        .stream_buffer_bytes
        .lock()
        .map(|buffer| *buffer)
        .unwrap_or(64 * 1024)
}

async fn torrents(
    State(state): State<EngineState>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    Json(request): Json<TorrRequest>,
) -> Response {
    if !request_authorized(&state, remote_addr, None) {
        return error_response(StatusCode::UNAUTHORIZED, "unauthorized");
    }
    let _ = request.save_to_db;
    let action = request.action.to_ascii_lowercase();
    match action.as_str() {
        "add" => {
            match ensure_torrent(
                &state,
                request.link.as_deref(),
                request.title.as_deref(),
                request.file_id,
                Duration::from_secs(90),
            )
            .await
            {
                Ok((id, details)) => {
                    touch_torrent_lifecycle(&state, id, !request.prewarm);
                    let focus = request.file_id.or_else(|| largest_file_id(&details));
                    if !request.prewarm {
                        if let Some(file_id) = focus {
                            prioritize_stream_file(&state, id, file_id, request.role).await;
                        }
                        // A torrent restored from session persistence, or paused by
                        // an earlier deactivation, stays paused until something
                        // starts it. Adding it again is a play intent, so resume it
                        // here instead of waiting for the stream request that the
                        // player only issues once buffering has already progressed.
                        // Starting one that is already initializing restarts its
                        // storage check and leaves the torrent in Error.
                        let paused = matches!(
                            state
                                .api
                                .api_stats_v1(TorrentIdOrHash::Id(id))
                                .ok()
                                .map(|stats| stats.state),
                            Some(TorrentStatsState::Paused)
                        );
                        if paused {
                            activate_torrent(&state, id).await;
                            let _ = state
                                .api
                                .api_torrent_action_start(TorrentIdOrHash::Id(id))
                                .await;
                        }
                    } else {
                        let delayed_state = state.clone();
                        tokio::spawn(async move {
                            // Give DHT/trackers a short discovery interval, then stop
                            // transfer work. A later play request resumes this torrent.
                            tokio::time::sleep(Duration::from_secs(10)).await;
                            let should_deactivate = delayed_state
                                .runtime
                                .lock()
                                .map(|runtime| should_deactivate_prewarm(&runtime.lifecycle, id))
                                .unwrap_or(false);
                            if should_deactivate {
                                deactivate_torrent(&delayed_state, id).await;
                            }
                        });
                    }
                    status_response(&state, id, Some(details), focus)
                        .await
                        .into_response()
                }
                Err(error) => error_response(StatusCode::BAD_REQUEST, error),
            }
        }
        "get" => {
            let id = match request
                .hash
                .as_deref()
                .and_then(|hash| hash.parse::<usize>().ok())
                .or_else(|| lookup_known_link(&state, request.link.as_deref()))
            {
                Some(id) => id,
                None => {
                    let resolving = match request.link.as_deref() {
                        Some(link) => add_is_pending(&state, link),
                        None => false,
                    };
                    return Json(empty_status_json(resolving)).into_response();
                }
            };
            if let Some(file_id) = request.file_id {
                prioritize_stream_file(&state, id, file_id, request.role).await;
            }
            touch_torrent_lifecycle(&state, id, false);
            status_response(&state, id, None, request.file_id)
                .await
                .into_response()
        }
        "rem" | "remove" | "delete" => {
            if let Some(id) = lookup_known_link(&state, request.link.as_deref()) {
                let _ = state
                    .api
                    .api_torrent_action_delete(TorrentIdOrHash::Id(id))
                    .await;
                if let Ok(mut runtime) = state.runtime.lock() {
                    runtime.known_links.retain(|_, known_id| *known_id != id);
                    runtime.prioritized_files.remove(&id);
                    runtime
                        .playback_windows
                        .retain(|(torrent_id, _), _| *torrent_id != id);
                    for session in runtime
                        .playback_sessions
                        .extract_if(|(torrent_id, _), _| *torrent_id == id)
                        .map(|(_, session)| session)
                    {
                        session.cancel.cancel();
                    }
                    runtime.lifecycle.remove(&id);
                    if runtime.active_torrent == Some(id) {
                        runtime.active_torrent = None;
                    }
                }
                cancel_torrent_root(&state, id);
                clear_playback_telemetry(&state, id);
            }
            Json(json!({})).into_response()
        }
        "deactivate" => {
            if let Some(id) = lookup_known_link(&state, request.link.as_deref()) {
                deactivate_torrent(&state, id).await;
            }
            Json(json!({})).into_response()
        }
        _ => error_response(StatusCode::BAD_REQUEST, "unsupported torrent action"),
    }
}

async fn record_telemetry(
    State(state): State<EngineState>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    Json(event): Json<TelemetryEvent>,
) -> Response {
    if !request_authorized(&state, remote_addr, None) {
        return error_response(StatusCode::UNAUTHORIZED, "unauthorized");
    }
    let Some(id) = lookup_known_link(&state, Some(&event.link)) else {
        return error_response(StatusCode::NOT_FOUND, "torrent not found");
    };
    if event.session_id.is_empty() || event.session_id.len() > 128 {
        return error_response(StatusCode::BAD_REQUEST, "invalid telemetry session");
    }
    if !telemetry_event_is_supported(&event.event) {
        return error_response(StatusCode::BAD_REQUEST, "unsupported telemetry event");
    }
    let mut telemetry = match state.telemetry.lock() {
        Ok(telemetry) => telemetry,
        Err(_) => {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "telemetry unavailable");
        }
    };
    if let Err(error) = apply_telemetry_event(&mut telemetry, id, &event) {
        let status = match error {
            "stale telemetry session" | "telemetry session mismatch" => StatusCode::CONFLICT,
            _ => StatusCode::BAD_REQUEST,
        };
        return error_response(status, error);
    }
    (StatusCode::OK, Json(json!({}))).into_response()
}

fn telemetry_event_is_supported(event: &str) -> bool {
    matches!(event, "firstFrame" | "stallStarted" | "stallEnded")
}

fn apply_telemetry_event(
    telemetry: &mut TelemetryState,
    torrent_id: usize,
    event: &TelemetryEvent,
) -> Result<(), &'static str> {
    // This guard deliberately lives before all session mutations as well as in
    // the HTTP handler, so callers cannot accidentally promote an invalid event.
    if !telemetry_event_is_supported(&event.event) {
        return Err("unsupported telemetry event");
    }
    match telemetry.active_sessions.get(&torrent_id) {
        Some(active) if event.session_generation < active.generation => {
            return Err("stale telemetry session");
        }
        Some(active)
            if event.session_generation == active.generation && event.session_id != active.id =>
        {
            return Err("telemetry session mismatch");
        }
        Some(active) if event.session_generation > active.generation => {
            telemetry.active_sessions.insert(
                torrent_id,
                ActiveTelemetrySession {
                    id: event.session_id.clone(),
                    generation: event.session_generation,
                },
            );
            telemetry
                .records
                .retain(|(stored_id, _), _| *stored_id != torrent_id);
        }
        None => {
            telemetry.active_sessions.insert(
                torrent_id,
                ActiveTelemetrySession {
                    id: event.session_id.clone(),
                    generation: event.session_generation,
                },
            );
        }
        _ => {}
    }
    let entry = telemetry
        .records
        .entry((torrent_id, event.session_id.clone()))
        .or_default();
    match event.event.as_str() {
        "firstFrame" => entry.first_frame_ms = event.elapsed_ms.or(entry.first_frame_ms),
        "stallStarted" => entry.stall_count = entry.stall_count.saturating_add(1),
        "stallEnded" => {
            entry.stall_duration_ms = entry
                .stall_duration_ms
                .saturating_add(event.elapsed_ms.unwrap_or_default())
        }
        _ => unreachable!("unsupported event was rejected before mutating telemetry"),
    }
    Ok(())
}

async fn stream_fname(
    State(state): State<EngineState>,
    Query(query): Query<StreamQuery>,
    headers: HeaderMap,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Response {
    if !request_authorized(&state, remote_addr, query.access_token.as_deref()) {
        return error_response(StatusCode::UNAUTHORIZED, "unauthorized");
    }
    let range_header = headers
        .get("Range")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("none");
    debug_log(format!(
        "[TorrServer] stream_fname link={} stat={} range={range_header}",
        &query.link[..query.link.len().min(60)],
        query.stat.is_some()
    ));

    // Stat requests return immediately. They must not start or block metadata
    // acquisition; otherwise UI polling can contend with the real stream GET.
    if query.stat.is_some() {
        if let Some(id) = lookup_known_link(&state, Some(&query.link)) {
            return status_response(&state, id, None, query.index)
                .await
                .into_response();
        }
        return Json(empty_status_json(add_is_pending(&state, &query.link))).into_response();
    }

    // Stream request: ensure_torrent does its own add+lookup. Calling it
    // once is enough — if metadata isn't ready yet, return 503 and let the
    // player retry the GET. No outer retry loop (the old 60s loop just hid
    // the latency from the user without saving any time).
    let (id, details) = match ensure_torrent(
        &state,
        Some(&query.link),
        query.title.as_deref(),
        query.index,
        Duration::from_secs(90),
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            debug_log(format!("[TorrServer] ensure_torrent failed: {error}"));
            return error_response(StatusCode::SERVICE_UNAVAILABLE, error);
        }
    };
    let file_id = query
        .index
        .unwrap_or_else(|| largest_file_id(&details).unwrap_or(0));
    debug_log(format!(
        "[TorrServer] streaming torrent={id} file={file_id} files={}",
        details.files.as_ref().map(|f| f.len()).unwrap_or(0)
    ));
    prioritize_stream_file(&state, id, file_id, query.role).await;
    activate_torrent(&state, id).await;
    let _ = state
        .api
        .api_torrent_action_start(TorrentIdOrHash::Id(id))
        .await;

    // Wait for rqbit to leave Initializing state before attempting to stream.
    // api_stream fails immediately with "invalid state: initializing" until this
    // transition happens, so polling was wasting 50ms slots per attempt.
    // wait_until_initialized uses a notify channel and fires as soon as it's ready.
    if let Ok(handle) = state.api.mgr_handle(TorrentIdOrHash::Id(id))
        && let Err(e) =
            tokio::time::timeout(Duration::from_secs(60), handle.wait_until_initialized()).await
    {
        debug_log(format!(
            "[TorrServer] wait_until_initialized timed out torrent={id}: {e}"
        ));
        return error_response(StatusCode::SERVICE_UNAVAILABLE, "torrent init timed out");
    }

    match state.api.api_stream(TorrentIdOrHash::Id(id), file_id) {
        Ok(mut stream) => {
            let mut status = StatusCode::OK;
            let mut output_headers = HeaderMap::new();
            output_headers.insert("Accept-Ranges", HeaderValue::from_static("bytes"));
            if let Ok(mime) = state
                .api
                .torrent_file_mime_type(TorrentIdOrHash::Id(id), file_id)
                && let Ok(value) = HeaderValue::from_str(mime)
            {
                output_headers.insert("Content-Type", value);
            }
            let total_len = stream.len();
            match parse_range(headers.get("Range"), total_len) {
                Ok(Some((start, end))) => {
                    let length = end.saturating_sub(start).saturating_add(1);
                    let probe = is_probe_range(&state, id, file_id, start, length);
                    let cancellation = if probe {
                        torrent_cancellation_token(&state, id).child_token()
                    } else {
                        let cancellation = playback_session_for(&state, id, file_id, start);
                        remember_playback_window(
                            &state,
                            id,
                            file_id,
                            start,
                            total_len,
                            query.duration_ms,
                        );
                        set_streaming_window(&state, id, file_id, start);
                        cancellation
                    };
                    if let Err(error) = stream.seek(SeekFrom::Start(start)).await {
                        debug_log(format!(
                            "[TorrServer] seek failed torrent={id} file={file_id} start={start} len={total_len}: {error}"
                        ));
                        return error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to seek stream",
                        );
                    }
                    status = StatusCode::PARTIAL_CONTENT;
                    insert_header(&mut output_headers, "Content-Length", length.to_string());
                    insert_header(
                        &mut output_headers,
                        "Content-Range",
                        format!("bytes {start}-{end}/{total_len}"),
                    );
                    let body = Body::from_stream(ReaderStream::with_capacity(
                        CancellableReader::new(stream.take(length), cancellation),
                        stream_buffer_size(&state),
                    ));
                    (status, output_headers, body).into_response()
                }
                Ok(None) => {
                    let cancellation = playback_session_for(&state, id, file_id, 0);
                    remember_playback_window(&state, id, file_id, 0, total_len, query.duration_ms);
                    set_streaming_window(&state, id, file_id, 0);
                    insert_header(&mut output_headers, "Content-Length", total_len.to_string());
                    let body = Body::from_stream(ReaderStream::with_capacity(
                        CancellableReader::new(stream, cancellation),
                        stream_buffer_size(&state),
                    ));
                    (status, output_headers, body).into_response()
                }
                Err(()) => range_not_satisfiable_response(total_len),
            }
        }
        Err(e) => {
            debug_log(format!(
                "[TorrServer] api_stream failed torrent={id} file={file_id}: {e:#}"
            ));
            error_response(StatusCode::NOT_FOUND, format!("{e:#}"))
        }
    }
}

fn magnet_info_hash(link: &str) -> Option<String> {
    let marker = "xt=urn:btih:";
    let start = link.find(marker)? + marker.len();
    let hash: String = link[start..]
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect();
    (hash.len() == 40).then(|| hash.to_ascii_lowercase())
}

fn find_torrent_by_info_hash(state: &EngineState, hash: &str) -> Option<usize> {
    state
        .api
        .api_torrent_list_ext(ApiTorrentListOpts { with_stats: false })
        .torrents
        .into_iter()
        .find(|torrent| torrent.info_hash.eq_ignore_ascii_case(hash))
        .and_then(|torrent| torrent.id)
}

async fn ensure_torrent(
    state: &EngineState,
    link: Option<&str>,
    title: Option<&str>,
    only_file: Option<usize>,
    metadata_timeout: Duration,
) -> Result<(usize, TorrentDetailsResponse), String> {
    let link = link
        .map(str::trim)
        .filter(|link| !link.is_empty())
        .ok_or_else(|| "missing torrent link".to_string())?;
    if let Some(id) = lookup_known_link(state, Some(link)) {
        let details = state
            .api
            .api_torrent_details(TorrentIdOrHash::Id(id))
            .map_err(|error| format!("{error:#}"))?;
        return Ok((id, details));
    }

    // The link map only knows what this process added. A torrent restored from
    // session persistence, or the same one behind a magnet with a different
    // tracker list, would otherwise be resolved from peers all over again.
    if let Some(id) = magnet_info_hash(link).and_then(|hash| find_torrent_by_info_hash(state, &hash))
    {
        remember_link(state, link, id);
        let details = state
            .api
            .api_torrent_details(TorrentIdOrHash::Id(id))
            .map_err(|error| format!("{error:#}"))?;
        return Ok((id, details));
    }

    // Metadata acquisition only serializes per link. Different magnets can
    // resolve in parallel while a duplicate request joins this operation.
    let link_lock = {
        let mut in_flight = state.in_flight_adds.lock().await;
        in_flight
            .entry(link.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    };
    let lock_timeout = metadata_timeout
        .checked_add(Duration::from_secs(5))
        .unwrap_or(metadata_timeout);
    let _add_guard = match tokio::time::timeout(lock_timeout, link_lock.lock()).await {
        Ok(guard) => guard,
        Err(_) => return Err("torrent add already in progress".to_string()),
    };
    if let Some(id) = lookup_known_link(state, Some(link)) {
        let details = state
            .api
            .api_torrent_details(TorrentIdOrHash::Id(id))
            .map_err(|error| format!("{error:#}"))?;
        return Ok((id, details));
    }

    let mut options = AddTorrentOptions {
        overwrite: true,
        output_folder: Some(state.output_dir.to_string_lossy().into_owned()),
        peer_opts: Some(PeerConnectionOptions {
            connect_timeout: Some(Duration::from_millis(2500)),
            read_write_timeout: Some(Duration::from_secs(20)),
            ..Default::default()
        }),
        ..Default::default()
    };
    // Limit rqbit initialization to just the target file so the Initializing
    // hash-check covers one file instead of every file in the torrent.
    if let Some(file_id) = only_file {
        options.only_files = Some(vec![file_id]);
    }
    set_add_pending(state, link, true);
    let add_started = std::time::Instant::now();
    let response = tokio::time::timeout(
        metadata_timeout,
        state
            .api
            .api_add_torrent(AddTorrent::Url(link.to_string().into()), Some(options)),
    )
    .await;
    let response = match response {
        Ok(Ok(response)) => response,
        Ok(Err(error)) => {
            set_add_pending(state, link, false);
            return Err(format!("{error:#}"));
        }
        Err(_) => {
            set_add_pending(state, link, false);
            debug_log(format!(
                "[TorrServer][timing] metadata timed out after {:?} link={}",
                add_started.elapsed(),
                &link[..link.len().min(80)]
            ));
            return Err("torrent metadata timed out".to_string());
        }
    };
    let Some(id) = response.id else {
        set_add_pending(state, link, false);
        return Err("torrent metadata is not ready".to_string());
    };
    debug_log(format!(
        "[TorrServer][timing] metadata ready in {:?} torrent={id}",
        add_started.elapsed()
    ));
    remember_link(state, link, id);
    if let Some(title) = title {
        remember_link(state, title, id);
    }
    set_add_pending(state, link, false);
    Ok((id, response.details))
}

fn empty_status_json(resolving: bool) -> Value {
    json!({
        "hash": "",
        "title": "",
        "download_speed": 0.0,
        "active_peers": 0,
        "total_peers": 0,
        "progress": 0.0,
        "stat": 0,
        "stat_string": if resolving { "resolving" } else { "initializing" },
        "resolving": resolving,
        "preload": 0,
        "loaded_size": 0,
        "streamed_size": 0,
        "preload_size": 0,
        "buffered_ahead_bytes": 0,
        "playback_offset": 0,
        "file_stats": []
    })
}

fn set_add_pending(state: &EngineState, link: &str, pending: bool) {
    if let Ok(mut pending_adds) = state.pending_adds.lock() {
        if pending {
            pending_adds.insert(link.to_string());
        } else {
            pending_adds.remove(link);
        }
    }
}

fn add_is_pending(state: &EngineState, link: &str) -> bool {
    state
        .pending_adds
        .lock()
        .map(|pending| pending.contains(link.trim()))
        .unwrap_or(false)
}

async fn status_response(
    state: &EngineState,
    id: usize,
    details: Option<TorrentDetailsResponse>,
    focus_file: Option<usize>,
) -> Json<Value> {
    let details = details.or_else(|| state.api.api_torrent_details(TorrentIdOrHash::Id(id)).ok());
    let stats = state.api.api_stats_v1(TorrentIdOrHash::Id(id)).ok();
    let file_stats = details
        .as_ref()
        .and_then(|details| details.files.as_ref())
        .map(|files| {
            files
                .iter()
                .enumerate()
                .map(|(idx, file)| json!({ "id": idx, "path": file.name, "length": file.length }))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let progress = stats
        .as_ref()
        .map(|stats| {
            if stats.total_bytes == 0 {
                0.0
            } else {
                (stats.progress_bytes as f64 / stats.total_bytes as f64) * 100.0
            }
        })
        .unwrap_or(0.0);
    if let (Some(stats), Ok(mut runtime)) = (stats.as_ref(), state.runtime.lock())
        && let Some(entry) = runtime.lifecycle.get_mut(&id)
    {
        entry.estimated_cache_bytes = stats.progress_bytes;
    }
    let download_speed = stats
        .as_ref()
        .and_then(|stats| stats.live.as_ref())
        .map(|live| live.download_speed.mbps * 1024.0 * 1024.0)
        .unwrap_or(0.0);
    let active_peers = stats
        .as_ref()
        .and_then(|stats| stats.live.as_ref())
        .map(|live| live.snapshot.peer_stats.live)
        .unwrap_or(0);
    let total_peers = stats
        .as_ref()
        .and_then(|stats| stats.live.as_ref())
        .map(|live| live.snapshot.peer_stats.seen)
        .unwrap_or(0);
    let peer_quality = state.api.api_peer_quality(TorrentIdOrHash::Id(id)).ok();
    let preload_size = state.preload_size.lock().map(|value| *value).unwrap_or(0);
    let mut window = focus_file.and_then(|file_id| playback_window_for(state, id, file_id));
    let playback_offset = window.map(|window| window.playback_offset).unwrap_or(0);
    // Fork API: verified bytes contiguous from the active HTTP Range start.
    // This is the playback buffer signal; aggregate file_progress remains
    // intentionally excluded because it may describe unrelated pieces.
    let loaded_size = focus_file
        .and_then(|file_id| {
            state
                .api
                .api_contiguous_bytes_from_with_limit(
                    TorrentIdOrHash::Id(id),
                    file_id,
                    playback_offset,
                    window
                        .map(|window| window.warm_ahead_bytes)
                        .unwrap_or(preload_size),
                )
                .ok()
                .map(|response| response.contiguous_bytes)
        })
        .unwrap_or(0);
    if let Some(mut current) = window {
        current.contiguous_ready_bytes = loaded_size;
        current.smoothed_download_bps = if current.smoothed_download_bps == 0.0 {
            download_speed
        } else {
            current.smoothed_download_bps * 0.8 + download_speed * 0.2
        };
        current.updated_at = Instant::now();
        current.was_ready |= loaded_size >= current.urgent_ahead_bytes;
        store_playback_window(state, current);
        window = Some(current);
    }
    let target_buffer_bytes = window
        .map(|window| window.urgent_ahead_bytes)
        .unwrap_or(preload_size);
    let buffered_ahead_seconds = window
        .map(|window| loaded_size as f64 * 8.0 / window.estimated_bitrate_bps.max(1) as f64)
        .unwrap_or(0.0);
    let target_buffer_seconds = window
        .map(|window| {
            window.urgent_ahead_bytes as f64 * 8.0 / window.estimated_bitrate_bps.max(1) as f64
        })
        .unwrap_or(0.0);
    let speed_to_bitrate_ratio = window
        .map(|window| {
            window.smoothed_download_bps * 8.0 / window.estimated_bitrate_bps.max(1) as f64
        })
        .unwrap_or(0.0);
    let playback_telemetry = state
        .telemetry
        .lock()
        .ok()
        .and_then(|telemetry| {
            telemetry
                .active_sessions
                .get(&id)
                .and_then(|session| telemetry.records.get(&(id, session.id.clone())).copied())
        })
        .unwrap_or_default();
    let scheduler = window.map(|window| {
        json!({
            "torrentId": window.torrent_id,
            "fileId": window.file_id,
            "urgentAheadBytes": window.urgent_ahead_bytes,
            "warmAheadBytes": window.warm_ahead_bytes,
            "contiguousReadyBytes": window.contiguous_ready_bytes,
            "seekGeneration": window.seek_generation,
            "seekElapsedMs": window.seek_started_at.map(|started| started.elapsed().as_millis() as u64),
            "windowAgeMs": window.updated_at.elapsed().as_millis() as u64,
            "wasReady": window.was_ready,
        })
    });
    let stat = match stats.as_ref().map(|stats| stats.state) {
        Some(TorrentStatsState::Live)
            if loaded_size >= target_buffer_bytes && target_buffer_bytes > 0 =>
        {
            3
        }
        Some(TorrentStatsState::Live) => 2,
        Some(TorrentStatsState::Initializing) => 0,
        Some(TorrentStatsState::Paused) => 1,
        Some(TorrentStatsState::Error) => -1,
        None => 0,
    };
    Json(json!({
        "hash": details.as_ref().map(|details| details.info_hash.clone()).unwrap_or_default(),
        "title": details.as_ref().and_then(|details| details.name.clone()).unwrap_or_default(),
        "download_speed": download_speed,
        "active_peers": active_peers,
        "total_peers": total_peers,
        "peer_connection_attempts": peer_quality.as_ref().map(|value| value.connection_attempts).unwrap_or(0),
        "peer_connections": peer_quality.as_ref().map(|value| value.connections).unwrap_or(0),
        "peer_errors": peer_quality.as_ref().map(|value| value.errors).unwrap_or(0),
        "peer_fetched_bytes": peer_quality.as_ref().map(|value| value.fetched_bytes).unwrap_or(0),
        "peer_fetched_chunks": peer_quality.as_ref().map(|value| value.fetched_chunks).unwrap_or(0),
        "progress": progress,
        "stat": stat,
        "stat_string": stats.as_ref().map(|stats| stats.state.to_string()).unwrap_or_else(|| "initializing".to_string()),
        "error": stats.as_ref().and_then(|stats| stats.error.as_deref()),
        "preload": if target_buffer_bytes == 0 { 0 } else { ((loaded_size.min(target_buffer_bytes) as f64 / target_buffer_bytes as f64) * 100.0).round() as i64 },
        "loaded_size": loaded_size,
        "streamed_size": 0,
        "preload_size": preload_size,
        "buffered_ahead_bytes": loaded_size,
        "buffered_ahead_seconds": buffered_ahead_seconds,
        "target_buffer_seconds": target_buffer_seconds,
        "estimated_bitrate": window.map(|window| window.estimated_bitrate_bps).unwrap_or(0),
        "speed_to_bitrate_ratio": speed_to_bitrate_ratio,
        "seek_generation": window.map(|window| window.seek_generation).unwrap_or(0),
        "phase": playback_phase(stats.as_ref(), loaded_size, target_buffer_bytes, window),
        "playback_offset": playback_offset,
        "requested_end": window.map(|window| window.requested_end).unwrap_or(0),
        "telemetry": {
            "downloadSpeedBps": download_speed,
            "speedToBitrateRatio": speed_to_bitrate_ratio,
            "bufferedAheadBytes": loaded_size,
            "bufferedAheadSeconds": buffered_ahead_seconds,
            "phase": playback_phase(stats.as_ref(), loaded_size, target_buffer_bytes, window),
            "scheduler": scheduler,
            "firstFrameMs": playback_telemetry.first_frame_ms,
            "stallCount": playback_telemetry.stall_count,
            "stallDurationMs": playback_telemetry.stall_duration_ms
        },
        "file_stats": file_stats
    }))
}

fn remember_playback_window(
    state: &EngineState,
    torrent_id: usize,
    file_id: usize,
    offset: u64,
    file_len: u64,
    duration_ms: Option<u64>,
) {
    let (bitrate, urgent, warm) = playback_buffer_targets(file_len, duration_ms);
    if let Ok(mut runtime) = state.runtime.lock() {
        let windows = &mut runtime.playback_windows;
        let key = (torrent_id, file_id);
        let previous = windows.get(&key).copied();
        let seek = previous
            .map(|window| offset.abs_diff(window.playback_offset) > window.warm_ahead_bytes / 4)
            .unwrap_or(false);
        windows.insert(
            key,
            PlaybackWindow {
                torrent_id,
                file_id,
                playback_offset: offset,
                requested_end: offset.saturating_add(warm).min(file_len),
                contiguous_ready_bytes: previous
                    .map(|window| window.contiguous_ready_bytes)
                    .unwrap_or(0),
                estimated_bitrate_bps: bitrate,
                urgent_ahead_bytes: urgent,
                warm_ahead_bytes: warm,
                smoothed_download_bps: previous
                    .map(|window| window.smoothed_download_bps)
                    .unwrap_or(0.0),
                seek_generation: previous
                    .map(|window| window.seek_generation + seek as u64)
                    .unwrap_or(0),
                was_ready: previous
                    .map(|window| window.was_ready && !seek)
                    .unwrap_or(false),
                seek_started_at: seek.then(Instant::now),
                updated_at: Instant::now(),
            },
        );
    }
}

fn playback_buffer_targets(file_len: u64, duration_ms: Option<u64>) -> (u64, u64, u64) {
    let bitrate = duration_ms
        .filter(|duration| *duration > 0)
        .map(|duration| file_len.saturating_mul(8).saturating_mul(1000) / duration)
        .unwrap_or(8 * 1024 * 1024);
    let startup_seconds = if bitrate >= 30 * 1024 * 1024 { 12 } else { 6 };
    let urgent = bitrate.saturating_mul(startup_seconds) / 8;
    let warm = bitrate.saturating_mul(45) / 8;
    (bitrate, urgent, warm)
}

fn touch_torrent_lifecycle(state: &EngineState, torrent_id: usize, active: bool) {
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

fn should_deactivate_prewarm(
    lifecycle: &HashMap<usize, TorrentLifecycle>,
    torrent_id: usize,
) -> bool {
    lifecycle
        .get(&torrent_id)
        .is_some_and(|entry| entry.prewarmed && !entry.active)
}

async fn activate_torrent(state: &EngineState, torrent_id: usize) {
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

async fn deactivate_torrent(state: &EngineState, torrent_id: usize) {
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

fn clear_playback_telemetry(state: &EngineState, torrent_id: usize) {
    if let Ok(mut telemetry) = state.telemetry.lock() {
        clear_telemetry_for_torrent(&mut telemetry, torrent_id);
    }
}

fn clear_telemetry_for_torrent(telemetry: &mut TelemetryState, torrent_id: usize) {
    telemetry.records.retain(|(id, _), _| *id != torrent_id);
    telemetry.active_sessions.remove(&torrent_id);
}

fn torrent_worker_threads() -> usize {
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
async fn prewarm_reaper(state: EngineState) {
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

async fn enforce_cache_limit(state: &EngineState) {
    let Some(limit) = state.cache_limit_bytes.lock().ok().and_then(|limit| *limit) else {
        return;
    };
    let snapshots = state
        .api
        .api_torrent_list_ext(ApiTorrentListOpts { with_stats: true });
    let mut entries = state
        .runtime
        .lock()
        .map(|runtime| {
            snapshots
                .torrents
                .iter()
                .filter_map(|torrent| {
                    let id = torrent.id?;
                    let lifecycle = runtime.lifecycle.get(&id)?;
                    Some((
                        id,
                        lifecycle.active,
                        lifecycle.last_accessed,
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

fn playback_window_for(
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

fn playback_session_for(
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

fn torrent_cancellation_token(state: &EngineState, torrent_id: usize) -> CancellationToken {
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

fn cancel_torrent_root(state: &EngineState, torrent_id: usize) {
    if let Ok(mut runtime) = state.runtime.lock()
        && let Some(token) = runtime.torrent_cancellations.remove(&torrent_id)
    {
        token.cancel();
    }
}

/// MPV/FFmpeg may issue a tiny distant cue/index read without seeking the
/// primary playback stream. Keep the live scheduler window in that case.
fn is_probe_range(
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

fn is_probe_for_window(
    window: PlaybackWindow,
    offset: u64,
    length: u64,
    max_probe_bytes: u64,
) -> bool {
    length <= max_probe_bytes
        && offset.abs_diff(window.playback_offset)
            > (window.warm_ahead_bytes / 4).max(max_probe_bytes)
}

fn store_playback_window(state: &EngineState, window: PlaybackWindow) {
    if let Ok(mut runtime) = state.runtime.lock() {
        runtime
            .playback_windows
            .insert((window.torrent_id, window.file_id), window);
    }
}

fn playback_phase(
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

fn set_streaming_window(state: &EngineState, torrent_id: usize, file_id: usize, offset: u64) {
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

mod http;

use http::*;
#[cfg(test)]
mod tests {
    use super::http::update_file_focus;
    use super::{
        ActiveTelemetrySession, CancellableReader, FileRole, PlaybackTelemetry, PlaybackWindow,
        TelemetryEvent, TelemetryState, TorrentFileFocus, TorrentLifecycle, apply_telemetry_event,
        clear_telemetry_for_torrent, is_probe_for_window, magnet_info_hash, parse_range,
        playback_buffer_targets,
        should_deactivate_prewarm,
    };
    use axum::http::HeaderValue;
    use std::collections::HashMap;
    use std::time::Instant;
    use tokio::io::AsyncReadExt;
    use tokio::time::{Duration, timeout};
    use tokio_util::sync::CancellationToken;

    fn range(value: &str, length: u64) -> Result<Option<(u64, u64)>, ()> {
        parse_range(Some(&HeaderValue::from_str(value).unwrap()), length)
    }

    fn telemetry_event(session_id: &str, generation: u64, event: &str) -> TelemetryEvent {
        TelemetryEvent {
            link: String::new(),
            session_id: session_id.to_string(),
            session_generation: generation,
            event: event.to_string(),
            elapsed_ms: Some(42),
        }
    }

    #[test]
    fn telemetry_rejects_stale_generation() {
        let mut telemetry = TelemetryState::default();
        apply_telemetry_event(
            &mut telemetry,
            7,
            &telemetry_event("current", 2, "firstFrame"),
        )
        .unwrap();

        assert_eq!(
            apply_telemetry_event(&mut telemetry, 7, &telemetry_event("old", 1, "firstFrame")),
            Err("stale telemetry session")
        );
        assert_eq!(telemetry.active_sessions.get(&7).unwrap().id, "current");
    }

    #[test]
    fn telemetry_rejects_different_session_at_same_generation() {
        let mut telemetry = TelemetryState::default();
        apply_telemetry_event(
            &mut telemetry,
            7,
            &telemetry_event("first", 2, "firstFrame"),
        )
        .unwrap();

        assert_eq!(
            apply_telemetry_event(
                &mut telemetry,
                7,
                &telemetry_event("second", 2, "firstFrame")
            ),
            Err("telemetry session mismatch")
        );
        assert_eq!(telemetry.active_sessions.get(&7).unwrap().id, "first");
    }

    #[test]
    fn telemetry_higher_generation_replaces_and_clears_old_records() {
        let mut telemetry = TelemetryState::default();
        apply_telemetry_event(
            &mut telemetry,
            7,
            &telemetry_event("first", 1, "firstFrame"),
        )
        .unwrap();
        apply_telemetry_event(
            &mut telemetry,
            7,
            &telemetry_event("second", 2, "stallStarted"),
        )
        .unwrap();

        assert_eq!(telemetry.active_sessions.get(&7).unwrap().id, "second");
        assert_eq!(telemetry.records.len(), 1);
        assert!(telemetry.records.contains_key(&(7, "second".to_string())));
    }

    #[test]
    fn telemetry_invalid_event_does_not_change_active_session() {
        let mut telemetry = TelemetryState::default();
        telemetry.active_sessions.insert(
            7,
            ActiveTelemetrySession {
                id: "current".to_string(),
                generation: 2,
            },
        );

        assert_eq!(
            apply_telemetry_event(&mut telemetry, 7, &telemetry_event("invalid", 99, "nope")),
            Err("unsupported telemetry event")
        );
        let active = telemetry.active_sessions.get(&7).unwrap();
        assert_eq!(active.id, "current");
        assert_eq!(active.generation, 2);
    }

    #[test]
    fn telemetry_teardown_removes_session_and_records() {
        let mut telemetry = TelemetryState::default();
        telemetry.active_sessions.insert(
            7,
            ActiveTelemetrySession {
                id: "session".to_string(),
                generation: 1,
            },
        );
        telemetry
            .records
            .insert((7, "session".to_string()), PlaybackTelemetry::default());
        telemetry
            .records
            .insert((8, "other".to_string()), PlaybackTelemetry::default());

        clear_telemetry_for_torrent(&mut telemetry, 7);

        assert!(!telemetry.active_sessions.contains_key(&7));
        assert!(!telemetry.records.contains_key(&(7, "session".to_string())));
        assert!(telemetry.records.contains_key(&(8, "other".to_string())));
    }

    #[test]
    fn parses_open_ended_range() {
        assert_eq!(range("bytes=100-", 1000), Ok(Some((100, 999))));
    }

    #[test]
    fn parses_bounded_range() {
        assert_eq!(range("bytes=100-199", 1000), Ok(Some((100, 199))));
    }

    #[test]
    fn clamps_bounded_range_to_file_end() {
        assert_eq!(range("bytes=900-2000", 1000), Ok(Some((900, 999))));
    }

    #[test]
    fn parses_suffix_range() {
        assert_eq!(range("bytes=-200", 1000), Ok(Some((800, 999))));
    }

    #[test]
    fn rejects_unsatisfiable_and_malformed_ranges() {
        assert_eq!(range("bytes=1000-", 1000), Err(()));
        assert_eq!(range("bytes=200-100", 1000), Err(()));
        assert_eq!(range("items=0-1", 1000), Err(()));
        assert_eq!(range("bytes=0-1,2-3", 1000), Err(()));
        assert_eq!(range("bytes=-0", 1000), Err(()));
    }

    #[test]
    fn magnet_hash_survives_a_different_tracker_list() {
        let with_trackers = "magnet:?xt=urn:btih:4B91C9289FD752522348D5A67A4E8157DD1B5DF2&dn=x&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337";
        let bare = "magnet:?xt=urn:btih:4b91c9289fd752522348d5a67a4e8157dd1b5df2";

        assert_eq!(magnet_info_hash(with_trackers), magnet_info_hash(bare));
        assert_eq!(
            magnet_info_hash(bare).as_deref(),
            Some("4b91c9289fd752522348d5a67a4e8157dd1b5df2")
        );
        assert!(magnet_info_hash("magnet:?xt=urn:btih:tooshort").is_none());
        assert!(magnet_info_hash("https://example.com/a.torrent").is_none());
    }

    #[test]
    fn replaying_the_same_file_reselects_it_after_deactivation() {
        use crate::torrent_engine::http::update_file_focus;

        let mut focus = TorrentFileFocus::default();
        assert!(update_file_focus(&mut focus, 3, FileRole::Video).is_some());
        assert!(
            update_file_focus(&mut focus, 3, FileRole::Video).is_none(),
            "a repeat request is skipped while the focus is cached"
        );

        let mut after_deactivation = TorrentFileFocus::default();
        assert!(
            update_file_focus(&mut after_deactivation, 3, FileRole::Video).is_some(),
            "dropping the focus on deactivation is what lets the next play re-apply it"
        );
    }

    #[test]
    fn playback_window_uses_duration_based_buffer_targets() {
        let (bitrate, urgent, warm) = playback_buffer_targets(1_000_000_000, Some(100_000));

        assert_eq!(bitrate, 80_000_000);
        assert_eq!(urgent, 120_000_000);
        assert_eq!(warm, 450_000_000);
    }

    #[test]
    fn distant_small_range_is_a_probe_not_a_seek() {
        let window = PlaybackWindow {
            torrent_id: 1,
            file_id: 0,
            playback_offset: 32 * 1024 * 1024,
            requested_end: 0,
            contiguous_ready_bytes: 0,
            estimated_bitrate_bps: 0,
            urgent_ahead_bytes: 0,
            warm_ahead_bytes: 64 * 1024 * 1024,
            smoothed_download_bps: 0.0,
            seek_generation: 0,
            was_ready: false,
            seek_started_at: None,
            updated_at: std::time::Instant::now(),
        };
        assert!(is_probe_for_window(
            window,
            500 * 1024 * 1024,
            1024,
            2 * 1024 * 1024
        ));
        assert!(!is_probe_for_window(
            window,
            500 * 1024 * 1024,
            8 * 1024 * 1024,
            2 * 1024 * 1024
        ));
    }

    #[test]
    fn delayed_prewarm_never_deactivates_active_playback() {
        let mut lifecycle = HashMap::new();
        lifecycle.insert(
            7,
            TorrentLifecycle {
                last_accessed: Instant::now(),
                prewarmed: true,
                active: false,
                estimated_cache_bytes: 0,
            },
        );
        assert!(should_deactivate_prewarm(&lifecycle, 7));

        lifecycle.get_mut(&7).unwrap().active = true;
        assert!(!should_deactivate_prewarm(&lifecycle, 7));
    }

    #[tokio::test]
    async fn cancellation_wakes_a_pending_reader() {
        let (reader, _writer) = tokio::io::duplex(1);
        let cancellation = CancellationToken::new();
        let mut reader = CancellableReader::new(reader, cancellation.clone());
        let read = tokio::spawn(async move {
            let mut buffer = [0; 1];
            reader.read(&mut buffer).await
        });

        tokio::task::yield_now().await;
        cancellation.cancel();
        let error = timeout(Duration::from_secs(1), read)
            .await
            .expect("cancellation should wake the pending reader")
            .expect("reader task should not panic")
            .expect_err("cancelled reader should fail");
        assert_eq!(error.kind(), std::io::ErrorKind::Interrupted);
    }

    #[test]
    fn no_range_header_means_full_response() {
        assert_eq!(parse_range(None, 1000), Ok(None));
    }

    #[test]
    fn subtitle_keeps_current_primary_video_selected() {
        let mut focus = TorrentFileFocus::default();
        assert_eq!(
            update_file_focus(&mut focus, 1, FileRole::Video),
            Some([1].into_iter().collect())
        );
        assert_eq!(
            update_file_focus(&mut focus, 2, FileRole::Subtitle),
            Some([1, 2].into_iter().collect())
        );
        assert_eq!(focus.primary_video, Some(1));
    }

    #[test]
    fn new_video_replaces_old_video_and_clears_old_auxiliaries() {
        let mut focus = TorrentFileFocus::default();
        update_file_focus(&mut focus, 1, FileRole::Video);
        update_file_focus(&mut focus, 2, FileRole::Subtitle);
        assert_eq!(
            update_file_focus(&mut focus, 3, FileRole::Video),
            Some([3].into_iter().collect())
        );
        assert_eq!(focus.primary_video, Some(3));
        assert!(focus.auxiliary_files.is_empty());
    }
}
