use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::probe::{ProbeFormat, Prober};

pub struct CliProber {
    pub binary: PathBuf,
    pub format: ProbeFormat,
    pub timeout_secs: u32,
}

impl CliProber {
    pub fn ffprobe() -> Self {
        Self {
            binary: PathBuf::from("ffprobe"),
            format: ProbeFormat::Ffprobe,
            timeout_secs: 15,
        }
    }

    pub fn mediainfo() -> Self {
        Self {
            binary: PathBuf::from("mediainfo"),
            format: ProbeFormat::MediaInfo,
            timeout_secs: 15,
        }
    }

    fn command(&self, url: &str, headers: &HashMap<String, String>) -> Command {
        let mut cmd = Command::new(&self.binary);
        match self.format {
            ProbeFormat::Ffprobe => {
                cmd.args(["-v", "error", "-print_format", "json", "-show_streams"]);
                cmd.args([
                    "-rw_timeout",
                    &(u64::from(self.timeout_secs) * 1_000_000).to_string(),
                ]);
                if !headers.is_empty() {
                    let block: String = headers
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}\r\n"))
                        .collect();
                    cmd.args(["-headers", &block]);
                }
                cmd.arg(url);
            }
            ProbeFormat::MediaInfo => {
                cmd.args(["--Output=JSON", url]);
            }
        }
        cmd
    }
}

impl Prober for CliProber {
    fn format(&self) -> ProbeFormat {
        self.format
    }

    async fn run(&self, url: &str, headers: &HashMap<String, String>) -> Result<String, String> {
        let output = self
            .command(url, headers)
            .output()
            .map_err(|e| format!("{}: {e}", self.binary.display()))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffprobe_gets_headers_and_the_url_last() {
        let headers: HashMap<String, String> =
            [("Referer".to_string(), "https://x".to_string())].into();
        let cmd = CliProber::ffprobe().command("https://x/a.mkv", &headers);
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args.last().map(String::as_str), Some("https://x/a.mkv"));
        assert!(args.iter().any(|a| a == "-headers"));
        assert!(args.iter().any(|a| a.contains("Referer: https://x")));
    }

    #[test]
    fn mediainfo_asks_for_json() {
        let cmd = CliProber::mediainfo().command("https://x/a.mkv", &HashMap::new());
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["--Output=JSON", "https://x/a.mkv"]);
    }
}
