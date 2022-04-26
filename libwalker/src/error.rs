use thiserror::Error;

pub type Result<T> = std::result::Result<T, WalkerError>;

#[derive(Error, Debug)]
pub enum WalkerError {
    #[error("Git library error {0}")]
    LibGit(#[from] git2::Error),

    #[error("Pathspec error, unable to convert to file name while walking")]
    InvalidPath,
}
