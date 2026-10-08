use crate::profile::prefs::safe_audio_processing_mode;
use serde_json::{Value, json};

const ROUTE_OWNED_OPTIONS: [&str; 3] = ["audio-spdif", "audio-channels", "af"];

fn audio_filter(mode: &str) -> &'static str {
    match mode {
        "balanced" => {
            "lavfi=[acompressor=threshold=0.78:ratio=1.5:attack=30:release=300:link=maximum,alimiter=limit=0.98]"
        }
        "night" => {
            "lavfi=[acompressor=threshold=0.55:ratio=3:attack=20:release=250:link=maximum,alimiter=limit=0.98]"
        }
        _ => "",
    }
}

fn custom_options(text: &str) -> String {
    text.lines()
        .filter_map(|line| {
            let (key, value) = line.trim().split_once('=')?;
            let key = key.trim();
            let owned = key.starts_with('#') || ROUTE_OWNED_OPTIONS.contains(&key);
            (!key.is_empty() && !owned).then(|| format!("{key}={}", value.trim()))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn video_config_plan_json(input: &str) -> Option<String> {
    let settings: Value = serde_json::from_str(input).ok()?;
    let text = |key: &str| {
        settings
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
    };
    let or = |key: &str, fallback: &'static str| match text(key) {
        "" => fallback.to_owned(),
        value => value.to_owned(),
    };
    let language = |key: &str| match text(key) {
        "none" => "",
        value => value,
    };
    let mode = safe_audio_processing_mode(Some(text("audioProcessingMode")));
    Some(
        json!({
            "mpvOptions": custom_options(text("mpvCustomOptions")),
            "audioFilter": audio_filter(mode),
            "passthrough": mode == "reference",
            "dolbyVisionMode": or("dolbyVisionMode", "auto").replace('_', "-"),
            "dolbyVisionTonemap": or("dolbyVisionTonemap", "auto"),
            "displayPeakNits": or("displayPeakNits", "0"),
            "audioLanguage": language("preferredAudioLanguage"),
            "subtitleLanguage": language("preferredSubtitleLanguage"),
        })
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(settings: Value) -> Value {
        serde_json::from_str(&video_config_plan_json(&settings.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn route_owned_audio_options_are_dropped() {
        let plan = plan(json!({
            "mpvCustomOptions": "# note\naudio-spdif=ac3\n af = lavfi=x \nhwdec=auto\nno-equals\n"
        }));
        assert_eq!(plan["mpvOptions"], "hwdec=auto");
    }

    #[test]
    fn processing_mode_picks_filter_and_passthrough() {
        let night = plan(json!({"audioProcessingMode": "night"}));
        assert!(night["audioFilter"].as_str().unwrap().contains("threshold=0.55"));
        assert_eq!(night["passthrough"], false);
        let unknown = plan(json!({"audioProcessingMode": "bogus"}));
        assert_eq!(unknown["audioFilter"], "");
        assert_eq!(unknown["passthrough"], true);
    }

    #[test]
    fn blank_settings_fall_back_to_defaults() {
        let plan = plan(json!({
            "dolbyVisionMode": "profile_5",
            "preferredAudioLanguage": "none",
            "preferredSubtitleLanguage": "tr",
        }));
        assert_eq!(plan["dolbyVisionMode"], "profile-5");
        assert_eq!(plan["dolbyVisionTonemap"], "auto");
        assert_eq!(plan["displayPeakNits"], "0");
        assert_eq!(plan["audioLanguage"], "");
        assert_eq!(plan["subtitleLanguage"], "tr");
    }
}
