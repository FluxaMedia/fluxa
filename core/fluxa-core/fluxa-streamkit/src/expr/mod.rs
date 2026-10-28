mod funcs;
mod syntax;

use std::collections::HashMap;
use std::fmt;

use crate::types::{MediaType, ParsedStream, PinPosition, RequestContext};
use syntax::{Expr, Op};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(f64),
    Bool(bool),
    Str(String),
    List(Vec<Value>),
    Streams(Vec<usize>),
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Num(n) => *n != 0.0,
            Value::Bool(b) => *b,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Streams(s) => !s.is_empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompileError(String);

impl CompileError {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CompileError {}

#[derive(Clone, Debug, PartialEq)]
pub struct EvalError(String);

impl EvalError {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EvalError {}

#[derive(Clone, Debug)]
pub struct Scope {
    vars: HashMap<String, Value>,
    health: HashMap<String, bool>,
}

impl Default for Scope {
    fn default() -> Self {
        let num = |n: f64| Value::Num(n);
        let vars = [
            ("queryType", Value::Str("movie".into())),
            ("isAnime", Value::Bool(false)),
            ("season", num(-1.0)),
            ("episode", num(-1.0)),
            ("absoluteEpisode", num(-1.0)),
            ("genres", Value::List(Vec::new())),
            ("title", Value::Str(String::new())),
            ("year", num(0.0)),
            ("yearEnd", num(0.0)),
            ("daysSinceRelease", num(-1.0)),
            ("runtime", num(0.0)),
            ("originalLanguage", Value::Str(String::new())),
            ("hasSeaDex", Value::Bool(false)),
            ("hasNextEpisode", Value::Bool(false)),
            ("daysUntilNextEpisode", num(-1.0)),
            ("daysSinceFirstAired", num(-1.0)),
            ("daysSinceLastAired", num(-1.0)),
            ("latestSeason", num(-1.0)),
            ("ongoingSeason", Value::Bool(false)),
            ("totalStreams", num(0.0)),
            ("previousGroupTimeTaken", num(0.0)),
            ("totalTimeTaken", num(0.0)),
            ("queriedAddons", Value::List(Vec::new())),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        Self {
            vars,
            health: HashMap::new(),
        }
    }
}

impl Scope {
    pub fn from_request(ctx: &RequestContext) -> Self {
        let mut s = Self::default();
        let opt = |v: Option<i64>, none: f64| Value::Num(v.map_or(none, |n| n as f64));
        let kind = match (ctx.is_anime, ctx.media_type) {
            (true, MediaType::Movie) => "anime.movie",
            (true, MediaType::Series) => "anime.series",
            (false, MediaType::Movie) => "movie",
            (false, MediaType::Series) => "series",
        };
        s.set("queryType", Value::Str(kind.into()));
        s.set("isAnime", Value::Bool(ctx.is_anime));
        s.set("season", opt(ctx.season.map(i64::from), -1.0));
        s.set("episode", opt(ctx.episode.map(i64::from), -1.0));
        s.set(
            "absoluteEpisode",
            opt(ctx.absolute_episode.map(i64::from), -1.0),
        );
        s.set(
            "genres",
            Value::List(ctx.genres.iter().cloned().map(Value::Str).collect()),
        );
        s.set(
            "title",
            Value::Str(
                ctx.titles
                    .first()
                    .map(|t| t.title.clone())
                    .unwrap_or_default(),
            ),
        );
        s.set("year", opt(ctx.year.map(i64::from), 0.0));
        s.set("yearEnd", opt(ctx.year_end.map(i64::from), 0.0));
        s.set("daysSinceRelease", opt(ctx.days_since_release, -1.0));
        s.set("runtime", opt(ctx.runtime.map(i64::from), 0.0));
        s.set(
            "originalLanguage",
            Value::Str(ctx.original_language.clone().unwrap_or_default()),
        );
        s.set(
            "latestSeason",
            opt(
                ctx.seasons.iter().map(|x| i64::from(x.season_number)).max(),
                -1.0,
            ),
        );
        s
    }

    pub fn set(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }

    pub fn set_health(&mut self, id: &str, ok: bool) {
        self.health.insert(id.to_string(), ok);
    }
}

#[derive(Clone, Debug)]
pub struct Evaluation {
    pub value: Value,
    pub pinned: Vec<(usize, PinPosition)>,
    pub passthrough: Vec<(usize, Vec<String>)>,
}

impl Evaluation {
    pub fn selected(&self) -> &[usize] {
        match &self.value {
            Value::Streams(s) => s,
            _ => &[],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Compiled {
    expr: Expr,
    names: Vec<String>,
}

impl Compiled {
    pub fn compile(src: &str) -> Result<Self, CompileError> {
        let (expr, names) = syntax::parse(src)?;
        Ok(Self { expr, names })
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn eval(&self, streams: &[ParsedStream], scope: &Scope) -> Result<Evaluation, EvalError> {
        let mut run = Run {
            streams,
            scope,
            pinned: Vec::new(),
            passthrough: Vec::new(),
        };
        let value = run.eval(&self.expr)?;
        Ok(Evaluation {
            value,
            pinned: run.pinned,
            passthrough: run.passthrough,
        })
    }

    pub fn condition(&self, scope: &Scope) -> bool {
        self.eval(&[], scope).is_ok_and(|e| e.value.truthy())
    }
}

struct Run<'a> {
    streams: &'a [ParsedStream],
    scope: &'a Scope,
    pinned: Vec<(usize, PinPosition)>,
    passthrough: Vec<(usize, Vec<String>)>,
}

fn num(v: &Value) -> Result<f64, EvalError> {
    match v {
        Value::Num(n) => Ok(*n),
        _ => Err(EvalError::new("expected a number")),
    }
}

fn same(a: &Value, b: &Value) -> bool {
    a == b
}

impl Run<'_> {
    fn all(&self) -> Vec<usize> {
        (0..self.streams.len()).collect()
    }

    fn eval(&mut self, e: &Expr) -> Result<Value, EvalError> {
        match e {
            Expr::Num(n) => Ok(Value::Num(*n)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Const(name) => match name.as_str() {
                "streams" | "previousStreams" => Ok(Value::Streams(self.all())),
                other => self
                    .scope
                    .vars
                    .get(other)
                    .cloned()
                    .ok_or_else(|| EvalError::new(format!("'{other}' has no value"))),
            },
            Expr::Not(inner) => Ok(Value::Bool(!self.eval(inner)?.truthy())),
            Expr::Neg(inner) => Ok(Value::Num(-num(&self.eval(inner)?)?)),
            Expr::Ternary(c, a, b) => {
                if self.eval(c)?.truthy() {
                    self.eval(a)
                } else {
                    self.eval(b)
                }
            }
            Expr::Binary(op, l, r) => self.binary(*op, l, r),
            Expr::Call(name, args) => {
                let values = args
                    .iter()
                    .map(|a| self.eval(a))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call(name, values)
            }
        }
    }

    fn binary(&mut self, op: Op, l: &Expr, r: &Expr) -> Result<Value, EvalError> {
        if op == Op::And {
            let left = self.eval(l)?;
            return Ok(Value::Bool(left.truthy() && self.eval(r)?.truthy()));
        }
        if op == Op::Or {
            let left = self.eval(l)?;
            return Ok(Value::Bool(left.truthy() || self.eval(r)?.truthy()));
        }
        let (a, b) = (self.eval(l)?, self.eval(r)?);
        let order = |a: &Value, b: &Value| -> Result<std::cmp::Ordering, EvalError> {
            match (a, b) {
                (Value::Num(x), Value::Num(y)) => x
                    .partial_cmp(y)
                    .ok_or_else(|| EvalError::new("cannot order NaN")),
                (Value::Str(x), Value::Str(y)) => Ok(x.cmp(y)),
                _ => Err(EvalError::new("cannot compare these values")),
            }
        };
        Ok(match op {
            Op::Eq => Value::Bool(same(&a, &b)),
            Op::Ne => Value::Bool(!same(&a, &b)),
            Op::Gt => Value::Bool(order(&a, &b)?.is_gt()),
            Op::Lt => Value::Bool(order(&a, &b)?.is_lt()),
            Op::Ge => Value::Bool(order(&a, &b)?.is_ge()),
            Op::Le => Value::Bool(order(&a, &b)?.is_le()),
            Op::In => Value::Bool(match (&a, &b) {
                (x, Value::List(l)) => l.iter().any(|v| same(v, x)),
                (Value::Str(x), Value::Str(y)) => y.contains(x.as_str()),
                (Value::Num(x), Value::Streams(s)) => s.iter().any(|i| *i as f64 == *x),
                _ => return Err(EvalError::new("'in' needs a sequence on the right")),
            }),
            Op::Add => match (&a, &b) {
                (Value::Str(x), y) => Value::Str(format!(
                    "{x}{}",
                    funcs::flatten_text(std::slice::from_ref(y)).join("")
                )),
                (Value::List(x), Value::List(y)) => {
                    Value::List(x.iter().chain(y).cloned().collect())
                }
                _ => Value::Num(num(&a)? + num(&b)?),
            },
            Op::Sub => Value::Num(num(&a)? - num(&b)?),
            Op::Mul => Value::Num(num(&a)? * num(&b)?),
            Op::Div => {
                let d = num(&b)?;
                if d == 0.0 {
                    return Err(EvalError::new("division by zero"));
                }
                Value::Num(num(&a)? / d)
            }
            Op::And | Op::Or => return Err(EvalError::new("logical operator reached comparison")),
        })
    }

    fn list<'v>(&self, args: &'v [Value]) -> Result<&'v [usize], EvalError> {
        match args.first() {
            Some(Value::Streams(s)) => Ok(s),
            _ => Err(EvalError::new("first argument must be a stream list")),
        }
    }

    fn keep(&self, idx: &[usize], f: impl Fn(&ParsedStream) -> bool) -> Value {
        Value::Streams(
            idx.iter()
                .copied()
                .filter(|&i| f(&self.streams[i]))
                .collect(),
        )
    }

    fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, EvalError> {
        if name == "health" {
            let id = funcs::flatten_text(&args)
                .into_iter()
                .next()
                .unwrap_or_default();
            return Ok(Value::Bool(
                self.scope.health.get(&id).copied().unwrap_or(false),
            ));
        }
        if funcs::ATTRS.contains(&name) {
            let idx = self.list(&args)?.to_vec();
            let wanted = funcs::flatten_text(&args[1..]);
            return Ok(Value::Streams(funcs::filter_attr(
                self.streams,
                &idx,
                name,
                &wanted,
            )));
        }
        if matches!(
            name,
            "count"
                | "max"
                | "min"
                | "avg"
                | "sum"
                | "median"
                | "mode"
                | "range"
                | "variance"
                | "stddev"
                | "q1"
                | "q3"
                | "iqr"
                | "skewness"
                | "kurtosis"
                | "percentile"
        ) {
            return funcs::stat(name, &args);
        }
        match name {
            "values" => {
                let idx = self.list(&args)?;
                let attr = funcs::flatten_text(&args[1..])
                    .into_iter()
                    .next()
                    .unwrap_or_default();
                if !funcs::NUMERIC.contains(&attr.as_str()) {
                    return Err(EvalError::new(format!("unknown value attribute '{attr}'")));
                }
                Ok(Value::List(
                    idx.iter()
                        .filter_map(|&i| funcs::numeric_of(&self.streams[i], &attr))
                        .map(Value::Num)
                        .collect(),
                ))
            }
            "seeders" | "age" | "size" | "folderSize" | "bitrate" | "seScore" | "regexScore" => {
                let idx = self.list(&args)?;
                let lo = funcs::bound(args.get(1), f64::NEG_INFINITY)?;
                let hi = funcs::bound(args.get(2), f64::INFINITY)?;
                Ok(Value::Streams(
                    idx.iter()
                        .copied()
                        .filter(|&i| {
                            funcs::numeric_of(&self.streams[i], name)
                                .is_some_and(|v| v >= lo && v <= hi)
                        })
                        .collect(),
                ))
            }
            "cached" => Ok(self.keep(self.list(&args)?, |s| s.is_cached() == Some(true))),
            "uncached" => Ok(self.keep(self.list(&args)?, |s| s.is_cached() == Some(false))),
            "library" => Ok(self.keep(self.list(&args)?, |s| s.library)),
            "idMatched" => Ok(self.keep(self.list(&args)?, |s| s.id_matched)),
            "seasonPack" => {
                let from_pack = match args.get(1) {
                    None => false,
                    Some(Value::Str(m)) if m == "onlySeasons" => false,
                    Some(Value::Str(m)) if m == "seasonPack" => true,
                    Some(_) => {
                        return Err(EvalError::new(
                            "seasonPack mode must be 'seasonPack' or 'onlySeasons'",
                        ));
                    }
                };
                Ok(self.keep(self.list(&args)?, |s| {
                    if from_pack {
                        s.parsed.season_pack
                    } else {
                        !s.parsed.seasons.is_empty() && s.parsed.episodes.is_empty()
                    }
                }))
            }
            "audioTrack" | "subtitleTrack" => {
                self.list(&args)?;
                Ok(Value::Streams(Vec::new()))
            }
            "message" => {
                let exact = match args.get(1) {
                    Some(Value::Str(m)) if m == "exact" => true,
                    Some(Value::Str(m)) if m == "includes" => false,
                    _ => return Err(EvalError::new("message mode must be 'exact' or 'includes'")),
                };
                let wanted = funcs::flatten_text(&args[2..]);
                Ok(self.keep(self.list(&args)?, |s| {
                    s.message.as_deref().is_some_and(|m| {
                        wanted.iter().any(|w| {
                            if exact {
                                m == w
                            } else {
                                m.contains(w.as_str())
                            }
                        })
                    })
                }))
            }
            "seadex" => {
                let kind = funcs::flatten_text(
                    args.get(1)
                        .into_iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
                .into_iter()
                .next()
                .unwrap_or_else(|| "all".into());
                let method = funcs::flatten_text(
                    args.get(2)
                        .into_iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
                .into_iter()
                .next();
                if !matches!(kind.as_str(), "best" | "alt" | "all") {
                    return Err(EvalError::new("seadex type must be 'best', 'alt' or 'all'"));
                }
                Ok(self.keep(self.list(&args)?, |s| {
                    let Some(x) = &s.seadex else { return false };
                    let kind_ok = match kind.as_str() {
                        "best" => x.is_best,
                        "alt" => x.is_seadex && !x.is_best,
                        _ => x.is_seadex,
                    };
                    let method_ok = method
                        .as_deref()
                        .is_none_or(|m| format!("{:?}", x.method).eq_ignore_ascii_case(m));
                    kind_ok && method_ok
                }))
            }
            "keyword" => {
                let idx = self.list(&args)?.to_vec();
                let attrs = funcs::flatten_text(&args[1..2])
                    .into_iter()
                    .next()
                    .unwrap_or_default();
                let words = funcs::flatten_text(&args[2..]);
                Ok(Value::Streams(funcs::filter_keywords(
                    self.streams,
                    &idx,
                    &attrs,
                    &words,
                )))
            }
            "regexMatched" => {
                let names = funcs::flatten_text(&args[1..]);
                Ok(self.keep(self.list(&args)?, |s| {
                    s.regex_matched.as_ref().is_some_and(|m| {
                        names.is_empty() || m.name.as_ref().is_some_and(|n| names.contains(n))
                    })
                }))
            }
            "seMatched" => {
                let names = funcs::flatten_text(&args[1..]);
                Ok(self.keep(self.list(&args)?, |s| {
                    s.stream_expression_matched.as_ref().is_some_and(|m| {
                        names.is_empty() || m.name.as_ref().is_some_and(|n| names.contains(n))
                    })
                }))
            }
            "rseMatched" => {
                let names = funcs::flatten_text(&args[1..]);
                Ok(self.keep(self.list(&args)?, |s| {
                    s.ranked_stream_expressions_matched.iter().any(|m| {
                        names.is_empty() || m.name.as_ref().is_some_and(|n| names.contains(n))
                    })
                }))
            }
            "regexMatchedInRange" | "seMatchedInRange" => {
                let lo = num(&args[1])?;
                let hi = num(&args[2])?;
                let regex = name == "regexMatchedInRange";
                Ok(self.keep(self.list(&args)?, |s| {
                    let index = if regex {
                        s.regex_matched.as_ref().map(|m| m.index)
                    } else {
                        s.stream_expression_matched.as_ref().map(|m| m.index)
                    };
                    index.is_some_and(|i| (i as f64) >= lo && (i as f64) <= hi)
                }))
            }
            "passthrough" => {
                let idx = self.list(&args)?.to_vec();
                let stages = funcs::flatten_text(&args[1..]);
                for &i in &idx {
                    self.passthrough.push((i, stages.clone()));
                }
                Ok(Value::Streams(idx))
            }
            "pin" => {
                let idx = self.list(&args)?.to_vec();
                let pos = funcs::pin_position(args.get(1))?;
                let ret = matches!(args.get(2), Some(v) if v.truthy());
                for &i in &idx {
                    self.pinned.push((i, pos));
                }
                Ok(Value::Streams(if ret { idx } else { Vec::new() }))
            }
            "merge" => {
                let mut out: Vec<usize> = Vec::new();
                for a in &args {
                    match a {
                        Value::Streams(s) => {
                            for i in s {
                                if !out.contains(i) {
                                    out.push(*i);
                                }
                            }
                        }
                        _ => return Err(EvalError::new("merge takes stream lists")),
                    }
                }
                Ok(Value::Streams(out))
            }
            "negate" => match (&args[0], &args[1]) {
                (Value::Streams(drop), Value::Streams(orig)) => Ok(Value::Streams(
                    orig.iter().copied().filter(|i| !drop.contains(i)).collect(),
                )),
                _ => Err(EvalError::new("negate takes two stream lists")),
            },
            "slice" => {
                let idx = self.list(&args)?;
                let len = idx.len() as i64;
                let norm = |n: f64| -> usize {
                    let n = n as i64;
                    (if n < 0 { len + n } else { n }).clamp(0, len) as usize
                };
                let start = norm(num(&args[1])?);
                let end = args
                    .get(2)
                    .map(|v| num(v).map(norm))
                    .transpose()?
                    .unwrap_or(idx.len());
                Ok(Value::Streams(
                    idx.get(start..end.max(start)).unwrap_or_default().to_vec(),
                ))
            }
            "perGroup" => {
                let idx = self.list(&args)?.to_vec();
                let attr = funcs::flatten_text(&args[1..2])
                    .into_iter()
                    .next()
                    .unwrap_or_default();
                let n = num(&args[2])?.max(0.0) as usize;
                let filters = funcs::flatten_text(&args[3..]);
                Ok(Value::Streams(funcs::per_group(
                    self.streams,
                    &idx,
                    &attr,
                    n,
                    &filters,
                )?))
            }
            other => Err(EvalError::new(format!("'{other}' is not implemented"))),
        }
    }
}
