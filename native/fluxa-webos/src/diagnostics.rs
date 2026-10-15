use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use serde_json::Value;

const TRACE_CAPACITY: usize = 64;

static TRACE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
static NOTES: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

const PRIVATE_KEYS: [&str; 6] = ["serial", "mac", "ssid", "uuid", "nduid", "device_id"];
const LIB_PREFIXES: [&str; 12] = [
    "libplayerAPIs", "libAcbAPI", "libav", "libsw", "libSDL2", "libEGL", "libGLES", "libmali", "libwayland",
    "libcurl", "libgst", "libstarfish",
];
const LIB_DIRS: [&str; 3] = ["/usr/lib", "/lib", "/usr/lib/gstreamer-1.0"];
const ENV_KEYS: [&str; 5] = ["APPID", "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "SDL_VIDEODRIVER", "LD_LIBRARY_PATH"];

pub fn note(key: impl Into<String>, value: impl Into<String>) {
    if let Ok(mut notes) = NOTES.lock() {
        notes.insert(key.into(), value.into());
    }
}

pub fn scrub_urls(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find("://") {
        out.push_str(&rest[..start + 3]);
        let after = &rest[start + 3..];
        let end = after.find(|c: char| c.is_whitespace() || c == '"' || c == '\\').unwrap_or(after.len());
        let authority_end = after[..end].find(['/', '?', '#']).unwrap_or(end);
        let authority = &after[..authority_end];
        out.push_str(authority.rsplit('@').next().unwrap_or(authority));
        if authority_end < end {
            out.push_str("/…");
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

pub fn trace(line: impl Into<String>) {
    let Ok(mut trace) = TRACE.lock() else {
        return;
    };
    if trace.len() == TRACE_CAPACITY {
        trace.pop_front();
    }
    trace.push_back(scrub_urls(&line.into()));
}

pub fn trace_dump() -> Vec<String> {
    TRACE.lock().map(|trace| trace.iter().cloned().collect()).unwrap_or_default()
}

fn library_inventory() -> Vec<String> {
    let mut libs = Vec::new();
    for dir in LIB_DIRS {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.contains(".so") || !LIB_PREFIXES.iter().any(|prefix| name.starts_with(prefix)) {
                continue;
            }
            let detail = match fs::read_link(entry.path()) {
                Ok(target) => format!("-> {}", target.display()),
                Err(_) => entry.metadata().map(|meta| format!("{} bytes", meta.len())).unwrap_or_default(),
            };
            libs.push(format!("{dir}/{name} {detail}"));
        }
    }
    libs.sort();
    libs
}

fn dev_nodes() -> Vec<String> {
    let mut nodes: Vec<String> = fs::read_dir("/dev")
        .map(|entries| entries.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    nodes.sort();
    nodes
}

pub fn already_reported(marker: &Path, release: &str) -> bool {
    fs::read_to_string(marker).is_ok_and(|saved| saved.trim() == release)
}

pub fn mark_reported(marker: &Path, release: &str) {
    if let Some(dir) = marker.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(marker, release);
}

#[derive(Default, Clone)]
pub struct Sources {
    pub os_info: Option<String>,
    pub device_info: Option<String>,
    pub kernel: Option<String>,
    pub cpuinfo: Option<String>,
    pub meminfo: Option<String>,
    pub rtkmem: bool,
    pub maps: Option<String>,
    pub libs: Vec<String>,
    pub dev_nodes: Vec<String>,
    pub env: Vec<(String, String)>,
    pub symbols: Vec<(String, bool)>,
}

#[derive(Default, Debug)]
pub struct Report {
    pub tags: BTreeMap<String, String>,
    pub extras: BTreeMap<String, String>,
}

fn flatten(prefix: &str, json: &str, tags: &mut BTreeMap<String, String>) {
    let Ok(Value::Object(map)) = serde_json::from_str::<Value>(json) else {
        return;
    };
    for (key, value) in map {
        let text = match value {
            Value::String(text) => text,
            Value::Number(number) => number.to_string(),
            Value::Bool(flag) => flag.to_string(),
            _ => continue,
        };
        let lower = key.to_lowercase();
        if PRIVATE_KEYS.iter().any(|private| lower.contains(private)) {
            continue;
        }
        tags.insert(format!("{prefix}.{key}"), text);
    }
}

fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .find(|line| line.split(':').next().is_some_and(|name| name.trim() == key))
        .and_then(|line| line.split_once(':'))
        .map(|(_, value)| value.trim())
}

impl Report {
    pub fn from_sources(sources: &Sources) -> Report {
        let mut report = Report::default();
        if let Some(json) = &sources.os_info {
            flatten("webos", json, &mut report.tags);
        }
        if let Some(json) = &sources.device_info {
            flatten("device", json, &mut report.tags);
        }
        if let Some(kernel) = &sources.kernel {
            report.extras.insert("kernel".into(), kernel.trim().to_owned());
        }
        if let Some(cpu) = &sources.cpuinfo {
            for key in ["Hardware", "model name", "CPU part", "CPU architecture"] {
                if let Some(value) = field(cpu, key) {
                    report.tags.insert(format!("cpu.{}", key.replace(' ', "_")), value.to_owned());
                }
            }
        }
        if let Some(total) = sources.meminfo.as_deref().and_then(|text| field(text, "MemTotal")) {
            report.tags.insert("mem.total".into(), total.to_owned());
        }
        report.tags.insert("devjail.rtkmem".into(), sources.rtkmem.to_string());
        if let Some(maps) = &sources.maps {
            let mut loaded: Vec<&str> = maps
                .lines()
                .filter_map(|line| line.split_whitespace().nth(5))
                .filter(|path| path.contains(".so"))
                .collect();
            loaded.sort_unstable();
            loaded.dedup();
            report.extras.insert("loaded_libraries".into(), loaded.join("\n"));
        }
        if !sources.libs.is_empty() {
            report.extras.insert("library_inventory".into(), sources.libs.join("\n"));
        }
        if !sources.dev_nodes.is_empty() {
            report.extras.insert("dev_nodes".into(), sources.dev_nodes.join(" "));
        }
        for (key, value) in &sources.env {
            report.tags.insert(format!("env.{key}"), value.clone());
        }
        if !sources.symbols.is_empty() {
            let missing: Vec<&str> =
                sources.symbols.iter().filter(|(_, found)| !found).map(|(name, _)| name.as_str()).collect();
            report.tags.insert("starfish.missing_symbols".into(), missing.len().to_string());
            let all: Vec<String> =
                sources.symbols.iter().map(|(name, found)| format!("{} {name}", if *found { '+' } else { '-' })).collect();
            report.extras.insert("starfish.symbols".into(), all.join("\n"));
        }
        report
    }

    pub fn collect() -> Report {
        let read = |path: &str| fs::read_to_string(path).ok();
        let mut report = Report::from_sources(&Sources {
            os_info: read("/var/run/nyx/os_info.json"),
            device_info: read("/var/run/nyx/device_info.json"),
            kernel: read("/proc/version"),
            cpuinfo: read("/proc/cpuinfo"),
            meminfo: read("/proc/meminfo"),
            rtkmem: fs::metadata("/dev/rtkmem").is_ok(),
            maps: read("/proc/self/maps"),
            libs: library_inventory(),
            dev_nodes: dev_nodes(),
            env: ENV_KEYS
                .iter()
                .filter_map(|key| std::env::var(key).ok().map(|value| (key.to_string(), value)))
                .collect(),
            symbols: crate::ffi::probe_symbols(),
        });
        if let Ok(notes) = NOTES.lock() {
            report.tags.extend(notes.iter().map(|(key, value)| (key.clone(), value.clone())));
        }
        report
    }

    pub fn webos_major(&self) -> Option<u32> {
        let release = self.tags.get("webos.webos_release")?;
        release.split('.').next()?.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources() -> Sources {
        Sources {
            os_info: Some(r#"{"webos_release":"4.10.2","webos_build_id":"123","nested":{"a":1}}"#.into()),
            device_info: Some(r#"{"model_name":"OLED65C7","hardware_id":"m16p"}"#.into()),
            kernel: Some("Linux version 4.4.84\n".into()),
            cpuinfo: Some("Hardware\t: Realtek\nCPU part\t: 0xc07\n".into()),
            meminfo: Some("MemTotal:        1741312 kB\n".into()),
            rtkmem: false,
            ..Sources::default()
        }
    }

    #[test]
    fn release_major_comes_from_os_info() {
        assert_eq!(Report::from_sources(&sources()).webos_major(), Some(4));
    }

    #[test]
    fn nested_json_values_are_skipped() {
        let report = Report::from_sources(&sources());
        assert!(!report.tags.keys().any(|key| key.contains("nested")));
        assert_eq!(report.tags["device.hardware_id"], "m16p");
    }

    #[test]
    fn cpu_and_memory_fields_are_tagged() {
        let report = Report::from_sources(&sources());
        assert_eq!(report.tags["cpu.Hardware"], "Realtek");
        assert_eq!(report.tags["mem.total"], "1741312 kB");
        assert_eq!(report.tags["devjail.rtkmem"], "false");
    }

    #[test]
    fn serial_and_mac_keys_never_become_tags() {
        let mut sources = sources();
        sources.device_info =
            Some(r#"{"serial_number":"X1","wifi_mac":"aa","model_name":"C7","device_id":"d"}"#.into());
        let report = Report::from_sources(&sources);
        assert!(!report.tags.keys().any(|key| key.contains("serial") || key.contains("mac") || key.contains("device_id")));
        assert_eq!(report.tags["device.model_name"], "C7");
    }

    #[test]
    fn urls_keep_host_but_lose_path_query_and_credentials() {
        let line = r#"load {"url":"https://user:pw@cdn.example.com:8443/a/b.mkv?token=abc"} done"#;
        let clean = scrub_urls(line);
        assert!(clean.contains("https://cdn.example.com:8443/…"));
        assert!(!clean.contains("token") && !clean.contains("pw@"));
        assert!(clean.ends_with("done"));
    }

    #[test]
    fn symbol_probe_is_summarised() {
        let mut sources = sources();
        sources.symbols = vec![("a".into(), true), ("b".into(), false)];
        let report = Report::from_sources(&sources);
        assert_eq!(report.tags["starfish.missing_symbols"], "1");
        assert!(report.extras["starfish.symbols"].contains("- b"));
    }

    #[test]
    fn report_marker_is_per_release() {
        let dir = std::env::temp_dir().join(format!("fluxa-webos-marker-{}", std::process::id()));
        let marker = dir.join("compat");
        assert!(!already_reported(&marker, "1.0"));
        mark_reported(&marker, "1.0");
        assert!(already_reported(&marker, "1.0"));
        assert!(!already_reported(&marker, "1.1"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_sources_still_produce_a_report() {
        let report = Report::from_sources(&Sources::default());
        assert_eq!(report.webos_major(), None);
        assert!(report.tags.contains_key("devjail.rtkmem"));
    }

    #[test]
    fn trace_keeps_only_the_newest_lines() {
        for index in 0..TRACE_CAPACITY + 5 {
            trace(format!("line {index}"));
        }
        let dump = trace_dump();
        assert_eq!(dump.len(), TRACE_CAPACITY);
        assert_eq!(dump.last().unwrap(), &format!("line {}", TRACE_CAPACITY + 4));
    }
}
