use super::*;
use crate::watch_together;

pub(super) fn route_watch_together(method: &str, args_json: &str) -> Outcome {
    let args = object(args_json)?;
    match method {
        "watchTogetherCreate" => Ok(watch_together::WatchTogetherProtocol::create(
            field_str(&args, "displayName")?,
        )),
        "watchTogetherJoin" => Ok(watch_together::WatchTogetherProtocol::join(
            field_str(&args, "roomCode")?,
            field_str(&args, "displayName")?,
        )),
        "watchTogetherLeave" => Ok(watch_together::WatchTogetherProtocol::leave()),
        "watchTogetherPing" => Ok(watch_together::WatchTogetherProtocol::ping(
            field(&args, "clientTimeMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "clientTimeMs must be a number"))?,
        )),
        "watchTogetherBuffering" => Ok(watch_together::WatchTogetherProtocol::buffering(
            field(&args, "buffering")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "buffering must be a boolean"))?,
        )),
        "watchTogetherContent" => {
            let content = serde_json::from_value(field(&args, "content")?.clone())
                .map_err(|_| fail(ErrorKind::InvalidArgs, "content is invalid"))?;
            Ok(watch_together::WatchTogetherProtocol::content(&content))
        }
        "watchTogetherPlaybackState" => {
            let snapshot = serde_json::from_value(field(&args, "snapshot")?.clone())
                .map_err(|_| fail(ErrorKind::InvalidArgs, "snapshot is invalid"))?;
            let content = args
                .get("content")
                .map(|value| {
                    serde_json::from_value(value.clone())
                        .map_err(|_| fail(ErrorKind::InvalidArgs, "content is invalid"))
                })
                .transpose()?;
            Ok(watch_together::WatchTogetherProtocol::playback_state(
                &snapshot,
                content.as_ref(),
            ))
        }
        "watchTogetherRoomCodeSpec" => Ok(json!({
            "alphabet": watch_together::WatchTogetherProtocol::ROOM_CODE_ALPHABET,
            "length": watch_together::WatchTogetherProtocol::ROOM_CODE_LENGTH,
        })),
        "watchTogetherDecode" => {
            let text = field_str(&args, "text")?;
            let local_content = args
                .get("localContent")
                .filter(|value| !value.is_null())
                .map(|value| {
                    serde_json::from_value(value.clone())
                        .map_err(|_| fail(ErrorKind::InvalidArgs, "localContent is invalid"))
                })
                .transpose()?;
            let value: serde_json::Value = match serde_json::from_str(text) {
                Ok(value) => value,
                Err(_) => serde_json::Value::Null,
            };
            let decoded = watch_together::WatchTogetherProtocol::decode(
                &value,
                local_content.as_ref(),
            );
            serde_json::to_value(decoded)
                .map_err(|_| fail(ErrorKind::Internal, "failed to encode decoded message"))
        }
        "watchTogetherDriftCorrection" => {
            let correction = watch_together::WatchTogetherDriftPolicy::correction(
                field(&args, "localPositionMs")?.as_i64().ok_or_else(|| {
                    fail(ErrorKind::InvalidArgs, "localPositionMs must be a number")
                })?,
                field(&args, "expectedPositionMs")?.as_i64().ok_or_else(|| {
                    fail(ErrorKind::InvalidArgs, "expectedPositionMs must be a number")
                })?,
                field(&args, "hostPlaying")?
                    .as_bool()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "hostPlaying must be a boolean"))?,
                field(&args, "speedCorrectionActive")?
                    .as_bool()
                    .ok_or_else(|| {
                        fail(
                            ErrorKind::InvalidArgs,
                            "speedCorrectionActive must be a boolean",
                        )
                    })?,
            );
            Ok(match correction {
                watch_together::WatchTogetherCorrection::None => json!({"type": "none"}),
                watch_together::WatchTogetherCorrection::Seek(position_ms) => {
                    json!({"type": "seek", "positionMs": position_ms})
                }
                watch_together::WatchTogetherCorrection::Speed(speed) => {
                    json!({"type": "speed", "value": speed})
                }
                watch_together::WatchTogetherCorrection::ResetSpeed => {
                    json!({"type": "resetSpeed"})
                }
            })
        }
        _ => Err(unknown_method()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_exposes_wire_protocol_for_wasm_and_shells() {
        let result = match route_watch_together(
            "watchTogetherPlaybackState",
            r#"{"snapshot":{"positionMs":12000,"durationMs":30000,"isPlaying":true,"isBuffering":false},"content":{"id":"tt123","contentType":"series","videoId":"tt123:1:2","title":"Episode 2"}}"#,
        ) {
            Ok(value) => value,
            Err(_) => panic!(),
        };
        assert_eq!(result["type"], "state");
        assert_eq!(result["positionMs"], 12000);
        assert_eq!(result["playing"], true);
    }
}
