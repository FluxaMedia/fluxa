use std::collections::HashMap;

use serde_json::Value;

use crate::ids::IdSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    Given,
    StaticTable,
    External,
    NameSearch,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedIds {
    pub ids: IdSet,
    pub source: HashMap<&'static str, Provenance>,
}

#[derive(Clone, Debug, Default)]
struct Row {
    imdb: Option<String>,
    tmdb: Option<u64>,
    tvdb: Option<u64>,
    tvmaze: Option<u64>,
}

#[derive(Default)]
pub struct IdTable {
    rows: Vec<Row>,
    by_imdb: HashMap<String, usize>,
    by_tmdb: HashMap<u64, usize>,
    by_tvdb: HashMap<u64, usize>,
    by_tvmaze: HashMap<u64, usize>,
    pub skipped: usize,
}

fn valid_imdb(raw: &str) -> Option<String> {
    let digits = raw.trim().strip_prefix("tt")?;
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("tt{digits}"))
}

fn valid_num(raw: &str) -> Option<u64> {
    let n: u64 = raw.trim().parse().ok()?;
    (n > 0 && n <= u64::from(u32::MAX)).then_some(n)
}

fn column(header: &[String], names: &[&str]) -> Option<usize> {
    header
        .iter()
        .position(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
}

fn parse_csv(text: &str) -> (Vec<Row>, usize) {
    let mut lines = text.lines();
    let header: Vec<String> = lines
        .next()
        .unwrap_or("")
        .split(',')
        .map(|h| h.trim().to_string())
        .collect();
    let (imdb_c, tmdb_c, tvdb_c, maze_c) = (
        column(&header, &["imdb_id", "imdbId"]),
        column(&header, &["tmdb_id", "tmdbId"]),
        column(&header, &["tvdb_id", "tvdbId"]),
        column(&header, &["tvmaze_id", "tvmazeId"]),
    );
    let mut rows = Vec::new();
    let mut skipped = 0;
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let cells: Vec<&str> = line.split(',').collect();
        let cell = |c: Option<usize>| {
            c.and_then(|i| cells.get(i))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
        };
        let imdb = cell(imdb_c).map(valid_imdb);
        let tmdb = cell(tmdb_c).map(valid_num);
        let tvdb = cell(tvdb_c).map(valid_num);
        let tvmaze = cell(maze_c).map(|v| valid_num(v.split('/').next().unwrap_or(v)));
        let invalid = matches!(imdb, Some(None))
            || matches!(tmdb, Some(None))
            || matches!(tvdb, Some(None))
            || matches!(tvmaze, Some(None));
        if invalid {
            skipped += 1;
            continue;
        }
        rows.push(Row {
            imdb: imdb.flatten(),
            tmdb: tmdb.flatten(),
            tvdb: tvdb.flatten(),
            tvmaze: tvmaze.flatten(),
        });
    }
    (rows, skipped)
}

impl IdTable {
    pub fn build(
        primary: &str,
        wikidata: Option<&str>,
        fill_wikidata_tvdb: bool,
    ) -> Result<Self, String> {
        let (rows, mut skipped) = parse_csv(primary);
        if rows.is_empty() {
            return Err("no valid rows".into());
        }
        let mut table = IdTable::default();
        for row in rows {
            table.insert(row);
        }
        if let Some(text) = wikidata {
            let (wrows, wskipped) = parse_csv(text);
            skipped += wskipped;
            for w in wrows {
                table.merge_wikidata(w, fill_wikidata_tvdb);
            }
        }
        table.skipped = skipped;
        Ok(table)
    }

    fn insert(&mut self, mut row: Row) {
        let idx = self.rows.len();
        let mut owns_any = false;
        if let Some(i) = row.imdb.clone() {
            if self.by_imdb.contains_key(&i) {
                row.imdb = None;
            } else {
                self.by_imdb.insert(i, idx);
                owns_any = true;
            }
        }
        if let Some(i) = row.tmdb {
            if self.by_tmdb.contains_key(&i) {
                row.tmdb = None;
            } else {
                self.by_tmdb.insert(i, idx);
                owns_any = true;
            }
        }
        if let Some(i) = row.tvdb {
            if self.by_tvdb.contains_key(&i) {
                row.tvdb = None;
            } else {
                self.by_tvdb.insert(i, idx);
                owns_any = true;
            }
        }
        if let Some(i) = row.tvmaze {
            if self.by_tvmaze.contains_key(&i) {
                row.tvmaze = None;
            } else {
                self.by_tvmaze.insert(i, idx);
                owns_any = true;
            }
        }
        if owns_any {
            self.rows.push(row);
        }
    }

    fn merge_wikidata(&mut self, w: Row, fill_tvdb: bool) {
        let mut hits: Vec<usize> = Vec::new();
        let mut note = |i: Option<&usize>| {
            if let Some(i) = i {
                if !hits.contains(i) {
                    hits.push(*i);
                }
            }
        };
        note(w.imdb.as_ref().and_then(|v| self.by_imdb.get(v)));
        note(w.tmdb.as_ref().and_then(|v| self.by_tmdb.get(v)));
        note(w.tvdb.as_ref().and_then(|v| self.by_tvdb.get(v)));
        note(w.tvmaze.as_ref().and_then(|v| self.by_tvmaze.get(v)));
        match hits.as_slice() {
            [] => self.insert(w),
            [only] => {
                let Some(row) = self.rows.get(*only).cloned() else {
                    return;
                };
                let disagrees = |a: &Option<u64>, b: &Option<u64>| matches!((a, b), (Some(x), Some(y)) if x != y);
                if row.imdb.is_some() && w.imdb.is_some() && row.imdb != w.imdb
                    || disagrees(&row.tmdb, &w.tmdb)
                    || disagrees(&row.tvdb, &w.tvdb)
                    || disagrees(&row.tvmaze, &w.tvmaze)
                {
                    return;
                }
                if !fill_tvdb {
                    return;
                }
                let mut fills = Vec::new();
                if row.tvdb.is_none() {
                    if let Some(v) = w.tvdb.filter(|v| !self.by_tvdb.contains_key(v)) {
                        fills.push(("tvdb", v));
                    }
                }
                if row.tvmaze.is_none() {
                    if let Some(v) = w.tvmaze.filter(|v| !self.by_tvmaze.contains_key(v)) {
                        fills.push(("tvmaze", v));
                    }
                }
                for (kind, v) in fills {
                    if let Some(r) = self.rows.get_mut(*only) {
                        if kind == "tvdb" {
                            r.tvdb = Some(v);
                            self.by_tvdb.insert(v, *only);
                        } else {
                            r.tvmaze = Some(v);
                            self.by_tvmaze.insert(v, *only);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    fn lookup(&self, ids: &IdSet) -> Option<&Row> {
        let idx = ids
            .imdb
            .as_ref()
            .and_then(|v| self.by_imdb.get(v))
            .or_else(|| ids.tmdb.as_ref().and_then(|v| self.by_tmdb.get(v)))
            .or_else(|| ids.tvdb.as_ref().and_then(|v| self.by_tvdb.get(v)))
            .or_else(|| ids.tvmaze.as_ref().and_then(|v| self.by_tvmaze.get(v)))?;
        self.rows.get(*idx)
    }

    pub fn fill(&self, resolved: &mut ResolvedIds) {
        let Some(row) = self.lookup(&resolved.ids) else {
            return;
        };
        let row = row.clone();
        let ids = &mut resolved.ids;
        let mut mark = |name: &'static str| {
            resolved
                .source
                .entry(name)
                .or_insert(Provenance::StaticTable);
        };
        if ids.imdb.is_none() && row.imdb.is_some() {
            ids.imdb = row.imdb;
            mark("imdb");
        }
        if ids.tmdb.is_none() && row.tmdb.is_some() {
            ids.tmdb = row.tmdb;
            mark("tmdb");
        }
        if ids.tvdb.is_none() && row.tvdb.is_some() {
            ids.tvdb = row.tvdb;
            mark("tvdb");
        }
        if ids.tvmaze.is_none() && row.tvmaze.is_some() {
            ids.tvmaze = row.tvmaze;
            mark("tvmaze");
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimeIdRow {
    pub mal: Option<u64>,
    pub anilist: Option<u64>,
    pub kitsu: Option<u64>,
    pub anidb: Option<u64>,
    pub simkl: Option<u64>,
    pub tmdb: Option<u64>,
    pub tvdb: Option<u64>,
    pub imdb: Option<String>,
    pub kind: Option<String>,
}

fn first_number(v: Option<&Value>) -> Option<u64> {
    match v? {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.trim().parse().ok(),
        Value::Array(a) => a.iter().find_map(|x| first_number(Some(x))),
        _ => None,
    }
}

fn first_imdb(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) => valid_imdb(s),
        Value::Array(a) => a.iter().find_map(|x| first_imdb(Some(x))),
        _ => None,
    }
}

#[derive(Default)]
pub struct AnimeIdTable {
    rows: Vec<AnimeIdRow>,
    by_mal: HashMap<u64, usize>,
    by_anilist: HashMap<u64, usize>,
    by_kitsu: HashMap<u64, usize>,
    by_anidb: HashMap<u64, usize>,
    by_imdb: HashMap<String, usize>,
}

impl AnimeIdTable {
    pub fn from_fribb(json: &str) -> Result<Self, String> {
        let list: Vec<Value> = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let mut t = AnimeIdTable::default();
        for item in &list {
            let row = AnimeIdRow {
                mal: first_number(item.get("mal_id")),
                anilist: first_number(item.get("anilist_id")),
                kitsu: first_number(item.get("kitsu_id")),
                anidb: first_number(item.get("anidb_id")),
                simkl: first_number(item.get("simkl_id")),
                tmdb: first_number(item.get("themoviedb_id")),
                tvdb: first_number(item.get("thetvdb_id")),
                imdb: first_imdb(item.get("imdb_id")),
                kind: item.get("type").and_then(Value::as_str).map(str::to_string),
            };
            let idx = t.rows.len();
            if let Some(v) = row.mal {
                t.by_mal.entry(v).or_insert(idx);
            }
            if let Some(v) = row.anilist {
                t.by_anilist.entry(v).or_insert(idx);
            }
            if let Some(v) = row.kitsu {
                t.by_kitsu.entry(v).or_insert(idx);
            }
            if let Some(v) = row.anidb {
                t.by_anidb.entry(v).or_insert(idx);
            }
            if let Some(v) = row.imdb.clone() {
                t.by_imdb.entry(v).or_insert(idx);
            }
            t.rows.push(row);
        }
        Ok(t)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn fill(&self, resolved: &mut ResolvedIds) {
        let ids = &resolved.ids;
        let idx = ids
            .mal
            .and_then(|v| self.by_mal.get(&v))
            .or_else(|| ids.anilist.and_then(|v| self.by_anilist.get(&v)))
            .or_else(|| ids.kitsu.and_then(|v| self.by_kitsu.get(&v)))
            .or_else(|| ids.anidb.and_then(|v| self.by_anidb.get(&v)))
            .or_else(|| ids.imdb.as_ref().and_then(|v| self.by_imdb.get(v)));
        let Some(row) = idx.and_then(|i| self.rows.get(*i)).cloned() else {
            return;
        };
        let ids = &mut resolved.ids;
        let mut mark = |name: &'static str| {
            resolved
                .source
                .entry(name)
                .or_insert(Provenance::StaticTable);
        };
        macro_rules! fill {
            ($slot:ident, $name:literal) => {
                if ids.$slot.is_none() && row.$slot.is_some() {
                    ids.$slot = row.$slot.clone();
                    mark($name);
                }
            };
        }
        fill!(mal, "mal");
        fill!(anilist, "anilist");
        fill!(kitsu, "kitsu");
        fill!(anidb, "anidb");
        fill!(tmdb, "tmdb");
        fill!(tvdb, "tvdb");
        fill!(imdb, "imdb");
    }
}

#[derive(Default)]
pub struct Resolver {
    pub series: Option<IdTable>,
    pub movies: Option<IdTable>,
    pub anime: Option<AnimeIdTable>,
}

impl Resolver {
    pub fn resolve_all(&self, given: IdSet, is_movie: bool) -> ResolvedIds {
        let mut resolved = ResolvedIds {
            source: given_sources(&given),
            ids: given,
        };
        for _ in 0..3 {
            let before = resolved.ids.clone();
            if let Some(a) = &self.anime {
                a.fill(&mut resolved);
            }
            let table = if is_movie { &self.movies } else { &self.series };
            if let Some(t) = table {
                t.fill(&mut resolved);
            }
            if resolved.ids == before {
                break;
            }
        }
        resolved
    }
}

fn given_sources(ids: &IdSet) -> HashMap<&'static str, Provenance> {
    let mut m = HashMap::new();
    let mut put = |name: &'static str, present: bool| {
        if present {
            m.insert(name, Provenance::Given);
        }
    };
    put("imdb", ids.imdb.is_some());
    put("tmdb", ids.tmdb.is_some());
    put("tvdb", ids.tvdb.is_some());
    put("tvmaze", ids.tvmaze.is_some());
    put("mal", ids.mal.is_some());
    put("kitsu", ids.kitsu.is_some());
    put("anilist", ids.anilist.is_some());
    put("anidb", ids.anidb.is_some());
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIMARY: &str = "imdb_id,tmdb_id,tvdb_id,tvmaze_id\ntt0944947,1399,121361,82\ntt0903747,1396,81189,169/breaking-bad\nbogus,1,2,3\n";

    #[test]
    fn rows_with_invalid_ids_are_skipped_and_counted() {
        let t = IdTable::build(PRIMARY, None, true).unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(t.skipped, 1);
    }

    #[test]
    fn file_without_valid_rows_is_rejected() {
        assert!(IdTable::build("imdb_id,tmdb_id\nzzz,1\n", None, true).is_err());
    }

    #[test]
    fn tvmaze_keeps_the_part_before_a_slash() {
        let t = IdTable::build(PRIMARY, None, true).unwrap();
        let r = Resolver {
            series: Some(t),
            ..Default::default()
        };
        let out = r.resolve_all(IdSet::from_prefixed("tt0903747"), false);
        assert_eq!(out.ids.tvmaze, Some(169));
        assert_eq!(out.source.get("tvmaze"), Some(&Provenance::StaticTable));
        assert_eq!(out.source.get("imdb"), Some(&Provenance::Given));
    }

    #[test]
    fn wikidata_fills_only_empty_tvdb_and_tvmaze() {
        let primary = "imdb_id,tmdb_id,tvdb_id,tvmaze_id\ntt1,10,,\n";
        let wiki = "imdb_id,tmdb_id,tvdb_id,tvmaze_id\ntt1,10,55,66\n";
        let t = IdTable::build(primary, Some(wiki), true).unwrap();
        let r = Resolver {
            series: Some(t),
            ..Default::default()
        };
        let out = r.resolve_all(IdSet::from_prefixed("tmdb:10"), false);
        assert_eq!(out.ids.tvdb, Some(55));
        assert_eq!(out.ids.tvmaze, Some(66));
    }

    #[test]
    fn wikidata_never_fills_movie_tables() {
        let primary = "imdb_id,tmdb_id,tvdb_id\ntt1,10,\n";
        let wiki = "imdb_id,tmdb_id,tvdb_id\ntt1,10,55\n";
        let t = IdTable::build(primary, Some(wiki), false).unwrap();
        let r = Resolver {
            movies: Some(t),
            ..Default::default()
        };
        assert_eq!(
            r.resolve_all(IdSet::from_prefixed("tt1"), true).ids.tvdb,
            None
        );
    }

    #[test]
    fn conflicting_wikidata_rows_are_ignored() {
        let primary = "imdb_id,tmdb_id,tvdb_id\ntt1,10,5\n";
        let wiki = "imdb_id,tmdb_id,tvdb_id\ntt1,10,99\n";
        let t = IdTable::build(primary, Some(wiki), true).unwrap();
        let r = Resolver {
            series: Some(t),
            ..Default::default()
        };
        assert_eq!(
            r.resolve_all(IdSet::from_prefixed("tt1"), false).ids.tvdb,
            Some(5)
        );
    }

    #[test]
    fn fribb_lists_take_the_first_id() {
        let json = r#"[{"mal_id":[20,21],"kitsu_id":"1","anilist_id":20,"anidb_id":23,"themoviedb_id":31910,"imdb_id":"tt0988824","type":"TV"}]"#;
        let r = Resolver {
            anime: Some(AnimeIdTable::from_fribb(json).unwrap()),
            ..Default::default()
        };
        let out = r.resolve_all(IdSet::from_prefixed("kitsu:1"), false);
        assert_eq!(out.ids.mal, Some(20));
        assert_eq!(out.ids.imdb.as_deref(), Some("tt0988824"));
        assert_eq!(out.ids.tmdb, Some(31910));
    }
}
