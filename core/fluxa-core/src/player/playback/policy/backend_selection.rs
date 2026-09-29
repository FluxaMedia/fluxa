use crate::runtime::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackendSelectionRequest {
    #[serde(default)]
    stream: Value,
    #[serde(default)]
    preferred_player: Option<String>,
}

pub(crate) fn player_backend_selection_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<BackendSelectionRequest>(request_json)
        .map_err(|e| CoreError::BadInput {
            context: "player_backend_selection_json",
            detail: e.to_string(),
        })
        .log_discard()?;
    let preferred = request.preferred_player.as_deref().unwrap_or("internal");
    let stream = &request.stream;

    let url = stream
        .get("playableUrl")
        .or_else(|| stream.get("url"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let is_external_player_url = url.starts_with("intent://")
        || stream
            .get("externalPlayerUrl")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        || preferred == "external";

    if is_external_player_url {
        return serde_json::to_string(&json!({
            "backend": "external",
            "reason": "external_player_preference"
        }))
        .ok();
    }

    serde_json::to_string(&json!({
        "backend": "mpv",
        "reason": "single_engine"
    }))
    .ok()
}
