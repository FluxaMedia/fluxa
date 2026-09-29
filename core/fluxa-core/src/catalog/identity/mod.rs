mod cache_keys;
mod discover_filters;
mod episode_matching;
mod feed_selection;
mod helpers;
mod id;
mod merge_keys;
mod playback_plan;
mod text;

pub(crate) use cache_keys::{
    discover_catalog_cache_key, episode_filename_candidate, parse_extra_args_json,
    stream_discovery_cache_key,
};
pub(crate) use discover_filters::filter_discover_results_json;
// unused outside the `fuzzing`-feature build: fuzz_targets (lib.rs) is the only
// consumer of this path, and default builds don't enable that feature.
pub(crate) use episode_matching::stream_matches_episode;
pub(crate) use episode_matching::text_matches_episode;
#[allow(unused_imports)]
pub use episode_matching::{contains_compact_episode, contains_spaced_episode};
pub(crate) use feed_selection::{
    cs3_metadata_feed_options_json, effective_metadata_feed_selection_json,
    move_metadata_feed_order_json, ordered_metadata_feed_keys,
    set_metadata_feed_group_enabled_json, toggle_metadata_feed_json,
    toggle_metadata_feed_limited_json,
};
pub(crate) use helpers::imdb_regex;
pub use id::parse_episode_locator;
pub(crate) use id::{
    base_content_id, build_trakt_ids_json, imdb_id, is_tmdb_like_content_id,
    normalize_series_lookup_id, parse_video_id_json, tmdb_numeric_id,
};
pub(crate) use merge_keys::{
    content_keys_json, content_trakt_keys_batch, content_watched_keys_batch,
    content_watched_keys_value, normalized_billboard_title,
};
pub(crate) use playback_plan::{
    direct_playback_plan_json, playback_intro_lookup_content_id, playback_stream_request_ids_json,
    split_channel_schedule, stream_discovery_episode_context_json, stream_request_ids,
};
// unused outside the `fuzzing`-feature build: fuzz_targets (lib.rs) is the only
// consumer of this path, and default builds don't enable that feature.
#[allow(unused_imports)]
pub use text::percent_decode_component;
pub(crate) use text::{
    content_matches_filter, cs3_catalog_feed_key, cs3_plugin_feed_key, is_series_content_type,
    normalize_catalog_type, normalize_content_type, provider_search_terms, shorten_synopsis,
    stable_feed_part,
};

#[cfg(test)]
mod tests;
