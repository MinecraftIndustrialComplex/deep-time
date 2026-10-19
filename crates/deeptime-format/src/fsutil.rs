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

/// Atomic writes whose fsyncs run beside the caller's next work (P1-19). [`Flusher::write`]
/// fills `<name>.tmp` and hands it to a thread of its own that fsyncs it; [`Flusher::commit`]
/// waits for those, renames each file over its name in the order written and fsyncs each folder
/// once. A file reaches its name only complete and on disk, as with [`write_atomic`], while the
/// disk writes one file as the caller computes the next. Until the commit a written file is only
/// at its temporary name; a flusher dropped without one removes its temporary files.
#[derive(Default)]
pub struct Flusher {
    pending: Vec<Pending>,
}

struct Pending {
    tmp: PathBuf,
    path: PathBuf,
    sync: Option<std::thread::JoinHandle<std::io::Result<()>>>,
}

impl Flusher {
    pub fn new() -> Flusher {
        Flusher::default()
    }

    /// Writes `path` (at its temporary name until [`Flusher::commit`]) with `write`, which gets the
    /// file opened for reading and writing, and starts its fsync. A failed `write` removes the
    /// temporary file. Returns `write`'s value.
    pub fn write<T, E: From<FormatError>>(
        &mut self,
        path: &Path,
        write: impl FnOnce(&mut File) -> std::result::Result<T, E>,
    ) -> std::result::Result<T, E> {
        let parent = path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(parent).map_err(|e| FormatError::io(parent, e))?;
        let tmp = tmp_path(path);
        let mut f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp)
            .map_err(|e| FormatError::io(&tmp, e))?;
        let v = match write(&mut f) {
            Ok(v) => v,
            Err(e) => {
                drop(f);
                let _ = fs::remove_file(&tmp);
                return Err(e);
            }
        };
        // No thread to spare: the file (closed with the failed spawn) is synced at the commit.
        let sync = std::thread::Builder::new()
            .name("deeptime-fsync".into())
            .spawn(move || f.sync_all())
            .ok();
        self.pending.push(Pending {
            tmp,
            path: path.to_owned(),
            sync,
        });
        Ok(v)
    }

    /// Waits for every fsync, then renames the files into place (in the order written) and
    /// fsyncs their folders. On an error no file is renamed and the temporary files are removed.
    pub fn commit(mut self) -> Result<()> {
        let mut pending = std::mem::take(&mut self.pending);
        let mut failed = None;
        for p in &mut pending {
            let done = match p.sync.take() {
                Some(h) => h
                    .join()
                    .unwrap_or_else(|_| Err(std::io::Error::other("the fsync thread panicked"))),
                None => File::open(&p.tmp).and_then(|f| f.sync_all()),
            };
            if let Err(e) = done
                && failed.is_none()
            {
                failed = Some(FormatError::io(&p.tmp, e));
            }
        }
        if let Some(e) = failed {
            for p in &pending {
                let _ = fs::remove_file(&p.tmp);
            }
            return Err(e);
        }
        let mut dirs: Vec<&Path> = Vec::new();
        for p in &pending {
            fs::rename(&p.tmp, &p.path).map_err(|e| FormatError::io(&p.path, e))?;
            let parent = p.path.parent().unwrap_or(Path::new("."));
            if !dirs.contains(&parent) {
                dirs.push(parent);
            }
        }
        for d in dirs {
            sync_dir(d);
        }
        Ok(())
    }
}

impl Drop for Flusher {
    fn drop(&mut self) {
        for p in self.pending.drain(..) {
            if let Some(h) = p.sync {
                let _ = h.join();
            }
            let _ = fs::remove_file(&p.tmp);
        }
    }
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

        // A flusher's files reach their names at the commit, in full; a failed write and a
        // flusher dropped uncommitted leave nothing.
        let (e1, e2) = (dir.join("e/1.bin"), dir.join("e/2.bin"));
        let mut fl = Flusher::new();
        fl.write::<_, FormatError>(&e1, |f| {
            f.write_all(b"one").map_err(|e| FormatError::io("e1", e))
        })
        .unwrap();
        let v = fl
            .write::<_, FormatError>(&e2, |f| {
                f.write_all(b"two").map_err(|e| FormatError::io("e2", e))?;
                Ok(7)
            })
            .unwrap();
        assert_eq!(v, 7);
        assert!(
            fl.write::<(), _>(&r, |_| Err(FormatError::Invalid("no".into())))
                .is_err()
        );
        assert!(!r.exists() && !tmp_path(&r).exists());
        assert!(
            !e1.exists() && tmp_path(&e1).exists(),
            "only the temporary file before the commit"
        );
        fl.commit().unwrap();
        assert_eq!(fs::read(&e1).unwrap(), b"one");
        assert_eq!(fs::read(&e2).unwrap(), b"two");
        assert!(!tmp_path(&e1).exists() && !tmp_path(&e2).exists());
        let e3 = dir.join("e/3.bin");
        let mut fl = Flusher::new();
        fl.write::<_, FormatError>(&e3, |f| {
            f.write_all(b"3").map_err(|e| FormatError::io("e3", e))
        })
        .unwrap();
        drop(fl);
        assert!(!e3.exists() && !tmp_path(&e3).exists());

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
