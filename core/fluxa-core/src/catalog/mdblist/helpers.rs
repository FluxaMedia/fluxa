use serde_json::{Value, json};

const MDBLIST_API_BASE_URL: &str = "https://api.mdblist.com";
pub(crate) fn mdblist_bearer(token: &str) -> String {
    format!("Bearer {token}")
}

pub(crate) fn mdblist_device_poll_outcome(body_json: &str) -> &'static str {
    let Ok(body) = serde_json::from_str::<Value>(body_json) else {
        return "success";
    };
    match body.get("error").and_then(Value::as_str) {
        Some("authorization_pending") => "pending",
        Some("slow_down") => "slow_down",
        Some("access_denied") => "denied",
        Some("expired_token") => "expired",
        Some(_) => "error",
        None => "success",
    }
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
    let mut url = format!("{MDBLIST_API_BASE_URL}{path}");
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

pub(crate) fn extract_repeated(args: &Value, key: &str, params: &mut Vec<(String, String)>) {
    match args.get(key) {
        Some(Value::Array(values)) => {
            for value in values {
                if let Some(s) = value_to_query_string(value) {
                    params.push((key.to_string(), s));
                }
            }
        }
        Some(other) => {
            if let Some(s) = value_to_query_string(other) {
                params.push((key.to_string(), s));
            }
        }
        None => {}
    }
}
