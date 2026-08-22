use fluxa_core::FluxaCore;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

const SSDP_ADDR: &str = "239.255.255.250:1900";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RokuDevice {
    pub id: String,
    pub name: String,
    host: String,
}

#[derive(Default)]
pub struct RokuState {
    active_host: Mutex<Option<String>>,
}

async fn fetch_device_name(host: &str) -> String {
    let url = format!("http://{host}:8060/query/device-info");
    match reqwest::get(&url).await {
        Ok(resp) => match resp.text().await {
            Ok(body) => FluxaCore::roku_device_name(&body).unwrap_or_else(|| "Roku".to_string()),
            Err(_) => "Roku".to_string(),
        },
        Err(_) => "Roku".to_string(),
    }
}

#[tauri::command]
pub async fn roku_discover_devices() -> Result<Vec<RokuDevice>, String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| e.to_string())?;
    let search = "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\nST: roku:ecp\r\n\r\n";
    socket
        .send_to(search.as_bytes(), SSDP_ADDR)
        .await
        .map_err(|e| e.to_string())?;

    let mut hosts: Vec<String> = Vec::new();
    let mut buf = [0u8; 2048];
    let deadline = timeout(Duration::from_secs(3), async {
        loop {
            if let Ok((len, _)) = socket.recv_from(&mut buf).await {
                let response = String::from_utf8_lossy(&buf[..len]);
                if let Some(location) = response
                    .lines()
                    .find(|line| line.to_ascii_uppercase().starts_with("LOCATION:"))
                    .and_then(|line| line.split_once(':').map(|(_, v)| v))
                {
                    if let Some(host) = location
                        .trim()
                        .strip_prefix("http://")
                        .and_then(|rest| rest.split('/').next())
                    {
                        let host = host.split(':').next().unwrap_or(host).to_string();
                        if !hosts.contains(&host) {
                            hosts.push(host);
                        }
                    }
                }
            }
        }
    });
    let _ = deadline.await;

    let mut found = Vec::new();
    for host in hosts {
        let name = fetch_device_name(&host).await;
        found.push(RokuDevice {
            id: host.clone(),
            name,
            host,
        });
    }
    Ok(found)
}

#[tauri::command]
pub async fn roku_set_media(
    state: tauri::State<'_, RokuState>,
    host: String,
    media_url: String,
    subtitle_url: Option<String>,
) -> Result<(), String> {
    let url = FluxaCore::roku_launch_url(&host, &media_url, subtitle_url.as_deref())
        .ok_or_else(|| "unsupported media url".to_string())?;
    reqwest::Client::new()
        .post(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    *state.active_host.lock().unwrap() = Some(host);
    Ok(())
}

fn active_host(state: &tauri::State<'_, RokuState>) -> Result<String, String> {
    state
        .active_host
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "no active roku session".to_string())
}

#[tauri::command]
pub async fn roku_play_pause(state: tauri::State<'_, RokuState>) -> Result<(), String> {
    let host = active_host(&state)?;
    reqwest::Client::new()
        .post(format!("http://{host}:8060/keypress/Play"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn roku_disconnect(state: tauri::State<'_, RokuState>) -> Result<(), String> {
    let host = active_host(&state)?;
    let result = reqwest::Client::new()
        .post(format!("http://{host}:8060/keypress/Home"))
        .send()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string());
    *state.active_host.lock().unwrap() = None;
    result
}
