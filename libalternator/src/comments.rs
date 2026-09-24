//! Associates comments with declarations.
//!
//! This follows the rules of the SourcePawn lexer that the old docparse tool
//! relied on:
//! - A *tail* comment block starts with a single line comment that follows a
//!   token on the same line and extends over directly following comments
//!   (no blank line).
//! - A *front* comment block is a sequence of comments with at most one line
//!   between them, followed by a token on its own line.
//!
//! A declaration on line `L` is documented by the front block ending on line
//! `L - 1`, or otherwise by a tail block starting on line `L`.

use crate::preprocess::{Item, RawComment};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommentRange {
    pub start: usize,
    pub end: usize,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Default)]
pub struct Comments {
    front: Vec<CommentRange>,
    tail: Vec<CommentRange>,
}

fn range(a: &RawComment, b: &RawComment) -> CommentRange {
    CommentRange {
        start: a.start,
        end: b.end,
        start_line: a.start_line,
        end_line: b.end_line,
    }
}

impl Comments {
    pub fn new(items: &[Item]) -> Self {
        let mut c = Comments::default();
        let mut i = 0;
        let mut last_token_line = 0;

        while i < items.len() {
            let first = match items[i] {
                Item::Token { line, .. } => {
                    last_token_line = line;
                    i += 1;
                    continue;
                }
                Item::Comment(first) => first,
            };

            // Tail block. Scanning a multi-line comment resets the lexer's
            // "token seen on this line" state, so those are never tails.
            if last_token_line == first.start_line && first.start_line == first.end_line {
                let mut last = first;
                i += 1;
                while let Some(Item::Comment(next)) = items.get(i) {
                    if next.start_line > last.end_line + 1 {
                        break;
                    }
                    last = *next;
                    i += 1;
                }
                c.tail.push(range(&first, &last));
                continue;
            }

            // Front block(s)
            let start = first;
            let mut committed: Option<RawComment> = None;
            let mut last = first;
            i += 1;
            loop {
                match items.get(i) {
                    Some(Item::Comment(next)) => {
                        committed = Some(last);
                        if next.start_line > last.end_line + 1 {
                            // Block finished, the next comment starts a new one
                            break;
                        }
                        last = *next;
                        i += 1;
                    }
                    Some(Item::Token { line, directive }) => {
                        if start.start_line == *line || *directive {
                            // `/* ... */ token` is not a front comment, and
                            // comments before a directive belong to nothing
                            // (typically `@section` headers above `#define`s)
                            committed = None;
                        } else if *line != last.end_line {
                            committed = Some(last);
                        }
                        break;
                    }
                    None => {
                        committed = Some(last);
                        break;
                    }
                }
            }

            if let Some(end) = committed {
                c.front.push(range(&start, &end));
            }
        }

        c
    }

    /// Finds the documentation comment for a declaration on `line`
    pub fn find(&self, line: usize) -> Option<CommentRange> {
        if line == 0 {
            return None;
        }
        self.front
            .iter()
            .find(|r| r.end_line == line - 1)
            .or_else(|| self.tail.iter().find(|r| r.start_line == line))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preprocess::preprocess;

    fn find(src: &str, line: usize) -> Option<String> {
        let p = preprocess(src);
        let c = Comments::new(&p.items);
        c.find(line).map(|r| src[r.start..r.end].to_string())
    }

    #[test]
    fn front_comment() {
        let src = "/**\n * Doc\n */\nnative void A();\n";
        assert_eq!(find(src, 4).unwrap(), "/**\n * Doc\n */");
    }

    #[test]
    fn blank_line_breaks_association() {
        let src = "/** Doc */\n\nnative void A();\n";
        assert_eq!(find(src, 3), None);
    }

    #[test]
    fn tail_comment() {
        let src = "enum X {\n  A, // first\n  B /**< second */\n};\n";
        assert_eq!(find(src, 2).unwrap(), "// first");
        assert_eq!(find(src, 3).unwrap(), "/**< second */");
    }

    #[test]
    fn defines_use_tail_comments() {
        let src = "/** @section Flags */\n#define A 1 /**< Tail */\n// Label\n#define B 2\n";
        assert_eq!(find(src, 2).unwrap(), "/**< Tail */");
        assert_eq!(find(src, 4), None);
    }

    #[test]
    fn multi_line_trailing_comment_documents_next_line() {
        // Mirrors docparse: a multi-line comment after a token is a front comment
        let src = "enum X {\n  A, /**< one\n        two */\n  B\n};\n";
        assert_eq!(find(src, 2), None);
        assert!(find(src, 4).unwrap().starts_with("/**< one"));
    }

    #[test]
    fn consecutive_line_comments() {
        let src = "// a\n// b\nnative void A();\n";
        assert_eq!(find(src, 3).unwrap(), "// a\n// b");
    }
}
