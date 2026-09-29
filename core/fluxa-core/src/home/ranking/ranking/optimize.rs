use super::*;

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
pub(super) fn sorted_unpinned_candidates(request: &HomeOptimizeRequest) -> Vec<NativeHomeCategory> {
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
pub(super) fn select_diverse_categories(
    candidates: &[NativeHomeCategory],
) -> Vec<NativeHomeCategory> {
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
pub(super) fn fallback_categories(
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

pub(super) fn output_pinned_count(categories: &[NativeHomeCategory]) -> usize {
    categories
        .iter()
        .filter(|category| is_pinned(category))
        .count()
}

pub(super) fn distinct_categories<I>(categories: I) -> Vec<NativeHomeCategory>
where
    I: IntoIterator<Item = NativeHomeCategory>,
{
    let mut seen = HashSet::new();
    categories
        .into_iter()
        .filter(|category| seen.insert(category.id.clone()))
        .collect()
}
