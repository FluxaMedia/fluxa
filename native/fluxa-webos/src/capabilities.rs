use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

use serde_json::Value;

const HDR_KEYS: [&str; 2] = ["tv.model.supportHDR", "tv.config.supportHDR"];
const DOLBY_KEYS: [&str; 2] = ["tv.config.supportDolbyHDRContents", "tv.model.supportDolbyHDRContents"];

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Capabilities {
    pub hdr: Option<bool>,
    pub dolby_vision: Option<bool>,
}

pub fn parse(json: &str) -> Capabilities {
    let Ok(value) = serde_json::from_str::<Value>(json) else {
        return Capabilities::default();
    };
    let configs = &value["configs"];
    let flag = |keys: &[&str]| {
        let answers: Vec<bool> = keys
            .iter()
            .filter_map(|key| match &configs[*key] {
                Value::Bool(flag) => Some(*flag),
                Value::String(text) => Some(text == "true"),
                _ => None,
            })
            .collect();
        (!answers.is_empty()).then(|| answers.contains(&true))
    };
    Capabilities { hdr: flag(&HDR_KEYS), dolby_vision: flag(&DOLBY_KEYS) }
}

pub fn query(webos_major: u32) -> Capabilities {
    let names: Vec<&str> = HDR_KEYS.iter().chain(DOLBY_KEYS.iter()).copied().collect();
    let payload = serde_json::json!({ "configNames": names }).to_string();
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let output = Command::new("luna-send")
            .args(["-n", "1", "-f", "luna://com.webos.service.config/getConfigs", &payload])
            .output();
        let _ = tx.send(output);
    });
    let mut caps = match rx.recv_timeout(Duration::from_secs(3)) {
        Ok(Ok(output)) => parse(&String::from_utf8_lossy(&output.stdout)),
        _ => Capabilities::default(),
    };
    if webos_major < 4 {
        caps.dolby_vision = Some(false);
    }
    caps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_booleans_are_read() {
        let json = r#"{"returnValue":true,"configs":{"tv.model.supportHDR":true,"tv.model.supportDolbyHDRContents":false}}"#;
        assert_eq!(parse(json), Capabilities { hdr: Some(true), dolby_vision: Some(false) });
    }

    #[test]
    fn missing_or_garbage_output_means_unknown() {
        assert_eq!(parse("not json"), Capabilities::default());
        assert_eq!(parse(r#"{"configs":{}}"#), Capabilities::default());
    }

    #[test]
    fn either_key_saying_true_counts() {
        let json = r#"{"configs":{"tv.config.supportDolbyHDRContents":true,"tv.model.supportDolbyHDRContents":false}}"#;
        assert_eq!(parse(json).dolby_vision, Some(true));
    }

    #[test]
    fn string_flags_are_accepted() {
        let json = r#"{"configs":{"tv.model.supportHDR":"true"}}"#;
        assert_eq!(parse(json).hdr, Some(true));
    }
}
