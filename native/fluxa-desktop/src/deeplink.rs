use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

const PREFIX: &str = "fluxa://oauth/";

static PENDING: Mutex<Vec<String>> = Mutex::new(Vec::new());
static WINDOW: OnceLock<Arc<winit::window::Window>> = OnceLock::new();

pub enum Role {
    Primary,
    Secondary,
    Forwarded,
}

pub fn start() -> Role {
    let url = std::env::args().skip(1).find(|arg| arg.starts_with(PREFIX));
    let Some(path) = port_file() else {
        return Role::Primary;
    };
    if let Some(mut stream) = connect(&path) {
        return match url {
            Some(url) if writeln!(stream, "{url}").is_ok() => Role::Forwarded,
            Some(_) => Role::Primary,
            None => Role::Secondary,
        };
    }
    let Ok(listener) = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)) else {
        return Role::Primary;
    };
    let Ok(address) = listener.local_addr() else {
        return Role::Primary;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, address.port().to_string());
    std::thread::spawn(move || accept(listener));
    Role::Primary
}

pub fn attach(window: Arc<winit::window::Window>) {
    let _ = WINDOW.set(window);
}

pub fn take() -> Vec<String> {
    std::mem::take(
        &mut *PENDING
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
    )
}

fn accept(listener: TcpListener) {
    for stream in listener.incoming().flatten() {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let mut line = String::new();
        if BufReader::new(stream).read_line(&mut line).is_err() {
            continue;
        }
        let url = line.trim();
        if url.starts_with(PREFIX) {
            PENDING
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(url.to_owned());
            if let Some(window) = WINDOW.get() {
                window.request_redraw();
            }
        }
    }
}

fn connect(path: &PathBuf) -> Option<TcpStream> {
    let port: u16 = std::fs::read_to_string(path).ok()?.trim().parse().ok()?;
    TcpStream::connect_timeout(
        &(Ipv4Addr::LOCALHOST, port).into(),
        Duration::from_millis(300),
    )
    .ok()
}

fn port_file() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
    #[cfg(target_os = "macos")]
    let base = PathBuf::from(std::env::var_os("HOME")?).join("Library/Caches");
    #[cfg(target_os = "linux")]
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))?;
    Some(base.join("fluxa").join("deeplink-port"))
}

pub fn open_external(url: &str) {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return;
    }
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = std::process::Command::new("rundll32");
        command.args(["url.dll,FileProtocolHandler", url]);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = std::process::Command::new("open");
        command.arg(url);
        command
    };
    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(url);
        command
    };
    if let Err(error) = command.spawn() {
        log::warn!("open url failed: {error}");
    }
}
