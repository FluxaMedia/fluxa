use regex::Regex;
use std::sync::OnceLock;

pub(crate) fn format_runtime_label(raw: Option<&str>) -> Option<String> {
    let value = raw?.trim();
    if value.is_empty() {
        return None;
    }
    let normalized = value
        .replace('·', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if let Some(captures) = runtime_regex().captures(&normalized) {
        let hours = captures
            .get(1)
            .and_then(|value| value.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        let minutes = captures
            .get(2)
            .and_then(|value| value.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        if hours > 0 || minutes > 0 {
            return Some(build_runtime_label(hours, minutes));
        }
    }
    Some(value.to_string())
}

fn runtime_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"^(?:(\d+)\s*h(?:ours?)?\s*)?(?:(\d+)\s*(?:m|min|mins|minute|minutes))?$")
            .expect("runtime label regex is valid")
    })
}

fn build_runtime_label(hours: u64, minutes: u64) -> String {
    match (hours, minutes) {
        (hours, minutes) if hours > 0 && minutes > 0 => format!("{hours}h {minutes}min"),
        (hours, _) if hours > 0 => format!("{hours}h"),
        (_, minutes) => format!("{minutes}min"),
    }
}

#[cfg(test)]
mod tests {
    use super::format_runtime_label;

    #[test]
    fn normalizes_supported_runtime_shapes() {
        assert_eq!(
            format_runtime_label(Some("1h 42min")),
            Some("1h 42min".into())
        );
        assert_eq!(
            format_runtime_label(Some("90 minutes")),
            Some("90min".into())
        );
        assert_eq!(format_runtime_label(Some("2 hours")), Some("2h".into()));
        assert_eq!(
            format_runtime_label(Some("  unknown ")),
            Some("unknown".into())
        );
        assert_eq!(format_runtime_label(Some(" ")), None);
    }
}
