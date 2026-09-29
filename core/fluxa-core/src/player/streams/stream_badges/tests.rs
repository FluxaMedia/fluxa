use super::*;

fn sample_payload() -> &'static str {
    r##"{
        "filters": [
            {"name":"4K","pattern":"\\b4k\\b","imageURL":"https://example.com/4k.png"},
            {"name":"HDR","pattern":"\\bhdr\\b"},
            {"name":"Disabled","pattern":"nope","isEnabled":false}
        ],
        "groups": [{"id":"g1","name":"Quality","color":"#fff"}]
    }"##
}

#[test]
fn parses_import_and_drops_blank_filters() {
    let import_json =
        parse_stream_badge_import_json("https://example.com/badges.json", sample_payload())
            .unwrap();
    let import: StreamBadgeImport = serde_json::from_str(&import_json).unwrap();
    assert_eq!(import.filters.len(), 3);
    assert_eq!(import.groups.len(), 1);
    assert_eq!(import.source_url, "https://example.com/badges.json");
}

#[test]
fn rejects_payload_with_no_usable_filters() {
    let result = parse_stream_badge_import_json(
        "https://example.com/badges.json",
        r#"{"filters":[{"name":"","pattern":""}]}"#,
    );
    assert!(result.is_err());
}

#[test]
fn normalized_caps_imports_and_keeps_one_active() {
    let rules = StreamBadgeRules {
        imports: vec![
            StreamBadgeImport {
                source_url: "a".to_string(),
                filters: vec![StreamBadgeFilter {
                    name: "x".to_string(),
                    pattern: "x".to_string(),
                    ..Default::default()
                }],
                is_active: true,
                ..Default::default()
            },
            StreamBadgeImport {
                source_url: "b".to_string(),
                filters: vec![StreamBadgeFilter {
                    name: "y".to_string(),
                    pattern: "y".to_string(),
                    ..Default::default()
                }],
                is_active: true,
                ..Default::default()
            },
            StreamBadgeImport {
                source_url: "c".to_string(),
                filters: vec![StreamBadgeFilter {
                    name: "z".to_string(),
                    pattern: "z".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            StreamBadgeImport {
                source_url: "d".to_string(),
                filters: vec![StreamBadgeFilter {
                    name: "w".to_string(),
                    pattern: "w".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
    }
    .normalized();
    assert_eq!(rules.imports.len(), STREAM_BADGE_IMPORT_LIMIT);
    assert_eq!(rules.imports.iter().filter(|i| i.is_active).count(), 1);
}

#[test]
fn upsert_replaces_same_source_url_case_insensitively() {
    let rules = StreamBadgeRules::default();
    let first =
        parse_stream_badge_import_json("https://EXAMPLE.com/a.json", sample_payload()).unwrap();
    let rules = serde_json::from_str::<StreamBadgeImport>(&first)
        .map(|import| rules.upsert(import, true))
        .unwrap();
    assert_eq!(rules.imports.len(), 1);

    let second = parse_stream_badge_import_json(
        "https://example.com/a.json",
        r#"{"filters":[{"name":"Only","pattern":"only"}]}"#,
    )
    .unwrap();
    let rules = serde_json::from_str::<StreamBadgeImport>(&second)
        .map(|import| rules.upsert(import, true))
        .unwrap();
    assert_eq!(rules.imports.len(), 1);
    assert_eq!(rules.imports[0].filters.len(), 1);
}

#[test]
fn matches_badges_against_stream_title_and_dedupes_by_image_url() {
    let import_json =
        parse_stream_badge_import_json("https://example.com/badges.json", sample_payload())
            .unwrap();
    let rules_json = format!(r#"{{"imports":[{import_json}]}}"#);

    let stream_json = r#"{"title":"Movie.2024.4K.HDR.mkv"}"#;
    let matched = match_stream_badges_json(stream_json, &rules_json);
    let badges: Vec<StreamBadge> = serde_json::from_str(&matched).unwrap();
    assert_eq!(badges.len(), 2);
    assert_eq!(badges[0].name, "4K");
    assert_eq!(badges[1].name, "HDR");
}

#[test]
fn disabled_filters_and_inactive_imports_do_not_match() {
    let import_json =
        parse_stream_badge_import_json("https://example.com/badges.json", sample_payload())
            .unwrap();
    let rules_json = format!(r#"{{"imports":[{import_json}]}}"#);
    let stream_json = r#"{"title":"nope 4k"}"#;
    let matched = match_stream_badges_json(stream_json, &rules_json);
    let badges: Vec<StreamBadge> = serde_json::from_str(&matched).unwrap();
    assert!(badges.iter().all(|b| b.name != "Disabled"));

    let empty_rules = normalize_stream_badge_rules_json(r#"{"imports":[]}"#);
    assert_eq!(match_stream_badges_json(stream_json, &empty_rules), "[]");
}

#[test]
fn set_active_and_remove_source_manage_the_rule_set() {
    let a =
        parse_stream_badge_import_json("https://a.example/badges.json", sample_payload()).unwrap();
    let b = parse_stream_badge_import_json(
        "https://b.example/badges.json",
        r#"{"filters":[{"name":"Only","pattern":"only"}]}"#,
    )
    .unwrap();
    let rules = StreamBadgeRules::default()
        .upsert(serde_json::from_str(&a).unwrap(), true)
        .upsert(serde_json::from_str(&b).unwrap(), false);
    assert!(
        rules
            .imports
            .iter()
            .find(|i| i.source_url.contains("a.example"))
            .unwrap()
            .is_active
    );

    let rules = rules.set_active_source("https://b.example/badges.json");
    assert!(
        rules
            .imports
            .iter()
            .find(|i| i.source_url.contains("b.example"))
            .unwrap()
            .is_active
    );

    let rules = rules.remove_source("https://a.example/badges.json");
    assert_eq!(rules.imports.len(), 1);
    assert_eq!(rules.imports[0].source_url, "https://b.example/badges.json");
}

#[test]
fn malformed_input_falls_back_to_empty_results() {
    assert_eq!(
        match_stream_badges_json("not json", r#"{"imports":[]}"#),
        "[]"
    );
    assert_eq!(match_stream_badges_json("{}", "not json"), "[]");
}
