use crate::constants::DEFAULT_LANGUAGE;

pub(crate) fn tmdb_content_type(content_type: &str) -> &str {
    if content_type == "series" {
        "tv"
    } else {
        "movie"
    }
}
pub(crate) fn tmdb_language(language: &str) -> String {
    match language {
        "" | DEFAULT_LANGUAGE | "english_us" => "en-US".to_string(),
        "tr" | "tr_tr" => "tr-TR".to_string(),
        lang if lang.contains('-') => lang.to_string(),
        lang => format!("{}-{}", lang, lang.to_uppercase()),
    }
}
pub(crate) fn tmdb_region_from_language(language: &str) -> String {
    tmdb_language(language)
        .split('-')
        .nth(1)
        .unwrap_or("US")
        .to_string()
}
pub(crate) fn tmdb_image_url(path: Option<&str>, size: &str) -> Option<String> {
    let path = path?.trim();
    if path.is_empty() {
        return None;
    }
    Some(format!("https://image.tmdb.org/t/p/{size}{path}"))
}
/// Returns (numeric_tmdb_id, already_resolved) — if already_resolved is true
/// the caller can use the id directly without an extra API call.
pub(crate) fn tmdb_resolve_id_hint(content_id: &str) -> (String, bool) {
    let base = content_id.replace("tmdb:", "");
    let base = base.split(':').next().unwrap_or(&base);
    if base.chars().all(|c| c.is_ascii_digit()) && !base.is_empty() {
        return (base.to_string(), true);
    }
    let imdb_part = content_id.split(':').next().unwrap_or(content_id);
    (imdb_part.to_string(), false)
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
pub(crate) fn tmdb_api_url(
    path: &str,
    api_key: &str,
    language: &str,
    extra: &[(&str, &str)],
) -> String {
    let mut params = format!(
        "api_key={}&language={}",
        encode_query(api_key),
        encode_query(&tmdb_language(language))
    );
    for (key, value) in extra {
        params.push_str(&format!("&{key}={}", encode_query(value)));
    }
    format!("https://api.themoviedb.org/{path}?{params}")
}
pub(crate) fn is_imdb_id(id: &str) -> bool {
    id.len() > 2
        && id[..2].eq_ignore_ascii_case("tt")
        && id[2..].bytes().all(|b| b.is_ascii_digit())
}
pub(crate) fn normalize_person_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
