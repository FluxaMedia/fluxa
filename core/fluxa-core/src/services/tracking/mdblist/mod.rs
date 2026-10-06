mod routes;
pub(crate) use routes::*;
mod api;
mod discussion;
mod helpers;
mod lists;
mod media_ratings;
mod scrobble;
mod user;
mod watchlist_sync;

pub(crate) use api::*;
pub(crate) use lists::{mdblist_list_items_response_to_metas_json, mdblist_list_items_url};
pub(crate) use media_ratings::{
    mdblist_content_type, mdblist_media_info_batch_plan, mdblist_media_info_url,
    mdblist_media_ratings_from_response_json,
};
pub(crate) use scrobble::mdblist_scrobble_plan;
pub(crate) use watchlist_sync::{
    mdblist_sync_get_url, mdblist_sync_mutate_plan, mdblist_upnext_url, mdblist_watched_body_plan,
    mdblist_watchlist_items_url, mdblist_watchlist_mutate_plan,
};

#[cfg(test)]
mod tests;
