use fluxa_streamkit::parse::{AddonParser, DefaultParser, ParseEnv, ParseOutcome, parse_file};
use fluxa_streamkit::rules::Pipeline;
use fluxa_streamkit::rules::config::{BucketMode, Criterion, Direction, PipelineConfig, SortKey};
use fluxa_streamkit::types::{
    AddonRef, MediaType, ParsedStream, RawStream, RequestContext, StreamType,
};

fn addon(id: &str) -> AddonRef {
    AddonRef {
        instance_id: id.into(),
        preset_id: id.into(),
        name: id.into(),
        ..Default::default()
    }
}

fn parse_raw(json: &str, addon_id: &str) -> ParsedStream {
    let raw: RawStream = serde_json::from_str(json).unwrap();
    let mut seq = 0;
    match DefaultParser
        .parse(raw, &addon(addon_id), &mut seq, &ParseEnv { now_ms: 0 })
        .unwrap()
    {
        ParseOutcome::Stream(s) => *s,
        ParseOutcome::Skip => panic!("skipped"),
    }
}

#[test]
fn addon_style_debrid_stream() {
    let s = parse_raw(
        r#"{"name":"[RD+] ExampleAddon\n4k","title":"Dune.2021.2160p.UHD.BluRay.x265.HDR10-GRP\n👤 120 💾 14.5 GB ⚙️ ExampleIndexer","infoHash":"abcdef0123456789abcdef0123456789abcdef01"}"#,
        "exampleaddon",
    );
    assert_eq!(s.stream_type, StreamType::Debrid);
    let svc = s.service.clone().unwrap();
    assert_eq!(svc.id, "realdebrid");
    assert!(svc.cached);
    assert_eq!(s.seeders(), Some(120));
    assert_eq!(s.indexer.as_deref(), Some("ExampleIndexer"));
    assert_eq!(s.parsed.resolution.as_deref(), Some("2160p"));
    assert_eq!(
        s.size,
        Some((14.5f64 * 1024.0 * 1024.0 * 1024.0).round() as u64)
    );
}

#[test]
fn magnet_url_becomes_info_hash() {
    let s = parse_raw(
        r#"{"title":"Movie.2020.1080p.WEB-DL-GRP","url":"magnet:?xt=urn:btih:ABCDEF0123456789ABCDEF0123456789ABCDEF01&dn=x"}"#,
        "a",
    );
    assert_eq!(s.stream_type, StreamType::P2p);
    assert_eq!(
        s.torrent.unwrap().info_hash,
        "abcdef0123456789abcdef0123456789abcdef01"
    );
}

#[test]
fn account_error_is_rejected() {
    let raw: RawStream =
        serde_json::from_str(r#"{"description":"Invalid RealDebrid apikey","url":"x"}"#).unwrap();
    let mut seq = 0;
    assert!(
        DefaultParser
            .parse(raw, &addon("a"), &mut seq, &ParseEnv { now_ms: 0 })
            .is_err()
    );
}

#[test]
fn folder_equal_to_filename_is_dropped() {
    let s = parse_raw(
        r#"{"url":"http://x/y","behaviorHints":{"filename":"Movie.2020.1080p.mkv","folderName":"Movie 2020 1080p"}}"#,
        "a",
    );
    assert!(s.folder_name.is_none());
}

#[test]
fn language_flags_add_languages() {
    let s = parse_raw(
        r#"{"url":"http://x/y","description":"Movie.2020.1080p.BluRay-GRP\n🇫🇷 🇩🇪"}"#,
        "a",
    );
    assert!(s.parsed.languages.contains(&"French".to_string()));
    assert!(s.parsed.languages.contains(&"German".to_string()));
}

#[test]
fn file_name_tags_ignore_title_words() {
    let f = parse_file("Dune.Part.Two.2024.2160p.WEB-DL.DDP5.1.Atmos.DV.HDR10+.H.265-FLUX");
    assert_eq!(f.resolution.as_deref(), Some("2160p"));
    assert_eq!(f.quality.as_deref(), Some("WEB-DL"));
    assert_eq!(f.encode.as_deref(), Some("HEVC"));
    assert!(f.audio_tags.contains(&"Atmos".to_string()));
    assert!(f.visual_tags.contains(&"HDR10+".to_string()));
    assert_eq!(f.release_group.as_deref(), Some("FLUX"));
}

fn stream(id: &str, json: &str, addon_id: &str) -> ParsedStream {
    let mut s = parse_raw(json, addon_id);
    s.id = id.into();
    s
}

fn movie_ctx() -> RequestContext {
    RequestContext {
        media_type: MediaType::Movie,
        ..Default::default()
    }
}

#[test]
fn duplicate_hash_keeps_cached_only() {
    let hash = "abcdef0123456789abcdef0123456789abcdef01";
    let cached = stream(
        "1",
        &format!(r#"{{"name":"[RD+]","title":"Movie.2020.1080p-GRP","infoHash":"{hash}"}}"#),
        "a",
    );
    let p2p = stream(
        "2",
        &format!(r#"{{"title":"Movie.2020.1080p-GRP","infoHash":"{hash}"}}"#),
        "b",
    );
    let mut cfg = PipelineConfig::default();
    cfg.deduplicator.enabled = true;
    let out = Pipeline::new(cfg).run(&movie_ctx(), vec![p2p, cached]);
    assert_eq!(out.streams.len(), 1);
    assert_eq!(out.streams[0].id, "1");
}

#[test]
fn per_addon_mode_keeps_one_copy_per_addon() {
    let make = |id: &str, addon_id: &str| {
        stream(
            id,
            r#"{"title":"Movie.2020.1080p-GRP","infoHash":"abcdef0123456789abcdef0123456789abcdef01"}"#,
            addon_id,
        )
    };
    let mut cfg = PipelineConfig::default();
    cfg.deduplicator.enabled = true;
    cfg.deduplicator.p2p = BucketMode::PerAddon;
    let out = Pipeline::new(cfg).run(
        &movie_ctx(),
        vec![make("1", "a"), make("2", "a"), make("3", "b")],
    );
    let mut ids: Vec<_> = out.streams.iter().map(|s| s.id.clone()).collect();
    ids.sort();
    assert_eq!(ids, vec!["1", "3"]);
}

#[test]
fn required_resolution_drops_unknown() {
    let known = stream(
        "1",
        r#"{"url":"http://a","title":"Movie.2020.1080p.WEB-DL-GRP"}"#,
        "a",
    );
    let unknown = stream("2", r#"{"url":"http://b","title":"Movie.2020-GRP"}"#, "a");
    let mut cfg = PipelineConfig::default();
    cfg.filters.resolutions.required = vec!["1080p".into()];
    let out = Pipeline::new(cfg).run(&movie_ctx(), vec![known, unknown]);
    assert_eq!(out.streams.len(), 1);
    assert_eq!(out.removed.len(), 1);
}

#[test]
fn sort_by_resolution_preference_then_size() {
    let a = stream(
        "1",
        r#"{"url":"http://a","title":"M.2020.720p-G","behaviorHints":{"videoSize":9000}}"#,
        "a",
    );
    let b = stream(
        "2",
        r#"{"url":"http://b","title":"M.2020.2160p-G","behaviorHints":{"videoSize":1000}}"#,
        "a",
    );
    let c = stream(
        "3",
        r#"{"url":"http://c","title":"M.2020.2160p-G","behaviorHints":{"videoSize":5000}}"#,
        "a",
    );
    let mut cfg = PipelineConfig::default();
    cfg.filters.resolutions.preferred = vec!["2160p".into(), "1080p".into(), "720p".into()];
    cfg.sort.global.global = vec![
        Criterion {
            key: SortKey::Resolution,
            direction: Direction::Desc,
        },
        Criterion {
            key: SortKey::Size,
            direction: Direction::Desc,
        },
    ];
    let out = Pipeline::new(cfg).run(&movie_ctx(), vec![a, b, c]);
    let ids: Vec<_> = out.streams.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["3", "2", "1"]);
}

#[test]
fn global_limit_keeps_best_streams() {
    let make = |id: &str| {
        stream(
            id,
            &format!(r#"{{"url":"http://{id}","title":"M.2020.1080p-G"}}"#),
            "a",
        )
    };
    let mut cfg = PipelineConfig::default();
    cfg.limits.global = Some(2);
    let out = Pipeline::new(cfg).run(&movie_ctx(), vec![make("1"), make("2"), make("3")]);
    assert_eq!(out.streams.len(), 2);
    assert_eq!(out.streams[0].id, "1");
}

#[test]
fn episode_filter_rejects_other_season() {
    let right = stream(
        "1",
        r#"{"url":"http://a","title":"Show.S02E05.1080p-G"}"#,
        "a",
    );
    let wrong = stream(
        "2",
        r#"{"url":"http://b","title":"Show.S03E05.1080p-G"}"#,
        "a",
    );
    let mut cfg = PipelineConfig::default();
    cfg.filters.season_episode_matching.enabled = true;
    let ctx = RequestContext {
        media_type: MediaType::Series,
        season: Some(2),
        episode: Some(5),
        ..Default::default()
    };
    let out = Pipeline::new(cfg).run(&ctx, vec![right, wrong]);
    assert_eq!(out.streams.len(), 1);
    assert_eq!(out.streams[0].id, "1");
}
