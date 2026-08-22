use super::*;
use crate::nuvio_pin;

pub(super) fn route_anime_detection(method: &str, args_json: &str) -> Outcome {
    match method {
        "detectAnimePlayback" => {
            let args = object(args_json)?;
            let empty: Vec<Value> = Vec::new();
            let addons = args
                .get("addons")
                .and_then(Value::as_array)
                .unwrap_or(&empty);
            Ok(anime_detection::detect_anime_playback(
                args.get("meta").unwrap_or(&Value::Null),
                args.get("episode").unwrap_or(&Value::Null),
                args.get("stream").unwrap_or(&Value::Null),
                addons,
            ))
        }
        // args_json IS the meta object
        "shouldAttemptAnimeTracking" => Ok(json!(anime_detection::should_attempt_anime_tracking(
            &object(args_json)?
        ))),

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_nuvio_sync(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "nuvioBuildLocalProfiles" => opt_json(nuvio_sync::build_local_profiles_json(args_json)),
        "nuvioLibraryToWatchlist" => opt_json(nuvio_sync::library_to_watchlist_json(args_json)),
        "nuvioProgressMetaNeeds" => opt_json(nuvio_sync::progress_meta_needs_json(args_json)),
        "nuvioProgressSyncRequestPlan" => {
            opt_json(nuvio_sync::progress_sync_request_plan_json(args_json))
        }
        "nuvioApplyProgressSync" => opt_json(nuvio_sync::apply_progress_sync_json(args_json)),
        "nuvioResolveContinueWatching" => {
            opt_json(nuvio_sync::resolve_continue_watching_json(args_json))
        }
        "nuvioDeltaSyncRequestPlan" => {
            opt_json(nuvio_sync::delta_sync_request_plan_json(args_json))
        }
        "nuvioApplyDeltaSync" => opt_json(nuvio_sync::apply_delta_sync_json(args_json)),
        "nuvioImportMergePlan" => opt_json(nuvio_sync::import_merge_plan_json(args_json)),
        "nuvioExportPushPlan" => opt_json(nuvio_sync::export_push_plan_json(args_json)),
        "nuvioLibraryMutationPlan" => opt_json(nuvio_sync::library_mutation_plan_json(args_json)),
        "nuvioMapCollections" => opt_json(nuvio_sync::map_collections_json(args_json)),
        "nuvioSortAddonsByPriority" => {
            opt_json(nuvio_sync::sort_addons_by_priority_json(args_json))
        }
        "nuvioAddonState" => opt_json(nuvio_sync::addon_state_json(args_json)),
        "nuvioAddonReconciliationPlan" => {
            opt_json(nuvio_sync::addon_reconciliation_plan_json(args_json))
        }
        "nuvioLibraryItemRequest" => opt_json(nuvio_sync::library_item_request_json(args_json)),
        "nuvioWatchedItemsRequest" => opt_json(nuvio_sync::watched_items_request_json(args_json)),
        "nuvioPlaybackProgressRequest" => {
            opt_json(nuvio_sync::playback_progress_request_json(args_json))
        }
        "nuvioCollectionRequest" => opt_json(nuvio_sync::collection_request_json(args_json)),

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_nuvio_pin(method: &str, args_json: &str) -> Outcome {
    match method {
        "nuvioPinHash" => opt_str(nuvio_pin::pin_hash_json(args_json)),
        "nuvioPinCachePayload" => opt_json(nuvio_pin::cache_payload_json(args_json)),
        "nuvioPinVerifyCached" => opt_json(nuvio_pin::verify_cached_json(args_json)),
        _ => Err(unknown_method()),
    }
}
