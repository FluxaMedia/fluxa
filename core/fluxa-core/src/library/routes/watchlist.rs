use crate::ffi::*;

pub(crate) fn route_watchlist(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "libraryCommandPlan" => opt_json(crate::library::watchlist::library_command_plan_json(
            args_json,
        )),
        "playbackProgressWritePlan" => {
            opt_json(crate::library::watchlist::playback_progress_write_plan_json(args_json))
        }
        "libraryViewPlan" => opt_json(crate::library::watchlist::library_view_plan_json(args_json)),
        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_offline(method: &str, _args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        _ => Err(unknown_method()),
    }
}
