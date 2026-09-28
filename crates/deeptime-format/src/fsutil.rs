// SPDX-License-Identifier: LGPL-3.0-or-later
//! Crash-safe file writes: every world-data file is written to `<name>.tmp`, fsynced and renamed
//! over its final name, and the manifest goes last (plan §5.7.1), so a folder with a manifest
//! always holds complete files.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{FormatError, Result};

/// Writes `bytes` to `path` atomically: `path.tmp`, fsync, rename, fsync of the directory.
/// Returns the file's CRC32.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<u32> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| FormatError::io(parent, e))?;
    let tmp = tmp_path(path);
    {
        let mut f = File::create(&tmp).map_err(|e| FormatError::io(&tmp, e))?;
        f.write_all(bytes).map_err(|e| FormatError::io(&tmp, e))?;
        f.sync_all().map_err(|e| FormatError::io(&tmp, e))?;
    }
    fs::rename(&tmp, path).map_err(|e| FormatError::io(path, e))?;
    sync_dir(parent);
    Ok(crc32fast::hash(bytes))
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Makes a rename durable. Directories cannot be opened for syncing on Windows; there the
/// rename is as durable as the OS makes it.
fn sync_dir(dir: &Path) {
    #[cfg(unix)]
    if let Ok(d) = File::open(dir) {
        let _ = d.sync_all();
    }
    #[cfg(not(unix))]
    let _ = dir;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_leaves_no_tmp() {
        let dir = std::env::temp_dir().join(format!("dt-fsutil-{}", std::process::id()));
        let p = dir.join("a/b.bin");
        let crc = write_atomic(&p, b"hello").unwrap();
        assert_eq!(fs::read(&p).unwrap(), b"hello");
        assert_eq!(crc, crc32fast::hash(b"hello"));
        assert!(!tmp_path(&p).exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
