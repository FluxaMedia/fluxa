use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use std::time::Duration;

// Mirrors apps/desktop/src-tauri/src/oauth.rs's token exchange — the
// client_secret can't ship in browser code, so the companion server holds it
// and the browser only ever sees the authorization code.
fn env_or_empty(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

fn redirect_uri(service: &str) -> String {
    let env_key = format!("FLUXA_{}_WEB_REDIRECT_URI", service.to_uppercase());
    std::env::var(&env_key).unwrap_or_else(|_| format!("http://127.0.0.1:1430/oauth/{service}"))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeBody {
    code: String,
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())
}

async fn forward(
    res: Result<reqwest::Response, reqwest::Error>,
    failure_label: &str,
) -> Result<String, (StatusCode, String)> {
    let res = res.map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let status = res.status();
    let text = res
        .text()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    if !status.is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("{failure_label} failed: HTTP {status}: {text}"),
        ));
    }
    Ok(text)
}

async fn trakt_exchange(body: ExchangeBody) -> Result<String, (StatusCode, String)> {
    let client = client().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let res = client
        .post("https://api.trakt.tv/oauth/token")
        .json(&serde_json::json!({
            "code": body.code,
            "client_id": env_or_empty("FLUXA_TRAKT_CLIENT_ID"),
            "client_secret": env_or_empty("FLUXA_TRAKT_CLIENT_SECRET"),
            "redirect_uri": redirect_uri("trakt"),
            "grant_type": "authorization_code",
        }))
        .send()
        .await;
    forward(res, "Trakt token exchange").await
}

async fn simkl_exchange(body: ExchangeBody) -> Result<String, (StatusCode, String)> {
    let client = client().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let res = client
        .post("https://api.simkl.com/oauth/token")
        .json(&serde_json::json!({
            "code": body.code,
            "client_id": env_or_empty("FLUXA_SIMKL_CLIENT_ID"),
            "client_secret": env_or_empty("FLUXA_SIMKL_CLIENT_SECRET"),
            "redirect_uri": redirect_uri("simkl"),
            "grant_type": "authorization_code",
        }))
        .send()
        .await;
    forward(res, "SIMKL token exchange").await
}

async fn handle_exchange(Path(service): Path<String>, Json(body): Json<ExchangeBody>) -> Response {
    let result = match service.as_str() {
        "trakt" => trakt_exchange(body).await,
        "simkl" => simkl_exchange(body).await,
        _ => Err((
            StatusCode::NOT_FOUND,
            format!("unknown oauth service `{service}`"),
        )),
    };
    match result {
        Ok(text) => (StatusCode::OK, text).into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}

pub fn router() -> Router {
    Router::new().route("/oauth/{service}/exchange", post(handle_exchange))
}
