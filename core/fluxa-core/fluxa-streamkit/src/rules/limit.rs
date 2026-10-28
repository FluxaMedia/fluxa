use std::collections::HashMap;

use super::config::{LimitConfig, LimitMode};
use super::filter::{Reason, Removal};
use crate::types::{ParsedStream, StreamType};
use crate::vocab::UNKNOWN;

struct Category {
    name: &'static str,
    cap: usize,
    value: fn(&ParsedStream) -> Option<String>,
}

fn categories(cfg: &LimitConfig) -> Vec<Category> {
    let mut out = Vec::new();
    if let Some(cap) = cfg.indexer {
        out.push(Category {
            name: "indexer",
            cap,
            value: |s| s.indexer.clone(),
        });
    }
    if let Some(cap) = cfg.release_group {
        out.push(Category {
            name: "releaseGroup",
            cap,
            value: |s| s.parsed.release_group.clone().or(Some(UNKNOWN.into())),
        });
    }
    if let Some(cap) = cfg.resolution {
        out.push(Category {
            name: "resolution",
            cap,
            value: |s| s.parsed.resolution.clone().or(Some(UNKNOWN.into())),
        });
    }
    if let Some(cap) = cfg.quality {
        out.push(Category {
            name: "quality",
            cap,
            value: |s| s.parsed.quality.clone().or(Some(UNKNOWN.into())),
        });
    }
    if let Some(cap) = cfg.addon {
        out.push(Category {
            name: "addon",
            cap,
            value: |s| Some(s.addon.instance_id.clone()),
        });
    }
    if let Some(cap) = cfg.stream_type {
        out.push(Category {
            name: "streamType",
            cap,
            value: |s| Some(s.stream_type.as_str().to_string()),
        });
    }
    if let Some(cap) = cfg.service {
        out.push(Category {
            name: "service",
            cap,
            value: |s| s.service.as_ref().map(|x| x.id.clone()),
        });
    }
    out
}

pub fn limit(cfg: &LimitConfig, streams: Vec<ParsedStream>) -> (Vec<ParsedStream>, Vec<Removal>) {
    let cats = categories(cfg);
    if cats.is_empty() && cfg.global.is_none() {
        return (streams, Vec::new());
    }
    let mut kept = Vec::new();
    let mut removed = Vec::new();
    let mut global = 0usize;
    let mut counts: HashMap<(&'static str, String), usize> = HashMap::new();
    let tuple_cap = cats.iter().map(|c| c.cap).min();
    let mut tuple_counts: HashMap<Vec<String>, usize> = HashMap::new();

    for s in streams {
        if s.passthrough.covers("limit") || s.stream_type == StreamType::Info {
            kept.push(s);
            continue;
        }
        let cut = |s: ParsedStream, detail: String, removed: &mut Vec<Removal>| {
            removed.push(Removal {
                stream_id: s.id,
                reason: Reason::Limit,
                detail: Some(detail),
            });
        };
        if cfg.global.is_some_and(|g| global >= g) {
            cut(s, "global".into(), &mut removed);
            continue;
        }
        match cfg.mode {
            LimitMode::Independent => {
                let values: Vec<(&Category, Option<String>)> =
                    cats.iter().map(|c| (c, (c.value)(&s))).collect();
                let over = values.iter().find(|(c, v)| match v {
                    Some(v) => counts.get(&(c.name, v.clone())).copied().unwrap_or(0) >= c.cap,
                    None => false,
                });
                if let Some((c, _)) = over {
                    let name = c.name;
                    cut(s, name.into(), &mut removed);
                    continue;
                }
                for (c, v) in values {
                    if let Some(v) = v {
                        *counts.entry((c.name, v)).or_default() += 1;
                    }
                }
            }
            LimitMode::Conjunctive => {
                let key: Vec<String> = cats
                    .iter()
                    .map(|c| (c.value)(&s).unwrap_or_default())
                    .collect();
                let seen = tuple_counts.get(&key).copied().unwrap_or(0);
                if tuple_cap.is_some_and(|cap| seen >= cap) {
                    cut(s, "tuple".into(), &mut removed);
                    continue;
                }
                *tuple_counts.entry(key).or_default() += 1;
            }
        }
        global += 1;
        kept.push(s);
    }
    (kept, removed)
}
