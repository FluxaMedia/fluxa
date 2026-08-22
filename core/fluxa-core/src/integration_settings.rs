use serde_json::{Value, json};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntegrationSettings {
    pub library_source: String,
    pub watch_progress_source: String,
    pub continue_watching_days: i64,
    pub similar_titles_source: String,
    pub trakt_comments_enabled: bool,
}

impl Default for IntegrationSettings {
    fn default() -> Self {
        Self {
            library_source: "local".to_string(),
            watch_progress_source: "all".to_string(),
            continue_watching_days: 0,
            similar_titles_source: "auto".to_string(),
            trakt_comments_enabled: false,
        }
    }
}

pub(crate) fn integration_settings_from_value(value: Option<&Value>) -> IntegrationSettings {
    let value = value.and_then(Value::as_object);
    let enum_value = |key: &str, allowed: &[&str], fallback: &str| {
        value
            .and_then(|object| object.get(key))
            .and_then(Value::as_str)
            .filter(|candidate| allowed.contains(candidate))
            .unwrap_or(fallback)
            .to_string()
    };
    IntegrationSettings {
        library_source: enum_value(
            "librarySource",
            &["local", "trakt", "simkl", "nuvio", "anilist", "stremio"],
            "local",
        ),
        watch_progress_source: enum_value(
            "watchProgressSource",
            &["all", "trakt", "simkl", "nuvio", "stremio"],
            "all",
        ),
        continue_watching_days: value
            .and_then(|object| object.get("continueWatchingDays"))
            .and_then(Value::as_i64)
            .filter(|days| *days == 0 || (1..=365).contains(days))
            .unwrap_or(0),
        similar_titles_source: enum_value(
            "similarTitlesSource",
            &["auto", "trakt", "simkl", "tmdb"],
            "auto",
        ),
        trakt_comments_enabled: value
            .and_then(|object| object.get("traktCommentsEnabled"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

pub(crate) fn integration_settings_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let settings = integration_settings_from_value(args.get("settings").or(Some(&args)));
    serde_json::to_string(&json!({
        "librarySource": settings.library_source,
        "watchProgressSource": settings.watch_progress_source,
        "continueWatchingDays": settings.continue_watching_days,
        "similarTitlesSource": settings.similar_titles_source,
        "traktCommentsEnabled": settings.trakt_comments_enabled,
    }))
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_integration_settings() {
        let result = integration_settings_plan_json(r#"{"settings":{"librarySource":"trakt","watchProgressSource":"simkl","continueWatchingDays":30,"similarTitlesSource":"trakt","traktCommentsEnabled":true}}"#).unwrap();
        let value: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value["librarySource"], "trakt");
        assert_eq!(value["watchProgressSource"], "simkl");
        assert_eq!(value["continueWatchingDays"], 30);
        assert_eq!(value["similarTitlesSource"], "trakt");
        assert_eq!(value["traktCommentsEnabled"], true);
    }

    #[test]
    fn rejects_invalid_integration_settings() {
        let settings = integration_settings_from_value(Some(
            &json!({"watchProgressSource":"invalid","continueWatchingDays":366}),
        ));
        assert_eq!(settings, IntegrationSettings::default());
    }
}
