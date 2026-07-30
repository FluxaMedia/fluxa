use crate::DesktopState;
use fluxa_core::FluxaCore;
use serde_json::Value;
use std::collections::HashMap;
use tauri::State;

#[derive(serde::Serialize)]
pub struct HttpTextResponse {
    pub status_code: u16,
    pub body: String,
}

#[tauri::command]
pub fn engine_init(state: State<DesktopState>, initial_json: String) -> u64 {
    let handle = FluxaCore::create_headless_engine(&initial_json);
    *state.engine_handle.lock().unwrap() = Some(handle);
    handle
}

#[tauri::command]
pub fn engine_dispatch(state: State<DesktopState>, action_json: String) -> Option<String> {
    FluxaCore::headless_engine_dispatch_json((*state.engine_handle.lock().unwrap())?, &action_json)
}

#[tauri::command]
pub fn engine_complete_effect(state: State<DesktopState>, result_json: String) -> Option<String> {
    FluxaCore::headless_engine_complete_effect_json((*state.engine_handle.lock().unwrap())?, &result_json)
}

#[tauri::command]
pub fn engine_snapshot(state: State<DesktopState>) -> Option<String> {
    FluxaCore::headless_engine_snapshot_json((*state.engine_handle.lock().unwrap())?)
}

#[tauri::command]
pub async fn http_fetch_text(url: String) -> Result<HttpTextResponse, String> {
    let response = crate::net_guard::vetted_client(&url, std::time::Duration::from_secs(10))
        .await?
        .get(&url)
        .header("User-Agent", "Fluxa/1.0")
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status_code = response.status().as_u16();
    let body = response.text().await.map_err(|error| error.to_string())?;
    Ok(HttpTextResponse { status_code, body })
}

#[tauri::command]
pub async fn http_execute_text(url: String, method: String, headers: HashMap<String, String>, body: Option<Value>) -> Result<HttpTextResponse, String> {
    let client = crate::net_guard::vetted_client(&url, std::time::Duration::from_secs(10)).await?;
    let method = reqwest::Method::from_bytes(method.as_bytes()).map_err(|error| error.to_string())?;
    let mut request = client.request(method, &url).header("User-Agent", "Fluxa/1.0");
    for (name, value) in headers {
        request = request.header(name, value);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.map_err(|error| error.to_string())?;
    let status_code = response.status().as_u16();
    let body = response.text().await.map_err(|error| error.to_string())?;
    Ok(HttpTextResponse { status_code, body })
}

#[tauri::command]
pub async fn run_plugin_scraper(code: String, scraper_id: String, scraper_settings_json: String, tmdb_id: String, media_type: String, season: Option<i32>, episode: Option<i32>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || crate::plugin_executor::execute_scraper(code, scraper_id, scraper_settings_json, tmdb_id, media_type, season, episode))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn core_invoke(method: String, args_json: String) -> String {
    tauri::async_runtime::spawn_blocking(move || fluxa_core::ffi::core_invoke(&method, &args_json)).await.unwrap_or_default()
}
