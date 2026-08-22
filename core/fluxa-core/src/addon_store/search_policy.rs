use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::OnceLock;

#[derive(Deserialize)]
struct SearchPolicyRequest {
    query: String,
    #[serde(rename = "nowMillis")]
    now_millis: i64,
    #[serde(rename = "cachedAtMillis")]
    cached_at_millis: Option<i64>,
    #[serde(rename = "ttlMillis")]
    ttl_millis: i64,
}

pub(crate) fn addon_store_search_policy_json(request_json: &str) -> Option<String> {
    let request: SearchPolicyRequest = serde_json::from_str(request_json).ok()?;
    let normalized_query = request.query.trim().to_ascii_lowercase();
    if normalized_query.len() < 2 {
        return serde_json::to_string(&json!({
            "normalizedQuery": normalized_query,
            "url": "",
            "useCache": false,
            "shouldFetch": false
        }))
        .ok();
    }
    let use_cache = request
        .cached_at_millis
        .and_then(|cached_at| request.now_millis.checked_sub(cached_at))
        .map(|elapsed| elapsed <= request.ttl_millis)
        .unwrap_or(false);
    serde_json::to_string(&json!({
        "normalizedQuery": normalized_query,
        "url": format!(
            "https://stremio-addons.net/addons?query={}",
            form_urlencode(&normalized_query)
        ),
        "useCache": use_cache,
        "shouldFetch": !use_cache
    }))
    .ok()
}

fn addon_key(addon: &Value) -> String {
    addon
        .get("transportUrl")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            addon
                .get("manifest")
                .and_then(|manifest| manifest.get("id"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        })
}

pub(crate) fn filter_enabled_addons_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let addons = args.get("addons")?.as_array()?.clone();
    let disabled_keys: Vec<&str> = args
        .get("disabledKeys")?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let filtered: Vec<Value> = addons
        .into_iter()
        .filter(|addon| !disabled_keys.contains(&addon_key(addon).as_str()))
        .collect();
    serde_json::to_string(&filtered).ok()
}

#[expect(
    clippy::expect_used,
    reason = "static literal regex is validated at build review and cannot depend on input"
)]
fn manifest_url_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"https?://[^"'\\ ]+manifest\.json[^"'\\ ]*"#)
            .expect("valid manifest url regex")
    })
}

pub(crate) fn extract_addon_manifest_url(detail_text: &str) -> Option<String> {
    let unescaped_text = detail_text.replace("\\/", "/").replace("\\u0026", "&");
    manifest_url_regex()
        .find(&unescaped_text)
        .map(|match_| match_.as_str().trim_end_matches('\\').to_string())
}

fn form_urlencode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'*');
        if keep {
            encoded.push(byte as char);
        } else if byte == b' ' {
            encoded.push('+');
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}
