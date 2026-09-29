use crate::headless_engine::helpers::{active_profile_id, normalize_error};
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod complete;
mod dispatch;

pub(crate) use complete::*;
pub(crate) use dispatch::*;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct DiscoverState {
    content_type: String,
    filters: Value,
    is_loading: bool,
    catalogs_loading: bool,
    results: Value,
    result_sources: Value,
    catalogs: Value,
    genres: Value,
    content_types: Value,
    selected_catalog_key: Option<String>,
    error: Value,
    generation: u64,
    paging: DiscoverPaging,
    #[serde(skip)]
    last_page_appended: Vec<Value>,
    #[serde(skip)]
    last_page_sources: Value,
    #[serde(skip)]
    pending_discover_after_catalogs: Option<PendingDiscoverAfterCatalogs>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct PendingDiscoverAfterCatalogs {
    filters: Value,
    profile: Value,
    language: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct DiscoverPaging {
    is_loading: bool,
    items: Value,
    error: Value,
    next_skip: i32,
    has_more: bool,
    #[serde(skip)]
    requested_skip: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiscoverPageDelta {
    pub(crate) appended: Vec<Value>,
    pub(crate) sources: Value,
    pub(crate) is_loading: bool,
    pub(crate) next_skip: i32,
    pub(crate) has_more: bool,
    pub(crate) error: Value,
}

pub(crate) fn take_page_delta(engine: &mut HeadlessEngine) -> DiscoverPageDelta {
    let discover = &mut engine.state.discover;
    DiscoverPageDelta {
        appended: std::mem::take(&mut discover.last_page_appended),
        sources: std::mem::replace(&mut discover.last_page_sources, serde_json::json!({})),
        is_loading: discover.paging.is_loading,
        next_skip: discover.paging.next_skip,
        has_more: discover.paging.has_more,
        error: discover.paging.error.clone(),
    }
}
