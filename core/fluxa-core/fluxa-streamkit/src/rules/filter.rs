use regex::Regex;
use serde::{Deserialize, Serialize};

use super::config::{
    Ambiguous, Band, FilterConfig, KeywordFilter, Limits, ListRule, PatternRule, RangeRule, Scope,
    TitleMode,
};
use crate::fuzzy::{normalise_title, partial_ratio, ratio};
use crate::types::{MediaType, ParsedStream, RequestContext, StreamType};
use crate::vocab::{self, UNKNOWN};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Reason {
    TooOld,
    StreamType,
    Resolution,
    Quality,
    Encode,
    VisualTag,
    AudioTag,
    AudioChannel,
    Language,
    Subtitle,
    ReleaseGroup,
    Uncached,
    Cached,
    SeasonPack,
    Regex,
    Keyword,
    Seeders,
    Age,
    Year,
    SeasonEpisode,
    EpisodeTitle,
    Title,
    Size,
    Bitrate,
    Duplicate,
    Limit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Removal {
    pub stream_id: String,
    pub reason: Reason,
    pub detail: Option<String>,
}

pub struct Filter<'a> {
    cfg: &'a FilterConfig,
    included_regex: Vec<(Regex, bool)>,
    excluded_regex: Vec<(Regex, bool)>,
    required_regex: Vec<(Regex, bool)>,
    included_kw: Option<Regex>,
    excluded_kw: Option<Regex>,
    required_kw: Option<Regex>,
}

pub fn compile_patterns(rules: &[PatternRule]) -> Vec<(Regex, bool)> {
    rules
        .iter()
        .filter_map(|r| {
            Regex::new(&format!("(?i){}", r.pattern))
                .ok()
                .map(|re| (re, r.negate))
        })
        .collect()
}

pub fn compile_keywords(words: &[String]) -> Option<Regex> {
    if words.is_empty() {
        return None;
    }
    let alt = words
        .iter()
        .map(|w| regex::escape(w).replace(' ', "[ ._-]"))
        .collect::<Vec<_>>()
        .join("|");
    Regex::new(&format!(r"(?i)(?:^|[ ._\-\[(])(?:{alt})(?:$|[ ._\-\])])")).ok()
}

pub fn searchable_fields(s: &ParsedStream) -> Vec<&str> {
    [
        s.filename.as_deref(),
        s.folder_name.as_deref(),
        s.parsed.release_group.as_deref(),
        s.indexer.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn pattern_hit(re: &Regex, negate: bool, fields: &[&str]) -> bool {
    let any = fields.iter().any(|f| re.is_match(f));
    any != negate
}

fn any_pattern(list: &[(Regex, bool)], fields: &[&str]) -> bool {
    list.iter().any(|(re, neg)| pattern_hit(re, *neg, fields))
}

fn kw_hit(re: &Option<Regex>, fields: &[&str]) -> bool {
    re.as_ref()
        .is_some_and(|re| fields.iter().any(|f| re.is_match(f)))
}

fn one(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or(UNKNOWN)
}

fn many(values: &[String]) -> Vec<String> {
    if values.is_empty() {
        vec![UNKNOWN.to_string()]
    } else {
        values.to_vec()
    }
}

fn hits(list: &[String], values: &[String]) -> bool {
    values.iter().any(|v| list.iter().any(|l| l == v))
}

pub fn request_kind(ctx: &RequestContext) -> &'static str {
    if ctx.is_anime {
        "anime"
    } else {
        match ctx.media_type {
            MediaType::Movie => "movie",
            MediaType::Series => "series",
        }
    }
}

fn in_scope(scope: &Scope, ctx: &RequestContext, s: &ParsedStream) -> bool {
    let type_ok = scope.request_types.is_empty()
        || scope.request_types.iter().any(|t| t == request_kind(ctx));
    let addon_ok = scope.addons.is_empty()
        || scope
            .addons
            .iter()
            .any(|a| *a == s.addon.preset_id || *a == s.addon.instance_id);
    type_ok && addon_ok
}

pub fn visual_view(tags: &[String]) -> Vec<String> {
    let mut out = many(tags);
    let hdr_idx = out.iter().position(|t| t.starts_with("HDR"));
    let dv_idx = out.iter().position(|t| t.starts_with("DV"));
    match (hdr_idx, dv_idx) {
        (Some(h), Some(d)) => out.insert(h.min(d), "HDR+DV".into()),
        (None, Some(_)) => out.push("DV Only".into()),
        (Some(_), None) => out.push("HDR Only".into()),
        _ => {}
    }
    out
}

fn language_view(s: &ParsedStream, ctx: &RequestContext) -> Vec<String> {
    let mut out = many(&s.parsed.languages);
    if let Some(orig) = ctx
        .original_language
        .as_deref()
        .and_then(vocab::language_by_iso)
    {
        if out.iter().any(|l| l == orig.name) {
            out.push("Original".into());
        }
    }
    out
}

fn band_check(band: &Band, value: Option<f64>) -> bool {
    value.is_some_and(|v| band.contains(v))
}

fn scoped(types: &[String], s: &ParsedStream, kind: &str) -> bool {
    if types.is_empty() {
        return true;
    }
    types.iter().any(|t| match (kind, t.as_str()) {
        ("seeders", "p2p") => s.stream_type == StreamType::P2p,
        ("seeders", "cached") => s.is_cached() == Some(true),
        ("seeders", "uncached") => s.is_cached() == Some(false),
        ("age", "debrid") => s.stream_type == StreamType::Debrid,
        ("age", "usenet") => s.stream_type.is_usenet(),
        ("age", "p2p") => s.stream_type == StreamType::P2p,
        _ => false,
    })
}

fn range_required(rule: &RangeRule, value: Option<f64>, s: &ParsedStream, kind: &str) -> bool {
    if rule.required.is_unset() || !scoped(&rule.types, s, kind) {
        return true;
    }
    let v = value.unwrap_or(0.0);
    if rule.required.min.is_some_and(|m| v < m) {
        return false;
    }
    !(value.is_some() && rule.required.max.is_some_and(|m| v > m))
}

fn range_excluded(rule: &RangeRule, value: Option<f64>, s: &ParsedStream, kind: &str) -> bool {
    !rule.excluded.is_unset() && scoped(&rule.types, s, kind) && band_check(&rule.excluded, value)
}

fn limits_for<'a>(limits: &'a Limits, ctx: &RequestContext) -> Vec<&'a Band> {
    let specific: Vec<&Band> = match request_kind(ctx) {
        "movie" => vec![&limits.movies],
        "series" => vec![&limits.series],
        _ => vec![&limits.anime, &limits.series],
    };
    specific
        .into_iter()
        .chain(std::iter::once(&limits.global))
        .filter(|b| !b.is_unset())
        .collect()
}

fn size_out_of_range(
    cfg: &super::config::ScaledLimits,
    resolution: &str,
    value: Option<f64>,
    ctx: &RequestContext,
) -> bool {
    let Some(v) = value else { return false };
    let per_res = cfg
        .resolutions
        .get(resolution)
        .map(|l| limits_for(l, ctx))
        .filter(|b| !b.is_empty());
    let bands = per_res.unwrap_or_else(|| limits_for(&cfg.global, ctx));
    match bands.first() {
        Some(b) => !b.contains(v),
        None => false,
    }
}

impl<'a> Filter<'a> {
    pub fn new(cfg: &'a FilterConfig) -> Self {
        let KeywordFilter {
            included,
            excluded,
            required,
            ..
        } = &cfg.keywords;
        Self {
            cfg,
            included_regex: compile_patterns(&cfg.regex.included),
            excluded_regex: compile_patterns(&cfg.regex.excluded),
            required_regex: compile_patterns(&cfg.regex.required),
            included_kw: compile_keywords(included),
            excluded_kw: compile_keywords(excluded),
            required_kw: compile_keywords(required),
        }
    }

    fn passes_stage(s: &ParsedStream, stage: &str) -> bool {
        s.addon.passthrough || s.passthrough.covers(stage)
    }

    fn list_hard(rule: &ListRule, values: &[String], reason: Reason) -> Result<(), Reason> {
        if hits(&rule.excluded, values) {
            return Err(reason);
        }
        if !rule.required.is_empty() && !hits(&rule.required, values) {
            return Err(reason);
        }
        Ok(())
    }

    fn list_soft(rule: &ListRule, values: &[String], reason: Reason) -> Result<(), Reason> {
        if !rule.excluded.is_empty() && values.iter().all(|v| rule.excluded.contains(v)) {
            return Err(reason);
        }
        if !rule.required.is_empty() && !hits(&rule.required, values) {
            return Err(reason);
        }
        Ok(())
    }

    fn included(&self, s: &ParsedStream, ctx: &RequestContext) -> bool {
        let c = self.cfg;
        let p = &s.parsed;
        let fields = searchable_fields(s);
        let stream_type = vec![s.stream_type.as_str().to_string()];
        let group: Vec<String> = p.release_group.iter().map(|g| g.to_lowercase()).collect();
        let included_groups: Vec<String> = c
            .release_groups
            .included
            .iter()
            .map(|g| g.to_lowercase())
            .collect();
        hits(&c.stream_types.included, &stream_type)
            || hits(&c.resolutions.included, &[one(&p.resolution).to_string()])
            || hits(&c.qualities.included, &[one(&p.quality).to_string()])
            || hits(&c.encodes.included, &[one(&p.encode).to_string()])
            || hits(&c.visual_tags.included, &visual_view(&p.visual_tags))
            || hits(&c.audio_tags.included, &many(&p.audio_tags))
            || hits(&c.audio_channels.included, &many(&p.audio_channels))
            || (!c.skip_language_filter && hits(&c.languages.included, &language_view(s, ctx)))
            || (!c.skip_language_filter && hits(&c.subtitles.included, &many(&p.subtitles)))
            || hits(&included_groups, &group)
            || any_pattern(&self.included_regex, &fields)
            || kw_hit(&self.included_kw, &fields)
            || (!c.seeders.included.is_unset()
                && scoped(&c.seeders.types, s, "seeders")
                && band_check(&c.seeders.included, s.seeders().map(f64::from)))
            || (!c.age.included.is_unset()
                && scoped(&c.age.types, s, "age")
                && band_check(&c.age.included, s.age))
    }

    pub fn decide(&self, s: &ParsedStream, ctx: &RequestContext) -> Result<Option<String>, Reason> {
        let c = self.cfg;
        let p = &s.parsed;
        if let Some(max) = c.max_age_hours {
            if s.age.is_some_and(|a| a > max) && !Self::passes_stage(s, "digitalRelease") {
                return Err(Reason::TooOld);
            }
        }
        if Self::passes_stage(s, "filter") || self.included(s, ctx) {
            return Ok(None);
        }

        let stream_type = vec![s.stream_type.as_str().to_string()];
        Self::list_hard(&c.stream_types, &stream_type, Reason::StreamType)?;
        if s.stream_type == StreamType::Info {
            return Ok(None);
        }
        Self::list_hard(
            &c.resolutions,
            &[one(&p.resolution).to_string()],
            Reason::Resolution,
        )?;
        Self::list_hard(
            &c.qualities,
            &[one(&p.quality).to_string()],
            Reason::Quality,
        )?;
        Self::list_hard(&c.encodes, &[one(&p.encode).to_string()], Reason::Encode)?;
        Self::list_hard(
            &c.visual_tags,
            &visual_view(&p.visual_tags),
            Reason::VisualTag,
        )?;
        Self::list_hard(&c.audio_tags, &many(&p.audio_tags), Reason::AudioTag)?;
        Self::list_hard(
            &c.audio_channels,
            &many(&p.audio_channels),
            Reason::AudioChannel,
        )?;

        if !c.skip_language_filter && !Self::passes_stage(s, "language") {
            Self::list_soft(&c.languages, &language_view(s, ctx), Reason::Language)?;
        }
        if !Self::passes_stage(s, "subtitle") {
            Self::list_soft(&c.subtitles, &many(&p.subtitles), Reason::Subtitle)?;
        }

        let group = p.release_group.clone().map(|g| g.to_lowercase());
        let group_view = vec![group.clone().unwrap_or_else(|| UNKNOWN.to_lowercase())];
        let lower_rule = ListRule {
            excluded: c
                .release_groups
                .excluded
                .iter()
                .map(|g| g.to_lowercase())
                .collect(),
            required: c
                .release_groups
                .required
                .iter()
                .map(|g| g.to_lowercase())
                .collect(),
            ..Default::default()
        };
        Self::list_hard(&lower_rule, &group_view, Reason::ReleaseGroup)?;

        match s.is_cached() {
            Some(false) if c.exclude_uncached => return Err(Reason::Uncached),
            Some(true) if c.exclude_cached => return Err(Reason::Cached),
            _ => {}
        }
        if c.exclude_season_packs
            && ctx.media_type == MediaType::Series
            && !p.seasons.is_empty()
            && p.episodes.is_empty()
        {
            return Err(Reason::SeasonPack);
        }

        let fields = searchable_fields(s);
        if any_pattern(&self.excluded_regex, &fields) {
            return Err(Reason::Regex);
        }
        if !self.required_regex.is_empty() && !any_pattern(&self.required_regex, &fields) {
            return Err(Reason::Regex);
        }
        if kw_hit(&self.excluded_kw, &fields) {
            return Err(Reason::Keyword);
        }
        if self.required_kw.is_some() && !kw_hit(&self.required_kw, &fields) {
            return Err(Reason::Keyword);
        }

        let seeders = s.seeders().map(f64::from);
        if !range_required(&c.seeders, seeders, s, "seeders")
            || range_excluded(&c.seeders, seeders, s, "seeders")
        {
            return Err(Reason::Seeders);
        }
        if !range_required(&c.age, s.age, s, "age") || range_excluded(&c.age, s.age, s, "age") {
            return Err(Reason::Age);
        }

        if !Self::passes_stage(s, "year") && !self.year_ok(s, ctx) {
            return Err(Reason::Year);
        }
        if !Self::passes_stage(s, "episode") {
            if !self.season_episode_ok(s, ctx) {
                return Err(Reason::SeasonEpisode);
            }
            if !self.episode_title_ok(s, ctx) {
                return Err(Reason::EpisodeTitle);
            }
        }
        let mut inferred = None;
        if !Self::passes_stage(s, "title") {
            match self.title_match(s, ctx) {
                Some(lang) => inferred = lang,
                None => return Err(Reason::Title),
            }
        }

        let resolution = one(&p.resolution);
        if size_out_of_range(&c.size, resolution, s.size.map(|v| v as f64), ctx) {
            return Err(Reason::Size);
        }
        if size_out_of_range(&c.bitrate, resolution, s.bitrate.map(|v| v as f64), ctx) {
            return Err(Reason::Bitrate);
        }
        Ok(inferred)
    }

    fn year_ok(&self, s: &ParsedStream, ctx: &RequestContext) -> bool {
        let y = &self.cfg.year_matching;
        let Some(req_year) = ctx.year else {
            return true;
        };
        if !y.enabled || !in_scope(&y.scope, ctx, s) {
            return true;
        }
        let Some(stream_year) = s.parsed.year.as_deref() else {
            return !(y.strict && y.strict_types.iter().any(|t| t == request_kind(ctx)));
        };
        let mut parts = stream_year.split('-').filter_map(|p| p.parse::<i32>().ok());
        let (Some(lo), hi) = (parts.next(), parts.next()) else {
            return true;
        };
        let (lo, hi) = (lo - y.tolerance, hi.unwrap_or(lo) + y.tolerance);
        let multi_season = s.parsed.seasons.len() > 1;
        if matches!(ctx.media_type, MediaType::Series) && !y.use_initial_air_date && !multi_season {
            let mut years = ctx.release_years.clone();
            years.extend(ctx.season_year);
            if !years.is_empty() {
                return years.iter().any(|ry| (lo..=hi).contains(ry));
            }
        }
        let end = if y.use_initial_air_date && ctx.media_type == MediaType::Series {
            req_year
        } else {
            ctx.year_end.unwrap_or(req_year)
        };
        req_year <= hi && end >= lo
    }

    fn season_episode_ok(&self, s: &ParsedStream, ctx: &RequestContext) -> bool {
        let m = &self.cfg.season_episode_matching;
        let (Some(season), Some(episode)) = (ctx.season, ctx.episode) else {
            if m.enabled
                && ctx.media_type == MediaType::Movie
                && in_scope(&m.scope, ctx, s)
                && (!s.parsed.seasons.is_empty() || !s.parsed.episodes.is_empty())
            {
                return false;
            }
            return true;
        };
        if !m.enabled || !in_scope(&m.scope, ctx, s) {
            return true;
        }
        let p = &s.parsed;
        if ctx.media_type == MediaType::Movie {
            return p.seasons.is_empty() && p.episodes.is_empty();
        }
        if let Some(date) = &p.date {
            if !ctx.episode_air_dates.is_empty() {
                return ctx.episode_air_dates.contains(date);
            }
        }
        let mut seasons = p.seasons.clone();
        if m.strict {
            if seasons.is_empty() && p.episodes.is_empty() {
                return false;
            }
            if seasons.is_empty() {
                seasons = vec![1];
            }
        }
        if let Some(ts) = ctx.tvdb_season {
            if seasons.contains(&ts)
                && (p.episodes.is_empty()
                    || ctx.tvdb_episode.is_some_and(|te| p.episodes.contains(&te)))
            {
                return true;
            }
        }
        let abs = ctx.absolute_episode;
        let rel_abs = ctx.relative_absolute_episode;
        let has = |n: Option<u32>| n.is_some_and(|n| p.episodes.contains(&n));
        if !seasons.is_empty() && !seasons.contains(&season) {
            let first_is_one = seasons.first() == Some(&1);
            if !(first_is_one && (has(abs) || has(rel_abs))) {
                return false;
            }
        }
        if !p.episodes.is_empty() && !p.episodes.contains(&episode) {
            let no_or_first = seasons.is_empty() || seasons.contains(&1);
            let abs_hit = has(abs) && no_or_first;
            let rel_hit = has(rel_abs) && no_or_first;
            let anime_hit = ctx.is_anime
                && has(abs)
                && seasons.len() == 1
                && seasons.first() == Some(&season)
                && abs.is_some_and(|a| {
                    ctx.seasons
                        .iter()
                        .find(|si| si.season_number == season)
                        .is_some_and(|si| a > si.episode_count)
                });
            if !(abs_hit || rel_hit || anime_hit) {
                return false;
            }
        }
        true
    }

    fn episode_title_ok(&self, s: &ParsedStream, ctx: &RequestContext) -> bool {
        let m = &self.cfg.episode_title_matching;
        if !m.enabled || !in_scope(&m.scope, ctx, s) {
            return true;
        }
        let Some(title) = s.parsed.episode_title.as_deref() else {
            return true;
        };
        if ctx.episode_titles.is_empty() {
            return true;
        }
        let norm = normalise_title(title);
        let matched = ctx
            .episode_titles
            .iter()
            .any(|e| ratio(&norm, &normalise_title(&e.title)) / 100.0 >= m.similarity_threshold);
        if matched {
            return true;
        }
        let known_langs: Vec<String> = ctx
            .episode_titles
            .iter()
            .filter_map(|e| e.language.as_deref())
            .filter_map(|l| vocab::language_by_iso(l).map(|l| l.name.to_string()))
            .collect();
        !s.parsed.languages.is_empty() && !hits(&known_langs, &s.parsed.languages)
    }

    fn title_match(&self, s: &ParsedStream, ctx: &RequestContext) -> Option<Option<String>> {
        let t = &self.cfg.title_matching;
        if !t.enabled || ctx.titles.is_empty() || !in_scope(&t.scope, ctx, s) {
            return Some(None);
        }
        let (Some(parsed_title), Some(_)) = (s.parsed.title.as_deref(), s.filename.as_deref())
        else {
            return None;
        };
        let known: Vec<&crate::types::TitleEntry> = ctx.titles.iter().take(100).collect();
        let first = preprocess_title(parsed_title, &known);
        let stream_norm = normalise_title(&first);
        let mut best: Option<(f64, Option<String>)> = None;
        for entry in &known {
            let known_norm = normalise_title(&entry.title);
            let score = if t.similarity_threshold >= 1.0 {
                if known_norm == stream_norm {
                    100.0
                } else {
                    0.0
                }
            } else {
                match t.mode {
                    TitleMode::Exact => ratio(&stream_norm, &known_norm),
                    TitleMode::Contains => partial_ratio(&stream_norm, &known_norm),
                }
            };
            if best.as_ref().is_none_or(|(b, _)| score > *b) {
                best = Some((score, entry.language.clone()));
            }
        }
        let (score, lang) = best?;
        if score / 100.0 < t.similarity_threshold {
            return None;
        }
        if let (Some(a), Some(b)) = (s.parsed.country.as_deref(), ctx.country.as_deref()) {
            if !a.eq_ignore_ascii_case(b) {
                return None;
            }
        }
        if t.ambiguous_results == Ambiguous::Discard && !ctx.title_conflicts.is_empty() {
            let earliest = ctx
                .title_conflicts
                .iter()
                .filter_map(|c| c.year)
                .all(|cy| ctx.year.is_some_and(|y| y <= cy));
            let country = s.parsed.country.is_some();
            let year_close = s
                .parsed
                .year
                .as_deref()
                .and_then(|y| y.split('-').next()?.parse::<i32>().ok())
                .is_some_and(|sy| {
                    ctx.release_years
                        .iter()
                        .chain(ctx.year.iter())
                        .any(|ry| (sy - ry).abs() <= 1)
                });
            let ep_title = s.parsed.episode_title.as_deref().is_some_and(|et| {
                ctx.episode_titles
                    .iter()
                    .any(|e| normalise_title(&e.title) == normalise_title(et))
            });
            if !(earliest || country || year_close || ep_title) {
                return None;
            }
        }
        let inferred = (self.cfg.language_inference)
            .then(|| lang.as_deref().and_then(vocab::language_by_iso))
            .flatten()
            .map(|l| l.name.to_string())
            .filter(|name| {
                name != "English"
                    && !(ctx.is_anime && name == "Japanese")
                    && s.parsed
                        .languages
                        .iter()
                        .all(|l| !vocab::is_specific_language(l))
            });
        Some(inferred)
    }
}

fn preprocess_title(title: &str, known: &[&crate::types::TitleEntry]) -> String {
    let separators = [" / ", " | ", " aka ", " AKA "];
    let mut out = title.to_string();
    for sep in separators {
        let sep_in_known = known.iter().filter(|k| k.title.contains(sep)).count();
        if (sep_in_known as f64) <= known.len() as f64 * 0.2 {
            if let Some((head, _)) = out.split_once(sep) {
                out = head.to_string();
            }
        }
    }
    if let Some(open) = out.rfind(" (") {
        if out.ends_with(')') {
            out.truncate(open);
        }
    }
    out
}
