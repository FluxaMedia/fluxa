use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::sync::Mutex;

use serde_json::Value;

const TRACE_CAPACITY: usize = 64;

static TRACE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

pub fn trace(line: impl Into<String>) {
    let Ok(mut trace) = TRACE.lock() else {
        return;
    };
    if trace.len() == TRACE_CAPACITY {
        trace.pop_front();
    }
    trace.push_back(line.into());
}

pub fn trace_dump() -> Vec<String> {
    TRACE.lock().map(|trace| trace.iter().cloned().collect()).unwrap_or_default()
}

#[derive(Default, Clone)]
pub struct Sources {
    pub os_info: Option<String>,
    pub device_info: Option<String>,
    pub kernel: Option<String>,
    pub cpuinfo: Option<String>,
    pub meminfo: Option<String>,
    pub rtkmem: bool,
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
        report
    }

    pub fn collect() -> Report {
        let read = |path: &str| fs::read_to_string(path).ok();
        Report::from_sources(&Sources {
            os_info: read("/var/run/nyx/os_info.json"),
            device_info: read("/var/run/nyx/device_info.json"),
            kernel: read("/proc/version"),
            cpuinfo: read("/proc/cpuinfo"),
            meminfo: read("/proc/meminfo"),
            rtkmem: fs::metadata("/dev/rtkmem").is_ok(),
        })
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
