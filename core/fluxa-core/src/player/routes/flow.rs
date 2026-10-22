use crate::ffi::*;

pub(crate) fn route_player_flow(_method: &str, _args_json: &str) -> Outcome {
    Err(unknown_method())
}
