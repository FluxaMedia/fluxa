use serde_json::{json, Value};

pub(crate) fn request(url: &str, body: Option<&str>, read_timeout: std::time::Duration) -> Option<String> {
    use std::net::ToSocketAddrs;
    let rest = url.strip_prefix("http://")?;
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let (host, port) = authority
        .split_once(':')
        .and_then(|(host, port)| port.parse::<u16>().ok().map(|port| (host, port)))
        .unwrap_or((authority, 80));
    let path = format!("/{path}");
    let addr = (host, port).to_socket_addrs().ok()?.next()?;
    let mut stream = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)).ok()?;
    stream.set_read_timeout(Some(read_timeout)).ok()?;
    let request = match body {
        Some(body) => format!("POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()),
        None => format!("GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n"),
    };
    std::io::Write::write_all(&mut stream, request.as_bytes()).ok()?;
    let mut response = Vec::new();
    let _ = std::io::Read::read_to_end(&mut stream, &mut response);
    Some(String::from_utf8_lossy(&response).into_owned())
}

pub(crate) fn healthy(base_url: &str) -> bool {
    request(&format!("{}/health", base_url.trim_end_matches('/')), None, std::time::Duration::from_secs(2))
        .is_some_and(|response| response.starts_with("HTTP/1.1 200"))
}

pub(crate) fn add(base_url: &str, link: &str, file_id: Option<usize>) {
    post_async(base_url, "torrents", json!({ "action": "add", "link": link, "file_id": file_id }).to_string(), 60);
}

pub(crate) fn remove(base_url: &str, link: &str) {
    post_async(base_url, "torrents", json!({ "action": "rem", "link": link }).to_string(), 15);
}

pub(crate) fn deactivate(base_url: &str, link: &str) {
    post_async(base_url, "torrents", json!({ "action": "deactivate", "link": link }).to_string(), 15);
}

pub(crate) fn apply_preferences(base_url: &str, preferences: Option<&Value>) {
    let preload_size = match preferences.and_then(|value| value.get("torrentSpeedPreset")).and_then(Value::as_str) {
        Some("fast") => 32,
        Some("ultra_fast") => 64,
        _ => 16,
    };
    let cache_limit_mb = preferences
        .and_then(|value| value.get("torrentCacheLimitMb"))
        .and_then(Value::as_u64);
    post_async(base_url, "settings", json!({ "PreloadSize": preload_size, "CacheLimitMb": cache_limit_mb }).to_string(), 5);
}

fn post_async(base_url: &str, endpoint: &str, body: String, timeout_seconds: u64) {
    let url = format!("{}/{}", base_url.trim_end_matches('/'), endpoint);
    std::thread::spawn(move || { request(&url, Some(&body), std::time::Duration::from_secs(timeout_seconds)); });
}
