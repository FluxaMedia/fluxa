use base64::Engine;
use serde::Serialize;
#[cfg(unix)]
use tauri::Emitter;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::path::Path;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::io::{BufRead, Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::io::BufReader;
use std::net::TcpStream;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalPlayerOption {
    pub id: String,
    pub label: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalPlayerLaunch {
    pub session_id: String,
    pub tracking: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalPlayerStatus {
    pub active: bool,
    pub paused: Option<bool>,
    pub time_pos: Option<f64>,
    pub duration: Option<f64>,
}

#[derive(Clone)]
enum SessionTransport {
    Mpv { socket: PathBuf },
    Vlc { port: u16, password: String },
    #[cfg(target_os = "macos")]
    Infuse,
    Passive,
}

#[derive(Clone)]
struct ExternalSession {
    transport: SessionTransport,
    created_ms: u128,
    child: Option<Arc<Mutex<Child>>>,
    #[cfg(unix)]
    status_cache: Option<Arc<Mutex<ExternalPlayerStatus>>>,
}

fn child_has_exited(child: &Arc<Mutex<Child>>) -> bool {
    matches!(child.lock().unwrap().try_wait(), Ok(Some(_)))
}

static SESSIONS: LazyLock<Mutex<HashMap<String, ExternalSession>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}

fn new_session_id() -> String {
    format!("external-{}-{}", now_ms(), NEXT_SESSION.fetch_add(1, Ordering::Relaxed))
}

fn command_path(command: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    env::split_paths(&paths)
        .map(|directory| directory.join(command))
        .find(|path| path.is_file())
}

#[cfg(target_os = "linux")]
fn linux_desktop_players() -> Vec<ExternalPlayerOption> {
    let mut directories = vec![PathBuf::from("/usr/share/applications"), PathBuf::from("/usr/local/share/applications")];
    if let Some(home) = env::var_os("HOME") { directories.push(PathBuf::from(home).join(".local/share/applications")); }
    let mut players = Vec::new();
    for directory in directories {
        let Ok(entries) = std::fs::read_dir(directory) else { continue; };
        for entry in entries.flatten() {
            let Ok(text) = std::fs::read_to_string(entry.path()) else { continue; };
            let media_player = text.lines().any(|line| line.starts_with("Categories=") && line.split(';').any(|category| category == "Player"));
            if !media_player { continue; }
            let label = text.lines().find_map(|line| line.strip_prefix("Name=")).unwrap_or("").trim();
            let executable = text.lines().find_map(|line| line.strip_prefix("Exec=")).and_then(|value| value.split_whitespace().next()).map(|value| value.trim_matches('"')).and_then(command_path);
            let Some(executable) = executable else { continue; };
            if label.is_empty() { continue; }
            players.push(ExternalPlayerOption { id: executable.to_string_lossy().to_string(), label: label.to_string() });
        }
    }
    players
}

#[tauri::command]
pub fn external_player_options() -> Vec<ExternalPlayerOption> {
    let mut options = Vec::new();
    #[cfg(target_os = "macos")]
    if Path::new("/Applications/Infuse.app").exists() {
        options.push(ExternalPlayerOption { id: "infuse".into(), label: "Infuse".into() });
    }
    #[cfg(target_os = "linux")]
    options.extend(linux_desktop_players());
    options.push(ExternalPlayerOption { id: "system".into(), label: "system".into() });
    options.sort_by(|left, right| left.id.cmp(&right.id));
    options.dedup_by(|left, right| left.id == right.id);
    options.sort_by(|left, right| left.label.to_lowercase().cmp(&right.label.to_lowercase()));
    options
}

fn open_system(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut command = { let mut command = Command::new("cmd"); command.args(["/C", "start", "", url]); command };
    #[cfg(target_os = "macos")]
    let mut command = { let mut command = Command::new("open"); command.arg(url); command };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = { let mut command = Command::new("xdg-open"); command.arg(url); command };
    command.spawn().map(|_| ()).map_err(|error| error.to_string())
}

fn choose_vlc_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    listener.local_addr().map(|address| address.port()).map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn percent_encode(value: &str) -> String {
    value.bytes().map(|byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            (byte as char).to_string()
        } else {
            format!("%{byte:02X}")
        }
    }).collect()
}

fn session_is_starting(session: &ExternalSession) -> bool {
    now_ms().saturating_sub(session.created_ms) < 12_000
}

#[tauri::command]
pub fn external_player_launch(
    app: tauri::AppHandle,
    target: String,
    url: String,
    title: Option<String>,
    resume_seconds: Option<f64>,
    subtitle_url: Option<String>,
    request_headers: Option<HashMap<String, String>>,
) -> Result<ExternalPlayerLaunch, String> {
    if url.trim().is_empty() { return Err("missing playback URL".into()); }
    let session_id = new_session_id();
    let target = target.trim();
    let created_ms = now_ms();
    if target.is_empty() || target == "system" {
        open_system(&url)?;
        SESSIONS.lock().unwrap().insert(session_id.clone(), ExternalSession { transport: SessionTransport::Passive, created_ms, child: None, #[cfg(unix)] status_cache: None });
        return Ok(ExternalPlayerLaunch { session_id, tracking: "passive".into() });
    }
    #[cfg(target_os = "macos")]
    if target == "infuse" {
        let mut link = format!("infuse://x-callback-url/play?url={}", percent_encode(&url));
        if let Some(value) = title.as_deref().filter(|value| !value.trim().is_empty()) { link.push_str(&format!("&filename={}", percent_encode(value))); }
        if let Some(value) = subtitle_url.as_deref().filter(|value| !value.trim().is_empty()) { link.push_str(&format!("&sub={}", percent_encode(value))); }
        if let Some(seconds) = resume_seconds.filter(|value| *value > 0.0) { link.push_str(&format!("&position={}", seconds.floor())); }
        link.push_str(&format!("&x-success={}", percent_encode(&format!("fluxa://external-player?session={session_id}"))));
        link.push_str(&format!("&x-error={}", percent_encode(&format!("fluxa://external-player-error?session={session_id}"))));
        Command::new("open").arg(link).spawn().map_err(|error| error.to_string())?;
        SESSIONS.lock().unwrap().insert(session_id.clone(), ExternalSession { transport: SessionTransport::Infuse, created_ms, child: None, status_cache: None });
        return Ok(ExternalPlayerLaunch { session_id, tracking: "callback".into() });
    }
    #[cfg(not(target_os = "macos"))]
    if target == "infuse" { return Err("Infuse is available only on macOS and iOS".into()); }
    let executable = match target {
        "mpv" | "vlc" => command_path(target).ok_or_else(|| format!("{target} is not installed"))?,
        custom => PathBuf::from(custom),
    };
    if !executable.is_file() && !matches!(target, "mpv" | "vlc") { return Err("selected player executable does not exist".into()); }
    let player_kind = if target == "mpv" || executable.file_name().is_some_and(|name| name == "mpv") {
        "mpv"
    } else if target == "vlc" || executable.file_name().is_some_and(|name| name == "vlc") {
        "vlc"
    } else {
        "other"
    };
    let mut command = Command::new(executable);
    let transport = if player_kind == "mpv" {
        #[cfg(unix)]
        {
            let socket = env::temp_dir().join(format!("fluxa-mpv-{session_id}.sock"));
            let _ = std::fs::remove_file(&socket);
            command.arg(format!("--input-ipc-server={}", socket.to_string_lossy()));
            SessionTransport::Mpv { socket }
        }
        #[cfg(not(unix))]
        { SessionTransport::Passive }
    } else if player_kind == "vlc" {
        let port = choose_vlc_port()?;
        let password = format!("fluxa-{}", NEXT_SESSION.fetch_add(1, Ordering::Relaxed));
        command.args(["--extraintf=http", "--http-host=127.0.0.1", &format!("--http-port={port}"), &format!("--http-password={password}")]);
        SessionTransport::Vlc { port, password }
    } else {
        SessionTransport::Passive
    };
    if player_kind == "mpv" {
        if let Some(value) = title.as_deref().filter(|value| !value.trim().is_empty()) { command.arg(format!("--force-media-title={value}")); }
        if let Some(seconds) = resume_seconds.filter(|value| *value > 0.0) { command.arg(format!("--start={seconds}")); }
        if let Some(subtitle) = subtitle_url.as_deref().filter(|value| !value.trim().is_empty()) { command.arg(format!("--sub-file={subtitle}")); }
        if let Some(headers) = request_headers.filter(|value| !value.is_empty()) {
            let fields = headers.into_iter().map(|(name, value)| format!("{name}: {value}")).collect::<Vec<_>>().join(",");
            command.arg(format!("--http-header-fields={fields}"));
        }
    }
    if player_kind == "vlc" {
        if let Some(value) = title.as_deref().filter(|value| !value.trim().is_empty()) { command.arg(format!("--meta-title={value}")); }
        if let Some(seconds) = resume_seconds.filter(|value| *value > 0.0) { command.arg(format!("--start-time={seconds}")); }
        if let Some(subtitle) = subtitle_url.as_deref().filter(|value| !value.trim().is_empty()) { command.arg(format!("--sub-file={subtitle}")); }
    }
    let child = command.arg(url).spawn().map_err(|error| error.to_string())?;
    let tracking = match &transport { SessionTransport::Mpv { .. } | SessionTransport::Vlc { .. } => "live", _ => "passive" };
    log::debug!("[external_player] launched session={session_id} kind={player_kind} tracking={tracking} pid={}", child.id());
    #[cfg(unix)]
    let status_cache = if let SessionTransport::Mpv { socket } = &transport {
        let cache = Arc::new(Mutex::new(ExternalPlayerStatus { active: true, paused: None, time_pos: None, duration: None }));
        spawn_mpv_watcher(app, session_id.clone(), socket.clone(), cache.clone());
        Some(cache)
    } else {
        None
    };
    SESSIONS.lock().unwrap().insert(session_id.clone(), ExternalSession {
        transport,
        created_ms,
        child: Some(Arc::new(Mutex::new(child))),
        #[cfg(unix)]
        status_cache,
    });
    Ok(ExternalPlayerLaunch { session_id, tracking: tracking.into() })
}

#[cfg(unix)]
fn spawn_mpv_watcher(app: tauri::AppHandle, session_id: String, socket: PathBuf, cache: Arc<Mutex<ExternalPlayerStatus>>) {
    std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        let mut stream = loop {
            if !SESSIONS.lock().map(|s| s.contains_key(&session_id)).unwrap_or(false) {
                return;
            }
            match UnixStream::connect(&socket) {
                Ok(stream) => break stream,
                Err(_) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(250));
                }
                Err(error) => {
                    log::debug!("[external_player] mpv watcher gave up connecting session={session_id}: {error}");
                    return;
                }
            }
        };
        if stream.set_read_timeout(Some(Duration::from_secs(2))).is_err() {
            return;
        }
        let observe = b"{\"command\":[\"observe_property\",1,\"pause\"],\"request_id\":1}\n{\"command\":[\"observe_property\",2,\"time-pos\"],\"request_id\":2}\n{\"command\":[\"observe_property\",3,\"duration\"],\"request_id\":3}\n";
        if stream.write_all(observe).is_err() {
            return;
        }
        let mut reader = BufReader::new(stream);
        let mut last_paused: Option<bool> = None;
        loop {
            if !SESSIONS.lock().map(|s| s.contains_key(&session_id)).unwrap_or(false) {
                return;
            }
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    log::debug!("[external_player] mpv watcher socket closed session={session_id}");
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock || error.kind() == std::io::ErrorKind::TimedOut => {
                    continue;
                }
                Err(error) => {
                    log::debug!("[external_player] mpv watcher read error session={session_id}: {error}");
                    return;
                }
                Ok(_) => {}
            }
            let Ok(value) = serde_json::from_str::<Value>(&line) else { continue; };
            let is_property_event = value.get("event").and_then(Value::as_str) == Some("property-change");
            let name = value.get("name").and_then(Value::as_str);
            if !is_property_event && value.get("request_id").is_none() {
                continue;
            }
            let Ok(mut status) = cache.lock() else { continue; };
            match name.or_else(|| match value.get("id").and_then(Value::as_u64) {
                Some(1) => Some("pause"),
                Some(2) => Some("time-pos"),
                Some(3) => Some("duration"),
                _ => None,
            }) {
                Some("pause") => status.paused = value.get("data").and_then(Value::as_bool),
                Some("time-pos") => status.time_pos = value.get("data").and_then(Value::as_f64),
                Some("duration") => status.duration = value.get("data").and_then(Value::as_f64),
                _ => continue,
            }
            let snapshot = status.clone();
            drop(status);
            if is_property_event && name == Some("pause") && snapshot.paused != last_paused {
                last_paused = snapshot.paused;
                let _ = app.emit("external-player-status", serde_json::json!({
                    "sessionId": session_id,
                    "active": snapshot.active,
                    "paused": snapshot.paused,
                    "timePos": snapshot.time_pos,
                    "duration": snapshot.duration,
                }));
            }
        }
    });
}

fn xml_number(text: &str, name: &str) -> Option<f64> {
    let value = text.split_once(&format!("<{name}>"))?.1.split_once(&format!("</{name}>"))?.0;
    value.trim().parse().ok()
}

fn vlc_status(port: u16, password: &str) -> Option<ExternalPlayerStatus> {
    let mut stream = TcpStream::connect_timeout(&format!("127.0.0.1:{port}").parse().ok()?, Duration::from_millis(400)).ok()?;
    stream.set_read_timeout(Some(Duration::from_millis(500))).ok()?;
    let authorization = base64::engine::general_purpose::STANDARD.encode(format!(":{password}"));
    write!(stream, "GET /requests/status.xml HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Basic {authorization}\r\nConnection: close\r\n\r\n").ok()?;
    let mut response = String::new();
    BufReader::new(stream).read_to_string(&mut response).ok()?;
    if !response.contains(" 200 ") { return None; }
    let paused = response.split_once("<state>")?.1.split_once("</state>")?.0.trim() != "playing";
    Some(ExternalPlayerStatus { active: true, paused: Some(paused), time_pos: xml_number(&response, "time"), duration: xml_number(&response, "length") })
}

#[tauri::command]
pub fn external_player_stop(session_id: String) -> bool {
    let Some(session) = SESSIONS.lock().ok().and_then(|mut sessions| sessions.remove(&session_id)) else {
        log::debug!("[external_player] stop requested for unknown session={session_id}");
        return false;
    };
    let Some(child) = &session.child else {
        log::debug!("[external_player] stop requested for session={session_id} with no tracked process");
        return false;
    };
    let mut child = child.lock().unwrap();
    if child.try_wait().ok().flatten().is_some() {
        log::debug!("[external_player] stop requested for session={session_id}, process already exited");
        return true;
    }
    match child.kill() {
        Ok(()) => {
            log::debug!("[external_player] killed session={session_id}");
            true
        }
        Err(error) => {
            log::debug!("[external_player] failed to kill session={session_id}: {error}");
            false
        }
    }
}

#[tauri::command]
pub fn external_player_status(session_id: String) -> Option<ExternalPlayerStatus> {
    let Some(session) = SESSIONS.lock().ok()?.get(&session_id).cloned() else {
        log::debug!("[external_player] status requested for unknown session={session_id}");
        return None;
    };
    if session.child.as_ref().is_some_and(child_has_exited) {
        log::debug!("[external_player] session={session_id} process has exited, ending session");
        SESSIONS.lock().ok()?.remove(&session_id);
        return Some(ExternalPlayerStatus { active: false, paused: None, time_pos: None, duration: None });
    }
    let status = match &session.transport {
        #[cfg(unix)]
        SessionTransport::Mpv { .. } => session.status_cache.as_ref().and_then(|cache| cache.lock().ok().map(|status| status.clone())),
        #[cfg(not(unix))]
        SessionTransport::Mpv { .. } => None,
        SessionTransport::Vlc { port, password } => vlc_status(*port, password).or_else(|| session_is_starting(&session).then_some(ExternalPlayerStatus { active: true, paused: None, time_pos: None, duration: None })),
        #[cfg(target_os = "macos")]
        SessionTransport::Infuse => None,
        SessionTransport::Passive => None,
    };
    if status.is_none() {
        log::debug!("[external_player] status session={session_id} -> none (starting={})", session_is_starting(&session));
    }
    status
}
