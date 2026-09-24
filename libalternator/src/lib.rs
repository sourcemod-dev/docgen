//! SourcePawn include parser.
//!
//! Parses a single include (or plugin) file into a documentation [`Strand`]
//! using [tree-sitter-sourcepawn](https://github.com/nilshelmig/tree-sitter-sourcepawn)
//! together with a small preprocessor for conditional compilation and
//! `#define`s. `#include` directives are not followed; every file is
//! documented on its own.

use tree_sitter::{Node, Parser};

use schema::bundle::Strand;

mod comments;
mod error;
mod expr;
mod extract;
mod preprocess;

pub use error::{AlternatorError, Result};

/// Declaration keywords. A top level syntax error containing one of these
/// most likely swallowed a declaration, so the file can't be trusted.
const DECLARATION_KEYWORDS: &[&str] = &[
    "native", "forward", "methodmap", "enum", "typedef", "typeset", "functag", "funcenum",
    "property",
];

/// Parses SourcePawn source into a documentation strand.
///
/// Syntax errors inside function bodies are tolerated. Errors that may have
/// swallowed declarations fail the whole file, so that callers walking
/// history don't mistake a failed parse for removed symbols.
pub fn parse(source: &str) -> Result<Strand> {
    let pp = preprocess::preprocess(source);

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_sourcepawn::language())?;

    let tree = parser
        .parse(pp.text.as_bytes(), None)
        .ok_or(AlternatorError::ParseFail)?;
    let root = tree.root_node();

    if let Some(err) = find_fatal_error(&root, &pp.text) {
        return Err(err);
    }

    let mut extractor = extract::Extractor::new(source, &pp);
    extractor.run(&root);

    Ok(extractor.strand)
}

/// Parses raw file content. Invalid UTF-8 is replaced.
pub fn parse_bytes(content: &[u8]) -> Result<Strand> {
    match std::str::from_utf8(content) {
        Ok(s) => parse(s),
        Err(_) => parse(&String::from_utf8_lossy(content)),
    }
}

/// Parses an include file into a strand.
///
/// Kept for compatibility with the previous docparse based implementation;
/// `atom` (the file name) is unused.
pub async fn consume<T: Into<Vec<u8>>>(_atom: T, content: Vec<u8>) -> Result<Strand> {
    parse_bytes(&content)
}

fn find_fatal_error(root: &Node, text: &str) -> Option<AlternatorError> {
    let mut stack = vec![*root];

    while let Some(node) = stack.pop() {
        // Errors in function bodies don't affect declarations
        if node.kind() == "block" || !node.has_error() && !node.is_missing() {
            continue;
        }

        // Missing tokens (such as the type of an untyped `...`) are
        // recovered by tree-sitter without losing declarations
        if node.is_missing() {
            continue;
        }

        if node.is_error() {
            let snippet = &text[node.start_byte()..node.end_byte()];
            let top_level = node.parent().map_or(true, |p| p.kind() == "source_file");
            let has_keyword = snippet
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .any(|w| DECLARATION_KEYWORDS.contains(&w));

            // Top level errors without declaration keywords are typically
            // legacy global variables, which aren't documented anyway.
            // Anything inside a declaration may have corrupted it.
            if !top_level || has_keyword {
                let pos = node.start_position();
                return Some(AlternatorError::Syntax {
                    line: pos.row + 1,
                    column: pos.column + 1,
                    snippet: snippet.chars().take(80).collect(),
                });
            }
            continue;
        }

        let mut c = node.walk();
        stack.extend(node.children(&mut c));
    }

    None
}
