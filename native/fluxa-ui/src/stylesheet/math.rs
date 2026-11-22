use crate::gfx::Vec2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Env {
    pub percent: Option<f32>,
    pub view: Option<Vec2>,
    pub rem: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    Num(f64),
    Px(f64),
    Rem(f64),
    Percent(f64),
    Vw(f64),
    Vh(f64),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Min(Vec<Expr>),
    Max(Vec<Expr>),
    Clamp(Box<Expr>, Box<Expr>, Box<Expr>),
}

impl Expr {
    pub(crate) fn parse(text: &str) -> Option<Expr> {
        let mut parser = Parser {
            text: text.trim().as_bytes(),
            at: 0,
        };
        let expr = parser.sum()?;
        parser.skip_space();
        (parser.at == parser.text.len()).then_some(expr)
    }

    pub(crate) fn eval(&self, env: &Env) -> Option<f32> {
        self.value(env).map(|value| value as f32)
    }

    pub(crate) fn needs_env(&self) -> bool {
        self.any(&|expr| {
            matches!(
                expr,
                Expr::Rem(_) | Expr::Percent(_) | Expr::Vw(_) | Expr::Vh(_)
            )
        })
    }

    pub(crate) fn uses_percent(&self) -> bool {
        self.any(&|expr| matches!(expr, Expr::Percent(_)))
    }

    fn any(&self, found: &dyn Fn(&Expr) -> bool) -> bool {
        if found(self) {
            return true;
        }
        match self {
            Expr::Neg(inner) => inner.any(found),
            Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b) | Expr::Div(a, b) => {
                a.any(found) || b.any(found)
            }
            Expr::Min(items) | Expr::Max(items) => items.iter().any(|item| item.any(found)),
            Expr::Clamp(low, mid, high) => low.any(found) || mid.any(found) || high.any(found),
            _ => false,
        }
    }

    fn value(&self, env: &Env) -> Option<f64> {
        Some(match self {
            Expr::Num(value) | Expr::Px(value) => *value,
            Expr::Rem(value) => value * f64::from(env.rem),
            Expr::Percent(value) => value * f64::from(env.percent?) / 100.0,
            Expr::Vw(value) => value * f64::from(env.view?.x) / 100.0,
            Expr::Vh(value) => value * f64::from(env.view?.y) / 100.0,
            Expr::Neg(inner) => -inner.value(env)?,
            Expr::Add(a, b) => a.value(env)? + b.value(env)?,
            Expr::Sub(a, b) => a.value(env)? - b.value(env)?,
            Expr::Mul(a, b) => a.value(env)? * b.value(env)?,
            Expr::Div(a, b) => a.value(env)? / b.value(env)?,
            Expr::Min(items) => items
                .iter()
                .map(|item| item.value(env))
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .fold(f64::INFINITY, f64::min),
            Expr::Max(items) => items
                .iter()
                .map(|item| item.value(env))
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .fold(f64::NEG_INFINITY, f64::max),
            Expr::Clamp(low, mid, high) => {
                let (low, mid, high) = (low.value(env)?, mid.value(env)?, high.value(env)?);
                low.max(mid.min(high))
            }
        })
    }
}

struct Parser<'a> {
    text: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn skip_space(&mut self) {
        while self.text.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at += 1;
        }
    }

    fn eat(&mut self, byte: u8) -> bool {
        self.skip_space();
        let found = self.text.get(self.at) == Some(&byte);
        if found {
            self.at += 1;
        }
        found
    }

    fn sum(&mut self) -> Option<Expr> {
        let mut total = self.product()?;
        loop {
            total = if self.eat(b'+') {
                Expr::Add(Box::new(total), Box::new(self.product()?))
            } else if self.eat(b'-') {
                Expr::Sub(Box::new(total), Box::new(self.product()?))
            } else {
                return Some(total);
            };
        }
    }

    fn product(&mut self) -> Option<Expr> {
        let mut total = self.unary()?;
        loop {
            total = if self.eat(b'*') {
                Expr::Mul(Box::new(total), Box::new(self.unary()?))
            } else if self.eat(b'/') {
                Expr::Div(Box::new(total), Box::new(self.unary()?))
            } else {
                return Some(total);
            };
        }
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.eat(b'-') {
            return Some(Expr::Neg(Box::new(self.unary()?)));
        }
        if self.eat(b'+') {
            return self.unary();
        }
        self.atom()
    }

    fn atom(&mut self) -> Option<Expr> {
        self.skip_space();
        if self.eat(b'(') {
            let inner = self.sum()?;
            return self.eat(b')').then_some(inner);
        }
        let start = self.at;
        while self
            .text
            .get(self.at)
            .is_some_and(|byte| byte.is_ascii_alphabetic())
        {
            self.at += 1;
        }
        if self.at > start {
            let name = std::str::from_utf8(&self.text[start..self.at]).ok()?;
            return self.call(name);
        }
        self.number()
    }

    fn call(&mut self, name: &str) -> Option<Expr> {
        if !self.eat(b'(') {
            return None;
        }
        let mut args = vec![self.sum()?];
        while self.eat(b',') {
            args.push(self.sum()?);
        }
        if !self.eat(b')') {
            return None;
        }
        match (name, args.len()) {
            ("calc", 1) => args.pop(),
            ("min", 1..) => Some(Expr::Min(args)),
            ("max", 1..) => Some(Expr::Max(args)),
            ("clamp", 3) => {
                let high = args.pop()?;
                let mid = args.pop()?;
                Some(Expr::Clamp(
                    Box::new(args.pop()?),
                    Box::new(mid),
                    Box::new(high),
                ))
            }
            _ => None,
        }
    }

    fn number(&mut self) -> Option<Expr> {
        let start = self.at;
        while self
            .text
            .get(self.at)
            .is_some_and(|byte| byte.is_ascii_digit() || *byte == b'.')
        {
            self.at += 1;
        }
        let value: f64 = std::str::from_utf8(&self.text[start..self.at])
            .ok()?
            .parse()
            .ok()?;
        let unit_start = self.at;
        while self
            .text
            .get(self.at)
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'%')
        {
            self.at += 1;
        }
        match &self.text[unit_start..self.at] {
            b"" => Some(Expr::Num(value)),
            b"px" => Some(Expr::Px(value)),
            b"rem" => Some(Expr::Rem(value)),
            b"%" => Some(Expr::Percent(value)),
            b"vw" => Some(Expr::Vw(value)),
            b"vh" => Some(Expr::Vh(value)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(text: &str, env: Env) -> Option<f32> {
        Expr::parse(text)?.eval(&env)
    }

    fn env() -> Env {
        Env {
            percent: Some(200.0),
            view: Some(Vec2::new(1000.0, 500.0)),
            rem: 16.0,
        }
    }

    #[test]
    fn units_mix_in_one_expression() {
        assert_eq!(
            eval("calc(50% + 10vw - 2rem + 4px)", env()),
            Some(100.0 + 100.0 - 32.0 + 4.0)
        );
        assert_eq!(eval("calc(10vh * 2)", env()), Some(100.0));
    }

    #[test]
    fn precedence_and_nesting_follow_css() {
        assert_eq!(eval("calc(2px + 3px * 4)", env()), Some(14.0));
        assert_eq!(eval("calc((2px + 3px) * 4)", env()), Some(20.0));
        assert_eq!(eval("calc(100px / 4 - -5px)", env()), Some(30.0));
    }

    #[test]
    fn min_max_clamp_take_expressions() {
        assert_eq!(eval("min(100%, 150px)", env()), Some(150.0));
        assert_eq!(eval("max(10vw, 50% - 20px)", env()), Some(100.0));
        assert_eq!(eval("max(5vw, 50% - 20px)", env()), Some(80.0));
        assert_eq!(eval("clamp(100px, 20vw, 600px)", env()), Some(200.0));
        assert_eq!(eval("clamp(100px, 1vw, 600px)", env()), Some(100.0));
        assert_eq!(eval("clamp(100px, 90vw, 600px)", env()), Some(600.0));
        assert_eq!(eval("min(10px, max(2px, 5px), 40px)", env()), Some(5.0));
    }

    #[test]
    fn bare_sums_parse_for_variable_values() {
        assert_eq!(eval("50% + 20px", env()), Some(120.0));
        assert_eq!(eval("14px", env()), Some(14.0));
    }

    #[test]
    fn clamp_prefers_min_when_bounds_cross() {
        assert_eq!(eval("clamp(30px, 10px, 20px)", env()), Some(30.0));
    }

    #[test]
    fn missing_context_is_none() {
        let bare = Env {
            rem: 16.0,
            ..Env::default()
        };
        assert_eq!(eval("calc(50% + 1px)", bare), None);
        assert_eq!(eval("calc(10vw)", bare), None);
        assert_eq!(eval("calc(1rem + 1px)", bare), Some(17.0));
    }

    #[test]
    fn static_expressions_need_no_context() {
        assert!(!Expr::parse("calc(2px * 3)").unwrap().needs_env());
        assert!(Expr::parse("calc(2rem * 3)").unwrap().needs_env());
        assert!(Expr::parse("min(100%, 3px)").unwrap().uses_percent());
        assert!(!Expr::parse("min(100vw, 3px)").unwrap().uses_percent());
    }

    #[test]
    fn malformed_text_is_rejected() {
        for text in [
            "calc(1px +)",
            "min()",
            "clamp(1px, 2px)",
            "calc(1px",
            "2 em",
            "foo(1px)",
        ] {
            assert!(Expr::parse(text).is_none(), "{text}");
        }
    }
}
