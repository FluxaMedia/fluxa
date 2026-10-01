use crate::ffi::*;

pub(crate) fn route_player_scrobble(method: &str, _args_json: &str) -> Outcome {
    match method {
        _ => Err(unknown_method()),
    }
}
