use super::*;

#[derive(Clone, Debug, Default)]
pub struct SettingsModel {
    pub values: serde_json::Value,
    pub last_write_error: Option<String>,
    pub active_section: usize,
    pub profile: serde_json::Value,
    pub addons: Vec<serde_json::Value>,
    pub addon_error: Option<String>,
    pub plugins: serde_json::Value,
    pub addon_url: String,
    pub plugin_url: String,
    pub poster_fields: [String; 6],
    pub search: String,
    pub section_open: bool,
    pub page_open: bool,
    pub account_auth: Option<AccountPrompt>,
    pub media_servers: Vec<serde_json::Value>,
    pub server_fields: [String; 3],
    pub shortcuts: Vec<ShortcutRow>,
    pub playback_summary: Vec<crate::StatsSection>,
    pub shortcut_recording: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ShortcutRow {
    pub id: String,
    pub player: bool,
    pub label: String,
    pub custom: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AccountPrompt {
    pub provider: String,
    pub code: String,
    pub url: String,
    pub failed: bool,
}

impl SettingsModel {
    pub fn language(&self) -> &str {
        self.values
            .get("language")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("en")
    }
    pub fn value(&self, key: &str) -> Option<&serde_json::Value> {
        self.values
            .get(key)
            .filter(|value| !value.is_null())
            .or_else(|| setting_default(key))
            .or_else(|| poster_overlay::setting_default(key))
    }

    pub fn app_icon(&self) -> Option<&str> {
        self.value("appIcon").and_then(|value| value.as_str())
    }

    pub fn ui_scale(&self) -> f32 {
        self.value("uiScale")
            .and_then(|value| {
                value
                    .as_f64()
                    .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
            })
            .map(|percent| (percent as f32 / 100.0).clamp(0.5, 2.0))
            .unwrap_or(1.0)
    }

    pub fn number_value(&self, key: &str) -> Option<f64> {
        self.value(key).and_then(|value| {
            value
                .as_f64()
                .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        })
    }

    pub fn str_value(&self, key: &str) -> Option<&str> {
        self.value(key).and_then(serde_json::Value::as_str)
    }

    pub fn bool_value(&self, key: &str) -> bool {
        self.value(key)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
    }

    pub fn display_value(&self, row: &SettingsRow) -> String {
        if row.options.is_empty() {
            if self.bool_value(row.key) {
                "On"
            } else {
                "Off"
            }
            .to_owned()
        } else {
            self.value(row.key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or(row.options[0])
                .replace('_', " ")
        }
    }

    pub fn next_value(&self, row: &SettingsRow) -> serde_json::Value {
        self.next_value_for(row, UiFormFactor::Mobile)
    }

    pub fn next_value_for(
        &self,
        row: &SettingsRow,
        _form_factor: UiFormFactor,
    ) -> serde_json::Value {
        if row.options.is_empty() {
            serde_json::Value::Bool(!self.bool_value(row.key))
        } else {
            let options = row.options;
            let current = self.value(row.key).and_then(serde_json::Value::as_str);
            let next = current
                .and_then(|value| options.iter().position(|option| *option == value))
                .map(|index| (index + 1) % options.len())
                .unwrap_or(1 % options.len());
            serde_json::Value::String(options[next].to_owned())
        }
    }
}

pub fn settings_model_from_core_snapshot(snapshot: &serde_json::Value) -> SettingsModel {
    let settings = snapshot.get("settings").unwrap_or(&serde_json::Value::Null);
    let addon_state = snapshot.get("addons").unwrap_or(&serde_json::Value::Null);
    SettingsModel {
        values: settings
            .get("values")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        last_write_error: settings
            .get("lastWriteError")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        active_section: 0,
        section_open: false,
        page_open: false,
        profile: snapshot
            .pointer("/profile/active")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        addons: addon_state
            .get("installed")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default(),
        addon_error: addon_state
            .get("error")
            .and_then(|v| v.get("message"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        plugins: snapshot
            .get("plugins")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        addon_url: String::new(),
        plugin_url: String::new(),
        poster_fields: POSTER_FIELDS.map(|field| {
            settings
                .pointer(&format!("/values/{}", field.key))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_owned()
        }),
        search: String::new(),
        account_auth: None,
        media_servers: Vec::new(),
        server_fields: Default::default(),
        shortcuts: Vec::new(),
        playback_summary: Vec::new(),
        shortcut_recording: None,
    }
}
