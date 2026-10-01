use crate::ffi::*;

pub(crate) fn route_headless_adapter_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "providerAvailabilityPlan" => opt_json(
            crate::addons::streams::headless_adapter::provider_availability_plan_json(args_json),
        ),
        "detailStreamResultPlan" => opt_json(
            crate::addons::streams::headless_adapter::detail_stream_result_plan_json(args_json),
        ),
        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_discovery_plan(method: &str, _args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_addon_uptime(method: &str, _args_json: &str) -> Outcome {
    match method {
        _ => Err(fail(
            ErrorKind::UnknownMethod,
            "unknown addon uptime method",
        )),
    }
}
