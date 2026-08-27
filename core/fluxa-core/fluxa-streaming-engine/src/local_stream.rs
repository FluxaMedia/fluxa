#[cfg(not(all(feature = "apple", fluxa_ffmpeg_bridge)))]
use crate::ffmpeg_locator;
use serde_json::json;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
#[cfg(not(all(feature = "apple", fluxa_ffmpeg_bridge)))]
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener as TokioTcpListener;
use tokio::net::TcpStream as TokioTcpStream;
#[cfg(not(all(feature = "apple", fluxa_ffmpeg_bridge)))]
use tokio::process::Command;

const PROXY_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";
const MAX_LOCAL_STREAM_CONNECTIONS: usize = 32;

pub(crate) fn build_proxy_client() -> Arc<reqwest::blocking::Client> {
    static CLIENT: OnceLock<Arc<reqwest::blocking::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            Arc::new(
                reqwest::blocking::Client::builder()
                    .redirect(reqwest::redirect::Policy::limited(10))
                    .connect_timeout(Duration::from_secs(15))
                    .timeout(Duration::from_secs(90))
                    .user_agent(PROXY_USER_AGENT)
                    .build()
                    .expect("proxy client build"),
            )
        })
        .clone()
}

pub(crate) fn build_async_proxy_client() -> Arc<reqwest::Client> {
    static CLIENT: OnceLock<Arc<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            Arc::new(
                reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::limited(10))
                    .connect_timeout(Duration::from_secs(15))
                    .timeout(Duration::from_secs(90))
                    .user_agent(PROXY_USER_AGENT)
                    .build()
                    .expect("proxy client build"),
            )
        })
        .clone()
}

pub(crate) fn local_stream_runtime() -> Result<&'static tokio::runtime::Runtime, String> {
    static RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)
}

#[derive(Clone)]
pub(crate) struct LocalStreamConfig {
    pub(crate) id: String,
    pub(crate) target_url: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) client: Arc<reqwest::blocking::Client>,
    pub(crate) async_client: Arc<reqwest::Client>,
    pub(crate) active_connections: Arc<AtomicUsize>,
    pub(crate) port: u16,
}

pub(crate) struct LocalStreamHandle {
    pub(crate) stop: Arc<AtomicBool>,
    pub(crate) thread: Option<thread::JoinHandle<()>>,
}

pub(crate) struct ParsedLocalRequest {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: HashMap<String, String>,
}

pub(crate) static LOCAL_STREAM_SERVERS: OnceLock<Mutex<HashMap<String, LocalStreamHandle>>> =
    OnceLock::new();
static SHARED_LOCAL_CONFIGS: OnceLock<Mutex<HashMap<String, LocalStreamConfig>>> = OnceLock::new();
static SHARED_LOCAL_SERVER: OnceLock<Result<u16, String>> = OnceLock::new();
pub(crate) static LOCAL_STREAM_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) fn local_stream_servers() -> &'static Mutex<HashMap<String, LocalStreamHandle>> {
    LOCAL_STREAM_SERVERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn shared_local_configs() -> &'static Mutex<HashMap<String, LocalStreamConfig>> {
    SHARED_LOCAL_CONFIGS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn next_local_stream_id() -> String {
    let counter = LOCAL_STREAM_ID.fetch_add(1, Ordering::Relaxed);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    counter.hash(&mut hasher);
    now.hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub(crate) struct ActiveConnectionGuard {
    counter: Arc<AtomicUsize>,
}

impl ActiveConnectionGuard {
    pub(crate) fn try_acquire(counter: Arc<AtomicUsize>) -> Option<Self> {
        let previous = counter.fetch_add(1, Ordering::AcqRel);
        if previous >= MAX_LOCAL_STREAM_CONNECTIONS {
            counter.fetch_sub(1, Ordering::AcqRel);
            return None;
        }
        Some(Self { counter })
    }
}

impl Drop for ActiveConnectionGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(crate) fn parse_request(stream: &mut TcpStream) -> Option<ParsedLocalRequest> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).ok()?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next()?.to_ascii_uppercase();
    let path = request_parts.next()?.to_string();
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            headers.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    Some(ParsedLocalRequest {
        method,
        path,
        headers,
    })
}

pub(crate) fn write_simple_response(stream: &mut TcpStream, status: &str) {
    let body = status.as_bytes();
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(body);
}

pub(crate) fn retryable_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::REQUEST_TIMEOUT
        || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

pub(crate) fn send_upstream_request(
    client: &reqwest::blocking::Client,
    config: &LocalStreamConfig,
    method: &str,
    request_headers: &HashMap<String, String>,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let mut last_error = None;
    for attempt in 0..3 {
        let mut request = if method == "HEAD" {
            client.head(&config.target_url)
        } else {
            client.get(&config.target_url)
        };
        for (key, value) in config.headers.iter() {
            request = request.header(key, value);
        }
        if let Some(range) = request_headers.get("range") {
            request = request.header("Range", range);
        }
        match request.send() {
            Ok(response) if retryable_status(response.status()) && attempt < 2 => {
                thread::sleep(Duration::from_millis(80 * (attempt + 1) as u64));
            }
            Ok(response) => return Ok(response),
            Err(error) if attempt < 2 => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(80 * (attempt + 1) as u64));
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error.expect("retry loop should keep the last error"))
}

pub(crate) async fn parse_async_request(stream: &mut TokioTcpStream) -> Option<ParsedLocalRequest> {
    let mut data = Vec::with_capacity(1024);
    let mut chunk = [0u8; 512];
    loop {
        let count = stream.read(&mut chunk).await.ok()?;
        if count == 0 {
            return None;
        }
        data.extend_from_slice(&chunk[..count]);
        if data.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
        if data.len() > 16 * 1024 {
            return None;
        }
    }
    let text = std::str::from_utf8(&data).ok()?;
    let mut lines = text.split("\r\n");
    let mut request_parts = lines.next()?.split_whitespace();
    let method = request_parts.next()?.to_ascii_uppercase();
    let path = request_parts.next()?.to_string();
    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    Some(ParsedLocalRequest {
        method,
        path,
        headers,
    })
}

async fn send_async_upstream_request(
    client: &reqwest::Client,
    config: &LocalStreamConfig,
    method: &str,
    request_headers: &HashMap<String, String>,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut last_error = None;
    for attempt in 0..3 {
        let mut request = if method == "HEAD" {
            client.head(&config.target_url)
        } else {
            client.get(&config.target_url)
        };
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }
        if let Some(range) = request_headers.get("range") {
            request = request.header("Range", range);
        }
        match request.send().await {
            Ok(response) if retryable_status(response.status()) && attempt < 2 => {
                tokio::time::sleep(Duration::from_millis(80 * (attempt + 1) as u64)).await;
            }
            Ok(response) => return Ok(response),
            Err(error) if attempt < 2 => {
                last_error = Some(error);
                tokio::time::sleep(Duration::from_millis(80 * (attempt + 1) as u64)).await;
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error.expect("retry loop should keep the last error"))
}

async fn handle_async_local_stream(
    mut stream: TokioTcpStream,
    config: LocalStreamConfig,
    request: ParsedLocalRequest,
) {
    let stream_path = format!("/stream/{}", config.id);
    let remux_path = format!("{stream_path}/remux");
    let (request_path, query) = request.path.split_once('?').unwrap_or((&request.path, ""));
    let remux = request_path == remux_path;
    if request_path != stream_path && !remux {
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n")
            .await;
        return;
    };
    let Some(_connection_guard) =
        ActiveConnectionGuard::try_acquire(config.active_connections.clone())
    else {
        let _ = stream
            .write_all(b"HTTP/1.1 503 Service Unavailable\r\nConnection: close\r\n\r\n")
            .await;
        return;
    };
    if request.method != "GET" && request.method != "HEAD" {
        let _ = stream
            .write_all(b"HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\n\r\n")
            .await;
        return;
    }
    if remux {
        handle_ffmpeg_remux(&mut stream, &config, &request, query).await;
        return;
    }
    // Direct streams retain normal upstream byte-range seeks. Adapted streams
    // return above and let FFmpeg apply the requested output-side seek.
    let upstream_headers = request.headers.clone();
    let Ok(mut response) = send_async_upstream_request(
        &config.async_client,
        &config,
        &request.method,
        &upstream_headers,
    )
    .await
    else {
        let _ = stream
            .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\n\r\n")
            .await;
        return;
    };
    let status = response.status();
    let reason = status.canonical_reason().unwrap_or("OK");
    let mut header = format!("HTTP/1.1 {} {}\r\n", status.as_u16(), reason);
    for name in [
        "content-type",
        "content-length",
        "content-range",
        "accept-ranges",
        "etag",
        "last-modified",
    ] {
        if let Some(value) = response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
        {
            header.push_str(name);
            header.push_str(": ");
            header.push_str(value);
            header.push_str("\r\n");
        }
    }
    header.push_str("Connection: close\r\n\r\n");
    if stream.write_all(header.as_bytes()).await.is_err() || request.method == "HEAD" {
        return;
    }
    while let Ok(Some(chunk)) = response.chunk().await {
        if stream.write_all(&chunk).await.is_err() {
            break;
        }
    }
}

#[cfg(all(feature = "apple", fluxa_ffmpeg_bridge))]
async fn handle_ffmpeg_remux(
    stream: &mut TokioTcpStream,
    config: &LocalStreamConfig,
    request: &ParsedLocalRequest,
    query: &str,
) {
    if request.method == "HEAD" {
        let _ = stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n")
            .await;
        return;
    }

    let headers = config
        .headers
        .iter()
        .map(|(name, value)| format!("{name}: {value}\r\n"))
        .collect::<String>();
    let start_microseconds = query
        .split('&')
        .find_map(|item| {
            let (name, value) = item.split_once('=')?;
            (name == "start")
                .then(|| value.parse::<f64>().ok())
                .flatten()
        })
        .filter(|seconds| *seconds > 0.0)
        .map(|seconds| (seconds * 1_000_000.0) as i64)
        .unwrap_or(0);
    let mut output = crate::apple_ffmpeg::start_remux(
        config.target_url.clone(),
        headers,
        start_microseconds,
    );
    let Some(crate::apple_ffmpeg::RemuxMessage::Chunk(first_chunk)) = output.recv().await else {
        let _ = stream
            .write_all(
                b"HTTP/1.1 415 Unsupported Media Type\r\nConnection: close\r\n\r\nsource cannot be adapted to fMP4",
            )
            .await;
        return;
    };
    if stream
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n")
        .await
        .is_err()
    {
        return;
    }
    if stream.write_all(&first_chunk).await.is_err() {
        return;
    }
    while let Some(message) = output.recv().await {
        match message {
            crate::apple_ffmpeg::RemuxMessage::Chunk(chunk) => {
                if stream.write_all(&chunk).await.is_err() {
                    return;
                }
            }
            crate::apple_ffmpeg::RemuxMessage::Finished(_) => return,
        }
    }
}

#[cfg(not(all(feature = "apple", fluxa_ffmpeg_bridge)))]
async fn handle_ffmpeg_remux(
    stream: &mut TokioTcpStream,
    config: &LocalStreamConfig,
    request: &ParsedLocalRequest,
    query: &str,
) {
    if request.method == "HEAD" {
        let _ = stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n")
            .await;
        return;
    }

    let audio_codec = probe_audio_codec(config).await;
    let audio_args: &[&str] = match audio_codec.as_deref() {
        // If probing fails, preserve the elementary stream and let FFmpeg
        // report a real muxing error instead of silently dropping audio.
        None => &["-c:a", "copy"],
        Some("aac" | "ac3" | "eac3" | "mp3" | "alac" | "flac") => &["-c:a", "copy"],
        // Preserve decoded samples without introducing AAC/MP3 loss. Codec
        // object metadata (for example TrueHD Atmos) cannot survive PCM/ALAC.
        Some(_) => &["-c:a", "alac"],
    };
    let mut command = Command::new(ffmpeg_locator::resolve("ffmpeg"));
    command.args(["-hide_banner", "-loglevel", "error"]);
    if !config.headers.is_empty() {
        let headers = config
            .headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}\r\n"))
            .collect::<String>();
        command.args(["-headers", &headers]);
    }
    command.args(["-i", &config.target_url]);
    if let Some(seconds) = query.split('&').find_map(|item| {
        let (name, value) = item.split_once('=')?;
        (name == "start")
            .then(|| value.parse::<f64>().ok())
            .flatten()
    }) {
        command.args(["-ss", &seconds.to_string()]);
    }
    command
        .args(["-map", "0:v:0", "-map", "0:a:0?"])
        .args(["-c:v", "copy"])
        .args(audio_args)
        .args([
            "-map_metadata",
            "0",
            "-sn",
            "-avoid_negative_ts",
            "make_zero",
            "-movflags",
            "frag_keyframe+empty_moov+default_base_moof",
            "-f",
            "mp4",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let Ok(mut child) = command.spawn() else {
        let _ = stream
            .write_all(b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\nffmpeg unavailable")
            .await;
        return;
    };
    let Some(mut stdout) = child.stdout.take() else {
        let _ = stream
            .write_all(b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\n\r\nffmpeg produced no output")
            .await;
        return;
    };
    // Do not send a misleading 200 response when FFmpeg rejects the source
    // or cannot mux its tracks into fMP4. The first stdout read is the
    // adaptation gate for this local Apple endpoint.
    let mut first_chunk = vec![0; 64 * 1024];
    let first_size = match stdout.read(&mut first_chunk).await {
        Ok(0) => {
            let _ = child.wait().await;
            let _ = stream
                .write_all(
                    b"HTTP/1.1 415 Unsupported Media Type\r\nConnection: close\r\n\r\nsource cannot be adapted to fMP4",
                )
                .await;
            return;
        }
        Ok(size) => size,
        Err(_) => {
            let _ = child.kill().await;
            let _ = stream
                .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\n\r\nffmpeg output failed")
                .await;
            return;
        }
    };
    if stream
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n")
        .await
        .is_err()
    {
        let _ = child.kill().await;
        return;
    }
    if stream.write_all(&first_chunk[..first_size]).await.is_err() {
        let _ = child.kill().await;
        return;
    }
    let _ = tokio::io::copy(&mut stdout, stream).await;
    let _ = child.wait().await;
}

async fn probe_audio_codec(config: &LocalStreamConfig) -> Option<String> {
    let mut command = Command::new(ffmpeg_locator::resolve("ffprobe"));
    command.args([
        "-v",
        "error",
        "-select_streams",
        "a:0",
        "-show_entries",
        "stream=codec_name",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
    ]);
    if !config.headers.is_empty() {
        let headers = config
            .headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}\r\n"))
            .collect::<String>();
        command.args(["-headers", &headers]);
    }
    let output = command
        .arg(&config.target_url)
        .stdin(Stdio::null())
        .output()
        .await
        .ok()?;
    String::from_utf8(output.stdout)
        .ok()
        .map(|codec| codec.trim().to_ascii_lowercase())
}

async fn handle_shared_async_stream(stream: TokioTcpStream) {
    let mut stream = stream;
    let Some(request) = parse_async_request(&mut stream).await else {
        return;
    };
    let Some(id) = request
        .path
        .strip_prefix("/stream/")
        .and_then(|path| path.split('/').next())
    else {
        return;
    };
    let Some(config) = shared_local_configs()
        .lock()
        .ok()
        .and_then(|configs| configs.get(id).cloned())
    else {
        return;
    };
    handle_async_local_stream(stream, config, request).await;
}

fn shared_local_port(preferred_port: u16) -> Result<u16, String> {
    SHARED_LOCAL_SERVER
        .get_or_init(|| {
            let listener = TcpListener::bind(("127.0.0.1", preferred_port))
                .map_err(|error| error.to_string())?;
            let port = listener
                .local_addr()
                .map_err(|error| error.to_string())?
                .port();
            listener
                .set_nonblocking(true)
                .map_err(|error| error.to_string())?;
            thread::spawn(move || {
                let Ok(runtime) = local_stream_runtime() else {
                    return;
                };
                runtime.block_on(async move {
                    let Ok(listener) = TokioTcpListener::from_std(listener) else {
                        return;
                    };
                    loop {
                        tokio::select! {
                            accepted = listener.accept() => match accepted {
                                Ok((stream, _)) => {
                            tokio::spawn(async move {
                                handle_shared_async_stream(stream).await;
                            });
                                }
                                Err(_) => break,
                            },
                            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                        }
                    }
                });
            });
            Ok(port)
        })
        .clone()
}

pub fn start_local_stream_server(
    target_url: &str,
    headers_json: &str,
    preferred_port: i32,
) -> Option<String> {
    let headers = serde_json::from_str::<HashMap<String, String>>(headers_json).unwrap_or_default();
    let id = next_local_stream_id();
    let bind_port = preferred_port.clamp(0, u16::MAX as i32) as u16;
    let port = shared_local_port(bind_port).ok()?;
    let config = LocalStreamConfig {
        id: id.clone(),
        target_url: target_url.to_string(),
        headers,
        client: build_proxy_client(),
        async_client: build_async_proxy_client(),
        active_connections: Arc::new(AtomicUsize::new(0)),
        port,
    };
    shared_local_configs()
        .lock()
        .ok()?
        .insert(id.clone(), config);
    serde_json::to_string(&json!({
        "id": id.clone(),
        "url": format!("http://127.0.0.1:{port}/stream/{id}"),
        "port": port
    }))
    .ok()
}

pub fn stop_local_stream_server(id: &str) -> bool {
    if crate::dv_rewrite::remove_shared_dv_config(id) {
        return true;
    }
    if shared_local_configs()
        .lock()
        .ok()
        .and_then(|mut configs| configs.remove(id))
        .is_some()
    {
        return true;
    }
    let Some(mut handle) = local_stream_servers()
        .lock()
        .ok()
        .and_then(|mut servers| servers.remove(id))
    else {
        return false;
    };
    handle.stop.store(true, Ordering::Relaxed);
    if let Some(thread) = handle.thread.take() {
        let _ = thread.join();
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{ActiveConnectionGuard, MAX_LOCAL_STREAM_CONNECTIONS};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn active_connection_guard_caps_and_releases_slots() {
        let counter = Arc::new(AtomicUsize::new(0));
        let guards: Vec<_> = (0..MAX_LOCAL_STREAM_CONNECTIONS)
            .map(|_| ActiveConnectionGuard::try_acquire(counter.clone()).unwrap())
            .collect();

        assert!(ActiveConnectionGuard::try_acquire(counter.clone()).is_none());
        assert_eq!(
            counter.load(Ordering::Acquire),
            MAX_LOCAL_STREAM_CONNECTIONS
        );

        drop(guards);
        assert_eq!(counter.load(Ordering::Acquire), 0);
        assert!(ActiveConnectionGuard::try_acquire(counter.clone()).is_some());
    }
}
