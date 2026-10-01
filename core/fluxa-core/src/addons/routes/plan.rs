use crate::ffi::*;
use serde_json::Value;

pub(crate) fn route_resource_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // Repository / resource flow — args_json IS the request object
        "addonResourceRequestPlan" => opt_json(
            crate::addons::sources::repository::addon_resource_request_plan_json(args_json),
        ),
        "resourceFetchPlan" => opt_json(
            crate::addons::streams::platform::resource_fetch_plan_json(args_json),
        ),
        "resourceFetchExecutionPolicy" => opt_json(
            crate::addons::streams::platform::resource_fetch_execution_policy_json(args_json),
        ),
        // Platform plan — args_json IS the request object
        "playbackPreparePlan" => {
            opt_json(crate::addons::streams::platform::playback_prepare_plan_json(args_json))
        }
        "resourceKindToResource" => {
            let args = object(args_json)?;
            Ok(Value::String(
                crate::addons::streams::platform::resource_kind_to_resource(
                    field_str(&args, "kind")?,
                    args.get("requestResource").and_then(Value::as_str),
                    args.get("itemResource").and_then(Value::as_str),
                ),
            ))
        }
        "parseAndPlanAddonResource" => {
            let args = object(args_json)?;
            let body = args.get("body").and_then(Value::as_str).map(str::to_string);
            let status_code = field(&args, "statusCode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "statusCode must be a number"))?
                as i32;
            let addon_name = args
                .get("addonName")
                .and_then(Value::as_str)
                .map(str::to_string);
            let season = args.get("season").and_then(Value::as_i64);
            into_json(
                crate::addons::streams::platform::parse_and_plan_addon_resource_json(
                    field_str(&args, "resource")?,
                    field_str(&args, "url")?,
                    status_code,
                    body.as_deref(),
                    field_str(&args, "kind")?,
                    addon_name.as_deref(),
                    season,
                ),
            )
        }

        _ => Err(unknown_method()),
    }
}
