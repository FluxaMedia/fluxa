use super::CompileError;
use super::funcs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Or,
    And,
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    In,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Num(f64),
    Str(String),
    Bool(bool),
    Const(String),
    Call(String, Vec<Expr>),
    Not(Box<Expr>),
    Neg(Box<Expr>),
    Binary(Op, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Str(String),
    Ident(String),
    LParen,
    RParen,
    Comma,
    Question,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    End,
}

fn lex(src: &str) -> Result<(Vec<Tok>, Vec<String>), CompileError> {
    let chars: Vec<char> = src.chars().collect();
    let mut toks = Vec::new();
    let mut names = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match c {
            c if c.is_whitespace() => i += 1,
            '/' if next == Some('*') => {
                let start = i + 2;
                let mut end = start;
                while end + 1 < chars.len() && !(chars[end] == '*' && chars[end + 1] == '/') {
                    end += 1;
                }
                if end + 1 >= chars.len() {
                    return Err(CompileError::new("unterminated comment"));
                }
                let text: String = chars[start..end].iter().collect();
                let text = text.trim();
                if !text.is_empty() && !text.starts_with('#') {
                    names.push(text.to_string());
                }
                i = end + 2;
            }
            '0'..='9' | '.' if c != '.' || next.is_some_and(|n| n.is_ascii_digit()) => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let n = text
                    .parse::<f64>()
                    .map_err(|_| CompileError::new(format!("bad number '{text}'")))?;
                toks.push(Tok::Num(n));
            }
            '\'' | '"' => {
                let mut out = String::new();
                i += 1;
                loop {
                    match chars.get(i) {
                        None => return Err(CompileError::new("unterminated string")),
                        Some('\\') => {
                            if let Some(&e) = chars.get(i + 1) {
                                out.push(e);
                            }
                            i += 2;
                        }
                        Some(&q) if q == c => {
                            i += 1;
                            break;
                        }
                        Some(&other) => {
                            out.push(other);
                            i += 1;
                        }
                    }
                }
                toks.push(Tok::Str(out));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                toks.push(Tok::Ident(chars[start..i].iter().collect()));
            }
            '(' => {
                toks.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                toks.push(Tok::RParen);
                i += 1;
            }
            ',' => {
                toks.push(Tok::Comma);
                i += 1;
            }
            '?' => {
                toks.push(Tok::Question);
                i += 1;
            }
            ':' => {
                toks.push(Tok::Colon);
                i += 1;
            }
            '+' => {
                toks.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                toks.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                toks.push(Tok::Star);
                i += 1;
            }
            '/' => {
                toks.push(Tok::Slash);
                i += 1;
            }
            '=' if next == Some('=') => {
                toks.push(Tok::Eq);
                i += 2;
            }
            '!' if next == Some('=') => {
                toks.push(Tok::Ne);
                i += 2;
            }
            '>' if next == Some('=') => {
                toks.push(Tok::Ge);
                i += 2;
            }
            '<' if next == Some('=') => {
                toks.push(Tok::Le);
                i += 2;
            }
            '>' => {
                toks.push(Tok::Gt);
                i += 1;
            }
            '<' => {
                toks.push(Tok::Lt);
                i += 1;
            }
            other => return Err(CompileError::new(format!("unexpected '{other}'"))),
        }
    }
    toks.push(Tok::End);
    Ok((toks, names))
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        self.toks.get(self.pos).unwrap_or(&Tok::End)
    }

    fn bump(&mut self) -> Tok {
        let t = self.peek().clone();
        self.pos += 1;
        t
    }

    fn word(&self, w: &str) -> bool {
        matches!(self.peek(), Tok::Ident(i) if i == w)
    }

    fn expect(&mut self, t: Tok) -> Result<(), CompileError> {
        if *self.peek() == t {
            self.pos += 1;
            Ok(())
        } else {
            Err(CompileError::new(format!(
                "expected {t:?}, found {:?}",
                self.peek()
            )))
        }
    }

    fn ternary(&mut self) -> Result<Expr, CompileError> {
        let cond = self.or()?;
        if *self.peek() != Tok::Question {
            return Ok(cond);
        }
        self.pos += 1;
        let yes = self.ternary()?;
        self.expect(Tok::Colon)?;
        let no = self.ternary()?;
        Ok(Expr::Ternary(Box::new(cond), Box::new(yes), Box::new(no)))
    }

    fn or(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.and()?;
        while self.word("or") {
            self.pos += 1;
            left = Expr::Binary(Op::Or, Box::new(left), Box::new(self.and()?));
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.not()?;
        while self.word("and") {
            self.pos += 1;
            left = Expr::Binary(Op::And, Box::new(left), Box::new(self.not()?));
        }
        Ok(left)
    }

    fn not(&mut self) -> Result<Expr, CompileError> {
        if self.word("not") {
            self.pos += 1;
            return Ok(Expr::Not(Box::new(self.not()?)));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.additive()?;
        loop {
            let op = match self.peek() {
                Tok::Eq => Op::Eq,
                Tok::Ne => Op::Ne,
                Tok::Gt => Op::Gt,
                Tok::Lt => Op::Lt,
                Tok::Ge => Op::Ge,
                Tok::Le => Op::Le,
                Tok::Ident(w) if w == "in" => Op::In,
                _ => return Ok(left),
            };
            self.pos += 1;
            left = Expr::Binary(op, Box::new(left), Box::new(self.additive()?));
        }
    }

    fn additive(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.multiplicative()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => Op::Add,
                Tok::Minus => Op::Sub,
                _ => return Ok(left),
            };
            self.pos += 1;
            left = Expr::Binary(op, Box::new(left), Box::new(self.multiplicative()?));
        }
    }

    fn multiplicative(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => Op::Mul,
                Tok::Slash => Op::Div,
                _ => return Ok(left),
            };
            self.pos += 1;
            left = Expr::Binary(op, Box::new(left), Box::new(self.unary()?));
        }
    }

    fn unary(&mut self) -> Result<Expr, CompileError> {
        if *self.peek() == Tok::Minus {
            self.pos += 1;
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, CompileError> {
        match self.bump() {
            Tok::Num(n) => Ok(Expr::Num(n)),
            Tok::Str(s) => Ok(Expr::Str(s)),
            Tok::LParen => {
                let inner = self.ternary()?;
                self.expect(Tok::RParen)?;
                Ok(inner)
            }
            Tok::Ident(name) => match name.as_str() {
                "true" => Ok(Expr::Bool(true)),
                "false" => Ok(Expr::Bool(false)),
                _ if *self.peek() == Tok::LParen => {
                    self.pos += 1;
                    let mut args = Vec::new();
                    if *self.peek() != Tok::RParen {
                        loop {
                            args.push(self.ternary()?);
                            if *self.peek() == Tok::Comma {
                                self.pos += 1;
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(Tok::RParen)?;
                    let (min, max) = funcs::arity(&name)
                        .ok_or_else(|| CompileError::new(format!("unknown function '{name}'")))?;
                    if args.len() < min || args.len() > max {
                        return Err(CompileError::new(format!(
                            "'{name}' takes {min}..{} arguments, got {}",
                            if max == usize::MAX {
                                "n".to_string()
                            } else {
                                max.to_string()
                            },
                            args.len()
                        )));
                    }
                    Ok(Expr::Call(funcs::canonical(&name).to_string(), args))
                }
                _ if funcs::is_constant(&name) => Ok(Expr::Const(name)),
                _ => Err(CompileError::new(format!("unknown constant '{name}'"))),
            },
            other => Err(CompileError::new(format!("unexpected {other:?}"))),
        }
    }
}

pub fn parse(src: &str) -> Result<(Expr, Vec<String>), CompileError> {
    let (toks, names) = lex(src)?;
    let mut p = Parser { toks, pos: 0 };
    let expr = p.ternary()?;
    if *p.peek() != Tok::End {
        return Err(CompileError::new(format!("trailing {:?}", p.peek())));
    }
    Ok((expr, names))
}
