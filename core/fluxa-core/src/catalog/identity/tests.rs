use super::*;







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
fn shorten_synopsis_does_not_split_on_titles_or_initials() {
    let text = "Olive Smith, a biology PhD candidate, and Dr. Adam Carlsen fake a relationship. Then things get complicated.";
    assert_eq!(
        shorten_synopsis(text),
        "Olive Smith, a biology PhD candidate, and Dr. Adam Carlsen fake a relationship."
    );
    let text = "A reclusive author named J. R. Hartley returns home. Nobody is pleased.";
    assert_eq!(
        shorten_synopsis(text),
        "A reclusive author named J. R. Hartley returns home."
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
fn stream_matching_uses_torrent_filename_episode_suffix() {
    let e1 = "[SubsPlease] Sousou no Frieren - 01v2 (1080p) [AAA94036].mkv".to_string();
    let e2 = "[SubsPlease] Sousou no Frieren - 02v2 (1080p) [00DB7386].mkv".to_string();
    assert!(!stream_matches_episode("tt123:1:2", &[e1]));
    assert!(stream_matches_episode("tt123:1:2", &[e2]));
}






#[test]
fn cinemeta_series_keeps_its_episodes() {
    let meta = serde_json::json!({
        "id": "tt0903747",
        "type": "series",
        "behaviorHints": {"defaultVideoId": null, "hasScheduledVideos": true},
        "videos": [{"id": "tt0903747:1:1", "season": 1, "episode": 1}]
    });
    let split = split_channel_schedule(meta);
    assert_eq!(split["videos"][0]["id"], "tt0903747:1:1");
    assert!(split.get("schedule").is_none());
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
