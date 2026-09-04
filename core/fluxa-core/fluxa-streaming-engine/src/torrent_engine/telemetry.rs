use super::{
    ActiveTelemetrySession, EngineState, TelemetryEvent, TelemetryState, debug_log, error_response,
    lookup_known_link, request_authorized,
};
use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use librqbit::api::TorrentIdOrHash;
use serde_json::json;
use std::collections::HashSet;
use std::net::SocketAddr;

pub(super) async fn peer_stats_logger(state: EngineState) {
    if std::env::var_os("FLUXA_TORRENT_DEBUG").is_none() {
        return;
    }
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
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

pub(super) async fn record_telemetry(
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

pub(super) fn apply_telemetry_event(
    telemetry: &mut TelemetryState,
    torrent_id: usize,
    event: &TelemetryEvent,
) -> Result<(), &'static str> {
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
