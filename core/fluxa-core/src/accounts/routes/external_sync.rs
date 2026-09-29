use crate::ffi::*;
use serde_json::Value;

pub(crate) fn route_external_sync(method: &str, args_json: &str) -> Outcome {
    match method {
        "externalSyncResponseAction" => {
            let args = object(args_json)?;
            let status_code = field(&args, "statusCode")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "statusCode must be a number"))?;
            Ok(Value::String(
                crate::accounts::external_sync::external_sync_response_action(
                    field_str(&args, "provider")?,
                    status_code,
                )
                .to_string(),
            ))
        }
        "externalSyncRefreshRetryAction" => {
            let args = object(args_json)?;
            Ok(Value::String(
                crate::accounts::external_sync::external_sync_refresh_retry_action(
                    args.get("statusCode").and_then(Value::as_i64),
                )
                .to_string(),
            ))
        }
        "externalSyncWorkerRetryAction" => {
            let args = object(args_json)?;
            let status_code = args.get("statusCode").and_then(Value::as_i64);
            let attempt = args.get("attempt").and_then(Value::as_i64).unwrap_or(0);
            Ok(Value::String(
                crate::accounts::external_sync::external_sync_worker_retry_action(
                    status_code,
                    attempt,
                )
                .to_string(),
            ))
        }
        "providerCalendarItems" => {
            opt_json(crate::accounts::external_sync::provider_calendar_items_json(args_json))
        }
        "providerPaginationPlan" => {
            opt_json(crate::accounts::external_sync::provider_pagination_plan_json(args_json))
        }
        "mergeExternalWatchlist" => {
            let args = object(args_json)?;
            into_json(
                crate::accounts::external_sync::merge_external_watchlist_json(
                    field_str(&args, "localJson")?,
                    field_str(&args, "externalJson")?,
                ),
            )
        }
        "mergeExternalWatched" => {
            let args = object(args_json)?;
            into_json(crate::accounts::external_sync::merge_external_watched_json(
                field_str(&args, "localJson")?,
                field_str(&args, "externalJson")?,
            ))
        }
        "pushPlan" => opt_json(crate::accounts::external_sync::push_plan_json(args_json)),
        "importApplyPlan" => opt_json(crate::accounts::external_sync::import_apply_plan_json(
            args_json,
        )),
        "mergeContinueWatchingLists" => {
            let args = object(args_json)?;
            opt_json(
                crate::accounts::external_sync::merge_continue_watching_lists_json(
                    field_str(&args, "localJson")?,
                    field_str(&args, "externalJson")?,
                    field_str(&args, "progressJson")?,
                    args.get("sourceOfTruth").and_then(Value::as_str),
                    args.get("rankingMode").and_then(Value::as_str),
                ),
            )
        }
        "mergeWatchlistTimestamped" => {
            let args = object(args_json)?;
            into_json(
                crate::accounts::external_sync::merge_watchlist_timestamped_json(
                    &field(&args, "local")?.to_string(),
                    &field(&args, "remote")?.to_string(),
                ),
            )
        }
        "mergeWatchedTimestamped" => {
            let args = object(args_json)?;
            into_json(
                crate::accounts::external_sync::merge_watched_timestamped_json(
                    &field(&args, "local")?.to_string(),
                    &field(&args, "remote")?.to_string(),
                ),
            )
        }
        "replaceExternalContinueWatching" => {
            let args = object(args_json)?;
            let provider = args.get("provider").and_then(Value::as_str);
            into_json(
                crate::accounts::external_sync::replace_external_continue_watching_json(
                    field_str(&args, "existingJson")?,
                    provider,
                    field_str(&args, "itemsJson")?,
                    args.get("sourceOfTruth").and_then(Value::as_str),
                    args.get("rankingMode").and_then(Value::as_str),
                    args.get("continueWatchingDays").and_then(Value::as_i64),
                ),
            )
        }
        "promoteExternalProgressPlan" => {
            opt_json(crate::accounts::external_sync::promote_external_progress_plan_json(args_json))
        }
        "externalProviderActionPlan" => {
            opt_json(crate::accounts::external_sync::external_provider_action_plan_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
