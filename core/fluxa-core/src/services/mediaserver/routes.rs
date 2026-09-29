use crate::ffi::*;

pub(crate) fn route_mediaserver(method: &str, args_json: &str) -> Outcome {
    match method {
        "mediaServerRequest" => opt_json(crate::services::mediaserver::request_json(args_json)),
        "mediaServerParse" => opt_json(crate::services::mediaserver::parse_json(args_json)),
        _ => Err(unknown_method()),
    }
}
