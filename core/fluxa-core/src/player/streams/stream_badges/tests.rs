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
