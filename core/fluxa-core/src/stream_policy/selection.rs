use super::language::{
    SubtitleSelectionTrack, find_preferred_subtitle_index_in_tracks,
    resolve_preferred_audio_language, resolve_profile_audio_language,
};
use super::meta::SourceSelectionMode;
use crate::content_identity::stream_matches_episode;
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerTrackStateRequest {
    #[serde(default)]
    available_subtitles: Vec<SubtitleSelectionTrack>,
    last_audio_language: Option<String>,
    preferred_audio_language: Option<String>,
    original_language: Option<String>,
    #[serde(default)]
    content_genres: Vec<String>,
    profile_audio_language: Option<String>,
    #[serde(default)]
    anime_prefer_japanese_audio: bool,
    device_language: Option<String>,
    last_subtitle_language: Option<String>,
    preferred_subtitle_language: Option<String>,
    secondary_subtitle_language: Option<String>,
}
pub(crate) fn player_track_state_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<PlayerTrackStateRequest>(request_json).ok()?;
    let profile_audio_language = resolve_profile_audio_language(
        &request.content_genres,
        request.anime_prefer_japanese_audio,
        request
            .profile_audio_language
            .as_deref()
            .filter(|value| !value.is_empty()),
        request.original_language.as_deref(),
        request.device_language.as_deref(),
    );
    let preferred_audio_language = resolve_preferred_audio_language(
        request
            .last_audio_language
            .as_deref()
            .filter(|value| !value.is_empty()),
        request
            .preferred_audio_language
            .as_deref()
            .filter(|value| !value.is_empty())
            .or(profile_audio_language.as_deref()),
        request
            .original_language
            .as_deref()
            .filter(|value| !value.is_empty()),
    );
    let preferred_subtitle_index = find_preferred_subtitle_index_in_tracks(
        &request.available_subtitles,
        request
            .last_subtitle_language
            .as_deref()
            .filter(|value| !value.is_empty()),
        request
            .preferred_subtitle_language
            .as_deref()
            .filter(|value| !value.is_empty()),
        request
            .secondary_subtitle_language
            .as_deref()
            .filter(|value| !value.is_empty()),
    );
    let preferred_subtitle_id = if preferred_subtitle_index >= 0 {
        request
            .available_subtitles
            .get(preferred_subtitle_index as usize)
            .and_then(|track| track.id.clone())
    } else {
        None
    };
    serde_json::to_string(&json!({
        "preferredAudioLanguage": preferred_audio_language,
        "preferredSubtitleIndex": preferred_subtitle_index,
        "preferredSubtitleId": preferred_subtitle_id,
        "subtitlesDisabled": preferred_subtitle_index < 0
    }))
    .ok()
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamSelectionItem {
    name: Option<String>,
    title: Option<String>,
    description: Option<String>,
    addon_name: Option<String>,
    playable_url: Option<String>,
    binge_group: Option<String>,
    filename: Option<String>,
    effective_filename: Option<String>,
}
impl StreamSelectionItem {
    pub(crate) fn matches_episode(&self, video_id: &str) -> bool {
        stream_matches_episode(
            video_id,
            &[
                self.title.clone().unwrap_or_default(),
                self.name.clone().unwrap_or_default(),
                self.description.clone().unwrap_or_default(),
                self.filename.clone().unwrap_or_default(),
                self.effective_filename.clone().unwrap_or_default(),
            ],
        )
    }

    pub(crate) fn is_playable_for_episode(&self, video_id: &str) -> bool {
        self.playable_url
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            && self.matches_episode(video_id)
    }

    pub(crate) fn selection_text(&self) -> String {
        [
            self.name.as_deref(),
            self.title.as_deref(),
            self.description.as_deref(),
            self.addon_name.as_deref(),
            self.playable_url.as_deref(),
            self.binge_group.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
    }
}
fn stream_selection_item_from_value(v: &Value) -> StreamSelectionItem {
    StreamSelectionItem {
        name: v.get("name").and_then(Value::as_str).map(str::to_string),
        title: v.get("title").and_then(Value::as_str).map(str::to_string),
        description: v
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string),
        addon_name: v
            .get("addonName")
            .and_then(Value::as_str)
            .map(str::to_string),
        playable_url: v
            .get("playableUrl")
            .and_then(Value::as_str)
            .map(str::to_string),
        binge_group: v
            .get("bingeGroup")
            .and_then(Value::as_str)
            .map(str::to_string),
        filename: v
            .get("filename")
            .and_then(Value::as_str)
            .map(str::to_string),
        effective_filename: v
            .get("effectiveFilename")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}
pub(crate) fn index_of_first_playable<F>(
    streams: &[StreamSelectionItem],
    video_id: &str,
    predicate: F,
) -> Option<usize>
where
    F: Fn(&StreamSelectionItem) -> bool,
{
    streams
        .iter()
        .position(|stream| stream.is_playable_for_episode(video_id) && predicate(stream))
}
pub(crate) fn manual_stream_index(
    streams: &[StreamSelectionItem],
    video_id: &str,
    initial_stream_index: i32,
    saved_url: Option<&str>,
    saved_title: Option<&str>,
) -> i32 {
    let matched_index = saved_url
        .filter(|value| !value.is_empty())
        .and_then(|value| {
            index_of_first_playable(streams, video_id, |stream| {
                stream.playable_url.as_deref() == Some(value)
            })
        })
        .or_else(|| {
            saved_title
                .filter(|value| !value.is_empty())
                .and_then(|value| {
                    index_of_first_playable(streams, video_id, |stream| {
                        stream.title.as_deref() == Some(value)
                    })
                })
        });
    if let Some(index) = matched_index {
        return index as i32;
    }

    if initial_stream_index >= 0
        && streams
            .get(initial_stream_index as usize)
            .is_some_and(|stream| stream.matches_episode(video_id))
    {
        return initial_stream_index;
    }

    streams
        .iter()
        .position(|stream| stream.matches_episode(video_id))
        .map(|index| index as i32)
        .unwrap_or(-1)
}
#[allow(clippy::too_many_arguments)]
fn select_stream_index_inner(
    streams: &[StreamSelectionItem],
    current_video_id: &str,
    initial_stream_index: i32,
    saved_url: Option<&str>,
    saved_title: Option<&str>,
    source_selection_mode: SourceSelectionMode,
    regex_pattern: Option<&str>,
    preferred_binge_group: Option<&str>,
) -> i32 {
    if streams.is_empty() {
        return -1;
    }

    if let Some(group) = preferred_binge_group.filter(|value| !value.trim().is_empty())
        && let Some(index) = index_of_first_playable(streams, current_video_id, |stream| {
            stream.binge_group.as_deref() == Some(group)
        })
    {
        return index as i32;
    }

    match source_selection_mode {
        SourceSelectionMode::Regex => {
            let Some(pattern) = regex_pattern.filter(|value| !value.trim().is_empty()) else {
                return manual_stream_index(
                    streams,
                    current_video_id,
                    initial_stream_index,
                    saved_url,
                    saved_title,
                );
            };
            let regex = match regex::RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
            {
                Ok(regex) => regex,
                Err(_) => {
                    return manual_stream_index(
                        streams,
                        current_video_id,
                        initial_stream_index,
                        saved_url,
                        saved_title,
                    );
                }
            };
            if let Some(index) = index_of_first_playable(streams, current_video_id, |stream| {
                regex.is_match(&stream.selection_text())
            }) {
                return index as i32;
            }
        }
        SourceSelectionMode::First => {
            if let Some(index) = index_of_first_playable(streams, current_video_id, |_| true) {
                return index as i32;
            }
        }
        SourceSelectionMode::Manual => {}
    }

    manual_stream_index(
        streams,
        current_video_id,
        initial_stream_index,
        saved_url,
        saved_title,
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn select_stream_index(
    streams_json: &str,
    current_video_id: &str,
    initial_stream_index: i32,
    saved_url: Option<&str>,
    saved_title: Option<&str>,
    source_selection_mode: SourceSelectionMode,
    regex_pattern: Option<&str>,
    preferred_binge_group: Option<&str>,
) -> i32 {
    let Ok(streams) = serde_json::from_str::<Vec<StreamSelectionItem>>(streams_json) else {
        return -1;
    };
    select_stream_index_inner(
        &streams,
        current_video_id,
        initial_stream_index,
        saved_url,
        saved_title,
        source_selection_mode,
        regex_pattern,
        preferred_binge_group,
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn select_stream_index_values(
    streams: &[Value],
    current_video_id: &str,
    initial_stream_index: i32,
    saved_url: Option<&str>,
    saved_title: Option<&str>,
    source_selection_mode: SourceSelectionMode,
    regex_pattern: Option<&str>,
    preferred_binge_group: Option<&str>,
) -> i32 {
    let items: Vec<StreamSelectionItem> = streams
        .iter()
        .map(stream_selection_item_from_value)
        .collect();
    select_stream_index_inner(
        &items,
        current_video_id,
        initial_stream_index,
        saved_url,
        saved_title,
        source_selection_mode,
        regex_pattern,
        preferred_binge_group,
    )
}
