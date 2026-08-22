// pub rather than pub(crate): re-exported under fuzz_targets for the `fuzz/`
// crate (see lib.rs). Not part of the supported public API otherwise.
#[expect(
    clippy::indexing_slicing,
    reason = "raw-byte bounds are checked before percent-decoding reads"
)]
pub fn percent_decode_component(value: &str) -> String {
    let mut bytes = Vec::with_capacity(value.len());
    let raw = value.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        // Decode the two hex digits as raw bytes rather than slicing `value` —
        // a `%` next to a multi-byte UTF-8 character can put the slice bound
        // mid-character, which panics; byte-at-a-time reads can't.
        if raw[index] == b'%' && index + 2 < raw.len() {
            let hi = (raw[index + 1] as char).to_digit(16);
            let lo = (raw[index + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                bytes.push((hi * 16 + lo) as u8);
                index += 3;
                continue;
            }
        }
        bytes.push(raw[index]);
        index += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

pub(crate) fn form_decode(value: &str) -> String {
    percent_decode_component(&value.replace('+', " "))
}

pub(crate) fn stable_feed_part(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut replaced = false;
    for ch in value.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
            output.push(ch);
            replaced = false;
        } else if !replaced {
            output.push('_');
            replaced = true;
        }
    }
    output.trim_matches('_').to_string()
}

pub(crate) fn normalize_content_type(value: &str) -> Option<&'static str> {
    match value.to_lowercase().as_str() {
        "movie" | "movies" => Some("movie"),
        "series" | "tv" | "show" | "shows" => Some("series"),
        "anime" => Some("anime"),
        _ => None,
    }
}

pub(crate) fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn normalize_provider_search_text(value: &str) -> String {
    collapse_whitespace(
        &value
            .to_lowercase()
            .replace(['+', '-', '_'], " ")
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == ' ' {
                    ch
                } else {
                    ' '
                }
            })
            .collect::<String>(),
    )
}

pub(crate) fn provider_search_terms(provider: &str) -> Vec<String> {
    match provider.trim().to_lowercase().as_str() {
        "8" => vec!["netflix".to_string()],
        "9" => vec!["prime".to_string(), "amazon".to_string()],
        "337" => vec!["disney".to_string()],
        "49" => vec!["hbo".to_string(), "max".to_string()],
        "350" => vec!["apple".to_string()],
        _ => {
            let normalized = normalize_provider_search_text(provider);
            if normalized.is_empty() {
                Vec::new()
            } else {
                vec![normalized]
            }
        }
    }
}

pub(crate) fn parse_string_list(json: &str) -> Vec<String> {
    serde_json::from_str::<Option<Vec<String>>>(json)
        .ok()
        .flatten()
        .unwrap_or_default()
}

pub(crate) fn json_string_list(values: &[String]) -> Option<String> {
    serde_json::to_string(values).ok()
}

const SYNOPSIS_MAX_LENGTH: usize = 200;
const SYNOPSIS_TOLERANCE: usize = 60;

fn dash_regex() -> &'static regex::Regex {
    use std::sync::OnceLock;
    static REGEX: OnceLock<regex::Regex> = OnceLock::new();
    REGEX.get_or_init(|| regex::Regex::new(r"\s*[—–]\s*").expect("valid dash regex"))
}

fn normalize_dashes_to_commas(text: &str) -> String {
    let replaced = dash_regex().replace_all(text, ", ");
    collapse_whitespace(&replaced.replace(",,", ","))
}

fn first_sentence(text: &str) -> Option<String> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for (i, &(byte_idx, ch)) in chars.iter().enumerate() {
        if matches!(ch, '.' | '!' | '?') && byte_idx >= 20 {
            let at_boundary = chars.get(i + 1).is_none_or(|&(_, c)| c.is_whitespace());
            if at_boundary {
                let end = byte_idx + ch.len_utf8();
                return Some(text[..end].to_string());
            }
        }
    }
    None
}

pub(crate) fn shorten_synopsis(text: &str) -> String {
    let normalized = normalize_dashes_to_commas(text.trim());
    if let Some(sentence) = first_sentence(&normalized) {
        if sentence.chars().count() <= SYNOPSIS_MAX_LENGTH + SYNOPSIS_TOLERANCE {
            return sentence;
        }
    }
    if normalized.chars().count() <= SYNOPSIS_MAX_LENGTH + SYNOPSIS_TOLERANCE {
        return normalized;
    }
    let window: String = normalized.chars().take(SYNOPSIS_MAX_LENGTH).collect();
    if let Some(last_comma) = window.rfind(',') {
        if last_comma as f64 > SYNOPSIS_MAX_LENGTH as f64 * 0.4 {
            return format!("{}.", &window[..last_comma]);
        }
    }
    if let Some(last_space) = window.rfind(' ') {
        if last_space > 0 {
            return format!("{}.", &window[..last_space]);
        }
    }
    format!("{window}.")
}
