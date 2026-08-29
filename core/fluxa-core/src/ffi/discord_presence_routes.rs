use super::*;
use crate::discord_presence;

pub(super) fn route_discord_presence(method: &str, args_json: &str) -> Outcome {
    match method {
        "discordPresenceSnapshot" => opt_json(discord_presence::snapshot_json(args_json)),
        _ => Err(unknown_method()),
    }
}
