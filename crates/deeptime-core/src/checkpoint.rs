// SPDX-License-Identifier: LGPL-3.0-or-later
//! Stage checkpoints (plan §3.3): the in-memory run state serialised with postcard, so one stage
//! can be iterated (`deeptime stage <name> --in <ckpt> --out <ckpt>`) without rerunning the ones
//! before it. Dev and CLI only; never needed to play (plan §5.7.1 `checkpoints/`).
//!
//! A `.dtcp` file is a sectioned file (`FORMAT.md`, "Checkpoints"): magic `DTCP`, fixed fields
//! `u32 schema, u32 history_frames`, section 0 a JSON summary (readable without decoding the body),
//! section 1 the postcard body (everything but the recorded history's frame data), then one
//! section per history frame: the frame's record, its fields' LZ4 blobs back to back
//! (`deposits::recorded`). Postcard is not self-describing, so a checkpoint is read only by a
//! build with the same [`CHECKPOINT_SCHEMA`] and core version; any other is refused with both
//! named.
//!
//! **Streamed (P1-17).** A long run's history does not fit in memory beside the world (it spills
//! to disk), so a checkpoint is written section by section into `<path>.tmp` and renamed, the
//! body straight from the world and each frame from memory or from the spill file, and never held
//! whole. [`Checkpoint::read`] reads the body and checks every frame section's CRC but leaves the
//! frames in the file: the history it returns reads them from the checkpoint on demand (the file
//! stays open; on Unix a later write over the same path is safe, the rename leaves the open file
//! readable).

use std::fs::File;
use std::io::{Seek, Write};
use std::path::Path;
use std::sync::Arc;

use deeptime_format::FormatError;
use deeptime_format::container::{self, StreamWriter};
use deeptime_format::fsutil::write_atomic_with;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::deposits::recorded::{HistoryHead, Record, RecordedHistory};
use crate::deposits::spill::FrameFile;
use crate::error::CoreError;
use crate::world::WorldState;

/// Magic of a checkpoint file.
pub const MAGIC: [u8; 4] = *b"DTCP";
/// Bump whenever [`Checkpoint`], [`WorldState`] or any stage's saved state changes shape (5: the
/// surface's frame accumulator keeps the crust's origin and the deposit stage's state its frame
/// count, P1-14; 6: the recorded history's frames in sections of their own, streamed, P1-17;
/// 7: `WorldState::globals` and the climate's carbon cycle, fine clock and ice state, deep-time
/// climate; 8: the world's volcano catalogue, P1-21; 9: the world's crater catalogue, P2-08; 10: the
/// climate's ice sheet, weathering supply and snowball list, 2026-09-30).
pub const CHECKPOINT_SCHEMA: u32 = 10;
const FIXED_BYTES: usize = 8;
/// The sections before the history's frames.
const HEAD_SECTIONS: usize = 2;

/// A run paused between (or inside) stages.
#[derive(Clone, Debug, PartialEq)]
pub struct Checkpoint {
    /// The canonical preset JSON the run was created with.
    pub preset_json: String,
    pub content_json: String,
    /// The pipeline, for checking the preset still builds the same stages.
    pub stage_names: Vec<String>,
    /// Stages `[0, next_stage)` are finished; `next_stage` is the one to run next.
    pub next_stage: u32,
    /// The saved state of stage `next_stage` once it has stepped (inside it, or after its last
    /// step before the session moved on); empty at a stage boundary.
    pub stage_state: Vec<u8>,
    pub world: WorldState,
}

/// A checkpoint borrowing its parts (a session writes one without copying its world).
pub struct CheckpointRef<'a> {
    pub preset_json: &'a str,
    pub content_json: &'a str,
    pub stage_names: &'a [String],
    pub next_stage: u32,
    pub stage_state: &'a [u8],
    pub world: &'a WorldState,
}

/// Section 1 as written.
#[derive(Serialize)]
struct BodyRef<'a> {
    preset_json: &'a str,
    content_json: &'a str,
    stage_names: &'a [String],
    next_stage: u32,
    stage_state: &'a [u8],
    world: &'a WorldState,
    history: Option<HistoryHead>,
}

/// Section 1 as read.
#[derive(Deserialize)]
struct Body {
    preset_json: String,
    content_json: String,
    stage_names: Vec<String>,
    next_stage: u32,
    stage_state: Vec<u8>,
    world: WorldState,
    history: Option<HistoryHead>,
}

fn fixed(frames: usize) -> Result<[u8; FIXED_BYTES], CoreError> {
    let frames = u32::try_from(frames).map_err(|_| {
        CoreError::InvalidState(format!(
            "{frames} history frames: too many for a checkpoint"
        ))
    })?;
    let mut f = [0u8; FIXED_BYTES];
    f[..4].copy_from_slice(&CHECKPOINT_SCHEMA.to_le_bytes());
    f[4..].copy_from_slice(&frames.to_le_bytes());
    Ok(f)
}

/// The section count a fixed block announces (a schema-5 file's reserved 0 gives the two it has,
/// so it parses and is then refused by its schema).
fn sections_of(fixed: &[u8]) -> usize {
    HEAD_SECTIONS + u32::from_le_bytes(fixed[4..8].try_into().expect("4 bytes")) as usize
}

/// Refuses a checkpoint of another schema or core version, naming both.
fn check_version(path: &Path, fixed: &[u8], summary: &[u8]) -> Result<(), CoreError> {
    let schema = u32::from_le_bytes(fixed[..4].try_into().expect("4 bytes"));
    let summary: Value = serde_json::from_slice(summary).map_err(|source| FormatError::Json {
        what: format!("{}: checkpoint summary", path.display()),
        source,
    })?;
    let version = summary["core_version"].as_str().unwrap_or("?");
    if schema != CHECKPOINT_SCHEMA || version != crate::VERSION {
        return Err(CoreError::Format(FormatError::Version {
            found: format!("checkpoint schema {schema} (core {version})"),
            supported: format!("schema {CHECKPOINT_SCHEMA} (core {})", crate::VERSION),
        }));
    }
    Ok(())
}

fn decode_body(path: &Path, b: &[u8]) -> Result<Body, CoreError> {
    postcard::from_bytes(b).map_err(|e| {
        CoreError::Format(FormatError::Corrupt {
            path: path.to_owned(),
            reason: format!("checkpoint body does not decode: {e}"),
        })
    })
}

fn corrupt(path: &Path, reason: String) -> CoreError {
    CoreError::Format(FormatError::Corrupt {
        path: path.to_owned(),
        reason,
    })
}

impl CheckpointRef<'_> {
    /// The name of the stage that runs next, if any.
    pub fn next_stage_name(&self) -> Option<&str> {
        self.stage_names
            .get(self.next_stage as usize)
            .map(String::as_str)
    }

    /// The JSON summary stored beside the body.
    pub fn summary(&self) -> Value {
        let h = self.world.history.as_ref();
        json!({
            "core_version": crate::VERSION,
            "schema": CHECKPOINT_SCHEMA,
            "stage_names": self.stage_names,
            "next_stage": self.next_stage,
            "next_stage_name": self.next_stage_name(),
            "mid_stage": !self.stage_state.is_empty(),
            "t_years": self.world.t_years,
            "events": self.world.events.len(),
            "sites": self.world.sites.len(),
            "fields": self.world.fields.keys().collect::<Vec<_>>(),
            "history_frames": h.map_or(0, RecordedHistory::frame_count),
            "history_bytes": h.map_or(0, RecordedHistory::bytes),
        })
    }

    /// Streams the checkpoint into `w` (`path` names it in errors).
    pub fn write_to<W: Write + Seek>(&self, path: &Path, w: W) -> Result<W, CoreError> {
        let history = self.world.history.as_ref();
        let frames = history.map_or(0, RecordedHistory::frame_count);
        let summary = serde_json::to_vec(&self.summary())
            .map_err(|e| CoreError::InvalidState(format!("checkpoint summary: {e}")))?;
        let mut out = StreamWriter::new(path, w, MAGIC, &fixed(frames)?, HEAD_SECTIONS + frames)?;
        out.section(|s| s.write_all(&summary))?;
        let body = BodyRef {
            preset_json: self.preset_json,
            content_json: self.content_json,
            stage_names: self.stage_names,
            next_stage: self.next_stage,
            stage_state: self.stage_state,
            world: self.world,
            history: history.map(RecordedHistory::head),
        };
        out.section(|s| {
            postcard::to_io(&body, &mut *s)
                .map(|_| ())
                .map_err(|e| std::io::Error::other(format!("checkpoint serialisation: {e}")))
        })?;
        if let Some(h) = history {
            for k in 0..frames {
                out.section(|s| {
                    h.write_record(k, s)
                        .map_err(|e| std::io::Error::other(e.to_string()))
                })?;
            }
        }
        Ok(out.finish()?)
    }

    /// Writes the checkpoint to `path` atomically (`<path>.tmp`, fsync, rename), streamed.
    pub fn write(&self, path: &Path) -> Result<(), CoreError> {
        write_atomic_with(path, |f| {
            let w = std::io::BufWriter::with_capacity(1 << 20, &mut *f);
            let w = self.write_to(path, w)?;
            w.into_inner()
                .map(|_| ())
                .map_err(|e| CoreError::io(path, e.into_error()))
        })
    }
}

impl Checkpoint {
    /// The checkpoint's parts, borrowed.
    pub fn as_ref(&self) -> CheckpointRef<'_> {
        CheckpointRef {
            preset_json: &self.preset_json,
            content_json: &self.content_json,
            stage_names: &self.stage_names,
            next_stage: self.next_stage,
            stage_state: &self.stage_state,
            world: &self.world,
        }
    }

    /// The name of the stage that runs next, if any.
    pub fn next_stage_name(&self) -> Option<&str> {
        self.stage_names
            .get(self.next_stage as usize)
            .map(String::as_str)
    }

    /// The JSON summary stored beside the body.
    pub fn summary(&self) -> Value {
        self.as_ref().summary()
    }

    /// The whole file in memory (tests; [`Checkpoint::write`] streams).
    pub fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        let w = self
            .as_ref()
            .write_to(Path::new("checkpoint"), std::io::Cursor::new(Vec::new()))?;
        Ok(w.into_inner())
    }

    /// Decodes a checkpoint held in memory (`path` only names it in errors); the history's frames
    /// are held in memory.
    pub fn from_bytes(path: &Path, b: &[u8]) -> Result<Checkpoint, CoreError> {
        let p = container::parse(path, b, MAGIC, FIXED_BYTES, sections_of)?;
        check_version(path, p.fixed, p.sections[0])?;
        let body = decode_body(path, p.sections[1])?;
        let records = p.sections[HEAD_SECTIONS..]
            .iter()
            .map(|s| Record::Bytes(s.to_vec()))
            .collect();
        Checkpoint::assemble(path, body, records)
    }

    fn assemble(path: &Path, body: Body, records: Vec<Record>) -> Result<Checkpoint, CoreError> {
        let mut world = body.world;
        world.history = match body.history {
            Some(head) => {
                if head.frames.len() != records.len() {
                    return Err(corrupt(
                        path,
                        format!(
                            "the history has {} frames and the file {} frame sections",
                            head.frames.len(),
                            records.len()
                        ),
                    ));
                }
                Some(RecordedHistory::from_head(head, records)?)
            }
            None if records.is_empty() => None,
            None => {
                return Err(corrupt(
                    path,
                    format!("{} frame sections and no history", records.len()),
                ));
            }
        };
        Ok(Checkpoint {
            preset_json: body.preset_json,
            content_json: body.content_json,
            stage_names: body.stage_names,
            next_stage: body.next_stage,
            stage_state: body.stage_state,
            world,
        })
    }

    /// Writes the checkpoint to `path` atomically, streamed.
    pub fn write(&self, path: &Path) -> Result<(), CoreError> {
        self.as_ref().write(path)
    }

    /// Reads a checkpoint: the body into memory, every frame section's CRC checked, the frames
    /// left in the file for the history to read on demand.
    pub fn read(path: &Path) -> Result<Checkpoint, CoreError> {
        let file = File::open(path).map_err(|e| CoreError::io(path, e))?;
        let t = container::read_table(path, &file, MAGIC, FIXED_BYTES, sections_of)?;
        let summary = container::read_section(path, &file, &t.sections[0], "section 0")?;
        check_version(path, &t.fixed, &summary)?;
        let body = decode_body(
            path,
            &container::read_section(path, &file, &t.sections[1], "section 1")?,
        )?;
        let frames = &t.sections[HEAD_SECTIONS..];
        if let Some(head) = &body.history {
            for (k, (s, fh)) in frames.iter().zip(&head.frames).enumerate() {
                if u64::from(s.len) != fh.record_len() {
                    return Err(corrupt(
                        path,
                        format!(
                            "history frame {k}: {} bytes, its head says {}",
                            s.len,
                            fh.record_len()
                        ),
                    ));
                }
            }
        }
        for (k, s) in frames.iter().enumerate() {
            container::check_section(
                path,
                &file,
                s,
                &format!("section {} (history frame {k})", HEAD_SECTIONS + k),
            )?;
        }
        let source = Arc::new(FrameFile::read_only(file, path));
        let records = frames
            .iter()
            .map(|s| Record::File {
                file: source.clone(),
                offset: s.offset,
                crc32: s.crc32,
            })
            .collect();
        Checkpoint::assemble(path, body, records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deposits::fields::FieldFrame;
    use crate::world::Field;
    use deeptime_format::{GridDesc, Place};

    fn checkpoint(frames: usize) -> Checkpoint {
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let mut world = WorldState::new(9, 1024, -2.0e8);
        world
            .fields
            .insert("elevation".into(), Field::filled(grid.clone(), 12.5));
        let e = world
            .event(-2.0e8, "deeptime:rift_onset", "deeptime:dummy")
            .at(Place::Point { lat: 1.0, lon: 2.0 })
            .param("step", 3i64)
            .param("x", 0.5)
            .param("s", "granite");
        world.emit(e).unwrap();
        if frames > 0 {
            let n = grid.cells();
            let mut h = RecordedHistory::new(grid);
            for k in 0..frames {
                let mut f = FieldFrame {
                    t_years: -2.0e8 + k as f64 * 1.0e7,
                    dt_years: 1.0e7,
                    ..FieldFrame::default()
                };
                f.fields.insert(
                    "deeptime:elevation_m".into(),
                    (0..n).map(|c| ((c * 31 + k) % 977) as f32).collect(),
                );
                f.fields
                    .insert("deeptime:shelf".into(), vec![k as f32 / 10.0; n]);
                h.push(&f);
            }
            h.receivers = Some((0..n as u32).rev().collect());
            world.history = Some(h);
        }
        Checkpoint {
            preset_json: r#"{"dummy":{"steps":1,"sleep_ms":0}}"#.into(),
            content_json: String::new(),
            stage_names: vec!["a".into(), "b".into()],
            next_stage: 1,
            stage_state: vec![7],
            world,
        }
    }

    #[test]
    fn round_trips_through_postcard() {
        let ck = checkpoint(0);
        let b = ck.to_bytes().unwrap();
        let back = Checkpoint::from_bytes(Path::new("c.dtcp"), &b).unwrap();
        assert_eq!(back, ck);
        assert_eq!(back.next_stage_name(), Some("b"));

        let mut other = b.clone();
        other[16] = 99; // another schema (the header CRC then fails first, which is fine too)
        assert!(Checkpoint::from_bytes(Path::new("c"), &other).is_err());
    }

    /// A history's frames travel in sections of their own: in memory or on disk, from bytes or
    /// from a file, the same checkpoint; a damaged frame section is refused when the file is read;
    /// a checkpoint of the previous schema is refused naming both.
    #[test]
    fn history_frames_stream_into_their_own_sections() {
        let ck = checkpoint(5);
        let b = ck.to_bytes().unwrap();
        let back = Checkpoint::from_bytes(Path::new("c.dtcp"), &b).unwrap();
        assert_eq!(back, ck);
        assert_eq!(back.summary()["history_frames"], 5);

        let dir = std::env::temp_dir().join(format!("dt-checkpoint-{}", std::process::id()));
        let path = dir.join("h.dtcp");
        ck.write(&path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b, "written = to_bytes");
        let read = Checkpoint::read(&path).unwrap();
        let h = read.world.history.as_ref().unwrap();
        assert_eq!(h.frames_on_disk(), 5, "frames stay in the file");
        assert_eq!(read, ck);

        // Frames spilled to a session's spill file stream into a checkpoint the same.
        let spilled = ck.clone();
        let store = Arc::new(crate::deposits::spill::SpillStore::new(&dir));
        let sh = spilled.world.history.as_ref().unwrap();
        sh.attach(&store);
        assert_eq!(sh.handle().evict(u64::MAX).unwrap().frames, 5);
        assert_eq!(spilled.to_bytes().unwrap(), b);
        // A checkpoint written from a checkpoint's own frames (read from the file).
        let again = dir.join("again.dtcp");
        read.write(&again).unwrap();
        assert_eq!(std::fs::read(&again).unwrap(), b);

        // A flipped byte in the last frame section: refused on reading, naming the frame.
        let mut bad = b.clone();
        let n = bad.len();
        bad[n - 20] ^= 1;
        std::fs::write(&path, &bad).unwrap();
        let err = Checkpoint::read(&path).unwrap_err().to_string();
        assert!(err.contains("history frame 4"), "{err}");
        assert!(Checkpoint::from_bytes(&path, &bad).is_err());

        // Schema 5 (the history inside the body): refused, both schemas named.
        let mut summary = ck.summary();
        summary["schema"] = json!(5);
        let old = container::encode(
            MAGIC,
            &[5, 0, 0, 0, 0, 0, 0, 0],
            &[&serde_json::to_vec(&summary).unwrap(), b"body"],
        )
        .unwrap();
        std::fs::write(&path, &old).unwrap();
        for err in [
            Checkpoint::read(&path).unwrap_err(),
            Checkpoint::from_bytes(&path, &old).unwrap_err(),
        ] {
            let text = err.to_string();
            assert!(
                text.contains("checkpoint schema 5")
                    && text.contains(&format!("schema {CHECKPOINT_SCHEMA}")),
                "{text}"
            );
        }
        drop((read, spilled));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
