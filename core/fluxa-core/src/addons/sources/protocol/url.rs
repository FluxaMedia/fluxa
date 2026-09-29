use serde_json::{Map, Value, json};

pub(crate) fn is_http_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

pub(crate) fn strip_http_scheme(value: &str) -> &str {
    value
        .strip_prefix("http://")
        .or_else(|| value.strip_prefix("https://"))
        .or_else(|| value.strip_prefix("HTTP://"))
        .or_else(|| value.strip_prefix("HTTPS://"))
        .unwrap_or(value)
}

pub(crate) fn is_ipv4_like_host(value: &str) -> bool {
    let host = value
        .split('/')
        .next()
        .unwrap_or(value)
        .split(':')
        .next()
        .unwrap_or(value);
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4 && parts.iter().all(|part| part.parse::<u8>().is_ok())
}

pub(crate) fn is_local_url(value: &str) -> bool {
    let lower = strip_http_scheme(value).to_ascii_lowercase();
    if lower.starts_with("localhost")
        || lower.starts_with("127.")
        || lower.starts_with("10.")
        || lower.starts_with("192.168.")
    {
        return true;
    }
    // 172.16.0.0/12 private range
    if let Some(rest) = lower.strip_prefix("172.")
        && let Some(second_octet) = rest.split('.').next().and_then(|s| s.parse::<u8>().ok())
        && (16..=31).contains(&second_octet)
    {
        return true;
    }
    false
}

pub(crate) fn normalize_manifest_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lower = trimmed.to_ascii_lowercase();
    let with_scheme = if lower.starts_with("stremio://") {
        format!("https://{}", &trimmed[10..])
    } else if lower.starts_with("http://") {
        if is_local_url(trimmed) {
            trimmed.to_string()
        } else {
            format!("https://{}", &trimmed[7..])
        }
    } else if lower.starts_with("https://") {
        trimmed.to_string()
    } else if lower.starts_with("127.0.0.1")
        || lower.starts_with("10.0.2.2")
        || lower.starts_with("localhost")
        || is_ipv4_like_host(trimmed)
    {
        format!("http://{trimmed}")
    } else {
        format!("https://{trimmed}")
    };

    if with_scheme.to_ascii_lowercase().ends_with("manifest.json") {
        with_scheme
    } else if with_scheme.ends_with('/') {
        format!("{with_scheme}manifest.json")
    } else {
        format!("{with_scheme}/manifest.json")
    }
}

pub(crate) fn identity(raw: &str) -> String {
    normalize_manifest_url(raw)
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string()
}

pub(crate) fn manifest_candidates(raw: &str) -> Vec<String> {
    let normalized = normalize_manifest_url(raw);
    let mut values = vec![normalized.clone()];
    if is_local_url(&normalized) && normalized.to_ascii_lowercase().starts_with("https://") {
        let fallback = format!("http://{}", &normalized[8..]);
        if !values.contains(&fallback) {
            values.push(fallback);
        }
    }
    values
}

pub(crate) fn manifest_fetch_plan_json(raw: &str) -> Option<String> {
    let normalized_transport_url = normalize_manifest_url(raw);
    if normalized_transport_url.is_empty() {
        return None;
    }
    serde_json::to_string(&json!({
        "normalizedTransportUrl": normalized_transport_url,
        "cacheKey": format!("manifest_v10_{}", normalized_transport_url),
        "candidateUrls": manifest_candidates(&normalized_transport_url)
    }))
    .ok()
}

pub(crate) fn base_url(raw: &str) -> String {
    let normalized = normalize_manifest_url(raw);
    let without_manifest = match normalized.to_ascii_lowercase().rfind("manifest.json") {
        Some(index) => normalized[..index].to_string(),
        None => normalized,
    };
    let mut base = if without_manifest.ends_with('/') {
        without_manifest
    } else {
        format!("{without_manifest}/")
    };
    let lower = base.to_ascii_lowercase();
    if (lower.contains("localhost") || lower.contains("127.0.0.1")) && lower.starts_with("https://")
    {
        base = format!("http://{}", &base[8..]);
    }
    base
}

pub(crate) fn prefer_https_asset_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http://") {
        if is_local_url(trimmed) {
            Some(trimmed.to_string())
        } else {
            Some(format!("https://{}", &trimmed[7..]))
        }
    } else if trimmed.starts_with("//") {
        Some(format!("https:{trimmed}"))
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'*');
        if keep {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

pub(crate) fn build_resource_url(
    raw: &str,
    resource: &str,
    content_type: &str,
    id: &str,
    extra_json: Option<&str>,
) -> String {
    let extra_path = extra_json
        .and_then(|value| serde_json::from_str::<Map<String, Value>>(value).ok())
        .map(|map| {
            map.into_iter()
                .filter_map(|(key, value)| {
                    let text = value
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| value.to_string());
                    if text.trim().is_empty() {
                        None
                    } else {
                        Some(format!(
                            "{}={}",
                            encode_path_segment(&key),
                            encode_path_segment(&text)
                        ))
                    }
                })
                .collect::<Vec<_>>()
                .join("&")
        })
        .filter(|value| !value.is_empty())
        .map(|value| format!("/{value}"))
        .unwrap_or_default();
    format!(
        "{}{}/{}/{}{}.json",
        base_url(raw),
        resource,
        encode_path_segment(content_type),
        encode_path_segment(id),
        extra_path
    )
}
