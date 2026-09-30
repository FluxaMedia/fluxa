use crate::ffi::*;
use serde_json::Value;

pub(crate) fn route_external_sync(method: &str, args_json: &str) -> Outcome {
    match method {
        "providerCalendarItems" => opt_json(
            crate::services::tracking::external_sync::provider_calendar_items_json(args_json),
        ),
        "replaceExternalContinueWatching" => {
            let args = object(args_json)?;
            let provider = args.get("provider").and_then(Value::as_str);
            into_json(
                crate::library::continue_watching::replace_external_continue_watching_json(
                    field_str(&args, "existingJson")?,
                    provider,
                    field_str(&args, "itemsJson")?,
                    args.get("sourceOfTruth").and_then(Value::as_str),
                    args.get("rankingMode").and_then(Value::as_str),
                    args.get("continueWatchingDays").and_then(Value::as_i64),
                ),
            )
        }
        _ => Err(unknown_method()),
    }
}
