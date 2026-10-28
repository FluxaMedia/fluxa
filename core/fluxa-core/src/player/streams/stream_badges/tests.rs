use super::*;

fn labels(text: &str, size: Option<i64>) -> Vec<String> {
    parse(text, size, Options::default(), &Rules::default())
        .into_iter()
        .map(|badge| badge.label)
        .collect()
}

#[test]
fn torrentio_style_description() {
    let found = labels(
        "Movie.2024.2160p.UHD.BluRay.REMUX.DV.HDR10.HEVC.TrueHD.Atmos.7.1-GRP\n👤 142 💾 58.4 GB ⚙️ ThePirateBay",
        None,
    );
    assert_eq!(
        found,
        [
            "4K", "REMUX", "BluRay", "DV", "HDR10", "HEVC", "Atmos", "TrueHD", "7.1", "58.4 GB",
            "142"
        ]
    );
}

#[test]
fn hdr10_plus_replaces_plain_hdr() {
    let found = labels("Show S01E01 1080p WEB-DL HDR10+ DDP5.1 x265", None);
    assert!(found.contains(&"HDR10+".to_owned()));
    assert!(!found.contains(&"HDR".to_owned()));
    assert!(!found.contains(&"HDR10".to_owned()));
}

#[test]
fn behavior_hint_size_wins_over_text() {
    let found = labels("1080p 2.1 GB", Some(3_221_225_472));
    assert!(found.contains(&"3.00 GB".to_owned()));
    assert!(!found.contains(&"2.1 GB".to_owned()));
}

#[test]
fn plain_names_have_no_badges() {
    assert!(labels("Some Addon", None).is_empty());
}

fn pack(json: &str) -> Pack {
    parse_pack(json, Some("https://example.com/p.json")).unwrap()
}

#[test]
fn custom_filter_matches_by_regex_with_its_own_look() {
    let mut rules = Rules::default();
    rules.add_pack(pack(
        r##"{"name":"Mine","filters":[{"id":"a","name":"Scene","pattern":"-(GRP|FOO)\\b","tagColor":"#FF0000","tagStyle":"OUTLINE","imageURL":"https://x/i.png"}]}"##,
    ));
    let found = parse("Movie.1080p-GRP", None, Options::default(), &rules);
    let custom = found.last().unwrap();
    assert_eq!(custom.label, "Scene");
    assert_eq!(custom.look.fill, Some(0xff0000ff));
    assert!(custom.look.outline);
    assert_eq!(custom.image_url.as_deref(), Some("https://x/i.png"));
    assert!(
        parse("Movie.1080p-BAR", None, Options::default(), &rules)
            .iter()
            .all(|b| b.label != "Scene")
    );
}

#[test]
fn pack_import_drops_broken_patterns_and_caps_at_three() {
    let imported = pack(r#"[{"name":"ok","pattern":"x"},{"name":"bad","pattern":"("}]"#);
    assert_eq!(imported.filters.len(), 1);
    let mut rules = Rules::default();
    for n in 0..4 {
        let url = format!("https://e/{n}.json");
        let pack = parse_pack(r#"[{"name":"ok","pattern":"x"}]"#, Some(&url)).unwrap();
        assert_eq!(rules.add_pack(pack), n < 3);
    }
}

#[test]
fn built_in_and_size_can_be_turned_off() {
    let options = Options {
        built_in: false,
        file_size: false,
        ..Options::default()
    };
    assert!(parse("2160p HDR 4.5 GB", None, options, &Rules::default()).is_empty());
}

#[test]
fn custom_badge_needs_name_and_valid_regex() {
    let mut rules = Rules::default();
    let filter = |name: &str, pattern: &str| Filter {
        name: name.into(),
        pattern: pattern.into(),
        is_enabled: true,
        ..Default::default()
    };
    assert!(!rules.upsert_custom(filter("", "x")));
    assert!(!rules.upsert_custom(filter("a", "(")));
    assert!(rules.upsert_custom(filter("a", "x")));
    assert_eq!(rules.custom.len(), 1);
}

#[test]
fn theme_colours_built_in_badges() {
    let options = Options {
        theme: Theme::Classic,
        ..Options::default()
    };
    let plain = parse("2160p HDR", None, options, &Rules::default());
    assert!(plain.iter().all(|badge| badge.look.fill.is_none()));
    let vivid = parse("2160p HDR", None, Options::default(), &Rules::default());
    assert!(vivid.iter().all(|badge| badge.look.fill.is_some()));
}

fn playing_labels(text: &str, playing: &Playing) -> Vec<String> {
    parse_playing(text, None, Options::default(), &Rules::default(), playing)
        .into_iter()
        .map(|badge| badge.label)
        .collect()
}

#[test]
fn declared_hdr_dropped_when_playback_is_sdr() {
    let playing = Playing {
        width: Some(3840),
        video_codec: Some("hevc".to_owned()),
        gamma: Some("bt.1886".to_owned()),
        ..Playing::default()
    };
    let found = playing_labels("Movie 2160p BluRay HDR10 DV HEVC", &playing);
    assert!(found.contains(&"4K".to_owned()));
    assert!(found.contains(&"BluRay".to_owned()));
    assert!(!found.contains(&"HDR10".to_owned()));
    assert!(!found.contains(&"DV".to_owned()));
}

#[test]
fn undeclared_dolby_vision_shows_up_from_playback() {
    let playing = Playing {
        width: Some(3840),
        gamma: Some("pq".to_owned()),
        dolby_vision: true,
        ..Playing::default()
    };
    let found = playing_labels("Movie 2160p WEB-DL", &playing);
    assert!(found.contains(&"DV".to_owned()));
    assert!(found.contains(&"HDR10".to_owned()));
}

#[test]
fn atmos_kept_when_decoder_is_an_atmos_carrier() {
    let playing = Playing {
        audio_codec: Some("truehd".to_owned()),
        channels: Some("7.1".to_owned()),
        ..Playing::default()
    };
    let found = playing_labels("Movie TrueHD Atmos 7.1", &playing);
    assert!(found.contains(&"Atmos".to_owned()));
    assert!(found.contains(&"TrueHD".to_owned()));
    let aac = Playing {
        audio_codec: Some("aac".to_owned()),
        ..Playing::default()
    };
    assert!(!playing_labels("Movie TrueHD Atmos 7.1", &aac).contains(&"Atmos".to_owned()));
}

#[test]
fn scoped_pack_ignores_other_packs() {
    let mut rules = Rules::default();
    for (id, name) in [("a", "Alpha"), ("b", "Beta")] {
        rules.add_pack(Pack {
            id: id.to_owned(),
            name: name.to_owned(),
            is_active: true,
            filters: vec![Filter {
                id: id.to_owned(),
                name: name.to_owned(),
                pattern: "movie".to_owned(),
                is_enabled: true,
                ..Filter::default()
            }],
            ..Pack::default()
        });
    }
    let (scoped, built_in) = rules.scoped("b");
    assert_eq!(built_in, Some(false));
    let names: Vec<_> = scoped.active_filters().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Beta"]);
    assert_eq!(rules.scoped("missing").1, None);
}
