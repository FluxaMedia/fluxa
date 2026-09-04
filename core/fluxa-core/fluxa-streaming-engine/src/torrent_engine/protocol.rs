use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct TorrRequest {
    pub(super) action: String,
    pub(super) link: Option<String>,
    pub(super) hash: Option<String>,
    pub(super) title: Option<String>,
    #[serde(default)]
    pub(super) save_to_db: bool,
    pub(super) file_id: Option<usize>,
    #[serde(default)]
    pub(super) role: FileRole,
    #[serde(default)]
    pub(super) prewarm: bool,
}

#[derive(Deserialize)]
pub(super) struct TorrSettings {
    #[serde(rename = "PreloadSize")]
    pub(super) preload_size: Option<u64>,
    #[serde(rename = "CacheLimitMb")]
    pub(super) cache_limit_mb: Option<u64>,
    #[serde(rename = "StreamBufferBytes")]
    pub(super) stream_buffer_bytes: Option<u64>,
    #[serde(alias = "deviceBudget", alias = "DeviceBudget")]
    pub(super) device_budget: Option<DeviceBudgetSettings>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DeviceBudgetSettings {
    pub(super) torrent_preload_mb: Option<u64>,
    pub(super) torrent_cache_mb: Option<u64>,
    pub(super) stream_reader_buffer_bytes: Option<u64>,
}

#[derive(Deserialize)]
pub(super) struct StreamQuery {
    pub(super) link: String,
    pub(super) title: Option<String>,
    pub(super) index: Option<usize>,
    pub(super) stat: Option<String>,
    pub(super) access_token: Option<String>,
    #[serde(default)]
    pub(super) role: FileRole,
    #[serde(alias = "durationMs")]
    pub(super) duration_ms: Option<u64>,
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum FileRole {
    #[default]
    Video,
    Subtitle,
    Auxiliary,
}
