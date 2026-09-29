use super::*;

pub(super) fn route_anime_detection(method: &str, args_json: &str) -> Outcome {
    match method {
        "detectAnimePlayback" => {
            let args = object(args_json)?;
            let empty: Vec<Value> = Vec::new();
            let addons = args
                .get("addons")
                .and_then(Value::as_array)
                .unwrap_or(&empty);
            Ok(catalog::anime::detect_anime_playback(
                args.get("meta").unwrap_or(&Value::Null),
                args.get("episode").unwrap_or(&Value::Null),
                args.get("stream").unwrap_or(&Value::Null),
                addons,
            ))
        }
        // args_json IS the meta object
        "shouldAttemptAnimeTracking" => Ok(json!(catalog::anime::should_attempt_anime_tracking(
            &object(args_json)?
        ))),

        _ => Err(unknown_method()),
    }
}
