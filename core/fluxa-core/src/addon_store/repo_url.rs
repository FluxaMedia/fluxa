pub(crate) fn addon_store_input_type(raw: &str) -> &'static str {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "unknown";
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("manifest.json") {
        return "stremio_manifest";
    }
    if lower.starts_with("cloudstreamrepo://")
        || lower.starts_with("cloudstream://")
        || lower.contains("cloudstream")
        || lower.contains(".cs3")
        || lower.contains("repo.json")
        || ((lower.starts_with("http://") || lower.starts_with("https://")) && trimmed.len() > 20)
    {
        return "cloudstream_repo";
    }

    "search_query"
}

pub(crate) fn normalize_cloudstream_repo_url(raw: &str) -> String {
    let trimmed = raw.trim();
    replace_ascii_prefix(
        &replace_ascii_prefix(trimmed, "cloudstreamrepo://", "https://"),
        "cloudstream://",
        "https://",
    )
}

pub(crate) fn normalize_plugin_repository_url(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some(scheme_end) = trimmed.find("://") else {
        return trimmed.to_string();
    };
    let scheme = trimmed[..scheme_end].to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return format!("https://{}", &trimmed[scheme_end + 3..]);
    }
    replace_ascii_prefix(trimmed, "http://", "https://")
}

pub(crate) fn is_secure_remote_url(raw: &str) -> bool {
    let trimmed = raw.trim();
    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("https://") {
        return false;
    }
    let host = trimmed["https://".len()..]
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .trim();
    !host.is_empty() && !host.contains(char::is_whitespace)
}

pub(crate) fn same_plugin_repository_url(left: &str, right: &str) -> bool {
    canonical_url_for_compare(left) == canonical_url_for_compare(right)
}

fn canonical_url_for_compare(raw: &str) -> String {
    let trimmed = raw.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("https://") {
        format!("https://{}", rest.trim_end_matches('/'))
    } else if let Some(rest) = lower.strip_prefix("http://") {
        format!("https://{}", rest.trim_end_matches('/'))
    } else {
        lower.trim_end_matches('/').to_string()
    }
}

fn replace_ascii_prefix(value: &str, prefix: &str, replacement: &str) -> String {
    if value.len() >= prefix.len() && value[..prefix.len()].eq_ignore_ascii_case(prefix) {
        format!("{replacement}{}", &value[prefix.len()..])
    } else {
        value.to_string()
    }
}
