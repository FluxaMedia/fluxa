use crate::ffi::*;

pub(crate) fn route_media_session(method: &str, args_json: &str) -> Outcome {
    match method {
        "playerMediaCommandPlan" => {
            opt_json(crate::player::media_session::player_media_command_plan_json(args_json))
        }
        "playerMediaSessionPlan" => {
            opt_json(crate::player::media_session::player_media_session_plan_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
