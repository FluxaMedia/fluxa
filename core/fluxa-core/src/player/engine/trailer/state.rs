use serde_json::Value;

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct TrailerState {
    pub(crate) resolutions: std::collections::HashMap<String, Value>,
    #[serde(skip)]
    pub(crate) requests: std::collections::HashMap<String, TrailerRequest>,
    #[serde(skip)]
    pub(crate) watch_config: Option<WatchConfig>,
}

#[derive(Clone, Debug)]
pub(crate) struct WatchConfig {
    pub(crate) api_key: String,
    pub(crate) visitor_data: Option<String>,
    pub(crate) player_script_url: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct TrailerRequest {
    pub(crate) video_id: String,
    pub(crate) max_height: Option<u32>,
    pub(crate) player_response: Option<Value>,
}
