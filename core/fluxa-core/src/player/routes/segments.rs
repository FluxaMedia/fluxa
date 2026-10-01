use crate::ffi::*;

pub(crate) fn route_segments(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the data JSON for single-arg methods
        "playerSegmentsStep" => opt_json(crate::player::segments::player_segments_step_json(
            args_json,
        )),
        "playerSegmentsSubmitPlan" => opt_json(
            crate::player::segments::player_segments_submit_plan_json(args_json),
        ),
        _ => Err(unknown_method()),
    }
}
