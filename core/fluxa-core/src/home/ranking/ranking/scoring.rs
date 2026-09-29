use super::*;

pub(super) fn is_pinned(category: &NativeHomeCategory) -> bool {
    category.id == "library"
        || category.id == "watchlist"
        || category.id == "continue_watching"
        || category.content_type == "collection"
        || category.content_type == "collection_folder"
}

pub(super) fn priority_boost(category: &NativeHomeCategory, labels: &HomePriorityLabels) -> i32 {
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

pub(super) fn personalization_score(
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

pub(super) fn overlap_ratio(first: &NativeHomeCategory, second: &NativeHomeCategory) -> f32 {
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

pub(super) fn is_core_genre_shelf(category: &NativeHomeCategory) -> bool {
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

pub(super) fn cluster_key(category: &NativeHomeCategory) -> Option<String> {
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

pub(super) fn cluster_overlap_ratio(
    first: &NativeHomeCategory,
    second: &NativeHomeCategory,
) -> f32 {
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
