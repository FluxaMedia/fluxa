use crate::ffi::*;

pub(crate) fn route_discord_presence(method: &str, args_json: &str) -> Outcome {
    match method {
        "discordPresenceSnapshot" => {
            opt_json(crate::settings::discord_presence::snapshot_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
