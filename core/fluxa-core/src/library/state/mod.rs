mod artwork_diff;
mod helpers;
mod library_lists;
mod playback_progress;

pub(crate) use crate::library::continue_watching::{
    UP_NEXT_DURATION_SECONDS, UP_NEXT_POSITION_SECONDS, build_continue_watching_from_progress,
    build_continue_watching_from_progress_json, continue_watching_source_plan_json,
    format_episode_line_json,
    normalized_continue_watching_source, remember_last_watched_episodes_json, resolve_next_episode_json,
};
pub(crate) use library_lists::{
    normalize_library_document,
    normalize_library_document_json, normalize_library_read_result_json,
};

#[cfg(test)]
mod tests;
