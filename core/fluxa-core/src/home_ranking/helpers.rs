use serde_json::Value;

pub(super) fn meta_text<'a>(meta: &'a Value, key: &str) -> &'a str {
    meta.get(key).and_then(Value::as_str).unwrap_or("")
}

pub(super) fn meta_i64(meta: &Value, key: &str) -> Option<i64> {
    meta.get(key).and_then(Value::as_i64)
}

pub(super) fn meta_string_array(meta: &Value, key: &str) -> Vec<String> {
    meta.get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
