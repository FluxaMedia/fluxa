use super::*;
use crate::settings;

pub(super) fn route_discord_presence(method: &str, args_json: &str) -> Outcome {
    match method {
        "discordPresenceSnapshot" => opt_json(settings::discord_presence::snapshot_json(args_json)),
        _ => Err(unknown_method()),
    }
}
