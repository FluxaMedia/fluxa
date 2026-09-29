use crate::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackendSelectionRequest {
    #[serde(default)]
    stream: Value,
    #[serde(default)]
    preferred_player: Option<String>,
    #[serde(default)]
    device_has_dolby_vision_decoder: bool,
    #[serde(default)]
    device_has_hdr_display: bool,
    #[serde(default)]
    force_software_audio: bool,
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

    // MPV is preferred for:
    // - HDR / Dolby Vision streams when device doesn't have native HW decoder
    // - Streams that specify mpv hints
    // - User explicitly chose MPV
    let has_mpv_hint = stream
        .get("behaviorHints")
        .and_then(|h| h.get("preferMpv"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let is_dv_stream = stream.get("dv").and_then(Value::as_bool).unwrap_or(false)
        || stream
            .get("dolbyVision")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    let is_hdr_stream = stream.get("hdr").and_then(Value::as_bool).unwrap_or(false);
    let needs_mpv_for_hdr = (is_dv_stream && !request.device_has_dolby_vision_decoder)
        || (is_hdr_stream && !request.device_has_hdr_display);

    let use_mpv = preferred == "mpv"
        || has_mpv_hint
        || needs_mpv_for_hdr
        || (request.force_software_audio && preferred != "exoplayer");

    let backend = if use_mpv { "mpv" } else { "exoplayer" };
    let reason = if preferred == "mpv" || preferred == "exoplayer" {
        "user_preference"
    } else if has_mpv_hint {
        "stream_hint"
    } else if needs_mpv_for_hdr {
        "hdr_no_hw_decoder"
    } else if request.force_software_audio {
        "software_audio"
    } else {
        "default"
    };

    serde_json::to_string(&json!({
        "backend": backend,
        "reason": reason
    }))
    .ok()
}
