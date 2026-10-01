use std::sync::OnceLock;

pub(crate) const TMDB_ID_PREFIX: &str = "tmdb:";

#[expect(
    clippy::expect_used,
    reason = "static literal regex is not input-dependent"
)]
pub(crate) fn imdb_regex() -> &'static regex::Regex {
    pub(crate) static REGEX: OnceLock<regex::Regex> = OnceLock::new();
    REGEX.get_or_init(|| regex::Regex::new(r"tt\d+").expect("valid imdb regex"))
}

pub(crate) fn push_unique(values: &mut Vec<String>, value: String) {
    if !value.is_empty() && !values.contains(&value) {
        values.push(value);
    }
}
