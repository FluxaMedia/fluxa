use crate::ffi::*;
use crate::services;

pub(crate) fn route_simkl(method: &str, args_json: &str) -> Outcome {
    match method {
        "simklSyncPlan" => opt_json(services::simkl::simkl_sync_plan_json(args_json)),
        "simklCalendarPlan" => opt_json(services::simkl::simkl_calendar_plan_json(args_json)),
        "simklSyncApply" => opt_json(services::simkl::simkl_sync_apply_json(args_json)),
        _ => Err(unknown_method()),
    }
}
