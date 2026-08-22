use super::text::{json_string_list, parse_string_list};

pub(crate) fn effective_metadata_feed_selection_json(
    selected_keys_json: &str,
    available_keys_json: &str,
) -> Option<String> {
    let selected = serde_json::from_str::<Option<Vec<String>>>(selected_keys_json)
        .ok()
        .flatten()?;
    let available = parse_string_list(available_keys_json);
    let filtered = selected
        .into_iter()
        .filter(|key| available.contains(key))
        .collect::<Vec<_>>();
    json_string_list(&filtered)
}

pub(crate) fn toggle_metadata_feed_json(
    selected_keys_json: &str,
    available_keys_json: &str,
    key: &str,
) -> Option<String> {
    let selected = serde_json::from_str::<Option<Vec<String>>>(selected_keys_json)
        .ok()
        .flatten();
    let current = selected.unwrap_or_else(|| parse_string_list(available_keys_json));
    let mut output = Vec::<String>::new();
    let mut contains = false;
    for item in current {
        if item == key {
            contains = true;
        } else if !output.contains(&item) {
            output.push(item);
        }
    }
    if !contains {
        output.push(key.to_string());
    }
    json_string_list(&output)
}

pub(crate) fn toggle_metadata_feed_limited_json(
    selected_keys_json: &str,
    available_keys_json: &str,
    key: &str,
    max_enabled: i32,
) -> Option<String> {
    let current = serde_json::from_str::<Option<Vec<String>>>(selected_keys_json)
        .ok()
        .flatten()
        .unwrap_or_else(|| parse_string_list(available_keys_json));
    let output: Vec<String> = if current.iter().any(|item| item == key) {
        current.into_iter().filter(|item| item != key).collect()
    } else {
        let mut appended = current;
        appended.push(key.to_string());
        let keep = max_enabled.max(0) as usize;
        appended
            .into_iter()
            .rev()
            .take(keep)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    };
    json_string_list(&output)
}

pub(crate) fn set_metadata_feed_group_enabled_json(
    selected_keys_json: &str,
    available_keys_json: &str,
    group_keys_json: &str,
    enabled: bool,
) -> Option<String> {
    let current = serde_json::from_str::<Option<Vec<String>>>(selected_keys_json)
        .ok()
        .flatten()
        .unwrap_or_else(|| parse_string_list(available_keys_json));
    let group = parse_string_list(group_keys_json);
    let mut output = Vec::<String>::new();
    for item in current {
        if (enabled || !group.contains(&item)) && !output.contains(&item) {
            output.push(item);
        }
    }
    if enabled {
        for item in group {
            if !output.contains(&item) {
                output.push(item);
            }
        }
    }
    json_string_list(&output)
}

pub(crate) fn ordered_metadata_feed_keys(
    option_keys_json: &str,
    order_json: &str,
) -> Option<String> {
    let option_keys = parse_string_list(option_keys_json);
    let order = parse_string_list(order_json);
    let mut output = Vec::<String>::new();
    for key in order {
        if option_keys.contains(&key) && !output.contains(&key) {
            output.push(key);
        }
    }
    for key in option_keys {
        if !output.contains(&key) {
            output.push(key);
        }
    }
    json_string_list(&output)
}

pub(crate) fn move_metadata_feed_order_json(
    option_keys_json: &str,
    current_order_json: &str,
    key: &str,
    delta: i32,
) -> Option<String> {
    let ordered_json = ordered_metadata_feed_keys(option_keys_json, current_order_json)?;
    let mut keys = parse_string_list(&ordered_json);
    let Some(from) = keys.iter().position(|item| item == key) else {
        return json_string_list(&keys);
    };
    if keys.is_empty() {
        return json_string_list(&keys);
    }
    let to = (from as i32 + delta).clamp(0, keys.len() as i32 - 1) as usize;
    if from != to {
        let moved = keys.remove(from);
        keys.insert(to, moved);
    }
    json_string_list(&keys)
}
