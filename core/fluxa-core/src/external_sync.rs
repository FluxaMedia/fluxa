mod calendar;
mod merge;
mod plan;
mod simkl;
mod stremio;
mod trakt;

pub(crate) use calendar::provider_calendar_items_json;
pub(crate) use merge::{
    merge_continue_watching_lists_json, merge_external_watched_json, merge_external_watchlist_json,
    merge_watched_timestamped_json, merge_watchlist_timestamped_json, ranked_winner, saved_at_ms,
};
pub(crate) use plan::{
    external_provider_action_plan_json, external_sync_refresh_retry_action,
    external_sync_response_action, import_apply_plan_json, promote_external_progress_plan_json,
    provider_pagination_plan_json, push_plan_json,
};
pub(crate) use simkl::{
    simkl_history_request_json, simkl_playback_delete_ids_json,
    simkl_playback_item_to_continue_meta_json, simkl_watchlist_request_json,
};
pub(crate) use stremio::{
    stremio_library_mutation_plan_json, stremio_watched_to_ids_json,
    stremio_watchlist_to_items_json,
};
pub(crate) use trakt::{
    trakt_artwork, trakt_bearer, trakt_comments_request_json, trakt_content_id_from_ids_json,
    trakt_episode_locator_json, trakt_has_client, trakt_history_request_json, trakt_id_from_source,
    trakt_ids_from_content_id_json, trakt_image_url, trakt_oauth_error_code,
    trakt_playback_delete_ids_json, trakt_playback_items_to_library_json, trakt_playback_url,
    trakt_scrobble_media_id, trakt_scrobble_url, trakt_show_id_from_episode_id,
    trakt_sync_item_to_meta_json, trakt_token_expires_at, trakt_watched_to_ids_json,
    trakt_watchlist_to_items_json,
};

mod provider_mappers;

pub(crate) use provider_mappers::{
    replace_external_continue_watching_json, simkl_lookup_id_for_type,
    simkl_mark_watched_body_json, simkl_match_episode_json, simkl_merge_delta_json,
    simkl_merge_playback_progress_json, simkl_recommendation_candidates_json,
    simkl_recommendation_to_meta_json, simkl_resource_sync_plan_json, simkl_watched_to_ids_json,
    simkl_watching_to_items_json, simkl_watchlist_body_json, simkl_watchlist_to_items_json,
    trakt_activity_diff_json, trakt_mark_watched_body_json, trakt_playback_items_dedup_json,
    trakt_related_items_to_metas_json, trakt_related_lookup_slug, trakt_up_next_to_items_json,
    trakt_watched_shows_to_items_json,
};
mod anilist;

pub(crate) use anilist::{
    anilist_entries_to_sync, anilist_graphql_queries_json, anilist_media_list_status,
    anilist_save_media_list_entry_variables_json, anilist_search_best_match_json,
    extract_anilist_id_from_links, merge_library_items_by_id,
};

#[cfg(test)]
mod tests;
