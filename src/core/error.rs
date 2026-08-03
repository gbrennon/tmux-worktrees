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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_from_string() {
        let err = Error::new("boom".to_string());
        assert_eq!(err.to_string(), "boom");
    }

    #[test]
    fn new_from_str() {
        let err = Error::new("boom");
        assert_eq!(err.to_string(), "boom");
    }

    #[test]
    fn display_formats_message() {
        let err = Error::new("something went wrong");
        assert_eq!(format!("{err}"), "something went wrong");
    }

    #[test]
    fn debug_includes_message() {
        let err = Error::new("bad");
        let debug = format!("{err:?}");
        assert!(
            debug.contains("bad"),
            "debug output should contain the message: {debug}"
        );
    }

    #[test]
    fn std_error_source_returns_none() {
        let err = Error::new("nope");
        let boxed: Box<dyn std::error::Error> = Box::new(err);
        assert!(boxed.source().is_none());
    }

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::from(io);
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn from_io_error_via_into() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err: Error = io.into();
        assert!(err.to_string().contains("denied"));
    }

    #[test]
    fn from_string() {
        let err = Error::from("owned".to_string());
        assert_eq!(err.to_string(), "owned");
    }

    #[test]
    fn from_string_via_into() {
        let err: Error = "owned".to_string().into();
        assert_eq!(err.to_string(), "owned");
    }

    #[test]
    fn from_str_ref() {
        let err = Error::from("static str");
        assert_eq!(err.to_string(), "static str");
    }

    #[test]
    fn from_str_ref_via_into() {
        let err: Error = "static str".into();
        assert_eq!(err.to_string(), "static str");
    }

    #[test]
    fn result_ok_variant_works() {
        let res: Result<i32> = Ok(7);
        assert_eq!(res.unwrap(), 7);
    }

    #[test]
    fn result_err_variant_works() {
        let res: Result<i32> = Err(Error::new("fail"));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "fail");
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }
}
