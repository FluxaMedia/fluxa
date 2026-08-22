use super::super::HeadlessEngine;
use super::state::WatchConfig;
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::Serialize;
use serde_json::{Value, json};

pub(super) const WATCH_URL: &str = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&hl=en";
pub(super) const PLAYER_URL: &str = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
pub(super) const DEFAULT_API_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HttpEffectPayload {
    request_id: Option<String>,
    url: String,
    method: String,
    headers: Value,
    body: Option<Value>,
}

pub(super) fn player_script_effect(
    engine: &mut HeadlessEngine,
    generation: u64,
    request_id: &str,
    url: String,
) -> Vec<EffectEnvelope> {
    vec![engine.effect(
        EffectKind::FetchYoutubeTrailerPlayerScript,
        generation,
        HttpEffectPayload {
            request_id: Some(request_id.to_string()),
            url,
            method: "GET".to_string(),
            headers: json!({}),
            body: None,
        },
    )]
}

pub(super) fn watch_config_effect(
    engine: &mut HeadlessEngine,
    generation: u64,
    request_id: Option<String>,
) -> EffectEnvelope {
    engine.effect(
        EffectKind::FetchYoutubeTrailerWatchConfig,
        generation,
        HttpEffectPayload {
            request_id,
            url: WATCH_URL.to_string(),
            method: "GET".to_string(),
            headers: json!({ "Accept-Language": "en-US,en;q=0.9" }),
            body: None,
        },
    )
}

pub(super) fn dispatch_player(
    engine: &mut HeadlessEngine,
    generation: u64,
    request_id: &str,
) -> Vec<EffectEnvelope> {
    let Some(request) = engine.state.trailer.requests.get(request_id).cloned() else {
        return vec![];
    };
    let config = engine
        .state
        .trailer
        .watch_config
        .clone()
        .unwrap_or(WatchConfig {
            api_key: DEFAULT_API_KEY.to_string(),
            visitor_data: None,
            player_script_url: None,
        });
    let mut headers = json!({
        "User-Agent": "com.google.android.apps.youtube.vr.oculus/1.56.21 (Linux; U; Android 12; en_US; Quest 3; Build/SQ3A.220605.009.A1) gzip",
        "X-YouTube-Client-Name": "28",
        "X-YouTube-Client-Version": "1.56.21"
    });
    if let Some(visitor_data) = config.visitor_data
        && let Some(headers) = headers.as_object_mut()
    {
        headers.insert("X-Goog-Visitor-Id".to_string(), Value::String(visitor_data));
    }
    vec![engine.effect(
        EffectKind::FetchYoutubeTrailerPlayer,
        generation,
        HttpEffectPayload {
            request_id: Some(request_id.to_string()),
            url: format!("{PLAYER_URL}&key={}", config.api_key),
            method: "POST".to_string(),
            headers,
            body: Some(json!({
                "videoId": request.video_id,
                "contentCheckOk": true,
                "racyCheckOk": true,
                "context": {
                    "client": {
                        "clientName": "ANDROID_VR",
                        "clientVersion": "1.56.21",
                        "deviceMake": "Oculus",
                        "deviceModel": "Quest 3",
                        "osName": "Android",
                        "osVersion": "12",
                        "platform": "MOBILE",
                        "androidSdkVersion": 32,
                        "hl": "en",
                        "gl": "US"
                    }
                },
                "playbackContext": { "contentPlaybackContext": { "html5Preference": "HTML5_PREF_WANTS" } }
            })),
        },
    )]
}
