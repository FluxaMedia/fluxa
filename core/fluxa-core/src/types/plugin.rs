use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub scrapers: Vec<PluginManifestScraper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifestScraper {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub filename: String,
    #[serde(default = "default_supported_types")]
    pub supported_types: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub has_settings: bool,
    pub logo: Option<String>,
    pub content_language: Option<Vec<String>>,
    pub formats: Option<Vec<String>>,
}

fn default_supported_types() -> Vec<String> {
    vec!["movie".to_string(), "tv".to_string()]
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginStreamResult {
    pub title: String,
    pub name: Option<String>,
    pub url: String,
    pub quality: Option<String>,
    pub size: Option<String>,
    pub language: Option<String>,
    pub provider: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub seeders: Option<i64>,
    pub peers: Option<i64>,
    pub info_hash: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub subtitles: Option<Vec<PluginSubtitleResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSubtitleResult {
    pub url: String,
    pub language: String,
    pub name: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}
