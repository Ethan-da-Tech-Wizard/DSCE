//! Deterministic, serializable `compute` hooks for rules.
//!
//! The Python prototype attached raw lambdas to rules and `eval()`ed their
//! source when loading from SQLite — unserializable and unsafe. Here a
//! compute hook is either:
//!
//! - [`Compute::Expr`]: a tiny arithmetic DSL, parsed once and evaluated
//!   deterministically. This is the form that round-trips through the
//!   database. Syntax: semicolon-separated assignments of variables to
//!   arithmetic expressions over already-bound variables and literals:
//!
//!   ```text
//!       ?a = ?w * ?h
//!       ?next = ?v + 1
//!       ?area = ?w * ?h; ?perimeter = 2 * (?w + ?h)
//!   ```
//!
//! - [`Compute::Native`]: an arbitrary Rust closure for callers embedding
//!   the engine as a library. Not serializable (`source()` returns `None`),
//!   so it cannot be stored in a database — but it is `Send + Sync` and
//!   fires fine from the rayon-parallel rule loop. The closure MUST be pure
//!   and deterministic or the engine's determinism guarantee is broken.
//!
//! The evaluator supports `+ - * /`, unary minus, parentheses, integer,
//! float, and quoted string literals (`'...'` or `"..."` with `\n`, `\t`,
//! `\\`, `\'`, `\"` escapes), and `?variables`. Int op Int stays Int
//! (except `/`, which always yields a float, mirroring Python's true
//! division); any float operand promotes the result to float. When either
//! side of `+` is a STRING the operator concatenates — this is what lets
//! the software assembler splice code fragments with rules like
//! `?code = ?code1 + ?code2`. Division by zero, arithmetic on strings, or
//! an unbound variable makes evaluation fail, which the engine treats as
//! "this rule cannot conclude anything for this match".

use std::fmt;
use std::sync::Arc;

use crate::facts::{Bindings, Term};

/// A rule's optional deterministic computation from bindings to EXTRA
/// bindings — what lets a conclusion contain a value no premise supplied.
#[derive(Clone)]
pub enum Compute {
    /// Parsed arithmetic program; serializable via [`Compute::source`].
    Expr(ExprProgram),
    /// Arbitrary pure closure; library-only, not serializable.
    Native(Arc<dyn Fn(&Bindings) -> Bindings + Send + Sync>),
}

impl Compute {
    /// Parse an expression program, e.g. `"?a = ?w * ?h"`.
    pub fn expr(source: &str) -> Result<Compute, String> {
        Ok(Compute::Expr(ExprProgram::parse(source)?))
    }

    /// Wrap a native closure.
    pub fn native(f: impl Fn(&Bindings) -> Bindings + Send + Sync + 'static) -> Compute {
        Compute::Native(Arc::new(f))
    }

    /// Evaluate against the matched bindings, returning the EXTRA bindings
    /// to merge in. `None` means the computation is undefined for these
    /// bindings (unbound variable, division by zero, ...) and the rule
    /// firing should be skipped.
    pub fn eval(&self, bindings: &Bindings) -> Option<Bindings> {
        match self {
            Compute::Expr(program) => program.eval(bindings),
            Compute::Native(f) => Some(f(bindings)),
        }
    }

    /// The serializable source, if any. This is what `db_store` writes to
    /// the `compute_body` column.
    pub fn source(&self) -> Option<&str> {
        match self {
            Compute::Expr(program) => Some(&program.source),
            Compute::Native(_) => None,
        }
    }
}

impl fmt::Debug for Compute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Compute::Expr(program) => write!(f, "Compute::Expr({:?})", program.source),
            Compute::Native(_) => write!(f, "Compute::Native(<closure>)"),
        }
    }
}

/// A parsed sequence of `?var = expression` assignments.
///
/// Assignments run in order and later ones may reference variables defined
/// by earlier ones.
#[derive(Debug, Clone)]
pub struct ExprProgram {
    source: String,
    assignments: Vec<(String, Expr)>,
}

impl ExprProgram {
    pub fn parse(source: &str) -> Result<ExprProgram, String> {
        let mut assignments = Vec::new();
        for stmt in source.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            let (target, body) = stmt
                .split_once('=')
                .ok_or_else(|| format!("expected '?var = expr' in {stmt:?}"))?;
            let target = target.trim();
            if !target.starts_with('?') {
                return Err(format!("assignment target {target:?} must be a ?variable"));
            }
            let mut parser = Parser::new(body)?;
            let expr = parser.parse_expr()?;
            parser.expect_end()?;
            assignments.push((target.to_string(), expr));
        }
        if assignments.is_empty() {
            return Err("empty compute expression".to_string());
        }
        Ok(ExprProgram {
            source: source.to_string(),
            assignments,
        })
    }

    fn eval(&self, bindings: &Bindings) -> Option<Bindings> {
        let mut extra = Bindings::new();
        for (target, expr) in &self.assignments {
            let value = expr.eval(bindings, &extra)?;
            extra.insert(target.clone(), value.into_term());
        }
        Some(extra)
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Var(String),
    Neg(Box<Expr>),
    Bin(Op, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

/// Intermediate value: keeps ints exact until a float enters; strings
/// participate only in `+` (concatenation).
#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Float(f64),
    Str(String),
}

impl Value {
    fn into_term(self) -> Term {
        match self {
            Value::Int(i) => Term::Int(i),
            Value::Float(f) => Term::float(f),
            Value::Str(s) => Term::Str(s),
        }
    }

    /// Numeric view; `None` for strings so arithmetic on them fails softly.
    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            Value::Str(_) => None,
        }
    }

    /// Text view, used when `+` concatenates a string with anything.
    fn render(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => format!("{f:?}"),
            Value::Str(s) => s.clone(),
        }
    }
}

impl Expr {
    fn eval(&self, bindings: &Bindings, extra: &Bindings) -> Option<Value> {
        match self {
            Expr::Int(i) => Some(Value::Int(*i)),
            Expr::Float(f) => Some(Value::Float(*f)),
            Expr::Str(s) => Some(Value::Str(s.clone())),
            Expr::Var(name) => {
                let term = extra.get(name).or_else(|| bindings.get(name))?;
                match term {
                    Term::Int(i) => Some(Value::Int(*i)),
                    Term::Float(f) => Some(Value::Float(f.0)),
                    Term::Str(s) => Some(Value::Str(s.clone())),
                    Term::Bool(_) => None, // no boolean algebra in this DSL
                }
            }
            Expr::Neg(inner) => match inner.eval(bindings, extra)? {
                Value::Int(i) => Some(Value::Int(i.checked_neg()?)),
                Value::Float(f) => Some(Value::Float(-f)),
                Value::Str(_) => None,
            },
            Expr::Bin(op, lhs, rhs) => {
                let l = lhs.eval(bindings, extra)?;
                let r = rhs.eval(bindings, extra)?;
                match op {
                    Op::Add => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Some(Value::Int(a.checked_add(b)?)),
                        // If either side is a string, `+` concatenates —
                        // this is how the assembler splices code fragments
                        // (`?code = ?code1 + ?code2`).
                        (l, r) if matches!(l, Value::Str(_)) || matches!(r, Value::Str(_)) => {
                            Some(Value::Str(format!("{}{}", l.render(), r.render())))
                        }
                        (l, r) => Some(Value::Float(l.as_f64()? + r.as_f64()?)),
                    },
                    Op::Sub => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Some(Value::Int(a.checked_sub(b)?)),
                        (l, r) => Some(Value::Float(l.as_f64()? - r.as_f64()?)),
                    },
                    Op::Mul => match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Some(Value::Int(a.checked_mul(b)?)),
                        (l, r) => Some(Value::Float(l.as_f64()? * r.as_f64()?)),
                    },
                    // Division is always true division (Python semantics).
                    Op::Div => {
                        let d = r.as_f64()?;
                        if d == 0.0 {
                            None
                        } else {
                            Some(Value::Float(l.as_f64()? / d))
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(String),
    Str(String),
    Var(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(src: &str) -> Result<Parser, String> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = src.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            match c {
                ' ' | '\t' | '\n' | '\r' => i += 1,
                '+' => {
                    tokens.push(Token::Plus);
                    i += 1;
                }
                '-' => {
                    tokens.push(Token::Minus);
                    i += 1;
                }
                '*' => {
                    tokens.push(Token::Star);
                    i += 1;
                }
                '/' => {
                    tokens.push(Token::Slash);
                    i += 1;
                }
                '(' => {
                    tokens.push(Token::LParen);
                    i += 1;
                }
                ')' => {
                    tokens.push(Token::RParen);
                    i += 1;
                }
                '"' | '\'' => {
                    let quote = c;
                    i += 1;
                    let mut text = String::new();
                    loop {
                        match chars.get(i) {
                            None => return Err("unterminated string literal".to_string()),
                            Some(&ch) if ch == quote => {
                                i += 1;
                                break;
                            }
                            Some('\\') => {
                                let escaped = chars
                                    .get(i + 1)
                                    .ok_or_else(|| "dangling escape at end of string".to_string())?;
                                text.push(match escaped {
                                    'n' => '\n',
                                    't' => '\t',
                                    'r' => '\r',
                                    '\\' => '\\',
                                    '\'' => '\'',
                                    '"' => '"',
                                    other => return Err(format!("unknown escape \\{other} in string")),
                                });
                                i += 2;
                            }
                            Some(&ch) => {
                                text.push(ch);
                                i += 1;
                            }
                        }
                    }
                    tokens.push(Token::Str(text));
                }
                '?' => {
                    let start = i;
                    i += 1;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    if i == start + 1 {
                        return Err("bare '?' is not a variable".to_string());
                    }
                    tokens.push(Token::Var(chars[start..i].iter().collect()));
                }
                c if c.is_ascii_digit() || c == '.' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    tokens.push(Token::Num(chars[start..i].iter().collect()));
                }
                other => return Err(format!("unexpected character {other:?} in expression")),
            }
        }
        Ok(Parser { tokens, pos: 0 })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect_end(&self) -> Result<(), String> {
        if self.pos == self.tokens.len() {
            Ok(())
        } else {
            Err(format!("trailing tokens after expression: {:?}", &self.tokens[self.pos..]))
        }
    }

    /// expr := term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_term()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Plus => Op::Add,
                Token::Minus => Op::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_term()?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// term := factor (('*' | '/') factor)*
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_factor()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Star => Op::Mul,
                Token::Slash => Op::Div,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_factor()?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// factor := NUM | VAR | '-' factor | '(' expr ')'
    fn parse_factor(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Some(Token::Num(text)) => {
                if text.contains('.') {
                    text.parse::<f64>()
                        .map(Expr::Float)
                        .map_err(|_| format!("bad number literal {text:?}"))
                } else {
                    text.parse::<i64>()
                        .map(Expr::Int)
                        .map_err(|_| format!("bad number literal {text:?}"))
                }
            }
            Some(Token::Str(text)) => Ok(Expr::Str(text)),
            Some(Token::Var(name)) => Ok(Expr::Var(name)),
            Some(Token::Minus) => Ok(Expr::Neg(Box::new(self.parse_factor()?))),
            Some(Token::LParen) => {
                let inner = self.parse_expr()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(inner),
                    _ => Err("missing closing ')'".to_string()),
                }
            }
            other => Err(format!("unexpected token {other:?} in expression")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings(pairs: &[(&str, Term)]) -> Bindings {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    #[test]
    fn multiplies_bound_ints() {
        let c = Compute::expr("?a = ?w * ?h").unwrap();
        let extra = c
            .eval(&bindings(&[("?w", Term::Int(10)), ("?h", Term::Int(20))]))
            .unwrap();
        assert_eq!(extra.get("?a"), Some(&Term::Int(200)));
    }

    #[test]
    fn successor_expression() {
        let c = Compute::expr("?next = ?v + 1").unwrap();
        let extra = c.eval(&bindings(&[("?v", Term::Int(41))])).unwrap();
        assert_eq!(extra.get("?next"), Some(&Term::Int(42)));
    }

    #[test]
    fn precedence_and_parentheses() {
        let c = Compute::expr("?x = 2 + 3 * 4; ?y = (2 + 3) * 4").unwrap();
        let extra = c.eval(&Bindings::new()).unwrap();
        assert_eq!(extra.get("?x"), Some(&Term::Int(14)));
        assert_eq!(extra.get("?y"), Some(&Term::Int(20)));
    }

    #[test]
    fn division_yields_float_and_guards_zero() {
        let c = Compute::expr("?half = ?v / 2").unwrap();
        let extra = c.eval(&bindings(&[("?v", Term::Int(5))])).unwrap();
        assert_eq!(extra.get("?half"), Some(&Term::float(2.5)));

        let z = Compute::expr("?bad = 1 / 0").unwrap();
        assert!(z.eval(&Bindings::new()).is_none());
    }

    #[test]
    fn unbound_variable_fails_softly() {
        let c = Compute::expr("?a = ?missing * 2").unwrap();
        assert!(c.eval(&Bindings::new()).is_none());
    }

    #[test]
    fn later_assignment_sees_earlier() {
        let c = Compute::expr("?double = ?v * 2; ?quad = ?double * 2").unwrap();
        let extra = c.eval(&bindings(&[("?v", Term::Int(3))])).unwrap();
        assert_eq!(extra.get("?quad"), Some(&Term::Int(12)));
    }

    #[test]
    fn string_concatenation() {
        let c = Compute::expr("?code = ?code1 + ?code2").unwrap();
        let extra = c
            .eval(&bindings(&[
                ("?code1", Term::str("import pygame\n")),
                ("?code2", Term::str("pygame.init()\n")),
            ]))
            .unwrap();
        assert_eq!(
            extra.get("?code"),
            Some(&Term::str("import pygame\npygame.init()\n"))
        );
    }

    #[test]
    fn string_literals_and_mixed_concat() {
        let c = Compute::expr("?msg = 'score: ' + ?points + \"\\n\"").unwrap();
        let extra = c.eval(&bindings(&[("?points", Term::Int(42))])).unwrap();
        assert_eq!(extra.get("?msg"), Some(&Term::str("score: 42\n")));
    }

    #[test]
    fn arithmetic_on_strings_fails_softly() {
        let c = Compute::expr("?x = ?s * 2").unwrap();
        assert!(c.eval(&bindings(&[("?s", Term::str("abc"))])).is_none());
    }

    #[test]
    fn source_round_trips() {
        let c = Compute::expr("?a = ?w * ?h").unwrap();
        assert_eq!(c.source(), Some("?a = ?w * ?h"));
        let native = Compute::native(|_| Bindings::new());
        assert_eq!(native.source(), None);
    }
}
