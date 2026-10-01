use crate::ffi::*;
use serde_json::json;

pub(crate) fn route_anime_detection(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the meta object
        "shouldAttemptAnimeTracking" => Ok(json!(
            crate::catalog::anime::should_attempt_anime_tracking(&object(args_json)?)
        )),

        _ => Err(unknown_method()),
    }
}
