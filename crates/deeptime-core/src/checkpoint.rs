// SPDX-License-Identifier: LGPL-3.0-or-later
//! Stage checkpoints (plan §3.3): the in-memory run state serialised with postcard, so one stage
//! can be iterated (`deeptime stage <name> --in <ckpt> --out <ckpt>`) without rerunning the ones
//! before it. Dev and CLI only; never needed to play (plan §5.7.1 `checkpoints/`).
//!
//! A `.dtcp` file is a sectioned file (`FORMAT.md`, "Checkpoints"): magic `DTCP`, fixed fields
//! `u32 schema, u32 reserved`, section 0 a JSON summary (readable without decoding the body),
//! section 1 the postcard body. Postcard is not self-describing, so a checkpoint is read only by a
//! build with the same [`CHECKPOINT_SCHEMA`] and core version.

use std::path::Path;

use deeptime_format::FormatError;
use deeptime_format::container;
use deeptime_format::fsutil::write_atomic;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::CoreError;
use crate::world::WorldState;

/// Magic of a checkpoint file.
pub const MAGIC: [u8; 4] = *b"DTCP";
/// Bump whenever [`Checkpoint`], [`WorldState`] or any stage's saved state changes shape.
pub const CHECKPOINT_SCHEMA: u32 = 4;
const FIXED_BYTES: usize = 8;

/// A run paused between (or inside) stages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl Checkpoint {
    /// The name of the stage that runs next, if any.
    pub fn next_stage_name(&self) -> Option<&str> {
        self.stage_names
            .get(self.next_stage as usize)
            .map(String::as_str)
    }

    /// The JSON summary stored beside the body.
    pub fn summary(&self) -> Value {
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
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CoreError> {
        let body = postcard::to_stdvec(self)
            .map_err(|e| CoreError::InvalidState(format!("checkpoint serialisation: {e}")))?;
        let summary = serde_json::to_vec(&self.summary())
            .map_err(|e| CoreError::InvalidState(format!("checkpoint summary: {e}")))?;
        let mut fixed = Vec::with_capacity(FIXED_BYTES);
        fixed.extend_from_slice(&CHECKPOINT_SCHEMA.to_le_bytes());
        fixed.extend_from_slice(&0u32.to_le_bytes());
        Ok(container::encode(MAGIC, &fixed, &[&summary, &body])?)
    }

    /// Decodes a checkpoint (`path` only names it in errors).
    pub fn from_bytes(path: &Path, b: &[u8]) -> Result<Checkpoint, CoreError> {
        let p = container::parse(path, b, MAGIC, FIXED_BYTES, |_| 2)?;
        let schema = u32::from_le_bytes(p.fixed[..4].try_into().expect("4 bytes"));
        let summary: Value =
            serde_json::from_slice(p.sections[0]).map_err(|source| FormatError::Json {
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
        postcard::from_bytes(p.sections[1]).map_err(|e| {
            CoreError::Format(FormatError::Corrupt {
                path: path.to_owned(),
                reason: format!("checkpoint body does not decode: {e}"),
            })
        })
    }

    /// Writes the checkpoint to `path` atomically.
    pub fn write(&self, path: &Path) -> Result<(), CoreError> {
        write_atomic(path, &self.to_bytes()?)?;
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Checkpoint, CoreError> {
        let b = std::fs::read(path).map_err(|e| CoreError::io(path, e))?;
        Checkpoint::from_bytes(path, &b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Field;
    use deeptime_format::{GridDesc, Place};

    #[test]
    fn round_trips_through_postcard() {
        let mut world = WorldState::new(9, 1024, -2.0e8);
        world.fields.insert(
            "elevation".into(),
            Field::filled(GridDesc::for_planet("L1", 1024, 16).unwrap(), 12.5),
        );
        let e = world
            .event(-2.0e8, "deeptime:rift_onset", "deeptime:dummy")
            .at(Place::Point { lat: 1.0, lon: 2.0 })
            .param("step", 3i64)
            .param("x", 0.5)
            .param("s", "granite");
        world.emit(e).unwrap();
        let ck = Checkpoint {
            preset_json: r#"{"dummy":{"steps":1,"sleep_ms":0}}"#.into(),
            content_json: String::new(),
            stage_names: vec!["a".into(), "b".into()],
            next_stage: 1,
            stage_state: vec![7],
            world,
        };
        let b = ck.to_bytes().unwrap();
        let back = Checkpoint::from_bytes(Path::new("c.dtcp"), &b).unwrap();
        assert_eq!(back, ck);
        assert_eq!(back.next_stage_name(), Some("b"));

        let mut other = b.clone();
        other[16] = 99; // another schema (the header CRC then fails first, which is fine too)
        assert!(Checkpoint::from_bytes(Path::new("c"), &other).is_err());
    }
}
