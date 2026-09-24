//! Minimal SourcePawn preprocessor.
//!
//! Include files are documented one at a time without resolving `#include`s,
//! so this only has to:
//! - evaluate `#if`/`#elseif`/`#else`/`#endif` and drop inactive branches,
//! - record `#define`s (they're documented symbols and are expanded when
//!   rendering constant expressions),
//! - honour `#endinput`,
//! - collect comments and a coarse token stream used for attaching
//!   documentation comments to declarations.
//!
//! The source handed to tree-sitter has every directive line and every
//! inactive region blanked out with spaces, so byte offsets and line numbers
//! stay identical to the original file.

use crate::expr::{self, MacroLookup};

/// Macros predefined by the SourcePawn compiler
const PREDEFINED: &[(&str, &str)] = &[("__sourcepawn__", "2"), ("__sourcepawn2__", "1")];

#[derive(Debug, Clone)]
pub struct Macro {
    pub name: String,
    /// Raw replacement text as written in the source
    pub value: String,
    /// Replacement text with comments and line continuations removed
    pub clean_value: String,
    pub function_like: bool,
    /// Byte offset of the start of the replacement text (or the name when empty)
    pub offset: usize,
    /// Byte offset of the `#define` directive
    pub defined_at: usize,
    /// Byte offset of the `#undef` (or redefinition) removing this macro
    pub undefined_at: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawComment {
    pub start: usize,
    pub end: usize,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item {
    /// Any non comment token. Only its line matters, and whether it starts
    /// a preprocessor directive.
    Token { line: usize, directive: bool },
    Comment(RawComment),
}

pub struct Preprocessed {
    /// Source with directives and inactive regions blanked out
    pub text: String,
    pub items: Vec<Item>,
    pub macros: Vec<Macro>,
}

impl MacroLookup for Preprocessed {
    fn lookup(&self, name: &str, offset: usize) -> Option<&str> {
        macro_at(&self.macros, name, offset).map(|m| m.clean_value.as_str())
    }
}

fn macro_at<'a>(macros: &'a [Macro], name: &str, offset: usize) -> Option<&'a Macro> {
    macros.iter().rev().find(|m| {
        m.name == name
            && !m.function_like
            && m.defined_at <= offset
            && m.undefined_at.map_or(true, |u| u > offset)
    })
}

struct Frame {
    parent_active: bool,
    taken: bool,
    active: bool,
}

struct Scanner<'s> {
    src: &'s [u8],
    pos: usize,
    line: usize,
    stack: Vec<Frame>,
    items: Vec<Item>,
    macros: Vec<Macro>,
    /// Byte ranges to blank out for tree-sitter
    masked: Vec<(usize, usize)>,
    inactive_since: Option<usize>,
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b >= 0x80
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80
}

impl<'s> Scanner<'s> {
    fn active(&self) -> bool {
        self.stack.last().map_or(true, |f| f.active)
    }

    fn peek(&self, n: usize) -> u8 {
        *self.src.get(self.pos + n).unwrap_or(&0)
    }

    fn push_token(&mut self, line: usize, directive: bool) {
        if let Some(Item::Token { line: l, .. }) = self.items.last() {
            if *l == line && !directive {
                return;
            }
        }
        self.items.push(Item::Token { line, directive });
    }

    /// Consumes a newline sequence at the current position if there is one
    fn newline(&mut self) -> bool {
        match self.peek(0) {
            b'\n' => {
                self.pos += 1;
                self.line += 1;
                true
            }
            b'\r' => {
                self.pos += 1;
                if self.peek(0) == b'\n' {
                    self.pos += 1;
                }
                self.line += 1;
                true
            }
            _ => false,
        }
    }

    /// Scans a comment starting at the current position (which must be `//` or `/*`)
    fn comment(&mut self, record: bool) {
        let start = self.pos;
        let start_line = self.line;
        if self.peek(1) == b'/' {
            while self.pos < self.src.len() && !matches!(self.peek(0), b'\n' | b'\r') {
                self.pos += 1;
            }
        } else {
            self.pos += 2;
            loop {
                if self.pos >= self.src.len() {
                    break;
                }
                if self.peek(0) == b'*' && self.peek(1) == b'/' {
                    self.pos += 2;
                    break;
                }
                if !self.newline() {
                    self.pos += 1;
                }
            }
        }
        if record {
            self.items.push(Item::Comment(RawComment {
                start,
                end: self.pos,
                start_line,
                end_line: self.line,
            }));
        }
    }

    /// Skips a string or character literal
    fn literal(&mut self) {
        let quote = self.peek(0);
        self.pos += 1;
        while self.pos < self.src.len() {
            match self.peek(0) {
                b'\\' => {
                    self.pos += 1;
                    if !self.newline() {
                        self.pos += 1;
                    }
                }
                b'\n' | b'\r' => break,
                c => {
                    self.pos += 1;
                    if c == quote {
                        break;
                    }
                }
            }
        }
    }

    fn run(&mut self) {
        let mut at_line_start = true;

        while self.pos < self.src.len() {
            let c = self.peek(0);

            if self.newline() {
                at_line_start = true;
                continue;
            }

            if c == b' ' || c == b'\t' || c == 0x0B || c == 0x0C {
                self.pos += 1;
                continue;
            }

            // Line continuation outside of a directive
            if c == b'\\' && matches!(self.peek(1), b'\n' | b'\r') {
                self.pos += 1;
                self.newline();
                continue;
            }

            let active = self.active();

            if c == b'/' && (self.peek(1) == b'/' || self.peek(1) == b'*') {
                self.comment(active);
                continue;
            }

            if at_line_start && c == b'#' {
                if !self.directive() {
                    // #endinput
                    return;
                }
                at_line_start = true;
                continue;
            }

            at_line_start = false;

            if active {
                self.push_token(self.line, false);
            }

            if c == b'"' || c == b'\'' {
                self.literal();
            } else {
                self.pos += 1;
            }
        }
    }

    /// Handles a directive at the current position. Returns false on `#endinput`.
    fn directive(&mut self) -> bool {
        let start = self.pos;
        let start_line = self.line;
        let was_active = self.active();

        if was_active {
            self.push_token(start_line, true);
        }

        // Collect the logical line: strip comments, join continuations
        self.pos += 1;
        let mut clean = String::new();
        // Byte ranges of the raw text that make up the directive body (for define values)
        let mut body_start: Option<usize> = None;
        let mut body_end = self.pos;

        loop {
            if self.pos >= self.src.len() {
                break;
            }
            let c = self.peek(0);
            if c == b'\n' || c == b'\r' {
                break;
            }
            if c == b'\\' && matches!(self.peek(1), b'\n' | b'\r') {
                self.pos += 1;
                self.newline();
                clean.push(' ');
                continue;
            }
            if c == b'/' && (self.peek(1) == b'/' || self.peek(1) == b'*') {
                self.comment(was_active);
                clean.push(' ');
                continue;
            }
            if c == b'"' || c == b'\'' {
                let s = self.pos;
                self.literal();
                clean.push_str(&String::from_utf8_lossy(&self.src[s..self.pos]));
                body_start.get_or_insert(s);
                body_end = self.pos;
                continue;
            }
            if c != b' ' && c != b'\t' {
                body_start.get_or_insert(self.pos);
                body_end = self.pos + 1;
            }
            // Keep multi-byte characters intact
            let len = utf8_len(c);
            let end = (self.pos + len).min(self.src.len());
            clean.push_str(&String::from_utf8_lossy(&self.src[self.pos..end]));
            self.pos = end;
        }

        let end_line = self.line;
        let end = self.pos;
        self.masked.push((start, end));

        if was_active {
            // Directives end with an end of line token which terminates tail comments
            self.push_token(end_line, false);
        }

        let trimmed = clean.trim_start();
        let name_len = trimmed
            .bytes()
            .take_while(|b| is_ident(*b))
            .count();
        let name = &trimmed[..name_len];
        let rest = trimmed[name_len..].trim();

        match name {
            "if" => {
                let parent = self.active();
                let cond = parent && self.evaluate(rest, start);
                self.push_frame(Frame {
                    parent_active: parent,
                    taken: cond || !parent,
                    active: cond,
                }, end);
            }
            "elseif" | "elif" => {
                if let Some(f) = self.stack.last() {
                    let (parent, taken) = (f.parent_active, f.taken);
                    let cond = !taken && parent && self.evaluate(rest, start);
                    self.set_top(cond, taken || cond, start, end);
                }
            }
            "else" => {
                if let Some(f) = self.stack.last() {
                    let (parent, taken) = (f.parent_active, f.taken);
                    self.set_top(parent && !taken, true, start, end);
                }
            }
            "endif" => {
                if self.stack.pop().is_some() {
                    self.update_inactive(start, end);
                }
            }
            _ if !was_active => {}
            "define" => self.define(start, body_start, body_end),
            "undef" => {
                let n: String = rest.bytes().take_while(|b| is_ident(*b)).map(|b| b as char).collect();
                self.undefine(&n, start);
            }
            "endinput" => {
                self.masked.push((end, self.src.len()));
                return false;
            }
            _ => {}
        }

        true
    }

    fn push_frame(&mut self, frame: Frame, directive_end: usize) {
        let before = self.active();
        self.stack.push(frame);
        let after = self.active();
        if before && !after {
            self.inactive_since = Some(directive_end);
        }
    }

    fn set_top(&mut self, active: bool, taken: bool, start: usize, end: usize) {
        if let Some(f) = self.stack.last_mut() {
            f.active = active;
            f.taken = taken;
        }
        self.update_inactive(start, end);
    }

    /// Opens or closes the current masked inactive range after a
    /// conditional directive spanning `start..end` was processed.
    fn update_inactive(&mut self, start: usize, end: usize) {
        let active = self.active();
        match (self.inactive_since, active) {
            (Some(since), true) => {
                self.masked.push((since, start));
                self.inactive_since = None;
            }
            (None, false) => self.inactive_since = Some(end),
            _ => {}
        }
    }

    fn evaluate(&self, cond: &str, offset: usize) -> bool {
        let e = match expr::parse(cond) {
            Some(e) => e,
            None => return false,
        };
        let defined = |n: &str| self.is_defined(n, offset);
        let value_of = |n: &str| -> Option<String> {
            if let Some((_, v)) = PREDEFINED.iter().find(|(p, _)| *p == n) {
                return Some(v.to_string());
            }
            macro_at(&self.macros, n, offset).map(|m| m.clean_value.clone())
        };
        expr::eval(&e, &defined, &value_of, 0).map_or(false, |v| v != 0)
    }

    fn is_defined(&self, name: &str, offset: usize) -> bool {
        PREDEFINED.iter().any(|(p, _)| *p == name)
            || self.macros.iter().any(|m| {
                m.name == name && m.defined_at <= offset && m.undefined_at.map_or(true, |u| u > offset)
            })
    }

    fn undefine(&mut self, name: &str, at: usize) {
        for m in self.macros.iter_mut() {
            if m.name == name && m.undefined_at.is_none() {
                m.undefined_at = Some(at);
            }
        }
    }

    fn define(&mut self, directive: usize, body_start: Option<usize>, body_end: usize) {
        let body_start = match body_start {
            Some(b) => b,
            None => return,
        };
        let src = self.src;
        // Skip `#`, spaces and `define`
        let mut p = directive + 1;
        while p < body_end && (src[p] == b' ' || src[p] == b'\t') {
            p += 1;
        }
        p += "define".len();
        while p < body_end && matches!(src[p], b' ' | b'\t' | b'\\' | b'\r' | b'\n') {
            p += 1;
        }
        let _ = body_start;
        if p >= body_end || !is_ident_start(src[p]) {
            return;
        }
        let name_start = p;
        while p < body_end && is_ident(src[p]) {
            p += 1;
        }
        let name = String::from_utf8_lossy(&src[name_start..p]).to_string();

        let mut function_like = false;
        if p < body_end && src[p] == b'(' {
            function_like = true;
            while p < body_end && src[p] != b')' {
                p += 1;
            }
            p += 1;
        }

        // Skip whitespace and comments before the replacement
        loop {
            while p < body_end && matches!(src[p], b' ' | b'\t' | b'\r' | b'\n') {
                p += 1;
            }
            if p + 1 < body_end && src[p] == b'\\' && matches!(src[p + 1], b'\r' | b'\n') {
                p += 1;
                continue;
            }
            if p + 1 < body_end && src[p] == b'/' && src[p + 1] == b'*' {
                match find(src, p + 2, b"*/") {
                    Some(e) => {
                        p = e + 2;
                        continue;
                    }
                    None => break,
                }
            }
            break;
        }

        let (value, offset) = if p < body_end {
            (String::from_utf8_lossy(&src[p..body_end]).to_string(), p)
        } else {
            (String::new(), name_start)
        };

        let clean_value = strip_comments(&value);

        self.undefine(&name, directive);
        self.macros.push(Macro {
            name,
            value,
            clean_value,
            function_like,
            offset,
            defined_at: directive,
            undefined_at: None,
        });
    }
}

fn utf8_len(first: u8) -> usize {
    match first {
        0xF0..=0xFF => 4,
        0xE0..=0xEF => 3,
        0xC0..=0xDF => 2,
        _ => 1,
    }
}

fn find(src: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from >= src.len() {
        return None;
    }
    src[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

/// Removes comments and line continuations from a macro body
fn strip_comments(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while !rest.is_empty() {
        if rest.starts_with("/*") {
            match rest[2..].find("*/") {
                Some(e) => rest = &rest[2 + e + 2..],
                None => break,
            }
            out.push(' ');
        } else if rest.starts_with("//") {
            break;
        } else if rest.starts_with('"') || rest.starts_with('\'') {
            let q = rest.as_bytes()[0];
            let bytes = rest.as_bytes();
            let mut i = 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == q {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let i = i.min(rest.len());
            out.push_str(&rest[..i]);
            rest = &rest[i..];
        } else if rest.starts_with("\\\r\n") {
            out.push(' ');
            rest = &rest[3..];
        } else if rest.starts_with("\\\n") || rest.starts_with("\\\r") {
            out.push(' ');
            rest = &rest[2..];
        } else {
            let c = rest.chars().next().unwrap();
            out.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    out.trim().to_string()
}

pub fn preprocess(source: &str) -> Preprocessed {
    let mut s = Scanner {
        src: source.as_bytes(),
        pos: 0,
        line: 1,
        stack: Vec::new(),
        items: Vec::new(),
        macros: Vec::new(),
        masked: Vec::new(),
        inactive_since: None,
    };

    s.run();

    if let Some(since) = s.inactive_since {
        s.masked.push((since, source.len()));
    }

    let mut bytes = source.as_bytes().to_vec();
    for (a, b) in &s.masked {
        for byte in &mut bytes[*a..(*b).min(source.len())] {
            if *byte != b'\n' && *byte != b'\r' {
                *byte = b' ';
            }
        }
    }

    blank_bodies(&mut bytes);

    Preprocessed {
        // Only ASCII bytes and whole characters were replaced
        text: String::from_utf8(bytes).expect("masking preserves utf-8"),
        items: s.items,
        macros: s.macros,
    }
}

/// Blanks out the contents of function bodies.
///
/// Documentation never needs statements, and blanking them keeps unusual
/// statement syntax from confusing the parser about the surrounding
/// declarations. A body is a `{` directly following a parameter list `)`,
/// which at declaration level only happens for functions, methods and
/// property accessors. `enum E (+= 1) {` is excluded.
fn blank_bodies(bytes: &mut [u8]) {
    let len = bytes.len();
    let mut i = 0;
    // Stack of the first significant byte inside each open paren
    let mut parens: Vec<usize> = Vec::new();
    // Last significant byte, and for `)` whether its group looked like an enum increment
    let mut prev: u8 = 0;
    let mut prev_enum_increment = false;

    while i < len {
        let c = bytes[i];
        match c {
            b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            b'"' | b'\'' => {
                i = skip_literal(bytes, i);
                prev = c;
                continue;
            }
            b'(' => parens.push(i + 1),
            b')' => {
                prev_enum_increment = parens.pop().map_or(false, |start| {
                    let inner = &bytes[start..i];
                    let first = inner.iter().position(|b| !b.is_ascii_whitespace());
                    first.map_or(false, |f| {
                        let rest = &inner[f..];
                        ["+=", "-=", "*=", "/=", "|=", "&=", "^=", "~=", "<<=", ">>=", "="]
                            .iter()
                            .any(|op| rest.starts_with(op.as_bytes()))
                    })
                });
            }
            b'{' if prev == b')' && !prev_enum_increment => {
                if let Some(end) = matching_brace(bytes, i) {
                    for b in &mut bytes[i + 1..end] {
                        if *b != b'\n' && *b != b'\r' {
                            *b = b' ';
                        }
                    }
                    i = end;
                    prev = b'}';
                    i += 1;
                    continue;
                }
            }
            _ => {}
        }
        if !c.is_ascii_whitespace() {
            prev = c;
        }
        i += 1;
    }
}

fn skip_literal(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'\n' => return i,
            c if c == quote => return i + 1,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// Finds the `}` closing the `{` at `open`, skipping comments and literals
fn matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let len = bytes.len();
    let mut depth = 0usize;
    let mut i = open;
    while i < len {
        match bytes[i] {
            b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            b'"' | b'\'' => {
                i = skip_literal(bytes, i);
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_inactive_branches() {
        let src = "#if defined FOO\nnative A();\n#else\nnative B();\n#endif\n";
        let p = preprocess(src);
        assert!(!p.text.contains("A()"));
        assert!(p.text.contains("native B();"));
        assert_eq!(p.text.len(), src.len());
    }

    #[test]
    fn tracks_defines() {
        let src = "#define FOO 1\n#if FOO\nnative A();\n#endif\n#undef FOO\n#if defined FOO\nnative B();\n#endif";
        let p = preprocess(src);
        assert!(p.text.contains("native A();"));
        assert!(!p.text.contains("native B();"));
        assert_eq!(p.macros.len(), 1);
        assert_eq!(p.macros[0].value, "1");
    }

    #[test]
    fn define_values() {
        let p = preprocess("#define A   (1<<0) /* doc */\n#define B\n#define C(%1) %1+1\n");
        assert_eq!(p.macros[0].value, "(1<<0)");
        assert_eq!(p.macros[1].value, "");
        assert!(p.macros[2].function_like);
        assert_eq!(p.macros[2].value, "%1+1");
    }

    #[test]
    fn blanks_function_bodies() {
        let src = "stock int Foo(int a)\n{\n    switch (a) { case 1: return 2; }\n}\nenum E (<<= 1) { A = 1, B }\n";
        let p = preprocess(src);
        assert!(p.text.contains("stock int Foo(int a)\n{\n"));
        assert!(!p.text.contains("switch"));
        assert!(p.text.contains("A = 1, B"));
    }

    #[test]
    fn endinput_stops() {
        let p = preprocess("native A();\n#endinput\nnative B();");
        assert!(!p.text.contains("B"));
    }
}
