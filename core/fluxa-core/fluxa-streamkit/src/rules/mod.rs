pub mod config;
pub mod dedupe;
pub mod filter;
pub mod limit;
pub mod sort;

use std::collections::HashMap;

use regex::Regex;

pub use config::PipelineConfig;
pub use filter::{Reason, Removal};

use crate::types::{ParsedStream, RegexMatch, RequestContext, Seadex, SeadexMethod};
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

    pub fn run(&self, ctx: &RequestContext, parsed: Vec<ParsedStream>) -> PipelineOutput {
        let (survivors, mut removed) = self.filter(ctx, parsed);
        let (survivors, dup_removed) = dedupe::dedupe(&self.cfg, survivors);
        removed.extend(dup_removed);
        let mut survivors = survivors;
        self.precompute(&mut survivors);
        let sorted = sort::sort(&self.cfg, ctx, survivors);
        let (streams, limit_removed) = limit::limit(&self.cfg.limits, sorted);
        removed.extend(limit_removed);
        PipelineOutput { streams, removed }
    }

    fn filter(
        &self,
        ctx: &RequestContext,
        parsed: Vec<ParsedStream>,
    ) -> (Vec<ParsedStream>, Vec<Removal>) {
        let filter = Filter::new(&self.cfg.filters);
        let mut kept = Vec::with_capacity(parsed.len());
        let mut removed = Vec::new();
        for mut s in parsed {
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
        (kept, removed)
    }

    fn precompute(&self, streams: &mut [ParsedStream]) {
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
    }
}
