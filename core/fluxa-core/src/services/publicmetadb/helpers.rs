use serde_json::{Map, Value, json};

const PUBLICMETADB_API_BASE_URL: &str = "https://publicmetadb.com/api/external";

pub(crate) fn publicmetadb_bearer(api_key: &str) -> String {
    format!("Bearer {api_key}")
}

pub(crate) fn encode_query(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

pub(crate) fn build_url(path: &str, params: &[(String, String)]) -> String {
    let mut url = format!("{PUBLICMETADB_API_BASE_URL}{path}");
    let mut first = true;
    for (key, value) in params {
        if value.is_empty() {
            continue;
        }
        url.push(if first { '?' } else { '&' });
        first = false;
        url.push_str(key);
        url.push('=');
        url.push_str(&encode_query(value));
    }
    url
}

pub(crate) fn plan(method: &str, url: String, body: Option<Value>) -> Option<String> {
    serde_json::to_string(&json!({ "method": method, "url": url, "body": body })).ok()
}

pub(crate) fn value_to_query_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

pub(crate) fn extract_query(args: &Value, keys: &[&str]) -> Vec<(String, String)> {
    keys.iter()
        .filter_map(|&key| {
            let raw = args.get(key)?;
            Some((key.to_string(), value_to_query_string(raw)?))
        })
        .collect()
}

pub(crate) fn parse_args(args_json: &str) -> Value {
    serde_json::from_str(args_json).unwrap_or(json!({}))
}

// Copies `required` keys verbatim (failing the whole body if any is absent)
// then any present `optional` keys — including explicit JSON `null`, which
// several endpoints here use on purpose (e.g. watched_at: null for "unknown
// date"), so this can't be a plain `Value::is_null` filter.
pub(crate) fn body_from_keys(args: &Value, required: &[&str], optional: &[&str]) -> Option<Value> {
    let mut body = Map::new();
    for &key in required {
        body.insert(key.to_string(), args.get(key)?.clone());
    }
    for &key in optional {
        if let Some(value) = args.get(key) {
            body.insert(key.to_string(), value.clone());
        }
    }
    Some(Value::Object(body))
}
