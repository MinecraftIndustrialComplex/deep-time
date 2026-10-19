// SPDX-License-Identifier: LGPL-3.0-or-later
//! Where a recorded history keeps the frames it does not hold in memory (P1-17; decisions log
//! 2026-09-29, "Spill history to the SSD"): frame records in a file, read back by position.
//!
//! A frame record is its fields' LZ4 blobs back to back, in the frame's field order; what the
//! blobs are (name, range, bits, length) stays in memory ([`super::recorded`]'s `FrameHead`), so a
//! reader can fetch one field (the crust-origin offsets) without the rest. Fields added to a frame
//! after it was spilled follow as a record of their own (P1-19), and the frame's head says which
//! record holds each field; a checkpoint still gets each frame as one record in field order. Two kinds of file hold
//! records:
//!
//! - a **spill file**, one per session, in the run's folder (`SessionOptions::out_dir`: the
//!   workbench's run folder, the game's staging folder or first-start work folder). It is
//!   append-only and anonymous: on Unix it is unlinked as soon as it is created, on Windows it is
//!   opened delete-on-close, so it takes disk space only while the session (or a clone of its
//!   history) holds it and leaves nothing behind after a crash;
//! - a **checkpoint** (`checkpoint.rs`), read-only, whose frame sections are records.
//!
//! Reads and writes are positional (`pread`/`pwrite`), so a prefetch thread can read while the
//! stepping thread appends.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use deeptime_format::fsutil::{read_exact_at, write_all_at};

use crate::error::CoreError;

/// A file of frame records.
pub(crate) struct FrameFile {
    file: File,
    /// The file's name when it was opened (a spill file no longer has one on Unix).
    path: PathBuf,
    /// The append position of a spill file; `None` for a read-only source.
    end: Option<Mutex<u64>>,
    /// Bytes read, and the time reads and writes took (ns): what the spill cost.
    read_bytes: AtomicU64,
    read_nanos: AtomicU64,
    write_nanos: AtomicU64,
}

impl std::fmt::Debug for FrameFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FrameFile({})", self.path.display())
    }
}

static SPILL_SEQ: AtomicU64 = AtomicU64::new(0);

impl FrameFile {
    /// A new anonymous spill file in `dir` (created if missing).
    pub fn spill_in(dir: &Path) -> Result<FrameFile, CoreError> {
        std::fs::create_dir_all(dir).map_err(|e| CoreError::io(dir, e))?;
        let path = dir.join(format!(
            "history-{}-{}.spill",
            std::process::id(),
            SPILL_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let mut o = OpenOptions::new();
        o.read(true).write(true).create_new(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // FILE_FLAG_DELETE_ON_CLOSE; share read, write and delete.
            o.custom_flags(0x0400_0000).share_mode(0x7);
        }
        let file = o.open(&path).map_err(|e| CoreError::io(&path, e))?;
        // Unix: the open handle keeps the data; the name goes now, so nothing outlives the
        // process, and moving or deleting the run's folder never trips over it.
        #[cfg(unix)]
        std::fs::remove_file(&path).map_err(|e| CoreError::io(&path, e))?;
        Ok(FrameFile {
            file,
            path,
            end: Some(Mutex::new(0)),
            read_bytes: AtomicU64::new(0),
            read_nanos: AtomicU64::new(0),
            write_nanos: AtomicU64::new(0),
        })
    }

    /// A read-only source (a checkpoint's frame sections).
    #[cfg(test)]
    pub fn open(path: &Path) -> Result<FrameFile, CoreError> {
        let file = File::open(path).map_err(|e| CoreError::io(path, e))?;
        Ok(FrameFile::read_only(file, path))
    }

    /// A read-only source from a file already open.
    pub fn read_only(file: File, path: &Path) -> FrameFile {
        FrameFile {
            file,
            path: path.to_owned(),
            end: None,
            read_bytes: AtomicU64::new(0),
            read_nanos: AtomicU64::new(0),
            write_nanos: AtomicU64::new(0),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends `parts` back to back; returns the offset of the first byte.
    pub fn append(&self, parts: &[&[u8]]) -> Result<u64, CoreError> {
        let end = self.end.as_ref().ok_or_else(|| {
            CoreError::InvalidState(format!("{} is read-only", self.path.display()))
        })?;
        let mut end = end.lock().map_err(|_| CoreError::Poisoned)?;
        let started = Instant::now();
        let start = *end;
        let mut at = start;
        for p in parts {
            write_all_at(&self.file, p, at).map_err(|e| CoreError::io(&self.path, e))?;
            at += p.len() as u64;
        }
        *end = at;
        self.write_nanos
            .fetch_add(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
        Ok(start)
    }

    /// Bytes read so far, and the seconds reads and writes took (a read-ahead's seconds overlap
    /// the reader's work).
    pub fn io_stats(&self) -> (u64, f64, f64) {
        (
            self.read_bytes.load(Ordering::Relaxed),
            self.read_nanos.load(Ordering::Relaxed) as f64 * 1e-9,
            self.write_nanos.load(Ordering::Relaxed) as f64 * 1e-9,
        )
    }

    /// `len` bytes at `offset`.
    pub fn read(&self, offset: u64, len: u64) -> Result<Vec<u8>, CoreError> {
        let len = usize::try_from(len)
            .map_err(|_| CoreError::InvalidState(format!("a {len}-byte read")))?;
        let started = Instant::now();
        let mut b = vec![0u8; len];
        read_exact_at(&self.file, &mut b, offset).map_err(|e| CoreError::io(&self.path, e))?;
        self.read_bytes.fetch_add(len as u64, Ordering::Relaxed);
        self.read_nanos
            .fetch_add(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
        Ok(b)
    }
}

/// The session's spill store: the folder its spill file goes in, created on the first eviction,
/// and what went into it.
#[derive(Debug)]
pub struct SpillStore {
    dir: PathBuf,
    file: Mutex<Option<Arc<FrameFile>>>,
    /// Bytes written to the spill file (frames spilled twice count twice).
    written: AtomicU64,
    /// Frames written.
    frames: AtomicU64,
}

impl SpillStore {
    pub fn new(dir: &Path) -> SpillStore {
        SpillStore {
            dir: dir.to_owned(),
            file: Mutex::new(None),
            written: AtomicU64::new(0),
            frames: AtomicU64::new(0),
        }
    }

    /// The spill file, created on first use; `true` when this call created it.
    pub(crate) fn file(&self) -> Result<(Arc<FrameFile>, bool), CoreError> {
        let mut f = self.file.lock().map_err(|_| CoreError::Poisoned)?;
        if let Some(f) = f.as_ref() {
            return Ok((f.clone(), false));
        }
        let made = Arc::new(FrameFile::spill_in(&self.dir)?);
        *f = Some(made.clone());
        Ok((made, true))
    }

    pub(crate) fn wrote(&self, bytes: u64) {
        self.written.fetch_add(bytes, Ordering::Relaxed);
        self.frames.fetch_add(1, Ordering::Relaxed);
    }

    /// Fields added to a frame already on disk were written (a record of their own, not a frame).
    pub(crate) fn wrote_fields(&self, bytes: u64) {
        self.written.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Bytes written to the spill file so far: the disk the session uses for its history.
    pub fn written_bytes(&self) -> u64 {
        self.written.load(Ordering::Relaxed)
    }

    /// Frames written to the spill file so far.
    pub fn written_frames(&self) -> u64 {
        self.frames.load(Ordering::Relaxed)
    }

    /// Where the spill file is (or will be) created.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The spill file's reads and writes: bytes read, seconds reading, seconds writing.
    pub fn io_stats(&self) -> (u64, f64, f64) {
        match self.file.lock() {
            Ok(f) => f.as_ref().map_or((0, 0.0, 0.0), |f| f.io_stats()),
            Err(_) => (0, 0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spill_file_appends_reads_and_leaves_no_name() {
        let dir = std::env::temp_dir().join(format!("dt-spill-{}", std::process::id()));
        let store = SpillStore::new(&dir);
        let (f, created) = store.file().unwrap();
        assert!(created);
        let (g, again) = store.file().unwrap();
        assert!(!again && Arc::ptr_eq(&f, &g));
        let a = f.append(&[b"abc", b"de"]).unwrap();
        let b = f.append(&[b"fghij"]).unwrap();
        assert_eq!((a, b), (0, 5));
        assert_eq!(f.read(3, 4).unwrap(), b"defg");
        assert!(f.read(8, 4).is_err(), "past the end");
        #[cfg(unix)]
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            0,
            "the spill file has no name on Unix"
        );
        // A read-only source cannot be appended to.
        let p = dir.join("ro.bin");
        std::fs::write(&p, b"0123456789").unwrap();
        let ro = FrameFile::open(&p).unwrap();
        assert_eq!(ro.read(2, 3).unwrap(), b"234");
        assert!(ro.append(&[b"x"]).is_err());
        drop((f, g, ro));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
