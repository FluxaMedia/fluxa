use super::*;

pub(super) fn route_search_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for single-arg methods
        "searchResultGrouping" => opt_json(catalog::search::search_result_grouping_json(args_json)),
        "searchSuggestionsPlan" => {
            opt_json(catalog::search::search_suggestions_plan_json(args_json))
        }
        "searchScreenPlan" => opt_json(catalog::search::search_screen_plan_json(args_json)),
        "mergeDiscoverPages" => opt_json(catalog::search::merge_discover_pages_json(args_json)),
        "mergeDiscoverSources" => opt_json(catalog::search::merge_discover_sources_json(args_json)),
        "discoverSourceRequests" => {
            opt_json(catalog::search::discover_source_requests_json(args_json))
        }
        "recentSearchesPlan" => opt_json(catalog::search::recent_searches_plan_json(args_json)),
        // args_json IS the sources array
        "mergeSearchSources" => opt_json(catalog::search::merge_search_sources_json(args_json)),
        "buildMetadataFeedOptions" => {
            opt_json(catalog::search::build_metadata_feed_options_json(args_json))
        }
        "discoverCatalogOptions" => {
            let args = object(args_json)?;
            opt_json(catalog::search::discover_catalog_options_json(
                field_str(&args, "addons")?,
                field_str(&args, "selectedType")?,
            ))
        }
        "discoverContentTypes" => opt_json(catalog::search::discover_content_types_json(args_json)),
        "discoverCatalogRequestTypes" => {
            opt_json(catalog::search::discover_catalog_request_types_json(
                &arg_str(args_json, "catalogType")?,
            ))
        }
        "discoverSelectionPlan" => {
            opt_json(catalog::search::discover_selection_plan_json(args_json))
        }
        "discoverCatalogCandidates" => {
            opt_json(catalog::search::discover_catalog_candidates_json(args_json))
        }
        "librarySortPlan" => opt_json(catalog::search::library_sort_plan_json(args_json)),
        "discoverSortPlan" => opt_json(catalog::search::discover_sort_plan_json(args_json)),
        "detailSeriesLookupId" => Ok(Value::String(catalog::search::detail_series_lookup_id(
            &arg_str(args_json, "id")?,
        ))),
        "detailSeasonLoadPlan" => {
            opt_json(catalog::search::detail_season_load_plan_json(args_json))
        }
        "detailAvailableSeasons" => {
            opt_json(catalog::search::detail_available_seasons_json(args_json))
        }
        "detailLoadPlan" => opt_json(catalog::search::detail_load_plan_json(args_json)),
        "detailSeasonVideos" => opt_json(catalog::search::detail_season_videos_json(args_json)),
        "resolveTransportUrl" => {
            let args = object(args_json)?;
            opt_json(catalog::search::resolve_transport_url_json(
                field_str(&args, "sourceJson")?,
                field_str(&args, "addonsJson")?,
            ))
        }
        "resolveFeedOptionGenre" => {
            let args = object(args_json)?;
            opt_json(catalog::search::resolve_feed_option_genre_json(
                field_str(&args, "feedOptionJson")?,
                field_str(&args, "addonsJson")?,
            ))
        }

        _ => Err(unknown_method()),
    }
}
