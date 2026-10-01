use crate::ffi::*;

pub(crate) fn route_discord_presence(method: &str, _args_json: &str) -> Outcome {
    match method {
        _ => Err(unknown_method()),
    }
}
