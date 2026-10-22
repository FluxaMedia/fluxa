use crate::ffi::*;

pub(crate) fn route_stream_badges(_method: &str, _args_json: &str) -> Outcome {
    Err(unknown_method())
}
