use serde_json::{Value, json};

pub(crate) fn normalize_library_document_json(json: &str) -> String {
    let lib = serde_json::from_str(json).unwrap_or(Value::Null);
    serde_json::to_string(&normalize_library_document(&lib)).unwrap_or_else(|_| "{}".to_string())
}

pub(crate) fn normalize_library_document(lib: &Value) -> Value {
    let mut lib = lib.as_object().cloned().unwrap_or_default();
    lib.insert("schemaVersion".to_string(), json!(2));
    if !lib.get("watchlist").map(Value::is_array).unwrap_or(false) {
        lib.insert("watchlist".to_string(), json!([]));
    }
    if !lib.get("liked").map(Value::is_array).unwrap_or(false) {
        lib.insert("liked".to_string(), json!([]));
    }
    if !lib.get("history").map(Value::is_array).unwrap_or(false) {
        lib.insert("history".to_string(), json!([]));
    }
    if !lib
        .get("continueWatching")
        .map(Value::is_array)
        .unwrap_or(false)
    {
        lib.insert("continueWatching".to_string(), json!([]));
    }
    if !lib
        .get("progress")
        .map(|v| v.is_object() && !v.is_array())
        .unwrap_or(false)
    {
        lib.insert("progress".to_string(), json!({}));
    }
    if !lib
        .get("watched")
        .map(|v| v.is_object() && !v.is_array())
        .unwrap_or(false)
    {
        lib.insert("watched".to_string(), json!({}));
    }
    if !lib.get("dropped").map(Value::is_array).unwrap_or(false) {
        lib.insert("dropped".to_string(), json!([]));
    }
    if !lib.get("completed").map(Value::is_array).unwrap_or(false) {
        lib.insert("completed".to_string(), json!([]));
    }
    Value::Object(lib)
}

pub(crate) fn normalize_library_read_result_json(json: &str) -> String {
    let mut result = serde_json::from_str::<Value>(json)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    for (field, aliases) in [
        ("watchlist", &["watchlist", "libraryItems"][..]),
        ("continueWatching", &["continueWatching"][..]),
        ("liked", &["liked", "favorites"][..]),
        ("dropped", &["dropped"][..]),
        ("onHold", &["onHold"][..]),
        ("completed", &["completed"][..]),
    ] {
        let value = aliases
            .iter()
            .find_map(|alias| result.get(*alias).filter(|value| value.is_array()))
            .cloned()
            .unwrap_or_else(|| json!([]));
        result.insert(field.to_string(), value);
    }

    let watched = result
        .get("watched")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut canonical_watched = serde_json::Map::new();
    for (key, value) in watched {
        match value {
            Value::Bool(watched) => {
                canonical_watched.insert(key, Value::Bool(watched));
            }
            Value::Array(ids) => {
                for id in ids
                    .into_iter()
                    .filter_map(|id| id.as_str().map(str::to_owned))
                {
                    canonical_watched.insert(id, Value::Bool(true));
                }
            }
            _ => {}
        }
    }
    result.insert("watched".to_string(), Value::Object(canonical_watched));
    serde_json::to_string(&Value::Object(result)).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_library_document_supplies_shared_empty_views() {
        let value: Value = serde_json::from_str(&normalize_library_document_json("{}"))
            .expect("normalized library document");
        assert_eq!(value["watchlist"], json!([]));
        assert_eq!(value["liked"], json!([]));
        assert_eq!(value["continueWatching"], json!([]));
        assert_eq!(value["progress"], json!({}));
        assert_eq!(value["schemaVersion"], 2);
    }

    #[test]
    fn library_read_result_normalizes_android_and_desktop_watched_shapes() {
        let android: Value = serde_json::from_str(&normalize_library_read_result_json(
            r#"{"watchlist":[],"continueWatching":[],"favorites":[{"id":"f"}],"watched":{"tt1":["tt1:1:2","tt1:1:3"]}}"#,
        ))
        .unwrap();
        let desktop: Value = serde_json::from_str(&normalize_library_read_result_json(
            r#"{"watchlist":[],"continueWatching":[],"liked":[{"id":"f"}],"watched":{"tt1:1:2":true,"tt1:1:3":true}}"#,
        ))
        .unwrap();

        assert_eq!(android["liked"], desktop["liked"]);
        assert_eq!(android["watched"], desktop["watched"]);
        assert_eq!(android["completed"], json!([]));
        assert_eq!(android["dropped"], json!([]));
    }
}
