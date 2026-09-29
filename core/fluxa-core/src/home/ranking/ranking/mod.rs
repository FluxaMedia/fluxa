use super::helpers::{meta_i64, meta_string_array, meta_text};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

mod curation;
mod optimize;
mod scoring;

pub(crate) use curation::*;
pub(crate) use optimize::*;
pub(crate) use scoring::*;

const CORE_SHELF_KEYS: &[&str] = &[
    "action",
    "adventure",
    "aksiyon",
    "macera",
    "sci fi",
    "science fiction",
    "bilim kurgu",
    "fantasy",
    "fantastik",
    "thriller",
    "gerilim",
    "crime",
    "suc",
    "comedy",
    "komedi",
    "drama",
    "dram",
    "family",
    "aile",
    "kids",
    "cocuk",
    "anime",
    "mini series",
    "mini dizi",
];

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NativeHomeCategory {
    name: String,
    items: Vec<Value>,
    id: String,
    #[serde(rename = "type")]
    content_type: String,
    semantic_name: Option<String>,
    movie_genre: Option<String>,
    series_genre: Option<String>,
    skip: Option<i32>,
    can_load_more: Option<bool>,
    catalog_id: Option<String>,
    addon_transport_url: Option<String>,
    addon_genre: Option<String>,
    catalog_sources: Option<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HomeOptimizeRequest {
    categories: Vec<NativeHomeCategory>,
    preferred_order_labels: Vec<String>,
    preferred_genres: HashMap<String, i32>,
    preferred_types: HashMap<String, i32>,
    priority_labels: HomePriorityLabels,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HomePriorityLabels {
    trending_now: String,
    popular_for_you: String,
    most_watched: String,
}

fn category_semantic_name(category: &NativeHomeCategory) -> &str {
    category
        .semantic_name
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or(&category.name)
}
