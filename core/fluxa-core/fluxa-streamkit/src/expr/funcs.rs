use super::{EvalError, Value};
use crate::rules::filter::{compile_keywords, searchable_fields};
use crate::types::{ParsedStream, PinPosition};

pub const ATTRS: &[&str] = &[
    "resolution",
    "quality",
    "encode",
    "type",
    "visualTag",
    "audioTag",
    "audioChannels",
    "language",
    "subtitle",
    "indexer",
    "service",
    "releaseGroup",
    "addon",
    "editions",
    "network",
    "container",
    "extension",
    "mediaInfoQuality",
];

pub const NUMERIC: &[&str] = &[
    "seeders",
    "age",
    "size",
    "folderSize",
    "bitrate",
    "duration",
    "seScore",
    "regexScore",
];

const STATS: &[&str] = &[
    "count", "max", "min", "avg", "sum", "median", "mode", "range", "variance", "stddev", "q1",
    "q3", "iqr", "skewness", "kurtosis",
];

const CONSTANTS: &[&str] = &[
    "streams",
    "previousStreams",
    "queryType",
    "isAnime",
    "season",
    "episode",
    "absoluteEpisode",
    "genres",
    "title",
    "year",
    "yearEnd",
    "daysSinceRelease",
    "runtime",
    "originalLanguage",
    "hasSeaDex",
    "hasNextEpisode",
    "daysUntilNextEpisode",
    "daysSinceFirstAired",
    "daysSinceLastAired",
    "latestSeason",
    "ongoingSeason",
    "totalStreams",
    "previousGroupTimeTaken",
    "totalTimeTaken",
    "queriedAddons",
];

pub fn is_constant(name: &str) -> bool {
    CONSTANTS.contains(&name)
}

pub fn canonical(name: &str) -> &str {
    match name {
        "subtitles" => "subtitle",
        "keywords" => "keyword",
        "streamExpressionScore" => "seScore",
        "mean" => "avg",
        "q2" => "median",
        "audioChannel" => "audioChannels",
        other => other,
    }
}

pub fn arity(name: &str) -> Option<(usize, usize)> {
    let n = canonical(name);
    if ATTRS.contains(&n) {
        return Some(if n == "releaseGroup" {
            (1, usize::MAX)
        } else {
            (2, usize::MAX)
        });
    }
    if STATS.contains(&n) {
        return Some((1, usize::MAX));
    }
    Some(match n {
        "seeders" | "age" | "size" | "folderSize" | "bitrate" | "seScore" | "regexScore" => (1, 3),
        "cached" | "uncached" | "library" | "idMatched" => (1, 1),
        "seasonPack" => (1, 2),
        "audioTrack" | "subtitleTrack" | "regexMatched" | "seMatched" | "rseMatched"
        | "passthrough" | "merge" => (1, usize::MAX),
        "message" => (2, usize::MAX),
        "seadex" => (1, 3),
        "keyword" => (3, usize::MAX),
        "regexMatchedInRange" | "seMatchedInRange" => (3, 3),
        "pin" => (1, 3),
        "negate" => (2, 2),
        "slice" => (2, 3),
        "perGroup" => (3, usize::MAX),
        "values" => (2, 2),
        "percentile" => (2, usize::MAX),
        "health" => (1, 1),
        _ => return None,
    })
}

pub fn attr_values(s: &ParsedStream, attr: &str) -> Option<Vec<String>> {
    let p = &s.parsed;
    let one = |v: &Option<String>| v.iter().cloned().collect::<Vec<_>>();
    Some(match attr {
        "resolution" => one(&p.resolution),
        "quality" => one(&p.quality),
        "encode" => one(&p.encode),
        "type" => vec![s.stream_type.as_str().to_string()],
        "visualTag" => p.visual_tags.clone(),
        "audioTag" => p.audio_tags.clone(),
        "audioChannels" => p.audio_channels.clone(),
        "language" => p.languages.clone(),
        "subtitle" => p.subtitles.clone(),
        "indexer" => one(&s.indexer),
        "service" => s.service.iter().map(|x| x.id.clone()).collect(),
        "releaseGroup" => one(&p.release_group),
        "addon" => vec![s.addon.name.clone()],
        "editions" => p.editions.clone(),
        "network" => one(&p.network),
        "container" => one(&p.container),
        "extension" => one(&p.extension),
        "mediaInfoQuality" => s
            .media_info_quality
            .iter()
            .map(|q| q.as_str().to_string())
            .collect(),
        _ => return None,
    })
}

fn case_sensitive(attr: &str) -> bool {
    matches!(attr, "indexer" | "releaseGroup" | "addon")
}

fn text_eq(a: &str, b: &str, sensitive: bool) -> bool {
    if sensitive {
        a == b
    } else {
        a.eq_ignore_ascii_case(b)
    }
}

pub fn flatten_text(args: &[Value]) -> Vec<String> {
    let mut out = Vec::new();
    for a in args {
        match a {
            Value::Str(s) => out.push(s.clone()),
            Value::Num(n) => out.push(n.to_string()),
            Value::Bool(b) => out.push(b.to_string()),
            Value::List(l) => out.extend(flatten_text(l)),
            Value::Streams(_) => {}
        }
    }
    out
}

pub fn parse_quantity(text: &str) -> Option<f64> {
    let t = text.trim();
    let split = t
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(t.len());
    let (num, unit) = t.split_at(split);
    let n: f64 = num.parse().ok()?;
    let mult = match unit.trim().to_lowercase().as_str() {
        "" | "b" | "bps" => 1.0,
        "kb" | "kbps" => 1e3,
        "mb" | "mbps" => 1e6,
        "gb" | "gbps" => 1e9,
        "tb" | "tbps" => 1e12,
        "kib" => 1024.0,
        "mib" => 1024.0 * 1024.0,
        "gib" => 1024.0 * 1024.0 * 1024.0,
        "tib" => 1024.0f64.powi(4),
        _ => return None,
    };
    Some(n * mult)
}

pub fn bound(arg: Option<&Value>, default: f64) -> Result<f64, EvalError> {
    match arg {
        None => Ok(default),
        Some(Value::Num(n)) => Ok(*n),
        Some(Value::Str(s)) => {
            parse_quantity(s).ok_or_else(|| EvalError::new(format!("bad quantity '{s}'")))
        }
        Some(_) => Err(EvalError::new(
            "range bounds must be numbers or quantity strings",
        )),
    }
}

pub fn numeric_of(s: &ParsedStream, attr: &str) -> Option<f64> {
    match attr {
        "seeders" => s.seeders().map(f64::from),
        "age" => s.age,
        "size" => s.size.map(|v| v as f64),
        "folderSize" => s.folder_size.map(|v| v as f64),
        "bitrate" => s.bitrate.map(|v| v as f64),
        "duration" => s.duration.map(|v| v as f64),
        "seScore" => Some(s.stream_expression_score),
        "regexScore" => Some(s.regex_score),
        _ => None,
    }
}

pub fn filter_attr(
    streams: &[ParsedStream],
    idx: &[usize],
    attr: &str,
    wanted: &[String],
) -> Vec<usize> {
    let sensitive = case_sensitive(attr);
    idx.iter()
        .copied()
        .filter(|&i| {
            let values = attr_values(&streams[i], attr).unwrap_or_default();
            if attr == "releaseGroup" && wanted.is_empty() {
                return !values.is_empty();
            }
            if values.is_empty() {
                return wanted.iter().any(|w| w.eq_ignore_ascii_case("Unknown"));
            }
            values
                .iter()
                .any(|v| wanted.iter().any(|w| text_eq(v, w, sensitive)))
        })
        .collect()
}

pub fn filter_keywords(
    streams: &[ParsedStream],
    idx: &[usize],
    attrs: &str,
    words: &[String],
) -> Vec<usize> {
    let Some(re) = compile_keywords(words) else {
        return Vec::new();
    };
    let all = attrs.trim() == "all" || attrs.trim() == "*";
    let wanted: Vec<&str> = attrs.split(',').map(str::trim).collect();
    let uses = |name: &str| all || wanted.contains(&name);
    idx.iter()
        .copied()
        .filter(|&i| {
            let s = &streams[i];
            let mut fields: Vec<&str> = Vec::new();
            if all {
                fields = searchable_fields(s);
            } else {
                if uses("filename") {
                    fields.extend(s.filename.as_deref());
                }
                if uses("folderName") {
                    fields.extend(s.folder_name.as_deref());
                }
                if uses("indexer") {
                    fields.extend(s.indexer.as_deref());
                }
                if uses("releaseGroup") {
                    fields.extend(s.parsed.release_group.as_deref());
                }
            }
            fields.iter().any(|f| re.is_match(f))
        })
        .collect()
}

pub fn per_group(
    streams: &[ParsedStream],
    idx: &[usize],
    attr: &str,
    n: usize,
    filters: &[String],
) -> Result<Vec<usize>, EvalError> {
    let canon = canonical(attr);
    if !ATTRS.contains(&canon) {
        return Err(EvalError::new(format!("unknown attribute '{attr}'")));
    }
    let fallback = if canon == "service" {
        "none"
    } else {
        "Unknown"
    };
    let mut keys: Vec<String> = filters.to_vec();
    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); keys.len()];
    for &i in idx {
        let values = attr_values(&streams[i], canon).unwrap_or_default();
        let key = if filters.is_empty() {
            Some(
                values
                    .first()
                    .cloned()
                    .unwrap_or_else(|| fallback.to_string()),
            )
        } else {
            filters
                .iter()
                .find(|f| {
                    if values.is_empty() {
                        f.eq_ignore_ascii_case(fallback)
                    } else {
                        values.iter().any(|v| v.eq_ignore_ascii_case(f))
                    }
                })
                .cloned()
        };
        let Some(key) = key else { continue };
        match keys.iter().position(|k| k.eq_ignore_ascii_case(&key)) {
            Some(p) => buckets[p].push(i),
            None => {
                keys.push(key);
                buckets.push(vec![i]);
            }
        }
    }
    for b in &mut buckets {
        b.truncate(n);
    }
    let mut out = Vec::new();
    let mut round = 0;
    loop {
        let mut any = false;
        for b in &buckets {
            if let Some(&i) = b.get(round) {
                if !out.contains(&i) {
                    out.push(i);
                }
                any = true;
            }
        }
        if !any {
            break;
        }
        round += 1;
    }
    Ok(out)
}

pub fn pin_position(v: Option<&Value>) -> Result<PinPosition, EvalError> {
    match v {
        None => Ok(PinPosition::Top),
        Some(Value::Str(s)) if s == "top" => Ok(PinPosition::Top),
        Some(Value::Str(s)) if s == "bottom" => Ok(PinPosition::Bottom),
        Some(_) => Err(EvalError::new("pin position must be 'top' or 'bottom'")),
    }
}

pub fn numbers(args: &[Value]) -> Vec<f64> {
    let mut out = Vec::new();
    for a in args {
        match a {
            Value::Num(n) => out.push(*n),
            Value::List(l) => out.extend(numbers(l)),
            _ => {}
        }
    }
    out
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (p.clamp(0.0, 100.0) / 100.0) * (sorted.len() - 1) as f64;
    let (lo, hi) = (rank.floor() as usize, rank.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (rank - lo as f64)
}

pub fn stat(name: &str, args: &[Value]) -> Result<Value, EvalError> {
    if name == "count" {
        let n = match args {
            [Value::Streams(s)] => s.len(),
            [Value::List(l)] => l.len(),
            other => other.len(),
        };
        return Ok(Value::Num(n as f64));
    }
    let (arr, p) = if name == "percentile" {
        let p = match args.last() {
            Some(Value::Num(p)) => *p,
            _ => return Err(EvalError::new("percentile needs a number as last argument")),
        };
        (numbers(&args[..args.len() - 1]), Some(p))
    } else {
        (numbers(args), None)
    };
    if name == "sum" {
        return Ok(Value::Num(arr.iter().sum()));
    }
    if arr.is_empty() {
        return Err(EvalError::new(format!("{name} of no values")));
    }
    let mut sorted = arr.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = arr.len() as f64;
    let mean = arr.iter().sum::<f64>() / len;
    let moment = |k: i32| arr.iter().map(|v| (v - mean).powi(k)).sum::<f64>() / len;
    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    let out = match name {
        "max" => max,
        "min" => min,
        "avg" => mean,
        "range" => max - min,
        "median" => percentile(&sorted, 50.0),
        "q1" => percentile(&sorted, 25.0),
        "q3" => percentile(&sorted, 75.0),
        "iqr" => percentile(&sorted, 75.0) - percentile(&sorted, 25.0),
        "percentile" => percentile(&sorted, p.unwrap_or(50.0)),
        "variance" => moment(2),
        "stddev" => moment(2).sqrt(),
        "skewness" => {
            let m2 = moment(2);
            if m2 == 0.0 {
                0.0
            } else {
                moment(3) / m2.powf(1.5)
            }
        }
        "kurtosis" => {
            let m2 = moment(2);
            if m2 == 0.0 {
                0.0
            } else {
                moment(4) / (m2 * m2) - 3.0
            }
        }
        "mode" => {
            let mut best = (sorted[0], 0usize);
            let mut i = 0;
            while i < sorted.len() {
                let run = sorted[i..].iter().take_while(|v| **v == sorted[i]).count();
                if run > best.1 {
                    best = (sorted[i], run);
                }
                i += run;
            }
            best.0
        }
        other => return Err(EvalError::new(format!("unknown statistic '{other}'"))),
    };
    Ok(Value::Num(out))
}
