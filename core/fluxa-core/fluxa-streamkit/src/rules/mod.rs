pub mod config;
pub mod dedupe;
pub mod filter;
pub mod limit;
pub mod sort;

use std::collections::HashMap;

use regex::Regex;

pub use config::PipelineConfig;
pub use filter::{Reason, Removal};

use crate::expr::{Compiled, Evaluation, Scope};
use crate::types::{ExprMatch, ParsedStream, RegexMatch, RequestContext, Seadex, SeadexMethod};
use filter::{Filter, compile_keywords, compile_patterns, pattern_hit, searchable_fields};

pub struct PipelineOutput {
    pub streams: Vec<ParsedStream>,
    pub removed: Vec<Removal>,
}

impl PipelineOutput {
    pub fn removal_counts(&self) -> HashMap<Reason, usize> {
        let mut counts = HashMap::new();
        for r in &self.removed {
            *counts.entry(r.reason).or_insert(0) += 1;
        }
        counts
    }
}

pub struct Pipeline {
    cfg: PipelineConfig,
}

impl Pipeline {
    pub fn new(cfg: PipelineConfig) -> Self {
        Self { cfg }
    }

    pub fn config(&self) -> &PipelineConfig {
        &self.cfg
    }

    pub fn validate(&self) -> Vec<(String, crate::expr::CompileError)> {
        let e = &self.cfg.filters.expressions;
        e.included
            .iter()
            .chain(&e.excluded)
            .chain(&e.required)
            .chain(&e.preferred)
            .chain(&e.pinned)
            .chain(e.ranked.iter().map(|r| &r.expression))
            .filter_map(|src| Compiled::compile(src).err().map(|err| (src.clone(), err)))
            .collect()
    }

    pub fn run(&self, ctx: &RequestContext, parsed: Vec<ParsedStream>) -> PipelineOutput {
        let scope = Scope::from_request(ctx);
        let (survivors, mut removed) = self.filter(ctx, &scope, parsed);
        let (survivors, dup_removed) = dedupe::dedupe(&self.cfg, survivors);
        removed.extend(dup_removed);
        let mut survivors = survivors;
        self.precompute(&scope, &mut survivors);
        let sorted = sort::sort(&self.cfg, ctx, survivors);
        let (streams, limit_removed) = limit::limit(&self.cfg.limits, sorted);
        removed.extend(limit_removed);
        PipelineOutput { streams, removed }
    }

    fn filter(
        &self,
        ctx: &RequestContext,
        scope: &Scope,
        mut parsed: Vec<ParsedStream>,
    ) -> (Vec<ParsedStream>, Vec<Removal>) {
        let exprs = &self.cfg.filters.expressions;
        let mut included = vec![false; parsed.len()];
        for src in &exprs.included {
            let Some(done) = compile(src).and_then(|c| c.eval(&parsed, scope).ok()) else {
                continue;
            };
            for &i in done.selected() {
                included[i] = true;
            }
            apply_effects(&mut parsed, &done);
        }
        let filter = Filter::new(&self.cfg.filters);
        let mut kept = Vec::with_capacity(parsed.len());
        let mut removed = Vec::new();
        for (n, mut s) in parsed.into_iter().enumerate() {
            if included[n] {
                kept.push(s);
                continue;
            }
            match filter.decide(&s, ctx) {
                Ok(inferred) => {
                    if let Some(lang) = inferred {
                        s.parsed
                            .languages
                            .retain(|l| crate::vocab::is_specific_language(l));
                        s.parsed.languages.push(lang);
                    }
                    kept.push(s);
                }
                Err(reason) => removed.push(Removal {
                    stream_id: s.id,
                    reason,
                    detail: None,
                }),
            }
        }
        let kept = self.expression_gate(scope, kept, &mut removed);
        (kept, removed)
    }

    fn expression_gate(
        &self,
        scope: &Scope,
        mut streams: Vec<ParsedStream>,
        removed: &mut Vec<Removal>,
    ) -> Vec<ParsedStream> {
        let exprs = &self.cfg.filters.expressions;
        let mut drop: Vec<Option<Reason>> = vec![None; streams.len()];
        for src in &exprs.excluded {
            let Some(done) = compile(src).and_then(|c| c.eval(&streams, scope).ok()) else {
                continue;
            };
            for &i in done.selected() {
                drop[i].get_or_insert(Reason::ExcludedExpression);
            }
            apply_effects(&mut streams, &done);
        }
        let required: Vec<Compiled> = exprs.required.iter().filter_map(|s| compile(s)).collect();
        if !required.is_empty() {
            let mut hit = vec![false; streams.len()];
            for c in &required {
                if let Ok(done) = c.eval(&streams, scope) {
                    for &i in done.selected() {
                        hit[i] = true;
                    }
                    apply_effects(&mut streams, &done);
                }
            }
            for (i, h) in hit.iter().enumerate() {
                if !h {
                    drop[i].get_or_insert(Reason::RequiredExpression);
                }
            }
        }
        let mut kept = Vec::with_capacity(streams.len());
        for (s, reason) in streams.into_iter().zip(drop) {
            match reason {
                Some(reason) => removed.push(Removal {
                    stream_id: s.id,
                    reason,
                    detail: None,
                }),
                None => kept.push(s),
            }
        }
        kept
    }

    fn precompute(&self, scope: &Scope, streams: &mut [ParsedStream]) {
        let f = &self.cfg.filters;
        let preferred: Vec<(Regex, bool, Option<String>, String)> = f
            .regex
            .preferred
            .iter()
            .zip(compile_patterns(&f.regex.preferred))
            .map(|(rule, (re, neg))| (re, neg, rule.name.clone(), rule.pattern.clone()))
            .collect();
        let keywords = compile_keywords(&f.keywords.preferred);
        let ranked: Vec<(Regex, &config::RankedPattern)> = f
            .ranked_regex
            .iter()
            .filter_map(|r| {
                Regex::new(&format!("(?i){}", r.pattern))
                    .ok()
                    .map(|re| (re, r))
            })
            .collect();

        for s in streams.iter_mut() {
            let fields: Vec<String> = searchable_fields(s).into_iter().map(String::from).collect();
            let field_refs: Vec<&str> = fields.iter().map(String::as_str).collect();
            s.regex_matched = preferred
                .iter()
                .enumerate()
                .find(|(_, (re, neg, _, _))| pattern_hit(re, *neg, &field_refs))
                .map(|(index, (_, _, name, pattern))| RegexMatch {
                    name: name.clone(),
                    pattern: pattern.clone(),
                    index,
                });
            s.keyword_matched = keywords
                .as_ref()
                .is_some_and(|re| field_refs.iter().any(|f| re.is_match(f)));

            let scored: Vec<&str> = [s.filename.as_deref(), s.folder_name.as_deref()]
                .into_iter()
                .flatten()
                .collect();
            s.ranked_regexes_matched.clear();
            s.regex_score = 0.0;
            for (index, (re, rule)) in ranked.iter().enumerate() {
                if scored.iter().any(|t| re.is_match(t)) {
                    s.regex_score += rule.score;
                    s.ranked_regexes_matched.push(RegexMatch {
                        name: rule.name.clone(),
                        pattern: rule.pattern.clone(),
                        index,
                    });
                }
            }

            let by_hash = s.torrent.as_ref().is_some_and(|t| {
                self.cfg
                    .best_release_hashes
                    .iter()
                    .any(|h| h.eq_ignore_ascii_case(&t.info_hash))
            });
            let by_group = s.parsed.release_group.as_deref().is_some_and(|g| {
                self.cfg
                    .best_release_groups
                    .iter()
                    .any(|b| b.eq_ignore_ascii_case(g))
            });
            s.seadex = if by_hash {
                Some(Seadex {
                    is_best: true,
                    is_seadex: true,
                    method: SeadexMethod::Hash,
                })
            } else if by_group {
                Some(Seadex {
                    is_best: true,
                    is_seadex: true,
                    method: SeadexMethod::Group,
                })
            } else {
                None
            };
        }
        self.precompute_expressions(scope, streams);
    }

    fn precompute_expressions(&self, scope: &Scope, streams: &mut [ParsedStream]) {
        let exprs = &self.cfg.filters.expressions;
        for s in streams.iter_mut() {
            s.stream_expression_score = 0.0;
            s.ranked_stream_expressions_matched.clear();
            s.stream_expression_matched = None;
        }
        for (index, rule) in exprs.ranked.iter().enumerate().filter(|(_, r)| r.enabled) {
            let Some(c) = compile(&rule.expression) else {
                continue;
            };
            let Ok(done) = c.eval(streams, scope) else {
                continue;
            };
            for &i in done.selected() {
                streams[i].stream_expression_score += rule.score;
                if c.names().is_empty() {
                    streams[i]
                        .ranked_stream_expressions_matched
                        .push(ExprMatch { name: None, index });
                }
                for n in c.names() {
                    streams[i]
                        .ranked_stream_expressions_matched
                        .push(ExprMatch {
                            name: Some(n.clone()),
                            index,
                        });
                }
            }
            apply_effects(streams, &done);
        }
        for (index, src) in exprs.preferred.iter().enumerate() {
            let Some(c) = compile(src) else { continue };
            let Ok(done) = c.eval(streams, scope) else {
                continue;
            };
            for &i in done.selected() {
                streams[i]
                    .stream_expression_matched
                    .get_or_insert(ExprMatch {
                        name: c.names().first().cloned(),
                        index,
                    });
            }
            apply_effects(streams, &done);
        }
        for src in &exprs.pinned {
            let Some(c) = compile(src) else { continue };
            let Ok(done) = c.eval(streams, scope) else {
                continue;
            };
            apply_effects(streams, &done);
        }
    }
}

fn apply_effects(streams: &mut [ParsedStream], done: &Evaluation) {
    for &(i, pos) in &done.pinned {
        streams[i].addon.pin = Some(pos);
    }
    for (i, stages) in &done.passthrough {
        let p = &mut streams[*i].passthrough;
        if stages.is_empty() {
            p.all = true;
        } else {
            p.stages.extend(stages.iter().cloned());
        }
    }
}

fn compile(src: &str) -> Option<Compiled> {
    Compiled::compile(src).ok()
}
