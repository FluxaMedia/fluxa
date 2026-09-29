use super::*;

pub(super) fn route_watchlist(method: &str, args_json: &str) -> Outcome {
    match method {
        "remoteCollectionRequestPlan" => opt_json(
            library::watchlist::remote_collection_request_plan_json(args_json),
        ),
        "remoteCollectionResponsePlan" => opt_json(
            library::watchlist::remote_collection_response_plan_json(args_json),
        ),
        // args_json IS the request object
        "watchlistTogglePlan" => {
            opt_json(library::watchlist::watchlist_toggle_plan_json(args_json))
        }
        "libraryCommandPlan" => opt_json(library::watchlist::library_command_plan_json(args_json)),
        "playbackProgressMergePlan" => opt_json(
            library::watchlist::playback_progress_merge_plan_json(args_json),
        ),
        "playbackProgressWritePlan" => opt_json(
            library::watchlist::playback_progress_write_plan_json(args_json),
        ),
        "libraryApplyMarkWatched" => {
            let args = object(args_json)?;
            opt_json(library::watchlist::library_apply_mark_watched_json(
                field_str(&args, "libJson")?,
                field_str(&args, "videoIdsJson")?,
            ))
        }
        "mergeProgressMeta" => {
            let args = object(args_json)?;
            into_json(library::watchlist::merge_progress_meta_json(
                field_str(&args, "incomingMetaJson")?,
                field_str(&args, "existingMetaJson")?,
            ))
        }
        "airDateRefreshCandidates" => opt_json(
            library::watchlist::air_date_refresh_candidates_json(args_json),
        ),
        "airDateRefreshPlan" => opt_json(library::watchlist::air_date_refresh_plan_json(args_json)),
        "applyAirDateUpdates" => {
            opt_json(library::watchlist::apply_air_date_updates_json(args_json))
        }
        "libraryViewPlan" => opt_json(library::watchlist::library_view_plan_json(args_json)),
        "collectionMergePlan" => {
            opt_json(library::watchlist::collection_merge_plan_json(args_json))
        }
        "collectionFolderItemsPlan" => opt_json(
            library::watchlist::collection_folder_items_plan_json(args_json),
        ),
        "collectionFolderTabsPlan" => opt_json(
            library::watchlist::collection_folder_tabs_plan_json(args_json),
        ),
        "collectionMutationPlan" => {
            opt_json(library::watchlist::collection_mutation_plan_json(args_json))
        }
        "importCollections" => opt_json(library::watchlist::import_collections_json(args_json)),
        "exportCollections" => opt_json(library::watchlist::export_collections_json(args_json)),
        "collectionFolderPresentation" => opt_json(
            library::watchlist::collection_folder_presentation_json(args_json),
        ),
        "libraryExternalMergePlan" => opt_json(
            library::watchlist::library_external_merge_plan_json(args_json),
        ),
        "libraryCollectionImportValidation" => {
            opt_json(library::watchlist::library_collection_import_validation_json(args_json))
        }
        "libraryOfflineGrouping" => {
            opt_json(library::watchlist::library_offline_grouping_json(args_json))
        }

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_offline(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "offlineDownloadPlan" => opt_json(library::offline_download::offline_download_plan_json(
            args_json,
        )),

        _ => Err(unknown_method()),
    }
}
