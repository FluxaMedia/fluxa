
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

fn replace_ascii_prefix(value: &str, prefix: &str, replacement: &str) -> String {
    if value.len() >= prefix.len() && value[..prefix.len()].eq_ignore_ascii_case(prefix) {
        format!("{replacement}{}", &value[prefix.len()..])
    } else {
        value.to_string()
    }
}
