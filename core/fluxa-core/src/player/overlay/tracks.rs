use crate::player::streams::stream_policy::{
    SubtitleSelectionTrack, find_preferred_subtitle_index_in_tracks, iso_639_1_of,
    normalize_language_preference, resolve_preferred_audio_language,
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
        .then(|| {
            find_preferred_subtitle_index_in_tracks(
                &input.subtitle_tracks,
                input.last_subtitle_language.as_deref(),
                prefs.preferred_subtitle_language.as_deref(),
                prefs.secondary_subtitle_language.as_deref(),
            )
        })
        .filter(|index| *index >= 0)
        .map(|index| index as usize);
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

fn pick_audio(input: &TrackInput) -> Option<String> {
    let tracks: Vec<AudioTrackIn> = input
        .audio_tracks
        .iter()
        .filter_map(|value| serde_json::from_value(value.clone()).ok())
        .collect();
    if tracks.is_empty() {
        return None;
    }
    let primary = resolve_preferred_audio_language(
        input.last_audio_language.as_deref(),
        input.prefs.preferred_audio_language.as_deref(),
        input.original_language.as_deref(),
    );
    let secondary = input
        .prefs
        .secondary_audio_language
        .as_deref()
        .map(canonical_language)
        .unwrap_or_default();
    [canonical_language(&primary), secondary]
        .into_iter()
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

}
