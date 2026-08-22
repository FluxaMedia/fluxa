use super::*;

pub(super) fn route_fluxa_sync(method: &str, args_json: &str) -> Outcome {
    match method {
        "fluxaSyncDocuments" => opt_json(fluxa_sync::documents_json(args_json)),
        "fluxaSyncPushPlan" => opt_json(fluxa_sync::push_plan_json(args_json)),
        "fluxaSyncApplyPushResult" => opt_json(fluxa_sync::apply_push_result_json(args_json)),
        "fluxaSyncApplyPull" => opt_json(fluxa_sync::apply_pull_json(args_json)),
        "fluxaSyncProfilePlan" => opt_json(fluxa_sync::profile_plan_json(args_json)),
        _ => Err(unknown_method()),
    }
}
