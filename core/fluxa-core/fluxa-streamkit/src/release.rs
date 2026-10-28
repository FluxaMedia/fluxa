#![allow(clippy::expect_used)]

use std::sync::LazyLock;

use regex::Regex;

use crate::vocab::{
    self, AUDIO_TAGS, CHANNELS, DUAL_AUDIO, ENCODES, LANGUAGES, MULTI, QUALITIES, RESOLUTIONS,
    VISUAL_TAGS, rx,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParsedRelease {
    pub title: Option<String>,
    pub year: Option<String>,
    pub date: Option<String>,
    pub country: Option<String>,
    pub resolution: Option<String>,
    pub quality: Option<String>,
    pub codec: Option<String>,
    pub bit_depth: Option<String>,
    pub hdr: Vec<String>,
    pub audio: Vec<String>,
    pub channels: Vec<String>,
    pub seasons: Vec<u32>,
    pub episodes: Vec<u32>,
    pub volumes: Vec<u32>,
    pub episode_title: Option<String>,
    pub episode_code: Option<String>,
    pub complete: bool,
    pub languages: Vec<String>,
    pub dubbed: bool,
    pub subbed: bool,
    pub hardcoded: bool,
    pub group: Option<String>,
    pub site: Option<String>,
    pub network: Option<String>,
    pub editions: Vec<String>,
    pub repack: bool,
    pub proper: bool,
    pub retail: bool,
    pub regraded: bool,
    pub unrated: bool,
    pub uncensored: bool,
    pub upscaled: bool,
    pub container: Option<String>,
    pub extension: Option<String>,
    pub region: Option<String>,
}

fn bounded(pat: &str) -> Regex {
    rx(&format!(
        r"(?i)(?:^|[\s\[(\-,])(?P<m>{pat})(?:$|[\s\[()\]\-,])"
    ))
}

static YEAR: LazyLock<Regex> = LazyLock::new(|| bounded(r"(?:19|20)\d{2}(?:-(?:19|20)\d{2})?"));
static DATE: LazyLock<Regex> = LazyLock::new(|| {
    bounded(r"(?:19|20)\d{2}[ .\-/](?:0[1-9]|1[0-2])[ .\-/](?:0[1-9]|[12]\d|3[01])")
});
static SXE: LazyLock<Regex> = LazyLock::new(|| {
    rx(
        r"(?i)(?:^|[\s\[(\-,])(?P<m>s(?P<s>\d{1,2})[ ]?e(?P<e>\d{1,4})(?:-e?(?P<e2>\d{1,4})|e(?P<e3>\d{1,4}))?)\b",
    )
});
static NXN: LazyLock<Regex> =
    LazyLock::new(|| bounded(r"(?P<s>\d{1,2})x(?P<e>\d{2,3})(?:-(?P<e2>\d{2,3}))?"));
static SEASON_RANGE: LazyLock<Regex> =
    LazyLock::new(|| bounded(r"(?:seasons?[ ]?|s)(?P<a>\d{1,2})[ ]?-[ ]?s?(?P<b>\d{1,2})"));
static SEASON_LIST: LazyLock<Regex> =
    LazyLock::new(|| bounded(r"seasons?[ ]?\(?(?P<list>\d{1,2}(?:[ ]?[,&+][ ]?\d{1,2})+)\)?"));
static SEASON_ONE: LazyLock<Regex> = LazyLock::new(|| bounded(r"(?:seasons?[ ]?|s)(?P<s>\d{1,2})"));
static EPISODE_WORD: LazyLock<Regex> = LazyLock::new(|| {
    bounded(r"(?:episodes?|ep|cap)\.?[ ]?(?P<a>\d{1,4})(?:[ ]?-[ ]?(?P<b>\d{1,4}))?")
});
static ANIME_EP: LazyLock<Regex> =
    LazyLock::new(|| rx(r"(?i)\s-\s(?P<e>\d{1,4})(?:v\d)?(?:$|[\s\[(])"));
static COMPLETE: LazyLock<Regex> = LazyLock::new(|| {
    bounded(
        r"(?:the[ ])?(?:complete(?:[ ](?:series|season|collection|set))?|full[ ]series|all[ ]seasons|collection)",
    )
});
static LEADING_GROUP: LazyLock<Regex> = LazyLock::new(|| rx(r"^\[(?P<g>[^\]]+)\]\s*"));
static EPISODE_CODE: LazyLock<Regex> = LazyLock::new(|| rx(r"\[(?P<c>[0-9A-Fa-f]{8})\]"));
static TRAILING_GROUP: LazyLock<Regex> =
    LazyLock::new(|| rx(r"-(?P<g>[A-Za-z0-9]{2,20})(?:\s*\[[^\]]*\])*$"));
static SUB_LANG: LazyLock<Regex> = LazyLock::new(|| bounded(r"(?P<l>[a-z]{3})sub(?:s|bed)?"));
static SUBBED: LazyLock<Regex> =
    LazyLock::new(|| bounded(r"(?:sub(?:s|bed|titles?)?|multi[ .-]?subs?|soft[ ]sub|hardsub)"));
static HARDCODED: LazyLock<Regex> = LazyLock::new(|| bounded(r"hc|hardcoded|hardsub"));
static REPACK: LazyLock<Regex> = LazyLock::new(|| bounded(r"repack|rerip"));
static PROPER: LazyLock<Regex> = LazyLock::new(|| bounded(r"proper"));
static RETAIL: LazyLock<Regex> = LazyLock::new(|| bounded(r"retail"));
static UNRATED: LazyLock<Regex> = LazyLock::new(|| bounded(r"unrated"));
static UNCENSORED: LazyLock<Regex> = LazyLock::new(|| bounded(r"uncensored"));
static REGRADED: LazyLock<Regex> = LazyLock::new(|| bounded(r"regraded"));
static UPSCALED: LazyLock<Regex> = LazyLock::new(|| bounded(r"upscaled?|ai[ .-]?enhanced"));
static REGION: LazyLock<Regex> = LazyLock::new(|| bounded(r"r[0-9]|region[ ]?[0-9]"));
static COUNTRY: LazyLock<Regex> = LazyLock::new(|| bounded(r"(?:us|uk|au|nz|ca)"));
static NETWORK: LazyLock<Regex> = LazyLock::new(|| {
    bounded(
        r"netflix|nf|amzn|amazon|disney\+?|dsnp|hulu|hbo|hmax|atvp|appletv\+?|pcok|peacock|pmtp|paramount\+?",
    )
});
static SITE: LazyLock<Regex> = LazyLock::new(|| {
    rx(
        r"(?i)(?:^|[\s\[(])(?P<m>(?:www\.)?[a-z0-9\-]+\.(?:com|org|net|to|me|io|cc|tv))(?:$|[\s\])])",
    )
});
static BIT_DEPTH: LazyLock<Regex> = LazyLock::new(|| bounded(r"(?P<d>8|10|12)[ .-]?bit|hi10p?"));
static STRONG_LANG: LazyLock<Regex> = LazyLock::new(|| {
    bounded(r"(?:audio[ ])?latino|castellano|vostfr|truefrench|(?:[a-z]{3})sub(?:s|bed)?")
});
static EDITIONS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    vec![
        ("Remastered", bounded(r"remastered|remaster")),
        ("Extended", bounded(r"extended(?:[ ](?:cut|edition))?")),
        ("Uncut", bounded(r"uncut")),
        ("IMAX", bounded(r"imax")),
        ("Director's Cut", bounded(r"director'?s[ ]cut|dc")),
        ("Theatrical", bounded(r"theatrical(?:[ ]cut)?")),
        (
            "Anniversary Edition",
            bounded(r"anniversary(?:[ ]edition)?"),
        ),
        ("Criterion", bounded(r"criterion(?:[ ]collection)?")),
    ]
});

fn normalise(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '_' => out.push(' '),
            '.' => {
                let digit_at = |j: Option<usize>| {
                    j.and_then(|j| chars.get(j))
                        .is_some_and(char::is_ascii_digit)
                };
                let single = digit_at(i.checked_sub(1))
                    && digit_at(Some(i + 1))
                    && !digit_at(i.checked_sub(2))
                    && !digit_at(Some(i + 2));
                out.push(if single { '.' } else { ' ' });
            }
            c => out.push(c),
        }
    }
    let mut collapsed = String::with_capacity(out.len());
    let mut last_space = false;
    for c in out.chars() {
        if c.is_whitespace() {
            if !last_space {
                collapsed.push(' ');
            }
            last_space = true;
        } else {
            collapsed.push(c);
            last_space = false;
        }
    }
    collapsed.trim().to_string()
}

fn expand(a: u32, b: u32) -> Vec<u32> {
    if a <= b && b - a < 200 {
        (a..=b).collect()
    } else {
        vec![a, b]
    }
}

fn push_unique(list: &mut Vec<String>, value: &str) {
    if !list.iter().any(|v| v == value) {
        list.push(value.to_string());
    }
}

fn canonical_resolution(value: &str) -> String {
    match value {
        "4k" => "2160p".to_string(),
        other => other.to_string(),
    }
}

fn clean_title(raw: &str) -> Option<String> {
    let mut title = raw.trim().to_string();
    title = title
        .trim_matches(|c: char| " -:,([{|~".contains(c) || c == '.')
        .to_string();
    if let Some(open) = title.rfind(" (") {
        if !title[open..].contains(')') {
            title.truncate(open);
        }
    }
    let title = title
        .trim_end_matches(|c: char| " -:,([{|~.".contains(c))
        .trim()
        .to_string();
    (!title.is_empty()).then_some(title)
}

struct Marks<'a> {
    text: &'a str,
    start: usize,
    positions: Vec<usize>,
}

impl Marks<'_> {
    fn add(&mut self, at: usize) {
        if at <= self.start {
            return;
        }
        let before = self.text.get(..at).and_then(|t| t.chars().next_back());
        let at = if matches!(before, Some('[') | Some('(')) {
            at - 1
        } else {
            at
        };
        if at > self.start {
            self.positions.push(at);
        }
    }

    fn end(&self) -> usize {
        self.positions
            .iter()
            .copied()
            .min()
            .unwrap_or(self.text.len())
    }
}

pub fn parse(input: &str) -> ParsedRelease {
    let mut out = ParsedRelease::default();
    let trimmed = input.trim();
    let (stem, ext) = vocab::split_video_extension(trimmed);
    if let Some(ext) = ext {
        out.extension = Some(ext.to_lowercase());
        out.container = Some(ext.to_lowercase());
    }
    let mut w = normalise(stem);

    let mut start = 0usize;
    if let Some(c) = LEADING_GROUP.captures(&w) {
        let end = c.get(0).map_or(0, |m| m.end());
        let group = c.name("g").map(|m| m.as_str().to_string());
        let looks_like_tag = group
            .as_deref()
            .is_some_and(|g| g.chars().any(|ch| ch.is_alphabetic()) && !is_metadata_only(g));
        if looks_like_tag {
            out.group = group;
            start = end;
        }
    }

    if let Some(c) = EPISODE_CODE.captures(&w) {
        out.episode_code = c.name("c").map(|m| m.as_str().to_uppercase());
        if let Some(m) = c.get(0) {
            let (a, b) = (m.start(), m.end());
            w.replace_range(a..b, "");
            w = w.trim().to_string();
        }
    }

    if out.group.is_none() {
        if let Some(c) = TRAILING_GROUP.captures(&w) {
            let g = c.name("g").map_or("", |m| m.as_str());
            if !is_token(g) {
                out.group = Some(g.to_string());
                if let Some(m) = c.get(0) {
                    let cut = m.start();
                    w.truncate(cut);
                }
            }
        }
    }

    let mut marks = Marks {
        text: &w,
        start,
        positions: Vec::new(),
    };

    if let Some(c) = DATE.captures(&w) {
        if let Some(m) = c.name("m") {
            let digits: Vec<&str> = m.as_str().split(|ch: char| !ch.is_ascii_digit()).collect();
            if let [y, mo, d] = digits.as_slice() {
                out.date = Some(format!("{y}-{mo}-{d}"));
            }
            marks.add(m.start());
        }
    }

    let years: Vec<(usize, String)> = YEAR
        .captures_iter(&w)
        .filter_map(|c| c.name("m").map(|m| (m.start(), m.as_str().to_string())))
        .collect();

    if let Some(c) = SXE.captures(&w) {
        let s: u32 = c["s"].parse().unwrap_or(0);
        out.seasons = vec![s];
        let e: u32 = c["e"].parse().unwrap_or(0);
        let last = c
            .name("e2")
            .or_else(|| c.name("e3"))
            .and_then(|m| m.as_str().parse().ok());
        out.episodes = match last {
            Some(b) if c.name("e2").is_some() => expand(e, b),
            Some(b) => vec![e, b],
            None => vec![e],
        };
        if let Some(m) = c.name("m") {
            marks.add(m.start());
        }
    } else if let Some(c) = NXN.captures(&w) {
        out.seasons = vec![c["s"].parse().unwrap_or(0)];
        let e: u32 = c["e"].parse().unwrap_or(0);
        out.episodes = match c.name("e2").and_then(|m| m.as_str().parse().ok()) {
            Some(b) => expand(e, b),
            None => vec![e],
        };
        if let Some(m) = c.name("m") {
            marks.add(m.start());
        }
    } else {
        if let Some(c) = SEASON_RANGE.captures(&w) {
            let a: u32 = c["a"].parse().unwrap_or(0);
            let b: u32 = c["b"].parse().unwrap_or(0);
            out.seasons = expand(a, b);
            if let Some(m) = c.name("m") {
                marks.add(m.start());
            }
        } else if let Some(c) = SEASON_LIST.captures(&w) {
            out.seasons = c["list"]
                .split(|ch: char| !ch.is_ascii_digit())
                .filter_map(|n| n.parse().ok())
                .collect();
            if let Some(m) = c.name("m") {
                marks.add(m.start());
            }
        } else if let Some(c) = SEASON_ONE.captures(&w) {
            out.seasons = vec![c["s"].parse().unwrap_or(0)];
            if let Some(m) = c.name("m") {
                marks.add(m.start());
            }
        }
        if let Some(c) = EPISODE_WORD.captures(&w) {
            let a: u32 = c["a"].parse().unwrap_or(0);
            out.episodes = match c.name("b").and_then(|m| m.as_str().parse().ok()) {
                Some(b) if !c["m"].to_lowercase().starts_with("cap") => expand(a, b),
                _ => vec![a],
            };
            if let Some(m) = c.name("m") {
                marks.add(m.start());
            }
        } else if out.seasons.is_empty() || w.starts_with('[') || start > 0 {
            if let Some(c) = ANIME_EP.captures(&w) {
                out.episodes = vec![c["e"].parse().unwrap_or(0)];
                if let Some(m) = c.get(0) {
                    marks.add(m.start());
                }
            }
        }
    }

    if let Some(c) = COMPLETE.captures(&w) {
        out.complete = true;
        if let Some(m) = c.name("m") {
            marks.add(m.start());
        }
    }

    let mut other_min = usize::MAX;
    let mut note = |marks: &mut Marks, at: usize| {
        marks.add(at);
        if at > marks.start {
            other_min = other_min.min(at);
        }
    };

    if let Some((e, s)) = vocab::first_match(&RESOLUTIONS, &w) {
        out.resolution = Some(canonical_resolution(e.value));
        note(&mut marks, s.start);
    }
    if let Some((e, s)) = vocab::first_match(&QUALITIES, &w) {
        out.quality = Some(e.value.to_string());
        note(&mut marks, s.start);
    }
    if let Some((_, s)) = vocab::first_match(&ENCODES, &w) {
        out.codec = w.get(s.start..s.end).map(str::to_lowercase);
        note(&mut marks, s.start);
    }
    for (e, s) in vocab::all_matches(&VISUAL_TAGS, &w) {
        let name = match e.value {
            "DV" => "DV",
            other => other,
        };
        match name {
            "10bit" => {}
            "Upscaled" | "AI" => {}
            tag => push_unique(&mut out.hdr, tag),
        }
        if !matches!(e.value, "Upscaled" | "AI") {
            note(&mut marks, s.start);
        }
    }
    for (e, s) in vocab::all_matches(&AUDIO_TAGS, &w) {
        push_unique(&mut out.audio, release_audio_name(e.value));
        note(&mut marks, s.start);
    }
    for (e, _) in vocab::all_matches(&CHANNELS, &w) {
        push_unique(&mut out.channels, e.value);
    }
    if let Some(c) = BIT_DEPTH.captures(&w) {
        let d = c.name("d").map_or("10", |m| m.as_str());
        out.bit_depth = Some(format!("{d}bit"));
        if let Some(m) = c.name("m") {
            note(&mut marks, m.start());
        }
    }

    for (name, re) in EDITIONS.iter() {
        if let Some(m) = re.captures(&w).and_then(|c| c.name("m")) {
            push_unique(&mut out.editions, name);
            note(&mut marks, m.start());
        }
    }

    let flags: [(&Regex, &mut bool); 8] = [
        (&REPACK, &mut out.repack),
        (&PROPER, &mut out.proper),
        (&RETAIL, &mut out.retail),
        (&UNRATED, &mut out.unrated),
        (&UNCENSORED, &mut out.uncensored),
        (&REGRADED, &mut out.regraded),
        (&UPSCALED, &mut out.upscaled),
        (&HARDCODED, &mut out.hardcoded),
    ];
    for (re, flag) in flags {
        if let Some(m) = re.captures(&w).and_then(|c| c.name("m")) {
            *flag = true;
            note(&mut marks, m.start());
        }
    }

    if let Some(m) = REGION.captures(&w).and_then(|c| c.name("m")) {
        if w.get(..m.start())
            .is_some_and(|before| before.len() > start)
        {
            out.region = Some(m.as_str().to_uppercase());
        }
    }
    if let Some(m) = NETWORK.captures(&w).and_then(|c| c.name("m")) {
        out.network = Some(network_name(m.as_str()));
    }
    if let Some(m) = SITE.captures(&w).and_then(|c| c.name("m")) {
        out.site = Some(m.as_str().to_string());
    }

    let mut subs_langs: Vec<String> = Vec::new();
    for c in SUB_LANG.captures_iter(&w) {
        let code = c.name("l").map_or("", |m| m.as_str()).to_lowercase();
        if let Some(l) = LANGUAGES
            .iter()
            .find(|l| l.iso2 == code || swe_alias(&code, l.iso1))
        {
            subs_langs.push(l.iso1.to_string());
            out.subbed = true;
        }
    }
    if let Some(m) = STRONG_LANG.captures(&w).and_then(|c| c.name("m")) {
        note(&mut marks, m.start());
    }
    if let Some(m) = MULTI.find(&w) {
        push_unique(&mut out.languages, "multi audio");
        out.dubbed = true;
        note(&mut marks, m.start);
    }
    if let Some(m) = DUAL_AUDIO.find(&w) {
        push_unique(&mut out.languages, "dual audio");
        out.dubbed = true;
        note(&mut marks, m.start);
    }
    if MULTI_SUBS.is_match(&w) {
        push_unique(&mut out.languages, "multi subs");
        out.subbed = true;
    }
    for lang in LANGUAGES.iter() {
        if let Some(s) = lang.find_audio(&w) {
            let code = lang.iso1.to_string();
            if lang.iso1 == "es-419" {
                out.languages.retain(|l| l != "es");
            } else if out.languages.iter().any(|l| l == "es-419") && lang.iso1 == "es" {
                continue;
            }
            if s.start > start {
                push_unique(&mut out.languages, &code);
            }
        }
    }
    for l in subs_langs {
        push_unique(&mut out.languages, &l);
    }
    if SUBBED.is_match(&w) {
        out.subbed = true;
    }
    if vocab::DUBBED.matches(&w) {
        out.dubbed = true;
    }

    let year_limit = if other_min == usize::MAX {
        w.len()
    } else {
        other_min
    };
    let candidates: Vec<&(usize, String)> = years
        .iter()
        .filter(|(pos, _)| *pos > start && out.date.as_ref().is_none_or(|_| true))
        .collect();
    let before_other: Vec<&&(usize, String)> = candidates
        .iter()
        .filter(|(pos, _)| *pos < year_limit)
        .collect();
    let chosen = before_other
        .last()
        .map(|c| (**c).clone())
        .or_else(|| candidates.first().map(|c| (*c).clone()));
    if let Some((pos, value)) = chosen {
        if out.date.is_none() {
            out.year = Some(value);
        }
        marks.add(pos);
    }

    if out.country.is_none() {
        if let Some(c) = COUNTRY.captures(&w) {
            if let Some(m) = c.name("m") {
                if m.start() > start && (!out.seasons.is_empty() || out.year.is_some()) {
                    out.country = Some(m.as_str().to_uppercase());
                }
            }
        }
    }

    let title_end = marks.end();
    let ep_marker_end = SXE
        .captures(&w)
        .and_then(|c| c.name("m").map(|m| m.end()))
        .or_else(|| NXN.captures(&w).and_then(|c| c.name("m").map(|m| m.end())));
    if let Some(ep_end) = ep_marker_end {
        let next = marks
            .positions
            .iter()
            .copied()
            .filter(|p| *p > ep_end)
            .min()
            .unwrap_or(w.len());
        if next > ep_end + 1 {
            out.episode_title = w.get(ep_end..next).and_then(clean_title);
        }
    }

    let raw_title = w.get(start..title_end).unwrap_or("");
    out.title = clean_title(raw_title);
    out
}

static MULTI_SUBS: LazyLock<Regex> =
    LazyLock::new(|| bounded(r"multi(?:ple)?[ .-]?sub(?:s|titles?)?"));

fn swe_alias(code: &str, iso1: &str) -> bool {
    matches!(
        (code, iso1),
        ("chi", "zh")
            | ("ger", "de")
            | ("fre", "fr")
            | ("jap", "ja")
            | ("eng", "en")
            | ("swe", "sv")
            | ("spa", "es")
            | ("ita", "it")
            | ("rus", "ru")
    )
}

fn network_name(token: &str) -> String {
    match token.to_lowercase().trim_end_matches('+') {
        "nf" | "netflix" => "Netflix",
        "amzn" | "amazon" => "Amazon",
        "dsnp" | "disney" => "Disney+",
        "hulu" => "Hulu",
        "hbo" | "hmax" => "HBO",
        "atvp" | "appletv" => "Apple TV+",
        "pcok" | "peacock" => "Peacock",
        "pmtp" | "paramount" => "Paramount+",
        other => return other.to_string(),
    }
    .to_string()
}

fn release_audio_name(value: &str) -> &str {
    match value {
        "DD" => "AC3",
        "DD+" => "EAC3",
        "DTS-HD MA" => "DTS Lossless",
        "DTS-HD" | "DTS-ES" | "DTS:X" => "DTS Lossless",
        "DTS" => "DTS Lossy",
        other => other,
    }
}

fn is_metadata_only(text: &str) -> bool {
    let w = normalise(text);
    vocab::first_match(&RESOLUTIONS, &w).is_some() || vocab::first_match(&QUALITIES, &w).is_some()
}

fn is_token(text: &str) -> bool {
    let probe = format!(" {text} ");
    let lower = text.to_lowercase();
    matches!(lower.as_str(), "hd" | "ma" | "dl" | "rip" | "hdr" | "atmos")
        || vocab::first_match(&RESOLUTIONS, &probe).is_some()
        || vocab::first_match(&ENCODES, &probe).is_some()
        || (text.len() <= 3 && text.chars().all(|c| c.is_ascii_digit()))
}
