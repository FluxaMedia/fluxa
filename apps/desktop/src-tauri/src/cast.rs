use fluxa_core::FluxaCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

const SSDP_ADDR: &str = "239.255.255.250:1900";
const AVTRANSPORT_URN: &str = "urn:schemas-upnp-org:service:AVTransport:1";
const RENDERING_CONTROL_URN: &str = "urn:schemas-upnp-org:service:RenderingControl:1";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CastDevice {
    pub id: String,
    pub name: String,
    control_url: String,
    rendering_control_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParsedDeviceDescription {
    name: String,
    control_url: String,
    rendering_control_url: Option<String>,
}

struct ActiveCast {
    control_url: String,
    rendering_control_url: Option<String>,
}

pub struct CastState {
    devices: Mutex<HashMap<String, CastDevice>>,
    active: Mutex<Option<ActiveCast>>,
}

impl Default for CastState {
    fn default() -> Self {
        Self {
            devices: Mutex::new(HashMap::new()),
            active: Mutex::new(None),
        }
    }
}

pub(crate) fn lan_ip() -> Option<IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}

async fn fetch_device_description(location: &str) -> Option<CastDevice> {
    let body = reqwest::get(location).await.ok()?.text().await.ok()?;
    let parsed: ParsedDeviceDescription =
        serde_json::from_str(&FluxaCore::dlna_parse_device_description(&body, location)?).ok()?;
    Some(CastDevice {
        id: location.to_string(),
        name: parsed.name,
        control_url: parsed.control_url,
        rendering_control_url: parsed.rendering_control_url,
    })
}

#[tauri::command]
pub async fn cast_discover_devices(
    state: tauri::State<'_, CastState>,
) -> Result<Vec<CastDevice>, String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| e.to_string())?;
    let search = "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\nST: urn:schemas-upnp-org:service:AVTransport:1\r\n\r\n";
    socket
        .send_to(search.as_bytes(), SSDP_ADDR)
        .await
        .map_err(|e| e.to_string())?;

    let mut locations: Vec<String> = Vec::new();
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
                    let location = location.trim().to_string();
                    if !locations.contains(&location) {
                        locations.push(location);
                    }
                }
            }
        }
    });
    let _ = deadline.await;

    let mut found = Vec::new();
    for location in locations {
        if let Some(device) = fetch_device_description(&location).await {
            found.push(device);
        }
    }

    let mut guard = state.devices.lock().unwrap();
    guard.clear();
    for device in &found {
        guard.insert(device.id.clone(), device.clone());
    }
    Ok(found)
}

async fn send_soap_action(
    control_url: &str,
    urn: &str,
    action: &str,
    args: &str,
) -> Result<(), String> {
    let body = FluxaCore::dlna_soap_action_body(urn, action, args);
    let client = reqwest::Client::new();
    client
        .post(control_url)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .header("SOAPACTION", format!("\"{urn}#{action}\""))
        .body(body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cast_resolve_media_url(stream_url: String) -> String {
    match lan_ip() {
        Some(ip) => FluxaCore::dlna_resolve_loopback_url(&stream_url, &ip.to_string()),
        None => stream_url,
    }
}

#[tauri::command]
pub async fn cast_set_media(
    state: tauri::State<'_, CastState>,
    device_id: String,
    media_url: String,
    title: String,
    subtitle_url: Option<String>,
) -> Result<(), String> {
    let (control_url, rendering_control_url) = {
        let guard = state.devices.lock().unwrap();
        let device = guard
            .get(&device_id)
            .ok_or_else(|| "device not found, discover again".to_string())?;
        (
            device.control_url.clone(),
            device.rendering_control_url.clone(),
        )
    };

    let args = FluxaCore::dlna_set_av_transport_args(&media_url, &title, subtitle_url.as_deref())
        .ok_or_else(|| "unsupported media url".to_string())?;
    send_soap_action(&control_url, AVTRANSPORT_URN, "SetAVTransportURI", &args).await?;
    send_soap_action(
        &control_url,
        AVTRANSPORT_URN,
        "Play",
        "<InstanceID>0</InstanceID><Speed>1</Speed>",
    )
    .await?;

    *state.active.lock().unwrap() = Some(ActiveCast {
        control_url,
        rendering_control_url,
    });
    Ok(())
}

fn active_session(state: &tauri::State<'_, CastState>) -> Result<(String, Option<String>), String> {
    state
        .active
        .lock()
        .unwrap()
        .as_ref()
        .map(|a| (a.control_url.clone(), a.rendering_control_url.clone()))
        .ok_or_else(|| "no active cast session".to_string())
}

#[tauri::command]
pub async fn cast_play(state: tauri::State<'_, CastState>) -> Result<(), String> {
    let (control_url, _) = active_session(&state)?;
    send_soap_action(
        &control_url,
        AVTRANSPORT_URN,
        "Play",
        "<InstanceID>0</InstanceID><Speed>1</Speed>",
    )
    .await
}

#[tauri::command]
pub async fn cast_pause(state: tauri::State<'_, CastState>) -> Result<(), String> {
    let (control_url, _) = active_session(&state)?;
    send_soap_action(
        &control_url,
        AVTRANSPORT_URN,
        "Pause",
        "<InstanceID>0</InstanceID>",
    )
    .await
}

#[tauri::command]
pub async fn cast_seek(
    state: tauri::State<'_, CastState>,
    position_secs: f64,
) -> Result<(), String> {
    let (control_url, _) = active_session(&state)?;
    let args = FluxaCore::dlna_seek_args(position_secs);
    send_soap_action(&control_url, AVTRANSPORT_URN, "Seek", &args).await
}

#[tauri::command]
pub async fn cast_set_volume(state: tauri::State<'_, CastState>, level: f64) -> Result<(), String> {
    let (_, rendering_control_url) = active_session(&state)?;
    let rendering_control_url = rendering_control_url
        .ok_or_else(|| "device does not support volume control".to_string())?;
    let args = FluxaCore::dlna_set_volume_args(level);
    send_soap_action(
        &rendering_control_url,
        RENDERING_CONTROL_URN,
        "SetVolume",
        &args,
    )
    .await
}

#[tauri::command]
pub async fn cast_disconnect(state: tauri::State<'_, CastState>) -> Result<(), String> {
    let (control_url, _) = active_session(&state)?;
    let result = send_soap_action(
        &control_url,
        AVTRANSPORT_URN,
        "Stop",
        "<InstanceID>0</InstanceID>",
    )
    .await;
    *state.active.lock().unwrap() = None;
    result
}
