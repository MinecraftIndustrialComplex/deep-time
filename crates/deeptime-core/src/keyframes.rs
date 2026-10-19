// SPDX-License-Identifier: LGPL-3.0-or-later
//! Timelapse keyframes (plan §6.1): the core renders the fields a stage names into already-coloured
//! RGBA8 images (with the shared `ramps.json`), keeps the newest in a bounded ring for
//! `copyKeyframe`, and writes every frame to `keyframes/kf-<n>.dtk` plus `keyframes/index.json`
//! as it is produced, so the ring can drop old frames without losing them (a dropped frame is read
//! back from disk on demand). The ring holds each layer LZ4-packed (memacct, 2026-10-04: a frame of
//! the coupled stages is 26 layers of 1024 × 326 RGBA, 35 MiB raw and about 4 MiB packed, so the
//! raw ring of 16 held up to 550 MiB); `copyKeyframe` unpacks the one layer it is asked for.
//!
//! Threading: the stepping thread renders and pushes; the render thread (Java) reads counts, info
//! and pixels at any time. The ring's mutex is held only to copy or swap `Arc`s, never while
//! rendering or writing files. A frame's layers are drawn side by side (each pixel is its own, so
//! the picture is the same bytes as drawn one by one) and packed once, in parallel, for both the
//! ring and the file: [`render`], [`KeyframeStore::push`] and [`KeyframeStore::replace`] run in
//! the session's pool (P1-19).

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use deeptime_format::Ramps;
use deeptime_format::keyframes::{
    KEYFRAMES_DIR, KeyframeEntry, KeyframeImage, KeyframeIndex, PackedKeyframe, check_image,
    read_keyframe, write_packed_keyframe,
};
use rayon::prelude::*;
use serde_json::json;

use crate::error::CoreError;
use crate::world::WorldState;

/// Frames kept in memory (older ones are re-read from disk).
pub const RING_CAPACITY: usize = 16;
/// The keyframe size of the globe/map view (plan §6.1): 1024 wide at the map's aspect (the
/// Lambert projection's π : 1, `Projection::picture_height`).
pub const DEFAULT_SIZE: (u32, u32) = (1024, 326);

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

/// What the derived view layers need of the preset's scale: block heights to metres, and the
/// sphere the model runs on (the satellite view's hill shade), and what the water view's
/// hydrology reads (the horizontal scale, the rivers' feature scale and the `hydro` section).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewScale {
    /// `scale.sea_level_y`.
    pub sea_level_y: f64,
    /// `scale.m_per_block_v`.
    pub m_per_block_v: f64,
    /// `tectonics.model_radius_km`, in metres.
    pub radius_m: f64,
    /// `scale.m_per_block_h`.
    pub m_per_block_h: f64,
    /// `scale.feature_scale.rivers`.
    pub rivers_scale: f64,
    /// The preset's `hydro` section (its defaults without one); `enabled: false` draws no water.
    pub hydro: crate::hydro::HydroParams,
}

impl Default for ViewScale {
    /// The defaults of a planet preset (sea level y 63, 35 m per block up, 6,370 km, 300 m a block
    /// across, the default hydrology).
    fn default() -> ViewScale {
        ViewScale {
            sea_level_y: 63.0,
            m_per_block_v: deeptime_format::DEFAULT_M_PER_BLOCK_V,
            radius_m: 6.37e6,
            m_per_block_h: 300.0,
            rivers_scale: 1.0,
            hydro: crate::hydro::HydroParams::default(),
        }
    }
}

/// Renders `layers` of `world.fields` (those present) at `size`. Pixel (px, py) is centred on
/// longitude 360°·px/w (the seam is 0°) and on sin φ = 1 − 2·py/h (the Lambert map,
/// `deeptime_format::projection`) (as a tile
/// pack's cell is a point sample, `FORMAT.md`) and takes the nearest grid cell, coloured with the
/// layer's ramp; NaN is transparent. The satellite view (`satellite`) is composited instead
/// (`crate::satellite`; `view` turns block heights into metres), with the water of the frame's
/// terrain drawn on it, and the `rivers` layer is drawn from that same water
/// (`crate::water_view`, derived once for both). `None` when none of the layers exists yet.
pub fn render(
    world: &WorldState,
    ramps: &Ramps,
    view: &ViewScale,
    stage: &str,
    layers: &[&str],
    n: u32,
    size: (u32, u32),
) -> Option<KeyframeImage> {
    let (w, h) = size;
    // The water view's hydrology beside the layers, and the layers side by side, each by rows.
    let wants_water = layers.iter().any(|l| WATER_LAYERS.contains(l));
    let (water, drawn) = rayon::join(
        || {
            wants_water
                .then(|| crate::water_view::derive(world, view, size))
                .flatten()
        },
        || {
            layers
                .par_iter()
                .map(|&name| draw_dry(world, ramps, view, name, size))
                .collect::<Vec<_>>()
        },
    );
    let mut out = Vec::new();
    for (&name, dry) in layers.iter().zip(drawn) {
        let rgba = if WATER_LAYERS.contains(&name) {
            with_water(ramps, view, name, dry, water.as_ref())
        } else {
            dry
        };
        if let Some(rgba) = rgba {
            out.push((name.to_owned(), rgba));
        }
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

/// Keyframe layers derived from the world's fields when a frame is rendered, not stored in it.
pub const DERIVED_LAYERS: [&str; 3] = [
    crate::satellite::LAYER,
    crate::provinces::LAYER,
    crate::water_view::LAYER,
];

/// The layers that show the water of the terrain (`crate::water_view`): the satellite view draws
/// it over the land, `rivers` is the water on a quiet map.
const WATER_LAYERS: [&str; 2] = [crate::satellite::LAYER, crate::water_view::LAYER];

/// Layer `name` at `size` as far as it needs no water: the satellite view before its water is
/// drawn on it, nothing for `rivers` (drawn from the water alone), the field's ramp picture for
/// any other layer (`None` when it does not exist yet).
fn draw_dry(
    world: &WorldState,
    ramps: &Ramps,
    view: &ViewScale,
    name: &str,
    size: (u32, u32),
) -> Option<Vec<u8>> {
    if name == crate::satellite::LAYER {
        return crate::satellite::render(world, ramps, view, size);
    }
    if WATER_LAYERS.contains(&name) {
        return None;
    }
    // A view layer derived from the fields at render time (`provinces`) is drawn like any
    // other; it is never stored in the world.
    let derived;
    let field = match world.fields.get(name) {
        Some(f) => f,
        None => {
            derived = derived_field(world, name)?;
            &derived
        }
    };
    let (_, ramp) = ramps.for_layer(name);
    let (w, h) = size;
    let mut rgba = vec![0u8; w as usize * h as usize * 4];
    rgba.par_chunks_mut((w as usize * 4).max(1))
        .enumerate()
        .for_each(|(py, row)| {
            for (px, p) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let (i, j) = field.grid.picture_cell(px as f64, py as f64, w, h);
                p.copy_from_slice(&ramp.rgba(f64::from(field.get(i, j))));
            }
        });
    Some(rgba)
}

/// The water layer `name` finished from its dry picture ([`draw_dry`]) and `water`, as
/// [`water_layer`] draws it.
fn with_water(
    ramps: &Ramps,
    view: &ViewScale,
    name: &str,
    dry: Option<Vec<u8>>,
    water: Option<&crate::water_view::WaterView>,
) -> Option<Vec<u8>> {
    if name == crate::satellite::LAYER {
        let mut rgba = dry?;
        if let Some(water) = water {
            water.overlay(&mut rgba, ramps);
        }
        Some(rgba)
    } else {
        water?.render(ramps, view)
    }
}

/// The RGBA picture of the water layer `name` (`satellite` or `rivers`) at `size`, with `water`
/// (derived from the terrain, or read from the bake; `None`: no water to draw, the satellite view
/// stays dry and the `rivers` layer is not drawn).
fn water_layer(
    world: &WorldState,
    ramps: &Ramps,
    view: &ViewScale,
    name: &str,
    size: (u32, u32),
    water: Option<&crate::water_view::WaterView>,
) -> Option<Vec<u8>> {
    if name == crate::satellite::LAYER {
        let mut rgba = crate::satellite::render(world, ramps, view, size)?;
        if let Some(water) = water {
            water.overlay(&mut rgba, ramps);
        }
        Some(rgba)
    } else {
        water?.render(ramps, view)
    }
}

/// `frame` with its water layers (`satellite`, `rivers`) drawn again from `water`: the run's last
/// frame once the bake has written the rivers and lakes (the other layers are kept). `None` when
/// the frame has no water layer, or none could be drawn.
pub fn redraw_water(
    frame: &KeyframeImage,
    world: &WorldState,
    ramps: &Ramps,
    view: &ViewScale,
    water: &crate::water_view::WaterView,
) -> Option<KeyframeImage> {
    let size = (frame.w, frame.h);
    let mut out = frame.clone();
    let mut changed = false;
    for (name, rgba) in &mut out.layers {
        if WATER_LAYERS.contains(&name.as_str())
            && let Some(new) = water_layer(world, ramps, view, name, size, Some(water))
        {
            *rgba = new;
            changed = true;
        }
    }
    changed.then_some(out)
}

/// The derived layer `name` for `world`'s current state (`None`: not one, or its inputs are missing).
fn derived_field(world: &WorldState, name: &str) -> Option<crate::world::Field> {
    (name == crate::provinces::LAYER)
        .then(|| crate::provinces::derive(world))
        .flatten()
}

/// Adds the derived layers a set of stage layers can feed: the geologic provinces need the plate
/// model's `regime` and `crust_age` frames; the satellite view goes first (the default layer of
/// the workbench, the viewer and the game's timelapse) on the plate model's frames (with or
/// without a climate) and on the young landscape's and its amplification's.
pub fn add_derived(layers: &mut Vec<&'static str>) {
    if layers.contains(&"regime") && layers.contains(&"crust_age") {
        layers.push(crate::provinces::LAYER);
    }
    let terrain = (layers.contains(&"elevation") && layers.contains(&"regime"))
        || layers.contains(&crate::surface::young::field::ELEVATION)
        || layers.contains(&crate::surface::amplify::field::ELEVATION);
    if terrain && !layers.contains(&crate::satellite::LAYER) {
        layers.insert(0, crate::satellite::LAYER);
    }
    // The water of a terrain frame: the rivers and lakes as a layer of their own (the satellite
    // view carries them too).
    if terrain && !layers.contains(&crate::water_view::LAYER) {
        layers.push(crate::water_view::LAYER);
    }
}

/// A frame in the ring: its layers LZ4-packed, the blocks its file holds.
type Packed = PackedKeyframe;

/// `f` packed for the ring and its file, the layers compressed side by side (in the caller's
/// pool).
fn pack(f: &KeyframeImage) -> Result<Packed, CoreError> {
    check_image(f)?;
    let blocks = f
        .layers
        .par_iter()
        .map(|(_, rgba)| lz4_flex::block::compress(rgba))
        .collect();
    Ok(PackedKeyframe::with_blocks(f, blocks))
}

/// Layer `k` of `p` into `dst` (exactly its raw length).
fn unpack_layer(p: &Packed, k: usize, dst: &mut [u8]) -> Result<(), CoreError> {
    let (name, packed) = &p.layers[k];
    lz4_flex::block::decompress_into(packed, dst)
        .map(|_| ())
        .map_err(|e| CoreError::InvalidState(format!("keyframe {} layer {name}: {e}", p.n)))
}

fn unpack(p: &Packed) -> Result<KeyframeImage, CoreError> {
    let len = p.layer_len();
    let mut layers = Vec::with_capacity(p.layers.len());
    for (k, (name, _)) in p.layers.iter().enumerate() {
        let mut rgba = vec![0u8; len];
        unpack_layer(p, k, &mut rgba)?;
        layers.push((name.clone(), rgba));
    }
    Ok(KeyframeImage {
        n: p.n,
        t: p.t,
        stage: p.stage.clone(),
        w: p.w,
        h: p.h,
        layers,
    })
}

struct Ring {
    frames: VecDeque<Arc<Packed>>,
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
        let packed = Arc::new(pack(&frame)?);
        drop(frame);
        let entry = write_packed_keyframe(&self.dir, &packed)?;
        let mut ring = self.lock();
        ring.index.frames.push(entry);
        ring.index.write(&self.dir)?;
        ring.frames.push_back(packed);
        while ring.frames.len() > self.capacity {
            ring.frames.pop_front();
        }
        self.count.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    /// Replaces frame `frame.n` (already pushed) by `frame`, in the ring, in its file and in the
    /// index: the run's last frame, drawn again from the bake's water. The frame's number, time,
    /// stage and size must be the old one's; the caller holds the stepper, so nothing else writes.
    pub fn replace(&self, frame: KeyframeImage) -> Result<(), CoreError> {
        self.check(frame.n)?;
        let packed = Arc::new(pack(&frame)?);
        drop(frame);
        let entry = write_packed_keyframe(&self.dir, &packed)?;
        let mut ring = self.lock();
        let n = packed.n;
        ring.index.frames[n as usize] = entry;
        ring.index.write(&self.dir)?;
        if let Some(slot) = ring.frames.iter_mut().find(|f| f.n == n) {
            *slot = packed;
        }
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
        match self.find(n)? {
            Ok(p) => Ok(Arc::new(unpack(&p)?)),
            Err(entry) => Ok(Arc::new(read_keyframe(&self.dir, &entry)?)),
        }
    }

    /// Frame `n` in the ring, or its index entry when it left the ring.
    fn find(&self, n: u32) -> Result<Result<Arc<Packed>, KeyframeEntry>, CoreError> {
        self.check(n)?;
        let ring = self.lock();
        Ok(match ring.frames.iter().find(|f| f.n == n) {
            Some(f) => Ok(Arc::clone(f)),
            None => Err(ring.index.frames[n as usize].clone()),
        })
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
        // From the ring: only the layer asked for is unpacked.
        if let Ok(p) = self.find(n)? {
            let len = &p.layer_len();
            if p.layers.get(layer as usize).is_none() {
                return Err(CoreError::InvalidArgument(format!(
                    "keyframe {n} has {} layers, not {}",
                    p.layers.len(),
                    layer + 1
                )));
            }
            if dst.len() < *len {
                return Err(CoreError::InvalidArgument(format!(
                    "the buffer holds {} bytes, keyframe {n} needs {len} ({}×{}×4)",
                    dst.len(),
                    p.w,
                    p.h
                )));
            }
            return unpack_layer(&p, layer as usize, &mut dst[..*len]);
        }
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

    /// A run resumed in the folder of the run it continues (docs/design/playability-steering.md
    /// §4): the frames that run drew up to `until_t` (world years) are kept as this store's first
    /// frames, the later ones are deleted (files and index), and the next frame is numbered on
    /// from the kept ones. Without an index (or none kept) the folder is cleaned as for a new run.
    /// Returns the number of frames kept. Call before the first frame is pushed.
    pub fn adopt(&self, until_t: f64) -> Result<u32, CoreError> {
        let mut ring = self.lock();
        let index = match KeyframeIndex::read(&self.dir) {
            Ok(i) => i,
            Err(_) => return Ok(0),
        };
        // The kept frames are a prefix: numbered 0, 1, … and no later than the resume point.
        let kept: Vec<_> = index
            .frames
            .iter()
            .enumerate()
            .take_while(|(k, f)| f.n as usize == *k && f.t <= until_t)
            .map(|(_, f)| f.clone())
            .collect();
        for f in &index.frames[kept.len()..] {
            let path = self.dir.join(&f.path);
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(CoreError::io(path, e)),
            }
        }
        ring.index = KeyframeIndex {
            frames: kept,
            ..index
        };
        ring.index.write(&self.dir)?;
        ring.cleaned = true;
        ring.frames.clear();
        let n = ring.index.frames.len() as u32;
        self.count.store(n, Ordering::Release);
        Ok(n)
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
        // NaN in the cell under the frame's first pixel (the seam and the north pole).
        let (i0, j0) = grid.picture_cell(0.0, 0.0, 32, 16);
        let nan_at = j0 as usize * grid.w as usize + i0 as usize;
        let mut field = Field::filled(grid, 63.0);
        field.data[nan_at] = f32::NAN;
        world.fields.insert("elevation".into(), field);
        let view = ViewScale::default();
        assert!(render(&world, &ramps, &view, "s", &["nothing"], 0, (8, 4)).is_none());

        let store = KeyframeStore::new(&dir, 2);
        for n in 0..5 {
            world.t_years = -1.0e6 + f64::from(n);
            let f = render(
                &world,
                &ramps,
                &view,
                "s",
                &["elevation", "missing"],
                n,
                (32, 16),
            )
            .unwrap();
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
        // Frame 4 is in the ring, packed: it comes back as it was drawn, whole or by layer.
        let drawn = render(
            &world,
            &ramps,
            &view,
            "s",
            &["elevation", "missing"],
            4,
            (32, 16),
        )
        .unwrap();
        let kept = store.get(4).unwrap();
        assert_eq!(kept.layers, drawn.layers);
        assert_eq!((kept.n, kept.t, kept.w, kept.h), (4, drawn.t, 32, 16));
        store.copy(4, 0, &mut buf).unwrap();
        assert_eq!(buf, drawn.layers[0].1);
        assert!(store.copy(4, 1, &mut buf).is_err());
        assert!(store.copy(4, 0, &mut buf[..10]).is_err());
        assert!(store.copy(5, 0, &mut buf).is_err());
        assert!(store.copy(0, 1, &mut buf).is_err());
        assert!(store.copy(0, 0, &mut buf[..10]).is_err());
        assert_eq!(KeyframeIndex::read(&dir).unwrap().frames.len(), 5);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_satellite_view_leads_terrain_frames_only() {
        let mut plates = vec!["elevation", "plates", "crust_age", "regime"];
        add_derived(&mut plates);
        assert_eq!(
            plates,
            [
                "satellite",
                "elevation",
                "plates",
                "crust_age",
                "regime",
                "provinces",
                "rivers"
            ]
        );
        let mut young = vec!["young/elevation", "young/drainage"];
        add_derived(&mut young);
        assert_eq!(young[0], "satellite");
        assert_eq!(young.last(), Some(&"rivers"));
        let mut amplify = vec!["amplify/elevation", "amplify/detail"];
        add_derived(&mut amplify);
        assert_eq!(amplify[0], "satellite");
        assert_eq!(amplify.last(), Some(&"rivers"));
        // A dummy stage's elevation and the deposit stage's layers get none.
        let mut dummy = vec!["elevation"];
        add_derived(&mut dummy);
        assert_eq!(dummy, ["elevation"]);
        let mut deposits = vec!["deposits/sites"];
        add_derived(&mut deposits);
        assert_eq!(deposits, ["deposits/sites"]);

        // A frame draws it first, RGBA like every layer.
        let ramps = Ramps::builtin();
        let mut world = WorldState::new(1, 1024, -1.0e6);
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        world
            .fields
            .insert("elevation".into(), Field::filled(grid.clone(), 40.0));
        world
            .fields
            .insert("regime".into(), Field::filled(grid, 0.0));
        let f = render(
            &world,
            &ramps,
            &ViewScale::default(),
            "s",
            &["satellite", "elevation", "regime"],
            0,
            (64, 32),
        )
        .unwrap();
        assert_eq!(f.layer_names(), ["satellite", "elevation", "regime"]);
        assert_eq!(f.layers[0].1.len(), 64 * 32 * 4);
        assert!(f.layers[0].1.chunks(4).all(|p| p[3] == 255));
    }

    /// A 64 × 24 planet (rows from z = −192, past the Lambert pole at 163): sea in the west, a
    /// plain in the east.
    fn half_land() -> WorldState {
        let mut world = WorldState::new(1, 1024, -1.0e6);
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let data = (0..grid.cells())
            .map(|k| if k % 64 < 32 { 40.0 } else { 70.0 })
            .collect();
        world
            .fields
            .insert("elevation".into(), Field { grid, data });
        world
    }

    #[test]
    fn a_frame_draws_the_water_beside_the_satellite_view_and_draws_it_again_from_a_bake() {
        let ramps = Ramps::builtin();
        let view = ViewScale::default();
        let world = half_land();
        let f = render(
            &world,
            &ramps,
            &view,
            "s",
            &["satellite", "rivers"],
            0,
            (64, 32),
        )
        .unwrap();
        assert_eq!(f.layer_names(), ["satellite", "rivers"]);
        assert!(f.layers.iter().all(|(_, rgba)| rgba.len() == 64 * 32 * 4));
        // The rivers layer: sea in the west, pale land in the east.
        let rivers = &f.layers[1].1;
        assert_eq!(&rivers[(16 * 64 + 5) * 4..][..3], &crate::water_view::SEA);
        assert!(rivers[(16 * 64 + 50) * 4] > 150);
        // Hydrology off: the satellite view is drawn dry and there is no rivers layer.
        let mut off = view;
        off.hydro.enabled = false;
        let dry = render(
            &world,
            &ramps,
            &off,
            "s",
            &["satellite", "rivers"],
            0,
            (64, 32),
        )
        .unwrap();
        assert_eq!(dry.layer_names(), ["satellite"]);
        // Drawn again from other water: the other layers are kept, these two are new.
        let other = {
            let mut v = view;
            v.hydro.lake_min_depth_blocks = 64.0;
            crate::water_view::derive(&world, &v, (64, 32)).unwrap()
        };
        let mut with = f.clone();
        with.layers.push(("elevation".into(), vec![9; 64 * 32 * 4]));
        let again = redraw_water(&with, &world, &ramps, &view, &other).unwrap();
        assert_eq!(again.layer_names(), ["satellite", "rivers", "elevation"]);
        assert_eq!(again.layers[2].1, vec![9; 64 * 32 * 4]);
        // A frame with no water layer is not touched.
        let mut bare = f;
        bare.layers.clear();
        assert!(redraw_water(&bare, &world, &ramps, &view, &other).is_none());
    }

    #[test]
    fn a_frame_can_be_replaced_in_the_ring_the_file_and_the_index() {
        let dir = std::env::temp_dir().join(format!("dt-core-kf-replace-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (ramps, view, world) = (Ramps::builtin(), ViewScale::default(), half_land());
        let store = KeyframeStore::new(&dir, 4);
        for n in 0..2 {
            let f = render(&world, &ramps, &view, "s", &["satellite"], n, (64, 32)).unwrap();
            store.push(f).unwrap();
        }
        let mut again = (*store.get(1).unwrap()).clone();
        again.layers[0].1[0] = 7;
        again.layers[0].1[1] = 8;
        store.replace(again.clone()).unwrap();
        let mut buf = vec![0u8; 64 * 32 * 4];
        store.copy(1, 0, &mut buf).unwrap();
        assert_eq!(&buf[..2], &[7, 8]);
        store.copy(0, 0, &mut buf).unwrap();
        assert_ne!(&buf[..2], &[7, 8], "frame 0 is untouched");
        // The file and the index say the same: the frame reads back, CRC checked.
        let index = KeyframeIndex::read(&dir).unwrap();
        assert_eq!(index.frames.len(), 2);
        let back = read_keyframe(&dir, &index.frames[1]).unwrap();
        assert_eq!(back, again);
        assert_eq!(store.index().frames[1].crc32, index.frames[1].crc32);
        assert!(store.replace(KeyframeImage { n: 5, ..again }).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
