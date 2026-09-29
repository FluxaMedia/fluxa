use super::*;

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

pub(super) fn semantic_score(category: &NativeHomeCategory, item: &Value) -> i32 {
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

pub(super) fn curated_items(category: &NativeHomeCategory) -> Vec<Value> {
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
