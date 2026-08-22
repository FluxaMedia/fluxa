use crate::stream_policy;
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
#[serde(rename_all = "camelCase")]
struct PlayerFlowResult {
    state: PlayerFlowState,
    effects: Vec<PlayerFlowEffect>,
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

pub(crate) fn player_flow_dispatch_json(state_json: &str, action_json: &str) -> Option<String> {
    let mut state: PlayerFlowState = serde_json::from_str(state_json).unwrap_or_default();
    let action: PlayerFlowAction = serde_json::from_str(action_json).ok()?;
    let effects = dispatch(&mut state, action);
    serde_json::to_string(&PlayerFlowResult { state, effects }).ok()
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

            let selected_index = stream_policy::select_stream_index_values(
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

fn playable_url(stream: &Value) -> Option<String> {
    stream
        .get("playableUrl")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
        .or_else(|| stream_policy::stream_playable_url(stream))
}

#[cfg(test)]
mod tests {
    use super::player_flow_dispatch_json;
    use serde_json::Value;

    #[test]
    fn load_request_returns_effect_and_resets_playback_state() {
        let result: Value = serde_json::from_str(
            &player_flow_dispatch_json(
                r#"{"currentUrl":"http://old","isVideoRendered":true}"#,
                r#"{"type":"loadStreamsRequested","contentType":"series","id":"tt1:1:2","currentVideoId":"tt1:1:2","initialVideoId":"tt1:1:2","initialStreams":[{"playableUrl":"http://s"}],"initialStreamIndex":2}"#,
            )
            .expect("result"),
        )
        .expect("json");

        assert_eq!(result["state"]["currentUrl"], Value::Null);
        assert_eq!(result["state"]["isBuffering"], true);
        assert_eq!(result["effects"][0]["type"], "loadStreams");
        assert_eq!(result["effects"][0]["useInitialStreams"], true);
    }

    #[test]
    fn loaded_streams_select_url_without_reordering_provider_results() {
        let result: Value = serde_json::from_str(
            &player_flow_dispatch_json(
                "{}",
                r#"{"type":"streamsLoaded","streams":[{"title":"A","playableUrl":"http://a"},{"title":"B","playableUrl":"http://b"}],"currentVideoId":"tt1","initialStreamIndex":1,"sourceSelectionMode":"manual"}"#,
            )
            .expect("result"),
        )
        .expect("json");

        assert_eq!(result["state"]["currentStreamIndex"], 1);
        assert_eq!(result["state"]["currentUrl"], "http://b");
        assert_eq!(result["state"]["currentStreams"][0]["title"], "A");
        assert_eq!(result["state"]["currentStreams"][1]["title"], "B");
    }

    #[test]
    fn empty_streams_return_no_source_error_code() {
        let result: Value = serde_json::from_str(
            &player_flow_dispatch_json(
                "{}",
                r#"{"type":"streamsLoaded","streams":[],"initialStreamIndex":0}"#,
            )
            .expect("result"),
        )
        .expect("json");

        assert_eq!(result["state"]["playerError"], "no_source");
        assert_eq!(result["state"]["isBuffering"], false);
    }
}
