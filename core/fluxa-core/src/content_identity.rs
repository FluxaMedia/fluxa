mod cache_keys;
mod discover_filters;
mod episode_matching;
mod feed_selection;
mod helpers;
mod id;
mod merge_keys;
mod playback_plan;
mod text;

#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use cache_keys::{
    discover_catalog_cache_key, episode_filename_candidate, parse_extra_args_json,
    stream_discovery_cache_key,
};
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use discover_filters::filter_discover_results_json;
// unused outside the `fuzzing`-feature build: fuzz_targets (lib.rs) is the only
// consumer of this path, and default builds don't enable that feature.
#[allow(unused_imports)]
pub use episode_matching::{contains_compact_episode, contains_spaced_episode};
pub(crate) use episode_matching::stream_matches_episode;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use episode_matching::text_matches_episode;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use feed_selection::{
    effective_metadata_feed_selection_json, move_metadata_feed_order_json,
    ordered_metadata_feed_keys, set_metadata_feed_group_enabled_json, toggle_metadata_feed_json,
    toggle_metadata_feed_limited_json,
};
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use helpers::imdb_regex;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub use id::parse_episode_locator;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use id::{
    base_content_id, build_trakt_ids_json, imdb_id, is_tmdb_like_content_id,
    normalize_series_lookup_id, parse_video_id_json, tmdb_numeric_id,
};
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use merge_keys::{
    content_keys_json, content_trakt_keys_batch, content_watched_keys_batch,
    content_watched_keys_value, normalized_billboard_title,
};
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use playback_plan::{
    direct_playback_plan_json, playback_intro_lookup_content_id, playback_stream_request_ids_json,
    stream_discovery_episode_context_json, stream_request_ids,
};
// unused outside the `fuzzing`-feature build: fuzz_targets (lib.rs) is the only
// consumer of this path, and default builds don't enable that feature.
#[allow(unused_imports)]
pub use text::percent_decode_component;
#[cfg(any(feature = "full-api", not(feature = "streaming-shared")))]
pub(crate) use text::{
    normalize_content_type, provider_search_terms, shorten_synopsis, stable_feed_part,
};
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn shorten_synopsis_joins_paired_em_dash_aside_with_commas() {
        let text = "Fighting crime full-time as Spider-Man in a world that doesn't remember him\u{2014}and the pressure of seeing his old friends move on without him\u{2014}sparks a change in Peter Parker he may not have the power to control. But that transformation might also be the only thing that can stop a shocking new threat to the city and those he loves - a powerful villain no one can even see.";
        let result = shorten_synopsis(text);
        assert_eq!(
            result,
            "Fighting crime full-time as Spider-Man in a world that doesn't remember him, and the pressure of seeing his old friends move on without him, sparks a change in Peter Parker he may not have the power to control."
        );
    }

    #[test]
    fn shorten_synopsis_keeps_short_single_sentence_untouched() {
        let text = "When Bonnie receives a Lilypad tablet as a gift and becomes obsessed, Buzz, Woody, Jessie and the rest of the gang's jobs become exponentially harder when they have to go head to head with the all-new threat to playtime.";
        assert_eq!(shorten_synopsis(text), text);
    }

    #[test]
    fn shorten_synopsis_cuts_long_text_at_comma_not_mid_word() {
        let text = "A family suddenly sealed inside their home must survive against dwindling resources and a mysterious threat, facing terror at every turn as the walls close in, unable to escape the nightmare that has consumed their once peaceful life, running out of options as each day brings new horrors, and losing hope with each passing hour, day, and night, until finally a stranger arrives.";
        let result = shorten_synopsis(text);
        assert!(result.ends_with('.'));
        assert!(result.len() < text.len());
        assert!(!result.ends_with(" ."));
    }

    #[test]
    fn playback_intro_lookup_prefers_imdb_then_base_tmdb_number() {
        assert_eq!(playback_intro_lookup_content_id("tmdb:42:1:2"), "42");
        assert_eq!(
            playback_intro_lookup_content_id("tt1234567:1:2"),
            "tt1234567"
        );
    }

    #[test]
    fn stream_matching_uses_torrent_filename_episode_suffix() {
        let e1 = "[SubsPlease] Sousou no Frieren - 01v2 (1080p) [AAA94036].mkv".to_string();
        let e2 = "[SubsPlease] Sousou no Frieren - 02v2 (1080p) [00DB7386].mkv".to_string();
        assert!(!stream_matches_episode("tt123:1:2", &[e1]));
        assert!(stream_matches_episode("tt123:1:2", &[e2]));
    }

    #[test]
    fn playback_stream_request_ids_use_detail_imdb_as_canonical_id() {
        let ids = playback_stream_request_ids_json("movie", "tmdb:42", Some("tt1234567"))
            .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
            .expect("ids");

        assert_eq!(ids, vec!["tt1234567", "tmdb:42"]);
    }

    #[test]
    fn direct_playback_plan_selects_first_released_episode_without_mutating_provider_streams() {
        let plan = direct_playback_plan_json(
            r#"{"id":"tt1","name":"Fallback","type":"series","description":"fallback","lastStreamIndex":3}"#,
            Some(
                r#"{"id":"tt1","name":"Detail","type":"series","poster":"p","videos":[{"id":"tt1:1:2","season":1,"number":2,"released":"2026-06-01T00:00:00.000Z"},{"id":"tt1:1:1","season":1,"number":1,"released":"2026-05-01T00:00:00.000Z"}]}"#,
            ),
            "2026-05-21",
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("plan");

        assert_eq!(plan["targetVideoId"], "tt1:1:1");
        assert_eq!(plan["lookupId"], "tt1:1:1");
        assert_eq!(plan["meta"]["name"], "Detail");
        assert_eq!(plan["meta"]["description"], "fallback");
        assert_eq!(plan["meta"]["episodesCount"], 2);
        assert_eq!(plan["meta"]["lastStreamIndex"], 3);
    }

    #[test]
    fn direct_playback_plan_prefers_saved_video_and_falls_back_to_meta_without_detail() {
        let plan = direct_playback_plan_json(
            r#"{"id":"tt1","name":"Movie","type":"movie","lastVideoId":"tt1:2:3"}"#,
            None,
            "2026-05-21",
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("plan");

        assert_eq!(plan["targetVideoId"], "tt1:2:3");
        assert_eq!(plan["lookupId"], "tt1:2:3");
        assert_eq!(plan["meta"]["name"], "Movie");
    }

    #[test]
    fn effective_metadata_feed_selection_preserves_explicit_empty_selection() {
        assert_eq!(
            effective_metadata_feed_selection_json("null", r#"["a","b"]"#),
            None
        );
        assert_eq!(
            effective_metadata_feed_selection_json("[]", r#"["a","b"]"#).as_deref(),
            Some("[]")
        );
        assert_eq!(
            effective_metadata_feed_selection_json(r#"["old"]"#, r#"["a","b"]"#).as_deref(),
            Some("[]")
        );
        assert_eq!(
            effective_metadata_feed_selection_json(r#"["a","old"]"#, r#"["a","b"]"#).as_deref(),
            Some(r#"["a"]"#)
        );
    }

    #[test]
    fn stream_discovery_episode_context_preserves_episode_order() {
        let context = stream_discovery_episode_context_json(
            "series",
            "tt1:1:2",
            Some(r#"{"videos":[{"id":"tt1:1:2","name":"From detail"}]}"#),
            r#"[{"id":"tt1:1:1","number":1,"name":"Pilot"},{"id":"tt1:1:2","number":2,"name":"Second"}]"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("context");

        assert_eq!(
            context
                .get("expectedEpisodeTitles")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(Value::as_str),
            Some("Second")
        );
        assert_eq!(
            context
                .get("seasonEpisodeIds")
                .and_then(Value::as_object)
                .and_then(|ids| ids.get("2"))
                .and_then(Value::as_str),
            Some("tt1:1:2")
        );
    }

    #[test]
    fn parse_episode_locator_finds_compact_code_with_no_colon_separators() {
        assert_eq!(
            parse_episode_locator("Show.Name.S01E02.1080p"),
            Some((String::new(), 1, 2))
        );
    }

    #[test]
    fn contains_compact_episode_rejects_longer_digit_run_than_target() {
        // "S01E100" must not match a search for episode 10 — the digit run
        // continues past what was parsed for the target.
        assert!(!contains_compact_episode("Show.S01E100.mkv", 1, 10));
        assert!(contains_compact_episode("Show.S01E100.mkv", 1, 100));
        assert!(contains_compact_episode("Show.S01E02.mkv", 1, 2));
    }

    #[test]
    fn contains_spaced_episode_matches_word_form_and_skips_wrong_season_occurrence() {
        assert!(contains_spaced_episode(
            "Show Name Season 1 Episode 2 1080p",
            1,
            2
        ));
        // First "Season 2" occurrence doesn't match the target season (1), so the
        // scan must continue past it to the second "Season ... Episode ..." pair.
        assert!(contains_spaced_episode(
            "Season 2 Episode 1 and Season 1 Episode 2",
            1,
            2
        ));
        // "Episode 10" must not match a search for episode 1 — same digit-run guard
        // as the compact S01E01 form.
        assert!(!contains_spaced_episode("Season 1 Episode 10", 1, 1));
        assert!(contains_spaced_episode("Season 1 Episode 10", 1, 10));
        // A season number that matches but with no "Episode" anywhere after it.
        assert!(!contains_spaced_episode(
            "Season 1 has no further structure",
            1,
            2
        ));
    }

    #[test]
    fn percent_decode_component_decodes_escapes_and_survives_multibyte_input() {
        assert_eq!(percent_decode_component("a%2Bb"), "a+b");
        // Same fix as the duplicate copy in stream_policy.rs — a '%' next to a
        // multi-byte UTF-8 character used to panic on a mid-character slice.
        assert_eq!(percent_decode_component("%xé"), "%xé");
    }
}
