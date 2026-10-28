use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefaultSeason {
    Number(u32),
    Absolute,
    Zero,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mapping {
    pub anidb_season: u32,
    pub tvdb_season: Option<u32>,
    pub start: Option<i32>,
    pub end: Option<i32>,
    pub offset: Option<i32>,
    pub pairs: Vec<(i32, i32)>,
}

#[derive(Clone, Debug)]
pub struct AnimeEntry {
    pub anidb: u64,
    pub tvdb: Option<u64>,
    pub default_season: DefaultSeason,
    pub episode_offset: i32,
    pub tmdb_tv: Option<u64>,
    pub tmdb_movie: Option<u64>,
    pub imdb: Vec<String>,
    pub name: String,
    pub mappings: Vec<Mapping>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnidbHit {
    pub anidb: u64,
    pub season: u32,
    pub episode: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TvdbHit {
    pub season: u32,
    pub episode: u32,
}

pub fn episode_pairs(text: &str) -> Vec<(i32, i32)> {
    text.split(';')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .filter_map(|p| {
            let (a, b) = p.split_once('-')?;
            Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
        })
        .collect()
}

#[derive(Default)]
pub struct AnimeLists {
    entries: Vec<AnimeEntry>,
    by_anidb: HashMap<u64, usize>,
    by_tvdb: HashMap<u64, Vec<usize>>,
    by_tmdb_tv: HashMap<u64, Vec<usize>>,
    by_imdb: HashMap<String, Vec<usize>>,
}

fn parse_default_season(raw: Option<&str>) -> DefaultSeason {
    match raw.map(str::trim) {
        Some("a") => DefaultSeason::Absolute,
        Some("0") => DefaultSeason::Zero,
        Some(n) => n
            .parse()
            .map(DefaultSeason::Number)
            .unwrap_or(DefaultSeason::Number(1)),
        None => DefaultSeason::Number(1),
    }
}

impl AnimeLists {
    pub fn from_entries(entries: Vec<AnimeEntry>) -> Self {
        let mut out = AnimeLists::default();
        for (i, e) in entries.iter().enumerate() {
            out.by_anidb.entry(e.anidb).or_insert(i);
            if let Some(t) = e.tvdb {
                out.by_tvdb.entry(t).or_default().push(i);
            }
            if let Some(t) = e.tmdb_tv {
                out.by_tmdb_tv.entry(t).or_default().push(i);
            }
            for imdb in &e.imdb {
                out.by_imdb.entry(imdb.clone()).or_default().push(i);
            }
        }
        out.entries = entries;
        out
    }

    pub fn parse_xml(xml: &str) -> Result<Self, String> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| e.to_string())?;
        let mut entries = Vec::new();
        for node in doc.descendants().filter(|n| n.has_tag_name("anime")) {
            let Some(anidb) = node.attribute("anidbid").and_then(|v| v.parse().ok()) else {
                continue;
            };
            let mappings = node
                .children()
                .filter(|c| c.has_tag_name("mapping-list"))
                .flat_map(|l| l.children().filter(|c| c.has_tag_name("mapping")))
                .map(|m| {
                    let num = |k: &str| m.attribute(k).and_then(|v| v.parse::<i32>().ok());
                    Mapping {
                        anidb_season: num("anidbseason").map_or(1, |v| v as u32),
                        tvdb_season: num("tvdbseason").map(|v| v as u32),
                        start: num("start"),
                        end: num("end"),
                        offset: num("offset"),
                        pairs: episode_pairs(m.text().unwrap_or("")),
                    }
                })
                .collect();
            entries.push(AnimeEntry {
                anidb,
                tvdb: node.attribute("tvdbid").and_then(|v| v.parse().ok()),
                default_season: parse_default_season(node.attribute("defaulttvdbseason")),
                episode_offset: node
                    .attribute("episodeoffset")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                tmdb_tv: node.attribute("tmdbtv").and_then(|v| v.parse().ok()),
                tmdb_movie: node.attribute("tmdbid").and_then(|v| v.parse().ok()),
                imdb: node
                    .attribute("imdbid")
                    .map(|v| {
                        v.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default(),
                name: node
                    .children()
                    .find(|c| c.has_tag_name("name"))
                    .and_then(|c| c.text())
                    .unwrap_or("")
                    .to_string(),
                mappings,
            });
        }
        Ok(Self::from_entries(entries))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn by_anidb(&self, id: u64) -> Option<&AnimeEntry> {
        self.by_anidb.get(&id).and_then(|i| self.entries.get(*i))
    }

    pub fn by_tvdb(&self, id: u64) -> Vec<&AnimeEntry> {
        self.collect(self.by_tvdb.get(&id))
    }

    pub fn by_tmdb_tv(&self, id: u64) -> Vec<&AnimeEntry> {
        self.collect(self.by_tmdb_tv.get(&id))
    }

    pub fn by_imdb(&self, id: &str) -> Vec<&AnimeEntry> {
        self.collect(self.by_imdb.get(id))
    }

    fn collect(&self, idx: Option<&Vec<usize>>) -> Vec<&AnimeEntry> {
        idx.into_iter()
            .flatten()
            .filter_map(|i| self.entries.get(*i))
            .collect()
    }

    fn same_season_group(&self, entry: &AnimeEntry) -> Vec<&AnimeEntry> {
        let Some(tvdb) = entry.tvdb else {
            return Vec::new();
        };
        let mut group: Vec<&AnimeEntry> = self
            .by_tvdb(tvdb)
            .into_iter()
            .filter(|e| e.default_season == entry.default_season)
            .collect();
        group.sort_by_key(|e| e.episode_offset);
        group
    }

    pub fn tvdb_to_anidb(&self, tvdb: u64, season: u32, episode: u32) -> Option<AnidbHit> {
        for entry in self.by_tvdb(tvdb) {
            if let Some(hit) = self.forward(entry, season, episode) {
                return Some(hit);
            }
        }
        None
    }

    fn forward(&self, entry: &AnimeEntry, season: u32, episode: u32) -> Option<AnidbHit> {
        let ep = episode as i32;
        let hit = |anidb_season: u32, e: i32| {
            (e > 0).then_some(AnidbHit {
                anidb: entry.anidb,
                season: anidb_season,
                episode: e as u32,
            })
        };
        match entry.default_season {
            DefaultSeason::Zero => None,
            DefaultSeason::Absolute => entry
                .mappings
                .iter()
                .filter(|m| m.tvdb_season == Some(season))
                .find_map(|m| {
                    let ae = ep - m.offset.unwrap_or(0);
                    let inside = match (m.start, m.end) {
                        (Some(s), Some(e)) => s <= ae && ae <= e,
                        (Some(s), None) => ae >= s,
                        _ => false,
                    };
                    inside.then(|| hit(m.anidb_season, ae)).flatten()
                }),
            DefaultSeason::Number(n) => {
                if n != season {
                    return None;
                }
                let group = self.same_season_group(entry);
                if group.len() <= 1 {
                    return hit(1, ep - entry.episode_offset);
                }
                let offset = entry.episode_offset;
                if let Some(m) = entry.mappings.iter().find(|m| {
                    matches!((m.start, m.end), (Some(s), Some(e)) if s + offset <= ep && ep <= e + offset)
                }) {
                    return hit(m.anidb_season, ep - offset);
                }
                let pos = group.iter().position(|e| e.anidb == entry.anidb)?;
                if let Some(next) = group.get(pos + 1) {
                    if ep > next.episode_offset {
                        return None;
                    }
                }
                hit(1, ep - offset)
            }
        }
    }

    pub fn anidb_to_tvdb(&self, anidb: u64, season: u32, episode: u32) -> Option<TvdbHit> {
        let entry = self.by_anidb(anidb)?;
        entry.tvdb?;
        let ep = episode as i32;
        let out = |s: u32, e: i32| {
            (e > 0).then_some(TvdbHit {
                season: s,
                episode: e as u32,
            })
        };
        match entry.default_season {
            DefaultSeason::Zero => None,
            DefaultSeason::Absolute => entry
                .mappings
                .iter()
                .filter(|m| m.anidb_season == season)
                .find_map(|m| {
                    let tvdb_season = m.tvdb_season?;
                    let offset = m.offset.unwrap_or(0);
                    let inside = match (m.start, m.end) {
                        (Some(s), Some(e)) => s <= ep && ep <= e,
                        (Some(s), None) => ep >= s,
                        _ => m.pairs.iter().any(|(a, b)| *a <= ep && ep <= *b),
                    };
                    inside.then(|| out(tvdb_season, ep + offset)).flatten()
                }),
            DefaultSeason::Number(n) => {
                let group = self.same_season_group(entry);
                if group.len() <= 1 {
                    return out(n, ep + entry.episode_offset);
                }
                if let Some(m) = entry.mappings.iter().find(|m| {
                    m.anidb_season == season
                        && matches!((m.start, m.end), (Some(s), Some(e)) if s <= ep && ep <= e)
                }) {
                    let start = m.start.unwrap_or(0);
                    let offset = m.offset.unwrap_or(entry.episode_offset);
                    return out(m.tvdb_season.unwrap_or(n), start + offset + (ep - start));
                }
                if ep < 1 {
                    return None;
                }
                let pos = group.iter().position(|e| e.anidb == entry.anidb)?;
                if let Some(next) = group.get(pos + 1) {
                    if ep >= next.episode_offset {
                        return None;
                    }
                }
                out(n, ep + entry.episode_offset)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(anidb: u64, season: DefaultSeason, offset: i32) -> AnimeEntry {
        AnimeEntry {
            anidb,
            tvdb: Some(100),
            default_season: season,
            episode_offset: offset,
            tmdb_tv: None,
            tmdb_movie: None,
            imdb: Vec::new(),
            name: String::new(),
            mappings: Vec::new(),
        }
    }

    fn hit(lists: &AnimeLists, s: u32, e: u32) -> Option<(u64, u32, u32)> {
        lists
            .tvdb_to_anidb(100, s, e)
            .map(|h| (h.anidb, h.season, h.episode))
    }

    #[test]
    fn single_entry_without_offset_maps_one_to_one() {
        let l = AnimeLists::from_entries(vec![entry(1, DefaultSeason::Number(2), 0)]);
        assert_eq!(hit(&l, 2, 5), Some((1, 1, 5)));
    }

    #[test]
    fn single_entry_offset_swallows_early_episodes() {
        let l = AnimeLists::from_entries(vec![entry(1, DefaultSeason::Number(2), 12)]);
        assert_eq!(hit(&l, 2, 5), None);
        assert_eq!(hit(&l, 2, 20), Some((1, 1, 8)));
    }

    #[test]
    fn split_cour_entries_cover_ranges_by_offset() {
        let l = AnimeLists::from_entries(vec![
            entry(1, DefaultSeason::Number(1), 0),
            entry(2, DefaultSeason::Number(1), 12),
        ]);
        assert_eq!(hit(&l, 1, 8), Some((1, 1, 8)));
        assert_eq!(hit(&l, 1, 20), Some((2, 1, 8)));
    }

    #[test]
    fn absolute_numbering_uses_the_mapping_range() {
        let mut e = entry(1, DefaultSeason::Absolute, 0);
        e.mappings.push(Mapping {
            anidb_season: 1,
            tvdb_season: Some(1),
            start: Some(1),
            end: Some(61),
            offset: Some(0),
            pairs: Vec::new(),
        });
        let l = AnimeLists::from_entries(vec![e]);
        assert_eq!(hit(&l, 1, 30), Some((1, 1, 30)));
        assert_eq!(hit(&l, 1, 70), None);
    }

    #[test]
    fn season_zero_default_is_skipped() {
        let l = AnimeLists::from_entries(vec![entry(1, DefaultSeason::Zero, 0)]);
        assert_eq!(hit(&l, 1, 1), None);
        assert_eq!(l.anidb_to_tvdb(1, 1, 1), None);
    }

    #[test]
    fn reverse_adds_the_offset() {
        let l = AnimeLists::from_entries(vec![entry(1, DefaultSeason::Number(2), 12)]);
        assert_eq!(
            l.anidb_to_tvdb(1, 1, 8),
            Some(TvdbHit {
                season: 2,
                episode: 20
            })
        );
    }

    #[test]
    fn xml_entries_are_indexed_and_parsed() {
        let xml = r#"<anime-list>
            <anime anidbid="9" tvdbid="77" defaulttvdbseason="a" episodeoffset="" imdbid="tt1,tt2">
                <name>Test</name>
                <mapping-list><mapping anidbseason="1" tvdbseason="1" start="1" end="10" offset="0">1-1;2-2</mapping></mapping-list>
            </anime>
            <anime anidbid="10" tvdbid="unknown" defaulttvdbseason="1"/>
        </anime-list>"#;
        let l = AnimeLists::parse_xml(xml).unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(l.by_imdb("tt2").len(), 1);
        assert_eq!(l.by_tvdb(77).first().map(|e| e.mappings.len()), Some(1));
        assert_eq!(l.by_anidb(10).and_then(|e| e.tvdb), None);
    }

    #[test]
    fn pair_text_drops_non_numeric_parts() {
        assert_eq!(episode_pairs("1-1; ;x-2;3-4"), vec![(1, 1), (3, 4)]);
    }
}
