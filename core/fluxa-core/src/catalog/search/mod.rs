mod addon_catalog;
mod detail_nav;
mod discover;
mod library_sort;
mod search;

pub(crate) use addon_catalog::{
    build_metadata_feed_options_json, discover_catalog_options_json, discover_content_types_json,
    resolve_feed_option_genre_json, resolve_transport_url_json,
};
pub(crate) use discover::discover_source_requests_json;
pub(crate) use discover::{
    merge_discover_sources, merge_discover_sources_json,
};
pub(crate) use search::{
    merge_search_sources, merge_search_sources_json,
    search_result_grouping_json,
};

#[cfg(test)]
mod tests;
