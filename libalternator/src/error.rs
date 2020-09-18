use thiserror::Error;

pub type Result<T> = std::result::Result<T, AlternatorError>;

#[derive(Error, Debug)]
pub enum AlternatorError {
    #[error("FFI Null {0}")]
    FFINul(#[from] std::ffi::NulError),

    #[error("Docparse unable to process content")]
    ParseFail,

    #[error("Schema mismatch from alternator's strand")]
    SchemaMismatch(#[from] serde_json::error::Error),
}
