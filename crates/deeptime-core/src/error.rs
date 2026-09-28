// SPDX-License-Identifier: LGPL-3.0-or-later
//! The core's error type. `deeptime-jni` maps each variant to a Java exception (plan §6.3); the
//! CLI prints it.

use std::fmt;
use std::path::PathBuf;

/// Everything a session, a stage or a preset parser can fail with.
#[derive(Debug)]
pub enum CoreError {
    /// The preset (or content bundle) is malformed. `path` is a JSON path such as `$.dummy.steps`.
    PresetInvalid { path: String, reason: String },
    /// A stage would exceed the session's native memory cap (plan §5.7.5).
    MemoryCap { needed: u64, cap: u64 },
    /// A file operation failed.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The session was cancelled; raised by calls that need a finished run (`finalize_write`).
    Cancelled,
    /// A call that the session's state does not allow (advance after a failure, finalize before
    /// the run is done).
    InvalidState(String),
    /// A stage panicked while holding the step lock; the session cannot be stepped again.
    Poisoned,
    /// An entry point that exists in the API but has no implementation yet in this phase, or a
    /// feature this session's preset does not produce (keyframes of a preset without any).
    Unsupported(&'static str),
    /// A caller passed an argument out of range (a keyframe index, a short buffer).
    InvalidArgument(String),
    /// World data (events, sites, keyframes, manifest) or a checkpoint could not be written or
    /// read; `deeptime-jni` maps it to `DeepTimeIoException` like [`CoreError::Io`].
    Format(deeptime_format::FormatError),
}

impl CoreError {
    pub fn preset(path: impl Into<String>, reason: impl Into<String>) -> Self {
        CoreError::PresetInvalid {
            path: path.into(),
            reason: reason.into(),
        }
    }

    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        CoreError::Io {
            path: path.into(),
            source,
        }
    }
}

impl From<deeptime_format::FormatError> for CoreError {
    fn from(e: deeptime_format::FormatError) -> Self {
        CoreError::Format(e)
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::PresetInvalid { path, reason } => write!(f, "{path}: {reason}"),
            CoreError::MemoryCap { needed, cap } => {
                write!(f, "needed {needed} bytes, cap {cap} bytes")
            }
            CoreError::Io { path, source } => write!(f, "{}: {source}", path.display()),
            CoreError::Cancelled => f.write_str("the session was cancelled"),
            CoreError::InvalidState(msg) => f.write_str(msg),
            CoreError::Poisoned => f.write_str("the session panicked earlier and is poisoned"),
            CoreError::Unsupported(what) => write!(f, "{what} is not supported yet"),
            CoreError::InvalidArgument(msg) => f.write_str(msg),
            CoreError::Format(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CoreError::Io { source, .. } => Some(source),
            CoreError::Format(e) => Some(e),
            _ => None,
        }
    }
}
