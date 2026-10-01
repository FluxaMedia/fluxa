use crate::ffi::*;

pub(crate) fn route_stream_policy(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the stream/request JSON
        "torrentRuntimeInfo" => {
            opt_json(crate::player::streams::stream_policy::torrent_runtime_info_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
