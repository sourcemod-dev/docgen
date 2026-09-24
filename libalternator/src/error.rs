use thiserror::Error;

pub type Result<T> = std::result::Result<T, AlternatorError>;

#[derive(Error, Debug)]
pub enum AlternatorError {
    #[error("Unable to load the SourcePawn grammar: {0}")]
    Language(#[from] tree_sitter::LanguageError),

    #[error("Unable to parse content")]
    ParseFail,

    #[error("Syntax error at {line}:{column} near `{snippet}`")]
    Syntax {
        line: usize,
        column: usize,
        snippet: String,
    },
}
