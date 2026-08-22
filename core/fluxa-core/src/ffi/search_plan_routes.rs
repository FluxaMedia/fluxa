use super::*;

pub(super) fn route_search_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for single-arg methods
        "searchResultGrouping" => opt_json(search_plan::search_result_grouping_json(args_json)),
        "searchSuggestionsPlan" => opt_json(search_plan::search_suggestions_plan_json(args_json)),
        "searchScreenPlan" => opt_json(search_plan::search_screen_plan_json(args_json)),
        "mergeDiscoverPages" => opt_json(search_plan::merge_discover_pages_json(args_json)),
        "recentSearchesPlan" => opt_json(search_plan::recent_searches_plan_json(args_json)),
        // args_json IS the sources array
        "mergeSearchSources" => opt_json(search_plan::merge_search_sources_json(args_json)),
        "buildMetadataFeedOptions" => {
            opt_json(search_plan::build_metadata_feed_options_json(args_json))
        }
        "discoverCatalogOptions" => {
            let args = object(args_json)?;
            opt_json(search_plan::discover_catalog_options_json(
                field_str(&args, "addons")?,
                field_str(&args, "selectedType")?,
            ))
        }
        "discoverContentTypes" => opt_json(search_plan::discover_content_types_json(args_json)),
        "discoverSelectionPlan" => opt_json(search_plan::discover_selection_plan_json(args_json)),
        "librarySortPlan" => opt_json(search_plan::library_sort_plan_json(args_json)),
        "discoverSortPlan" => opt_json(search_plan::discover_sort_plan_json(args_json)),
        "detailSeriesLookupId" => Ok(Value::String(search_plan::detail_series_lookup_id(
            &arg_str(args_json, "id")?,
        ))),
        "detailSeasonLoadPlan" => opt_json(search_plan::detail_season_load_plan_json(args_json)),
        "resolveTransportUrl" => {
            let args = object(args_json)?;
            opt_json(search_plan::resolve_transport_url_json(
                field_str(&args, "sourceJson")?,
                field_str(&args, "addonsJson")?,
            ))
        }
        "resolveFeedOptionGenre" => {
            let args = object(args_json)?;
            opt_json(search_plan::resolve_feed_option_genre_json(
                field_str(&args, "feedOptionJson")?,
                field_str(&args, "addonsJson")?,
            ))
        }

        _ => Err(unknown_method()),
    }
}
