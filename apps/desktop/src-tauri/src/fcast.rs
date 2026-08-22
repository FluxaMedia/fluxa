use fluxa_core::FluxaCore;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

const SERVICE_TYPE: &str = "_fcast._tcp.local.";
const DEFAULT_PORT: u16 = 46899;
const PROTOCOL_VERSION: u32 = 2;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FcastDevice {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FcastStatus {
    pub state: u8,
    pub time: f64,
    pub duration: f64,
    pub speed: f64,
}

enum SessionCmd {
    Pause,
    Resume,
    Stop,
    Seek(f64),
    SetVolume(f64),
    SetSpeed(f64),
    Disconnect,
}

#[derive(Default)]
pub struct FcastState {
    cmd_tx: Mutex<Option<mpsc::UnboundedSender<SessionCmd>>>,
}

async fn send(stream: &mut TcpStream, opcode: u8, body_json: &str) -> Result<(), String> {
    let message = FluxaCore::fcast_encode_message(opcode, body_json)
        .ok_or_else(|| "fcast message too large".to_string())?;
    let size = (message.len() as u32).to_le_bytes();
    stream.write_all(&size).await.map_err(|e| e.to_string())?;
    stream.write_all(&message).await.map_err(|e| e.to_string())
}

async fn read_message(stream: &mut TcpStream) -> Result<(u8, String), String> {
    let mut size_buf = [0u8; 4];
    stream
        .read_exact(&mut size_buf)
        .await
        .map_err(|e| e.to_string())?;
    let size = u32::from_le_bytes(size_buf) as usize;
    if size == 0 || size > FluxaCore::FCAST_MAX_MESSAGE_BYTES {
        return Err("invalid fcast message size".to_string());
    }
    let mut buf = vec![0u8; size];
    stream
        .read_exact(&mut buf)
        .await
        .map_err(|e| e.to_string())?;
    FluxaCore::fcast_decode_message(&buf).ok_or_else(|| "malformed fcast message".to_string())
}

#[tauri::command]
pub async fn fcast_discover_devices() -> Result<Vec<FcastDevice>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
        let receiver = daemon.browse(SERVICE_TYPE).map_err(|e| e.to_string())?;
        let mut devices: Vec<FcastDevice> = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while std::time::Instant::now() < deadline {
            if let Ok(ServiceEvent::ServiceResolved(info)) =
                receiver.recv_timeout(Duration::from_millis(300))
            {
                if let Some(addr) = info.get_addresses().iter().next() {
                    let host = addr.to_string();
                    let port = if info.get_port() == 0 {
                        DEFAULT_PORT
                    } else {
                        info.get_port()
                    };
                    let id = format!("{host}:{port}");
                    if devices.iter().any(|device| device.id == id) {
                        continue;
                    }
                    let name = info
                        .get_property_val_str("name")
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| {
                            info.get_fullname()
                                .split('.')
                                .next()
                                .unwrap_or("FCast")
                                .replace('\\', "")
                        });
                    devices.push(FcastDevice {
                        id,
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

#[tauri::command]
pub async fn fcast_connect(
    app: tauri::AppHandle,
    state: tauri::State<'_, FcastState>,
    host: String,
    port: Option<u16>,
    media_url: String,
    resume_position_secs: Option<f64>,
) -> Result<(), String> {
    let play_body = FluxaCore::fcast_play_body(&media_url, resume_position_secs.unwrap_or(0.0))
        .ok_or_else(|| "unsupported media url".to_string())?;

    if let Some(tx) = state.cmd_tx.lock().unwrap().take() {
        let _ = tx.send(SessionCmd::Disconnect);
    }

    let mut stream = TcpStream::connect((host.as_str(), port.unwrap_or(DEFAULT_PORT)))
        .await
        .map_err(|e| e.to_string())?;

    send(
        &mut stream,
        FluxaCore::FCAST_OP_VERSION,
        &FluxaCore::fcast_version_body(PROTOCOL_VERSION),
    )
    .await?;
    send(&mut stream, FluxaCore::FCAST_OP_PLAY, &play_body).await?;

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    *state.cmd_tx.lock().unwrap() = Some(cmd_tx);
    tauri::async_runtime::spawn(session_loop(app, stream, cmd_rx));
    Ok(())
}

async fn session_loop(
    app: tauri::AppHandle,
    mut stream: TcpStream,
    mut cmd_rx: mpsc::UnboundedReceiver<SessionCmd>,
) {
    let mut keepalive = tokio::time::interval(Duration::from_secs(15));
    keepalive.tick().await;
    loop {
        tokio::select! {
            incoming = read_message(&mut stream) => {
                let Ok((opcode, body)) = incoming else { break };
                if opcode == FluxaCore::FCAST_OP_PING {
                    if send(&mut stream, FluxaCore::FCAST_OP_PONG, "").await.is_err() {
                        break;
                    }
                } else if opcode == FluxaCore::FCAST_OP_PLAYBACK_UPDATE {
                    if let Some((state, time, duration, speed)) = FluxaCore::fcast_playback_update(&body) {
                        let _ = app.emit("fcast-status", FcastStatus { state, time, duration, speed });
                    }
                } else if opcode == FluxaCore::FCAST_OP_PLAYBACK_ERROR {
                    let message = FluxaCore::fcast_error_message(&body)
                        .unwrap_or_else(|| "playback failed".to_string());
                    let _ = app.emit("fcast-error", message);
                }
            }
            command = cmd_rx.recv() => {
                let Some(command) = command else { break };
                let sent = match command {
                    SessionCmd::Pause => send(&mut stream, FluxaCore::FCAST_OP_PAUSE, "").await,
                    SessionCmd::Resume => send(&mut stream, FluxaCore::FCAST_OP_RESUME, "").await,
                    SessionCmd::Stop => send(&mut stream, FluxaCore::FCAST_OP_STOP, "").await,
                    SessionCmd::Seek(position) => {
                        send(&mut stream, FluxaCore::FCAST_OP_SEEK, &FluxaCore::fcast_seek_body(position)).await
                    }
                    SessionCmd::SetVolume(level) => {
                        send(&mut stream, FluxaCore::FCAST_OP_SET_VOLUME, &FluxaCore::fcast_set_volume_body(level)).await
                    }
                    SessionCmd::SetSpeed(speed) => {
                        send(&mut stream, FluxaCore::FCAST_OP_SET_SPEED, &FluxaCore::fcast_set_speed_body(speed)).await
                    }
                    SessionCmd::Disconnect => {
                        let _ = send(&mut stream, FluxaCore::FCAST_OP_STOP, "").await;
                        break;
                    }
                };
                if sent.is_err() {
                    break;
                }
            }
            _ = keepalive.tick() => {
                if send(&mut stream, FluxaCore::FCAST_OP_PING, "").await.is_err() {
                    break;
                }
            }
        }
    }
    let _ = app.emit("fcast-disconnected", ());
}

fn send_session_cmd(state: &tauri::State<'_, FcastState>, cmd: SessionCmd) -> Result<(), String> {
    state
        .cmd_tx
        .lock()
        .unwrap()
        .as_ref()
        .ok_or_else(|| "no active fcast session".to_string())?
        .send(cmd)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn fcast_play(state: tauri::State<FcastState>) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::Resume)
}

#[tauri::command]
pub fn fcast_pause(state: tauri::State<FcastState>) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::Pause)
}

#[tauri::command]
pub fn fcast_seek(state: tauri::State<FcastState>, position_secs: f64) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::Seek(position_secs))
}

#[tauri::command]
pub fn fcast_set_volume(state: tauri::State<FcastState>, level: f64) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::SetVolume(level))
}

#[tauri::command]
pub fn fcast_set_speed(state: tauri::State<FcastState>, speed: f64) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::SetSpeed(speed))
}

#[tauri::command]
pub fn fcast_stop(state: tauri::State<FcastState>) -> Result<(), String> {
    send_session_cmd(&state, SessionCmd::Stop)
}

#[tauri::command]
pub fn fcast_disconnect(state: tauri::State<FcastState>) -> Result<(), String> {
    let result = send_session_cmd(&state, SessionCmd::Disconnect);
    *state.cmd_tx.lock().unwrap() = None;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    async fn connected_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr);
        let server = listener.accept();
        let (client, server) = tokio::join!(client, server);
        (client.unwrap(), server.unwrap().0)
    }

    #[tokio::test]
    async fn a_sent_message_arrives_behind_a_little_endian_size_prefix() {
        let (mut client, mut receiver) = connected_pair().await;
        let body = FluxaCore::fcast_seek_body(30.0);
        send(&mut client, FluxaCore::FCAST_OP_SEEK, &body)
            .await
            .unwrap();

        let mut size_buf = [0u8; 4];
        receiver.read_exact(&mut size_buf).await.unwrap();
        let size = u32::from_le_bytes(size_buf) as usize;
        let mut message = vec![0u8; size];
        receiver.read_exact(&mut message).await.unwrap();

        assert_eq!(size, body.len() + 1);
        assert_eq!(message[0], FluxaCore::FCAST_OP_SEEK);
        assert_eq!(String::from_utf8(message[1..].to_vec()).unwrap(), body);
    }

    #[tokio::test]
    async fn a_receivers_playback_update_is_read_back_whole() {
        let (mut client, mut receiver) = connected_pair().await;
        let body = r#"{"generationTime":1,"time":42,"duration":120,"state":1,"speed":1}"#;
        send(&mut receiver, FluxaCore::FCAST_OP_PLAYBACK_UPDATE, body)
            .await
            .unwrap();

        let (opcode, received) = read_message(&mut client).await.unwrap();
        assert_eq!(opcode, FluxaCore::FCAST_OP_PLAYBACK_UPDATE);
        assert_eq!(
            FluxaCore::fcast_playback_update(&received),
            Some((1, 42.0, 120.0, 1.0))
        );
    }

    #[tokio::test]
    async fn a_bodyless_message_round_trips() {
        let (mut client, mut receiver) = connected_pair().await;
        send(&mut receiver, FluxaCore::FCAST_OP_PING, "")
            .await
            .unwrap();
        assert_eq!(
            read_message(&mut client).await.unwrap(),
            (FluxaCore::FCAST_OP_PING, String::new())
        );
    }

    #[tokio::test]
    async fn an_oversized_size_prefix_is_rejected_before_allocating() {
        let (mut client, mut receiver) = connected_pair().await;
        receiver.write_all(&u32::MAX.to_le_bytes()).await.unwrap();
        assert!(read_message(&mut client).await.is_err());
    }
}
