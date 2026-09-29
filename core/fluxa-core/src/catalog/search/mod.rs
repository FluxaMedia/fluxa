mod addon_catalog;
mod detail_nav;
mod discover;
mod library_sort;
mod search;

pub(crate) use addon_catalog::{
    build_metadata_feed_options_json, discover_catalog_options_json,
    discover_catalog_request_types_json, discover_content_types_json,
    resolve_feed_option_genre_json, resolve_transport_url_json,
};
pub(crate) use detail_nav::{
    detail_available_seasons_json, detail_load_plan_json, detail_season_load_plan_json,
    detail_season_videos_json, detail_series_lookup_id,
};
pub(crate) use discover::{discover_catalog_candidates_json, discover_source_requests_json};
pub(crate) use discover::{
    discover_selection_plan_json, discover_sort_plan_json, merge_discover_pages_json,
    merge_discover_sources, merge_discover_sources_json,
};
pub(crate) use library_sort::library_sort_plan_json;
pub(crate) use search::{
    merge_search_sources, merge_search_sources_json, recent_searches_plan_json,
    search_result_grouping_json, search_screen_plan_json, search_suggestions_plan_json,
};

#[cfg(test)]
mod tests;
