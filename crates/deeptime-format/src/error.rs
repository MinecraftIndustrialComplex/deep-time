// SPDX-License-Identifier: LGPL-3.0-or-later
//! Errors of the world-data format.

use std::fmt;
use std::path::PathBuf;

/// Everything that can go wrong reading or writing world data.
#[derive(Debug)]
pub enum FormatError {
    /// An I/O error, with the file it happened on.
    Io { path: PathBuf, source: std::io::Error },
    /// A JSON document (manifest or layer descriptor) did not parse.
    Json {
        what: String,
        source: serde_json::Error,
    },
    /// The file is not what it claims to be (magic, lengths, offsets).
    Corrupt { path: PathBuf, reason: String },
    /// A CRC32 did not match.
    Crc {
        path: PathBuf,
        what: String,
        expected: u32,
        actual: u32,
    },
    /// The world data was written by another format version (plan §5.7.2).
    Version { found: String, supported: String },
    /// A caller passed something inconsistent (wrong buffer length, bad grid).
    Invalid(String),
}

/// The format crate's result type.
pub type Result<T> = std::result::Result<T, FormatError>;

impl FormatError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        FormatError::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn corrupt(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        FormatError::Corrupt {
            path: path.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Io { path, source } => write!(f, "{}: {source}", path.display()),
            FormatError::Json { what, source } => write!(f, "{what}: {source}"),
            FormatError::Corrupt { path, reason } => {
                write!(f, "{}: corrupt world data: {reason}", path.display())
            }
            FormatError::Crc {
                path,
                what,
                expected,
                actual,
            } => write!(
                f,
                "{}: CRC32 mismatch in {what}: expected {expected:#010x}, found {actual:#010x}",
                path.display()
            ),
            FormatError::Version { found, supported } => write!(
                f,
                "world data {found}, this Deep Time reads {supported}; before 1.0 worlds are not migrated"
            ),
            FormatError::Invalid(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for FormatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FormatError::Io { source, .. } => Some(source),
            FormatError::Json { source, .. } => Some(source),
            _ => None,
        }
    }
}
