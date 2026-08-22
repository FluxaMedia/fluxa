use crate::stream_policy::stream_playback_info_json;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackPrepareRequest {
    stream: Value,
    #[serde(default)]
    meta: Option<Value>,
    #[serde(default)]
    episode: Option<Value>,
    #[serde(default)]
    preferred_player: Option<String>,
}

pub(crate) fn playback_prepare_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<PlaybackPrepareRequest>(request_json).ok()?;
    let info = stream_playback_info_json(&request.stream.to_string())
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or(Value::Null);
    let playable_url = info
        .get("playableUrl")
        .or_else(|| request.stream.get("playableUrl"))
        .or_else(|| request.stream.get("url"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let is_torrent = info
        .get("isTorrentPlaybackUrl")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || playable_url.starts_with("stremio://torrent/")
        || request
            .stream
            .get("infoHash")
            .and_then(Value::as_str)
            .is_some();
    let compatible = info
        .get("isLikelyPlayerCompatible")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let external_url = info
        .get("externalUrl")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let wants_external = request
        .preferred_player
        .as_deref()
        .is_some_and(|player| player.eq_ignore_ascii_case("external"));
    let mode = if wants_external && (!playable_url.is_empty() || external_url.is_some()) {
        "external"
    } else if playable_url.is_empty() && external_url.is_some() {
        "external"
    } else if playable_url.is_empty() || !compatible {
        "reject"
    } else if is_torrent {
        "torrent"
    } else {
        "direct"
    };
    serde_json::to_string(&json!({
        "mode": mode,
        "url": if mode == "external" { external_url.clone().unwrap_or(playable_url.clone()) } else { playable_url.clone() },
        "isTorrent": is_torrent,
        "rejectReason": if playable_url.is_empty() && external_url.is_none() { "missing_playable_url" } else if !compatible { "incompatible_stream" } else { "" },
        "subtitleExtraArgs": info.get("subtitleExtraArgs").cloned().unwrap_or(Value::Null),
        "title": playback_title(request.meta.as_ref(), request.episode.as_ref(), &request.stream),
        "artwork": playback_artwork(request.meta.as_ref(), request.episode.as_ref()),
        "preferredPlayer": request.preferred_player.unwrap_or_else(|| "mpv".to_string())
    }))
    .ok()
}

fn playback_title(meta: Option<&Value>, episode: Option<&Value>, stream: &Value) -> Value {
    let content_title = meta
        .and_then(|value| value.get("name"))
        .or_else(|| stream.get("title"))
        .or_else(|| stream.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("Fluxa");
    let season = episode
        .and_then(|value| value.get("season"))
        .and_then(Value::as_i64);
    let episode_number = episode
        .and_then(|value| value.get("episode").or_else(|| value.get("number")))
        .and_then(Value::as_i64);
    let episode_name = episode
        .and_then(|value| value.get("name").or_else(|| value.get("title")))
        .and_then(Value::as_str);
    let episode_line = match (season, episode_number) {
        (Some(season), Some(number)) => {
            let prefix = format!("S{season}:E{number}");
            Some(
                match episode_name.filter(|value| !value.trim().is_empty()) {
                    Some(name) => format!("{prefix} {}", name.trim()),
                    None => prefix,
                },
            )
        }
        _ => None,
    };
    json!({ "contentTitle": content_title, "episodeLine": episode_line })
}
fn playback_artwork(meta: Option<&Value>, episode: Option<&Value>) -> Value {
    let background = meta
        .and_then(|value| {
            first_text(
                value,
                &["background", "backgroundUrl", "backdrop", "backdropUrl"],
            )
        })
        .or_else(|| {
            episode
                .and_then(|value| value.get("thumbnail"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            meta.and_then(|value| value.get("poster"))
                .and_then(Value::as_str)
        });
    let logo =
        meta.and_then(|value| first_text(value, &["logo", "logoUrl", "titleLogo", "titleLogoUrl"]));
    json!({ "background": background, "logo": logo })
}
fn first_text<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_preference_sends_a_playable_stream_to_the_platform_launcher() {
        let result = playback_prepare_plan_json(
            r#"{"stream":{"url":"https://example.com/video.mp4"},"preferredPlayer":"external"}"#,
        )
        .unwrap();
        let result: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(result["mode"], "external");
        assert_eq!(result["url"], "https://example.com/video.mp4");
    }
}
