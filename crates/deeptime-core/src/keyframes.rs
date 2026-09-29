// SPDX-License-Identifier: LGPL-3.0-or-later
//! Timelapse keyframes (plan §6.1): the core renders the fields a stage names into already-coloured
//! RGBA8 images (with the shared `ramps.json`), keeps the newest in a bounded ring for
//! `copyKeyframe`, and writes every frame to `keyframes/kf-<n>.dtk` plus `keyframes/index.json`
//! as it is produced, so the ring can drop old frames without losing them (a dropped frame is read
//! back from disk on demand).
//!
//! Threading: the stepping thread renders and pushes; the render thread (Java) reads counts, info
//! and pixels at any time. The ring's mutex is held only to copy or swap `Arc`s, never while
//! rendering or writing files.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use deeptime_format::Ramps;
use deeptime_format::keyframes::{
    KEYFRAMES_DIR, KeyframeImage, KeyframeIndex, read_keyframe, write_keyframe,
};
use serde_json::json;

use crate::error::CoreError;
use crate::world::WorldState;

/// Frames kept in memory (older ones are re-read from disk).
pub const RING_CAPACITY: usize = 16;
/// The keyframe size of the globe/map view (plan §6.1).
pub const DEFAULT_SIZE: (u32, u32) = (1024, 512);

/// When and how large a session renders keyframes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyframePolicy {
    /// A frame whenever the clock passes the next multiple of this many years (counted from the
    /// run's start), and one at the end of every stage; `None` = no keyframes.
    pub every_years: Option<f64>,
    pub size: (u32, u32),
}

impl KeyframePolicy {
    pub const NONE: KeyframePolicy = KeyframePolicy {
        every_years: None,
        size: DEFAULT_SIZE,
    };
}

/// Renders `layers` of `world.fields` (those present) at `size`. Pixel (px, py) is centred on
/// longitude 360°·px/w (the seam is 0°, plan §5.2) and latitude 90° − 180°·py/h (as a tile
/// pack's cell is a point sample, `FORMAT.md`) and takes the nearest grid cell, coloured with the
/// layer's ramp; NaN is transparent. `None` when none of the layers exists yet.
pub fn render(
    world: &WorldState,
    ramps: &Ramps,
    stage: &str,
    layers: &[&str],
    n: u32,
    size: (u32, u32),
) -> Option<KeyframeImage> {
    let (w, h) = size;
    let mut out = Vec::new();
    for &name in layers {
        let Some(field) = world.fields.get(name) else {
            continue;
        };
        let (_, ramp) = ramps.for_layer(name);
        let (gw, gh) = (field.grid.w as f64, field.grid.h as f64);
        let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
        for py in 0..h {
            let j = (f64::from(py) * gh / f64::from(h)).round() as i64;
            for px in 0..w {
                let i = (f64::from(px) * gw / f64::from(w)).round() as i64;
                rgba.extend_from_slice(&ramp.rgba(f64::from(field.get(i, j))));
            }
        }
        out.push((name.to_owned(), rgba));
    }
    if out.is_empty() {
        return None;
    }
    Some(KeyframeImage {
        n,
        t: world.t_years,
        stage: stage.to_owned(),
        w,
        h,
        layers: out,
    })
}

struct Ring {
    frames: VecDeque<Arc<KeyframeImage>>,
    index: KeyframeIndex,
    /// Stale frames of an earlier run into the same folder have been removed.
    cleaned: bool,
}

/// The session's keyframes: a bounded ring in memory, every frame on disk.
pub struct KeyframeStore {
    dir: PathBuf,
    capacity: usize,
    ring: Mutex<Ring>,
    count: AtomicU32,
}

impl KeyframeStore {
    /// Frames go to `<world_dir>/keyframes/`.
    pub fn new(world_dir: &Path, capacity: usize) -> KeyframeStore {
        KeyframeStore {
            dir: world_dir.to_owned(),
            capacity: capacity.max(1),
            ring: Mutex::new(Ring {
                frames: VecDeque::new(),
                index: KeyframeIndex::default(),
                cleaned: false,
            }),
            count: AtomicU32::new(0),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Ring> {
        self.ring.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Frames produced so far (all of them, in the ring or not).
    pub fn count(&self) -> u32 {
        self.count.load(Ordering::Acquire)
    }

    /// The next frame's number.
    pub fn next_n(&self) -> u32 {
        self.count()
    }

    /// Removes the frames an earlier run left in the folder (once, before the first write).
    fn clean(&self, ring: &mut Ring) -> Result<(), CoreError> {
        if ring.cleaned {
            return Ok(());
        }
        let dir = self.dir.join(KEYFRAMES_DIR);
        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                for e in entries.flatten() {
                    let name = e.file_name();
                    let name = name.to_string_lossy();
                    if (name.starts_with("kf-") && name.ends_with(".dtk")) || name == "index.json" {
                        std::fs::remove_file(e.path())
                            .map_err(|err| CoreError::io(e.path(), err))?;
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(CoreError::io(dir, e)),
        }
        ring.cleaned = true;
        Ok(())
    }

    /// Writes `frame` (whose `n` must be [`KeyframeStore::next_n`]) to disk, updates the index and
    /// puts it in the ring. Called by the stepping thread only.
    pub fn push(&self, frame: KeyframeImage) -> Result<(), CoreError> {
        if frame.n != self.next_n() {
            return Err(CoreError::InvalidArgument(format!(
                "keyframe {} pushed, expected {}",
                frame.n,
                self.next_n()
            )));
        }
        {
            let mut ring = self.lock();
            self.clean(&mut ring)?;
        }
        let entry = write_keyframe(&self.dir, &frame)?;
        let mut ring = self.lock();
        ring.index.frames.push(entry);
        ring.index.write(&self.dir)?;
        ring.frames.push_back(Arc::new(frame));
        while ring.frames.len() > self.capacity {
            ring.frames.pop_front();
        }
        self.count.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    /// Writes `keyframes/index.json` even when no frame was produced (and removes stale frames),
    /// so every finished world-data folder has one. Returns the index.
    pub fn finish(&self) -> Result<KeyframeIndex, CoreError> {
        let mut ring = self.lock();
        self.clean(&mut ring)?;
        ring.index.write(&self.dir)?;
        Ok(ring.index.clone())
    }

    fn check(&self, n: u32) -> Result<(), CoreError> {
        let count = self.count();
        if n >= count {
            return Err(CoreError::InvalidArgument(format!(
                "keyframe {n} does not exist ({count} produced so far)"
            )));
        }
        Ok(())
    }

    /// Frame `n`, from the ring or, if it was dropped, from disk.
    pub fn get(&self, n: u32) -> Result<Arc<KeyframeImage>, CoreError> {
        self.check(n)?;
        let entry = {
            let ring = self.lock();
            if let Some(f) = ring.frames.iter().find(|f| f.n == n) {
                return Ok(Arc::clone(f));
            }
            ring.index.frames[n as usize].clone()
        };
        Ok(Arc::new(read_keyframe(&self.dir, &entry)?))
    }

    /// `keyframeInfo`: `{"n", "t", "stage", "w", "h", "layers"}` (plan §6.1).
    pub fn info_json(&self, n: u32) -> Result<String, CoreError> {
        self.check(n)?;
        let e = self.lock().index.frames[n as usize].clone();
        Ok(json!({
            "n": e.n, "t": e.t, "stage": e.stage, "w": e.w, "h": e.h, "layers": e.layers,
        })
        .to_string())
    }

    /// `copyKeyframe`: layer `layer` of frame `n` into `dst` (at least `w·h·4` bytes).
    pub fn copy(&self, n: u32, layer: u32, dst: &mut [u8]) -> Result<(), CoreError> {
        let f = self.get(n)?;
        let Some((_, rgba)) = f.layers.get(layer as usize) else {
            return Err(CoreError::InvalidArgument(format!(
                "keyframe {n} has {} layers, not {}",
                f.layers.len(),
                layer + 1
            )));
        };
        if dst.len() < rgba.len() {
            return Err(CoreError::InvalidArgument(format!(
                "the buffer holds {} bytes, keyframe {n} needs {} ({}×{}×4)",
                dst.len(),
                rgba.len(),
                f.w,
                f.h
            )));
        }
        dst[..rgba.len()].copy_from_slice(rgba);
        Ok(())
    }

    /// The index as written so far.
    pub fn index(&self) -> KeyframeIndex {
        self.lock().index.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Field;
    use deeptime_format::GridDesc;

    #[test]
    fn ring_drops_old_frames_but_disk_keeps_them() {
        let dir = std::env::temp_dir().join(format!("dt-core-kf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(KEYFRAMES_DIR)).unwrap();
        std::fs::write(dir.join("keyframes/kf-99.dtk"), b"stale").unwrap();

        let ramps = Ramps::builtin();
        let mut world = WorldState::new(1, 1024, -1.0e6);
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let mut field = Field::filled(grid, 63.0);
        field.data[0] = f32::NAN;
        world.fields.insert("elevation".into(), field);
        assert!(render(&world, &ramps, "s", &["nothing"], 0, (8, 4)).is_none());

        let store = KeyframeStore::new(&dir, 2);
        for n in 0..5 {
            world.t_years = -1.0e6 + f64::from(n);
            let f = render(&world, &ramps, "s", &["elevation", "missing"], n, (32, 16)).unwrap();
            assert_eq!(f.layers.len(), 1);
            store.push(f).unwrap();
        }
        assert!(
            !dir.join("keyframes/kf-99.dtk").exists(),
            "stale frames removed"
        );
        assert_eq!(store.count(), 5);
        let info: serde_json::Value = serde_json::from_str(&store.info_json(1).unwrap()).unwrap();
        assert_eq!(info["t"], -1.0e6 + 1.0);
        assert_eq!(info["layers"], json!(["elevation"]));
        // Frame 0 left the ring; it comes back from disk, pixel for pixel.
        let mut buf = vec![0u8; 32 * 16 * 4];
        store.copy(0, 0, &mut buf).unwrap();
        assert_eq!(&buf[..4], &[0, 0, 0, 0], "NaN is transparent");
        assert_eq!(
            &buf[4..8],
            &[0x3f, 0x7f, 0x3a, 255],
            "y 63 is the land colour"
        );
        assert!(store.copy(5, 0, &mut buf).is_err());
        assert!(store.copy(0, 1, &mut buf).is_err());
        assert!(store.copy(0, 0, &mut buf[..10]).is_err());
        assert_eq!(KeyframeIndex::read(&dir).unwrap().frames.len(), 5);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
