// SPDX-License-Identifier: LGPL-3.0-or-later
//! A field history recorded by the real pipeline (P1-04): the frames the history stages publish
//! for the deposit engine (`fields.rs`), kept in the world state so a checkpoint carries them and
//! pass 2 can replay them.
//!
//! Frames are packed: each field is quantised over its own range in that frame — 16 bits for the
//! elevation and the erosion and deposition rates, 8 bits for the rest (code 0 is NaN) — and
//! LZ4-compressed (most facies and arc fields are zero over most of the planet and shrink to almost
//! nothing). The error is at
//! most half a step of the frame's range: 1/131,070 of it at 16 bits, 1/508 at 8 bits (1 °C of
//! burial temperature, 12 m of elevation-sized ranges). 16k quick: about 6.5 MiB per frame.
//!
//! **Memory and disk (P1-17; decisions log 2026-09-29, "Spill history to the SSD").** The history
//! is the one part of a run that grows with its length (a frame every `surface.frame_every_myr`).
//! Frames are held in memory while the session's memory cap allows; the session evicts the least
//! recently touched ones to its spill file ([`super::spill`]) when a stage's `reserve` would
//! otherwise fail, or when the gauge nears the cap after a step, and they are read back from disk
//! when a reader asks. A frame on disk is its fields' LZ4 blobs back to back; its head (names,
//! ranges, lengths) stays in memory, so the crust-origin offsets are read without the rest of the
//! frame. A frame read from disk is not cached: memory stays flat, the OS's page cache does the
//! caching, and a reader walking frames in order (the deposit engine's two passes) gets the next
//! frame read ahead on a thread of its own while it works on this one. Where a frame lives never
//! changes what it holds: the same bytes are written and read back, so a run is bit-identical
//! whatever its cap.

use std::collections::BTreeMap;
use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

use deeptime_format::GridDesc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::fields::{
    FieldFrame, FieldHistory, HOST_PREFIX, ORIGIN_FIELDS, origins_from_offsets, spec,
};
use super::spill::{FrameFile, SpillStore};
use crate::error::CoreError;

/// One packed field of a frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackedField {
    pub name: String,
    pub lo: f32,
    pub hi: f32,
    /// 8 or 16.
    pub bits: u8,
    /// LZ4 (size-prepended) of the codes, little-endian.
    pub data: Vec<u8>,
}

/// One packed frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackedFrame {
    pub t_years: f64,
    pub dt_years: f64,
    pub fields: Vec<PackedField>,
    pub clocks: BTreeMap<String, f32>,
}

impl PackedFrame {
    /// Bytes of packed field data.
    pub fn data_bytes(&self) -> u64 {
        self.fields.iter().map(|f| f.data.len() as u64).sum()
    }
}

/// A packed field without its data: what reading its blob from a frame record takes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldHead {
    pub name: String,
    pub lo: f32,
    pub hi: f32,
    pub bits: u8,
    /// Bytes of the field's LZ4 blob.
    pub len: u64,
}

/// A packed frame without its data. The frame's record (on disk, or a checkpoint's frame section)
/// is the fields' blobs back to back, in this order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameHead {
    pub t_years: f64,
    pub dt_years: f64,
    pub fields: Vec<FieldHead>,
    pub clocks: BTreeMap<String, f32>,
}

impl FrameHead {
    fn of(p: &PackedFrame) -> FrameHead {
        FrameHead {
            t_years: p.t_years,
            dt_years: p.dt_years,
            fields: p
                .fields
                .iter()
                .map(|f| FieldHead {
                    name: f.name.clone(),
                    lo: f.lo,
                    hi: f.hi,
                    bits: f.bits,
                    len: f.data.len() as u64,
                })
                .collect(),
            clocks: p.clocks.clone(),
        }
    }

    /// Bytes of the frame's record.
    pub fn record_len(&self) -> u64 {
        self.fields.iter().map(|f| f.len).sum()
    }

    /// The frame, its fields' data cut from `record`.
    fn with_data(&self, record: &[u8]) -> PackedFrame {
        let mut at = 0usize;
        let fields = self
            .fields
            .iter()
            .map(|f| {
                let data = record[at..at + f.len as usize].to_vec();
                at += f.len as usize;
                PackedField {
                    name: f.name.clone(),
                    lo: f.lo,
                    hi: f.hi,
                    bits: f.bits,
                    data,
                }
            })
            .collect();
        PackedFrame {
            t_years: self.t_years,
            dt_years: self.dt_years,
            fields,
            clocks: self.clocks.clone(),
        }
    }
}

/// A history without its frames' data: what a checkpoint's body carries (the data follows in its
/// frame sections, `checkpoint.rs`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoryHead {
    pub grid: GridDesc,
    pub receivers: Option<Vec<u32>>,
    pub frames: Vec<FrameHead>,
}

/// Where a frame's record is, when a history is rebuilt from a checkpoint.
pub(crate) enum Record {
    /// In memory (a checkpoint decoded from bytes).
    Bytes(Vec<u8>),
    /// In a file, with the record's CRC32 (a checkpoint read from disk).
    File {
        file: Arc<FrameFile>,
        offset: u64,
        crc32: u32,
    },
}

/// A frame whose record is in a file.
#[derive(Debug)]
struct DiskFrame {
    head: FrameHead,
    file: Arc<FrameFile>,
    offset: u64,
    crc32: u32,
}

impl DiskFrame {
    /// The whole record, checked against its CRC32.
    fn record(&self) -> Result<Vec<u8>, CoreError> {
        let b = self.file.read(self.offset, self.head.record_len())?;
        self.check(&b)?;
        Ok(b)
    }

    fn check(&self, record: &[u8]) -> Result<(), CoreError> {
        let actual = crc32fast::hash(record);
        if actual != self.crc32 {
            return Err(CoreError::InvalidState(format!(
                "history frame at t={} in {} (offset {}): CRC32 {actual:08x}, expected {:08x}",
                self.head.t_years,
                self.file.path().display(),
                self.offset,
                self.crc32
            )));
        }
        Ok(())
    }

    /// Field `i`'s blob alone (not CRC-checked: the whole record is checked when it is read).
    fn field(&self, i: usize) -> Result<Vec<u8>, CoreError> {
        let at: u64 = self.head.fields[..i].iter().map(|f| f.len).sum();
        self.file.read(self.offset + at, self.head.fields[i].len)
    }
}

#[derive(Clone, Debug)]
enum Slot {
    Memory {
        frame: Arc<PackedFrame>,
        /// When the frame was last written (pushed or added to): eviction takes the oldest.
        touched: u64,
    },
    Disk(Arc<DiskFrame>),
}

impl Slot {
    fn bytes(&self) -> u64 {
        match self {
            Slot::Memory { frame, .. } => frame.data_bytes(),
            Slot::Disk(d) => d.head.record_len(),
        }
    }
}

/// A record being read ahead.
struct Prefetch {
    /// The record being read (used only if frame `k` still is this record when it is asked for).
    frame: Arc<DiskFrame>,
    job: JoinHandle<Result<Vec<u8>, CoreError>>,
}

/// The frames and where they live (shared with the session's eviction hook, [`HistoryHandle`]).
#[derive(Default)]
struct Frames {
    slots: Vec<Slot>,
    /// Where evicted frames go; `None`: nothing is evicted (a history outside a session, a clone).
    store: Option<Arc<SpillStore>>,
    tick: u64,
    /// The last frame [`FieldHistory::frame`] returned: reading `k + 1` next is a scan, and the
    /// frame after it is read ahead.
    last_read: Option<usize>,
    prefetch: Option<Prefetch>,
}

/// What an eviction did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Evicted {
    /// Frames moved to disk.
    pub frames: u32,
    /// Their packed bytes (the memory freed, unless a clone of the world still holds them).
    pub bytes: u64,
    /// The eviction created the session's spill file.
    pub created_file: bool,
}

impl Frames {
    fn resident_bytes(&self) -> u64 {
        self.slots
            .iter()
            .filter(|s| matches!(s, Slot::Memory { .. }))
            .map(Slot::bytes)
            .sum()
    }

    /// Moves the least recently touched frames to disk until `bytes` of packed data left memory
    /// (or no frame is left in memory, or there is no store).
    fn evict(&mut self, bytes: u64) -> Result<Evicted, CoreError> {
        let mut out = Evicted::default();
        let Some(store) = self.store.clone() else {
            return Ok(out);
        };
        let mut order: Vec<(u64, usize)> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(k, s)| match s {
                Slot::Memory { touched, .. } => Some((*touched, k)),
                Slot::Disk(_) => None,
            })
            .collect();
        order.sort_unstable();
        for (_, k) in order {
            if out.bytes >= bytes {
                break;
            }
            let Slot::Memory { frame, .. } = &self.slots[k] else {
                continue;
            };
            let (file, created) = store.file()?;
            out.created_file |= created;
            let parts: Vec<&[u8]> = frame.fields.iter().map(|f| f.data.as_slice()).collect();
            let mut crc = crc32fast::Hasher::new();
            for p in &parts {
                crc.update(p);
            }
            let offset = file.append(&parts)?;
            let head = FrameHead::of(frame);
            let len = head.record_len();
            store.wrote(len);
            self.slots[k] = Slot::Disk(Arc::new(DiskFrame {
                head,
                file,
                offset,
                crc32: crc.finalize(),
            }));
            out.frames += 1;
            out.bytes += len;
        }
        Ok(out)
    }
}

/// The recorded history of one run.
pub struct RecordedHistory {
    pub grid: GridDesc,
    /// Present-day drainage (L1 cell → downstream cell), set when the history ends.
    pub receivers: Option<Vec<u32>>,
    frames: Arc<Mutex<Frames>>,
}

/// The session's hold on a world's history: evicts frames when a stage needs the memory
/// (`StageContext::reserve`) and after steps (`Session::advance`).
#[derive(Clone)]
pub(crate) struct HistoryHandle(Arc<Mutex<Frames>>);

impl HistoryHandle {
    /// Packed bytes of the frames in memory: what evicting everything would free.
    pub fn resident_bytes(&self) -> u64 {
        match self.0.try_lock() {
            Ok(f) => f.resident_bytes(),
            Err(_) => 0,
        }
    }

    /// Evicts at least `bytes` of frames (the oldest first) if the history can. Never waits: a
    /// history busy on this thread evicts nothing.
    pub fn evict(&self, bytes: u64) -> Result<Evicted, CoreError> {
        match self.0.try_lock() {
            Ok(mut f) => f.evict(bytes),
            Err(std::sync::TryLockError::Poisoned(p)) => p.into_inner().evict(bytes),
            Err(std::sync::TryLockError::WouldBlock) => Ok(Evicted::default()),
        }
    }

    #[cfg(test)]
    pub fn is(&self, h: &RecordedHistory) -> bool {
        Arc::ptr_eq(&self.0, &h.frames)
    }
}

/// Fields kept at 16 bits: the ones whose small values matter over a wide range (elevation for
/// the ledger's burial, the heavy-tailed rates). Everything else is 8-bit over the frame's range.
const WIDE: [&str; 3] = [
    "deeptime:elevation_m",
    "deeptime:erosion_rate_m_myr",
    "deeptime:deposition_rate_m_myr",
];

fn bits_for(name: &str) -> u8 {
    // Host-rock fractions (`deeptime:host/<class>`, 0–1) are an open family outside the specs.
    let known = spec(name).is_some() || name.starts_with(HOST_PREFIX);
    if WIDE.contains(&name) || !known {
        16
    } else {
        8
    }
}

fn pack(name: &str, v: &[f32]) -> PackedField {
    let bits = bits_for(name);
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for &x in v {
        if x.is_finite() {
            lo = lo.min(x);
            hi = hi.max(x);
        }
    }
    if !lo.is_finite() {
        (lo, hi) = (0.0, 0.0);
    }
    let max = if bits == 8 { 255.0 } else { 65535.0 };
    let span = f64::from(hi) - f64::from(lo);
    let code = |x: f32| -> u32 {
        if !x.is_finite() {
            0
        } else if span <= 0.0 {
            1
        } else {
            1 + ((f64::from(x) - f64::from(lo)) / span * (max - 1.0)).round() as u32
        }
    };
    let raw: Vec<u8> = if bits == 8 {
        v.iter().map(|&x| code(x) as u8).collect()
    } else {
        v.iter()
            .flat_map(|&x| (code(x) as u16).to_le_bytes())
            .collect()
    };
    PackedField {
        name: name.to_owned(),
        lo,
        hi,
        bits,
        data: lz4_flex::compress_prepend_size(&raw),
    }
}

fn unpack(
    name: &str,
    lo: f32,
    hi: f32,
    bits: u8,
    data: &[u8],
    cells: usize,
) -> Result<Vec<f32>, CoreError> {
    let raw = lz4_flex::decompress_size_prepended(data)
        .map_err(|e| CoreError::InvalidState(format!("history field {name}: {e}")))?;
    let max = if bits == 8 { 255.0 } else { 65535.0 };
    let span = f64::from(hi) - f64::from(lo);
    let value = |code: u32| -> f32 {
        if code == 0 {
            f32::NAN
        } else if span <= 0.0 {
            lo
        } else {
            (f64::from(lo) + f64::from(code - 1) / (max - 1.0) * span) as f32
        }
    };
    let out: Vec<f32> = if bits == 8 {
        raw.iter().map(|&b| value(u32::from(b))).collect()
    } else {
        raw.as_chunks::<2>()
            .0
            .iter()
            .map(|b| value(u32::from(u16::from_le_bytes(*b))))
            .collect()
    };
    if out.len() != cells {
        return Err(CoreError::InvalidState(format!(
            "history field {name}: {} cells, the grid has {cells}",
            out.len()
        )));
    }
    Ok(out)
}

/// Unpacks every field (in parallel on the caller's pool; each field on its own, so the result
/// does not depend on the thread count).
fn unpack_all(
    fields: &[(&FieldHead, &[u8])],
    cells: usize,
) -> Result<BTreeMap<String, Vec<f32>>, CoreError> {
    fields
        .par_iter()
        .map(|(h, d)| unpack(&h.name, h.lo, h.hi, h.bits, d, cells).map(|v| (h.name.clone(), v)))
        .collect()
}

impl RecordedHistory {
    pub fn new(grid: GridDesc) -> RecordedHistory {
        RecordedHistory {
            grid,
            receivers: None,
            frames: Arc::new(Mutex::new(Frames::default())),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Frames> {
        self.frames.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Packs and appends `frame` (fields in name order).
    pub fn push(&mut self, frame: &FieldFrame) {
        let fields: Vec<(&String, &Vec<f32>)> = frame.fields.iter().collect();
        let fields = fields.par_iter().map(|(name, v)| pack(name, v)).collect();
        let packed = PackedFrame {
            t_years: frame.t_years,
            dt_years: frame.dt_years,
            fields,
            clocks: frame.clocks.clone(),
        };
        let mut f = self.lock();
        f.tick += 1;
        let touched = f.tick;
        f.slots.push(Slot::Memory {
            frame: Arc::new(packed),
            touched,
        });
    }

    /// Adds fields to frame `k` (packed like [`RecordedHistory::push`]), replacing fields of the
    /// same name: stages that step after the one that closed the frame publish into it this way
    /// (P1-07's `trap_structure` and host fractions). Keeps the frame's fields in name order. A
    /// frame on disk comes back into memory.
    pub fn add_fields(
        &mut self,
        k: usize,
        fields: &BTreeMap<String, Vec<f32>>,
    ) -> Result<(), CoreError> {
        let cells = self.grid.cells();
        if k >= self.frame_count() {
            return Err(CoreError::InvalidArgument(format!("no frame {k}")));
        }
        for (name, v) in fields {
            if v.len() != cells {
                return Err(CoreError::InvalidArgument(format!(
                    "history field {name}: {} cells, the grid has {cells}",
                    v.len()
                )));
            }
        }
        let list: Vec<(&String, &Vec<f32>)> = fields.iter().collect();
        let packed: Vec<PackedField> = list.par_iter().map(|(name, v)| pack(name, v)).collect();
        let mut f = self.lock();
        f.tick += 1;
        let tick = f.tick;
        if let Slot::Disk(d) = &f.slots[k] {
            let frame = d.head.with_data(&d.record()?);
            f.slots[k] = Slot::Memory {
                frame: Arc::new(frame),
                touched: tick,
            };
        }
        let Slot::Memory { frame, touched } = &mut f.slots[k] else {
            unreachable!("brought into memory above");
        };
        *touched = tick;
        let frame = Arc::make_mut(frame);
        for p in packed {
            match frame
                .fields
                .binary_search_by(|x| x.name.as_str().cmp(&p.name))
            {
                Ok(i) => frame.fields[i] = p,
                Err(i) => frame.fields.insert(i, p),
            }
        }
        Ok(())
    }

    /// Frames recorded.
    pub fn frame_count(&self) -> usize {
        self.lock().slots.len()
    }

    /// Each frame's time and span, years.
    pub fn frame_times(&self) -> Vec<(f64, f64)> {
        self.lock()
            .slots
            .iter()
            .map(|s| match s {
                Slot::Memory { frame, .. } => (frame.t_years, frame.dt_years),
                Slot::Disk(d) => (d.head.t_years, d.head.dt_years),
            })
            .collect()
    }

    /// Compressed bytes of every frame, wherever it is.
    pub fn bytes(&self) -> u64 {
        self.lock().slots.iter().map(Slot::bytes).sum()
    }

    /// Compressed bytes of the frames held in memory.
    pub fn resident_bytes(&self) -> u64 {
        self.lock().resident_bytes()
    }

    /// Frames whose data is on disk (the session's spill file or a checkpoint).
    pub fn frames_on_disk(&self) -> usize {
        self.lock()
            .slots
            .iter()
            .filter(|s| matches!(s, Slot::Disk(_)))
            .count()
    }

    /// Frame `k` packed, read from disk if it is there.
    pub fn packed(&self, k: usize) -> Result<PackedFrame, CoreError> {
        let slot = self
            .lock()
            .slots
            .get(k)
            .cloned()
            .ok_or_else(|| CoreError::InvalidArgument(format!("no frame {k}")))?;
        match slot {
            Slot::Memory { frame, .. } => Ok((*frame).clone()),
            Slot::Disk(d) => Ok(d.head.with_data(&d.record()?)),
        }
    }

    /// Makes `store` where this history's evicted frames go (the session attaches its own before
    /// each step; a history from a checkpoint keeps reading the frames it has in the checkpoint).
    pub(crate) fn attach(&self, store: &Arc<SpillStore>) {
        let mut f = self.lock();
        if !f.store.as_ref().is_some_and(|s| Arc::ptr_eq(s, store)) {
            f.store = Some(store.clone());
        }
    }

    /// The session's eviction hook on this history.
    pub(crate) fn handle(&self) -> HistoryHandle {
        HistoryHandle(self.frames.clone())
    }

    /// Everything but the frames' data.
    pub fn head(&self) -> HistoryHead {
        HistoryHead {
            grid: self.grid.clone(),
            receivers: self.receivers.clone(),
            frames: self
                .lock()
                .slots
                .iter()
                .map(|s| match s {
                    Slot::Memory { frame, .. } => FrameHead::of(frame),
                    Slot::Disk(d) => d.head.clone(),
                })
                .collect(),
        }
    }

    /// Writes frame `k`'s record (its fields' blobs back to back) into `w`: from memory, or
    /// copied from disk after checking its CRC.
    pub(crate) fn write_record(&self, k: usize, w: &mut dyn Write) -> Result<(), CoreError> {
        let slot = self
            .lock()
            .slots
            .get(k)
            .cloned()
            .ok_or_else(|| CoreError::InvalidArgument(format!("no frame {k}")))?;
        let io = |e: std::io::Error| CoreError::InvalidState(format!("history frame {k}: {e}"));
        match slot {
            Slot::Memory { frame, .. } => {
                for f in &frame.fields {
                    w.write_all(&f.data).map_err(io)?;
                }
            }
            Slot::Disk(d) => w.write_all(&d.record()?).map_err(io)?,
        }
        Ok(())
    }

    /// A history from its head and one record per frame (a checkpoint's).
    pub(crate) fn from_head(
        head: HistoryHead,
        records: Vec<Record>,
    ) -> Result<RecordedHistory, CoreError> {
        if head.frames.len() != records.len() {
            return Err(CoreError::InvalidState(format!(
                "the history has {} frames and {} records",
                head.frames.len(),
                records.len()
            )));
        }
        let mut slots = Vec::with_capacity(records.len());
        let mut tick = 0u64;
        for (fh, rec) in head.frames.into_iter().zip(records) {
            tick += 1;
            slots.push(match rec {
                Record::Bytes(b) => {
                    if b.len() as u64 != fh.record_len() {
                        return Err(CoreError::InvalidState(format!(
                            "history frame at t={}: {} bytes, its head says {}",
                            fh.t_years,
                            b.len(),
                            fh.record_len()
                        )));
                    }
                    Slot::Memory {
                        frame: Arc::new(fh.with_data(&b)),
                        touched: tick,
                    }
                }
                Record::File {
                    file,
                    offset,
                    crc32,
                } => Slot::Disk(Arc::new(DiskFrame {
                    head: fh,
                    file,
                    offset,
                    crc32,
                })),
            });
        }
        Ok(RecordedHistory {
            grid: head.grid,
            receivers: head.receivers,
            frames: Arc::new(Mutex::new(Frames {
                slots,
                tick,
                ..Frames::default()
            })),
        })
    }
}

impl Clone for RecordedHistory {
    /// A snapshot: the same frames (shared, not copied), no spill store of its own.
    fn clone(&self) -> RecordedHistory {
        let f = self.lock();
        RecordedHistory {
            grid: self.grid.clone(),
            receivers: self.receivers.clone(),
            frames: Arc::new(Mutex::new(Frames {
                slots: f.slots.clone(),
                tick: f.tick,
                ..Frames::default()
            })),
        }
    }
}

impl PartialEq for RecordedHistory {
    /// The same grid, drainage and frames, wherever the frames are.
    fn eq(&self, other: &RecordedHistory) -> bool {
        if self.grid != other.grid
            || self.receivers != other.receivers
            || self.frame_count() != other.frame_count()
        {
            return false;
        }
        (0..self.frame_count()).all(|k| match (self.packed(k), other.packed(k)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        })
    }
}

impl std::fmt::Debug for RecordedHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordedHistory")
            .field("grid", &self.grid.name)
            .field("frames", &self.frame_count())
            .field("on_disk", &self.frames_on_disk())
            .field("bytes", &self.bytes())
            .finish()
    }
}

impl FieldHistory for RecordedHistory {
    fn grid(&self) -> &GridDesc {
        &self.grid
    }

    fn frame_count(&self) -> usize {
        RecordedHistory::frame_count(self)
    }

    fn frame(&self, k: usize) -> Result<FieldFrame, CoreError> {
        let cells = self.grid.cells();
        let (slot, ahead) = {
            let mut f = self.lock();
            let slot = f
                .slots
                .get(k)
                .cloned()
                .ok_or_else(|| CoreError::InvalidArgument(format!("no frame {k}")))?;
            let scan = k == 0 || f.last_read.is_some_and(|l| l + 1 == k);
            f.last_read = Some(k);
            // A read-ahead of another record (another frame, or this one since written anew) is
            // left to finish on its own.
            let ahead = f
                .prefetch
                .take()
                .filter(|p| matches!(&slot, Slot::Disk(d) if Arc::ptr_eq(d, &p.frame)));
            if scan && let Some(Slot::Disk(next)) = f.slots.get(k + 1) {
                let next = next.clone();
                let read = next.clone();
                f.prefetch = std::thread::Builder::new()
                    .name("deeptime-history-read".into())
                    .spawn(move || read.file.read(read.offset, read.head.record_len()))
                    .ok()
                    .map(|job| Prefetch { frame: next, job });
            }
            (slot, ahead)
        };
        let (t_years, dt_years, fields, clocks) = match slot {
            Slot::Memory { frame, .. } => {
                let heads: Vec<FieldHead> = FrameHead::of(&frame).fields;
                let parts: Vec<(&FieldHead, &[u8])> = heads
                    .iter()
                    .zip(&frame.fields)
                    .map(|(h, f)| (h, f.data.as_slice()))
                    .collect();
                (
                    frame.t_years,
                    frame.dt_years,
                    unpack_all(&parts, cells)?,
                    frame.clocks.clone(),
                )
            }
            Slot::Disk(d) => {
                let record = match ahead {
                    Some(p) => p.job.join().map_err(|_| {
                        CoreError::InvalidState("the history read-ahead thread panicked".into())
                    })??,
                    None => d.file.read(d.offset, d.head.record_len())?,
                };
                d.check(&record)?;
                let mut at = 0usize;
                let parts: Vec<(&FieldHead, &[u8])> = d
                    .head
                    .fields
                    .iter()
                    .map(|h| {
                        let s = &record[at..at + h.len as usize];
                        at += h.len as usize;
                        (h, s)
                    })
                    .collect();
                (
                    d.head.t_years,
                    d.head.dt_years,
                    unpack_all(&parts, cells)?,
                    d.head.clocks.clone(),
                )
            }
        };
        Ok(FieldFrame {
            t_years,
            dt_years,
            fields,
            clocks,
        })
    }

    fn names(&self) -> Vec<String> {
        let f = self.lock();
        match f.slots.first() {
            None => Vec::new(),
            Some(Slot::Memory { frame, .. }) => frame
                .fields
                .iter()
                .map(|x| x.name.clone())
                .chain(frame.clocks.keys().cloned())
                .collect(),
            Some(Slot::Disk(d)) => d
                .head
                .fields
                .iter()
                .map(|x| x.name.clone())
                .chain(d.head.clocks.keys().cloned())
                .collect(),
        }
    }

    fn receivers(&self) -> Option<&[u32]> {
        self.receivers.as_deref()
    }

    fn origins(&self, k: usize) -> Result<Option<Vec<u32>>, CoreError> {
        let Some(slot) = self.lock().slots.get(k).cloned() else {
            return Err(CoreError::InvalidArgument(format!("no frame {k}")));
        };
        if k == 0 {
            return Ok(None);
        }
        let cells = self.grid.cells();
        let (di, dj) = match slot {
            Slot::Memory { frame, .. } => {
                let find = |name: &str| frame.fields.iter().find(|f| f.name == name);
                let (Some(a), Some(b)) = (find(ORIGIN_FIELDS[0]), find(ORIGIN_FIELDS[1])) else {
                    return Ok(None);
                };
                (
                    unpack(&a.name, a.lo, a.hi, a.bits, &a.data, cells)?,
                    unpack(&b.name, b.lo, b.hi, b.bits, &b.data, cells)?,
                )
            }
            Slot::Disk(d) => {
                let find = |name: &str| d.head.fields.iter().position(|f| f.name == name);
                let (Some(a), Some(b)) = (find(ORIGIN_FIELDS[0]), find(ORIGIN_FIELDS[1])) else {
                    return Ok(None);
                };
                let (ha, hb) = (&d.head.fields[a], &d.head.fields[b]);
                (
                    unpack(&ha.name, ha.lo, ha.hi, ha.bits, &d.field(a)?, cells)?,
                    unpack(&hb.name, hb.lo, hb.hi, hb.bits, &d.field(b)?, cells)?,
                )
            }
        };
        Ok(Some(origins_from_offsets(self.grid.w as usize, &di, &dj)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(n: usize, t: f64, salt: f32) -> FieldFrame {
        let mut frame = FieldFrame {
            t_years: t,
            dt_years: 1.0e7,
            ..FieldFrame::default()
        };
        let elev: Vec<f32> = (0..n)
            .map(|c| (c as f32 * 37.0 + salt) % 9000.0 - 4500.0)
            .collect();
        let mut shelf = vec![0.0f32; n];
        shelf[5] = 1.0;
        shelf[6] = 0.5;
        let mut nan = elev.clone();
        nan[3] = f32::NAN;
        let di: Vec<f32> = (0..n)
            .map(|c| ((c + salt as usize) % 3) as f32 - 1.0)
            .collect();
        frame.fields.insert("deeptime:elevation_m".into(), elev);
        frame.fields.insert("deeptime:shelf".into(), shelf);
        frame.fields.insert("deeptime:heat_flow_mw_m2".into(), nan);
        frame.fields.insert(ORIGIN_FIELDS[0].into(), di.clone());
        frame.fields.insert(ORIGIN_FIELDS[1].into(), vec![0.0; n]);
        frame
            .clocks
            .insert("deeptime:clock/land_plants".into(), 1.0);
        frame
    }

    #[test]
    fn frames_round_trip_within_a_quantum() {
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let n = grid.cells();
        let mut h = RecordedHistory::new(grid);
        let frame = frame(n, -1.0e7, 0.0);
        let elev = frame.fields["deeptime:elevation_m"].clone();
        h.push(&frame);
        let back = h.frame(0).unwrap();
        assert_eq!(back.t_years, -1.0e7);
        let e = &back.fields["deeptime:elevation_m"];
        let step = 9000.0 / 65534.0;
        assert!(e.iter().zip(&elev).all(|(a, b)| (a - b).abs() <= step));
        let s = &back.fields["deeptime:shelf"];
        assert!((s[5] - 1.0).abs() < 1e-6 && (s[6] - 0.5).abs() < 0.5 / 254.0 + 1e-6);
        assert!(back.fields["deeptime:heat_flow_mw_m2"][3].is_nan());
        assert_eq!(h.names().len(), 6);
        // Two dense 16-bit fields of incompressible test data, the offsets and a nearly empty
        // fraction.
        assert!(h.bytes() < (n * 7) as u64, "{} bytes", h.bytes());
        // A later stage adds a field and replaces one; the names stay sorted.
        let mut more = BTreeMap::new();
        more.insert("deeptime:trap_structure".to_owned(), vec![0.25f32; n]);
        more.insert("deeptime:shelf".to_owned(), vec![1.0f32; n]);
        h.add_fields(0, &more).unwrap();
        let back = h.frame(0).unwrap();
        assert!((back.fields["deeptime:trap_structure"][7] - 0.25).abs() < 1e-6);
        assert!(back.fields["deeptime:shelf"].iter().all(|&x| x == 1.0));
        let names: Vec<String> = h
            .packed(0)
            .unwrap()
            .fields
            .into_iter()
            .map(|f| f.name)
            .collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        assert!(h.add_fields(3, &more).is_err());
    }

    /// Bit-for-bit equality of two frames (NaN cells included).
    fn same(a: &FieldFrame, b: &FieldFrame) -> bool {
        let bits = |f: &FieldFrame| -> Vec<(String, Vec<u32>)> {
            f.fields
                .iter()
                .map(|(k, v)| (k.clone(), v.iter().map(|x| x.to_bits()).collect()))
                .collect()
        };
        a.t_years == b.t_years
            && a.dt_years == b.dt_years
            && a.clocks == b.clocks
            && bits(a) == bits(b)
    }

    /// Frames evicted to a spill file read back as the same frames, origins included; a frame on
    /// disk that a later stage adds to comes back into memory; a clone shares the frames; the head
    /// and records rebuild the same history; a damaged record is an error, not a wrong frame.
    #[test]
    fn evicted_frames_read_back_identically() {
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let n = grid.cells();
        let mut h = RecordedHistory::new(grid.clone());
        for k in 0..6 {
            h.push(&frame(n, -6.0e7 + k as f64 * 1.0e7, k as f32));
        }
        h.receivers = Some((0..n as u32).collect());
        let memory = h.clone();
        let want: Vec<FieldFrame> = (0..6).map(|k| memory.frame(k).unwrap()).collect();
        let origins: Vec<_> = (0..6).map(|k| memory.origins(k).unwrap()).collect();
        assert!(origins[2].is_some() && origins[0].is_none());

        // No store: nothing moves.
        let handle = h.handle();
        assert!(handle.is(&h) && !handle.is(&memory));
        assert_eq!(handle.evict(u64::MAX).unwrap(), Evicted::default());
        let dir = std::env::temp_dir().join(format!("dt-recorded-{}", std::process::id()));
        let store = Arc::new(SpillStore::new(&dir));
        h.attach(&store);
        let total = h.bytes();
        let first = h.packed(0).unwrap().data_bytes();
        // The oldest frame goes first, and only as many as asked for.
        let e = handle.evict(1).unwrap();
        assert_eq!((e.frames, e.bytes, e.created_file), (1, first, true));
        assert_eq!(h.frames_on_disk(), 1);
        let e = handle.evict(u64::MAX).unwrap();
        assert_eq!(e.frames, 5);
        assert!(!e.created_file);
        assert_eq!((h.resident_bytes(), h.bytes()), (0, total));
        assert_eq!(store.written_bytes(), total);
        assert_eq!(store.written_frames(), 6);
        // Read in order (the read-ahead path), out of order and backwards.
        for (k, w) in want.iter().enumerate() {
            assert!(same(&h.frame(k).unwrap(), w), "frame {k} in order");
        }
        for k in [3, 1, 5, 0] {
            assert!(same(&h.frame(k).unwrap(), &want[k]), "frame {k}");
        }
        for k in (0..6).rev() {
            assert_eq!(h.origins(k).unwrap(), origins[k], "origins {k}");
        }
        assert_eq!(h.names(), memory.names());
        assert_eq!(h.frame_times(), memory.frame_times());
        assert!(h == memory, "the same history wherever the frames are");

        // A stage adds to a frame on disk: it comes back into memory, newest.
        let mut more = BTreeMap::new();
        more.insert("deeptime:trap_structure".to_owned(), vec![0.5f32; n]);
        h.add_fields(2, &more).unwrap();
        assert_eq!(h.frames_on_disk(), 5);
        let got = h.frame(2).unwrap();
        assert_eq!(got.fields["deeptime:trap_structure"][9], 0.5);
        assert_eq!(
            got.fields["deeptime:elevation_m"],
            want[2].fields["deeptime:elevation_m"]
        );
        // Evicted again, it is written anew (the old record stays as garbage).
        assert_eq!(handle.evict(u64::MAX).unwrap().frames, 1);
        assert!(same(&h.frame(2).unwrap(), &got));

        // Head + records rebuild it, from bytes (in memory) or from the file (on disk).
        let head = h.head();
        let records: Vec<Record> = (0..6)
            .map(|k| {
                let mut b = Vec::new();
                h.write_record(k, &mut b).unwrap();
                Record::Bytes(b)
            })
            .collect();
        let rebuilt = RecordedHistory::from_head(head.clone(), records).unwrap();
        assert!(rebuilt == h);
        assert_eq!(rebuilt.frames_on_disk(), 0);
        let path = dir.join("records.bin");
        let mut all = Vec::new();
        let mut recs = Vec::new();
        for k in 0..6 {
            let mut b = Vec::new();
            h.write_record(k, &mut b).unwrap();
            recs.push((all.len() as u64, crc32fast::hash(&b)));
            all.extend_from_slice(&b);
        }
        std::fs::write(&path, &all).unwrap();
        let file = Arc::new(FrameFile::open(&path).unwrap());
        let on_disk = RecordedHistory::from_head(
            head.clone(),
            recs.iter()
                .map(|&(offset, crc32)| Record::File {
                    file: file.clone(),
                    offset,
                    crc32,
                })
                .collect(),
        )
        .unwrap();
        assert_eq!(on_disk.frames_on_disk(), 6);
        assert!(on_disk == h);
        // A flipped byte: the frame's CRC catches it.
        all[recs[4].0 as usize + 3] ^= 0x40;
        std::fs::write(&path, &all).unwrap();
        let file = Arc::new(FrameFile::open(&path).unwrap());
        let bad = RecordedHistory::from_head(
            head,
            recs.iter()
                .map(|&(offset, crc32)| Record::File {
                    file: file.clone(),
                    offset,
                    crc32,
                })
                .collect(),
        )
        .unwrap();
        assert!(bad.frame(4).is_err());
        assert!(bad.frame(3).is_ok());
        drop((h, memory, rebuilt, on_disk, bad, file));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
