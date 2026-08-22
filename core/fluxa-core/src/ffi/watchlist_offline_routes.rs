use super::*;

pub(super) fn route_watchlist(method: &str, args_json: &str) -> Outcome {
    match method {
        "remoteCollectionRequestPlan" => opt_json(
            watchlist_plan::remote_collection_request_plan_json(args_json),
        ),
        "remoteCollectionResponsePlan" => opt_json(
            watchlist_plan::remote_collection_response_plan_json(args_json),
        ),
        // args_json IS the request object
        "watchlistTogglePlan" => opt_json(watchlist_plan::watchlist_toggle_plan_json(args_json)),
        "libraryCommandPlan" => opt_json(watchlist_plan::library_command_plan_json(args_json)),
        "playbackProgressMergePlan" => {
            opt_json(watchlist_plan::playback_progress_merge_plan_json(args_json))
        }
        "playbackProgressWritePlan" => {
            opt_json(watchlist_plan::playback_progress_write_plan_json(args_json))
        }
        "libraryApplyMarkWatched" => {
            let args = object(args_json)?;
            opt_json(watchlist_plan::library_apply_mark_watched_json(
                field_str(&args, "libJson")?,
                field_str(&args, "videoIdsJson")?,
            ))
        }
        "mergeProgressMeta" => {
            let args = object(args_json)?;
            into_json(watchlist_plan::merge_progress_meta_json(
                field_str(&args, "incomingMetaJson")?,
                field_str(&args, "existingMetaJson")?,
            ))
        }
        "airDateRefreshCandidates" => {
            opt_json(watchlist_plan::air_date_refresh_candidates_json(args_json))
        }
        "airDateRefreshPlan" => opt_json(watchlist_plan::air_date_refresh_plan_json(args_json)),
        "applyAirDateUpdates" => opt_json(watchlist_plan::apply_air_date_updates_json(args_json)),
        "libraryViewPlan" => opt_json(watchlist_plan::library_view_plan_json(args_json)),
        "collectionMergePlan" => opt_json(watchlist_plan::collection_merge_plan_json(args_json)),
        "collectionFolderItemsPlan" => {
            opt_json(watchlist_plan::collection_folder_items_plan_json(args_json))
        }
        "collectionFolderTabsPlan" => {
            opt_json(watchlist_plan::collection_folder_tabs_plan_json(args_json))
        }
        "importCollections" => opt_json(watchlist_plan::import_collections_json(args_json)),
        "exportCollections" => opt_json(watchlist_plan::export_collections_json(args_json)),
        "libraryExternalMergePlan" => {
            opt_json(watchlist_plan::library_external_merge_plan_json(args_json))
        }
        "libraryCollectionImportValidation" => opt_json(
            watchlist_plan::library_collection_import_validation_json(args_json),
        ),
        "libraryOfflineGrouping" => {
            opt_json(watchlist_plan::library_offline_grouping_json(args_json))
        }

        _ => Err(unknown_method()),
    }
}

pub(super) fn route_offline(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        "offlineDownloadPlan" => opt_json(offline_download::offline_download_plan_json(args_json)),

        _ => Err(unknown_method()),
    }
}
