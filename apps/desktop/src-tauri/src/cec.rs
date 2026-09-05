use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CecStatus {
    pub supported: bool,
    pub enabled: bool,
    pub device: Option<String>,
    pub backend: Option<String>,
}

#[derive(Clone, Debug)]
enum CecBackend {
    Kernel(PathBuf),
    LibCec,
}

pub struct CecController {
    enabled: bool,
    backend: Option<CecBackend>,
    stop_monitor: Option<Arc<AtomicBool>>,
    monitor_child: Option<Arc<Mutex<std::process::Child>>>,
    monitor_thread: Option<JoinHandle<()>>,
}

impl Default for CecController {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: None,
            stop_monitor: None,
            monitor_child: None,
            monitor_thread: None,
        }
    }
}

impl CecController {
    pub fn status(&mut self) -> CecStatus {
        self.refresh_backend();
        let (supported, device, backend) = match &self.backend {
            Some(CecBackend::Kernel(device)) => (
                true,
                Some(device.display().to_string()),
                Some("linux-cec-ctl".to_string()),
            ),
            Some(CecBackend::LibCec) => (true, None, Some("libcec-client".to_string())),
            None => (false, None, None),
        };
        CecStatus {
            supported,
            enabled: self.enabled,
            device,
            backend,
        }
    }

    pub fn activate_source(&mut self) {
        if !self.enabled || self.monitor_thread.is_some() {
            return;
        }
        let Some(backend) = self.refresh_backend() else {
            return;
        };
        let result = match backend {
            CecBackend::Kernel(device) => {
                let device_string = device.to_string_lossy().to_string();
                let physical_address =
                    physical_address(&device).unwrap_or_else(|| "0.0.0.0".to_string());
                let active_source = format!("phys-addr={physical_address}");
                run_cec_ctl(&[
                    "--device",
                    device_string.as_str(),
                    "--playback",
                    "--no-reply",
                    "--image-view-on",
                    "--active-source",
                    active_source.as_str(),
                ])
            }
            CecBackend::LibCec => run_cec_client("on 0").and_then(|_| run_cec_client("as")),
        };
        if let Err(error) = result {
            log::debug!("CEC source activation unavailable: {error}");
        }
    }

    pub fn standby(&mut self) {
        if !self.enabled {
            return;
        }
        let Some(backend) = self.refresh_backend() else {
            return;
        };
        let result = match backend {
            CecBackend::Kernel(device) => {
                let device_string = device.to_string_lossy().to_string();
                run_cec_ctl(&[
                    "--device",
                    device_string.as_str(),
                    "--playback",
                    "--no-reply",
                    "--standby",
                ])
            }
            CecBackend::LibCec => run_cec_client("standby 0"),
        };
        if let Err(error) = result {
            log::debug!("CEC standby unavailable: {error}");
        }
    }

    pub fn start_monitor(&mut self, app: AppHandle) {
        if self.monitor_thread.is_some() || !self.enabled {
            return;
        }
        let Some(backend) = self.refresh_backend() else {
            return;
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let mut child = match spawn_monitor(&backend) {
            Ok(child) => child,
            Err(error) => {
                log::debug!("CEC monitor unavailable: {error}");
                return;
            }
        };
        let Some(stdout) = child.stdout.take() else {
            let _ = child.kill();
            return;
        };
        let child = Arc::new(Mutex::new(child));
        let child_thread = child.clone();
        let thread = std::thread::spawn(move || {
            let reader = std::io::BufReader::new(stdout);
            for line in std::io::BufRead::lines(reader) {
                if stop_thread.load(Ordering::Acquire) {
                    break;
                }
                let Ok(line) = line else {
                    break;
                };
                let command = match &backend {
                    CecBackend::Kernel(_) => parse_user_control(&line),
                    CecBackend::LibCec => parse_libcec_user_control(&line),
                };
                if let Some(command) = command {
                    let _ = app.emit("native-cec-command", command);
                }
            }
            if let Ok(mut child) = child_thread.lock() {
                let _ = child.kill();
                let _ = child.wait();
            }
        });
        self.stop_monitor = Some(stop);
        self.monitor_child = Some(child);
        self.monitor_thread = Some(thread);
    }

    pub fn stop_monitor(&mut self) {
        if let Some(stop) = self.stop_monitor.take() {
            stop.store(true, Ordering::Release);
        }
        if let Some(child) = self.monitor_child.take() {
            if let Ok(mut child) = child.lock() {
                let _ = child.kill();
            }
        }
        if let Some(thread) = self.monitor_thread.take() {
            let _ = thread.join();
        }
    }

    fn refresh_backend(&mut self) -> Option<CecBackend> {
        if self.backend.is_some() {
            return self.backend.clone();
        }
        self.backend = discover_backend();
        self.backend.clone()
    }
}

impl Drop for CecController {
    fn drop(&mut self) {
        self.stop_monitor();
    }
}

#[cfg(target_os = "linux")]
fn discover_kernel_device() -> Option<PathBuf> {
    let mut devices = std::fs::read_dir("/dev")
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("cec"))
        })
        .collect::<Vec<_>>();
    devices.sort();
    devices.into_iter().find(|path| {
        std::fs::metadata(path)
            .map(|metadata| {
                use std::os::unix::fs::FileTypeExt;
                metadata.file_type().is_char_device()
            })
            .unwrap_or(false)
    })
}

fn discover_backend() -> Option<CecBackend> {
    #[cfg(target_os = "linux")]
    if let Some(device) = discover_kernel_device() {
        return Some(CecBackend::Kernel(device));
    }
    command_available("cec-client").then_some(CecBackend::LibCec)
}

#[cfg(not(target_os = "linux"))]
fn discover_kernel_device() -> Option<PathBuf> {
    None
}

fn physical_address(device: &Path) -> Option<String> {
    let output = Command::new("cec-ctl")
        .args([
            "--device",
            device.to_string_lossy().as_ref(),
            "--physical-address",
            "--skip-info",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .find_map(|line| line.split_once(':').map(|(_, value)| value.trim()))
        .filter(|value| value.chars().filter(|ch| *ch == '.').count() == 3)
        .map(str::to_string)
}

fn run_cec_ctl(args: &[&str]) -> Result<(), String> {
    let output = Command::new("cec-ctl")
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn command_available(command: &str) -> bool {
    Command::new(command)
        .arg("-l")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn run_cec_client(command: &str) -> Result<(), String> {
    let mut child = Command::new("cec-client")
        .args(["-s", "-d", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    child
        .stdin
        .take()
        .ok_or_else(|| "cec-client stdin is unavailable".to_string())?
        .write_all(format!("{command}\n").as_bytes())
        .map_err(|error| error.to_string())?;
    let output = child
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn spawn_monitor(backend: &CecBackend) -> Result<std::process::Child, String> {
    let mut command = match backend {
        CecBackend::Kernel(device) => {
            let mut command = Command::new("cec-ctl");
            command.args([
                "--device",
                device.to_string_lossy().as_ref(),
                "--playback",
                "--monitor-all",
                "--monitor-time",
                "0",
            ]);
            command
        }
        CecBackend::LibCec => {
            let mut command = Command::new("cec-client");
            command.args(["-d", "1"]).stdin(Stdio::null());
            command
        }
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| error.to_string())
}

fn parse_user_control(line: &str) -> Option<&'static str> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("user control pressed") {
        return None;
    }
    if lower.contains("play") && !lower.contains("pause") {
        Some("play")
    } else if lower.contains("pause") {
        Some("pause")
    } else if lower.contains("stop") {
        Some("stop")
    } else if lower.contains("volume up") {
        Some("volume-up")
    } else if lower.contains("volume down") {
        Some("volume-down")
    } else if lower.contains("mute") {
        Some("mute")
    } else if lower.contains("skip forward") {
        Some("skip-forward")
    } else if lower.contains("skip backward") {
        Some("skip-backward")
    } else {
        None
    }
}

fn parse_libcec_user_control(line: &str) -> Option<&'static str> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("key pressed") {
        return None;
    }
    if lower.contains("play") && !lower.contains("pause") {
        Some("play")
    } else if lower.contains("pause") {
        Some("pause")
    } else if lower.contains("stop") {
        Some("stop")
    } else if lower.contains("volume up") {
        Some("volume-up")
    } else if lower.contains("volume down") {
        Some("volume-down")
    } else if lower.contains("mute") {
        Some("mute")
    } else if lower.contains("skip forward") {
        Some("skip-forward")
    } else if lower.contains("skip backward") {
        Some("skip-backward")
    } else {
        None
    }
}

pub fn activate_source(state: &Mutex<CecController>) {
    state.lock().unwrap().activate_source();
}

pub fn start_monitor(state: &Mutex<CecController>, app: AppHandle) {
    state.lock().unwrap().start_monitor(app);
}

pub fn stop_monitor(state: &Mutex<CecController>) {
    state.lock().unwrap().stop_monitor();
}

pub fn standby(state: &Mutex<CecController>) {
    state.lock().unwrap().standby();
}

#[tauri::command]
pub fn cec_standby(state: tauri::State<'_, crate::DesktopState>) {
    standby(&state.cec);
}

#[tauri::command]
pub fn cec_status(state: tauri::State<'_, crate::DesktopState>) -> CecStatus {
    state.cec.lock().unwrap().status()
}

#[cfg(test)]
mod tests {
    use super::parse_user_control;

    #[test]
    fn parses_supported_remote_commands() {
        assert_eq!(
            parse_user_control("User Control Pressed (0x44): Play (0x44)"),
            Some("play")
        );
        assert_eq!(
            parse_user_control("User Control Pressed: Volume Up"),
            Some("volume-up")
        );
        assert_eq!(parse_user_control("Report Power Status"), None);
    }
}
