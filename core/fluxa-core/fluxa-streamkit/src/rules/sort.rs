use std::cmp::Ordering;

use super::config::{CriteriaSet, Criterion, Direction, PipelineConfig, SortKey};
use super::filter::request_kind;
use crate::types::{ParsedStream, PinPosition, RequestContext};
use crate::vocab::UNKNOWN;

fn list_score(list: &[String], value: &str) -> f64 {
    if list.is_empty() {
        return 0.0;
    }
    match list.iter().position(|v| v == value) {
        Some(i) => -(i as f64),
        None => f64::NEG_INFINITY,
    }
}

fn multi_score(list: &[String], values: &[String]) -> f64 {
    if list.is_empty() {
        return 0.0;
    }
    let owned;
    let values = if values.is_empty() {
        owned = vec![UNKNOWN.to_string()];
        &owned
    } else {
        values
    };
    values
        .iter()
        .filter_map(|v| list.iter().position(|l| l == v))
        .min()
        .map_or(f64::NEG_INFINITY, |i| -(i as f64))
}

fn lower(list: &[String]) -> Vec<String> {
    list.iter().map(|s| s.to_lowercase()).collect()
}

fn score(cfg: &PipelineConfig, s: &ParsedStream, key: SortKey) -> f64 {
    let f = &cfg.filters;
    let p = &s.parsed;
    let flag = |b: bool| if b { 1.0 } else { 0.0 };
    match key {
        SortKey::Size => s.size.map_or(0.0, |v| v as f64),
        SortKey::Bitrate => s.bitrate.map_or(0.0, |v| v as f64),
        SortKey::Seeders => s.seeders().map_or(0.0, f64::from),
        SortKey::Age => s.age.unwrap_or(0.0),
        SortKey::Cached => flag(s.is_cached().unwrap_or(true)),
        SortKey::Library => flag(s.library),
        SortKey::Private => flag(s.torrent.as_ref().is_some_and(|t| t.private)),
        SortKey::Keyword => flag(s.keyword_matched),
        SortKey::Resolution => list_score(
            &f.resolutions.preferred,
            p.resolution.as_deref().unwrap_or(UNKNOWN),
        ),
        SortKey::Quality => list_score(
            &f.qualities.preferred,
            p.quality.as_deref().unwrap_or(UNKNOWN),
        ),
        SortKey::Encode => list_score(&f.encodes.preferred, p.encode.as_deref().unwrap_or(UNKNOWN)),
        SortKey::StreamType => list_score(&cfg.preferred_stream_types, s.stream_type.as_str()),
        SortKey::Service => match &s.service {
            Some(svc) => list_score(&cfg.services, &svc.id),
            None => list_score(&cfg.services, UNKNOWN),
        },
        SortKey::Addon => {
            let by_instance = list_score(&cfg.addons, &s.addon.instance_id);
            if by_instance.is_finite() {
                by_instance
            } else {
                list_score(&cfg.addons, &s.addon.preset_id)
            }
        }
        SortKey::ReleaseGroup => list_score(
            &lower(&f.release_groups.preferred),
            &p.release_group.as_deref().unwrap_or(UNKNOWN).to_lowercase(),
        ),
        SortKey::VisualTag => multi_score(&f.visual_tags.preferred, &p.visual_tags),
        SortKey::AudioTag => multi_score(&f.audio_tags.preferred, &p.audio_tags),
        SortKey::Language => multi_score(&f.languages.preferred, &p.languages),
        SortKey::Subtitle => multi_score(&f.subtitles.preferred, &p.subtitles),
        SortKey::RegexPatterns => s
            .regex_matched
            .as_ref()
            .map_or(f64::NEG_INFINITY, |m| -(m.index as f64)),
        SortKey::StreamExpressionMatched => s
            .stream_expression_matched
            .as_ref()
            .map_or(f64::NEG_INFINITY, |m| -(m.index as f64)),
        SortKey::StreamExpressionScore => s.stream_expression_score,
        SortKey::RegexScore => s.regex_score,
        SortKey::Seadex => match &s.seadex {
            Some(x) if x.is_best => 2.0,
            Some(_) => 1.0,
            None => 0.0,
        },
        SortKey::AudioChannel => 0.0,
    }
}

fn sort_with(cfg: &PipelineConfig, streams: &mut [ParsedStream], criteria: &[Criterion]) {
    if criteria.is_empty() {
        return;
    }
    let mut keyed: Vec<(Vec<f64>, ParsedStream)> = streams
        .iter()
        .cloned()
        .map(|s| (criteria.iter().map(|c| score(cfg, &s, c.key)).collect(), s))
        .collect();
    keyed.sort_by(|(a, _), (b, _)| {
        for (i, c) in criteria.iter().enumerate() {
            let ord = a[i].partial_cmp(&b[i]).unwrap_or(Ordering::Equal);
            let ord = match c.direction {
                Direction::Desc => ord.reverse(),
                Direction::Asc => ord,
            };
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    });
    for (slot, (_, s)) in streams.iter_mut().zip(keyed) {
        *slot = s;
    }
}

fn pick<'a>(
    cfg: &'a PipelineConfig,
    ctx: &RequestContext,
) -> (&'a [Criterion], &'a [Criterion], &'a [Criterion]) {
    let specific: &CriteriaSet = match request_kind(ctx) {
        "anime" => &cfg.sort.anime,
        "series" => &cfg.sort.series,
        _ => &cfg.sort.movies,
    };
    let choose = |own: &'a Vec<Criterion>, global: &'a Vec<Criterion>| -> &'a [Criterion] {
        if own.is_empty() { global } else { own }
    };
    (
        choose(&specific.global, &cfg.sort.global.global),
        choose(&specific.cached, &cfg.sort.global.cached),
        choose(&specific.uncached, &cfg.sort.global.uncached),
    )
}

pub fn sort(
    cfg: &PipelineConfig,
    ctx: &RequestContext,
    streams: Vec<ParsedStream>,
) -> Vec<ParsedStream> {
    let (mut top, mut bottom, mut rest) = (Vec::new(), Vec::new(), Vec::new());
    for s in streams {
        match s.addon.pin {
            Some(PinPosition::Top) => top.push(s),
            Some(PinPosition::Bottom) => bottom.push(s),
            None => rest.push(s),
        }
    }
    let (primary, cached_list, uncached_list) = pick(cfg, ctx);
    let split = !cached_list.is_empty()
        && !uncached_list.is_empty()
        && primary.first().is_some_and(|c| c.key == SortKey::Cached);
    if split {
        let (mut cached, mut uncached): (Vec<_>, Vec<_>) = rest
            .into_iter()
            .partition(|s| s.is_cached().unwrap_or(true));
        sort_with(cfg, &mut cached, cached_list);
        sort_with(cfg, &mut uncached, uncached_list);
        let cached_first = primary
            .first()
            .is_some_and(|c| c.direction == Direction::Desc);
        rest = if cached_first {
            cached.into_iter().chain(uncached).collect()
        } else {
            uncached.into_iter().chain(cached).collect()
        };
    } else {
        sort_with(cfg, &mut rest, primary);
    }
    top.into_iter().chain(rest).chain(bottom).collect()
}
