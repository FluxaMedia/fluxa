use crate::ffi::*;

pub(crate) fn route_anilist(method: &str, _args_json: &str) -> Outcome {
    match method {
        // args_json IS the meta object
        _ => Err(unknown_method()),
    }
}
