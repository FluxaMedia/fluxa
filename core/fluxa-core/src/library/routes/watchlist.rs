use crate::ffi::*;

pub(crate) fn route_watchlist(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "libraryCommandPlan" => opt_json(crate::library::watchlist::library_command_plan_json(
            args_json,
        )),
        "playbackProgressWritePlan" => {
            opt_json(crate::library::watchlist::playback_progress_write_plan_json(args_json))
        }
        "remoteCollectionRequestPlan" => opt_json(
            crate::library::watchlist::remote_collection_request_plan_json(args_json),
        ),
        "remoteCollectionResponsePlan" => opt_json(
            crate::library::watchlist::remote_collection_response_plan_json(args_json),
        ),
        "collectionMutationPlan" => opt_json(
            crate::library::watchlist::collection_mutation_plan_json(args_json),
        ),
        "importCollections" => opt_json(crate::library::watchlist::import_collections_json(args_json)),
        "exportCollections" => opt_json(crate::library::watchlist::export_collections_json(args_json)),
        "libraryViewPlan" => opt_json(crate::library::watchlist::library_view_plan_json(args_json)),
        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_offline(method: &str, _args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        _ => Err(unknown_method()),
    }
}
