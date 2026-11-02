use crate::player;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerFlowState {
    #[serde(default)]
    pub(crate) current_video_id: Option<String>,
    #[serde(default)]
    pub(crate) current_streams: Vec<Value>,
    #[serde(default)]
    pub(crate) current_stream_index: i32,
    #[serde(default)]
    pub(crate) current_url: Option<String>,
    #[serde(default)]
    pub(crate) zero_speed_ticks: i32,
    #[serde(default)]
    pub(crate) is_buffering: bool,
    #[serde(default)]
    pub(crate) is_video_rendered: bool,
    #[serde(default)]
    pub(crate) player_error: Option<String>,
    #[serde(default)]
    pub(crate) preferred_binge_group: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub(crate) enum PlayerFlowAction {
    #[serde(rename = "loadStreamsRequested")]
    LoadStreamsRequested {
        content_type: String,
        id: String,
        current_video_id: Option<String>,
        initial_video_id: Option<String>,
        initial_streams: Vec<Value>,
        initial_stream_index: i32,
    },
    #[serde(rename = "streamsLoaded")]
    StreamsLoaded {
        streams: Vec<Value>,
        current_video_id: Option<String>,
        initial_stream_index: i32,
        saved_url: Option<String>,
        saved_title: Option<String>,
        source_selection_mode: Option<String>,
        regex_pattern: Option<String>,
        preferred_binge_group: Option<String>,
    },
    #[serde(rename = "streamsFailed")]
    StreamsFailed { error_code: Option<String> },
}

#[derive(Clone, Debug, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub(crate) enum PlayerFlowEffect {
    #[serde(rename = "loadStreams")]
    LoadStreams {
        content_type: String,
        id: String,
        use_initial_streams: bool,
    },
}

pub(crate) fn dispatch(
    state: &mut PlayerFlowState,
    action: PlayerFlowAction,
) -> Vec<PlayerFlowEffect> {
    match action {
        PlayerFlowAction::LoadStreamsRequested {
            content_type,
            id,
            current_video_id,
            initial_video_id,
            initial_streams,
            initial_stream_index,
        } => {
            state.current_video_id = current_video_id.clone();
            state.current_streams.clear();
            state.current_stream_index = initial_stream_index;
            state.current_url = None;
            state.zero_speed_ticks = 0;
            state.is_buffering = true;
            state.is_video_rendered = false;
            state.player_error = None;
            let use_initial_streams =
                !initial_streams.is_empty() && current_video_id == initial_video_id;
            vec![PlayerFlowEffect::LoadStreams {
                content_type,
                id,
                use_initial_streams,
            }]
        }
        PlayerFlowAction::StreamsLoaded {
            streams,
            current_video_id,
            initial_stream_index,
            saved_url,
            saved_title,
            source_selection_mode,
            regex_pattern,
            preferred_binge_group,
        } => {
            if streams.is_empty() {
                state.current_streams.clear();
                state.current_url = None;
                state.is_buffering = false;
                state.player_error = Some("no_source".to_string());
                return vec![];
            }

            let selected_index = player::streams::stream_policy::select_stream_index_values(
                &streams,
                current_video_id.as_deref().unwrap_or_default(),
                initial_stream_index,
                saved_url.as_deref(),
                saved_title.as_deref(),
                source_selection_mode.as_deref().unwrap_or("manual").into(),
                regex_pattern.as_deref(),
                preferred_binge_group.as_deref(),
            )
            .clamp(0, streams.len().saturating_sub(1) as i32);

            state.current_streams = streams;
            state.current_stream_index = selected_index;
            state.current_url = state
                .current_streams
                .get(selected_index as usize)
                .and_then(playable_url);
            state.is_buffering = state.current_url.is_none();
            state.is_video_rendered = false;
            state.player_error = None;
            state.preferred_binge_group = None;
            vec![]
        }
        PlayerFlowAction::StreamsFailed { error_code } => {
            state.current_url = None;
            state.is_buffering = false;
            state.player_error = Some(error_code.unwrap_or_else(|| "generic".to_string()));
            vec![]
        }
    }
}

pub(crate) fn playable_url(stream: &Value) -> Option<String> {
    stream
        .get("playableUrl")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
        .or_else(|| player::streams::stream_policy::stream_playable_url(stream))
}

pub(crate) fn next_failover_index(streams: &[Value], tried: &[i64]) -> Option<usize> {
    let untried = |index: &usize| {
        !tried.contains(&(*index as i64)) && streams.get(*index).and_then(playable_url).is_some()
    };
    let last = tried.last().copied().unwrap_or(-1).max(-1) + 1;
    (last as usize..streams.len())
        .find(untried)
        .or_else(|| (0..streams.len()).find(untried))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn streams() -> Vec<Value> {
        vec![
            json!({"url": "http://a/1"}),
            json!({"name": "no url"}),
            json!({"url": "http://a/3"}),
            json!({"url": "http://a/4"}),
        ]
    }

    #[test]
    fn failover_skips_streams_without_a_url() {
        assert_eq!(next_failover_index(&streams(), &[0]), Some(2));
    }

    #[test]
    fn failover_wraps_to_earlier_untried_streams() {
        assert_eq!(next_failover_index(&streams(), &[2, 3]), Some(0));
    }

    #[test]
    fn failover_gives_up_when_everything_failed() {
        assert_eq!(next_failover_index(&streams(), &[0, 2, 3]), None);
    }
}
