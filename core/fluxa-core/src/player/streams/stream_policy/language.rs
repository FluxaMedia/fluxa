use std::sync::Mutex;

pub(crate) fn normalize_language(value: &str) -> String {
    value.to_lowercase()
}
pub(crate) fn normalize_language_preference(value: &str) -> String {
    normalize_language(value)
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_string()
}
pub(crate) fn resolve_preferred_audio_language(
    last_audio_language: Option<&str>,
    preferred_audio_language: Option<&str>,
    original_language: Option<&str>,
) -> String {
    if let Some(memory) = last_audio_language
        .map(normalize_language)
        .filter(|value| !value.trim().is_empty())
    {
        return memory;
    }
    let Some(preferred) = preferred_audio_language
        .map(normalize_language)
        .filter(|value| value != "none")
    else {
        return String::new();
    };
    if preferred != "en" {
        return preferred;
    }
    if original_language.map(normalize_language).as_deref() == Some("ja") {
        "ja".to_string()
    } else {
        preferred
    }
}
// The preference string changes once per settings edit, not per stream, so a
// one-entry cache avoids recompiling the same word-boundary regex on every
// track in a per-stream ranking loop.
fn word_boundary_regex_for(normalized_preference: &str) -> Option<regex::Regex> {
    static CACHE: Mutex<Option<(String, regex::Regex)>> = Mutex::new(None);
    let mut cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((cached_preference, regex)) = cache.as_ref()
        && cached_preference == normalized_preference
    {
        return Some(regex.clone());
    }
    let regex =
        regex::Regex::new(&format!(r"\b{}\b", regex::escape(normalized_preference))).ok()?;
    *cache = Some((normalized_preference.to_string(), regex.clone()));
    Some(regex)
}
const LANGUAGES: [(&str, &str, &str, &str); 74] = [
    ("af", "afr", "afr", "afrikaans"),
    ("sq", "sqi", "alb", "albanian"),
    ("am", "amh", "amh", "amharic"),
    ("ar", "ara", "ara", "arabic"),
    ("hy", "hye", "arm", "armenian"),
    ("az", "aze", "aze", "azerbaijani"),
    ("eu", "eus", "baq", "basque"),
    ("be", "bel", "bel", "belarusian"),
    ("bn", "ben", "ben", "bengali"),
    ("bs", "bos", "bos", "bosnian"),
    ("bg", "bul", "bul", "bulgarian"),
    ("my", "mya", "bur", "burmese"),
    ("ca", "cat", "cat", "catalan"),
    ("zh", "zho", "chi", "chinese"),
    ("hr", "hrv", "hrv", "croatian"),
    ("cs", "ces", "cze", "czech"),
    ("da", "dan", "dan", "danish"),
    ("nl", "nld", "dut", "dutch"),
    ("en", "eng", "eng", "english"),
    ("et", "est", "est", "estonian"),
    ("tl", "fil", "tgl", "filipino"),
    ("fi", "fin", "fin", "finnish"),
    ("fr", "fra", "fre", "french"),
    ("gl", "glg", "glg", "galician"),
    ("ka", "kat", "geo", "georgian"),
    ("de", "deu", "ger", "german"),
    ("el", "ell", "gre", "greek"),
    ("gu", "guj", "guj", "gujarati"),
    ("he", "heb", "heb", "hebrew"),
    ("hi", "hin", "hin", "hindi"),
    ("hu", "hun", "hun", "hungarian"),
    ("is", "isl", "ice", "icelandic"),
    ("id", "ind", "ind", "indonesian"),
    ("ga", "gle", "gle", "irish"),
    ("it", "ita", "ita", "italian"),
    ("ja", "jpn", "jpn", "japanese"),
    ("kn", "kan", "kan", "kannada"),
    ("kk", "kaz", "kaz", "kazakh"),
    ("km", "khm", "khm", "khmer"),
    ("ko", "kor", "kor", "korean"),
    ("lo", "lao", "lao", "lao"),
    ("lv", "lav", "lav", "latvian"),
    ("lt", "lit", "lit", "lithuanian"),
    ("mk", "mkd", "mac", "macedonian"),
    ("ms", "msa", "may", "malay"),
    ("ml", "mal", "mal", "malayalam"),
    ("mt", "mlt", "mlt", "maltese"),
    ("mr", "mar", "mar", "marathi"),
    ("mn", "mon", "mon", "mongolian"),
    ("ne", "nep", "nep", "nepali"),
    ("no", "nor", "nor", "norwegian"),
    ("pa", "pan", "pan", "punjabi"),
    ("fa", "fas", "per", "persian"),
    ("pl", "pol", "pol", "polish"),
    ("pt", "por", "por", "portuguese"),
    ("ro", "ron", "rum", "romanian"),
    ("ru", "rus", "rus", "russian"),
    ("sr", "srp", "srp", "serbian"),
    ("si", "sin", "sin", "sinhala"),
    ("sk", "slk", "slo", "slovak"),
    ("sl", "slv", "slv", "slovenian"),
    ("es", "spa", "spa", "spanish"),
    ("sw", "swa", "swa", "swahili"),
    ("sv", "swe", "swe", "swedish"),
    ("ta", "tam", "tam", "tamil"),
    ("te", "tel", "tel", "telugu"),
    ("th", "tha", "tha", "thai"),
    ("tr", "tur", "tur", "turkish"),
    ("uk", "ukr", "ukr", "ukrainian"),
    ("ur", "urd", "urd", "urdu"),
    ("uz", "uzb", "uzb", "uzbek"),
    ("vi", "vie", "vie", "vietnamese"),
    ("cy", "cym", "wel", "welsh"),
    ("zu", "zul", "zul", "zulu"),
];
fn language_entry(
    code: &str,
) -> Option<&'static (&'static str, &'static str, &'static str, &'static str)> {
    LANGUAGES
        .iter()
        .find(|(one, two_t, two_b, _)| code == *one || code == *two_t || code == *two_b)
}
const COUNTRY_LANGUAGES: [(&str, &str, &str); 34] = [
    ("jp", "japan", "ja"),
    ("kr", "south korea", "ko"),
    ("cn", "china", "zh"),
    ("tw", "taiwan", "zh"),
    ("fr", "france", "fr"),
    ("de", "germany", "de"),
    ("it", "italy", "it"),
    ("es", "spain", "es"),
    ("pt", "portugal", "pt"),
    ("br", "brazil", "pt"),
    ("ru", "russia", "ru"),
    ("in", "india", "hi"),
    ("tr", "turkey", "tr"),
    ("pl", "poland", "pl"),
    ("nl", "netherlands", "nl"),
    ("se", "sweden", "sv"),
    ("no", "norway", "no"),
    ("dk", "denmark", "da"),
    ("fi", "finland", "fi"),
    ("th", "thailand", "th"),
    ("il", "israel", "he"),
    ("cz", "czech republic", "cs"),
    ("ro", "romania", "ro"),
    ("hu", "hungary", "hu"),
    ("ua", "ukraine", "uk"),
    ("gr", "greece", "el"),
    ("us", "united states", "en"),
    ("gb", "united kingdom", "en"),
    ("ca", "canada", "en"),
    ("au", "australia", "en"),
    ("mx", "mexico", "es"),
    ("ar", "argentina", "es"),
    ("eg", "egypt", "ar"),
    ("id", "indonesia", "id"),
];
pub(crate) fn resolve_content_language(
    language: Option<&str>,
    country: Option<&str>,
) -> Option<String> {
    let first = |value: Option<&str>| {
        value
            .and_then(|text| text.split([',', '/']).next())
            .map(|text| normalize_language(text.trim()))
            .filter(|text| !text.is_empty())
    };
    if let Some(entry) = first(language).as_deref().and_then(|text| {
        let base = text.split(['-', '_']).next().unwrap_or(text);
        language_entry(base).or_else(|| LANGUAGES.iter().find(|entry| entry.3 == text))
    }) {
        return Some(entry.0.to_string());
    }
    let country = first(country)?;
    COUNTRY_LANGUAGES
        .iter()
        .find(|(code, name, _)| country == *code || country == *name)
        .map(|(_, _, language)| language.to_string())
}
pub(crate) fn is_forced_subtitle(track: &SubtitleSelectionTrack) -> bool {
    if track.forced {
        return true;
    }
    let text = format!(
        "{} {} {}",
        track.label,
        track.language.as_deref().unwrap_or(""),
        track.id.as_deref().unwrap_or("")
    )
    .to_lowercase();
    text.contains("forced") || (text.contains("songs") && text.contains("sign"))
}
// HLS EXT-X-MEDIA and most scrapers tag renditions with ISO 639-2 ("tur",
// "eng"), while profile preferences are ISO 639-1 ("tr", "en").
pub(crate) fn iso_639_1_of(code: &str) -> &str {
    language_entry(code).map_or(code, |entry| entry.0)
}
pub(crate) fn subtitle_language_alias_matches(label: &str, normalized_preference: &str) -> bool {
    if normalized_preference == "tr" {
        return ["turkish", "turkce", "türkçe", "turk", "altyazi", "altyazı"]
            .iter()
            .any(|alias| label.contains(alias));
    }
    language_entry(normalized_preference).is_some_and(|(_, two_t, two_b, name)| {
        label.contains(name)
            || label
                .split(|c: char| !c.is_alphanumeric())
                .any(|word| word == *two_t || word == *two_b)
    })
}
fn subtitle_language_matches_precompiled(
    label: &str,
    language: Option<&str>,
    normalized_preference: &str,
    word_regex: Option<&regex::Regex>,
) -> bool {
    if normalized_preference.is_empty() {
        return false;
    }
    let language = normalize_language(language.unwrap_or(""));
    let language = iso_639_1_of(language.split(['-', '_']).next().unwrap_or(""));
    if language.starts_with(normalized_preference) {
        return true;
    }
    let label = normalize_language(label);
    word_regex.is_some_and(|regex| regex.is_match(&label))
        || subtitle_language_alias_matches(&label, normalized_preference)
}

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubtitleSelectionTrack {
    pub(crate) id: Option<String>,
    pub(crate) label: String,
    pub(crate) language: Option<String>,
    #[serde(default)]
    pub(crate) forced: bool,
}
pub(crate) fn find_preferred_subtitle_index_in_tracks(
    tracks: &[SubtitleSelectionTrack],
    last_subtitle_language: Option<&str>,
    preferred_subtitle_language: Option<&str>,
    secondary_subtitle_language: Option<&str>,
) -> i32 {
    let primary = last_subtitle_language
        .filter(|value| !value.is_empty() && *value != "__off__")
        .or_else(|| preferred_subtitle_language.filter(|value| *value != "none"));
    if let Some(preferred) = primary {
        let norm = normalize_language_preference(preferred);
        let word_regex = word_boundary_regex_for(&norm);
        if let Some(index) = tracks.iter().position(|track| {
            subtitle_language_matches_precompiled(
                &track.label,
                track.language.as_deref(),
                &norm,
                word_regex.as_ref(),
            )
        }) {
            return index as i32;
        }
    }
    if let Some(secondary) = secondary_subtitle_language.filter(|value| *value != "none") {
        let norm = normalize_language_preference(secondary);
        let word_regex = word_boundary_regex_for(&norm);
        if let Some(index) = tracks.iter().position(|track| {
            subtitle_language_matches_precompiled(
                &track.label,
                track.language.as_deref(),
                &norm,
                word_regex.as_ref(),
            )
        }) {
            return index as i32;
        }
    }
    -1
}
