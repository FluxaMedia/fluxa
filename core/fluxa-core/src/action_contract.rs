use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SavePlaybackProgressAction {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) profile: Option<Value>,
    pub(crate) meta: Value,
    pub(crate) time_offset: i64,
    pub(crate) duration: i64,
    pub(crate) last_video_id: Option<String>,
    pub(crate) last_stream_index: Option<i32>,
    pub(crate) last_episode_name: Option<String>,
    pub(crate) last_episode_season: Option<i64>,
    pub(crate) last_episode_number: Option<i64>,
    pub(crate) last_episode_thumbnail: Option<String>,
    pub(crate) last_stream_url: Option<String>,
    pub(crate) last_stream_title: Option<String>,
    pub(crate) last_audio_language: Option<String>,
    pub(crate) last_subtitle_language: Option<String>,
    pub(crate) scrobble_trakt_pause: Option<bool>,
    pub(crate) refresh_external_continue_watching: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MarkWatchedAction {
    pub(crate) series_id: String,
    pub(crate) video_ids: Vec<String>,
    pub(crate) watched: Option<bool>,
    pub(crate) meta: Option<Value>,
    pub(crate) episodes: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) profile: Option<Value>,
}

fn tagged_action(action_type: &str, value: Value) -> Value {
    let mut object = value.as_object().cloned().unwrap_or_else(Map::new);
    object.insert("type".to_string(), Value::String(action_type.to_string()));
    Value::Object(object)
}

pub(crate) fn save_playback_progress_action_value(action: &SavePlaybackProgressAction) -> Value {
    tagged_action(
        "savePlaybackProgressRequested",
        serde_json::to_value(action).unwrap_or(Value::Null),
    )
}

pub(crate) fn mark_watched_action_value(action: &MarkWatchedAction) -> Value {
    tagged_action(
        "markWatchedRequested",
        serde_json::to_value(action).unwrap_or(Value::Null),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn shared_action_values_match_headless_action_tags() {
        let progress = save_playback_progress_action_value(&SavePlaybackProgressAction {
            profile: None,
            meta: json!({"id":"tt1"}),
            time_offset: 10,
            duration: 100,
            last_video_id: Some("tt1".to_string()),
            last_stream_index: None,
            last_episode_name: None,
            last_episode_season: None,
            last_episode_number: None,
            last_episode_thumbnail: None,
            last_stream_url: None,
            last_stream_title: None,
            last_audio_language: None,
            last_subtitle_language: None,
            scrobble_trakt_pause: Some(true),
            refresh_external_continue_watching: Some(true),
        });
        assert_eq!(progress["type"], "savePlaybackProgressRequested");
        assert_eq!(progress["timeOffset"], 10);
    }
}
