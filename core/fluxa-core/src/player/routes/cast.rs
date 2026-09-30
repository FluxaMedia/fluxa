use crate::ffi::*;
use crate::player::sessions::cast;
use serde_json::{Value, json};

fn text(args: &Value, name: &str) -> Result<String, CallError> {
    Ok(field_str(args, name)?.to_string())
}

fn maybe_text(args: &Value, name: &str) -> Option<String> {
    args.get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn number(args: &Value, name: &str) -> f64 {
    args.get(name).and_then(Value::as_f64).unwrap_or(0.0)
}

fn bytes(args: &Value, name: &str) -> Result<Vec<u8>, CallError> {
    let items = field(args, name)?
        .as_array()
        .ok_or_else(|| fail(ErrorKind::InvalidArgs, format!("`{name}` must be an array")))?;
    Ok(items
        .iter()
        .filter_map(|item| item.as_u64().map(|byte| byte as u8))
        .collect())
}

fn opt<T: Into<Value>>(value: Option<T>) -> Outcome {
    Ok(value.map(Into::into).unwrap_or(Value::Null))
}

pub(crate) fn route_cast(method: &str, args_json: &str) -> Outcome {
    let args = object(args_json)?;
    match method {
        "castValidUrl" => Ok(json!(cast::validate_stream_url(&text(&args, "url")?))),
        "castLanUrl" => Ok(json!(cast::resolve_loopback_url(
            &text(&args, "url")?,
            &text(&args, "lanIp")?
        ))),
        "castContentType" => Ok(json!(cast::guess_cast_content_type(&text(&args, "url")?))),
        "castDlnaDevice" => opt_json(cast::dlna_parse_device_description_json(
            &text(&args, "xml")?,
            &text(&args, "baseUrl")?,
        )),
        "castDlnaSoap" => Ok(json!(cast::soap_action_body(
            &text(&args, "urn")?,
            &text(&args, "action")?,
            &text(&args, "args")?
        ))),
        "castDlnaLoadArgs" => opt(cast::dlna_set_av_transport_args(
            &text(&args, "mediaUrl")?,
            &text(&args, "title")?,
            maybe_text(&args, "subtitleUrl").as_deref(),
        )),
        "castDlnaSeekArgs" => Ok(json!(cast::dlna_seek_args(number(&args, "position")))),
        "castDlnaVolumeArgs" => Ok(json!(cast::dlna_set_volume_args(number(&args, "level")))),
        "castChromecastEncode" => Ok(json!(cast::encode_cast_message(
            &text(&args, "source")?,
            &text(&args, "destination")?,
            &text(&args, "namespace")?,
            &text(&args, "payload")?,
        ))),
        "castChromecastDecode" => Ok(cast::decode_cast_message(&bytes(&args, "bytes")?)
            .map(|message| json!({"namespace": message.namespace, "payload": message.payload_utf8}))
            .unwrap_or(Value::Null)),
        "castRokuName" => opt(cast::roku_device_name(&text(&args, "xml")?)),
        "castRokuLaunch" => opt(cast::roku_launch_url(
            &text(&args, "host")?,
            &text(&args, "mediaUrl")?,
            maybe_text(&args, "subtitleUrl").as_deref(),
        )),
        "castAirplayBody" => opt(cast::airplay_play_body(&text(&args, "mediaUrl")?)),
        "castAirplayVolume" => Ok(json!(cast::airplay_volume_db(number(&args, "level")))),
        "castFcastEncode" => Ok(cast::fcast_encode_message(
            field_u64(&args, "opcode")? as u8,
            &maybe_text(&args, "body").unwrap_or_default(),
        )
        .map(|message| json!(message))
        .unwrap_or(Value::Null)),
        "castFcastDecode" => Ok(cast::fcast_decode_message(&bytes(&args, "bytes")?)
            .map(|(opcode, body)| json!({"opcode": opcode, "body": body}))
            .unwrap_or(Value::Null)),
        "castFcastPlayBody" => opt(cast::fcast_play_body(
            &text(&args, "mediaUrl")?,
            number(&args, "resume"),
        )),
        "castFcastSeekBody" => Ok(json!(cast::fcast_seek_body(number(&args, "position")))),
        "castFcastVolumeBody" => Ok(json!(cast::fcast_set_volume_body(number(&args, "level")))),
        "castFcastVersionBody" => Ok(json!(cast::fcast_version_body(
            field_u64(&args, "version")? as u32
        ))),
        "castFcastUpdate" => Ok(cast::fcast_playback_update(&text(&args, "body")?)
            .map(|(state, time, duration, speed)| {
                json!({"state": state, "time": time, "duration": duration, "speed": speed})
            })
            .unwrap_or(Value::Null)),
        "castFcastError" => opt(cast::fcast_error_message(&text(&args, "body")?)),
        _ => Err(fail(ErrorKind::UnknownMethod, "unknown cast method")),
    }
}
