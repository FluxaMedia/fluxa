use super::*;

pub(super) fn route_headless_adapter_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "providerAvailabilityPlan" => opt_json(
            addons::headless_adapter::provider_availability_plan_json(args_json),
        ),
        "detailStreamResultPlan" => opt_json(
            addons::headless_adapter::detail_stream_result_plan_json(args_json),
        ),
        "prefetchDetailStreamsPlan" => opt_json(
            addons::headless_adapter::prefetch_detail_streams_plan_json(args_json),
        ),
        "directPlaybackPolicy" => {
            into_json(addons::headless_adapter::direct_playback_policy_json())
        }

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_discovery_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "streamDiscoveryPlan" => opt_json(addons::discovery::stream_discovery_plan_json(args_json)),
        "streamDiscoveryExecutionPolicy" => opt_json(
            addons::discovery::stream_discovery_execution_policy_json(args_json),
        ),
        "streamDiscoveryCachePrefix" => {
            let args = object(args_json)?;
            Ok(Value::String(
                addons::discovery::stream_discovery_cache_prefix(
                    field_str(&args, "contentType")?,
                    field_str(&args, "id")?,
                    field_str(&args, "language")?,
                ),
            ))
        }

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_data_policy(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for all of these
        "cacheEntryPolicy" => opt_json(settings::data_policy::cache_entry_policy_json(args_json)),
        "cacheTrimPolicy" => opt_json(settings::data_policy::cache_trim_policy_json(args_json)),
        "dataFailurePolicy" => opt_json(settings::data_policy::data_failure_policy_json(args_json)),

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_device_resource(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "deviceResourceBudget" => opt_json(settings::device_resource::device_resource_budget_json(
            args_json,
        )),
        "torrentCacheLimitMb" => {
            let args = object(args_json)?;
            let preset = args.get("preset").and_then(Value::as_str);
            let platform = args
                .get("platform")
                .and_then(Value::as_str)
                .unwrap_or("desktop");
            Ok(Value::from(
                settings::device_resource::torrent_cache_limit_mb(preset, platform),
            ))
        }

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_player_flow(method: &str, args_json: &str) -> Outcome {
    match method {
        "playerFlowDispatch" => {
            let args = object(args_json)?;
            opt_json(player::flow::player_flow_dispatch_json(
                field_str(&args, "state")?,
                field_str(&args, "action")?,
            ))
        }

        _ => Err(unknown_method()),
    }
}
