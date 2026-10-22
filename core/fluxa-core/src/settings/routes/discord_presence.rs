use crate::ffi::*;

pub(crate) fn route_discord_presence(_method: &str, _args_json: &str) -> Outcome {
    Err(unknown_method())
}
