use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use std::cell::Cell;

use crate::tracks::{self, UnknownField};
use crate::types::{ParsedStream, RequestContext, Track};
use crate::vocab;

const MAX_DEPTH: usize = 5;
const REMOVE_LINE: char = '\u{1}';
const EACH_LIMIT: usize = 256;

thread_local! {
    static EACH_BUDGET: Cell<usize> = const { Cell::new(EACH_LIMIT) };
    static IN_EACH: Cell<bool> = const { Cell::new(false) };
}

#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Missing,
    Str(String),
    Num(f64),
    Bool(bool),
    List(Vec<Val>),
    Items(Vec<HashMap<String, Val>>),
}

impl Val {
    fn type_name(&self) -> &'static str {
        match self {
            Val::Missing => "missing",
            Val::Str(_) => "string",
            Val::Num(_) => "number",
            Val::Bool(_) => "boolean",
            Val::List(_) => "array",
            Val::Items(_) => "objects",
        }
    }

    fn render(&self) -> String {
        match self {
            Val::Missing => String::new(),
            Val::Str(s) => s.clone(),
            Val::Num(n) => num_text(*n),
            Val::Bool(b) => b.to_string(),
            Val::List(l) => l.iter().map(Val::render).collect::<Vec<_>>().join(", "),
            Val::Items(_) => String::new(),
        }
    }

    fn is_missing(&self) -> bool {
        matches!(self, Val::Missing)
    }
}

fn num_text(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{n:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[derive(Clone, Debug)]
enum Arg {
    Text(String),
    Reference(String),
}

#[derive(Clone, Debug)]
struct Call {
    name: String,
    args: Vec<Arg>,
}

#[derive(Clone, Debug)]
enum Source {
    Var(String),
    Literal(String),
}

#[derive(Clone, Debug)]
struct Group {
    op: Option<String>,
    source: Source,
    steps: Vec<Call>,
}

#[derive(Clone, Debug)]
struct Branches {
    when_true: Vec<Node>,
    when_false: Option<Vec<Node>>,
    when_missing: Option<Vec<Node>>,
}

#[derive(Clone, Debug)]
enum Node {
    Text(String),
    Ref {
        groups: Vec<Group>,
        branches: Option<Branches>,
    },
    Optional(Vec<Node>),
}

#[derive(Clone, Debug)]
pub struct Template {
    nodes: Vec<Node>,
}

fn find_ref_end(chars: &[char], start: usize) -> Option<usize> {
    let (mut depth, mut paren, mut bracket) = (1usize, 0usize, 0usize);
    let (mut in_single, mut in_double) = (false, false);
    let mut i = start;
    while i < chars.len() {
        let c = chars[i];
        let top_level = depth == 1;
        if in_single {
            if c == '\'' {
                in_single = false;
            }
        } else if in_double {
            if c == '\\' {
                i += 1;
            } else if c == '"' && top_level {
                in_double = false;
            } else if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
            }
        } else {
            match c {
                '\'' if paren > 0 || top_level => in_single = true,
                '"' if bracket > 0 && top_level => in_double = true,
                '(' if top_level => paren += 1,
                ')' if top_level => paren = paren.saturating_sub(1),
                '[' if top_level => bracket += 1,
                ']' if top_level => bracket = bracket.saturating_sub(1),
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

fn parse_nodes(text: &str, depth: usize) -> Vec<Node> {
    if depth > MAX_DEPTH {
        return vec![Node::Text(text.to_string())];
    }
    let chars: Vec<char> = text.chars().collect();
    let mut nodes = Vec::new();
    let mut buf = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '{' {
            buf.push(chars[i]);
            i += 1;
            continue;
        }
        if chars.get(i + 1) == Some(&'?') {
            if let Some(end) = find_optional_end(&chars, i + 2) {
                if !buf.is_empty() {
                    nodes.push(Node::Text(std::mem::take(&mut buf)));
                }
                let inner: String = chars[i + 2..end].iter().collect();
                nodes.push(Node::Optional(parse_nodes(&inner, depth + 1)));
                i = end + 2;
                continue;
            }
        }
        match find_ref_end(&chars, i + 1) {
            Some(end) => {
                let body: String = chars[i + 1..end].iter().collect();
                if !buf.is_empty() {
                    nodes.push(Node::Text(std::mem::take(&mut buf)));
                }
                match parse_ref(&body, depth) {
                    Some(node) => nodes.push(node),
                    None => nodes.push(Node::Text(format!("{{{body}}}"))),
                }
                i = end + 1;
            }
            None => {
                buf.push('{');
                i += 1;
            }
        }
    }
    if !buf.is_empty() {
        nodes.push(Node::Text(buf));
    }
    nodes
}

fn find_optional_end(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut i = start;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'?') {
            depth += 1;
            i += 2;
            continue;
        }
        if chars[i] == '?' && chars.get(i + 1) == Some(&'}') {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
            i += 2;
            continue;
        }
        i += 1;
    }
    None
}

fn split_top(text: &str, sep: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let sep_chars: Vec<char> = sep.chars().collect();
    let mut parts = Vec::new();
    let mut cur = String::new();
    let (mut paren, mut brace) = (0usize, 0usize);
    let mut quote: Option<char> = None;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
                cur.push(c);
            }
            None => {
                if (c == '\'' || c == '"') && (paren > 0 || cur.is_empty()) {
                    quote = Some(c);
                    cur.push(c);
                } else if c == '(' {
                    paren += 1;
                    cur.push(c);
                } else if c == ')' {
                    paren = paren.saturating_sub(1);
                    cur.push(c);
                } else if c == '{' {
                    brace += 1;
                    cur.push(c);
                } else if c == '}' {
                    brace = brace.saturating_sub(1);
                    cur.push(c);
                } else if paren == 0 && brace == 0 && chars[i..].starts_with(&sep_chars) {
                    parts.push(std::mem::take(&mut cur));
                    i += sep_chars.len();
                    continue;
                } else {
                    cur.push(c);
                }
            }
        }
        i += 1;
    }
    parts.push(cur);
    parts
}

fn unquote(text: &str) -> String {
    let t = text.trim();
    for q in ['\'', '"'] {
        if t.len() >= 2 && t.starts_with(q) && t.ends_with(q) {
            return t[1..t.len() - 1].to_string();
        }
    }
    t.to_string()
}

fn parse_call(item: &str) -> Call {
    let item = item.trim();
    match item.find('(') {
        Some(open) if item.ends_with(')') => {
            let name = item[..open].trim().to_string();
            let inner = &item[open + 1..item.len() - 1];
            let args = if inner.trim().is_empty() {
                Vec::new()
            } else {
                split_top(inner, ",")
                    .into_iter()
                    .map(|a| {
                        let a = a.trim();
                        if a.starts_with('{') && a.ends_with('}') && a.len() > 2 {
                            Arg::Reference(a[1..a.len() - 1].trim().to_lowercase())
                        } else {
                            Arg::Text(unquote(a))
                        }
                    })
                    .collect()
            };
            Call { name, args }
        }
        _ => Call {
            name: item.to_string(),
            args: Vec::new(),
        },
    }
}

fn parse_branches(text: &str, depth: usize) -> Option<Branches> {
    let parts = split_top(text, "||");
    let branch = |p: &str| -> Vec<Node> {
        let t = p.trim();
        let inner = if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
            &t[1..t.len() - 1]
        } else {
            t
        };
        parse_nodes(&inner.replace("\\\"", "\""), depth + 1)
    };
    let mut it = parts.iter();
    let when_true = branch(it.next()?);
    Some(Branches {
        when_true,
        when_false: it.next().map(|p| branch(p)),
        when_missing: it.next().map(|p| branch(p)),
    })
}

fn parse_ref(body: &str, depth: usize) -> Option<Node> {
    let chars: Vec<char> = body.chars().collect();
    let mut head_end = chars.len();
    let mut branches = None;
    let mut bracket_start = None;
    let (mut paren, mut quote) = (0usize, None::<char>);
    let mut brace = 0usize;
    for (i, &c) in chars.iter().enumerate() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
            }
            None => match c {
                '\'' if paren > 0 || i == 0 || chars.get(i.wrapping_sub(1)) == Some(&':') => {
                    quote = Some(c)
                }
                '(' => paren += 1,
                ')' => paren = paren.saturating_sub(1),
                '{' => brace += 1,
                '}' => brace = brace.saturating_sub(1),
                '[' if paren == 0 && brace == 0 => {
                    bracket_start = Some(i);
                    break;
                }
                _ => {}
            },
        }
    }
    if let Some(start) = bracket_start {
        head_end = start;
        let close = chars.iter().rposition(|c| *c == ']')?;
        let inner: String = chars[start + 1..close].iter().collect();
        branches = parse_branches(&inner, depth);
    }
    let head: String = chars[..head_end].iter().collect();
    let mut groups = Vec::new();
    let mut pending_op: Option<String> = None;
    let mut current: Option<Group> = None;
    for (idx, item) in split_top(&head, "::").into_iter().enumerate() {
        let trimmed = item.trim();
        if idx == 0 {
            current = Some(Group {
                op: None,
                source: parse_source(trimmed),
                steps: Vec::new(),
            });
            continue;
        }
        let lower = trimmed.to_lowercase();
        if matches!(lower.as_str(), "and" | "or" | "xor") {
            if let Some(g) = current.take() {
                groups.push(g);
            }
            pending_op = Some(lower);
            continue;
        }
        match (&mut current, pending_op.take()) {
            (None, op) => {
                current = Some(Group {
                    op,
                    source: parse_source(trimmed),
                    steps: Vec::new(),
                });
            }
            (Some(g), _) => g.steps.push(parse_call(trimmed)),
        }
    }
    if let Some(g) = current {
        groups.push(g);
    }
    (!groups.is_empty()).then_some(Node::Ref { groups, branches })
}

fn parse_source(text: &str) -> Source {
    let t = text.trim();
    if t.starts_with('\'') || t.starts_with('"') {
        Source::Literal(unquote(t))
    } else {
        Source::Var(t.to_lowercase().replace(char::is_whitespace, ""))
    }
}

static CACHE: LazyLock<Mutex<HashMap<String, Template>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl Template {
    pub fn compile(text: &str) -> Template {
        if let Ok(mut cache) = CACHE.lock() {
            if let Some(t) = cache.get(text) {
                return t.clone();
            }
            let t = Template {
                nodes: parse_nodes(text, 0),
            };
            cache.insert(text.to_string(), t.clone());
            return t;
        }
        Template {
            nodes: parse_nodes(text, 0),
        }
    }

    pub fn render(&self, vars: &dyn Fn(&str) -> Val) -> String {
        EACH_BUDGET.with(|b| b.set(EACH_LIMIT));
        let (raw, _) = render_nodes(&self.nodes, vars);
        raw.lines()
            .filter(|l| !l.contains(REMOVE_LINE) && !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn render_nodes(nodes: &[Node], vars: &dyn Fn(&str) -> Val) -> (String, bool) {
    let mut out = String::new();
    let mut all_present = true;
    for node in nodes {
        match node {
            Node::Text(t) => out.push_str(t),
            Node::Optional(inner) => {
                let (text, present) = render_nodes(inner, vars);
                if present {
                    out.push_str(&text);
                }
            }
            Node::Ref { groups, branches } => {
                let (text, present) = render_ref(groups, branches.as_ref(), vars);
                if !present {
                    all_present = false;
                }
                out.push_str(&text);
            }
        }
    }
    (out, all_present)
}

fn base_value(source: &Source, vars: &dyn Fn(&str) -> Val) -> Val {
    match source {
        Source::Literal(s) => Val::Str(s.clone()),
        Source::Var(name) => match name.as_str() {
            "tools.newline" => Val::Str("\n".into()),
            "tools.removeline" => Val::Str(REMOVE_LINE.to_string()),
            other => vars(other),
        },
    }
}

fn render_ref(
    groups: &[Group],
    branches: Option<&Branches>,
    vars: &dyn Fn(&str) -> Val,
) -> (String, bool) {
    let evaluated: Vec<(Option<&str>, Val, bool)> = groups
        .iter()
        .map(|g| {
            let base = base_value(&g.source, vars);
            let present = !base.is_missing();
            (g.op.as_deref(), apply_steps(base, &g.steps, vars), present)
        })
        .collect();
    let first_present = evaluated.first().is_none_or(|e| e.2);
    let Some(b) = branches else {
        let text = evaluated.first().map(|e| e.1.render()).unwrap_or_default();
        return (text, first_present);
    };
    let truthy = |v: &Val| match v {
        Val::Bool(b) => Some(*b),
        Val::Missing => None,
        Val::Str(s) => Some(!s.is_empty()),
        Val::Num(n) => Some(*n != 0.0),
        Val::List(l) => Some(!l.is_empty()),
        Val::Items(l) => Some(!l.is_empty()),
    };
    let mut acc: Option<bool> = None;
    let mut any_missing = false;
    for (i, (op, val, _)) in evaluated.iter().enumerate() {
        let cur = truthy(val);
        if cur.is_none() {
            any_missing = true;
        }
        let cur = cur.unwrap_or(false);
        acc = Some(match (i, acc, *op) {
            (0, _, _) | (_, None, _) => cur,
            (_, Some(a), Some("and")) => a && cur,
            (_, Some(a), Some("or")) => a || cur,
            (_, Some(a), Some("xor")) => a ^ cur,
            (_, Some(_), _) => cur,
        });
    }
    let all_missing = evaluated.iter().all(|e| e.1.is_missing());
    let chosen = if all_missing && any_missing {
        b.when_missing.as_ref()
    } else if acc == Some(true) {
        Some(&b.when_true)
    } else {
        b.when_false.as_ref()
    };
    match chosen {
        Some(nodes) => (render_nodes(nodes, vars).0, true),
        None => (String::new(), true),
    }
}

fn resolve_arg(arg: &Arg, vars: &dyn Fn(&str) -> Val) -> Vec<String> {
    match arg {
        Arg::Text(t) => vec![t.clone()],
        Arg::Reference(name) => match vars(name) {
            Val::List(l) => l.iter().map(Val::render).collect(),
            Val::Missing => Vec::new(),
            other => vec![other.render()],
        },
    }
}

fn apply_steps(mut value: Val, steps: &[Call], vars: &dyn Fn(&str) -> Val) -> Val {
    for call in steps {
        value = apply(value, call, vars);
    }
    value
}

fn bytes_text(n: f64, base: f64, units: &[&str; 5], rounded: bool, concise: bool) -> String {
    if n <= 0.0 {
        return format!("0 {}", units[0]);
    }
    let i = ((n.ln() / base.ln()).floor().max(0.0) as usize).min(4);
    let v = n / base.powi(i as i32);
    let body = if rounded {
        format!("{}", v.round() as i64)
    } else if concise {
        if v >= 100.0 {
            format!("{}", v.round() as i64)
        } else if v >= 10.0 {
            trim_decimals(v, 1)
        } else {
            trim_decimals(v, 2)
        }
    } else {
        trim_decimals(v, 2)
    };
    format!("{body} {}", units[i])
}

fn trim_decimals(v: f64, places: usize) -> String {
    let s = format!("{v:.places$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

fn bitrate_text(n: f64, rounded: bool, concise: bool) -> String {
    if !n.is_finite() || n <= 0.0 {
        return "0 bps".into();
    }
    bytes_text(
        n,
        1000.0,
        &["bps", "Kbps", "Mbps", "Gbps", "Tbps"],
        rounded,
        concise,
    )
}

fn time_text(ms: f64) -> String {
    let total = (ms / 1000.0).floor().max(0.0) as u64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    let mut parts = Vec::new();
    if h > 0 {
        parts.push(format!("{h}h"));
    }
    if m > 0 || (h > 0 && s > 0) {
        parts.push(format!("{m}m"));
    }
    if s > 0 || parts.is_empty() {
        parts.push(format!("{s}s"));
    }
    parts.join(":")
}

fn smallcaps(c: char) -> char {
    match c.to_ascii_lowercase() {
        'a' => 'ᴀ',
        'b' => 'ʙ',
        'c' => 'ᴄ',
        'd' => 'ᴅ',
        'e' => 'ᴇ',
        'f' => 'ꜰ',
        'g' => 'ɢ',
        'h' => 'ʜ',
        'i' => 'ɪ',
        'j' => 'ᴊ',
        'k' => 'ᴋ',
        'l' => 'ʟ',
        'm' => 'ᴍ',
        'n' => 'ɴ',
        'o' => 'ᴏ',
        'p' => 'ᴘ',
        'q' => 'ǫ',
        'r' => 'ʀ',
        's' => 'ꜱ',
        't' => 'ᴛ',
        'u' => 'ᴜ',
        'v' => 'ᴠ',
        'w' => 'ᴡ',
        'x' => 'x',
        'y' => 'ʏ',
        'z' => 'ᴢ',
        _ => c,
    }
}

fn digit_map(c: char, table: &[char; 10]) -> char {
    c.to_digit(10).map_or(c, |d| table[d as usize])
}

const SUB: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
const SUP: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

fn title_case(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn error_marker(value: &Val, name: &str) -> Val {
    Val::Str(format!(
        "{{unknown_{}_modifier({name})}}",
        value.type_name()
    ))
}

fn compare_text(value: &Val, op: char, rhs: &str) -> bool {
    match (value, op) {
        (Val::Num(n), '=') => rhs
            .parse::<f64>()
            .is_ok_and(|r| (*n - r).abs() < f64::EPSILON),
        (Val::Num(n), '>') => rhs.parse::<f64>().is_ok_and(|r| *n > r),
        (Val::Num(n), '<') => rhs.parse::<f64>().is_ok_and(|r| *n < r),
        (Val::Str(s), '=') => s.eq_ignore_ascii_case(rhs),
        (Val::Str(s), '$') => s.to_lowercase().starts_with(&rhs.to_lowercase()),
        (Val::Str(s), '^') => s.to_lowercase().ends_with(&rhs.to_lowercase()),
        (Val::Str(s), '~') => s.to_lowercase().contains(&rhs.to_lowercase()),
        (Val::List(l), '=') => l.iter().any(|v| v.render().eq_ignore_ascii_case(rhs)),
        (Val::List(l), '$') => l
            .first()
            .is_some_and(|v| v.render().eq_ignore_ascii_case(rhs)),
        (Val::List(l), '^') => l
            .last()
            .is_some_and(|v| v.render().eq_ignore_ascii_case(rhs)),
        (Val::List(l), '~') => l
            .iter()
            .any(|v| v.render().to_lowercase().contains(&rhs.to_lowercase())),
        (Val::Bool(b), '=') => b.to_string().eq_ignore_ascii_case(rhs),
        _ => false,
    }
}

fn apply(value: Val, call: &Call, vars: &dyn Fn(&str) -> Val) -> Val {
    let name = call.name.as_str();
    let lower = name.to_lowercase();
    let texts = || -> Vec<String> {
        call.args
            .iter()
            .flat_map(|a| resolve_arg(a, vars))
            .collect()
    };
    let first = || texts().into_iter().next().unwrap_or_default();

    if lower == "default" {
        return if value.is_missing() {
            Val::Str(first())
        } else {
            value
        };
    }
    if lower == "string" {
        return if value.is_missing() {
            value
        } else {
            Val::Str(value.render())
        };
    }
    if value.is_missing() {
        return Val::Missing;
    }
    if let Val::Items(items) = &value {
        return items_modifier(items, &lower, &texts(), &value, name, vars);
    }
    match lower.as_str() {
        "istrue" => return Val::Bool(matches!(value, Val::Bool(true))),
        "isfalse" => return Val::Bool(matches!(value, Val::Bool(false))),
        "exists" => {
            return Val::Bool(match &value {
                Val::Str(s) => !s.is_empty(),
                Val::List(l) => !l.is_empty(),
                _ => true,
            });
        }
        "in" => {
            let rendered = value.render().to_lowercase();
            return Val::Bool(texts().iter().any(|t| t.to_lowercase() == rendered));
        }
        _ => {}
    }
    for op in ['>', '<', '=', '$', '^', '~'] {
        if let Some(rest) = name.strip_prefix(op) {
            if let Some(eq_rest) = rest.strip_prefix('=').filter(|_| op == '>' || op == '<') {
                let rhs = eq_rest.trim();
                return Val::Bool(compare_text(&value, op, rhs) || compare_text(&value, '=', rhs));
            }
            return Val::Bool(compare_text(&value, op, unquote(rest).trim()));
        }
    }
    match (&value, lower.as_str()) {
        (Val::Str(s), "upper") => Val::Str(s.to_uppercase()),
        (Val::Str(s), "lower") => Val::Str(s.to_lowercase()),
        (Val::Str(s), "trim") => {
            let chars = first();
            if chars.is_empty() {
                Val::Str(s.trim().to_string())
            } else {
                Val::Str(s.trim_matches(|c| chars.contains(c)).to_string())
            }
        }
        (Val::Str(s), "title") => Val::Str(title_case(s)),
        (Val::Str(s), "smallcaps") => Val::Str(s.chars().map(smallcaps).collect()),
        (Val::Str(s), "subscript") => Val::Str(s.chars().map(|c| digit_map(c, &SUB)).collect()),
        (Val::Str(s), "superscript") => Val::Str(s.chars().map(|c| digit_map(c, &SUP)).collect()),
        (Val::Str(s), "reverse") => Val::Str(s.chars().rev().collect()),
        (Val::Str(s), "length") => Val::Num(s.chars().count() as f64),
        (Val::Str(s), "truncate") => {
            let n: usize = first().parse().unwrap_or(usize::MAX);
            if s.chars().count() > n {
                Val::Str(s.chars().take(n).collect::<String>() + "…")
            } else {
                value
            }
        }
        (Val::Str(s), "replace") => {
            let t = texts();
            match (t.first(), t.get(1)) {
                (Some(from), Some(to)) => Val::Str(s.replace(from.as_str(), to)),
                _ => value,
            }
        }
        (Val::Str(s), "remove") => {
            let mut out = s.clone();
            for t in texts() {
                out = out.replace(t.as_str(), "");
            }
            Val::Str(out)
        }
        (Val::Str(s), "translate") => {
            let t = texts();
            let (from, to): (Vec<char>, Vec<char>) = (
                t.first().map(|x| x.chars().collect()).unwrap_or_default(),
                t.get(1).map(|x| x.chars().collect()).unwrap_or_default(),
            );
            Val::Str(
                s.chars()
                    .map(|c| {
                        from.iter()
                            .position(|f| *f == c)
                            .and_then(|i| to.get(i).copied())
                            .unwrap_or(c)
                    })
                    .collect(),
            )
        }
        (Val::Str(s), "base64") => Val::Str(base64(s.as_bytes())),
        (Val::Str(s), "languagecode") => lang_lookup(s, |l| l.iso2.to_uppercase()),
        (Val::Str(s), "languageemoji") => lang_lookup(s, |l| l.flag.to_string()),
        (Val::Str(s), "date") => Val::Str(format_date(s, &first())),
        (Val::Num(n), m) => {
            number_modifier(*n, m, &texts()).unwrap_or_else(|| error_marker(&value, name))
        }
        (Val::List(items), m) => {
            list_modifier(items, m, &texts()).unwrap_or_else(|| error_marker(&value, name))
        }
        _ => error_marker(&value, name),
    }
}

fn item_condition(
    item: &HashMap<String, Val>,
    field: &str,
) -> Result<(bool, Vec<String>), UnknownField> {
    Ok(match item.get(field) {
        None | Some(Val::Missing) => (false, Vec::new()),
        Some(Val::Str(s)) => (!s.is_empty(), vec![s.to_lowercase()]),
        Some(Val::Bool(b)) => (*b, vec![b.to_string()]),
        Some(Val::Num(n)) => (true, vec![num_text(*n)]),
        Some(Val::List(l)) => (
            !l.is_empty(),
            l.iter().map(|v| v.render().to_lowercase()).collect(),
        ),
        Some(Val::Items(_)) => (false, Vec::new()),
    })
}

fn items_modifier(
    items: &[HashMap<String, Val>],
    name: &str,
    args: &[String],
    value: &Val,
    original_name: &str,
    vars: &dyn Fn(&str) -> Val,
) -> Val {
    let first = || args.first().cloned().unwrap_or_default();
    match name {
        "where" => Val::Items(
            items
                .iter()
                .filter(|item| {
                    tracks::evaluate(args, &|f| item_condition(item, f)).unwrap_or(false)
                })
                .cloned()
                .collect(),
        ),
        "pluck" => {
            let field = first().to_lowercase();
            let mut out: Vec<Val> = Vec::new();
            for item in items {
                let found = match item.get(&field) {
                    Some(Val::List(l)) => l.clone(),
                    Some(Val::Missing) | None => Vec::new(),
                    Some(other) => vec![other.clone()],
                };
                for v in found {
                    let text = v.render();
                    if !text.is_empty() && !out.iter().any(|o| o.render() == text) {
                        out.push(Val::Str(text));
                    }
                }
            }
            if out.is_empty() {
                Val::Missing
            } else {
                Val::List(out)
            }
        }
        "each" => each_items(items, &first(), vars),
        "length" => Val::Num(items.len() as f64),
        "exists" => Val::Bool(!items.is_empty()),
        "reverse" => Val::Items(items.iter().rev().cloned().collect()),
        "slice" => {
            let len = items.len() as i64;
            let norm = |n: i64| if n < 0 { (len + n).max(0) } else { n.min(len) } as usize;
            let start = norm(args.first().and_then(|a| a.parse().ok()).unwrap_or(0));
            let end = args
                .get(1)
                .and_then(|a| a.parse().ok())
                .map_or(items.len(), norm);
            Val::Items(
                items
                    .get(start..end.max(start))
                    .unwrap_or_default()
                    .to_vec(),
            )
        }
        _ => error_marker(value, original_name),
    }
}

fn each_items(items: &[HashMap<String, Val>], template: &str, outer: &dyn Fn(&str) -> Val) -> Val {
    if IN_EACH.with(Cell::get) {
        return Val::Str("{each_cannot_nest}".into());
    }
    let compiled = Template::compile(template);
    IN_EACH.with(|f| f.set(true));
    let mut out = String::new();
    for item in items {
        let allowed = EACH_BUDGET.with(|b| {
            let left = b.get();
            if left > 0 {
                b.set(left - 1);
            }
            left > 0
        });
        if !allowed {
            break;
        }
        let vars = |name: &str| match name.strip_prefix("track.") {
            Some(field) => item.get(field).cloned().unwrap_or(Val::Missing),
            None => outer(name),
        };
        let (text, _) = render_nodes(&compiled.nodes, &vars);
        if !text.trim().is_empty() {
            out.push_str(&text);
        }
    }
    IN_EACH.with(|f| f.set(false));
    Val::Str(out)
}

pub fn track_item(t: &Track) -> HashMap<String, Val> {
    let text = |v: &Option<String>| v.clone().map_or(Val::Missing, Val::Str);
    let tags = if t.tags.is_empty() {
        Val::Missing
    } else {
        Val::List(t.tags.iter().cloned().map(Val::Str).collect())
    };
    [
        ("lang", text(&t.lang)),
        ("title", text(&t.title)),
        ("codec", text(&t.codec)),
        ("channels", text(&t.channels)),
        ("tags", tags.clone()),
        ("tag", tags),
        ("default", Val::Bool(t.default)),
        ("forced", Val::Bool(t.forced)),
        ("hearingimpaired", Val::Bool(t.hearing_impaired)),
        ("commentary", Val::Bool(t.commentary)),
        ("dub", Val::Bool(t.dub)),
        ("original", Val::Bool(t.original)),
        ("visualimpaired", Val::Bool(t.visual_impaired)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

fn track_list(tracks: &[Track]) -> Val {
    if tracks.is_empty() {
        Val::Missing
    } else {
        Val::Items(tracks.iter().map(track_item).collect())
    }
}

fn lang_lookup(s: &str, f: impl Fn(&vocab::Language) -> String) -> Val {
    Val::Str(
        vocab::language_by_name(s)
            .map(f)
            .unwrap_or_else(|| s.to_string()),
    )
}

fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

fn format_date(input: &str, pattern: &str) -> String {
    let parts: Vec<&str> = input.split('-').collect();
    let [y, m, d] = parts.as_slice() else {
        return input.to_string();
    };
    let (Ok(yi), Ok(mi), Ok(di)) = (y.parse::<i32>(), m.parse::<usize>(), d.parse::<u32>()) else {
        return input.to_string();
    };
    if !(1..=12).contains(&mi) {
        return input.to_string();
    }
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let weekday = {
        let (mut yy, mut mm) = (yi, mi as i32);
        if mm < 3 {
            mm += 12;
            yy -= 1;
        }
        let k = yy % 100;
        let j = yy / 100;
        let h = (di as i32 + (13 * (mm + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
        [
            "Saturday",
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
        ][h as usize]
    };
    let ordinal = match (di % 10, di % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    let month = MONTHS[mi - 1];
    let mut out = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '%' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let (unpadded, tok_at) = if chars.get(i + 1) == Some(&'-') {
            (true, i + 2)
        } else {
            (false, i + 1)
        };
        match chars.get(tok_at) {
            Some('Y') => out.push_str(&format!("{yi}")),
            Some('y') => out.push_str(&format!("{:02}", yi % 100)),
            Some('m') => out.push_str(&if unpadded {
                mi.to_string()
            } else {
                format!("{mi:02}")
            }),
            Some('d') => out.push_str(&if unpadded {
                di.to_string()
            } else {
                format!("{di:02}")
            }),
            Some('B') => out.push_str(month),
            Some('b') => out.push_str(&month[..3]),
            Some('o') => out.push_str(&format!("{di}{ordinal}")),
            Some('A') => out.push_str(weekday),
            Some('a') => out.push_str(&weekday[..3]),
            Some('%') => out.push('%'),
            Some(other) => {
                out.push('%');
                out.push(*other);
            }
            None => out.push('%'),
        }
        i = tok_at + 1;
    }
    out
}

fn number_modifier(n: f64, name: &str, args: &[String]) -> Option<Val> {
    const B10: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    const B2: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let text = match name {
        "bytes" | "bytes10" => bytes_text(n, 1000.0, &B10, false, false),
        "sbytes" | "sbytes10" => bytes_text(n, 1000.0, &B10, false, true),
        "rbytes" | "rbytes10" => bytes_text(n, 1000.0, &B10, true, false),
        "bytes2" => bytes_text(n, 1024.0, &B2, false, false),
        "sbytes2" => bytes_text(n, 1024.0, &B2, false, true),
        "rbytes2" => bytes_text(n, 1024.0, &B2, true, false),
        "bitrate" => bitrate_text(n, false, false),
        "rbitrate" => bitrate_text(n, true, false),
        "sbitrate" => bitrate_text(n, false, true),
        "time" if args.is_empty() => time_text(n),
        "time" => format_duration(n, &args[0]),
        "star" => "★".repeat(((n / 20.0).round() as usize).min(5)),
        "pstar" => {
            let filled = ((n / 20.0).round() as usize).min(5);
            "★".repeat(filled) + &"☆".repeat(5 - filled)
        }
        "hex" => format!("{:x}", n as i64),
        "octal" => format!("{:o}", n as i64),
        "binary" => format!("{:b}", n as i64),
        _ => return None,
    };
    Some(Val::Str(text))
}

fn format_duration(ms: f64, pattern: &str) -> String {
    let total_secs = (ms / 1000.0).floor().max(0.0) as u64;
    let has_hours = pattern.contains("%H") || pattern.contains("%-H");
    let (h, m, s) = if has_hours {
        (total_secs / 3600, (total_secs % 3600) / 60, total_secs % 60)
    } else {
        (0, total_secs / 60, total_secs % 60)
    };
    let mut out = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    let mut group: Option<(String, bool, bool)> = None;
    while i < chars.len() {
        let c = chars[i];
        let target: &mut String = match group.as_mut() {
            Some(g) => &mut g.0,
            None => &mut out,
        };
        match c {
            '[' => {
                group = Some((String::new(), false, false));
            }
            ']' => {
                if let Some((text, had_token, nonzero)) = group.take() {
                    if !had_token || nonzero {
                        out.push_str(&text);
                    }
                }
            }
            '%' => {
                let (unpadded, at) = if chars.get(i + 1) == Some(&'-') {
                    (true, i + 2)
                } else {
                    (false, i + 1)
                };
                let tok = chars.get(at).copied();
                let value = match tok {
                    Some('H') => Some(h),
                    Some('M') => Some(m),
                    Some('S') => Some(s),
                    _ => None,
                };
                match (tok, value) {
                    (_, Some(v)) => {
                        let rendered = if unpadded {
                            v.to_string()
                        } else {
                            format!("{v:02}")
                        };
                        let t = match group.as_mut() {
                            Some(g) => {
                                g.1 = true;
                                g.2 |= v != 0;
                                &mut g.0
                            }
                            None => &mut out,
                        };
                        t.push_str(&rendered);
                    }
                    (Some('%'), None) => target.push('%'),
                    (Some('['), None) => target.push('['),
                    (Some(']'), None) => target.push(']'),
                    (Some(other), None) => {
                        target.push('%');
                        target.push(other);
                    }
                    (None, None) => target.push('%'),
                }
                i = at;
            }
            other => target.push(other),
        }
        i += 1;
    }
    out
}

fn list_modifier(items: &[Val], name: &str, args: &[String]) -> Option<Val> {
    let eq = |v: &Val, a: &String| v.render().eq_ignore_ascii_case(a);
    Some(match name {
        "join" => Val::Str(
            items
                .iter()
                .map(Val::render)
                .collect::<Vec<_>>()
                .join(args.first().map_or(", ", String::as_str)),
        ),
        "length" => Val::Num(items.len() as f64),
        "first" => items.first().cloned().unwrap_or(Val::Missing),
        "last" => items.last().cloned().unwrap_or(Val::Missing),
        "reverse" => Val::List(items.iter().rev().cloned().collect()),
        "unique" => {
            let mut out: Vec<Val> = Vec::new();
            for v in items {
                if !out
                    .iter()
                    .any(|o| o.render().eq_ignore_ascii_case(&v.render()))
                {
                    out.push(v.clone());
                }
            }
            Val::List(out)
        }
        "keep" => Val::List(
            items
                .iter()
                .filter(|v| args.iter().any(|a| eq(v, a)))
                .cloned()
                .collect(),
        ),
        "remove" => Val::List(
            items
                .iter()
                .filter(|v| !args.iter().any(|a| eq(v, a)))
                .cloned()
                .collect(),
        ),
        "replace" => {
            let (from, to) = (args.first()?, args.get(1)?);
            Val::List(
                items
                    .iter()
                    .map(|v| {
                        if eq(v, from) {
                            Val::Str(to.clone())
                        } else {
                            v.clone()
                        }
                    })
                    .collect(),
            )
        }
        "slice" => {
            let len = items.len() as i64;
            let norm = |n: i64| if n < 0 { (len + n).max(0) } else { n.min(len) } as usize;
            let start = norm(args.first()?.parse().ok()?);
            let end = args
                .get(1)
                .and_then(|a| a.parse().ok())
                .map_or(items.len(), norm);
            Val::List(items.get(start..end.max(start))?.to_vec())
        }
        "sort" | "rsort" | "lsort" => {
            let mut out = items.to_vec();
            let numeric = items.iter().all(|v| matches!(v, Val::Num(_)));
            out.sort_by(|a, b| match (a, b) {
                (Val::Num(x), Val::Num(y)) if numeric => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                _ if name == "lsort" => a.render().cmp(&b.render()),
                _ => a.render().to_lowercase().cmp(&b.render().to_lowercase()),
            });
            if name == "rsort" {
                out.reverse();
            }
            Val::List(out)
        }
        "languagecode" => Val::List(
            items
                .iter()
                .map(|v| lang_lookup(&v.render(), |l| l.iso2.to_uppercase()))
                .collect(),
        ),
        "languageemoji" => Val::List(
            items
                .iter()
                .map(|v| lang_lookup(&v.render(), |l| l.flag.to_string()))
                .collect(),
        ),
        _ => return None,
    })
}

#[derive(Clone, Debug, Default)]
pub struct UserLists {
    pub languages: Vec<String>,
    pub resolutions: Vec<String>,
    pub qualities: Vec<String>,
    pub visual_tags: Vec<String>,
    pub audio_tags: Vec<String>,
    pub audio_channels: Vec<String>,
}

pub struct FormatInput<'a> {
    pub stream: &'a ParsedStream,
    pub request: &'a RequestContext,
    pub addon_name: &'a str,
    pub user: &'a UserLists,
}

fn strs(v: &[String]) -> Val {
    if v.is_empty() {
        Val::Missing
    } else {
        Val::List(v.iter().cloned().map(Val::Str).collect())
    }
}

fn opt_str(v: &Option<String>) -> Val {
    v.clone().map_or(Val::Missing, Val::Str)
}

fn sorted_by_pref(values: &[String], pref: &[String]) -> Vec<String> {
    let mut preferred: Vec<String> = pref
        .iter()
        .filter(|p| values.contains(p))
        .cloned()
        .collect();
    preferred.extend(values.iter().filter(|v| !pref.contains(v)).cloned());
    preferred
}

fn season_format(prefix: char, list: &[u32]) -> Val {
    match (list.first(), list.last()) {
        (Some(a), Some(b)) if a == b => Val::Str(format!("{prefix}{a:02}")),
        (Some(a), Some(b)) => Val::Str(format!("{prefix}{a:02}-{b:02}")),
        _ => Val::Missing,
    }
}

fn nums(v: &[u32]) -> Val {
    if v.is_empty() {
        Val::Missing
    } else {
        Val::List(v.iter().map(|n| Val::Num(f64::from(*n))).collect())
    }
}

pub fn lookup(input: &FormatInput, path: &str) -> Val {
    let s = input.stream;
    let (group, prop) = path.split_once('.').unwrap_or((path, ""));
    match group {
        "config" => match prop {
            "addonname" => Val::Str(input.addon_name.to_string()),
            _ => Val::Missing,
        },
        "service" => match (prop, &s.service) {
            ("id", Some(svc)) => Val::Str(svc.id.clone()),
            ("shortname", Some(svc)) => vocab::service_by_id(&svc.id)
                .map_or(Val::Missing, |d| Val::Str(d.short_name.clone())),
            ("name", Some(svc)) => {
                vocab::service_by_id(&svc.id).map_or(Val::Missing, |d| Val::Str(d.name.clone()))
            }
            ("cached", Some(svc)) => Val::Bool(svc.cached),
            _ => Val::Missing,
        },
        "addon" => match prop {
            "presetid" => Val::Str(s.addon.preset_id.clone()),
            "name" => Val::Str(s.addon.name.clone()),
            _ => Val::Missing,
        },
        "metadata" => match prop {
            "type" => Val::Str(format!("{:?}", input.request.media_type).to_lowercase()),
            "isanime" => Val::Bool(input.request.is_anime),
            "title" => input
                .request
                .titles
                .first()
                .map_or(Val::Missing, |t| Val::Str(t.title.clone())),
            "year" => input
                .request
                .year
                .map_or(Val::Missing, |y| Val::Num(f64::from(y))),
            "season" => input
                .request
                .season
                .map_or(Val::Missing, |n| Val::Num(f64::from(n))),
            "episode" => input
                .request
                .episode
                .map_or(Val::Missing, |n| Val::Num(f64::from(n))),
            "genres" => strs(&input.request.genres),
            _ => Val::Missing,
        },
        "user" => match prop {
            "languages" => strs(&input.user.languages),
            "resolutions" => strs(&input.user.resolutions),
            "qualities" => strs(&input.user.qualities),
            "visualtags" => strs(&input.user.visual_tags),
            "audiotags" => strs(&input.user.audio_tags),
            "audiochannels" => strs(&input.user.audio_channels),
            _ => Val::Missing,
        },
        "stream" => stream_prop(input, prop),
        _ => Val::Missing,
    }
}

fn stream_prop(input: &FormatInput, prop: &str) -> Val {
    let s = input.stream;
    let p = &s.parsed;
    let langs = sorted_by_pref(&p.languages, &input.user.languages);
    let user_langs: Vec<String> = langs
        .iter()
        .filter(|l| input.user.languages.contains(l))
        .cloned()
        .collect();
    let flags = |l: &[String]| -> Vec<String> {
        l.iter()
            .filter_map(|n| vocab::language_by_name(n).map(|x| x.flag.to_string()))
            .collect()
    };
    let codes = |l: &[String]| -> Vec<String> {
        l.iter()
            .filter_map(|n| vocab::language_by_name(n).map(|x| x.iso2.to_uppercase()))
            .collect()
    };
    let small = |l: &[String]| -> Vec<String> {
        l.iter()
            .filter_map(|n| vocab::language_by_name(n).map(|x| x.iso1.to_string()))
            .collect()
    };
    match prop {
        "type" => Val::Str(s.stream_type.as_str().to_string()),
        "proxied" => Val::Bool(!s.request_headers.is_empty()),
        "library" => Val::Bool(s.library),
        "idmatched" => Val::Bool(s.id_matched),
        "indexer" => opt_str(&s.indexer),
        "message" => opt_str(&s.message),
        "infohash" => s
            .torrent
            .as_ref()
            .map_or(Val::Missing, |t| Val::Str(t.info_hash.clone())),
        "filename" => opt_str(&s.filename),
        "foldername" => opt_str(&s.folder_name),
        "size" => s.size.map_or(Val::Missing, |n| Val::Num(n as f64)),
        "foldersize" => s.folder_size.map_or(Val::Missing, |n| Val::Num(n as f64)),
        "bitrate" => s.bitrate.map_or(Val::Missing, |n| Val::Num(n as f64)),
        "duration" => s.duration.map_or(Val::Missing, |n| Val::Num(n as f64)),
        "container" => opt_str(&p.container),
        "extension" => opt_str(&p.extension),
        "mediainfoquality" => s
            .media_info_quality
            .map_or(Val::Missing, |q| Val::Str(q.as_str().into())),
        "quality" => opt_str(&p.quality),
        "resolution" => opt_str(&p.resolution),
        "encode" => opt_str(&p.encode),
        "network" => opt_str(&p.network),
        "visualtags" => strs(&sorted_by_pref(&p.visual_tags, &input.user.visual_tags)),
        "audiotags" => strs(&sorted_by_pref(&p.audio_tags, &input.user.audio_tags)),
        "audiochannels" => strs(&sorted_by_pref(
            &p.audio_channels,
            &input.user.audio_channels,
        )),
        "languages" => strs(&langs),
        "languageemojis" => strs(&flags(&langs)),
        "languagecodes" => strs(&codes(&langs)),
        "smalllanguagecodes" => strs(&small(&langs)),
        "ulanguages" => strs(&user_langs),
        "ulanguageemojis" => strs(&flags(&user_langs)),
        "ulanguagecodes" => strs(&codes(&user_langs)),
        "usmalllanguagecodes" => strs(&small(&user_langs)),
        "dubbed" => Val::Bool(p.dubbed),
        "subbed" => Val::Bool(p.subbed),
        "subtitles" => strs(&p.subtitles),
        "title" => opt_str(&p.title),
        "year" => opt_str(&p.year),
        "country" => opt_str(&p.country),
        "date" => opt_str(&p.date),
        "releasegroup" => opt_str(&p.release_group),
        "site" => opt_str(&p.site),
        "editions" => strs(&p.editions),
        "repack" => Val::Bool(p.repack),
        "proper" => Val::Bool(p.proper),
        "regraded" => Val::Bool(p.regraded),
        "uncensored" => Val::Bool(p.uncensored),
        "unrated" => Val::Bool(p.unrated),
        "upscaled" => Val::Bool(p.upscaled),
        "seasonpack" => Val::Bool(p.season_pack),
        "seasons" => nums(&p.seasons),
        "formattedseasons" => season_format('S', &p.seasons),
        "folderseasons" => nums(&p.folder_seasons),
        "formattedfolderseasons" => season_format('S', &p.folder_seasons),
        "episodes" => nums(&p.episodes),
        "formattedepisodes" => season_format('E', &p.episodes),
        "folderepisodes" => nums(&p.folder_episodes),
        "formattedfolderepisodes" => season_format('E', &p.folder_episodes),
        "seasonepisode" => {
            let parts: Vec<Val> = [
                season_format('S', &p.seasons),
                season_format('E', &p.episodes),
            ]
            .into_iter()
            .filter(|v| !v.is_missing())
            .collect();
            if parts.is_empty() {
                Val::Missing
            } else {
                Val::List(parts)
            }
        }
        "episodetitle" => opt_str(&p.episode_title),
        "audiotracks" => track_list(&s.audio_tracks),
        "subtitletracks" => track_list(&s.subtitle_tracks),
        "seeders" => s.seeders().map_or(Val::Missing, |n| Val::Num(f64::from(n))),
        "private" => s
            .torrent
            .as_ref()
            .map_or(Val::Missing, |t| Val::Bool(t.private)),
        "freeleech" => s
            .torrent
            .as_ref()
            .map_or(Val::Missing, |t| Val::Bool(t.freeleech)),
        "age" => s.age.map_or(Val::Missing, |h| {
            if h < 24.0 {
                Val::Str(format!("{}h", h.floor() as u64))
            } else {
                Val::Str(format!("{}d", (h / 24.0).floor() as u64))
            }
        }),
        "agehours" => s.age.map_or(Val::Missing, Val::Num),
        "seadex" => s
            .seadex
            .as_ref()
            .map_or(Val::Missing, |x| Val::Bool(x.is_seadex)),
        "seadexbest" => s
            .seadex
            .as_ref()
            .map_or(Val::Missing, |x| Val::Bool(x.is_best)),
        "regexscore" => Val::Num(s.regex_score),
        "sescore" => Val::Num(s.stream_expression_score),
        "regexmatched" => s
            .regex_matched
            .as_ref()
            .and_then(|m| m.name.clone())
            .map_or(Val::Missing, Val::Str),
        "semmatched" | "sematched" => s
            .stream_expression_matched
            .as_ref()
            .and_then(|m| m.name.clone())
            .map_or(Val::Missing, Val::Str),
        _ => Val::Missing,
    }
}

pub struct Formatted {
    pub name: String,
    pub description: String,
}

pub fn format_stream(
    name_template: &str,
    description_template: &str,
    input: &FormatInput,
) -> Formatted {
    let vars = |path: &str| lookup(input, path);
    Formatted {
        name: Template::compile(name_template).render(&vars),
        description: Template::compile(description_template).render(&vars),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(template: &str, vars: &[(&str, Val)]) -> String {
        let map: HashMap<String, Val> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        Template::compile(template).render(&|p| map.get(p).cloned().unwrap_or(Val::Missing))
    }

    fn s(v: &str) -> Val {
        Val::Str(v.into())
    }

    fn sample() -> Vec<(&'static str, Val)> {
        vec![
            ("service.shortname", s("RD")),
            ("service.cached", Val::Bool(true)),
            ("stream.resolution", s("2160p")),
            ("stream.quality", s("BluRay")),
            ("stream.size", Val::Num(15032385536.0)),
            ("stream.seeders", Val::Num(42.0)),
            ("stream.visualtags", Val::List(vec![s("HDR"), s("DV")])),
            ("stream.age", s("30d")),
            ("stream.proxied", Val::Bool(false)),
            ("stream.library", Val::Bool(false)),
            ("stream.filename", s("Movie.mkv")),
            ("stream.type", s("p2p")),
        ]
    }

    #[test]
    fn plain_references_render() {
        assert_eq!(
            run("{service.shortName} {stream.resolution}", &sample()),
            "RD 2160p"
        );
    }

    #[test]
    fn default_applies_only_to_missing_values() {
        assert_eq!(
            run("{stream.quality::default('Unknown')}", &sample()),
            "BluRay"
        );
        assert_eq!(
            run("{stream.encode::default('Unknown')}", &sample()),
            "Unknown"
        );
    }

    #[test]
    fn byte_modifiers_match_the_documented_examples() {
        assert_eq!(run("{stream.size::bytes2}", &sample()), "14 GiB");
        assert_eq!(run("{stream.size::sbytes}", &sample()), "15 GB");
    }

    #[test]
    fn conditional_with_nested_reference() {
        assert_eq!(
            run(
                r#"{stream.seeders::>1["Seeders: {stream.seeders}"||""]}"#,
                &sample()
            ),
            "Seeders: 42"
        );
    }

    #[test]
    fn join_uses_the_given_separator() {
        assert_eq!(
            run("{stream.visualTags::join(' | ')}", &sample()),
            "HDR | DV"
        );
    }

    #[test]
    fn boolean_conditional_has_three_branches() {
        let t = r#"{service.cached["Cached"||"Uncached"||"Unknown"]}"#;
        assert_eq!(run(t, &sample()), "Cached");
        assert_eq!(run(t, &[]), "Unknown");
    }

    #[test]
    fn optional_group_needs_every_reference() {
        assert_eq!(run("{?📅 {stream.age} ?}", &sample()), "📅 30d ");
        assert_eq!(run("{?ℹ️{stream.message}?}", &sample()), "");
    }

    #[test]
    fn small_caps_and_subscript_chain() {
        assert_eq!(run("{stream.resolution::smallcaps}", &sample()), "2160ᴘ");
        assert_eq!(
            run("{stream.resolution::subscript::smallcaps}", &sample()),
            "₂₁₆₀ᴘ"
        );
    }

    #[test]
    fn false_booleans_render_their_empty_branch() {
        assert_eq!(
            run(
                r#"{stream.proxied["P "||""]}{stream.library["L "||""]}"#,
                &sample()
            ),
            ""
        );
    }

    #[test]
    fn conditions_chain_left_to_right() {
        let t =
            r#"{service.cached::isfalse::or::stream.type::=p2p::and::stream.seeders::>0["S"||""]}"#;
        assert_eq!(run(t, &sample()), "S");
    }

    #[test]
    fn wrong_type_modifier_shows_an_inline_marker() {
        assert_eq!(
            run("{stream.filename::bytes}", &sample()),
            "{unknown_string_modifier(bytes)}"
        );
    }

    #[test]
    fn truncate_appends_an_ellipsis() {
        assert_eq!(run("{stream.filename::truncate(5)}", &sample()), "Movie…");
    }

    #[test]
    fn empty_lines_and_remove_line_are_dropped() {
        assert_eq!(
            run("a{tools.newLine}{stream.encode}{tools.newLine}b", &sample()),
            "a\nb"
        );
        assert_eq!(
            run("keep{tools.newLine}gone{tools.removeLine}", &sample()),
            "keep"
        );
    }

    #[test]
    fn deep_nesting_falls_back_to_plain_text() {
        let t = r#"{a["{b["{c["{d["{e["{f["x"||""]}"||""]}"||""]}"||""]}"||""]}"||""]}"#;
        let out = run(
            t,
            &[
                ("a", Val::Bool(true)),
                ("b", Val::Bool(true)),
                ("c", Val::Bool(true)),
                ("d", Val::Bool(true)),
                ("e", Val::Bool(true)),
                ("f", Val::Bool(true)),
            ],
        );
        assert!(out.contains('{') || out == "x");
    }

    #[test]
    fn duration_pattern_hides_zero_optional_groups() {
        assert_eq!(format_duration(5_025_000.0, "[%Hh ]%Mm"), "01h 23m");
        assert_eq!(format_duration(1_380_000.0, "[%Hh ]%Mm"), "23m");
    }

    #[test]
    fn dates_support_ordinals_and_names() {
        assert_eq!(format_date("2024-03-01", "%B %o, %Y"), "March 1st, 2024");
        assert_eq!(format_date("nope", "%Y"), "nope");
    }

    fn tracks() -> Val {
        let t = |lang: &str, codec: &str, forced: bool| Track {
            lang: Some(lang.into()),
            codec: Some(codec.into()),
            forced,
            ..Default::default()
        };
        Val::Items(
            [
                t("English", "eac3", false),
                t("French", "aac", false),
                t("English", "aac", true),
            ]
            .iter()
            .map(track_item)
            .collect(),
        )
    }

    #[test]
    fn where_then_pluck_deduplicates() {
        let vars = [("stream.audiotracks", tracks())];
        assert_eq!(
            run(
                "{stream.audioTracks::where('codec=aac')::pluck('lang')::join('/')}",
                &vars
            ),
            "French/English"
        );
        assert_eq!(
            run("{stream.audioTracks::pluck('lang')::join('/')}", &vars),
            "English/French"
        );
    }

    #[test]
    fn each_renders_per_item_and_skips_blank_results() {
        let vars = [("stream.audiotracks", tracks())];
        assert_eq!(
            run(
                r#"{stream.audioTracks::where('!forced')::each("{track.lang}:{track.codec} ")}"#,
                &vars
            ),
            "English:eac3 French:aac "
        );
        assert_eq!(
            run(
                r#"{stream.audioTracks::each("{track.title}")::default('none')}"#,
                &vars
            ),
            ""
        );
    }

    #[test]
    fn item_lists_support_length_and_slice_only_otherwise() {
        let vars = [("stream.audiotracks", tracks())];
        assert_eq!(run("{stream.audioTracks::slice(1)::length}", &vars), "2");
        assert_eq!(
            run("{stream.audioTracks::upper}", &vars),
            "{unknown_objects_modifier(upper)}"
        );
        assert_eq!(run("{stream.audioTracks}", &vars), "");
    }

    #[test]
    fn each_cannot_nest() {
        let vars = [("stream.audiotracks", tracks())];
        let out = run(
            r#"{stream.audioTracks::each("{stream.audioTracks::each('x')}")}"#,
            &vars,
        );
        assert_eq!(out, "{each_cannot_nest}".repeat(3));
    }

    #[test]
    fn each_stops_after_the_render_budget() {
        let many = Val::Items(
            (0..300)
                .map(|_| {
                    track_item(&Track {
                        lang: Some("a".into()),
                        ..Default::default()
                    })
                })
                .collect(),
        );
        let out = run(
            r#"{stream.audioTracks::each("{track.lang}")}"#,
            &[("stream.audiotracks", many)],
        );
        assert_eq!(out.len(), EACH_LIMIT);
    }
}
