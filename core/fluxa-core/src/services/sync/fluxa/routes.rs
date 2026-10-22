use crate::ffi::*;

pub(crate) fn route_fluxa_sync(_method: &str, _args_json: &str) -> Outcome {
    Err(unknown_method())
}
