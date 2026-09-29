// SPDX-License-Identifier: LGPL-3.0-or-later
//! One error type for the tool: a message with its context already formatted in.

use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub struct Error(pub String);

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(msg: impl Into<String>) -> Error {
        Error(msg.into())
    }

    /// An I/O error on `path`.
    pub fn io(path: &Path, e: std::io::Error) -> Error {
        Error(format!("{}: {e}", path.display()))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error(format!("JSON: {e}"))
    }
}

impl From<deeptime_format::FormatError> for Error {
    fn from(e: deeptime_format::FormatError) -> Error {
        Error(format!("world data: {e}"))
    }
}

impl From<png::EncodingError> for Error {
    fn from(e: png::EncodingError) -> Error {
        Error(format!("PNG: {e}"))
    }
}

impl From<png::DecodingError> for Error {
    fn from(e: png::DecodingError) -> Error {
        Error(format!("PNG: {e}"))
    }
}

/// `bail!("...")`: return an [`Error`] built with `format!`.
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::error::Error(format!($($arg)*)))
    };
}
