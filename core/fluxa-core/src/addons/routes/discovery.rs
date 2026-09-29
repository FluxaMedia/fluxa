use crate::ffi::*;
use serde_json::Value;

pub(crate) fn route_headless_adapter_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "providerAvailabilityPlan" => opt_json(
            crate::addons::streams::headless_adapter::provider_availability_plan_json(args_json),
        ),
        "detailStreamResultPlan" => opt_json(
            crate::addons::streams::headless_adapter::detail_stream_result_plan_json(args_json),
        ),
        "prefetchDetailStreamsPlan" => opt_json(
            crate::addons::streams::headless_adapter::prefetch_detail_streams_plan_json(args_json),
        ),
        "directPlaybackPolicy" => {
            into_json(crate::addons::streams::headless_adapter::direct_playback_policy_json())
        }

        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_discovery_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "streamDiscoveryPlan" => {
            opt_json(crate::addons::streams::discovery::stream_discovery_plan_json(args_json))
        }
        "streamDiscoveryExecutionPolicy" => opt_json(
            crate::addons::streams::discovery::stream_discovery_execution_policy_json(args_json),
        ),
        "streamDiscoveryCachePrefix" => {
            let args = object(args_json)?;
            Ok(Value::String(
                crate::addons::streams::discovery::stream_discovery_cache_prefix(
                    field_str(&args, "contentType")?,
                    field_str(&args, "id")?,
                    field_str(&args, "language")?,
                ),
            ))
        }

        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_addon_uptime(method: &str, args_json: &str) -> Outcome {
    match method {
        "addonUptimeMatchPlan" => {
            opt_json(crate::addons::streams::uptime::addon_uptime_match_plan_json(args_json))
        }
        _ => Err(fail(
            ErrorKind::UnknownMethod,
            "unknown addon uptime method",
        )),
    }
}
