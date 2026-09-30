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

use super::character::{self, CharacterCtx, CharacterStats, Scratch};
use super::kernels::{
    Buf, Counts, NoiseCfg, Physics, Times, count_sinks, flood_repair, relief5, run_passes,
};
use super::resample::Level;
use super::window::Win;
use crate::error::CoreError;
use crate::surface::detail::DetailParams;

/// Bytes per cell of a window while a tile runs: the passes' arrays ([`Buf`]), the copy of the
/// terrain the change is measured against, the core's arrays handed to the sink, and the mountain
/// character's copy of the terrain before it.
pub const TILE_BYTES_PER_CELL: u64 = Buf::BYTES_PER_CELL + 4 + 17 + 4;
/// Bytes per cell of a [`Level`] (terrain, area, weight, erodibility, lake).
pub const LEVEL_BYTES_PER_CELL: u64 = 17;

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

/// A finished tile: its core's terrain (m) and detail amplitude (blocks), row-major, and for a
/// level before the last the arrays of the next level.
pub struct TileOut<'a> {
    pub rect: Rect,
    pub z_m: &'a [f32],
    pub amp_blocks: &'a [f32],
    pub level: Option<LevelTile<'a>>,
}

/// The arrays of a level a tile's core hands on ([`Level`]).
pub struct LevelTile<'a> {
    pub area: &'a [f32],
    pub wgt: &'a [f32],
    pub kfac: &'a [f32],
    pub lake: &'a [u8],
}

/// Collects the cores of a level before the last into the arrays of the next.
pub struct LevelSink {
    pub level: Level,
}

impl LevelSink {
    pub fn new(w: usize, h: usize, dx: f64) -> LevelSink {
        let n = w * h;
        LevelSink {
            level: Level {
                w,
                h,
                dx,
                z: vec![0.0; n],
                area: vec![0.0; n],
                wgt: vec![0.0; n],
                kfac: vec![0.0; n],
                lake: vec![0; n],
            },
        }
    }
}

impl TileSink for LevelSink {
    fn put(&mut self, t: &TileOut<'_>) -> Result<(), CoreError> {
        let Some(l) = &t.level else {
            return Err(CoreError::InvalidState(
                "a level sink was given the last level's tile".into(),
            ));
        };
        let (x0, y0, w, h) = t.rect;
        for j in 0..h {
            let dst = (y0 + j) * self.level.w + x0;
            let src = j * w..(j + 1) * w;
            self.level.z[dst..dst + w].copy_from_slice(&t.z_m[src.clone()]);
            self.level.area[dst..dst + w].copy_from_slice(&l.area[src.clone()]);
            self.level.wgt[dst..dst + w].copy_from_slice(&l.wgt[src.clone()]);
            self.level.kfac[dst..dst + w].copy_from_slice(&l.kfac[src.clone()]);
            self.level.lake[dst..dst + w].copy_from_slice(&l.lake[src]);
        }
        Ok(())
    }
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
    /// Σ detail amplitude over the same land cells, blocks · cells.
    pub detail_blocks_cells: f64,
    pub cells: u64,
    pub times: Times,
    pub sample_ms: f64,
    pub detail_ms: f64,
    /// The mountain character (the last level, ruggedness > 0), and the flood after it.
    pub character: CharacterStats,
    pub character_flood_raised_cells: u64,
    /// Cells the character was softened on around pits it closed.
    pub character_softened_cells: u64,
    pub character_flood_max_raise_m: f32,
    pub character_ms: f64,
    pub sink_ms: f64,
}

/// What [`run_tiles`] needs besides the level.
pub struct TileCfg<'a> {
    pub ph: Physics,
    pub counts: Counts,
    pub noise: NoiseCfg,
    pub detail: &'a DetailParams,
    pub drama: f64,
    /// Metres per block, vertically.
    pub mv: f64,
    pub plan: TilePlan,
    pub preview_stride: usize,
    /// The last level: the detail amplitude and the preview are made. An earlier level hands the
    /// sink the arrays of the next one instead ([`LevelTile`]).
    pub last: bool,
    /// The mountain character the last level gets after its passes (None: ruggedness 0).
    pub character: Option<CharacterCtx<'a>>,
}

/// Runs one ×2 level tile by tile from `level` into `sink`: the last level into the L3 layers,
/// an earlier one into the arrays of the next ([`LevelSink`]).
pub fn run_tiles(
    level: &Level,
    cfg: &TileCfg<'_>,
    sink: &mut dyn TileSink,
    cancelled: &dyn Fn() -> bool,
    reserve: &dyn Fn(u64) -> Result<(), CoreError>,
) -> Result<(Preview, LastStats), CoreError> {
    let plan = cfg.plan;
    let (gw, gh) = (plan.gw, plan.gh);
    let s = cfg.preview_stride.max(1);
    let mut preview = Preview::default();
    if cfg.last {
        preview = Preview {
            stride: s,
            w: gw.div_ceil(s),
            h: gh.div_ceil(s),
            ..Preview::default()
        };
        preview.z_m = vec![f32::NAN; preview.w * preview.h];
        preview.amp_blocks = vec![f32::NAN; preview.w * preview.h];
        preview.drainage = vec![f32::NAN; preview.w * preview.h];
    }
    let mut stats = LastStats::default();
    let sea = cfg.ph.sea;
    let a_fine = (cfg.ph.dx * cfg.ph.dx).max(f32::MIN_POSITIVE);
    for core in plan.cores() {
        if cancelled() {
            return Err(CoreError::Cancelled);
        }
        let (win, cx, cy) = plan.window(core);
        reserve(win.cells() as u64 * TILE_BYTES_PER_CELL)?;
        stats.tiles += 1;
        stats.window_cells_max = stats.window_cells_max.max(win.cells());
        let started = Instant::now();
        let fine = level.sample(&win);
        stats.sample_ms += started.elapsed().as_secs_f64() * 1000.0;
        let mut buf = Buf::new(win, fine);
        buf.core = (cx, cy, core.2, core.3);
        let before = buf.z.clone();
        let pass = run_passes(&mut buf, &cfg.ph, &cfg.counts, &cfg.noise, &mut stats.times);
        if let (true, Some(cc)) = (cfg.last, cfg.character.as_ref()) {
            // Mountain character, then a flood so the surface still drains (the arrays the
            // passes are done with are its scratch).
            let started = Instant::now();
            let z_pre = buf.z.clone();
            let st = character::apply(
                &win,
                &mut buf.z,
                &buf.a,
                &buf.kfac,
                &buf.lake,
                Scratch {
                    base: &mut buf.norm,
                    smooth: &mut buf.a2,
                    relief: &mut buf.cap,
                    blur: &mut buf.tmult,
                    acc: &mut buf.sed2,
                    tmp: &mut buf.own,
                    tmp2: &mut buf.z2,
                },
                cc,
                buf.core,
            );
            // Where the character closed a deep pit (a gully dammed by its neighbour's crest), it
            // is softened back towards the passes' terrain before the flood, so the flood fills
            // puddles, not 20-block basins.
            // (The character's scratch is free again: `a2` holds the flooded surface, `norm`
            // the spread pit depth.)
            buf.a2.copy_from_slice(&buf.z);
            flood_repair(
                &win,
                &mut buf.a2,
                &buf.lake,
                cfg.ph.sea,
                cfg.ph.eps_m,
                &mut buf.z2,
                buf.core,
            );
            let softened = character::soften_pits(
                &win,
                &mut buf.z,
                &z_pre,
                &buf.a2,
                &mut buf.z2,
                &mut buf.own,
                &mut buf.norm,
                cc,
                buf.core,
            );
            drop(z_pre);
            let fl = flood_repair(
                &win,
                &mut buf.z,
                &buf.lake,
                cfg.ph.sea,
                cfg.ph.eps_m,
                &mut buf.z2,
                buf.core,
            );
            stats.character.land_cells += st.land_cells;
            stats.character.abs_change_blocks_cells += st.abs_change_blocks_cells;
            stats.character.max_raise_blocks =
                stats.character.max_raise_blocks.max(st.max_raise_blocks);
            stats.character.max_cut_blocks = stats.character.max_cut_blocks.max(st.max_cut_blocks);
            stats.character_flood_raised_cells += fl.raised_cells;
            stats.character_softened_cells += softened;
            stats.character_flood_max_raise_m =
                stats.character_flood_max_raise_m.max(fl.max_raise_m);
            stats.character_ms += started.elapsed().as_secs_f64() * 1000.0;
        }
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
        if cfg.last {
            // Local relief, then the detail amplitude of every core cell.
            relief5(&win, &buf.z, &mut buf.z2);
        }
        let mut z_core = Vec::with_capacity(cw * ch);
        let mut amp_core = Vec::with_capacity(if cfg.last { cw * ch } else { 0 });
        let (mut area_core, mut wgt_core, mut kfac_core, mut lake_core) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        if !cfg.last {
            area_core.reserve(cw * ch);
            wgt_core.reserve(cw * ch);
            kfac_core.reserve(cw * ch);
            lake_core.reserve(cw * ch);
        }
        for j in 0..ch {
            let row = (cy + j) * win.w + cx;
            let mut eroded = 0.0f64;
            let mut transit = 0.0f64;
            let (mut change, mut land, mut detail) = (0.0f64, 0u64, 0.0f64);
            for i in 0..cw {
                let c = row + i;
                z_core.push(buf.z[c]);
                if before[c] > sea {
                    change += f64::from((buf.z[c] - before[c]).abs());
                    land += 1;
                }
                if cfg.last {
                    let relief_blocks = f64::from(buf.z2[c]) / cfg.mv;
                    let a =
                        cfg.detail
                            .amplitude_blocks(relief_blocks, f64::from(buf.a[c]), cfg.drama);
                    amp_core.push(a as f32);
                    if before[c] > sea {
                        detail += a;
                    }
                } else {
                    area_core.push(buf.a[c]);
                    wgt_core.push(buf.own[c] / a_fine);
                    kfac_core.push(buf.kfac[c]);
                    lake_core.push(buf.lake[c]);
                }
                eroded += f64::from(buf.eroded[c]);
                transit += f64::from(buf.sed[c]);
            }
            stats.eroded_m_cells += eroded;
            stats.transit_m_cells += transit;
            stats.change_m_cells += change;
            stats.land_cells += land;
            stats.detail_blocks_cells += detail;
        }
        stats.cells += (cw * ch) as u64;
        if cfg.last {
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
        }
        stats.detail_ms += started.elapsed().as_secs_f64() * 1000.0;
        let started = Instant::now();
        sink.put(&TileOut {
            rect: core,
            z_m: &z_core,
            amp_blocks: &amp_core,
            level: (!cfg.last).then_some(LevelTile {
                area: &area_core,
                wgt: &wgt_core,
                kfac: &kfac_core,
                lake: &lake_core,
            }),
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
