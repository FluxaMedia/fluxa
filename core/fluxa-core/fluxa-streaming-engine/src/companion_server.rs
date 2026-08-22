/// fluxa-web's local companion process: torrent streaming, ffmpeg
/// remux/transcode for browser playback, and OAuth token exchange (holds the
/// client_secret so it never ships to the browser). Mirrors the equivalent
/// Tauri commands in apps/desktop/src-tauri/src/lib.rs and oauth.rs, just
/// exposed over HTTP instead of IPC. Used by both the standalone
/// `companion_server` binary and the `fluxa-companion` tray app.
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use fluxa_core::FluxaCore;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

#[derive(Default)]
struct AppState {
    torrent_base_url: Mutex<Option<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartTorrentBody {
    stream_json: String,
    title: Option<String>,
    preferences: Option<Value>,
}

async fn health() -> &'static str {
    "ok"
}

fn companion_token() -> Option<String> {
    std::env::var("FLUXA_COMPANION_TOKEN")
        .ok()
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

fn token_authorized(
    expected: &str,
    authorization: Option<&HeaderValue>,
    x_token: Option<&HeaderValue>,
) -> bool {
    authorization
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| token == expected)
        || x_token
            .and_then(|value| value.to_str().ok())
            .is_some_and(|token| token == expected)
}

async fn require_companion_token(request: Request, next: Next) -> Response {
    if request.method() == Method::OPTIONS || request.uri().path() == "/health" {
        return next.run(request).await;
    }
    let Some(expected) = companion_token() else {
        return next.run(request).await;
    };
    let headers: &HeaderMap = request.headers();
    if token_authorized(
        &expected,
        headers.get(header::AUTHORIZATION),
        headers.get("x-fluxa-companion-token"),
    ) {
        return next.run(request).await;
    }
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "unauthorized" })),
    )
        .into_response()
}

async fn start_torrent(
    State(state): State<Arc<AppState>>,
    Json(body): Json<StartTorrentBody>,
) -> Response {
    match start_torrent_inner(&state, body).await {
        Ok(url) => (StatusCode::OK, Json(json!({ "url": url }))).into_response(),
        Err(message) => (StatusCode::BAD_REQUEST, message).into_response(),
    }
}

async fn start_torrent_inner(state: &AppState, body: StartTorrentBody) -> Result<String, String> {
    let mut guard = state.torrent_base_url.lock().await;
    let base_url = match guard.as_ref() {
        Some(url) => url.clone(),
        None => {
            let cache_dir = std::env::temp_dir().join("fluxa-web-torrent-cache");
            let access_token = std::env::var("FLUXA_COMPANION_TORRENT_TOKEN").unwrap_or_default();
            let server_json =
                crate::start_torrent_server(&cache_dir.to_string_lossy(), 0, &access_token)
                    .ok_or_else(|| "failed to start torrent server".to_string())?;
            let server: Value = serde_json::from_str(&server_json)
                .map_err(|e| format!("invalid torrent server response: {e}"))?;
            let url = server
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| "torrent server did not return url".to_string())?
                .to_string();
            *guard = Some(url.clone());
            url
        }
    };
    drop(guard);

    apply_torrent_preferences(&base_url, body.preferences.as_ref());

    let stream: Value =
        serde_json::from_str(&body.stream_json).map_err(|e| format!("invalid stream json: {e}"))?;
    let playback_json = FluxaCore::stream_playback_info_json(&body.stream_json)
        .ok_or_else(|| "stream playback info could not be resolved".to_string())?;
    let playback: Value =
        serde_json::from_str(&playback_json).map_err(|e| format!("invalid playback info: {e}"))?;
    let link = playback
        .get("playableUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| "torrent stream has no playable link".to_string())?;

    let requested_file_idx = stream
        .get("fileIdx")
        .and_then(Value::as_i64)
        .map(|v| v as i32);
    let preferred_filename = stream
        .get("behaviorHints")
        .and_then(|hints| hints.get("filename"))
        .and_then(Value::as_str)
        .or_else(|| stream.get("filename").and_then(Value::as_str));
    let sources = stream
        .get("sources")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();

    let runtime_request = json!({
        "link": link,
        "title": body.title
            .or_else(|| stream.get("title").and_then(Value::as_str).map(str::to_string))
            .or_else(|| stream.get("name").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "Fluxa stream".to_string()),
        "requestedFileIdx": requested_file_idx,
        "preferredFilename": preferred_filename,
        "sources": sources,
        "fileStats": [],
        "rejectedIndex": Value::Null,
        "baseUrl": base_url,
        "play": true,
        "stat": false,
        "durationMs": stream.get("durationMs").and_then(Value::as_u64)
    });
    let runtime_json = FluxaCore::torrent_runtime_info_json(&runtime_request.to_string())
        .ok_or_else(|| "torrent runtime info could not be resolved".to_string())?;
    let runtime: Value = serde_json::from_str(&runtime_json)
        .map_err(|e| format!("invalid torrent runtime response: {e}"))?;
    runtime
        .get("streamUrl")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "torrent runtime did not return streamUrl".to_string())
}

async fn stop_torrent(State(state): State<Arc<AppState>>) -> Json<bool> {
    *state.torrent_base_url.lock().await = None;
    let stopped = tokio::task::spawn_blocking(|| crate::stop_torrent_server(None))
        .await
        .unwrap_or(false);
    Json(stopped)
}

fn apply_torrent_preferences(base_url: &str, preferences: Option<&Value>) {
    let preset = preferences
        .and_then(|p| p.get("torrentSpeedPreset"))
        .and_then(Value::as_str)
        .unwrap_or("default");
    let preload_size = match preset {
        "fast" => 32,
        "ultra_fast" => 64,
        _ => 16,
    };
    let device_budget = preferences.and_then(|p| p.get("deviceBudget")).cloned();
    let url = format!("{}/settings", base_url.trim_end_matches('/'));
    tokio::spawn(async move {
        static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
        let client = CLIENT.get_or_init(|| {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("torrent settings client build")
        });
        let mut settings = json!({ "PreloadSize": preload_size });
        if let Some(device_budget) = device_budget {
            settings["deviceBudget"] = device_budget;
        }
        let _ = client.post(&url).json(&settings).send().await;
    });
}

// This server proxies OAuth secrets and runs arbitrary ffmpeg/torrent
// commands, so it must NOT accept requests from any origin — only from the
// fluxa-web frontend the user is actually running. Defaults cover the local
// Vite dev server; deployed builds (e.g. on Vercel) need FLUXA_WEB_ORIGIN set
// to the deployed URL, comma-separated if there's more than one.
fn cors_layer() -> CorsLayer {
    let origins: Vec<HeaderValue> = std::env::var("FLUXA_WEB_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:1430,http://127.0.0.1:1430,http://localhost:4173,http://127.0.0.1:4173,https://fluxamedia.github.io".to_string())
        .split(',')
        .filter_map(|s| HeaderValue::from_str(s.trim()).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-fluxa-companion-token"),
        ])
}

pub fn router() -> Router {
    let state = Arc::new(AppState::default());
    Router::new()
        .route("/health", get(health))
        .route("/torrent/start", post(start_torrent))
        .route("/torrent/stop", post(stop_torrent))
        .with_state(state)
        .merge(crate::http_proxy::router())
        .merge(crate::transcode::router())
        .merge(crate::oauth_proxy::router())
        .layer(middleware::from_fn(require_companion_token))
        .layer(cors_layer())
}

/// Binds and serves the companion server, returning once the listener is
/// bound (the serve loop itself keeps running on the current task).
pub async fn serve(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    eprintln!("[companion-server] listening on http://127.0.0.1:{port}");
    axum::serve(listener, router()).await
}

#[cfg(test)]
mod tests {
    use super::token_authorized;
    use axum::http::HeaderValue;

    #[test]
    fn companion_token_accepts_bearer_header() {
        assert!(token_authorized(
            "secret",
            Some(&HeaderValue::from_static("Bearer secret")),
            None,
        ));
    }

    #[test]
    fn companion_token_accepts_custom_header() {
        assert!(token_authorized(
            "secret",
            None,
            Some(&HeaderValue::from_static("secret")),
        ));
    }

    #[test]
    fn companion_token_rejects_missing_or_wrong_headers() {
        assert!(!token_authorized("secret", None, None));
        assert!(!token_authorized(
            "secret",
            Some(&HeaderValue::from_static("Bearer other")),
            None,
        ));
    }
}
