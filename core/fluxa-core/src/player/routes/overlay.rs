use crate::ffi::*;

pub(crate) fn route_player_overlay(method: &str, args_json: &str) -> Outcome {
    match method {
        "playerOverlayPlan" => {
            opt_json(crate::player::overlay::player_overlay_plan_json(args_json))
        }
        "playerToastPlan" => opt_json(crate::player::overlay::player_toast_plan_json(args_json)),
        "playerTrackPlan" => opt_json(crate::player::overlay::player_track_plan_json(args_json)),
        "playerTickPlan" => opt_json(crate::player::overlay::player_tick_plan_json(args_json)),
        "playerXrayCreditsPlan" => opt_json(crate::player::xray::player_xray_credits_plan_json(args_json)),
        "playerXrayFindId" => opt_json(crate::player::xray::player_xray_find_id_json(args_json)),
        "playerXrayCast" => opt_json(crate::player::xray::player_xray_cast_json(args_json)),
        "playerXrayRows" => opt_json(crate::player::xray::player_xray_rows_json(args_json)),
        "playerXrayToggle" => opt_json(crate::player::xray::player_xray_toggle_json(args_json)),
        _ => Err(unknown_method()),
    }
}
