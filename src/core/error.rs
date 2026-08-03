use std::fmt;

/// Lightweight error type — a single string message.
///
/// Replaces `anyhow::Error` so the crate has zero required dependencies
/// beyond the standard library.  Implements [`From`] for `io::Error`,
/// `String`, and `&str` so `?` and `into()` work naturally.
#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(msg: impl Into<String>) -> Self {
        Error {
            message: msg.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error {
            message: e.to_string(),
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error { message: s }
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error {
            message: s.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
