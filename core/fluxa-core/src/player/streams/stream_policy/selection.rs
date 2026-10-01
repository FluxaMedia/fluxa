use super::meta::SourceSelectionMode;
use crate::catalog::identity::stream_matches_episode;
use serde_json::Value;

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
    effective_filename: Option<String>
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
            .map(str::to_string)
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
