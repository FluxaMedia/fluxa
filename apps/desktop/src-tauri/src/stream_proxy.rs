use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const FORWARDED_HEADER_BLOCKLIST: [&str; 3] = ["host", "content-length", "connection"];
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(20);
const DIAGNOSTIC_BODY_SNIPPET_LEN: usize = 500;

#[derive(Clone)]
struct ProxyTarget {
    url: String,
    headers: Vec<(String, String)>,
}

#[derive(Default)]
pub struct StreamProxyState {
    targets: Arc<Mutex<HashMap<String, ProxyTarget>>>,
    port: Mutex<Option<u16>>,
    last_failure: Arc<Mutex<Option<(String, String)>>>,
}

pub fn take_last_failure(state: &StreamProxyState) -> Option<(String, String)> {
    state.last_failure.lock().unwrap().take()
}

fn random_token() -> String {
    let a = RandomState::new().build_hasher().finish();
    let b = RandomState::new().build_hasher().finish();
    format!("{a:016x}{b:016x}")
}

async fn ensure_server(state: &StreamProxyState) -> Result<u16, String> {
    if let Some(port) = *state.port.lock().unwrap() {
        return Ok(port);
    }
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let targets = state.targets.clone();
    let last_failure = state.last_failure.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                break;
            };
            tauri::async_runtime::spawn(handle_conn(socket, targets.clone(), last_failure.clone()));
        }
    });
    *state.port.lock().unwrap() = Some(port);
    Ok(port)
}

async fn read_request(socket: &mut TcpStream) -> Option<(String, String, Option<String>)> {
    let mut received = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) | Err(_) => return None,
            Ok(n) => n,
        };
        received.extend_from_slice(&buf[..n]);
        if received.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if received.len() > 16384 {
            return None;
        }
    }

    let request_text = String::from_utf8_lossy(&received);
    let mut lines = request_text.lines();
    let request_line = lines.next().unwrap_or("");
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or("GET").to_string();
    let path = request_parts.next().unwrap_or("").to_string();
    if path.is_empty() {
        return None;
    }
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
    Some((method, path, range_header))
}

async fn write_status_only(socket: &mut TcpStream, status_line: &[u8]) {
    let _ = socket.write_all(status_line).await;
}

fn report_upstream_failure(
    last_failure: &Mutex<Option<(String, String)>>,
    url: &str,
    status: Option<u16>,
    detail: &str,
    body_snippet: &str,
) {
    log::warn!("[stream_proxy] upstream failure url={url} status={status:?} detail={detail} body={body_snippet}");
    let shown = match status {
        Some(status) if !body_snippet.trim().is_empty() => {
            format!("HTTP {status}: {}", body_snippet.trim())
        }
        Some(status) => format!("HTTP {status}: {detail}"),
        None => detail.to_string(),
    };
    *last_failure.lock().unwrap() = Some((url.to_string(), shown));
}

async fn handle_conn(
    mut socket: TcpStream,
    targets: Arc<Mutex<HashMap<String, ProxyTarget>>>,
    last_failure: Arc<Mutex<Option<(String, String)>>>,
) {
    loop {
        let Some((method, path, range_header)) = read_request(&mut socket).await else {
            return;
        };

        let Some(token) = path.strip_prefix("/stream/").map(str::to_string) else {
            write_status_only(&mut socket, b"HTTP/1.1 404 Not Found\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n").await;
            continue;
        };
        let Some(target) = targets.lock().unwrap().get(&token).cloned() else {
            write_status_only(&mut socket, b"HTTP/1.1 404 Not Found\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n").await;
            continue;
        };

        let client = match crate::net_guard::vetted_client(&target.url, UPSTREAM_TIMEOUT).await {
            Ok(client) => client,
            Err(err) => {
                report_upstream_failure(&last_failure, &target.url, None, &format!("vetted_client failed: {err}"), "");
                write_status_only(&mut socket, b"HTTP/1.1 502 Bad Gateway\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n").await;
                continue;
            }
        };

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

        let response = match req.send().await {
            Ok(response) => response,
            Err(err) => {
                report_upstream_failure(&last_failure, &target.url, None, &format!("request failed: {err}"), "");
                write_status_only(&mut socket, b"HTTP/1.1 502 Bad Gateway\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n").await;
                continue;
            }
        };

        let status = response.status();
        if !status.is_success() && status.as_u16() != 206 {
            let body = response
                .text()
                .await
                .unwrap_or_default()
                .chars()
                .take(DIAGNOSTIC_BODY_SNIPPET_LEN)
                .collect::<String>();
            report_upstream_failure(&last_failure, &target.url, Some(status.as_u16()), "non-success status", &body);
            let status_line = format!(
                "HTTP/1.1 {} {}\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n",
                status.as_u16(),
                status.canonical_reason().unwrap_or("")
            );
            write_status_only(&mut socket, status_line.as_bytes()).await;
            continue;
        }

        let mut header_buf = format!(
            "HTTP/1.1 {} {}\r\n",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        );
        header_buf.push_str("Connection: keep-alive\r\nAccept-Ranges: bytes\r\n");
        for name in ["content-type", "content-length", "content-range"] {
            if let Some(value) = response.headers().get(name).and_then(|v| v.to_str().ok()) {
                header_buf.push_str(&format!("{name}: {value}\r\n"));
            }
        }
        header_buf.push_str("\r\n");
        if socket.write_all(header_buf.as_bytes()).await.is_err() {
            return;
        }
        if method == "HEAD" {
            continue;
        }

        let mut response = response;
        let mut sent = 0u64;
        loop {
            match response.chunk().await {
                Ok(Some(chunk)) => {
                    sent += chunk.len() as u64;
                    if socket.write_all(&chunk).await.is_err() {
                        return;
                    }
                }
                Ok(None) => break,
                Err(err) => {
                    report_upstream_failure(
                        &last_failure,
                        &target.url,
                        Some(status.as_u16()),
                        &format!("body read failed after {sent} bytes: {err}"),
                        "",
                    );
                    return;
                }
            }
        }
    }
}

pub async fn register(
    state: &StreamProxyState,
    url: String,
    headers: Vec<(String, String)>,
) -> Result<String, String> {
    let port = ensure_server(state).await?;
    let token = random_token();
    state
        .targets
        .lock()
        .unwrap()
        .insert(token.clone(), ProxyTarget { url, headers });
    Ok(format!("http://127.0.0.1:{port}/stream/{token}"))
}
