use crate::ffi::*;

pub(crate) fn route_anilist(_method: &str, _args_json: &str) -> Outcome {
    Err(unknown_method())
}
