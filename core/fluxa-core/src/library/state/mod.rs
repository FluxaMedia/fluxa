mod artwork_diff;
mod helpers;
mod library_lists;
mod playback_progress;

pub(crate) use crate::library::continue_watching::{
    UP_NEXT_DURATION_SECONDS, UP_NEXT_POSITION_SECONDS, build_continue_watching_from_progress,
    build_continue_watching_from_progress_json, compute_continue_watching_badges_json,
    continue_watching_resume_plan_json, continue_watching_source_plan_json,
    format_episode_line_json, is_episode_released, next_progress_info_plan_json,
    normalized_continue_watching_source, remember_last_watched_episodes_json,
    resolve_next_after_watched_json, resolve_next_episode_json,
};
pub(crate) use artwork_diff::{
    continue_watching_card_fields_json, continue_watching_progress_fields_json,
    item_list_diff_json, item_list_new_entries_json, select_continue_watching_artwork_json,
    value_map_diff_json, watched_map_diff_json,
};
pub(crate) use library_lists::{
    filter_home_continue_watching_json, is_up_next_continue_watching_item_json,
    library_continue_watching_items_json, library_watchlist_items_json, normalize_library_document,
    normalize_library_document_json, normalize_library_read_result_json, watched_video_ids_json,
};
pub(crate) use playback_progress::{
    clear_playback_progress_item_json, clear_playback_progress_plan_json,
    playback_progress_item_json, watched_state_items_json,
};

#[cfg(test)]
mod tests;
