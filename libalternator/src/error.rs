use thiserror::Error;

pub type Result<T> = std::result::Result<T, AlternatorError>;

#[derive(Error, Debug)]
pub enum AlternatorError {
    #[error("FFI Null {0}")]
    FFINul(#[from] std::ffi::NulError),
}
