use std::ffi::NulError;
use std::fmt;

use reqwest::Error as RequestError;
use serde_json::Error as JsonError;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    FFINul(NulError),
    ParseFailed,
    RequestError(RequestError),
    SchemaMismatch(JsonError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            Error::FFINul(_) => write!(f, "A null byte is present in the data"),
            Error::ParseFailed => write!(f, "Initial parsing during ffi stage has failed"),
            Error::RequestError(e) => write!(f, "Failed to fetch raw include file: {}", e),
            Error::SchemaMismatch(e) => {
                write!(f, "Initial parsed is different from local schema: {}", e)
            }
        }
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Error {
        Error::FFINul(err)
    }
}

impl From<RequestError> for Error {
    fn from(err: RequestError) -> Error {
        Error::RequestError(err)
    }
}
