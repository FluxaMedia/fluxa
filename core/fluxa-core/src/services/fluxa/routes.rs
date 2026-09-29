use crate::ffi::*;

pub(crate) fn route_fluxa_sync(method: &str, args_json: &str) -> Outcome {
    match method {
        "fluxaSyncDocuments" => opt_json(crate::services::fluxa::documents_json(args_json)),
        "fluxaSyncPushPlan" => opt_json(crate::services::fluxa::push_plan_json(args_json)),
        "fluxaSyncApplyPushResult" => {
            opt_json(crate::services::fluxa::apply_push_result_json(args_json))
        }
        "fluxaSyncApplyPull" => opt_json(crate::services::fluxa::apply_pull_json(args_json)),
        "fluxaSyncProfilePlan" => opt_json(crate::services::fluxa::profile_plan_json(args_json)),
        _ => Err(unknown_method()),
    }
}
