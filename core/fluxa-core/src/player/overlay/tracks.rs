use crate::player::streams::stream_policy::{
    SubtitleSelectionTrack, find_preferred_subtitle_index_in_tracks, is_forced_subtitle,
    iso_639_1_of, normalize_language_preference, resolve_content_language,
    resolve_preferred_audio_language,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AudioTrackIn {
    id: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    is_selected: bool,
}

fn canonical_language(language: &str) -> String {
    let base = normalize_language_preference(language);
    iso_639_1_of(&base).to_string()
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct TrackPrefs {
    preferred_audio_language: Option<String>,
    secondary_audio_language: Option<String>,
    preferred_subtitle_language: Option<String>,
    secondary_subtitle_language: Option<String>,
    auto_enable_subtitles: Option<bool>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct TrackInput {
    audio_tracks: Vec<Value>,
    subtitle_tracks: Vec<SubtitleSelectionTrack>,
    prefs: TrackPrefs,
    last_audio_language: Option<String>,
    last_subtitle_language: Option<String>,
    original_language: Option<String>,
    language: Option<String>,
    country: Option<String>,
    device_languages: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrackPlan {
    audio_id: Option<String>,
    subtitle_id: Option<String>,
    secondary_subtitle_id: Option<String>,
    disable_subtitles: bool,
}

pub(crate) fn player_track_plan_json(input: &str) -> Option<String> {
    let input: TrackInput = serde_json::from_str(input).ok()?;
    let prefs = &input.prefs;
    let audio_id = pick_audio(&input);

    let subtitles_off = input.last_subtitle_language.as_deref() == Some("__off__")
        || (input.last_subtitle_language.is_none() && prefs.auto_enable_subtitles == Some(false));
    let subtitle_index = (!subtitles_off)
        .then(|| pick_subtitle(&input))
        .flatten();
    let track_id = |index: usize| input.subtitle_tracks.get(index)?.id.clone();
    let secondary_index = subtitle_index.and_then(|primary| {
        let wanted = prefs.secondary_subtitle_language.as_deref()?;
        let others: Vec<SubtitleSelectionTrack> = input
            .subtitle_tracks
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != primary)
            .map(|(_, track): (usize, &SubtitleSelectionTrack)| track.clone())
            .collect();
        let found = find_preferred_subtitle_index_in_tracks(&others, None, Some(wanted), None);
        (found >= 0).then(|| {
            let found = found as usize;
            if found >= primary { found + 1 } else { found }
        })
    });
    serde_json::to_string(&TrackPlan {
        audio_id,
        subtitle_id: subtitle_index.and_then(track_id),
        secondary_subtitle_id: secondary_index.and_then(track_id),
        disable_subtitles: subtitle_index.is_none(),
    })
    .ok()
}

fn device_languages(input: &TrackInput) -> Vec<String> {
    input
        .device_languages
        .iter()
        .map(|language| canonical_language(language))
        .filter(|language| !language.is_empty())
        .collect()
}

fn pick_subtitle(input: &TrackInput) -> Option<usize> {
    let tracks = &input.subtitle_tracks;
    let prefs = &input.prefs;
    let secondary = prefs.secondary_subtitle_language.as_deref();
    let find = |primary: Option<&str>| {
        let index =
            find_preferred_subtitle_index_in_tracks(tracks, None, primary, secondary);
        (index >= 0).then_some(index as usize)
    };
    let last = input
        .last_subtitle_language
        .as_deref()
        .filter(|value| !value.is_empty() && *value != "__off__");
    if last.is_some() {
        let index = find_preferred_subtitle_index_in_tracks(
            tracks,
            last,
            prefs.preferred_subtitle_language.as_deref(),
            secondary,
        );
        return (index >= 0).then_some(index as usize);
    }
    match prefs.preferred_subtitle_language.as_deref() {
        Some("forced") => {
            let forced: Vec<usize> = (0..tracks.len())
                .filter(|index| is_forced_subtitle(&tracks[*index]))
                .collect();
            let device = device_languages(input);
            forced
                .iter()
                .copied()
                .find(|index| {
                    let language = tracks[*index]
                        .language
                        .as_deref()
                        .map(canonical_language)
                        .unwrap_or_default();
                    device.contains(&language)
                })
                .or_else(|| forced.first().copied())
                .or_else(|| find(None))
        }
        Some("device") => device_languages(input)
            .iter()
            .find_map(|language| {
                let index = find_preferred_subtitle_index_in_tracks(
                    tracks,
                    None,
                    Some(language),
                    None,
                );
                (index >= 0).then_some(index as usize)
            })
            .or_else(|| find(None)),
        preferred => find(preferred),
    }
}

fn pick_audio(input: &TrackInput) -> Option<String> {
    let tracks: Vec<AudioTrackIn> = input
        .audio_tracks
        .iter()
        .filter_map(|value| serde_json::from_value(value.clone()).ok())
        .collect();
    if tracks.is_empty() {
        return None;
    }
    let remembered = input
        .last_audio_language
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    let original = resolve_content_language(
        input.original_language.as_deref().or(input.language.as_deref()),
        input.country.as_deref(),
    );
    let primary = match (remembered, input.prefs.preferred_audio_language.as_deref()) {
        (Some(last), _) => vec![canonical_language(last)],
        (None, Some("default")) => Vec::new(),
        (None, Some("device")) => device_languages(input),
        (None, Some("original")) => original
            .map(|language| vec![language])
            .unwrap_or_else(|| device_languages(input)),
        (None, preferred) => vec![canonical_language(&resolve_preferred_audio_language(
            None,
            preferred,
            original.as_deref(),
        ))],
    };
    let secondary = input
        .prefs
        .secondary_audio_language
        .as_deref()
        .map(canonical_language)
        .unwrap_or_default();
    primary
        .into_iter()
        .chain([secondary])
        .filter(|language: &String| !language.is_empty() && language != "none")
        .find(|language| {
            tracks
                .iter()
                .any(|track| track_language(track) == *language)
        })
        .and_then(|language| {
            let request = json!({
                "tracksJson": json!(tracks.iter().map(|track| json!({
                    "id": track.id,
                    "language": track_language(track),
                    "isSelected": track.is_selected,
                })).collect::<Vec<_>>()).to_string(),
                "preferredLanguage": language,
            });
            let picked =
                crate::player::playback::policy::select_audio_track_json(&request.to_string())?;
            serde_json::from_str::<Option<String>>(&picked)
                .ok()
                .flatten()
        })
}

fn track_language(track: &AudioTrackIn) -> String {
    track
        .language
        .as_deref()
        .map(canonical_language)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tracks_follow_preferred_then_secondary_languages() {
        let result: Value = serde_json::from_str(
            &player_track_plan_json(
                &json!({
                    "audioTracks": [
                        {"id": "1", "language": "eng"},
                        {"id": "2", "language": "jpn"}
                    ],
                    "subtitleTracks": [
                        {"id": "s1", "label": "English", "language": "en"},
                        {"id": "s2", "label": "Turkish", "language": "tr"}
                    ],
                    "prefs": {
                        "preferredAudioLanguage": "fr",
                        "secondaryAudioLanguage": "ja",
                        "preferredSubtitleLanguage": "tr",
                        "secondarySubtitleLanguage": "en"
                    }
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["audioId"], "2");
        assert_eq!(result["subtitleId"], "s2");
        assert_eq!(result["secondarySubtitleId"], "s1");
    }

    #[test]
    fn subtitles_stay_off_when_user_turned_them_off() {
        let result: Value = serde_json::from_str(
            &player_track_plan_json(
                &json!({
                    "subtitleTracks": [{"id": "s1", "label": "English", "language": "en"}],
                    "lastSubtitleLanguage": "__off__",
                    "prefs": {"preferredSubtitleLanguage": "en"}
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["disableSubtitles"], true);
        assert!(result["subtitleId"].is_null());
    }

    fn plan(input: Value) -> Value {
        serde_json::from_str(&player_track_plan_json(&input.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn original_audio_uses_content_language_then_country_then_device() {
        let tracks = json!([{"id": "1", "language": "eng"}, {"id": "2", "language": "kor"}]);
        let by_language = plan(json!({
            "audioTracks": tracks,
            "prefs": {"preferredAudioLanguage": "original"},
            "originalLanguage": "ko",
            "deviceLanguages": ["en-US"]
        }));
        assert_eq!(by_language["audioId"], "2");
        let by_country = plan(json!({
            "audioTracks": tracks,
            "prefs": {"preferredAudioLanguage": "original"},
            "country": "South Korea"
        }));
        assert_eq!(by_country["audioId"], "2");
        let unknown = plan(json!({
            "audioTracks": tracks,
            "prefs": {"preferredAudioLanguage": "original"},
            "deviceLanguages": ["en-US"]
        }));
        assert_eq!(unknown["audioId"], "1");
    }

    #[test]
    fn default_audio_leaves_file_default_unless_secondary_is_set() {
        let tracks = json!([{"id": "1", "language": "eng"}, {"id": "2", "language": "jpn"}]);
        let plain = plan(json!({
            "audioTracks": tracks,
            "prefs": {"preferredAudioLanguage": "default"}
        }));
        assert!(plain["audioId"].is_null());
        let with_secondary = plan(json!({
            "audioTracks": tracks,
            "prefs": {"preferredAudioLanguage": "default", "secondaryAudioLanguage": "ja"}
        }));
        assert_eq!(with_secondary["audioId"], "2");
    }

    #[test]
    fn forced_subtitles_pick_flagged_or_labelled_track() {
        let flagged = plan(json!({
            "subtitleTracks": [
                {"id": "s1", "label": "English", "language": "en"},
                {"id": "s2", "label": "English", "language": "en", "forced": true}
            ],
            "prefs": {"preferredSubtitleLanguage": "forced"}
        }));
        assert_eq!(flagged["subtitleId"], "s2");
        let labelled = plan(json!({
            "subtitleTracks": [
                {"id": "s1", "label": "English", "language": "en"},
                {"id": "s2", "label": "Spanish [Forced]", "language": "es"}
            ],
            "prefs": {"preferredSubtitleLanguage": "forced"}
        }));
        assert_eq!(labelled["subtitleId"], "s2");
        let none_forced = plan(json!({
            "subtitleTracks": [{"id": "s1", "label": "English", "language": "en"}],
            "prefs": {"preferredSubtitleLanguage": "forced"}
        }));
        assert_eq!(none_forced["disableSubtitles"], true);
    }

    #[test]
    fn device_subtitles_follow_device_language_order() {
        let result = plan(json!({
            "subtitleTracks": [
                {"id": "s1", "label": "English", "language": "en"},
                {"id": "s2", "label": "Turkish", "language": "tr"}
            ],
            "prefs": {"preferredSubtitleLanguage": "device"},
            "deviceLanguages": ["tr-TR", "en-US"]
        }));
        assert_eq!(result["subtitleId"], "s2");
    }
}
