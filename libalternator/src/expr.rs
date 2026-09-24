//! A tiny SourcePawn constant expression parser.
//!
//! It is used for three things:
//! - evaluating `#if`/`#elseif` conditions,
//! - rendering default argument values and enum values,
//! - rendering array dimensions.
//!
//! Rendering intentionally mirrors the output of the old SourcePawn `docparse`
//! tool so that existing bundles don't churn: parentheses are dropped, binary
//! operators are separated by single spaces, integer literals are printed in
//! decimal and float literals with six decimal places.

use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int(i64),
    Float(f64),
    /// String literal, raw source text including quotes.
    Str(String),
    /// Character literal, raw source text including quotes.
    Char(String, i64),
    Ident(String),
    Punct(&'static str),
}

const PUNCTS: &[&str] = &[
    ">>>=", "<<=", ">>=", ">>>", "...", "::", "<<", ">>", "<=", ">=", "==", "!=", "&&", "||", "++",
    "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "~=", "(", ")", "[", "]", "{", "}", "<",
    ">", "+", "-", "*", "/", "%", "&", "|", "^", "!", "~", "?", ":", ",", ".", "=", ";",
];

/// Tokenizes an expression. Comments are skipped. Returns `None` on
/// characters that can't appear in an expression.
pub fn tokenize(src: &str) -> Option<Vec<Token>> {
    let mut out = Vec::new();
    let mut it: Peekable<CharIndices> = src.char_indices().peekable();

    while let Some(&(i, c)) = it.peek() {
        if c.is_whitespace() || c == '\\' {
            it.next();
            continue;
        }

        let rest = &src[i..];

        if rest.starts_with("//") {
            break;
        }

        if rest.starts_with("/*") {
            let end = rest[2..].find("*/").map(|e| i + 2 + e + 2).unwrap_or(src.len());
            while let Some(&(j, _)) = it.peek() {
                if j >= end {
                    break;
                }
                it.next();
            }
            continue;
        }

        if c.is_ascii_digit() {
            let mut end = i;
            let mut is_float = false;
            let bytes = src.as_bytes();
            let hex = rest.starts_with("0x") || rest.starts_with("0X");
            while end < src.len() {
                let b = bytes[end];
                let ok = b.is_ascii_alphanumeric()
                    || b == b'_'
                    || (b == b'.' && !hex && !is_float && !rest[end - i..].starts_with("..."))
                    || ((b == b'-' || b == b'+')
                        && is_float
                        && matches!(bytes[end - 1], b'e' | b'E'));
                if !ok {
                    break;
                }
                if b == b'.' {
                    is_float = true;
                }
                end += 1;
            }
            let text = &src[i..end];
            while let Some(&(j, _)) = it.peek() {
                if j >= end {
                    break;
                }
                it.next();
            }
            out.push(parse_number(text, is_float)?);
            continue;
        }

        if c.is_alphabetic() || c == '_' || c == '@' {
            let mut end = i;
            while let Some(&(j, ch)) = it.peek() {
                if ch.is_alphanumeric() || ch == '_' || (ch == '@' && j == i) {
                    end = j + ch.len_utf8();
                    it.next();
                } else {
                    break;
                }
            }
            out.push(Token::Ident(src[i..end].to_string()));
            continue;
        }

        if c == '"' || c == '\'' {
            it.next();
            let mut end = src.len();
            let mut escaped = false;
            for (j, ch) in it.by_ref() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == c {
                    end = j + 1;
                    break;
                }
            }
            let raw = src[i..end].to_string();
            if c == '"' {
                out.push(Token::Str(raw));
            } else {
                let value = char_value(&raw);
                out.push(Token::Char(raw, value));
            }
            continue;
        }

        let p = PUNCTS.iter().find(|p| rest.starts_with(**p))?;
        for _ in 0..p.len() {
            it.next();
        }
        out.push(Token::Punct(p));
    }

    Some(out)
}

fn parse_number(text: &str, is_float: bool) -> Option<Token> {
    let clean: String = text.chars().filter(|c| *c != '_').collect();

    if is_float {
        return clean.parse::<f64>().ok().map(Token::Float);
    }

    let lower = clean.to_ascii_lowercase();
    let (digits, radix) = if let Some(d) = lower.strip_prefix("0x") {
        (d, 16)
    } else if let Some(d) = lower.strip_prefix("0b") {
        (d, 2)
    } else if let Some(d) = lower.strip_prefix("0o") {
        (d, 8)
    } else {
        (lower.as_str(), 10)
    };

    u64::from_str_radix(digits, radix).ok().map(|v| Token::Int(v as i64))
}

fn char_value(raw: &str) -> i64 {
    let inner = raw.trim_start_matches('\'').trim_end_matches('\'');
    let mut chars = inner.chars();
    match chars.next() {
        Some('\\') => match chars.next() {
            Some('n') => 10,
            Some('r') => 13,
            Some('t') => 9,
            Some('a') => 7,
            Some('b') => 8,
            Some('e') => 27,
            Some('f') => 12,
            Some('v') => 11,
            Some('0') => 0,
            Some(c) => c as i64,
            None => '\\' as i64,
        },
        Some(c) => c as i64,
        None => 0,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Char(String, i64),
    Name(String),
    Bool(bool),
    Null,
    This,
    Unary(&'static str, Box<Expr>),
    Postfix(&'static str, Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, &'static str, String),
    ViewAs(String, Box<Expr>),
    Sizeof(String),
    Defined(String),
    Array(Vec<Expr>, bool),
}

struct Parser<'t> {
    tokens: &'t [Token],
    pos: usize,
}

fn binary_prec(p: &str) -> Option<u8> {
    Some(match p {
        "||" => 1,
        "&&" => 2,
        "|" => 3,
        "^" => 4,
        "&" => 5,
        "==" | "!=" => 6,
        "<" | "<=" | ">" | ">=" => 7,
        "<<" | ">>" | ">>>" => 8,
        "+" | "-" => 9,
        "*" | "/" | "%" => 10,
        "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=" => 0,
        _ => return None,
    })
}

impl<'t> Parser<'t> {
    fn peek(&self) -> Option<&'t Token> {
        self.tokens.get(self.pos)
    }

    fn peek_punct(&self, p: &str) -> bool {
        matches!(self.peek(), Some(Token::Punct(x)) if *x == p)
    }

    fn eat(&mut self, p: &str) -> bool {
        if self.peek_punct(p) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, p: &str) -> Option<()> {
        self.eat(p).then_some(())
    }

    fn expr(&mut self) -> Option<Expr> {
        self.ternary()
    }

    fn ternary(&mut self) -> Option<Expr> {
        let cond = self.binary(0)?;
        if self.eat("?") {
            let a = self.ternary()?;
            self.expect(":")?;
            let b = self.ternary()?;
            return Some(Expr::Ternary(Box::new(cond), Box::new(a), Box::new(b)));
        }
        Some(cond)
    }

    fn binary(&mut self, min: u8) -> Option<Expr> {
        let mut lhs = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Punct(p)) => *p,
                _ => break,
            };
            let prec = match binary_prec(op) {
                Some(p) if p >= min => p,
                _ => break,
            };
            self.pos += 1;
            // Assignments are right associative, everything else left.
            let rhs = if prec == 0 {
                self.binary(prec)?
            } else {
                self.binary(prec + 1)?
            };
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Some(lhs)
    }

    fn unary(&mut self) -> Option<Expr> {
        if let Some(Token::Punct(p)) = self.peek() {
            if matches!(*p, "!" | "~" | "-" | "+" | "++" | "--" | "&") {
                self.pos += 1;
                let e = self.unary()?;
                return Some(Expr::Unary(p, Box::new(e)));
            }
        }

        // Old style tag cast `Float:5`, `_:x`
        if let (Some(Token::Ident(_)), Some(Token::Punct(":"))) =
            (self.peek(), self.tokens.get(self.pos + 1))
        {
            if !matches!(self.tokens.get(self.pos + 2), Some(Token::Punct(":")) | None) {
                let tag = match self.peek() {
                    Some(Token::Ident(t)) => t.clone(),
                    _ => unreachable!(),
                };
                self.pos += 2;
                let e = self.unary()?;
                return Some(Expr::ViewAs(old_tag_to_type(&tag).to_string(), Box::new(e)));
            }
        }

        self.postfix()
    }

    fn postfix(&mut self) -> Option<Expr> {
        let mut e = self.primary()?;
        loop {
            if self.eat("(") {
                let mut args = Vec::new();
                if !self.eat(")") {
                    loop {
                        args.push(self.expr()?);
                        if self.eat(")") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                e = Expr::Call(Box::new(e), args);
            } else if self.eat("[") {
                let idx = self.expr()?;
                self.expect("]")?;
                e = Expr::Index(Box::new(e), Box::new(idx));
            } else if self.peek_punct(".") || self.peek_punct("::") {
                let op = if self.eat(".") { "." } else {
                    self.pos += 1;
                    "::"
                };
                match self.peek() {
                    Some(Token::Ident(n)) => {
                        self.pos += 1;
                        e = Expr::Field(Box::new(e), op, n.clone());
                    }
                    _ => return None,
                }
            } else if self.peek_punct("++") || self.peek_punct("--") {
                let op = if self.eat("++") { "++" } else {
                    self.pos += 1;
                    "--"
                };
                e = Expr::Postfix(op, Box::new(e));
            } else {
                break;
            }
        }
        Some(e)
    }

    fn primary(&mut self) -> Option<Expr> {
        let tok = self.peek()?.clone();
        self.pos += 1;
        Some(match tok {
            Token::Int(v) => Expr::Int(v),
            Token::Float(v) => Expr::Float(v),
            Token::Str(s) => {
                // Adjacent string literals are concatenated
                let mut s = s;
                while let Some(Token::Str(next)) = self.peek() {
                    s = format!("{}{}", &s[..s.len() - 1], &next[1..]);
                    self.pos += 1;
                }
                Expr::Str(s)
            }
            Token::Char(raw, v) => Expr::Char(raw, v),
            Token::Ident(name) => match name.as_str() {
                "true" => Expr::Bool(true),
                "false" => Expr::Bool(false),
                "null" => Expr::Null,
                "this" => Expr::This,
                "defined" => {
                    let paren = self.eat("(");
                    let n = match self.peek()? {
                        Token::Ident(n) => n.clone(),
                        _ => return None,
                    };
                    self.pos += 1;
                    if paren {
                        self.expect(")")?;
                    }
                    Expr::Defined(n)
                }
                "sizeof" => {
                    let paren = self.eat("(");
                    let start = self.pos;
                    let mut depth = 0i32;
                    // Consume the sizeof operand verbatim
                    while let Some(t) = self.peek() {
                        match t {
                            Token::Punct("(") | Token::Punct("[") => depth += 1,
                            Token::Punct(")") | Token::Punct("]") => {
                                if depth == 0 {
                                    break;
                                }
                                depth -= 1;
                            }
                            Token::Ident(_) | Token::Punct(".") | Token::Punct("::") => {}
                            _ if depth > 0 || paren => {}
                            _ => break,
                        }
                        self.pos += 1;
                    }
                    let operand = render_tokens(&self.tokens[start..self.pos]);
                    if paren {
                        self.expect(")")?;
                    }
                    Expr::Sizeof(operand)
                }
                "view_as" => {
                    self.expect("<")?;
                    let mut ty = String::new();
                    while let Some(t) = self.peek() {
                        match t {
                            Token::Punct(">") => break,
                            Token::Ident(n) => ty.push_str(n),
                            Token::Punct(p) => ty.push_str(p),
                            _ => return None,
                        }
                        self.pos += 1;
                    }
                    self.expect(">")?;
                    self.expect("(")?;
                    let e = self.expr()?;
                    self.expect(")")?;
                    Expr::ViewAs(ty, Box::new(e))
                }
                _ => Expr::Name(name),
            },
            Token::Punct("(") => {
                let e = self.expr()?;
                self.expect(")")?;
                e
            }
            Token::Punct("{") => {
                let mut items = Vec::new();
                let mut repeat = false;
                if !self.eat("}") {
                    loop {
                        if self.eat("...") {
                            repeat = true;
                            self.expect("}")?;
                            break;
                        }
                        items.push(self.expr()?);
                        if self.eat("}") {
                            break;
                        }
                        self.expect(",")?;
                        if self.eat("}") {
                            break;
                        }
                    }
                }
                Expr::Array(items, repeat)
            }
            _ => return None,
        })
    }
}

fn render_tokens(tokens: &[Token]) -> String {
    let mut s = String::new();
    for t in tokens {
        match t {
            Token::Int(v) => s += &v.to_string(),
            Token::Float(v) => s += &format!("{:.6}", v),
            Token::Str(r) | Token::Char(r, _) => s += r,
            Token::Ident(n) => s += n,
            Token::Punct(p) => s += p,
        }
    }
    s
}

/// Maps old style tags (`Float:`, `String:`, `_:`) to new style types
pub fn old_tag_to_type(tag: &str) -> &str {
    match tag {
        "Float" => "float",
        "String" => "char",
        "_" => "int",
        "bool" => "bool",
        other => other,
    }
}

pub fn parse(src: &str) -> Option<Expr> {
    let tokens = tokenize(src)?;
    if tokens.is_empty() {
        return None;
    }
    let mut p = Parser {
        tokens: &tokens,
        pos: 0,
    };
    let e = p.expr()?;
    if p.pos != tokens.len() {
        return None;
    }
    Some(e)
}

/// Access to macro definitions needed while rendering or evaluating
pub trait MacroLookup {
    /// Returns the replacement text of an object-like macro visible at `offset`
    fn lookup(&self, name: &str, offset: usize) -> Option<&str>;
}

#[cfg(test)]
pub struct NoMacros;

#[cfg(test)]
impl MacroLookup for NoMacros {
    fn lookup(&self, _: &str, _: usize) -> Option<&str> {
        None
    }
}

pub struct Renderer<'m, M: MacroLookup> {
    pub macros: &'m M,
    /// Byte offset where the expression appears (macros defined later aren't visible)
    pub offset: usize,
    depth: usize,
}

impl<'m, M: MacroLookup> Renderer<'m, M> {
    pub fn new(macros: &'m M, offset: usize) -> Self {
        Self {
            macros,
            offset,
            depth: 0,
        }
    }

    /// Renders an expression's source text. Falls back to the whitespace
    /// normalized source when it can't be parsed.
    pub fn render_src(&mut self, src: &str) -> String {
        match parse(src) {
            Some(e) => self.render(&e),
            None => src.split_whitespace().collect::<Vec<_>>().join(" "),
        }
    }

    pub fn render(&mut self, e: &Expr) -> String {
        match e {
            Expr::Int(v) => v.to_string(),
            Expr::Float(v) => format!("{:.6}", v),
            Expr::Str(s) => s.clone(),
            Expr::Char(raw, _) => raw.clone(),
            Expr::Bool(b) => b.to_string(),
            Expr::Null => "null".into(),
            Expr::This => "this".into(),
            Expr::Name(n) => {
                if self.depth < 16 {
                    if let Some(value) = self.macros.lookup(n, self.offset) {
                        let value = value.to_string();
                        self.depth += 1;
                        let r = self.render_src(&value);
                        self.depth -= 1;
                        return r;
                    }
                }
                n.clone()
            }
            Expr::Unary(op, e) => format!("{}{}", op, self.render(e)),
            Expr::Postfix(op, e) => format!("{}{}", self.render(e), op),
            Expr::Binary(op, l, r) => {
                format!("{} {} {}", self.render(l), op, self.render(r))
            }
            Expr::Ternary(c, a, b) => format!(
                "{} ? {} : {}",
                self.render(c),
                self.render(a),
                self.render(b)
            ),
            Expr::Call(f, args) => {
                let args: Vec<String> = args.iter().map(|a| self.render(a)).collect();
                format!("{}({})", self.render(f), args.join(", "))
            }
            Expr::Index(a, i) => format!("{}[{}]", self.render(a), self.render(i)),
            Expr::Field(b, op, n) => format!("{}{}{}", self.render(b), op, n),
            Expr::ViewAs(t, e) => format!("view_as<{}>({})", t, self.render(e)),
            Expr::Sizeof(s) => format!("sizeof({})", s),
            Expr::Defined(n) => format!("defined {}", n),
            Expr::Array(items, repeat) => {
                if items.is_empty() {
                    return "{}".into();
                }
                let mut s = "{ ".to_string();
                for (i, item) in items.iter().enumerate() {
                    s += &self.render(item);
                    if i != items.len() - 1 || *repeat {
                        s += ", ";
                    }
                }
                if *repeat {
                    s += "...";
                }
                s + " }"
            }
        }
    }
}

/// Evaluates a preprocessor condition.
pub fn eval<F>(e: &Expr, defined: &F, value_of: &dyn Fn(&str) -> Option<String>, depth: usize) -> Option<i64>
where
    F: Fn(&str) -> bool,
{
    if depth > 32 {
        return None;
    }
    let ev = |e: &Expr| eval(e, defined, value_of, depth + 1);
    Some(match e {
        Expr::Int(v) => *v,
        Expr::Char(_, v) => *v,
        Expr::Bool(b) => *b as i64,
        Expr::Float(f) => *f as i64,
        Expr::Defined(n) => defined(n) as i64,
        Expr::Name(n) => match value_of(n) {
            Some(v) => match parse(&v) {
                Some(inner) => eval(&inner, defined, value_of, depth + 1)?,
                None => 0,
            },
            None => 0,
        },
        Expr::Unary(op, e) => {
            let v = ev(e)?;
            match *op {
                "!" => (v == 0) as i64,
                "~" => !v,
                "-" => v.wrapping_neg(),
                "+" => v,
                _ => return None,
            }
        }
        Expr::Binary(op, l, r) => {
            let a = ev(l)?;
            // Short circuit
            match *op {
                "&&" => return Some((a != 0 && ev(r)? != 0) as i64),
                "||" => return Some((a != 0 || ev(r)? != 0) as i64),
                _ => {}
            }
            let b = ev(r)?;
            match *op {
                "|" => a | b,
                "^" => a ^ b,
                "&" => a & b,
                "==" => (a == b) as i64,
                "!=" => (a != b) as i64,
                "<" => (a < b) as i64,
                "<=" => (a <= b) as i64,
                ">" => (a > b) as i64,
                ">=" => (a >= b) as i64,
                "<<" => a.wrapping_shl(b as u32),
                ">>" => a.wrapping_shr(b as u32),
                ">>>" => ((a as u64).wrapping_shr(b as u32)) as i64,
                "+" => a.wrapping_add(b),
                "-" => a.wrapping_sub(b),
                "*" => a.wrapping_mul(b),
                "/" => a.checked_div(b)?,
                "%" => a.checked_rem(b)?,
                _ => return None,
            }
        }
        Expr::Ternary(c, a, b) => {
            if ev(c)? != 0 {
                ev(a)?
            } else {
                ev(b)?
            }
        }
        Expr::ViewAs(_, e) => ev(e)?,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(s: &str) -> String {
        Renderer::new(&NoMacros, 0).render_src(s)
    }

    #[test]
    fn renders_like_docparse() {
        assert_eq!(r("(1<<0)"), "1 << 0");
        assert_eq!(r("(3<<1)|(1<<0)"), "3 << 1 | 1 << 0");
        assert_eq!(r("0x0100"), "256");
        assert_eq!(r("-1"), "-1");
        assert_eq!(r("1.0"), "1.000000");
        assert_eq!(r("{0.0, 0.0, 1.0}"), "{ 0.000000, 0.000000, 1.000000 }");
        assert_eq!(r("{}"), "{}");
        assert_eq!(r("view_as<PropFieldType>(0)"), "view_as<PropFieldType>(0)");
        assert_eq!(r("\"GAME\""), "\"GAME\"");
        assert_eq!(r("'C'"), "'C'");
        assert_eq!(r("MenuAction_Select|MenuAction_Cancel"), "MenuAction_Select | MenuAction_Cancel");
        assert_eq!(r("sizeof(buffer)"), "sizeof(buffer)");
        assert_eq!(r("a ? b : c"), "a ? b : c");
        assert_eq!(r("Foo(1, 2)"), "Foo(1, 2)");
    }

    #[test]
    fn evaluates_conditions() {
        let defined = |n: &str| n == "FOO";
        let value = |n: &str| if n == "FOO" { Some("2".to_string()) } else { None };
        let e = |s: &str| eval(&parse(s).unwrap(), &defined, &value, 0).unwrap();
        assert_eq!(e("defined FOO"), 1);
        assert_eq!(e("!defined BAR"), 1);
        assert_eq!(e("defined(FOO) && FOO >= 2"), 1);
        assert_eq!(e("BAR"), 0);
        assert_eq!(e("FOO * 2 == 4"), 1);
    }
}
