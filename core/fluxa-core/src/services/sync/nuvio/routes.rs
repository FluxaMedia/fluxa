use crate::ffi::*;
use crate::services;
use serde_json::{Value, json};

pub(crate) fn route_nuvio(method: &str, args_json: &str) -> Outcome {
    match method {
        "nuvioApplyRemoteProfiles" => {
            opt_json(services::nuvio::apply_remote_profiles_json(args_json))
        }
        "nuvioBuildLocalProfiles" => {
            opt_json(services::nuvio::build_local_profiles_json(args_json))
        }
        "nuvioEffectiveProfileScopes" => {
            opt_json(services::nuvio::effective_profile_scopes_json(args_json))
        }
        "nuvioLibraryToWatchlist" => {
            opt_json(services::nuvio::library_to_watchlist_json(args_json))
        }
        "nuvioProviderLibrarySnapshot" => {
            opt_json(services::nuvio::provider_library_snapshot_json(args_json))
        }
        "nuvioProgressMetaNeeds" => opt_json(services::nuvio::progress_meta_needs_json(args_json)),
        "nuvioProgressPresentation" => {
            opt_json(services::nuvio::progress_presentation_json(args_json))
        }
        "nuvioProgressSyncRequestPlan" => {
            opt_json(services::nuvio::progress_sync_request_plan_json(args_json))
        }
        "nuvioApplyProgressSync" => opt_json(services::nuvio::apply_progress_sync_json(args_json)),
        "nuvioResolveContinueWatching" => {
            opt_json(services::nuvio::resolve_continue_watching_json(args_json))
        }
        "nuvioDeltaSyncRequestPlan" => {
            opt_json(services::nuvio::delta_sync_request_plan_json(args_json))
        }
        "nuvioApplyDeltaSync" => opt_json(services::nuvio::apply_delta_sync_json(args_json)),
        "nuvioImportMergePlan" => opt_json(services::nuvio::import_merge_plan_json(args_json)),
        "nuvioExportPushPlan" => opt_json(services::nuvio::export_push_plan_json(args_json)),
        "nuvioLibraryMutationPlan" => {
            opt_json(services::nuvio::library_mutation_plan_json(args_json))
        }
        "nuvioWriteRequests" => opt_json(services::nuvio::write_requests_json(args_json)),
        "nuvioHomeLayout" => opt_json(services::nuvio::home_layout_json(args_json)),
        "nuvioMapCollections" => opt_json(services::nuvio::map_collections_json(args_json)),
        "nuvioSortAddonsByPriority" => {
            opt_json(services::nuvio::sort_addons_by_priority_json(args_json))
        }
        "nuvioAddonSnapshotPlan" => opt_json(services::nuvio::addon_snapshot_plan_json(args_json)),
        "nuvioAddonState" => opt_json(services::nuvio::addon_state_json(args_json)),
        "nuvioAddonReconciliationPlan" => {
            opt_json(services::nuvio::addon_reconciliation_plan_json(args_json))
        }
        "nuvioLibraryItemRequest" => {
            opt_json(services::nuvio::library_item_request_json(args_json))
        }
        "nuvioWatchedItemsRequest" => {
            opt_json(services::nuvio::watched_items_request_json(args_json))
        }
        "nuvioPlaybackProgressRequest" => {
            opt_json(services::nuvio::playback_progress_request_json(args_json))
        }
        "nuvioCollectionRequest" => opt_json(services::nuvio::collection_request_json(args_json)),
        "nuvioCanonicalContentType" => Ok(Value::String(
            services::nuvio::canonical_content_type(&arg_str(args_json, "value")?).to_string(),
        )),
        "nuvioPluginContentId" => {
            let args = object(args_json)?;
            let season = args.get("season").and_then(Value::as_i64);
            let episode = args.get("episode").and_then(Value::as_i64);
            Ok(Value::String(services::nuvio::plugin_content_id(
                &arg_str(args_json, "videoId")?,
                season,
                episode,
            )))
        }
        "nuvioPluginContentType" => Ok(Value::String(services::nuvio::plugin_content_type(
            &arg_str(args_json, "value")?,
        ))),
        "nuvioCandidateContentTypes" => Ok(json!(services::nuvio::candidate_content_types(
            &arg_str(args_json, "value")?
        ))),
        "nuvioPinHash" => opt_str(services::nuvio::pin::pin_hash_json(args_json)),
        "nuvioPinCachePayload" => opt_json(services::nuvio::pin::cache_payload_json(args_json)),
        "nuvioPinVerifyCached" => opt_json(services::nuvio::pin::verify_cached_json(args_json)),
        _ => Err(unknown_method()),
    }
}
