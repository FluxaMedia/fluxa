use serde_json::Value;

fn simkl_item_key(value: &Value) -> Option<String> {
    let obj = value.as_object()?;
    let ids = obj.get("ids").and_then(Value::as_object);
    let candidate = ids
        .and_then(|ids| {
            ids.get("simkl")
                .or_else(|| ids.get("imdb"))
                .or_else(|| ids.get("tmdb"))
        })
        .or_else(|| obj.get("id"))?;
    match candidate {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn simkl_merge_delta_list(previous: &[Value], changes: &[Value]) -> Vec<Value> {
    let updates: std::collections::HashMap<String, &Value> = changes
        .iter()
        .filter_map(|item| simkl_item_key(item).map(|key| (key, item)))
        .collect();
    let mut merged: Vec<Value> = previous
        .iter()
        .map(|item| {
            simkl_item_key(item)
                .and_then(|key| updates.get(&key))
                .map(|updated| (*updated).clone())
                .unwrap_or_else(|| item.clone())
        })
        .collect();
    let existing: std::collections::HashSet<String> =
        previous.iter().filter_map(simkl_item_key).collect();
    for item in changes {
        let key = simkl_item_key(item);
        let already_present = key.as_ref().is_some_and(|k| existing.contains(k));
        if !already_present {
            merged.push(item.clone());
        }
    }
    merged
}

fn simkl_merge_delta(previous: &Value, changes: &Value) -> Value {
    match (previous.as_array(), changes.as_array()) {
        (Some(prev_arr), Some(changes_arr)) => {
            Value::Array(simkl_merge_delta_list(prev_arr, changes_arr))
        }
        _ => changes.clone(),
    }
}

fn simkl_merge_resource(previous: &Value, changes: &Value) -> Value {
    let previous_is_container = previous.is_object() || previous.is_array();
    let changes_is_container = changes.is_object() || changes.is_array();
    if !previous_is_container || !changes_is_container {
        return changes.clone();
    }
    if previous.is_array() || changes.is_array() {
        return simkl_merge_delta(previous, changes);
    }
    let mut merged = previous.as_object().cloned().unwrap_or_default();
    if let Some(changes_obj) = changes.as_object() {
        for (key, value) in changes_obj {
            let current = merged.get(key).cloned().unwrap_or(Value::Null);
            merged.insert(key.clone(), simkl_merge_resource(&current, value));
        }
    }
    Value::Object(merged)
}

pub(crate) fn simkl_merge_delta_json(previous_json: &str, changes_json: &str) -> Option<String> {
    let previous: Value = serde_json::from_str(previous_json).unwrap_or(Value::Null);
    let changes: Value = serde_json::from_str(changes_json).unwrap_or(Value::Null);
    serde_json::to_string(&simkl_merge_resource(&previous, &changes)).ok()
}
