use crate::ffi::*;
use crate::services;

pub(crate) fn route_nuvio(method: &str, args_json: &str) -> Outcome {
    match method {
        "nuvioApplyRemoteProfiles" => {
            opt_json(services::nuvio::apply_remote_profiles_json(args_json))
        }
        "nuvioEffectiveProfileScopes" => {
            opt_json(services::nuvio::effective_profile_scopes_json(args_json))
        }
        "nuvioProviderLibrarySnapshot" => {
            opt_json(services::nuvio::provider_library_snapshot_json(args_json))
        }
        "nuvioProgressMetaNeeds" => opt_json(services::nuvio::progress_meta_needs_json(args_json)),
        "nuvioApplyProgressSync" => opt_json(services::nuvio::apply_progress_sync_json(args_json)),
        "nuvioDeltaSyncRequestPlan" => {
            opt_json(services::nuvio::delta_sync_request_plan_json(args_json))
        }
        "nuvioApplyDeltaSync" => opt_json(services::nuvio::apply_delta_sync_json(args_json)),
        "nuvioWriteRequests" => opt_json(services::nuvio::write_requests_json(args_json)),
        "nuvioHomeLayout" => opt_json(services::nuvio::home_layout_json(args_json)),
        "nuvioMapCollections" => opt_json(services::nuvio::map_collections_json(args_json)),
        "nuvioAddonSnapshotPlan" => opt_json(services::nuvio::addon_snapshot_plan_json(args_json)),
        _ => Err(unknown_method()),
    }
}
