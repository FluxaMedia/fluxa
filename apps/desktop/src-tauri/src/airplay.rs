use fluxa_core::FluxaCore;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AirplayDevice {
    pub id: String,
    pub name: String,
    host: String,
    port: u16,
}

#[derive(Default)]
pub struct AirplayState {
    active: Mutex<Option<(String, u16)>>,
}

#[tauri::command]
pub async fn airplay_discover_devices() -> Result<Vec<AirplayDevice>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
        let receiver = daemon
            .browse("_airplay._tcp.local.")
            .map_err(|e| e.to_string())?;
        let mut devices = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while std::time::Instant::now() < deadline {
            if let Ok(ServiceEvent::ServiceResolved(info)) =
                receiver.recv_timeout(Duration::from_millis(300))
            {
                if let Some(addr) = info.get_addresses().iter().next() {
                    let name = info
                        .get_fullname()
                        .trim_end_matches(".local.")
                        .trim_end_matches("._airplay._tcp")
                        .to_string();
                    let host = addr.to_string();
                    let port = info.get_port();
                    devices.push(AirplayDevice {
                        id: format!("{host}:{port}"),
                        name,
                        host,
                        port,
                    });
                }
            }
        }
        let _ = daemon.shutdown();
        Ok(devices)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn active_endpoint(state: &tauri::State<'_, AirplayState>) -> Result<(String, u16), String> {
    state
        .active
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "no active airplay session".to_string())
}

#[tauri::command]
pub async fn airplay_set_media(
    state: tauri::State<'_, AirplayState>,
    host: String,
    port: u16,
    media_url: String,
) -> Result<(), String> {
    let body = FluxaCore::airplay_play_body(&media_url)
        .ok_or_else(|| "unsupported media url".to_string())?;
    reqwest::Client::new()
        .put(format!("http://{host}:{port}/play"))
        .header("Content-Type", "text/parameters")
        .body(body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    *state.active.lock().unwrap() = Some((host, port));
    Ok(())
}

#[tauri::command]
pub async fn airplay_play(state: tauri::State<'_, AirplayState>) -> Result<(), String> {
    let (host, port) = active_endpoint(&state)?;
    reqwest::Client::new()
        .post(format!("http://{host}:{port}/rate?value=1.000000"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn airplay_pause(state: tauri::State<'_, AirplayState>) -> Result<(), String> {
    let (host, port) = active_endpoint(&state)?;
    reqwest::Client::new()
        .post(format!("http://{host}:{port}/rate?value=0.000000"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn airplay_seek(
    state: tauri::State<'_, AirplayState>,
    position_secs: f64,
) -> Result<(), String> {
    let (host, port) = active_endpoint(&state)?;
    reqwest::Client::new()
        .post(format!(
            "http://{host}:{port}/scrub?position={position_secs:.3}"
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn airplay_set_volume(
    state: tauri::State<'_, AirplayState>,
    level: f64,
) -> Result<(), String> {
    let (host, port) = active_endpoint(&state)?;
    let db = FluxaCore::airplay_volume_db(level);
    reqwest::Client::new()
        .post(format!("http://{host}:{port}/volume?volume={db:.3}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn airplay_disconnect(state: tauri::State<'_, AirplayState>) -> Result<(), String> {
    let (host, port) = active_endpoint(&state)?;
    let result = reqwest::Client::new()
        .post(format!("http://{host}:{port}/stop"))
        .send()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string());
    *state.active.lock().unwrap() = None;
    result
}
