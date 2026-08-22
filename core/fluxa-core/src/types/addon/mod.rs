use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub transport_url: String,
    pub resources: Vec<ResourceDeclaration>,
    pub types: Vec<String>,
    pub id_prefixes: Option<Vec<String>>,
    pub catalogs: Vec<CatalogDeclaration>,
    pub addon_catalogs: Option<Vec<CatalogDeclaration>>,
    pub config: Option<Vec<ConfigDeclaration>>,
    pub background: Option<String>,
    pub logo: Option<String>,
    pub contact_email: Option<String>,
    pub behavior_hints: Option<BehaviorHints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ResourceDeclaration {
    Name(String),
    Object {
        name: String,
        types: Option<Vec<String>>,
        id_prefixes: Option<Vec<String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDeclaration {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub name: Option<String>,
    pub extra: Option<Vec<ExtraDeclaration>>,
    pub extra_supported: Option<Vec<String>>,
    pub extra_required: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraDeclaration {
    pub name: String,
    pub is_required: Option<bool>,
    pub options: Option<Vec<String>>,
    pub options_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDeclaration {
    pub key: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub default: Option<Value>,
    pub title: Option<String>,
    pub options: Option<Vec<String>>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorHints {
    pub adult: Option<bool>,
    pub p2p: Option<bool>,
    pub configurable: Option<bool>,
    pub configuration_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRef {
    pub resource: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub extra: Option<HashMap<String, String>>,
}
