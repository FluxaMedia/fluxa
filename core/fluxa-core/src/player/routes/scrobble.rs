use crate::ffi::*;

pub(crate) fn route_player_scrobble(_method: &str, _args_json: &str) -> Outcome {
    Err(unknown_method())
}
