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

/// Writes `path` atomically like [`write_atomic`], streaming: `write` fills `path.tmp` (opened
/// for reading and writing, so it can seek back and patch a header), then fsync, rename, fsync of
/// the directory. A failed `write` removes the temporary file. Generic over the caller's error
/// type (anything a [`FormatError`] converts into).
pub fn write_atomic_with<E: From<FormatError>>(
    path: &Path,
    write: impl FnOnce(&mut File) -> std::result::Result<(), E>,
) -> std::result::Result<(), E> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| FormatError::io(parent, e))?;
    let tmp = tmp_path(path);
    let done = (|| {
        let mut f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp)
            .map_err(|e| FormatError::io(&tmp, e))?;
        write(&mut f)?;
        f.sync_all().map_err(|e| E::from(FormatError::io(&tmp, e)))
    })();
    if let Err(e) = done {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    fs::rename(&tmp, path).map_err(|e| FormatError::io(path, e))?;
    sync_dir(parent);
    Ok(())
}

/// Reads exactly `buf.len()` bytes of `file` at `offset` without moving a shared cursor (`pread`),
/// so several threads can read one handle at once.
pub fn read_exact_at(file: &File, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::FileExt::read_exact_at(file, buf, offset)
    }
    #[cfg(windows)]
    {
        let mut done = 0usize;
        while done < buf.len() {
            let n = std::os::windows::fs::FileExt::seek_read(
                file,
                &mut buf[done..],
                offset + done as u64,
            )?;
            if n == 0 {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }
            done += n;
        }
        Ok(())
    }
}

/// Writes all of `buf` into `file` at `offset` (`pwrite`).
pub fn write_all_at(file: &File, buf: &[u8], offset: u64) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::FileExt::write_all_at(file, buf, offset)
    }
    #[cfg(windows)]
    {
        let mut done = 0usize;
        while done < buf.len() {
            let n = std::os::windows::fs::FileExt::seek_write(
                file,
                &buf[done..],
                offset + done as u64,
            )?;
            if n == 0 {
                return Err(std::io::ErrorKind::WriteZero.into());
            }
            done += n;
        }
        Ok(())
    }
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

        // Streaming: the writer can seek back; a failed writer leaves neither file.
        use std::io::{Seek, SeekFrom};
        let q = dir.join("c.bin");
        write_atomic_with::<FormatError>(&q, |f| {
            f.write_all(b"xxxxworld")
                .map_err(|e| FormatError::io("c", e))?;
            f.seek(SeekFrom::Start(0))
                .map_err(|e| FormatError::io("c", e))?;
            f.write_all(b"hell").map_err(|e| FormatError::io("c", e))
        })
        .unwrap();
        assert_eq!(fs::read(&q).unwrap(), b"hellworld");
        let r = dir.join("d.bin");
        assert!(write_atomic_with(&r, |_| Err(FormatError::Invalid("no".into()))).is_err());
        assert!(!r.exists() && !tmp_path(&r).exists());

        // Positional reads and writes.
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&q)
            .unwrap();
        write_all_at(&f, b"W", 4).unwrap();
        let mut buf = [0u8; 5];
        read_exact_at(&f, &mut buf, 4).unwrap();
        assert_eq!(&buf, b"World");
        assert!(read_exact_at(&f, &mut [0u8; 4], 7).is_err(), "past the end");
        fs::remove_dir_all(dir).unwrap();
    }
}
