mod cache_keys;
mod discover_filters;
mod episode_matching;
mod feed_selection;
mod helpers;
mod id;
mod merge_keys;
mod playback_plan;
mod text;

pub(crate) use cache_keys::parse_extra_args_json;
// unused outside the `fuzzing`-feature build: fuzz_targets (lib.rs) is the only
// consumer of this path, and default builds don't enable that feature.
pub(crate) use episode_matching::stream_matches_episode;
#[allow(unused_imports)]
pub use episode_matching::{contains_compact_episode, contains_spaced_episode};
pub(crate) use feed_selection::{
    cs3_metadata_feed_options_json, effective_metadata_feed_selection_json, ordered_metadata_feed_keys,
};
pub(crate) use helpers::imdb_regex;
pub use id::parse_episode_locator;
pub(crate) use id::{
    base_content_id, imdb_id, parse_video_id_json,
};
pub(crate) use playback_plan::{
    split_channel_schedule, stream_request_ids,
};
// unused outside the `fuzzing`-feature build: fuzz_targets (lib.rs) is the only
// consumer of this path, and default builds don't enable that feature.
#[allow(unused_imports)]
pub use text::percent_decode_component;
pub(crate) use text::{
    cs3_catalog_feed_key, cs3_plugin_feed_key, normalize_content_type, shorten_synopsis,
    stable_feed_part,
};

#[cfg(test)]
mod tests;
