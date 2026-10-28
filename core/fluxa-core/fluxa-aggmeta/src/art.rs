use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageCandidate {
    pub file_path: String,
    pub language: Option<String>,
    pub country: Option<String>,
    pub vote_average: f64,
    pub width: u32,
}

#[derive(Clone, Debug)]
pub struct ArtPolicy {
    pub user_language: String,
    pub english_art_only: bool,
    pub original_lang_fallback: bool,
    pub original_language: Option<String>,
}

impl Default for ArtPolicy {
    fn default() -> Self {
        Self {
            user_language: "en-US".into(),
            english_art_only: false,
            original_lang_fallback: false,
            original_language: None,
        }
    }
}

fn split_language(tag: &str) -> (String, Option<String>) {
    match tag.split_once(['-', '_']) {
        Some((l, c)) => (l.to_lowercase(), Some(c.to_uppercase())),
        None => (tag.to_lowercase(), None),
    }
}

fn is_textless(c: &ImageCandidate) -> bool {
    c.language
        .as_deref()
        .is_none_or(|l| l.is_empty() || l == "xx")
}

fn lang_of(c: &ImageCandidate) -> Option<&str> {
    c.language
        .as_deref()
        .filter(|l| !l.is_empty() && *l != "xx")
}

fn best<'a>(items: impl Iterator<Item = &'a ImageCandidate>) -> Option<&'a ImageCandidate> {
    items.fold(None, |acc: Option<&ImageCandidate>, c| match acc {
        Some(a) if a.vote_average >= c.vote_average => Some(a),
        _ => Some(c),
    })
}

fn in_language<'a>(candidates: &'a [ImageCandidate], lang: &str) -> Vec<&'a ImageCandidate> {
    candidates
        .iter()
        .filter(|c| lang_of(c).is_some_and(|x| x.eq_ignore_ascii_case(lang)))
        .collect()
}

pub fn select_image<'a>(
    candidates: &'a [ImageCandidate],
    policy: &ArtPolicy,
) -> Option<&'a ImageCandidate> {
    if candidates.is_empty() {
        return None;
    }
    let (user_lang, user_country) = split_language(&policy.user_language);
    let target_lang = if policy.english_art_only {
        "en".to_string()
    } else {
        user_lang
    };
    let target_country = user_country.unwrap_or_else(|| "US".into());
    let original = policy.original_language.as_deref().map(str::to_lowercase);
    let in_lang = |l: &str| in_language(candidates, l);

    if let Some(c) = best(in_lang(&target_lang).into_iter().filter(|c| {
        c.country
            .as_deref()
            .is_some_and(|x| x.eq_ignore_ascii_case(&target_country))
    })) {
        return Some(c);
    }
    if policy.original_lang_fallback {
        return best(in_lang("en").into_iter())
            .or_else(|| {
                original
                    .as_deref()
                    .and_then(|o| best(in_lang(o).into_iter()))
            })
            .or_else(|| best(candidates.iter().filter(|c| !is_textless(c))))
            .or_else(|| candidates.first());
    }
    best(in_lang(&target_lang).into_iter())
        .or_else(|| best(in_lang("en").into_iter()))
        .or_else(|| {
            original
                .as_deref()
                .and_then(|o| best(in_lang(o).into_iter()))
        })
        .or_else(|| best(candidates.iter().filter(|c| !is_textless(c))))
        .or_else(|| candidates.first())
}

#[derive(Clone, Debug, Default)]
pub struct Translation {
    pub language: String,
    pub country: String,
    pub title: String,
}

fn non_blank(s: &str) -> Option<&str> {
    let t = s.trim();
    (!t.is_empty()).then_some(t)
}

pub fn translate_title(
    translations: &[Translation],
    user_language: &str,
    original_language: &str,
    original_title: &str,
) -> Option<String> {
    let find = |lang: &str, country: &str| {
        translations
            .iter()
            .find(|t| {
                t.language.eq_ignore_ascii_case(lang) && t.country.eq_ignore_ascii_case(country)
            })
            .and_then(|t| non_blank(&t.title))
    };
    let (lang, country) = split_language(user_language);
    let country = country.unwrap_or_default();
    let same_as_original = lang.eq_ignore_ascii_case(original_language);
    let original = || {
        same_as_original
            .then(|| non_blank(original_title))
            .flatten()
    };

    if lang == "pt" && country == "PT" {
        return find("pt", "PT")
            .or_else(|| find("pt", "BR"))
            .or_else(original)
            .or_else(|| find("en", "US"))
            .or_else(original)
            .map(str::to_string);
    }
    find(&lang, &country)
        .or_else(original)
        .or_else(|| find("en", "US"))
        .or_else(original)
        .map(str::to_string)
}

pub fn tmdb_image_url(size: &str, path: &str) -> String {
    format!("https://image.tmdb.org/t/p/{size}{path}")
}

pub fn profile_url(path: &str) -> String {
    tmdb_image_url("w276_and_h350_face", path)
}

pub fn expand_template(template: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

pub fn template_id(imdb: Option<&str>, tmdb: Option<u64>, tvdb: Option<u64>) -> Option<String> {
    imdb.map(str::to_string)
        .or_else(|| tmdb.map(|n| format!("tmdb:{n}")))
        .or_else(|| tvdb.map(|n| format!("tvdb:{n}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(path: &str, lang: Option<&str>, country: Option<&str>, votes: f64) -> ImageCandidate {
        ImageCandidate {
            file_path: path.into(),
            language: lang.map(String::from),
            country: country.map(String::from),
            vote_average: votes,
            width: 1000,
        }
    }

    fn tr(lang: &str, country: &str, title: &str) -> Translation {
        Translation {
            language: lang.into(),
            country: country.into(),
            title: title.into(),
        }
    }

    #[test]
    fn exact_language_and_country_wins() {
        let c = [
            img("/en", Some("en"), Some("US"), 9.0),
            img("/fr", Some("fr"), Some("FR"), 1.0),
        ];
        let policy = ArtPolicy {
            user_language: "fr-FR".into(),
            ..Default::default()
        };
        assert_eq!(select_image(&c, &policy).unwrap().file_path, "/fr");
    }

    #[test]
    fn default_ladder_prefers_english_over_original() {
        let c = [
            img("/ja", Some("ja"), None, 9.0),
            img("/en", Some("en"), None, 1.0),
        ];
        let policy = ArtPolicy {
            user_language: "de-DE".into(),
            original_language: Some("ja".into()),
            ..Default::default()
        };
        assert_eq!(select_image(&c, &policy).unwrap().file_path, "/en");
    }

    #[test]
    fn original_fallback_mode_still_tries_english_first() {
        let c = [
            img("/ja", Some("ja"), None, 9.0),
            img("/en", Some("en"), None, 1.0),
        ];
        let policy = ArtPolicy {
            user_language: "de-DE".into(),
            original_lang_fallback: true,
            original_language: Some("ja".into()),
            ..Default::default()
        };
        assert_eq!(select_image(&c, &policy).unwrap().file_path, "/en");
    }

    #[test]
    fn textless_is_last_resort() {
        let c = [
            img("/none", None, None, 9.0),
            img("/xx", Some("xx"), None, 9.5),
        ];
        let policy = ArtPolicy::default();
        assert_eq!(select_image(&c, &policy).unwrap().file_path, "/none");
    }

    #[test]
    fn blank_translation_falls_back_to_english() {
        let t = [tr("fr", "FR", "  "), tr("en", "US", "Spirited Away")];
        let got = translate_title(&t, "fr-FR", "ja", "Sen to Chihiro");
        assert_eq!(got.as_deref(), Some("Spirited Away"));
    }

    #[test]
    fn original_title_is_used_when_language_matches() {
        let got = translate_title(&[], "ja-JP", "ja", "千と千尋の神隠し");
        assert_eq!(got.as_deref(), Some("千と千尋の神隠し"));
    }

    #[test]
    fn european_portuguese_falls_through_brazilian() {
        let t = [
            tr("pt", "BR", "Cidade de Deus"),
            tr("en", "US", "City of God"),
        ];
        assert_eq!(
            translate_title(&t, "pt-PT", "pt", "Cidade de Deus").as_deref(),
            Some("Cidade de Deus")
        );
    }
}
