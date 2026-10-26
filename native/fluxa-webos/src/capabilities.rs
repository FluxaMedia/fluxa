use std::time::Duration;

use serde_json::Value;

use crate::luna;

const CONFIG: &str = "luna://com.webos.service.config/getConfigs";
const ARC: &str = "luna://com.webos.service.arccontroller/getARCState";
const HDR_KEYS: [&str; 2] = ["tv.model.supportHDR", "tv.config.supportHDR"];
const DOLBY_KEYS: [&str; 2] = [
    "tv.config.supportDolbyHDRContents",
    "tv.model.supportDolbyHDRContents",
];
const EDID_KEY: &str = "tv.model.edidType";
const PLATFORM_KEY: &str = "tv.nyx.platformCode";
const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Capabilities {
    pub hdr: Option<bool>,
    pub dolby_vision: Option<bool>,
    pub dts: Option<bool>,
    pub eac3_out: bool,
    pub platform_code: Option<u32>,
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
    let platform_code = match &configs[PLATFORM_KEY] {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .and_then(|code| u32::try_from(code).ok())
    .filter(|code| (1..=30).contains(code));
    Capabilities {
        hdr: flag(&HDR_KEYS),
        dolby_vision: flag(&DOLBY_KEYS),
        dts: configs[EDID_KEY]
            .as_str()
            .map(|edid| edid.to_ascii_lowercase().contains("dts")),
        eac3_out: false,
        platform_code,
    }
}

pub fn parse_arc(json: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(json) else {
        return false;
    };
    value["arcActivation"] == "eARC"
        && value["earcCapability"]
            .as_array()
            .is_some_and(|caps| caps.iter().any(|cap| cap == "DD+"))
}

pub fn query(webos_major: u32) -> Capabilities {
    let names: Vec<&str> = HDR_KEYS
        .iter()
        .chain(DOLBY_KEYS.iter())
        .copied()
        .chain([EDID_KEY, PLATFORM_KEY])
        .collect();
    let payload = serde_json::json!({ "configNames": names }).to_string();
    let mut caps = luna::call(CONFIG, &payload, TIMEOUT)
        .map(|json| parse(&json))
        .unwrap_or_default();
    caps.eac3_out = luna::call(ARC, "{}", TIMEOUT).is_some_and(|json| parse_arc(&json));
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
        let caps = parse(json);
        assert_eq!((caps.hdr, caps.dolby_vision), (Some(true), Some(false)));
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

    #[test]
    fn dts_follows_the_edid_type() {
        assert_eq!(
            parse(r#"{"configs":{"tv.model.edidType":"hdmi_dts_ok"}}"#).dts,
            Some(true)
        );
        assert_eq!(
            parse(r#"{"configs":{"tv.model.edidType":"hdmi"}}"#).dts,
            Some(false)
        );
        assert_eq!(parse(r#"{"configs":{}}"#).dts, None);
    }

    #[test]
    fn platform_code_only_trusted_when_it_looks_like_a_version() {
        assert_eq!(
            parse(r#"{"configs":{"tv.nyx.platformCode":"6"}}"#).platform_code,
            Some(6)
        );
        assert_eq!(
            parse(r#"{"configs":{"tv.nyx.platformCode":7}}"#).platform_code,
            Some(7)
        );
        assert_eq!(
            parse(r#"{"configs":{"tv.nyx.platformCode":"O22"}}"#).platform_code,
            None
        );
        assert_eq!(
            parse(r#"{"configs":{"tv.nyx.platformCode":2022}}"#).platform_code,
            None
        );
    }

    #[test]
    fn earc_with_dd_plus_enables_eac3_output() {
        assert!(parse_arc(
            r#"{"arcActivation":"eARC","earcCapability":["PCM","DD+"]}"#
        ));
        assert!(!parse_arc(
            r#"{"arcActivation":"ARC","earcCapability":["DD+"]}"#
        ));
        assert!(!parse_arc(
            r#"{"arcActivation":"eARC","earcCapability":["PCM"]}"#
        ));
        assert!(!parse_arc("not json"));
    }
}
