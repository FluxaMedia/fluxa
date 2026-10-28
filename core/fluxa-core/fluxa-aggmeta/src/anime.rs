#[derive(Clone, Debug, Default)]
pub struct TitleFacts {
    pub genre_ids: Vec<u32>,
    pub genre_names: Vec<String>,
    pub original_language: Option<String>,
    pub origin_countries: Vec<String>,
}

pub fn is_anime(f: &TitleFacts) -> bool {
    let animation = f.genre_ids.contains(&16)
        || f.genre_names
            .iter()
            .any(|g| g.eq_ignore_ascii_case("animation"));
    let anime_genre = f
        .genre_names
        .iter()
        .any(|g| g.eq_ignore_ascii_case("anime"));
    if !(animation || anime_genre) {
        return false;
    }
    let japanese_language = f
        .original_language
        .as_deref()
        .is_some_and(|l| l.eq_ignore_ascii_case("ja"));
    let japanese_origin = f
        .origin_countries
        .iter()
        .any(|c| c.eq_ignore_ascii_case("jp") || c.eq_ignore_ascii_case("jpn"));
    japanese_language || japanese_origin || anime_genre
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(ids: &[u32], names: &[&str], lang: &str, countries: &[&str]) -> TitleFacts {
        TitleFacts {
            genre_ids: ids.to_vec(),
            genre_names: names.iter().map(|s| s.to_string()).collect(),
            original_language: Some(lang.into()),
            origin_countries: countries.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn japanese_animation_is_anime() {
        assert!(is_anime(&facts(&[16], &[], "ja", &["JP"])));
    }

    #[test]
    fn western_animation_is_not_anime() {
        assert!(!is_anime(&facts(&[16], &["Animation"], "en", &["US"])));
    }

    #[test]
    fn anime_genre_alone_counts_without_japanese_origin() {
        assert!(is_anime(&facts(&[], &["Anime"], "ko", &["KR"])));
    }

    #[test]
    fn localized_animation_name_is_matched_by_id_only() {
        assert!(!is_anime(&facts(&[], &["Animación"], "ja", &["JP"])));
        assert!(is_anime(&facts(&[16], &["Animación"], "ja", &["JP"])));
    }
}
