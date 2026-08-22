use serde_json::Value;

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(in crate::headless_engine) struct TrailerState {
    pub(super) resolutions: std::collections::HashMap<String, Value>,
    #[serde(skip)]
    pub(super) requests: std::collections::HashMap<String, TrailerRequest>,
    #[serde(skip)]
    pub(super) watch_config: Option<WatchConfig>,
}

#[derive(Clone, Debug)]
pub(super) struct WatchConfig {
    pub(super) api_key: String,
    pub(super) visitor_data: Option<String>,
    pub(super) player_script_url: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct TrailerRequest {
    pub(super) video_id: String,
    pub(super) max_height: Option<u32>,
    pub(super) player_response: Option<Value>,
}
