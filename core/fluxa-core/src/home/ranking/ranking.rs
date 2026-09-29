use super::helpers::{meta_i64, meta_string_array, meta_text};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

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

pub(crate) fn normalize_home_key(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut last_space = false;
    for ch in value.to_lowercase().chars() {
        let normalized = match ch {
            'ç' => 'c',
            'ğ' => 'g',
            'ı' => 'i',
            'ö' => 'o',
            'ş' => 's',
            'ü' => 'u',
            ch if ch.is_ascii_alphanumeric() => ch,
            _ => ' ',
        };
        if normalized == ' ' {
            if !last_space {
                output.push(' ');
                last_space = true;
            }
        } else {
            output.push(normalized);
            last_space = false;
        }
    }
    output.trim().to_string()
}

fn semantic_score(category: &NativeHomeCategory, item: &Value) -> i32 {
    let category_keys = [
        Some(category.name.as_str()),
        Some(category_semantic_name(category)),
        category.addon_genre.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(normalize_home_key)
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
    let genre_score = meta_string_array(item, "genres")
        .into_iter()
        .map(|genre| normalize_home_key(&genre))
        .filter(|genre| {
            category_keys
                .iter()
                .any(|key| key == genre || key.contains(genre) || genre.contains(key))
        })
        .count() as i32
        * 4;
    let title_score = [meta_text(item, "name"), meta_text(item, "originalName")]
        .into_iter()
        .map(normalize_home_key)
        .filter(|title| {
            category_keys
                .iter()
                .any(|key| !key.is_empty() && title.contains(key))
        })
        .count() as i32
        * 2;
    genre_score + title_score
}

fn curated_items(category: &NativeHomeCategory) -> Vec<Value> {
    let mut values = category
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let is_adult = meta_string_array(item, "genres")
                .iter()
                .any(|genre| normalize_home_key(genre) == "adult");
            (
                index,
                semantic_score(category, item),
                meta_i64(item, "rank").unwrap_or(i64::MAX),
                meta_text(item, "imdbRating").parse::<f32>().unwrap_or(0.0),
                is_adult,
            )
        })
        .collect::<Vec<_>>();
    values.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| {
                right
                    .3
                    .partial_cmp(&left.3)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter(|(_, _, _, _, is_adult)| !is_adult)
        .filter_map(|(index, _, _, _, _)| {
            let item = &category.items[index];
            let id = meta_text(item, "id");
            if seen.insert(id) {
                Some(item.clone())
            } else {
                None
            }
        })
        .take(24)
        .collect()
}

pub(crate) fn curate_home_items_json(category_json: &str) -> Option<String> {
    let category = serde_json::from_str::<NativeHomeCategory>(category_json).ok()?;
    serde_json::to_string(&curated_items(&category)).ok()
}

fn is_pinned(category: &NativeHomeCategory) -> bool {
    category.id == "library"
        || category.id == "watchlist"
        || category.id == "continue_watching"
        || category.content_type == "collection"
        || category.content_type == "collection_folder"
}

fn priority_boost(category: &NativeHomeCategory, labels: &HomePriorityLabels) -> i32 {
    let key = normalize_home_key(category_semantic_name(category));
    if key.contains(&normalize_home_key(&labels.trending_now)) {
        40
    } else if key.contains(&normalize_home_key(&labels.popular_for_you)) {
        32
    } else if key.contains(&normalize_home_key(&labels.most_watched)) {
        28
    } else if key.contains("new") || key.contains("yeni") {
        16
    } else {
        0
    }
}

fn personalization_score(
    category: &NativeHomeCategory,
    preferred_genres: &HashMap<String, i32>,
    preferred_types: &HashMap<String, i32>,
    labels: &HomePriorityLabels,
) -> i32 {
    let type_affinity = category
        .items
        .iter()
        .map(|item| {
            preferred_types
                .get(meta_text(item, "type"))
                .copied()
                .unwrap_or(0)
        })
        .sum::<i32>()
        * 12;
    let genre_affinity = category
        .items
        .iter()
        .flat_map(|item| meta_string_array(item, "genres"))
        .map(|genre| {
            preferred_genres
                .get(&normalize_home_key(&genre))
                .copied()
                .unwrap_or(0)
        })
        .sum::<i32>()
        * 10;
    let unique_top_items = category
        .items
        .iter()
        .take(10)
        .map(|item| meta_text(item, "id").to_string())
        .collect::<HashSet<_>>()
        .len() as i32
        * 8;
    let reason_boost = category
        .items
        .iter()
        .filter(|item| !meta_text(item, "reason").is_empty())
        .count() as i32
        * 14;
    type_affinity
        + genre_affinity
        + unique_top_items
        + reason_boost
        + priority_boost(category, labels)
}

fn overlap_ratio(first: &NativeHomeCategory, second: &NativeHomeCategory) -> f32 {
    let first_ids = first
        .items
        .iter()
        .take(12)
        .map(|item| meta_text(item, "id").to_string())
        .collect::<HashSet<_>>();
    let second_ids = second
        .items
        .iter()
        .take(12)
        .map(|item| meta_text(item, "id").to_string())
        .collect::<HashSet<_>>();
    if first_ids.is_empty() || second_ids.is_empty() {
        return 0.0;
    }
    first_ids.intersection(&second_ids).count() as f32
        / first_ids.len().min(second_ids.len()) as f32
}

pub(crate) fn home_overlap_ratio_json(first_json: &str, second_json: &str) -> Option<f32> {
    let first = serde_json::from_str::<NativeHomeCategory>(first_json).ok()?;
    let second = serde_json::from_str::<NativeHomeCategory>(second_json).ok()?;
    Some(overlap_ratio(&first, &second))
}

fn is_core_genre_shelf(category: &NativeHomeCategory) -> bool {
    if category.movie_genre.is_some()
        || category.series_genre.is_some()
        || category.addon_genre.is_some()
    {
        return true;
    }
    let key = normalize_home_key(category_semantic_name(category));
    CORE_SHELF_KEYS
        .iter()
        .any(|candidate| key == *candidate || key.contains(candidate))
}

fn cluster_key(category: &NativeHomeCategory) -> Option<String> {
    if let Some(genre) = category.movie_genre.as_deref() {
        return Some(format!("movie:{}", normalize_home_key(genre)));
    }
    if let Some(genre) = category.series_genre.as_deref() {
        return Some(format!("series:{}", normalize_home_key(genre)));
    }
    if let Some(genre) = category.addon_genre.as_deref() {
        return Some(format!("addon:{}", normalize_home_key(genre)));
    }
    let key = normalize_home_key(category_semantic_name(category));
    CORE_SHELF_KEYS
        .iter()
        .find(|candidate| key == **candidate || key.contains(*candidate))
        .map(|value| (*value).to_string())
}

fn cluster_overlap_ratio(first: &NativeHomeCategory, second: &NativeHomeCategory) -> f32 {
    let Some(first_cluster) = cluster_key(first) else {
        return 0.0;
    };
    let Some(second_cluster) = cluster_key(second) else {
        return 0.0;
    };
    if first_cluster == second_cluster {
        overlap_ratio(first, second)
    } else {
        0.0
    }
}

pub(crate) fn home_personalization_score_json(
    category_json: &str,
    preferred_genres_json: &str,
    preferred_types_json: &str,
    priority_labels_json: &str,
) -> Option<i32> {
    let category = serde_json::from_str::<NativeHomeCategory>(category_json).ok()?;
    let preferred_genres =
        serde_json::from_str::<HashMap<String, i32>>(preferred_genres_json).ok()?;
    let preferred_types =
        serde_json::from_str::<HashMap<String, i32>>(preferred_types_json).ok()?;
    let labels = serde_json::from_str::<HomePriorityLabels>(priority_labels_json).ok()?;
    Some(personalization_score(
        &category,
        &preferred_genres,
        &preferred_types,
        &labels,
    ))
}

pub(crate) fn home_prioritize_rows_json(
    categories_json: &str,
    preferred_order_labels_json: &str,
    preferred_genres_json: &str,
    preferred_types_json: &str,
    priority_labels_json: &str,
) -> Option<String> {
    let categories = serde_json::from_str::<Vec<NativeHomeCategory>>(categories_json).ok()?;
    let preferred_order_labels =
        serde_json::from_str::<Vec<String>>(preferred_order_labels_json).ok()?;
    let preferred_genres =
        serde_json::from_str::<HashMap<String, i32>>(preferred_genres_json).ok()?;
    let preferred_types =
        serde_json::from_str::<HashMap<String, i32>>(preferred_types_json).ok()?;
    let labels = serde_json::from_str::<HomePriorityLabels>(priority_labels_json).ok()?;
    let preferred_order = preferred_order_labels
        .iter()
        .map(|value| normalize_home_key(value))
        .collect::<Vec<_>>();
    let preferred_indexes =
        preferred_order
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut indexes, (index, key)| {
                indexes.entry(key.as_str()).or_insert(index);
                indexes
            });
    let mut ranked = categories
        .into_iter()
        .map(|category| {
            let normalized = normalize_home_key(category_semantic_name(&category));
            let preferred_index = preferred_indexes
                .get(normalized.as_str())
                .copied()
                .unwrap_or(usize::MAX);
            let score =
                personalization_score(&category, &preferred_genres, &preferred_types, &labels);
            (preferred_index, score, category)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
    let categories = ranked
        .into_iter()
        .map(|(_, _, category)| category)
        .collect::<Vec<_>>();
    serde_json::to_string(&categories).ok()
}

pub(crate) fn optimize_home_rows_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<HomeOptimizeRequest>(request_json).ok()?;
    if request.categories.is_empty() {
        return Some("[]".to_string());
    }
    let pinned = distinct_categories(
        request
            .categories
            .iter()
            .filter(|category| is_pinned(category))
            .cloned(),
    );
    let candidates = sorted_unpinned_candidates(&request);
    let kept = select_diverse_categories(&candidates);
    let fallback = fallback_categories(candidates, &kept);

    let mut output = pinned;
    output.extend(kept);
    output.extend(fallback);
    let limit = 24 + output_pinned_count(&output);
    let output = distinct_categories(output)
        .into_iter()
        .take(limit)
        .collect::<Vec<_>>();
    serde_json::to_string(&output).ok()
}

// Unpinned categories, curated down to their top items and sorted by the
// caller's preferred order first, personalization score second.
fn sorted_unpinned_candidates(request: &HomeOptimizeRequest) -> Vec<NativeHomeCategory> {
    let candidates = distinct_categories(
        request
            .categories
            .iter()
            .filter(|category| !is_pinned(category))
            .cloned(),
    )
    .into_iter()
    .map(|mut category| {
        category.items = curated_items(&category);
        category
    })
    .filter(|category| category.items.len() >= 4)
    .collect::<Vec<_>>();
    let preferred_order = request
        .preferred_order_labels
        .iter()
        .map(|value| normalize_home_key(value))
        .collect::<Vec<_>>();
    let preferred_indexes =
        preferred_order
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut indexes, (index, key)| {
                indexes.entry(key.as_str()).or_insert(index);
                indexes
            });
    let mut ranked = candidates
        .into_iter()
        .map(|category| {
            let normalized = normalize_home_key(category_semantic_name(&category));
            let preferred_index = preferred_indexes
                .get(normalized.as_str())
                .copied()
                .unwrap_or(usize::MAX);
            let score = personalization_score(
                &category,
                &request.preferred_genres,
                &request.preferred_types,
                &request.priority_labels,
            );
            (preferred_index, score, category)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)));
    ranked
        .into_iter()
        .map(|(_, _, category)| category)
        .collect()
}

// Greedily keep candidates that are either a core genre shelf or don't overlap
// too much with what's already kept, so the final list isn't redundant.
fn select_diverse_categories(candidates: &[NativeHomeCategory]) -> Vec<NativeHomeCategory> {
    let mut kept = Vec::<NativeHomeCategory>::new();
    for category in candidates.iter() {
        let overlap = kept
            .iter()
            .map(|existing| overlap_ratio(existing, category))
            .fold(0.0, f32::max);
        let cluster_overlap = kept
            .iter()
            .map(|existing| cluster_overlap_ratio(existing, category))
            .fold(0.0, f32::max);
        let min_unique = category
            .items
            .iter()
            .take(12)
            .map(|item| meta_text(item, "id").to_string())
            .collect::<HashSet<_>>()
            .len();
        if min_unique < 5 {
            continue;
        }
        if is_core_genre_shelf(category)
            || (overlap < 0.68 && cluster_overlap < 0.52)
            || kept.len() < 8
        {
            kept.push(category.clone());
        }
    }
    kept
}

// Fill remaining slots (up to 24 total) from leftover candidates that still
// don't overlap too much with anything already kept.
fn fallback_categories(
    candidates: Vec<NativeHomeCategory>,
    kept: &[NativeHomeCategory],
) -> Vec<NativeHomeCategory> {
    candidates
        .into_iter()
        .filter(|candidate| {
            kept.iter().all(|existing| existing.id != candidate.id)
                && kept.iter().all(|existing| {
                    overlap_ratio(existing, candidate) < 0.68
                        && cluster_overlap_ratio(existing, candidate) < 0.52
                })
        })
        .take(24usize.saturating_sub(kept.len()))
        .collect::<Vec<_>>()
}

fn output_pinned_count(categories: &[NativeHomeCategory]) -> usize {
    categories
        .iter()
        .filter(|category| is_pinned(category))
        .count()
}

fn distinct_categories<I>(categories: I) -> Vec<NativeHomeCategory>
where
    I: IntoIterator<Item = NativeHomeCategory>,
{
    let mut seen = HashSet::new();
    categories
        .into_iter()
        .filter(|category| seen.insert(category.id.clone()))
        .collect()
}
