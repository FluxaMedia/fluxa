const RTL_LANGUAGES: [&str; 9] = ["ar", "he", "iw", "fa", "ur", "ps", "yi", "dv", "ckb"];

pub fn is_rtl_language(code: &str) -> bool {
    let primary = code
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    RTL_LANGUAGES.contains(&primary.as_str())
}

#[cfg(test)]
mod tests {
    use super::is_rtl_language;

    #[test]
    fn regional_variants_follow_their_primary_language() {
        assert!(is_rtl_language("ar-SA"));
        assert!(is_rtl_language("he_IL"));
        assert!(is_rtl_language("FA"));
    }

    #[test]
    fn other_languages_stay_left_to_right() {
        assert!(!is_rtl_language("tr"));
        assert!(!is_rtl_language("en-US"));
        assert!(!is_rtl_language(""));
    }
}
