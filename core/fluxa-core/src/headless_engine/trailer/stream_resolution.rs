#[cfg(feature = "plugin-js-engine")]
use super::super::youtube_cipher;
use serde_json::{Value, json};

pub(super) fn resolve_player_response(
    response: &Value,
    max_height: Option<u32>,
    player_js: Option<&str>,
) -> Value {
    let payload = response
        .get("body")
        .and_then(Value::as_str)
        .and_then(|body| serde_json::from_str::<Value>(body).ok());
    let Some(payload) = payload else {
        return Value::Null;
    };
    if payload
        .pointer("/playabilityStatus/status")
        .and_then(Value::as_str)
        != Some("OK")
    {
        return Value::Null;
    }
    let adaptive_pair = best_adaptive_pair(
        payload.pointer("/streamingData/adaptiveFormats"),
        max_height,
        player_js,
    );
    let progressive_url = first_direct_url(payload.pointer("/streamingData/formats"), player_js);
    let stream_url = adaptive_pair
        .as_ref()
        .map(|(video_url, _, _)| video_url.to_owned())
        .or(progressive_url)
        .or_else(|| {
            payload
                .pointer("/streamingData/hlsManifestUrl")
                .and_then(Value::as_str)
                .map(str::to_owned)
        });
    let Some(stream_url) = stream_url else {
        return Value::Null;
    };
    let audio_url = adaptive_pair
        .as_ref()
        .filter(|(video_url, _, _)| *video_url == stream_url)
        .map(|(_, audio_url, _)| audio_url.clone());
    let height = adaptive_pair
        .as_ref()
        .filter(|(video_url, _, _)| *video_url == stream_url)
        .map(|(_, _, height)| *height);
    let subtitles = payload
        .pointer("/captions/playerCaptionsTracklistRenderer/captionTracks")
        .and_then(Value::as_array)
        .map(|tracks| tracks.iter().filter_map(caption_track).collect::<Vec<_>>())
        .unwrap_or_default();
    json!({ "status": "ok", "streamUrl": stream_url, "audioUrl": audio_url, "height": height, "subtitles": subtitles })
}

pub(super) fn player_script_url(response: &Value) -> Option<String> {
    response
        .get("body")?
        .as_str()
        .and_then(|body| serde_json::from_str::<Value>(body).ok())?
        .pointer("/assets/js")?
        .as_str()
        .map(normalize_youtube_url)
}

pub(super) fn normalize_youtube_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://www.youtube.com{url}")
    }
}

pub(super) fn requires_player_script(response: &Value) -> bool {
    response
        .get("body")
        .and_then(Value::as_str)
        .and_then(|body| serde_json::from_str::<Value>(body).ok())
        .is_some_and(|payload| {
            ["/streamingData/adaptiveFormats", "/streamingData/formats"]
                .iter()
                .filter_map(|pointer| payload.pointer(pointer).and_then(Value::as_array))
                .flatten()
                .any(format_requires_player_script)
        })
}

fn format_requires_player_script(format: &Value) -> bool {
    format.get("signatureCipher").is_some()
        || format.get("cipher").is_some()
        || format
            .get("url")
            .and_then(Value::as_str)
            .is_some_and(|url| {
                url.split_once('?').is_some_and(|(_, query)| {
                    query.split('#').next().is_some_and(|query| {
                        query
                            .split('&')
                            .any(|entry| entry.split_once('=').is_some_and(|(key, _)| key == "n"))
                    })
                })
            })
}

pub(super) fn first_direct_url(formats: Option<&Value>, player_js: Option<&str>) -> Option<String> {
    formats?
        .as_array()?
        .iter()
        .find_map(|format| format_url(format, player_js))
}

pub(super) fn best_adaptive_pair(
    formats: Option<&Value>,
    max_height: Option<u32>,
    player_js: Option<&str>,
) -> Option<(String, String, u32)> {
    let entries = formats?.as_array()?;
    let video = entries
        .iter()
        .filter(|format| format_url(format, player_js).is_some())
        .filter(|format| {
            format
                .get("mimeType")
                .and_then(Value::as_str)
                .is_some_and(|mime_type| mime_type.starts_with("video/mp4; codecs=\"avc1"))
        })
        .filter(|format| {
            max_height.is_none_or(|limit| {
                format
                    .get("height")
                    .and_then(Value::as_u64)
                    .is_some_and(|height| height <= limit as u64)
            })
        })
        .max_by_key(|format| {
            (
                format.get("height").and_then(Value::as_i64).unwrap_or(0),
                format.get("bitrate").and_then(Value::as_i64).unwrap_or(0),
            )
        })?;
    let height = video.get("height")?.as_u64()? as u32;
    let video = format_url(video, player_js)?;
    let audio_format = entries
        .iter()
        .filter(|format| format_url(format, player_js).is_some())
        .filter(|format| {
            format
                .get("mimeType")
                .and_then(Value::as_str)
                .is_some_and(|mime_type| mime_type.starts_with("audio/mp4"))
        })
        .max_by_key(|format| format.get("bitrate").and_then(Value::as_i64).unwrap_or(0))?;
    let audio = format_url(audio_format, player_js)?;
    Some((video, audio, height))
}

#[cfg(feature = "plugin-js-engine")]
fn format_url(format: &Value, player_js: Option<&str>) -> Option<String> {
    format
        .get("url")
        .and_then(Value::as_str)
        .and_then(|url| youtube_cipher::resolve_url(url, player_js))
        .or_else(|| {
            player_js.and_then(|script| {
                format
                    .get("signatureCipher")
                    .or_else(|| format.get("cipher"))
                    .and_then(Value::as_str)
                    .and_then(|cipher| youtube_cipher::decipher_url(cipher, script))
            })
        })
}

#[cfg(not(feature = "plugin-js-engine"))]
fn format_url(format: &Value, _player_js: Option<&str>) -> Option<String> {
    format.get("url").and_then(Value::as_str).map(str::to_owned)
}

fn caption_track(track: &Value) -> Option<Value> {
    Some(json!({
        "languageTag": track.get("languageCode").and_then(Value::as_str).unwrap_or("und"),
        "label": track.pointer("/name/simpleText").and_then(Value::as_str).or_else(|| track.get("languageCode").and_then(Value::as_str)).unwrap_or(""),
        "url": track.get("baseUrl")?.as_str()?,
        "mimeType": "text/vtt",
        "isAuto": track.get("kind").and_then(Value::as_str) == Some("asr")
    }))
}
