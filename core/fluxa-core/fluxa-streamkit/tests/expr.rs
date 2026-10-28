use fluxa_streamkit::expr::{Compiled, Scope, Value};
use fluxa_streamkit::rules::Pipeline;
use fluxa_streamkit::rules::config::{ExpressionConfig, PipelineConfig, RankedExpression};
use fluxa_streamkit::types::{AddonRef, ParsedStream, RequestContext, StreamType};

fn stream(id: &str, resolution: &str, quality: &str, size_gb: u64) -> ParsedStream {
    let addon = AddonRef {
        instance_id: "a".into(),
        preset_id: "a".into(),
        name: "Alpha".into(),
        ..Default::default()
    };
    let mut s = ParsedStream::blank(id, addon, StreamType::P2p);
    s.parsed.resolution = Some(resolution.into());
    s.parsed.quality = Some(quality.into());
    s.size = Some(size_gb * 1_000_000_000);
    s
}

fn sample() -> Vec<ParsedStream> {
    vec![
        stream("a", "2160p", "BluRay", 40),
        stream("b", "1080p", "WEB-DL", 8),
        stream("c", "1080p", "CAM", 2),
        stream("d", "720p", "WEB-DL", 1),
    ]
}

fn select(src: &str) -> Vec<usize> {
    let c = Compiled::compile(src).unwrap();
    c.eval(&sample(), &Scope::default())
        .unwrap()
        .selected()
        .to_vec()
}

#[test]
fn attribute_filters_nest() {
    assert_eq!(
        select("quality(resolution(streams, '1080p'), 'web-dl')"),
        vec![1]
    );
}

#[test]
fn negate_removes_listed_qualities() {
    assert_eq!(
        select("negate(quality(streams, 'CAM', 'TS'), streams)"),
        vec![0, 1, 3]
    );
}

#[test]
fn size_bounds_accept_unit_strings() {
    assert_eq!(select("size(streams, '1GB', '10GB')"), vec![1, 2, 3]);
}

#[test]
fn per_group_interleaves_round_robin() {
    assert_eq!(
        select("perGroup(streams, 'resolution', 2, '1080p', '2160p')"),
        vec![1, 0, 2]
    );
}

#[test]
fn ternary_picks_a_branch_from_scope() {
    let c = Compiled::compile("isAnime ? quality(streams, 'CAM') : streams").unwrap();
    let mut scope = Scope::default();
    assert_eq!(c.eval(&sample(), &scope).unwrap().selected().len(), 4);
    scope.set("isAnime", Value::Bool(true));
    assert_eq!(c.eval(&sample(), &scope).unwrap().selected(), &[2]);
}

#[test]
fn comments_name_the_expression_unless_hashed() {
    let c = Compiled::compile("/* 4K only */ /*# note */ resolution(streams, '2160p')").unwrap();
    assert_eq!(c.names(), ["4K only"]);
}

#[test]
fn aggregates_over_values() {
    let c = Compiled::compile(
        "avg(values(streams, 'size')) > 10000000000 and count(resolution(streams, '1080p')) == 2",
    )
    .unwrap();
    assert_eq!(
        c.eval(&sample(), &Scope::default()).unwrap().value,
        Value::Bool(true)
    );
    let c = Compiled::compile("count(streams)").unwrap();
    assert_eq!(
        c.eval(&sample(), &Scope::default()).unwrap().value,
        Value::Num(4.0)
    );
    let c = Compiled::compile("max(values(streams, 'size'))").unwrap();
    assert_eq!(
        c.eval(&sample(), &Scope::default()).unwrap().value,
        Value::Num(40e9)
    );
}

#[test]
fn not_binds_looser_than_comparison() {
    let c = Compiled::compile("not 1 == 2 and 3 > 2").unwrap();
    assert!(c.eval(&[], &Scope::default()).unwrap().value == Value::Bool(true));
}

#[test]
fn bad_expressions_fail_at_compile_time() {
    assert!(Compiled::compile("nope(streams)").is_err());
    assert!(Compiled::compile("quality(streams)").is_err());
    assert!(Compiled::compile("whatever + 1").is_err());
    assert!(Compiled::compile("size(streams, 'x').").is_err());
}

#[test]
fn runtime_errors_select_nothing_in_the_pipeline() {
    let mut cfg = PipelineConfig::default();
    cfg.filters.expressions = ExpressionConfig {
        excluded: vec!["streams[".into(), "pin(streams, 'middle')".into()],
        ..Default::default()
    };
    let pipeline = Pipeline::new(cfg);
    assert_eq!(pipeline.validate().len(), 1);
    let out = pipeline.run(&RequestContext::default(), sample());
    assert_eq!(out.streams.len(), 4);
}

#[test]
fn excluded_and_required_expressions_filter_the_survivors() {
    let mut cfg = PipelineConfig::default();
    cfg.filters.expressions = ExpressionConfig {
        excluded: vec!["quality(streams, 'CAM')".into()],
        required: vec!["resolution(streams, '2160p', '1080p')".into()],
        ..Default::default()
    };
    let out = Pipeline::new(cfg).run(&RequestContext::default(), sample());
    let ids: Vec<&str> = out.streams.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"a") && ids.contains(&"b"));
    assert_eq!(out.removed.len(), 2);
}

#[test]
fn included_expressions_bypass_the_filters() {
    let mut cfg = PipelineConfig::default();
    cfg.filters.resolutions.excluded = vec!["720p".into()];
    cfg.filters.expressions.included = vec!["resolution(streams, '720p')".into()];
    let out = Pipeline::new(cfg).run(&RequestContext::default(), sample());
    assert_eq!(out.streams.len(), 4);
}

#[test]
fn ranked_and_preferred_expressions_populate_stream_fields() {
    let mut cfg = PipelineConfig::default();
    cfg.filters.expressions = ExpressionConfig {
        preferred: vec![
            "/* hd */ resolution(streams, '1080p')".into(),
            "streams".into(),
        ],
        ranked: vec![
            RankedExpression {
                expression: "/* big */ size(streams, '5GB')".into(),
                score: 10.0,
                enabled: true,
            },
            RankedExpression {
                expression: "quality(streams, 'WEB-DL')".into(),
                score: 3.0,
                enabled: true,
            },
            RankedExpression {
                expression: "streams".into(),
                score: 100.0,
                enabled: false,
            },
        ],
        pinned: vec!["pin(quality(streams, 'CAM'), 'bottom')".into()],
        ..Default::default()
    };
    let out = Pipeline::new(cfg).run(&RequestContext::default(), sample());
    let by_id = |id: &str| out.streams.iter().find(|s| s.id == id).unwrap();
    assert_eq!(by_id("a").stream_expression_score, 10.0);
    assert_eq!(by_id("b").stream_expression_score, 13.0);
    assert_eq!(by_id("d").stream_expression_score, 3.0);
    assert_eq!(
        by_id("b")
            .stream_expression_matched
            .as_ref()
            .unwrap()
            .name
            .as_deref(),
        Some("hd")
    );
    assert_eq!(
        by_id("a").stream_expression_matched.as_ref().unwrap().index,
        1
    );
    assert_eq!(by_id("b").ranked_stream_expressions_matched.len(), 2);
    assert!(by_id("c").addon.pin.is_some());
}

#[test]
fn passthrough_and_pin_apply_from_any_expression_kind() {
    let mut cfg = PipelineConfig::default();
    cfg.filters.expressions = ExpressionConfig {
        preferred: vec!["pin(resolution(streams, '2160p'), 'top')".into()],
        ranked: vec![RankedExpression {
            expression: "passthrough(quality(streams, 'CAM'), 'limit')".into(),
            score: 0.0,
            enabled: true,
        }],
        ..Default::default()
    };
    let out = Pipeline::new(cfg).run(&RequestContext::default(), sample());
    let by_id = |id: &str| out.streams.iter().find(|s| s.id == id).unwrap();
    assert!(by_id("a").addon.pin.is_some());
    assert!(by_id("c").passthrough.covers("limit"));
    assert!(!by_id("b").passthrough.covers("limit"));
}

fn probed(id: &str, audio: &[(&str, &str)], subs: &[&str]) -> ParsedStream {
    use fluxa_streamkit::types::{MediaInfoQuality, Track};
    let mut s = stream(id, "1080p", "BluRay", 8);
    s.media_info_quality = Some(MediaInfoQuality::Probe);
    s.audio_tracks = audio
        .iter()
        .map(|(lang, codec)| Track {
            lang: Some((*lang).into()),
            codec: Some((*codec).into()),
            ..Default::default()
        })
        .collect();
    s.subtitle_tracks = subs
        .iter()
        .map(|lang| Track {
            lang: Some((*lang).into()),
            ..Default::default()
        })
        .collect();
    s
}

fn pick(src: &str, streams: &[ParsedStream]) -> Vec<usize> {
    Compiled::compile(src)
        .unwrap()
        .eval(streams, &Scope::default())
        .unwrap()
        .selected()
        .to_vec()
}

#[test]
fn track_filters_need_probe_data() {
    let mut unprobed = stream("u", "1080p", "BluRay", 8);
    unprobed.audio_tracks = probed("x", &[("English", "aac")], &[]).audio_tracks;
    let streams = vec![
        probed("a", &[("English", "eac3"), ("French", "aac")], &["English"]),
        probed("b", &[("Japanese", "flac")], &[]),
        unprobed,
    ];
    assert_eq!(
        pick("audioTrack(streams, 'lang=english', 'codec~ac3')", &streams),
        vec![0]
    );
    assert_eq!(pick("audioTrack(streams)", &streams), vec![0, 1]);
    assert_eq!(pick("subtitleTrack(streams)", &streams), vec![0]);
    assert_eq!(
        pick(
            "audioTrack(streams, 'lang=english', 'codec=flac')",
            &streams
        ),
        Vec::<usize>::new()
    );
}

#[test]
fn unknown_track_field_is_a_runtime_error() {
    let streams = vec![probed("a", &[("English", "aac")], &[])];
    let c = Compiled::compile("audioTrack(streams, 'bogus')").unwrap();
    assert!(c.eval(&streams, &Scope::default()).is_err());
}

#[test]
fn media_info_quality_filters_by_source() {
    let streams = vec![probed("a", &[], &[]), stream("b", "1080p", "BluRay", 8)];
    assert_eq!(
        pick("mediaInfoQuality(streams, 'probe')", &streams),
        vec![0]
    );
    assert_eq!(
        pick("mediaInfoQuality(streams, 'Unknown')", &streams),
        vec![1]
    );
}
