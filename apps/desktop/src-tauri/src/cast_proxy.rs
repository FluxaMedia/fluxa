use crate::cast::lan_ip;
use fluxa_core::FluxaCore;
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const FORWARDED_HEADER_BLOCKLIST: [&str; 3] = ["host", "content-length", "connection"];

#[derive(Clone)]
struct ProxyTarget {
    url: String,
    headers: HashMap<String, String>,
    token: String,
}

#[derive(Default)]
pub struct CastProxyState {
    target: Arc<Mutex<Option<ProxyTarget>>>,
    port: Mutex<Option<u16>>,
}

fn random_token() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

async fn ensure_server(state: &CastProxyState) -> Result<u16, String> {
    if let Some(port) = *state.port.lock().unwrap() {
        return Ok(port);
    }
    let listener = TcpListener::bind("0.0.0.0:0")
        .await
        .map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let target = state.target.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                break;
            };
            tauri::async_runtime::spawn(handle_conn(socket, target.clone()));
        }
    });
    *state.port.lock().unwrap() = Some(port);
    Ok(port)
}

async fn handle_conn(mut socket: TcpStream, target: Arc<Mutex<Option<ProxyTarget>>>) {
    let mut received = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) | Err(_) => return,
            Ok(n) => n,
        };
        received.extend_from_slice(&buf[..n]);
        if received.windows(4).any(|w| w == b"\r\n\r\n") || received.len() > 16384 {
            break;
        }
    }

    let request_text = String::from_utf8_lossy(&received);
    let mut lines = request_text.lines();
    let request_line = lines.next().unwrap_or("");
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or("GET").to_string();
    let path = request_parts.next().unwrap_or("").to_string();
    let mut range_header: Option<String> = None;
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            if key.trim().eq_ignore_ascii_case("range") {
                range_header = Some(value.trim().to_string());
            }
        }
    }

    let Some(target) = target.lock().unwrap().clone() else {
        let _ = socket
            .write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n")
            .await;
        return;
    };

    if path != format!("/stream/{}", target.token) {
        let _ = socket
            .write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n")
            .await;
        return;
    }

    let client = reqwest::Client::new();
    let reqwest_method = if method == "HEAD" {
        reqwest::Method::HEAD
    } else {
        reqwest::Method::GET
    };
    let mut req = client.request(reqwest_method, &target.url);
    for (key, value) in &target.headers {
        if FORWARDED_HEADER_BLOCKLIST.contains(&key.to_ascii_lowercase().as_str()) {
            continue;
        }
        req = req.header(key.as_str(), value.as_str());
    }
    if let Some(range) = &range_header {
        req = req.header("Range", range.as_str());
    }

    let Ok(resp) = req.send().await else {
        let _ = socket
            .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\n\r\n")
            .await;
        return;
    };

    let status = resp.status();
    let mut header_buf = format!(
        "HTTP/1.1 {} {}\r\n",
        status.as_u16(),
        status.canonical_reason().unwrap_or("")
    );
    header_buf.push_str("Connection: close\r\nAccept-Ranges: bytes\r\n");
    for name in ["content-type", "content-length", "content-range"] {
        if let Some(value) = resp.headers().get(name).and_then(|v| v.to_str().ok()) {
            header_buf.push_str(&format!("{name}: {value}\r\n"));
        }
    }
    header_buf.push_str("\r\n");
    if socket.write_all(header_buf.as_bytes()).await.is_err() || method == "HEAD" {
        return;
    }

    let mut resp = resp;
    loop {
        match resp.chunk().await {
            Ok(Some(chunk)) => {
                if socket.write_all(&chunk).await.is_err() {
                    break;
                }
            }
            _ => break,
        }
    }
}

#[tauri::command]
pub async fn cast_proxy_serve(
    state: tauri::State<'_, CastProxyState>,
    url: String,
    headers: HashMap<String, String>,
) -> Result<String, String> {
    if !FluxaCore::validate_stream_url(&url) {
        return Err("unsupported media url".to_string());
    }
    let port = ensure_server(&state).await?;
    let token = random_token();
    *state.target.lock().unwrap() = Some(ProxyTarget {
        url,
        headers,
        token: token.clone(),
    });
    let ip = lan_ip().ok_or_else(|| "could not determine LAN IP".to_string())?;
    Ok(format!("http://{ip}:{port}/stream/{token}"))
}
