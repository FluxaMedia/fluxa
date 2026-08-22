use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone)]
struct ProxyTarget {
    url: String,
}

#[derive(Default)]
pub struct TrailerProxyState {
    targets: Arc<Mutex<HashMap<String, ProxyTarget>>>,
    port: Mutex<Option<u16>>,
    client: reqwest::Client,
}

fn random_token() -> String {
    let a = RandomState::new().build_hasher().finish();
    let b = RandomState::new().build_hasher().finish();
    format!("{a:016x}{b:016x}")
}

async fn ensure_server(state: &TrailerProxyState) -> Result<u16, String> {
    if let Some(port) = *state.port.lock().unwrap() {
        return Ok(port);
    }
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let targets = state.targets.clone();
    let client = state.client.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                break;
            };
            tauri::async_runtime::spawn(handle_conn(socket, targets.clone(), client.clone()));
        }
    });
    *state.port.lock().unwrap() = Some(port);
    Ok(port)
}

fn content_length_hint(url: &str) -> Option<u64> {
    reqwest::Url::parse(url)
        .ok()?
        .query_pairs()
        .find(|(key, _)| key == "clen")
        .and_then(|(_, value)| value.parse::<u64>().ok())
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

async fn handle_conn(
    mut socket: TcpStream,
    targets: Arc<Mutex<HashMap<String, ProxyTarget>>>,
    client: reqwest::Client,
) {
    loop {
        let Some((method, path, range_header)) = read_request(&mut socket).await else {
            return;
        };

        let Some(token) = path.strip_prefix("/trailer/").map(str::to_string) else {
            let _ = socket
                .write_all(b"HTTP/1.1 404 Not Found\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n")
                .await;
            continue;
        };
        let Some(target) = targets.lock().unwrap().get(&token).cloned() else {
            let _ = socket
                .write_all(b"HTTP/1.1 404 Not Found\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n")
                .await;
            continue;
        };

        if method == "HEAD" {
            let total = content_length_hint(&target.url);
            let mut header_buf = String::from(
                "HTTP/1.1 200 OK\r\nConnection: keep-alive\r\nAccept-Ranges: bytes\r\n",
            );
            if let Some(total) = total {
                header_buf.push_str(&format!("content-length: {total}\r\n"));
            }
            header_buf.push_str("\r\n");
            if socket.write_all(header_buf.as_bytes()).await.is_err() {
                return;
            }
            continue;
        }

        let upstream_range = match &range_header {
            Some(value) => value.clone(),
            None => match content_length_hint(&target.url) {
                Some(total) => format!("bytes=0-{}", total.saturating_sub(1)),
                None => "bytes=0-".to_string(),
            },
        };

        let response = client
            .get(&target.url)
            .header("Range", &upstream_range)
            .send()
            .await;
        let mut response = match response {
            Ok(response) if response.status().is_success() => response,
            Ok(response) => {
                log::warn!(
                    "[trailer_proxy] upstream returned {} for url={}",
                    response.status(),
                    target.url
                );
                let _ = socket
                    .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n")
                    .await;
                continue;
            }
            Err(err) => {
                log::warn!(
                    "[trailer_proxy] upstream request failed url={}: {err}",
                    target.url
                );
                let _ = socket
                    .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n")
                    .await;
                continue;
            }
        };

        let status = response.status();
        let mut header_buf = format!(
            "HTTP/1.1 {} {}\r\n",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        );
        header_buf.push_str("Connection: keep-alive\r\nAccept-Ranges: bytes\r\n");
        if let Some(value) = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
        {
            header_buf.push_str(&format!("content-type: {value}\r\n"));
        }
        if let Some(value) = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
        {
            header_buf.push_str(&format!("content-length: {value}\r\n"));
        }
        if let Some(value) = response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
        {
            header_buf.push_str(&format!("content-range: {value}\r\n"));
        }
        header_buf.push_str("\r\n");
        if socket.write_all(header_buf.as_bytes()).await.is_err() {
            return;
        }

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
                    log::warn!(
                        "[trailer_proxy] upstream body read failed after {sent} bytes: {err}"
                    );
                    return;
                }
            }
        }
    }
}

pub async fn register(state: &TrailerProxyState, url: String) -> Result<String, String> {
    let port = ensure_server(state).await?;
    let token = random_token();
    state
        .targets
        .lock()
        .unwrap()
        .insert(token.clone(), ProxyTarget { url });
    Ok(format!("http://127.0.0.1:{port}/trailer/{token}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_clen_query_param_as_content_length_hint() {
        let url = "https://googlevideo.com/videoplayback?itag=140&clen=2085268&mime=audio%2Fmp4";
        assert_eq!(content_length_hint(url), Some(2_085_268));
    }

    #[test]
    fn missing_clen_param_yields_no_hint() {
        let url = "https://googlevideo.com/videoplayback?itag=140";
        assert_eq!(content_length_hint(url), None);
    }
}
