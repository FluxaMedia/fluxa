use fluxa_core::FluxaCore;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;

const SENDER_ID: &str = "sender-fluxa";
const RECEIVER_ID: &str = "receiver-0";
const NS_CONNECTION: &str = "urn:x-cast:com.google.cast.tp.connection";
const NS_HEARTBEAT: &str = "urn:x-cast:com.google.cast.tp.heartbeat";
const NS_RECEIVER: &str = "urn:x-cast:com.google.cast.receiver";
const NS_MEDIA: &str = "urn:x-cast:com.google.cast.media";
const DEFAULT_RECEIVER_APP_ID: &str = "CC1AD845";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChromecastDevice {
    pub id: String,
    pub name: String,
    host: String,
    port: u16,
}

enum SessionCmd {
    Play,
    Pause,
    Disconnect,
    Seek(f64),
    SetVolume(f64),
}

pub struct ChromecastState {
    cmd_tx: Mutex<Option<mpsc::UnboundedSender<SessionCmd>>>,
}

impl Default for ChromecastState {
    fn default() -> Self {
        Self {
            cmd_tx: Mutex::new(None),
        }
    }
}

#[derive(Debug)]
struct AcceptAnyServerCert;

impl ServerCertVerifier for AcceptAnyServerCert {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

struct DecodedMessage {
    namespace: String,
    payload_utf8: String,
}

async fn send_message(
    stream: &mut TlsStream<TcpStream>,
    source: &str,
    destination: &str,
    namespace: &str,
    payload: &Value,
) -> Result<(), String> {
    let msg =
        FluxaCore::chromecast_encode_message(source, destination, namespace, &payload.to_string());
    let len = (msg.len() as u32).to_be_bytes();
    stream.write_all(&len).await.map_err(|e| e.to_string())?;
    stream.write_all(&msg).await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn read_message(stream: &mut TlsStream<TcpStream>) -> Result<DecodedMessage, String> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| e.to_string())?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream
        .read_exact(&mut buf)
        .await
        .map_err(|e| e.to_string())?;
    let (namespace, payload_utf8) = FluxaCore::chromecast_decode_message(&buf)
        .ok_or_else(|| "malformed cast message".to_string())?;
    Ok(DecodedMessage {
        namespace,
        payload_utf8,
    })
}

async fn wait_for_json<F>(
    stream: &mut TlsStream<TcpStream>,
    namespace: &str,
    mut matches: F,
) -> Result<Value, String>
where
    F: FnMut(&Value) -> bool,
{
    let result = timeout(Duration::from_secs(8), async {
        loop {
            let decoded = read_message(stream).await?;
            if decoded.namespace == NS_HEARTBEAT {
                continue;
            }
            if decoded.namespace != namespace {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<Value>(&decoded.payload_utf8) {
                if matches(&value) {
                    return Ok(value);
                }
            }
        }
    })
    .await;
    result.map_err(|_| "timed out waiting for cast device response".to_string())?
}

#[tauri::command]
pub async fn chromecast_discover_devices() -> Result<Vec<ChromecastDevice>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
        let receiver = daemon
            .browse("_googlecast._tcp.local.")
            .map_err(|e| e.to_string())?;
        let mut devices = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while std::time::Instant::now() < deadline {
            if let Ok(ServiceEvent::ServiceResolved(info)) =
                receiver.recv_timeout(Duration::from_millis(300))
            {
                if let Some(addr) = info.get_addresses().iter().next() {
                    let name = info
                        .get_property_val_str("fn")
                        .unwrap_or_else(|| info.get_fullname())
                        .to_string();
                    let host = addr.to_string();
                    let port = info.get_port();
                    devices.push(ChromecastDevice {
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

async fn session_loop(
    mut stream: TlsStream<TcpStream>,
    transport_id: String,
    mut media_session_id: i64,
    mut request_id: i64,
    mut cmd_rx: mpsc::UnboundedReceiver<SessionCmd>,
) {
    loop {
        tokio::select! {
            incoming = read_message(&mut stream) => {
                match incoming {
                    Ok(decoded) if decoded.namespace == NS_HEARTBEAT => {
                        let _ = send_message(&mut stream, SENDER_ID, RECEIVER_ID, NS_HEARTBEAT, &json!({"type": "PONG"})).await;
                    }
                    Ok(decoded) if decoded.namespace == NS_MEDIA => {
                        if let Ok(value) = serde_json::from_str::<Value>(&decoded.payload_utf8) {
                            if let Some(id) = value.get("status").and_then(|s| s.get(0)).and_then(|s| s.get("mediaSessionId")).and_then(Value::as_i64) {
                                media_session_id = id;
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
            cmd = cmd_rx.recv() => {
                let Some(cmd) = cmd else { break };
                request_id += 1;
                if let SessionCmd::SetVolume(level) = cmd {
                    let _ = send_message(&mut stream, SENDER_ID, RECEIVER_ID, NS_RECEIVER, &json!({
                        "type": "SET_VOLUME",
                        "volume": {"level": level.clamp(0.0, 1.0)},
                        "requestId": request_id,
                    })).await;
                    continue;
                }
                let payload = match cmd {
                    SessionCmd::Play => json!({"type": "PLAY", "mediaSessionId": media_session_id, "requestId": request_id}),
                    SessionCmd::Pause => json!({"type": "PAUSE", "mediaSessionId": media_session_id, "requestId": request_id}),
                    SessionCmd::Seek(position_secs) => json!({"type": "SEEK", "mediaSessionId": media_session_id, "currentTime": position_secs, "requestId": request_id}),
                    SessionCmd::Disconnect => json!({"type": "STOP", "mediaSessionId": media_session_id, "requestId": request_id}),
                    SessionCmd::SetVolume(_) => unreachable!(),
                };
                let _ = send_message(&mut stream, SENDER_ID, &transport_id, NS_MEDIA, &payload).await;
                if matches!(cmd, SessionCmd::Disconnect) {
                    break;
                }
            }
        }
    }
}

#[tauri::command]
pub async fn chromecast_connect(
    state: tauri::State<'_, ChromecastState>,
    host: String,
    port: u16,
    media_url: String,
    title: String,
    subtitle_url: Option<String>,
) -> Result<(), String> {
    if !FluxaCore::validate_stream_url(&media_url) {
        return Err("unsupported media url".to_string());
    }
    if let Some(subtitle_url) = &subtitle_url {
        if !FluxaCore::validate_stream_url(subtitle_url) {
            return Err("unsupported subtitle url".to_string());
        }
    }
    if let Some(tx) = state.cmd_tx.lock().unwrap().take() {
        let _ = tx.send(SessionCmd::Disconnect);
    }

    let tcp = TcpStream::connect((host.as_str(), port))
        .await
        .map_err(|e| e.to_string())?;
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyServerCert))
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let server_name = ServerName::try_from(host.clone()).map_err(|_| "invalid host".to_string())?;
    let mut stream = connector
        .connect(server_name, tcp)
        .await
        .map_err(|e| e.to_string())?;

    send_message(
        &mut stream,
        SENDER_ID,
        RECEIVER_ID,
        NS_CONNECTION,
        &json!({"type": "CONNECT"}),
    )
    .await?;

    let mut request_id = 1i64;
    send_message(
        &mut stream,
        SENDER_ID,
        RECEIVER_ID,
        NS_RECEIVER,
        &json!({"type": "LAUNCH", "appId": DEFAULT_RECEIVER_APP_ID, "requestId": request_id}),
    )
    .await?;

    let status = wait_for_json(&mut stream, NS_RECEIVER, |v| {
        v.get("status")
            .and_then(|s| s.get("applications"))
            .and_then(Value::as_array)
            .map(|apps| {
                apps.iter().any(|a| {
                    a.get("appId").and_then(Value::as_str) == Some(DEFAULT_RECEIVER_APP_ID)
                })
            })
            .unwrap_or(false)
    })
    .await?;

    let transport_id = status
        .get("status")
        .and_then(|s| s.get("applications"))
        .and_then(Value::as_array)
        .and_then(|apps| {
            apps.iter()
                .find(|a| a.get("appId").and_then(Value::as_str) == Some(DEFAULT_RECEIVER_APP_ID))
        })
        .and_then(|app| app.get("transportId"))
        .and_then(Value::as_str)
        .ok_or_else(|| "chromecast did not return a transport id".to_string())?
        .to_string();

    send_message(
        &mut stream,
        SENDER_ID,
        &transport_id,
        NS_CONNECTION,
        &json!({"type": "CONNECT"}),
    )
    .await?;

    request_id += 1;
    let load_request_id = request_id;
    let mut media = json!({
        "contentId": media_url,
        "contentType": FluxaCore::chromecast_guess_content_type(&media_url),
        "streamType": "BUFFERED",
    });
    let mut active_track_ids: Vec<i64> = Vec::new();
    if let Some(subtitle_url) = &subtitle_url {
        media["tracks"] = json!([{
            "trackId": 1,
            "type": "TEXT",
            "subtype": "SUBTITLES",
            "trackContentId": subtitle_url,
            "trackContentType": "text/vtt",
            "name": "Subtitles",
            "language": "en",
        }]);
        active_track_ids.push(1);
    }
    send_message(
        &mut stream,
        SENDER_ID,
        &transport_id,
        NS_MEDIA,
        &json!({
            "type": "LOAD",
            "requestId": load_request_id,
            "media": media,
            "currentTime": 0,
            "autoplay": true,
            "activeTrackIds": active_track_ids,
            "customData": {"title": title}
        }),
    )
    .await?;

    let media_status = wait_for_json(&mut stream, NS_MEDIA, |v| {
        v.get("status")
            .and_then(|s| s.get(0))
            .and_then(|s| s.get("mediaSessionId"))
            .is_some()
    })
    .await?;

    let media_session_id = media_status
        .get("status")
        .and_then(|s| s.get(0))
        .and_then(|s| s.get("mediaSessionId"))
        .and_then(Value::as_i64)
        .ok_or_else(|| "chromecast did not return a media session id".to_string())?;

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    *state.cmd_tx.lock().unwrap() = Some(cmd_tx);
    tauri::async_runtime::spawn(session_loop(
        stream,
        transport_id,
        media_session_id,
        request_id,
        cmd_rx,
    ));

    Ok(())
}

fn send_session_cmd(
    state: &tauri::State<'_, ChromecastState>,
    cmd: SessionCmd,
) -> Result<(), String> {
    state
        .cmd_tx
        .lock()
        .unwrap()
        .as_ref()
        .ok_or_else(|| "no active chromecast session".to_string())?
        .send(cmd)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn chromecast_play(state: tauri::State<ChromecastState>) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::Play)
}

#[tauri::command]
pub fn chromecast_pause(state: tauri::State<ChromecastState>) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::Pause)
}

#[tauri::command]
pub fn chromecast_seek(
    state: tauri::State<ChromecastState>,
    position_secs: f64,
) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::Seek(position_secs))
}

#[tauri::command]
pub fn chromecast_set_volume(
    state: tauri::State<ChromecastState>,
    level: f64,
) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::SetVolume(level))
}

#[tauri::command]
pub fn chromecast_disconnect(state: tauri::State<ChromecastState>) -> Result<(), String> {
    let result = send_session_cmd(&state, SessionCmd::Disconnect);
    *state.cmd_tx.lock().unwrap() = None;
    result
}
