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
pub(crate) use discussion::{
    mdblist_discussion_comment_delete_plan, mdblist_discussion_comment_like_plan,
    mdblist_discussion_comment_update_plan, mdblist_discussion_create_plan,
    mdblist_discussion_hot_url, mdblist_discussion_replies_url,
    mdblist_discussion_reply_create_plan, mdblist_discussion_reply_delete_plan,
    mdblist_discussion_reply_like_plan, mdblist_discussion_reply_update_plan,
    mdblist_discussion_summary_url, mdblist_discussion_url,
};
pub(crate) use helpers::{mdblist_bearer, mdblist_device_poll_outcome};
pub(crate) use lists::{
    mdblist_list_by_name_url, mdblist_list_changes_url, mdblist_list_create_plan,
    mdblist_list_delete_plan, mdblist_list_info_url, mdblist_list_items_mutate_plan,
    mdblist_list_items_response_to_metas_json, mdblist_list_items_url, mdblist_list_like_plan,
    mdblist_list_membership_url, mdblist_list_update_plan, mdblist_lists_curated_url,
    mdblist_lists_liked_url, mdblist_lists_official_url, mdblist_lists_recommended_url,
    mdblist_lists_search_url, mdblist_lists_top_url, mdblist_lists_user_url,
};
pub(crate) use media_ratings::{
    mdblist_catalog_url, mdblist_content_type, mdblist_genres_url, mdblist_media_info_batch_plan,
    mdblist_media_info_url, mdblist_media_ratings_from_response_json, mdblist_ratings_batch_plan,
    mdblist_search_url, mdblist_watchprovider_links_url,
};
pub(crate) use scrobble::{mdblist_checkin_plan, mdblist_scrobble_plan};
pub(crate) use user::{
    mdblist_public_user_url, mdblist_user_follow_plan, mdblist_user_stats_url, mdblist_user_url,
};
pub(crate) use watchlist_sync::{
    mdblist_sync_get_url, mdblist_sync_mutate_plan, mdblist_upnext_url, mdblist_watched_body_plan,
    mdblist_watchlist_items_url, mdblist_watchlist_mutate_plan,
};

#[cfg(test)]
mod tests;
