// SPDX-License-Identifier: LGPL-3.0-or-later
//! Tile streaming for L3 (plan §5.7.5, P1-06): the last level of the amplifier runs one tile at a
//! time, each in a window of the tile plus a halo, and the finished tiles go to a sink — the
//! tile packs `rasters/elevation.dtr` and `rasters/detail/amplitude.dtr` when the run spills, or
//! plain arrays in the tests. A whole-raster run is the same code with one tile the size of the
//! planet, which is what "streamed and in-memory agree" checks: every pass is a pure function of
//! a bounded neighbourhood (the halo covers their radii, `amplify::halo`), and every value a tile
//! starts from is a function of the level below and the cell's own global coordinates.

use std::time::Instant;

use deeptime_format::{Codec, DType, GridDesc, LayerDesc, TilePackWriter, Written};

use super::kernels::{Buf, Counts, NoiseCfg, Physics, Times, count_sinks, relief5, run_passes};
use super::resample::Level;
use super::window::Win;
use crate::error::CoreError;
use crate::surface::detail::DetailParams;

/// A rectangle of the fine grid: first column, first row, width, height.
pub type Rect = (usize, usize, usize, usize);

/// How the last level is cut up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TilePlan {
    pub gw: usize,
    pub gh: usize,
    /// Tile edge, cells.
    pub tile: usize,
    /// Halo, cells.
    pub halo: usize,
}

impl TilePlan {
    /// One tile: the whole raster.
    pub fn whole(gw: usize, gh: usize) -> TilePlan {
        TilePlan {
            gw,
            gh,
            tile: gw.max(gh),
            halo: 0,
        }
    }

    /// The tiles' cores in row-major order.
    pub fn cores(&self) -> Vec<Rect> {
        let mut out = Vec::new();
        for y0 in (0..self.gh).step_by(self.tile) {
            for x0 in (0..self.gw).step_by(self.tile) {
                out.push((
                    x0,
                    y0,
                    self.tile.min(self.gw - x0),
                    self.tile.min(self.gh - y0),
                ));
            }
        }
        out
    }

    /// The window a tile is processed in, and the core's offset inside it. A window that would be
    /// as wide as the planet holds every column once, starting at column 0 (it wraps inside).
    pub fn window(&self, core: Rect) -> (Win, usize, usize) {
        let (x0, y0, cw, ch) = core;
        let j0 = y0.saturating_sub(self.halo);
        let j1 = (y0 + ch + self.halo).min(self.gh);
        let (gi0, w, cx) = if cw + 2 * self.halo >= self.gw {
            (0i64, self.gw, x0)
        } else {
            (x0 as i64 - self.halo as i64, cw + 2 * self.halo, self.halo)
        };
        (
            Win {
                w,
                h: j1 - j0,
                gi0,
                gj0: j0 as i64,
                gw: self.gw,
                gh: self.gh,
            },
            cx,
            y0 - j0,
        )
    }
}

/// A finished tile: its core's terrain (m) and detail amplitude (blocks), row-major.
pub struct TileOut<'a> {
    pub rect: Rect,
    pub z_m: &'a [f32],
    pub amp_blocks: &'a [f32],
}

/// Where finished tiles go.
pub trait TileSink {
    fn put(&mut self, t: &TileOut<'_>) -> Result<(), CoreError>;
}

/// The whole raster in memory (the tests, and the reference a streamed run is compared with).
pub struct MemSink {
    pub gw: usize,
    pub gh: usize,
    pub z: Vec<f32>,
    pub amp: Vec<f32>,
}

impl MemSink {
    pub fn new(gw: usize, gh: usize) -> MemSink {
        MemSink {
            gw,
            gh,
            z: vec![f32::NAN; gw * gh],
            amp: vec![f32::NAN; gw * gh],
        }
    }
}

impl TileSink for MemSink {
    fn put(&mut self, t: &TileOut<'_>) -> Result<(), CoreError> {
        let (x0, y0, w, h) = t.rect;
        for j in 0..h {
            let dst = (y0 + j) * self.gw + x0;
            self.z[dst..dst + w].copy_from_slice(&t.z_m[j * w..(j + 1) * w]);
            self.amp[dst..dst + w].copy_from_slice(&t.amp_blocks[j * w..(j + 1) * w]);
        }
        Ok(())
    }
}

/// The two L3 layers as tile packs, one tile at a time.
pub struct PackSink {
    elevation: TilePackWriter,
    amplitude: TilePackWriter,
    /// Sea level in metres and in blocks, and metres per block: y = sea_y + (z − sea)/mv.
    sea: f64,
    sea_y: f64,
    mv: f64,
    tile: usize,
    tiles_x: usize,
    gw: usize,
    gh: usize,
}

/// What the tile packs hold when they are finished.
pub struct PackFiles {
    pub elevation: (LayerDesc, String, Written),
    pub amplitude: (LayerDesc, String, Written),
}

impl PackSink {
    /// Starts `rasters/elevation.dtr` and `rasters/detail/amplitude.dtr` under `dir` for the L3
    /// grid `grid`, with tiles of `storage_tile` cells (256 in the format).
    pub fn create(
        dir: &std::path::Path,
        grid: &GridDesc,
        storage_tile: u32,
        sea: f64,
        sea_y: f64,
        mv: f64,
    ) -> Result<PackSink, CoreError> {
        let mut e = LayerDesc::scalar("elevation", DType::I16, grid.clone(), Codec::Lz4Block);
        e.units = "block".into();
        e.scale = 1.0 / 16.0;
        e.tile = storage_tile;
        let mut a = LayerDesc::scalar("detail/amplitude", DType::U8, grid.clone(), Codec::Lz4Block);
        a.units = "block".into();
        a.scale = crate::surface::detail::AMPLITUDE_STEP_BLOCKS;
        a.tile = storage_tile;
        Ok(PackSink {
            elevation: TilePackWriter::create(&dir.join(ELEVATION_PATH), &e)?,
            amplitude: TilePackWriter::create(&dir.join(AMPLITUDE_PATH), &a)?,
            sea,
            sea_y,
            mv,
            tile: storage_tile as usize,
            tiles_x: (grid.w as usize).div_ceil(storage_tile as usize),
            gw: grid.w as usize,
            gh: grid.h as usize,
        })
    }

    /// The stored elevation of terrain `z_m`: round(16 · y), saturated (`synth::quantise`).
    pub fn elevation_raw(&self, z_m: f32) -> i16 {
        crate::synth::quantise(16.0 * (self.sea_y + (f64::from(z_m) - self.sea) / self.mv))
    }

    /// Finishes both packs.
    pub fn finish(self) -> Result<PackFiles, CoreError> {
        let ed = self.elevation.desc().clone();
        let ad = self.amplitude.desc().clone();
        let e = self.elevation.finish()?;
        let a = self.amplitude.finish()?;
        Ok(PackFiles {
            elevation: (ed, ELEVATION_PATH.into(), e),
            amplitude: (ad, AMPLITUDE_PATH.into(), a),
        })
    }
}

/// Where the stage puts its layers, relative to the world-data folder.
pub const ELEVATION_PATH: &str = "rasters/elevation.dtr";
pub const AMPLITUDE_PATH: &str = "rasters/detail/amplitude.dtr";

impl TileSink for PackSink {
    fn put(&mut self, t: &TileOut<'_>) -> Result<(), CoreError> {
        let (x0, y0, w, h) = t.rect;
        let ts = self.tile;
        if !x0.is_multiple_of(ts) || !y0.is_multiple_of(ts) {
            return Err(CoreError::InvalidState(format!(
                "a streamed tile at ({x0}, {y0}) does not start on the {ts}-cell storage tiles"
            )));
        }
        let mut ebytes = Vec::with_capacity(ts * ts * 2);
        let mut abytes = Vec::with_capacity(ts * ts);
        for ty in (y0..y0 + h).step_by(ts) {
            for tx in (x0..x0 + w).step_by(ts) {
                let tw = ts.min(self.gw - tx);
                let th = ts.min(self.gh - ty);
                ebytes.clear();
                abytes.clear();
                for j in ty..ty + th {
                    let row = (j - y0) * w + (tx - x0);
                    for i in 0..tw {
                        ebytes.extend_from_slice(&self.elevation_raw(t.z_m[row + i]).to_le_bytes());
                        abytes.push(DetailParams::amplitude_raw(f64::from(
                            t.amp_blocks[row + i],
                        )));
                    }
                }
                let idx = (ty / ts) * self.tiles_x + tx / ts;
                self.elevation.put_tile(idx, &ebytes)?;
                self.amplitude.put_tile(idx, &abytes)?;
            }
        }
        Ok(())
    }
}

/// A thinned copy of the last level (every `stride`-th cell each way) for keyframes and the viewer.
#[derive(Clone, Debug, Default)]
pub struct Preview {
    pub stride: usize,
    pub w: usize,
    pub h: usize,
    pub z_m: Vec<f32>,
    pub amp_blocks: Vec<f32>,
    /// log₁₀ of the effective drainage area in km².
    pub drainage: Vec<f32>,
}

/// Everything the last level measured.
#[derive(Clone, Debug, Default)]
pub struct LastStats {
    pub tiles: usize,
    pub window_cells_max: usize,
    /// Land cells outside the lakes with no strictly lower neighbour, over the tiles' cores.
    pub sinks: u64,
    pub flood_raised_cells: u64,
    pub flood_raised_m_cells: f64,
    pub flood_max_raise_m: f32,
    /// Where it was: global cell (column, row).
    pub flood_max_raise_at: (usize, usize),
    pub flood_unreached: u64,
    /// The flood after the seeding: cells raised and the most.
    pub pre_flood_raised_cells: u64,
    pub pre_flood_max_raise_m: f32,
    /// Σ eroded thickness and Σ sediment still in transit, metres · cells over the cores.
    pub eroded_m_cells: f64,
    pub transit_m_cells: f64,
    /// Σ |z after − z before| over the land cells of the cores (m · cells) and their number.
    pub change_m_cells: f64,
    pub land_cells: u64,
    pub cells: u64,
    pub times: Times,
    pub sample_ms: f64,
    pub detail_ms: f64,
    pub sink_ms: f64,
}

/// What [`run_last`] needs besides the level.
pub struct LastCfg<'a> {
    pub ph: Physics,
    pub counts: Counts,
    pub noise: NoiseCfg,
    pub detail: &'a DetailParams,
    pub drama: f64,
    /// Metres per block, vertically.
    pub mv: f64,
    pub plan: TilePlan,
    pub preview_stride: usize,
}

/// Runs the last ×2 level tile by tile from `level` into `sink`.
pub fn run_last(
    level: &Level,
    cfg: &LastCfg<'_>,
    sink: &mut dyn TileSink,
    cancelled: &dyn Fn() -> bool,
    reserve: &dyn Fn(u64) -> Result<(), CoreError>,
) -> Result<(Preview, LastStats), CoreError> {
    let plan = cfg.plan;
    let (gw, gh) = (plan.gw, plan.gh);
    let s = cfg.preview_stride.max(1);
    let mut preview = Preview {
        stride: s,
        w: gw.div_ceil(s),
        h: gh.div_ceil(s),
        ..Preview::default()
    };
    preview.z_m = vec![f32::NAN; preview.w * preview.h];
    preview.amp_blocks = vec![f32::NAN; preview.w * preview.h];
    preview.drainage = vec![f32::NAN; preview.w * preview.h];
    let mut stats = LastStats::default();
    let sea = cfg.ph.sea;
    for core in plan.cores() {
        if cancelled() {
            return Err(CoreError::Cancelled);
        }
        let (win, cx, cy) = plan.window(core);
        reserve(win.cells() as u64 * Buf::BYTES_PER_CELL)?;
        stats.tiles += 1;
        stats.window_cells_max = stats.window_cells_max.max(win.cells());
        let started = Instant::now();
        let fine = level.sample(&win);
        stats.sample_ms += started.elapsed().as_secs_f64() * 1000.0;
        let mut buf = Buf::new(win, fine);
        buf.core = (cx, cy, core.2, core.3);
        let before = buf.z.clone();
        let pass = run_passes(&mut buf, &cfg.ph, &cfg.counts, &cfg.noise, &mut stats.times);
        let (_, _, cw, ch) = core;
        let started = Instant::now();
        stats.sinks += count_sinks(&win, &buf.z, &buf.lake, sea, (cx, cy, cw, ch));
        stats.pre_flood_raised_cells += pass.pre_flood.raised_cells;
        stats.pre_flood_max_raise_m = stats.pre_flood_max_raise_m.max(pass.pre_flood.max_raise_m);
        stats.flood_raised_cells += pass.flood.raised_cells;
        stats.flood_raised_m_cells += pass.flood.raised_m_cells;
        if pass.flood.max_raise_m > stats.flood_max_raise_m {
            stats.flood_max_raise_m = pass.flood.max_raise_m;
            stats.flood_max_raise_at = win.global(
                pass.flood.max_raise_at % win.w,
                pass.flood.max_raise_at / win.w,
            );
        }
        stats.flood_unreached += pass.flood.unreached;
        // Local relief, then the detail amplitude of every core cell.
        relief5(&win, &buf.z, &mut buf.z2);
        let mut z_core = Vec::with_capacity(cw * ch);
        let mut amp_core = Vec::with_capacity(cw * ch);
        for j in 0..ch {
            let row = (cy + j) * win.w + cx;
            let mut eroded = 0.0f64;
            let mut transit = 0.0f64;
            let (mut change, mut land) = (0.0f64, 0u64);
            for i in 0..cw {
                let c = row + i;
                z_core.push(buf.z[c]);
                if before[c] > sea {
                    change += f64::from((buf.z[c] - before[c]).abs());
                    land += 1;
                }
                let relief_blocks = f64::from(buf.z2[c]) / cfg.mv;
                let a = cfg
                    .detail
                    .amplitude_blocks(relief_blocks, f64::from(buf.a[c]), cfg.drama);
                amp_core.push(a as f32);
                eroded += f64::from(buf.eroded[c]);
                transit += f64::from(buf.sed[c]);
            }
            stats.eroded_m_cells += eroded;
            stats.transit_m_cells += transit;
            stats.change_m_cells += change;
            stats.land_cells += land;
        }
        stats.cells += (cw * ch) as u64;
        // The preview: the cells of the thinned grid this core holds.
        let (x0, y0, _, _) = core;
        for pj in y0.div_ceil(s)..(y0 + ch).div_ceil(s) {
            for pi in x0.div_ceil(s)..(x0 + cw).div_ceil(s) {
                let (gi, gj) = (pi * s, pj * s);
                let c = (cy + gj - y0) * win.w + cx + gi - x0;
                let p = pj * preview.w + pi;
                preview.z_m[p] = buf.z[c];
                preview.amp_blocks[p] = amp_core[(gj - y0) * cw + gi - x0];
                preview.drainage[p] = (f64::from(buf.a[c]).max(1.0) / 1.0e6).log10() as f32;
            }
        }
        stats.detail_ms += started.elapsed().as_secs_f64() * 1000.0;
        let started = Instant::now();
        sink.put(&TileOut {
            rect: core,
            z_m: &z_core,
            amp_blocks: &amp_core,
        })?;
        stats.sink_ms += started.elapsed().as_secs_f64() * 1000.0;
    }
    Ok((preview, stats))
}

/// The halo a plan needs for `counts` (`Counts::radius`), the flood's reach twice (it runs after
/// the seeding and again at the end) and the detail layer's 5 × 5 window and the sink count's
/// 1-cell look.
pub fn halo(counts: &Counts, repair_margin: usize) -> usize {
    counts.radius() + 2 * repair_margin + 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cores_tile_the_planet_and_windows_hold_them() {
        let plan = TilePlan {
            gw: 100,
            gh: 50,
            tile: 32,
            halo: 10,
        };
        let cores = plan.cores();
        assert_eq!(cores.len(), 4 * 2);
        let area: usize = cores.iter().map(|c| c.2 * c.3).sum();
        assert_eq!(area, 100 * 50);
        assert_eq!(*cores.last().unwrap(), (96, 32, 4, 18));
        for c in cores {
            let (win, cx, cy) = plan.window(c);
            // The core sits where the window says it does.
            let (gi, gj) = win.global(cx, cy);
            assert_eq!((gi, gj), (c.0, c.1));
            assert!(win.w <= 100 && win.h <= 50);
        }
        // A window as wide as the planet holds every column once.
        let wide = TilePlan {
            gw: 40,
            gh: 20,
            tile: 32,
            halo: 10,
        };
        let (win, cx, _) = wide.window((0, 0, 32, 20));
        assert_eq!((win.w, win.gi0, cx), (40, 0, 0));
        assert!(win.wraps());
        let one = TilePlan::whole(40, 20);
        assert_eq!(one.cores(), vec![(0, 0, 40, 20)]);
    }
}
