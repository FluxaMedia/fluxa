use super::*;

#[derive(Clone, Debug, Default)]
pub struct SettingsModel {
    pub values: serde_json::Value,
    pub last_write_error: Option<String>,
    pub active_section: usize,
    pub profile: serde_json::Value,
    pub addons: Vec<serde_json::Value>,
    pub addon_error: Option<String>,
    pub unreachable_addon_hosts: Vec<(String, String)>,
    pub plugins: serde_json::Value,
    pub addon_url: String,
    pub plugin_url: String,
    pub plugin_config: Option<usize>,
    pub plugin_config_fields: [String; PLUGIN_CONFIG_FIELDS],
    pub poster_fields: [String; 8],
    pub search: String,
    pub section_open: bool,
    pub page_open: bool,
    pub account_auth: Option<AccountPrompt>,
    pub mdblist_lists: Vec<(String, String)>,
    pub tracker_names: Vec<(String, String)>,
    pub media_servers: Vec<serde_json::Value>,
    pub server_fields: [String; 3],
    pub server_kind: usize,
    pub shortcuts: Vec<ShortcutRow>,
    pub gamepad_active: bool,
    pub playback_summary: Vec<crate::StatsSection>,
    pub shortcut_recording: Option<String>,
    pub badge_fields: [String; BADGE_INPUT_COUNT],
    pub badge_outline: bool,
    pub badge_preview: crate::SourceBadge,
    pub badge_packs: Vec<BadgePackRow>,
    pub badge_custom: Vec<BadgeCustomRow>,
    pub formatter_rules: Vec<FormatterRow>,
    pub formatter_presets: Vec<FormatterPresetRow>,
    pub layout_preview: Vec<String>,
    pub badge_status: Option<String>,
    pub formatter_status: Option<String>,
    pub collection_fields: [String; COLLECTION_INPUT_COUNT],
    pub collections: Vec<CollectionListRow>,
    pub collection_edit: Option<CollectionEdit>,
    pub collection_status: Option<String>,
    pub update: UpdateView,
    pub watch_room: Option<String>,
    pub watch_code: String,
    pub watch_ticket: crate::WatchTicket,
    pub ambient_catalogs: Vec<AmbientCatalogRow>,
    pub hero_catalogs: Vec<AmbientCatalogRow>,
    pub ambient_minutes: u32,
    pub ambient_limit: usize,
    pub detail_sections: Vec<DetailSectionRow>,
}

#[derive(Clone, Copy, Debug)]
pub struct DetailSectionRow {
    pub section: DetailSection,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AmbientCatalogRow {
    pub name: String,
    pub selected: bool,
}

#[derive(Clone, Debug, Default)]
pub struct CollectionListRow {
    pub title: String,
    pub folders: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CollectionEdit {
    pub view_mode: String,
    pub show_all_tab: bool,
    pub pin_to_top: bool,
    pub focus_glow: bool,
    pub show_on_home: bool,
    pub folders: Vec<CollectionListRow>,
    pub folder: Option<FolderEdit>,
}

#[derive(Clone, Debug, Default)]
pub struct FolderEdit {
    pub shape: String,
    pub hide_title: bool,
    pub provider: String,
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct BadgePackRow {
    pub name: String,
    pub count: usize,
    pub active: bool,
}

#[derive(Clone, Debug, Default)]
pub struct BadgeCustomRow {
    pub badge: crate::SourceBadge,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct FormatterPresetRow {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct FormatterRow {
    pub pattern: String,
    pub replacement: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(String),
    Downloading(u8),
    Ready(String),
    Failed(String),
}

#[derive(Clone, Debug, Default)]
pub struct UpdateView {
    pub supported: bool,
    pub current: String,
    pub state: UpdateState,
    pub prompt: bool,
    pub reveal: f32,
    pub top: f32,
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
        unreachable_addon_hosts: Vec::new(),
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
        plugin_config: None,
        plugin_config_fields: Default::default(),
        poster_fields: POSTER_FIELDS.map(|field| {
            settings
                .pointer(&format!("/values/{}", field.key))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_owned()
        }),
        search: String::new(),
        account_auth: None,
        ambient_catalogs: Vec::new(),
        hero_catalogs: Vec::new(),
        ambient_minutes: 5,
        ambient_limit: 3,
        detail_sections: Vec::new(),
        mdblist_lists: Vec::new(),
        tracker_names: Vec::new(),
        media_servers: Vec::new(),
        server_fields: Default::default(),
        server_kind: 0,
        shortcuts: Vec::new(),
        gamepad_active: false,
        playback_summary: Vec::new(),
        shortcut_recording: None,
        badge_fields: Default::default(),
        badge_outline: false,
        badge_preview: Default::default(),
        badge_packs: Vec::new(),
        badge_custom: Vec::new(),
        formatter_rules: Vec::new(),
        formatter_presets: Vec::new(),
        layout_preview: Vec::new(),
        badge_status: None,
        formatter_status: None,
        collection_fields: Default::default(),
        collections: Vec::new(),
        collection_edit: None,
        collection_status: None,
        update: UpdateView::default(),
        watch_room: None,
        watch_code: String::new(),
        watch_ticket: Default::default(),
    }
}
