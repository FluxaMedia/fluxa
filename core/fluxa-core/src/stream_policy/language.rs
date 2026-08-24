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
pub(crate) fn resolve_profile_audio_language(
    genres: &[String],
    anime_preferred: bool,
    preference: Option<&str>,
    original_language: Option<&str>,
    device_language: Option<&str>,
) -> Option<String> {
    let is_anime = genres
        .iter()
        .any(|genre| genre.to_ascii_lowercase().contains("anime"));
    if is_anime && anime_preferred {
        return Some("ja".to_string());
    }
    match preference {
        Some("original") => original_language
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        Some("device_language") => device_language
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        Some(value) => Some(value.to_string()),
        None => None,
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
pub(crate) fn iso_639_1_of(code: &str) -> &str {
    match code {
        "tur" => "tr",
        "eng" => "en",
        "jpn" => "ja",
        "ger" | "deu" => "de",
        "fre" | "fra" => "fr",
        "spa" => "es",
        "ita" => "it",
        "rus" => "ru",
        "por" => "pt",
        "ara" => "ar",
        other => other,
    }
}
pub(crate) fn subtitle_language_alias_matches(label: &str, normalized_preference: &str) -> bool {
    match normalized_preference {
        "tr" => ["turkish", "turkce", "türkçe", "turk", "altyazi", "altyazı"]
            .iter()
            .any(|alias| label.contains(alias)),
        "en" => ["english", "eng"].iter().any(|alias| label.contains(alias)),
        "ja" => ["japanese", "jpn"]
            .iter()
            .any(|alias| label.contains(alias)),
        _ => false,
    }
}
pub(crate) fn subtitle_language_matches(
    label: &str,
    language: Option<&str>,
    preferred_language: &str,
) -> bool {
    let normalized_preference = normalize_language_preference(preferred_language);
    let word_regex = word_boundary_regex_for(&normalized_preference);
    subtitle_language_matches_precompiled(
        label,
        language,
        &normalized_preference,
        word_regex.as_ref(),
    )
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
pub(crate) fn find_preferred_subtitle_index(
    tracks_json: &str,
    last_subtitle_language: Option<&str>,
    preferred_subtitle_language: Option<&str>,
    secondary_subtitle_language: Option<&str>,
) -> i32 {
    let Ok(tracks) = serde_json::from_str::<Vec<SubtitleSelectionTrack>>(tracks_json) else {
        return -1;
    };
    find_preferred_subtitle_index_in_tracks(
        &tracks,
        last_subtitle_language,
        preferred_subtitle_language,
        secondary_subtitle_language,
    )
}
