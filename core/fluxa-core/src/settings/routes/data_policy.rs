use crate::ffi::*;
use serde_json::Value;

pub(crate) fn route_data_policy(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for all of these
        "cacheEntryPolicy" => opt_json(crate::settings::data_policy::cache_entry_policy_json(
            args_json,
        )),
        "cacheTrimPolicy" => opt_json(crate::settings::data_policy::cache_trim_policy_json(
            args_json,
        )),
        "dataFailurePolicy" => opt_json(crate::settings::data_policy::data_failure_policy_json(
            args_json,
        )),

        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_device_resource(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "deviceResourceBudget" => {
            opt_json(crate::settings::device_resource::device_resource_budget_json(args_json))
        }
        "torrentCacheLimitMb" => {
            let args = object(args_json)?;
            let preset = args.get("preset").and_then(Value::as_str);
            let platform = args
                .get("platform")
                .and_then(Value::as_str)
                .unwrap_or("desktop");
            Ok(Value::from(
                crate::settings::device_resource::torrent_cache_limit_mb(preset, platform),
            ))
        }

        _ => Err(unknown_method()),
    }
}
