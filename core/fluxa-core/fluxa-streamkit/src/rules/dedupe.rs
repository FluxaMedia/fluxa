use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use super::config::{
    BucketMode, DedupConfig, DedupKey, LibraryBehaviour, MultiGroup, PipelineConfig, Tiebreaker,
};
use super::filter::{Reason, Removal};
use crate::types::{ParsedStream, StreamType};
use crate::vocab::{normalise_key, strip_repost};

struct Union {
    parent: Vec<usize>,
}

impl Union {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut cur = x;
        while self.parent[cur] != root {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        root
    }

    fn join(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.parent[rb.max(ra)] = rb.min(ra);
        }
    }
}

fn geometric_bucket(value: f64, rounding_percent: f64) -> i64 {
    let step = (1.0 + rounding_percent / 100.0).ln();
    if value <= 0.0 || step <= 0.0 {
        return 0;
    }
    (value.ln() / step).round() as i64
}

fn smart_value(s: &ParsedStream, attr: &str, rounding: f64) -> Option<String> {
    let p = &s.parsed;
    let list = |v: &[String]| -> Option<String> {
        (!v.is_empty()).then(|| {
            let mut sorted = v.to_vec();
            sorted.sort();
            sorted.join(",")
        })
    };
    match attr {
        "size" => s
            .size
            .map(|v| geometric_bucket(v as f64, rounding).to_string()),
        "bitrate" => s
            .bitrate
            .map(|v| geometric_bucket(v as f64, rounding).to_string()),
        "resolution" => p.resolution.clone(),
        "quality" => p.quality.clone(),
        "encode" => p.encode.clone(),
        "releaseGroup" => p.release_group.clone(),
        "network" => p.network.clone(),
        "visualTags" => list(&p.visual_tags),
        "audioTags" => list(&p.audio_tags),
        "audioChannels" => list(&p.audio_channels),
        "languages" => list(&p.languages),
        "editions" => list(&p.editions),
        _ => None,
    }
}

fn keys_for(s: &ParsedStream, cfg: &DedupConfig) -> Vec<String> {
    let mut keys = Vec::new();
    for key in &cfg.keys {
        match key {
            DedupKey::Filename => {
                if let Some(name) = s.filename.as_deref() {
                    let k = normalise_key(&strip_repost(name));
                    if !k.is_empty() {
                        keys.push(format!("f:{k}"));
                    }
                }
            }
            DedupKey::InfoHash => {
                if s.stream_type.is_usenet() {
                    if let Some(nzb) = &s.nzb_url {
                        keys.push(format!("n:{nzb}"));
                    }
                } else if let Some(t) = &s.torrent {
                    keys.push(format!("h:{}#{}", t.info_hash, t.file_idx.unwrap_or(0)));
                }
            }
            DedupKey::SmartDetect => {
                let parts: Vec<String> = cfg
                    .smart_detect_attributes
                    .iter()
                    .filter_map(|a| {
                        smart_value(s, a, cfg.rounding_percent).map(|v| format!("{a}={v}"))
                    })
                    .collect();
                if !parts.is_empty() {
                    keys.push(format!("s:{}", parts.join("|")));
                }
            }
        }
    }
    keys
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Bucket {
    Cached,
    Uncached,
    P2p,
    Http,
    Live,
    Youtube,
    External,
    Other,
}

fn bucket_of(s: &ParsedStream) -> Bucket {
    match s.stream_type {
        StreamType::Debrid | StreamType::Usenet | StreamType::StremioUsenet => {
            match s.service.as_ref() {
                Some(svc) if svc.cached => Bucket::Cached,
                Some(_) => Bucket::Uncached,
                None => Bucket::Other,
            }
        }
        StreamType::P2p => Bucket::P2p,
        StreamType::Http => Bucket::Http,
        StreamType::Live => Bucket::Live,
        StreamType::Youtube => Bucket::Youtube,
        StreamType::External => Bucket::External,
        _ => Bucket::Other,
    }
}

fn mode_for(cfg: &DedupConfig, bucket: Bucket) -> BucketMode {
    match bucket {
        Bucket::Cached => cfg.cached,
        Bucket::Uncached => cfg.uncached,
        Bucket::P2p => cfg.p2p,
        Bucket::Http => cfg.http,
        Bucket::Live => cfg.live,
        Bucket::Youtube => cfg.youtube,
        Bucket::External => cfg.external,
        Bucket::Other => BucketMode::Disabled,
    }
}

fn tiebreak(kind: Tiebreaker, a: &ParsedStream, b: &ParsedStream) -> Ordering {
    match kind {
        Tiebreaker::TorrentSeeders => {
            let applies = |s: &ParsedStream| {
                s.stream_type == StreamType::P2p || bucket_of(s) == Bucket::Uncached
            };
            match (applies(a), applies(b), a.seeders(), b.seeders()) {
                (true, true, Some(x), Some(y)) if x != y => y.cmp(&x),
                _ => Ordering::Equal,
            }
        }
        Tiebreaker::UsenetAge => match (
            a.stream_type.is_usenet(),
            b.stream_type.is_usenet(),
            a.age,
            b.age,
        ) {
            (true, true, Some(x), Some(y)) if (x - y).abs() > 24.0 => {
                x.partial_cmp(&y).unwrap_or(Ordering::Equal)
            }
            _ => Ordering::Equal,
        },
    }
}

fn index_of(list: &[String], value: &str) -> usize {
    list.iter().position(|v| v == value).unwrap_or(usize::MAX)
}

fn compare(cfg: &PipelineConfig, a: &ParsedStream, b: &ParsedStream) -> Ordering {
    let d = &cfg.deduplicator;
    if d.library_behaviour == LibraryBehaviour::Prefer {
        let ord = b.library.cmp(&a.library);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    let svc = |s: &ParsedStream| {
        s.service
            .as_ref()
            .map_or(usize::MAX, |x| index_of(&cfg.services, &x.id))
    };
    let ord = svc(a).cmp(&svc(b));
    if ord != Ordering::Equal {
        return ord;
    }
    for t in &d.tiebreakers_before_addon {
        let ord = tiebreak(*t, a, b);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    let ord = index_of(&cfg.addons, &a.addon.instance_id)
        .cmp(&index_of(&cfg.addons, &b.addon.instance_id));
    if ord != Ordering::Equal {
        return ord;
    }
    for t in &d.tiebreakers_after_addon {
        let ord = tiebreak(*t, a, b);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    index_of(&cfg.preferred_stream_types, a.stream_type.as_str()).cmp(&index_of(
        &cfg.preferred_stream_types,
        b.stream_type.as_str(),
    ))
}

fn best_of<'a>(cfg: &PipelineConfig, members: &[&'a ParsedStream]) -> Option<&'a ParsedStream> {
    members.iter().copied().min_by(|a, b| compare(cfg, a, b))
}

fn pick(
    cfg: &PipelineConfig,
    mode: BucketMode,
    members: &[usize],
    streams: &[ParsedStream],
    drop: &mut HashSet<usize>,
) {
    let mut candidates: Vec<usize> = members.to_vec();
    if cfg.deduplicator.library_behaviour == LibraryBehaviour::Exclusive
        && candidates.iter().any(|i| streams[*i].library)
    {
        for i in candidates.iter().filter(|i| !streams[**i].library) {
            drop.insert(*i);
        }
        candidates.retain(|i| streams[*i].library);
    }
    let group_key = |i: usize| -> String {
        match mode {
            BucketMode::PerService => streams[i]
                .service
                .as_ref()
                .map_or_else(|| "none".to_string(), |s| s.id.clone()),
            BucketMode::PerAddon => streams[i].addon.preset_id.clone(),
            _ => String::new(),
        }
    };
    if mode == BucketMode::Disabled {
        return;
    }
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for i in &candidates {
        groups.entry(group_key(*i)).or_default().push(*i);
    }
    for (_, ids) in groups {
        let refs: Vec<&ParsedStream> = ids.iter().map(|i| &streams[*i]).collect();
        let Some(winner) = best_of(cfg, &refs) else {
            continue;
        };
        for i in ids {
            if streams[i].id != winner.id {
                drop.insert(i);
            }
        }
    }
}

pub fn dedupe(
    cfg: &PipelineConfig,
    streams: Vec<ParsedStream>,
) -> (Vec<ParsedStream>, Vec<Removal>) {
    let d = &cfg.deduplicator;
    if !d.enabled {
        return (streams, Vec::new());
    }
    let mut uf = Union::new(streams.len());
    let mut owner: HashMap<String, usize> = HashMap::new();
    for (i, s) in streams.iter().enumerate() {
        for key in keys_for(s, d) {
            match owner.get(&key) {
                Some(&o) => uf.join(o, i),
                None => {
                    owner.insert(key, i);
                }
            }
        }
    }
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..streams.len() {
        let root = uf.find(i);
        groups.entry(root).or_default().push(i);
    }

    let protected = |s: &ParsedStream| {
        d.exclude_addons
            .iter()
            .any(|a| *a == s.addon.preset_id || *a == s.addon.instance_id)
            || s.passthrough.covers("dedup")
            || s.stream_type == StreamType::Info
    };

    let mut drop: HashSet<usize> = HashSet::new();
    for (_, members) in groups.iter().filter(|(_, m)| m.len() > 1) {
        let candidates: Vec<usize> = members
            .iter()
            .copied()
            .filter(|i| !protected(&streams[*i]))
            .collect();
        let mut buckets: HashMap<Bucket, Vec<usize>> = HashMap::new();
        for i in &candidates {
            buckets.entry(bucket_of(&streams[*i])).or_default().push(*i);
        }
        let present = [Bucket::Cached, Bucket::Uncached, Bucket::P2p]
            .iter()
            .filter(|b| buckets.contains_key(b))
            .count();
        if present >= 2 {
            let has = |b: Bucket| buckets.contains_key(&b);
            match d.multi_group_behaviour {
                MultiGroup::Aggressive => {
                    if has(Bucket::Cached) {
                        for b in [Bucket::Uncached, Bucket::P2p] {
                            drop.extend(buckets.remove(&b).unwrap_or_default());
                        }
                    } else {
                        drop.extend(buckets.remove(&Bucket::Uncached).unwrap_or_default());
                    }
                }
                MultiGroup::Conservative => {
                    if has(Bucket::Cached) {
                        drop.extend(buckets.remove(&Bucket::P2p).unwrap_or_default());
                        let cached_services: HashSet<String> = buckets[&Bucket::Cached]
                            .iter()
                            .filter_map(|i| streams[*i].service.as_ref().map(|s| s.id.clone()))
                            .collect();
                        if let Some(unc) = buckets.get_mut(&Bucket::Uncached) {
                            unc.retain(|i| {
                                let same = streams[*i]
                                    .service
                                    .as_ref()
                                    .is_some_and(|s| cached_services.contains(&s.id));
                                if same {
                                    drop.insert(*i);
                                }
                                !same
                            });
                        }
                    }
                }
                MultiGroup::KeepAll => {}
            }
        }
        for (bucket, ids) in buckets {
            pick(cfg, mode_for(d, bucket), &ids, &streams, &mut drop);
        }
    }

    let mut kept = Vec::with_capacity(streams.len());
    let mut removed = Vec::new();
    for (i, s) in streams.into_iter().enumerate() {
        if drop.contains(&i) {
            removed.push(Removal {
                stream_id: s.id,
                reason: Reason::Duplicate,
                detail: None,
            });
        } else {
            kept.push(s);
        }
    }
    (kept, removed)
}
