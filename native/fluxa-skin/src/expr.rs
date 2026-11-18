use serde_json::Value;
use std::cmp::Ordering;

const MAX_DEPTH: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(Value),
    Path(Vec<Segment>),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Compare(Op, Box<Expr>, Box<Expr>),
    Len(Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub type Reads = Vec<(String, u64)>;

pub struct Scope<'a> {
    data: &'a Value,
    vars: Vec<(String, Value)>,
    frames: std::cell::RefCell<Vec<Reads>>,
}

fn fingerprint(value: Option<&Value>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    match value {
        Some(value) => {
            1u8.hash(&mut hasher);
            value.to_string().hash(&mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }
    hasher.finish()
}

fn walk<'v>(root: &'v Value, rest: &[Segment]) -> Option<&'v Value> {
    rest.iter().try_fold(root, |value, segment| match segment {
        Segment::Key(key) => value.get(key),
        Segment::Index(index) => value.get(*index),
    })
}

fn path_name(first: &str, rest: &[Segment]) -> String {
    let mut name = first.to_owned();
    for segment in rest {
        match segment {
            Segment::Key(key) => {
                name.push('.');
                name.push_str(key);
            }
            Segment::Index(index) => name.push_str(&format!("[{index}]")),
        }
    }
    name
}

impl<'a> Scope<'a> {
    pub fn new(data: &'a Value) -> Self {
        Self {
            data,
            vars: Vec::new(),
            frames: Default::default(),
        }
    }

    pub fn push(&mut self, name: &str, value: Value) {
        self.vars.push((name.to_owned(), value));
    }

    pub fn pop(&mut self) {
        self.vars.pop();
    }

    pub fn signature(&self, item: &Value) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for (name, value) in &self.vars {
            name.hash(&mut hasher);
            value.to_string().hash(&mut hasher);
        }
        item.to_string().hash(&mut hasher);
        hasher.finish()
    }

    pub fn begin_reads(&self) {
        self.frames.borrow_mut().push(Vec::new());
    }

    pub fn end_reads(&self) -> Reads {
        let mut frames = self.frames.borrow_mut();
        let reads = frames.pop().unwrap_or_default();
        if let Some(parent) = frames.last_mut() {
            parent.extend(reads.iter().cloned());
        }
        reads
    }

    pub fn reads_unchanged(&self, reads: &Reads) -> bool {
        reads.iter().all(|(name, hash)| {
            let mut parts = name.split('.');
            let first = parts.next().unwrap_or_default();
            if self.vars.iter().any(|(var, _)| var == first) {
                return true;
            }
            let value = self.data.get(first).and_then(|root| {
                parts.try_fold(root, |value, part| match part.split_once('[') {
                    Some((key, index)) => {
                        let index = index.trim_end_matches(']').parse::<usize>().ok()?;
                        let value = if key.is_empty() { value } else { value.get(key)? };
                        value.get(index)
                    }
                    None => value.get(part),
                })
            });
            fingerprint(value) == *hash
        })
    }

    fn record(&self, first: &str, rest: &[Segment], value: Option<&Value>) {
        if let Some(frame) = self.frames.borrow_mut().last_mut() {
            frame.push((path_name(first, rest), fingerprint(value)));
        }
    }

    fn data_root(&self, key: &str) -> Option<&'a Value> {
        if self.vars.iter().any(|(name, _)| name == key) {
            return None;
        }
        self.data.get(key)
    }

    fn root(&self, key: &str) -> Option<&Value> {
        self.vars
            .iter()
            .rev()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
            .or_else(|| self.data.get(key))
    }

    fn is_var(&self, key: &str) -> bool {
        self.vars.iter().any(|(name, _)| name == key)
    }
}

impl Expr {
    pub fn parse(source: &str) -> Result<Expr, String> {
        let mut parser = Parser {
            tokens: tokenize(source)?,
            pos: 0,
            depth: 0,
        };
        let expr = parser.or()?;
        match parser.tokens.get(parser.pos) {
            None => Ok(expr),
            Some(token) => Err(format!("unexpected {token:?}")),
        }
    }

    pub fn borrow<'a>(&self, scope: &Scope<'a>) -> Option<&'a Value> {
        let Expr::Path(segments) = self else {
            return None;
        };
        let (Segment::Key(first), rest) = segments.split_first()? else {
            return None;
        };
        let found = scope
            .data_root(first)
            .and_then(|root| walk(root, rest));
        scope.record(first, rest, found);
        found
    }

    pub fn eval(&self, scope: &Scope) -> Value {
        match self {
            Expr::Literal(value) => value.clone(),
            Expr::Path(segments) => lookup(segments, scope).cloned().unwrap_or(Value::Null),
            Expr::Not(inner) => Value::Bool(!truthy(&inner.eval(scope))),
            Expr::And(left, right) => {
                Value::Bool(truthy(&left.eval(scope)) && truthy(&right.eval(scope)))
            }
            Expr::Or(left, right) => {
                Value::Bool(truthy(&left.eval(scope)) || truthy(&right.eval(scope)))
            }
            Expr::Compare(op, left, right) => {
                Value::Bool(compare(*op, &left.eval(scope), &right.eval(scope)))
            }
            Expr::Len(inner) if matches!(**inner, Expr::Path(_)) => {
                let Expr::Path(segments) = &**inner else {
                    return Value::from(0);
                };
                match lookup(segments, scope) {
                    Some(Value::Array(items)) => Value::from(items.len()),
                    Some(Value::String(text)) => Value::from(text.chars().count()),
                    Some(Value::Object(map)) => Value::from(map.len()),
                    _ => Value::from(0),
                }
            }
            Expr::Len(inner) => match inner.eval(scope) {
                Value::Array(items) => Value::from(items.len()),
                Value::String(text) => Value::from(text.chars().count()),
                Value::Object(map) => Value::from(map.len()),
                _ => Value::from(0),
            },
        }
    }
}

fn lookup<'a>(segments: &[Segment], scope: &'a Scope) -> Option<&'a Value> {
    let (Segment::Key(first), rest) = segments.split_first()? else {
        return None;
    };
    let found = scope.root(first).and_then(|root| walk(root, rest));
    if !scope.is_var(first) {
        scope.record(first, rest, found);
    }
    found
}

pub fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|n| n != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => !map.is_empty(),
    }
}

pub fn display(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        _ => String::new(),
    }
}

fn compare(op: Op, left: &Value, right: &Value) -> bool {
    let numbers = left.as_f64().zip(right.as_f64());
    match op {
        Op::Eq => numbers.map_or(left == right, |(a, b)| a == b),
        Op::Ne => !compare(Op::Eq, left, right),
        _ => {
            let ordering = match (numbers, left, right) {
                (Some((a, b)), _, _) => a.partial_cmp(&b),
                (None, Value::String(a), Value::String(b)) => Some(a.cmp(b)),
                _ => None,
            };
            matches!(
                (op, ordering),
                (Op::Lt, Some(Ordering::Less))
                    | (Op::Le, Some(Ordering::Less | Ordering::Equal))
                    | (Op::Gt, Some(Ordering::Greater))
                    | (Op::Ge, Some(Ordering::Greater | Ordering::Equal))
            )
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Ident(String),
    Number(f64),
    Str(String),
    Sym(&'static str),
}

const TWO_CHAR: [&str; 6] = ["==", "!=", "<=", ">=", "&&", "||"];
const ONE_CHAR: [&str; 7] = ["!", "<", ">", "(", ")", ".", "["];

fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit()
            || (c == '-' && chars.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let number = text.parse().map_err(|_| format!("bad number {text}"))?;
            tokens.push(Token::Number(number));
        } else if c == '"' || c == '\'' {
            let end = chars[i + 1..]
                .iter()
                .position(|&ch| ch == c)
                .ok_or("unterminated string")?;
            tokens.push(Token::Str(chars[i + 1..i + 1 + end].iter().collect()));
            i += end + 2;
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(Token::Ident(chars[start..i].iter().collect()));
        } else if c == ']' {
            tokens.push(Token::Sym("]"));
            i += 1;
        } else {
            let pair: String = chars[i..chars.len().min(i + 2)].iter().collect();
            if let Some(sym) = TWO_CHAR.iter().find(|sym| **sym == pair) {
                tokens.push(Token::Sym(sym));
                i += 2;
            } else if let Some(sym) = ONE_CHAR.iter().find(|sym| sym.starts_with(c)) {
                tokens.push(Token::Sym(sym));
                i += 1;
            } else {
                return Err(format!("unexpected character {c:?}"));
            }
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
}

impl Parser {
    fn eat(&mut self, sym: &str) -> bool {
        let hit = matches!(self.tokens.get(self.pos), Some(Token::Sym(s)) if *s == sym);
        if hit {
            self.pos += 1;
        }
        hit
    }

    fn enter(&mut self) -> Result<(), String> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err("expression nested too deeply".into());
        }
        Ok(())
    }

    fn or(&mut self) -> Result<Expr, String> {
        self.enter()?;
        let mut left = self.and()?;
        while self.eat("||") {
            left = Expr::Or(Box::new(left), Box::new(self.and()?));
        }
        self.depth -= 1;
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut left = self.comparison()?;
        while self.eat("&&") {
            left = Expr::And(Box::new(left), Box::new(self.comparison()?));
        }
        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let left = self.unary()?;
        let op = [
            ("==", Op::Eq),
            ("!=", Op::Ne),
            ("<=", Op::Le),
            (">=", Op::Ge),
            ("<", Op::Lt),
            (">", Op::Gt),
        ]
        .into_iter()
        .find(|(sym, _)| self.eat(sym))
        .map(|(_, op)| op);
        match op {
            Some(op) => Ok(Expr::Compare(op, Box::new(left), Box::new(self.unary()?))),
            None => Ok(left),
        }
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.eat("!") {
            self.enter()?;
            let inner = self.unary()?;
            self.depth -= 1;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.tokens.get(self.pos).cloned().ok_or("unexpected end")?;
        self.pos += 1;
        match token {
            Token::Number(number) => Ok(Expr::Literal(Value::from(number))),
            Token::Str(text) => Ok(Expr::Literal(Value::String(text))),
            Token::Sym("(") => {
                let inner = self.or()?;
                if !self.eat(")") {
                    return Err("missing )".into());
                }
                Ok(inner)
            }
            Token::Ident(name) => match name.as_str() {
                "true" => Ok(Expr::Literal(Value::Bool(true))),
                "false" => Ok(Expr::Literal(Value::Bool(false))),
                "null" => Ok(Expr::Literal(Value::Null)),
                "len" if self.eat("(") => {
                    let inner = self.or()?;
                    if !self.eat(")") {
                        return Err("missing )".into());
                    }
                    Ok(Expr::Len(Box::new(inner)))
                }
                _ => self.path(name),
            },
            other => Err(format!("unexpected {other:?}")),
        }
    }

    fn path(&mut self, first: String) -> Result<Expr, String> {
        let mut segments = vec![Segment::Key(first)];
        loop {
            if self.eat(".") {
                match self.tokens.get(self.pos) {
                    Some(Token::Ident(key)) => segments.push(Segment::Key(key.clone())),
                    _ => return Err("expected a name after .".into()),
                }
                self.pos += 1;
            } else if self.eat("[") {
                match self.tokens.get(self.pos) {
                    Some(Token::Number(n)) if *n >= 0.0 && n.fract() == 0.0 => {
                        segments.push(Segment::Index(*n as usize))
                    }
                    _ => return Err("expected an index inside []".into()),
                }
                self.pos += 1;
                if !self.eat("]") {
                    return Err("missing ]".into());
                }
            } else {
                return Ok(Expr::Path(segments));
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    parts: Vec<Part>,
}

#[derive(Clone, Debug, PartialEq)]
enum Part {
    Text(String),
    Expr(Expr),
}

thread_local! {
    static PARSED: std::cell::RefCell<std::collections::HashMap<String, Option<std::rc::Rc<Template>>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

const PARSED_LIMIT: usize = 8192;

impl Template {
    pub fn parse_shared(source: &str) -> Option<std::rc::Rc<Template>> {
        PARSED.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(found) = cache.get(source) {
                return found.clone();
            }
            if cache.len() >= PARSED_LIMIT {
                cache.clear();
            }
            let parsed = Template::parse(source).ok().map(std::rc::Rc::new);
            cache.insert(source.to_owned(), parsed.clone());
            parsed
        })
    }

    pub fn parse(source: &str) -> Result<Template, String> {
        let chars: Vec<char> = source.chars().collect();
        let mut parts = Vec::new();
        let mut text = String::new();
        let mut i = 0;
        while i < chars.len() {
            match chars[i] {
                '\\' if chars.get(i + 1) == Some(&'{') => {
                    text.push('{');
                    i += 2;
                }
                '{' => {
                    let mut end = i + 1;
                    let mut quote = None;
                    while end < chars.len() {
                        match (quote, chars[end]) {
                            (None, '}') => break,
                            (None, c @ ('"' | '\'')) => quote = Some(c),
                            (Some(q), c) if c == q => quote = None,
                            _ => {}
                        }
                        end += 1;
                    }
                    if end >= chars.len() {
                        return Err("missing } in template".into());
                    }
                    let inner: String = chars[i + 1..end].iter().collect();
                    if !text.is_empty() {
                        parts.push(Part::Text(std::mem::take(&mut text)));
                    }
                    parts.push(Part::Expr(Expr::parse(&inner)?));
                    i = end + 1;
                }
                c => {
                    text.push(c);
                    i += 1;
                }
            }
        }
        if !text.is_empty() {
            parts.push(Part::Text(text));
        }
        Ok(Template { parts })
    }

    pub fn render(&self, scope: &Scope) -> String {
        self.parts
            .iter()
            .map(|part| match part {
                Part::Text(text) => text.clone(),
                Part::Expr(expr) => display(&expr.eval(scope)),
            })
            .collect()
    }

    pub fn value(&self, scope: &Scope) -> Value {
        match self.parts.as_slice() {
            [Part::Expr(expr)] => expr.eval(scope),
            _ => Value::String(self.render(scope)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn eval(source: &str, data: Value) -> Value {
        Expr::parse(source).unwrap().eval(&Scope::new(&data))
    }

    #[test]
    fn missing_paths_are_null_not_errors() {
        assert_eq!(
            eval("home.nothing.deeper", json!({"home": {}})),
            Value::Null
        );
    }

    #[test]
    fn numbers_compare_across_integer_and_float() {
        assert_eq!(
            eval("card.progress >= 1", json!({"card": {"progress": 1.0}})),
            json!(true)
        );
        assert_eq!(eval("n == 2", json!({"n": 2})), json!(true));
    }

    #[test]
    fn precedence_binds_and_tighter_than_or() {
        assert_eq!(
            eval("a || b && c", json!({"a": true, "b": false, "c": false})),
            json!(true)
        );
    }

    #[test]
    fn len_counts_arrays_and_strings() {
        assert_eq!(eval("len(rows)", json!({"rows": [1, 2, 3]})), json!(3));
        assert_eq!(eval("len(name) > 2", json!({"name": "abc"})), json!(true));
    }

    #[test]
    fn index_paths_read_array_slots() {
        assert_eq!(
            eval(
                "rows[1].title",
                json!({"rows": [{"title": "a"}, {"title": "b"}]})
            ),
            json!("b")
        );
    }

    #[test]
    fn template_keeps_type_for_single_expression() {
        let data = json!({"card": {"progress": 0.5, "title": "T"}});
        let scope = Scope::new(&data);
        assert_eq!(
            Template::parse("{card.progress}").unwrap().value(&scope),
            json!(0.5)
        );
        assert_eq!(
            Template::parse("Up next: {card.title}")
                .unwrap()
                .value(&scope),
            json!("Up next: T")
        );
    }

    #[test]
    fn escaped_brace_stays_literal() {
        let data = json!({});
        assert_eq!(
            Template::parse("\\{x}").unwrap().render(&Scope::new(&data)),
            "{x}"
        );
    }

    #[test]
    fn closing_brace_inside_a_string_does_not_end_the_binding() {
        let data = json!({"a": "}"});
        assert_eq!(
            Template::parse("{a == '}'}")
                .unwrap()
                .value(&Scope::new(&data)),
            json!(true)
        );
    }

    #[test]
    fn deep_nesting_is_rejected() {
        let source = format!("{}1{}", "(".repeat(200), ")".repeat(200));
        assert!(Expr::parse(&source).is_err());
    }

    #[test]
    fn bad_input_reports_instead_of_panicking() {
        for source in ["", "a ==", "a.", "a[", "'x", "a & b", "len(", "1..2"] {
            assert!(Expr::parse(source).is_err(), "{source}");
        }
    }

    #[test]
    fn scope_variables_shadow_data() {
        let data = json!({"row": "data"});
        let mut scope = Scope::new(&data);
        scope.push("row", json!("var"));
        assert_eq!(Expr::parse("row").unwrap().eval(&scope), json!("var"));
        scope.pop();
        assert_eq!(Expr::parse("row").unwrap().eval(&scope), json!("data"));
    }
}
