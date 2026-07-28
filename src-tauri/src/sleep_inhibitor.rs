#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::{Child, Command, Stdio};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};

pub struct SleepInhibitor {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    process: Option<Child>,
}

impl Default for SleepInhibitor {
    fn default() -> Self {
        Self {
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            process: None,
        }
    }
}

impl SleepInhibitor {
    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            if enabled == self.process.is_some() {
                return Ok(());
            }
            if enabled {
                let mut command = Command::new("systemd-inhibit");
                command
                    .args([
                        "--what=sleep",
                        "--mode=block",
                        "--why=Video playback in Fluxa",
                        "sleep",
                        "infinity",
                    ])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                unsafe {
                    command.pre_exec(|| {
                        if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) == -1 {
                            return Err(std::io::Error::last_os_error());
                        }
                        Ok(())
                    });
                }
                self.process = Some(command.spawn().map_err(|error| error.to_string())?);
            } else { match self.process.take() { Some(mut process) => {
                let _ = process.kill();
                let _ = process.wait();
            } _ => {}}}
        }

        #[cfg(target_os = "macos")]
        {
            if enabled == self.process.is_some() {
                return Ok(());
            }
            if enabled {
                self.process = Some(
                    Command::new("caffeinate")
                        .args(["-i", "-s"])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .map_err(|error| error.to_string())?,
                );
            } else if let Some(mut process) = self.process.take() {
                let _ = process.kill();
                let _ = process.wait();
            }
        }

        #[cfg(target_os = "windows")]
        unsafe {
            SetThreadExecutionState(if enabled {
                ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED
            } else {
                ES_CONTINUOUS
            });
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let _ = enabled;

        Ok(())
    }
}

impl Drop for SleepInhibitor {
    fn drop(&mut self) {
        let _ = self.set_enabled(false);
    }
}
