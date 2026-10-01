use crate::ffi::*;

pub(crate) fn route_search_plan(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for single-arg methods
        "searchResultGrouping" => opt_json(crate::catalog::search::search_result_grouping_json(
            args_json,
        )),
        "mergeDiscoverSources" => opt_json(crate::catalog::search::merge_discover_sources_json(
            args_json,
        )),
        "discoverSourceRequests" => opt_json(
            crate::catalog::search::discover_source_requests_json(args_json),
        ),
        // args_json IS the sources array
        "mergeSearchSources" => {
            opt_json(crate::catalog::search::merge_search_sources_json(args_json))
        }
        "buildMetadataFeedOptions" => opt_json(
            crate::catalog::search::build_metadata_feed_options_json(args_json),
        ),
        "discoverCatalogOptions" => {
            let args = object(args_json)?;
            opt_json(crate::catalog::search::discover_catalog_options_json(
                field_str(&args, "addons")?,
                field_str(&args, "selectedType")?,
            ))
        }
        _ => Err(unknown_method()),
    }
}
