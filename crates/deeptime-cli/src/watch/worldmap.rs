// SPDX-License-Identifier: LGPL-3.0-or-later
//! The final world as a deep-zoom map: what the workbench's "Final world" view draws from a run's
//! **bake** (`<run>/world`, FORMAT.md "The bake"). View only: it reads the finished world data
//! and writes nothing into it, adds nothing to the simulation or the bake, and renders each tile
//! when a page asks.
//!
//! # The pyramid
//! Equirectangular, 256-pixel tiles, XYZ-style with the geodetic 2:1 layout: zoom `z` is
//! `2^(z+1) × 2^z` tiles, `x` east from the seam, `y` south from the north edge, so a pixel is
//! `s = C / (512·2^z)` blocks wide. Zoom 0 is the whole planet in two tiles; the last native zoom
//! is the one where `s` reaches one block (C = 16,384: zoom 5; C = 32,768: zoom 6). Pixel `(i, j)`
//! of a tile samples block `(x0 + i·s, z0 + j·s)`, the top left of the block it draws at one block
//! a pixel, so block columns line up with pixels there. x wraps. The frame spans z ∈ [−C/4, C/4)
//! (`Projection::tile_frame_z`). On a Lambert world it is wider than the planet: the poles are at
//! z = ∓R = ∓C/2π, and pixels past them are drawn as the page's background. On a Petroff–Guyou
//! world (format 0.5, the default) the frame is exactly the world: x ∈ [−b, b) with C = 2b, z from
//! −b/2 to b/2, every pixel on the planet; rows past the north and south edges fold through their
//! half-turns (`GridDesc::cell`), so the map continues through the seams.
//!
//! # What the pixels are
//! The finest data the bake has is the **terrain height** `H = E + A·N` (FORMAT.md "The terrain
//! height"), evaluated at every block: `E` the L3 `elevation` bilinear (C / 8192 blocks a cell at
//! the 16k quick size: two), `A` the L3 `detail/amplitude` and `N` the game's sub-cell noise
//! (wavelengths 32, 16 and 8 blocks). Tiles with at most about four blocks a pixel use exactly
//! that (a box average of several samples for four); coarser tiles read **mips**, box averages
//! of the L3 cells built once per run when the map is first opened (8 blocks a cell and up,
//! 0.3 s at 16k), so every zoom is alias-free. Colours are the core's satellite look
//! (`deeptime_core::satellite`: `colour`, `cover`, `box_filter`, `Light`, the `satellite` ramp)
//! fed with the bake's fields: the twelve monthly sea-level temperatures lapsed to the block's own
//! height (the climate API's rule, FORMAT.md "The climate at a block", so snow lines and tree lines
//! follow block-scale relief), precipitation and Köppen class, P/PET and the equilibrium line
//! rebuilt from them with the climate stage's own functions, the ice cover, the surface rock's
//! colour and lakes and rivers from `water_level` and `rivers.dtrv`.
//! What the bake does not hold (the sea-ice cover, a hotspot flag) is approximated or left out:
//! sea ice is the share of months the baked sea-level temperature sits at the freezing point.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use deeptime_core::climate::{glaciers, koppen, snapshot};
use deeptime_core::satellite::{self, Cell, Light, Look, Palette};
use deeptime_core::world::Field as CoreField;
use deeptime_format::raster::cover_of;
use deeptime_format::rivers::read_rivers;
use deeptime_format::{
    DetailSpec, GridDesc, LayerDesc, Manifest, Noise, Ramps, Rivers, TilePackReader,
};
use serde_json::{Value, json};

/// Tile edge, pixels.
pub const TILE: usize = 256;
/// Bumped whenever a change to the renderer makes cached tiles stale; the cache folder carries it.
pub const RENDER_VERSION: u32 = 2;
/// The colour of the pixels past the poles (the workbench's map background, #04060a).
pub const BEYOND_POLE: [u8; 4] = [4, 6, 10, 255];
/// A tile is computed with a one pixel border so the hill shade has its neighbours.
const N: usize = TILE + 2;
/// Mips start at this level (cells of `r·2^m` blocks): finer tiles read the L3 cells themselves.
const MIP_MIN: u32 = 2;
/// Decoded tile bytes one layer keeps for window reads.
const STORE_BYTES: usize = 96 << 20;

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// ---------------------------------------------------------------------------------------------
// The pyramid
// ---------------------------------------------------------------------------------------------

/// The layers a tile can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layer {
    /// The satellite look (true colour).
    Satellite,
    /// The elevation ramp under a hill shade.
    Elevation,
    /// Water first: a muted land under the sea, lakes and rivers.
    Water,
    /// Lakes and rivers alone, transparent, to lay over the others.
    Overlay,
}

impl Layer {
    pub const ALL: [Layer; 4] = [
        Layer::Satellite,
        Layer::Elevation,
        Layer::Water,
        Layer::Overlay,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Layer::Satellite => "sat",
            Layer::Elevation => "elev",
            Layer::Water => "water",
            Layer::Overlay => "overlay",
        }
    }

    pub fn parse(s: &str) -> Option<Layer> {
        Layer::ALL.into_iter().find(|l| l.id() == s)
    }

    /// Whether every pixel is opaque (the PNG needs no alpha).
    pub fn opaque(self) -> bool {
        self != Layer::Overlay
    }
}

/// The last native zoom for circumference `c`: the one at which a pixel is a block or less.
pub fn max_zoom(c: u32) -> u32 {
    let ratio = f64::from(c) / (2.0 * TILE as f64);
    if ratio < 1.0 {
        0
    } else {
        ratio.log2().floor() as u32
    }
}

/// Blocks a pixel is wide at zoom `z`.
pub fn blocks_per_px(c: u32, z: u32) -> f64 {
    f64::from(c) / (2.0 * TILE as f64 * (1u64 << z) as f64)
}

/// Tiles in a full pyramid of zooms `0..=max_zoom` (one layer).
pub fn pyramid_tiles(max_zoom: u32) -> u64 {
    (0..=max_zoom).map(|z| 2u64 << z << z).sum()
}

/// Where one tile lies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileGeom {
    pub z: u32,
    pub tx: u32,
    pub ty: u32,
    /// Blocks a pixel is wide.
    pub s: f64,
    /// Block of pixel (0, 0).
    pub x0: f64,
    pub z0: f64,
}

impl TileGeom {
    /// The tile `z/tx/ty` of a planet of circumference `c`; `None` outside the pyramid.
    pub fn new(c: u32, z: u32, tx: u32, ty: u32) -> Option<TileGeom> {
        if z > max_zoom(c) || u64::from(tx) >= 2u64 << z || u64::from(ty) >= 1u64 << z {
            return None;
        }
        let s = blocks_per_px(c, z);
        let c = f64::from(c);
        Some(TileGeom {
            z,
            tx,
            ty,
            s,
            x0: -c / 2.0 + f64::from(tx) * TILE as f64 * s,
            z0: deeptime_format::Projection::of(c).tile_frame_z() + f64::from(ty) * TILE as f64 * s,
        })
    }
}

// ---------------------------------------------------------------------------------------------
// Reading layers
// ---------------------------------------------------------------------------------------------

/// A layer of L3 size read tile by tile on demand (the decoded tiles are kept, bounded).
struct LayerStore {
    reader: TilePackReader,
    cache: Mutex<TileLru>,
}

#[derive(Default)]
struct TileLru {
    map: HashMap<usize, (Arc<Vec<u8>>, u64)>,
    bytes: usize,
    clock: u64,
}

/// A rectangle of one layer's physical values (NaN: nodata), kept as f32 with the grid's frame.
struct Window {
    i0: i64,
    j0: i64,
    w: usize,
    h: usize,
    r: f64,
    ox: f64,
    oz: f64,
    data: Vec<f32>,
}

impl Window {
    #[inline]
    fn at(&self, i: i64, j: i64) -> f32 {
        let a = (i - self.i0).clamp(0, self.w as i64 - 1) as usize;
        let b = (j - self.j0).clamp(0, self.h as i64 - 1) as usize;
        self.data[b * self.w + a]
    }

    #[inline]
    fn bilinear(&self, bx: f64, bz: f64) -> f32 {
        let u = (bx - self.ox) / self.r;
        let v = (bz - self.oz) / self.r;
        let (fi, fj) = (u.floor(), v.floor());
        let (fu, fv) = ((u - fi) as f32, (v - fj) as f32);
        let (i, j) = (fi as i64, fj as i64);
        let (a, b) = (self.at(i, j), self.at(i + 1, j));
        let (c, d) = (self.at(i, j + 1), self.at(i + 1, j + 1));
        let top = a + (b - a) * fu;
        let bottom = c + (d - c) * fu;
        top + (bottom - top) * fv
    }

    #[inline]
    fn nearest(&self, bx: f64, bz: f64) -> f32 {
        let i = ((bx - self.ox) / self.r + 0.5).floor() as i64;
        let j = ((bz - self.oz) / self.r + 0.5).floor() as i64;
        self.at(i, j)
    }
}

impl LayerStore {
    fn open(path: &Path) -> Result<LayerStore, String> {
        let reader = TilePackReader::open(path).map_err(|e| e.to_string())?;
        let d = reader.desc();
        if d.dtype.scalar_bytes().is_none() {
            return Err(format!("{}: not a scalar layer", path.display()));
        }
        Ok(LayerStore {
            reader,
            cache: Mutex::new(TileLru::default()),
        })
    }

    fn desc(&self) -> &LayerDesc {
        self.reader.desc()
    }

    fn tile(&self, t: usize) -> Result<Arc<Vec<u8>>, String> {
        {
            let mut c = lock(&self.cache);
            c.clock += 1;
            let clock = c.clock;
            if let Some(hit) = c.map.get_mut(&t) {
                hit.1 = clock;
                return Ok(Arc::clone(&hit.0));
            }
        }
        // Decoded outside the lock: two threads may decode one tile, which is harmless.
        let cells = Arc::new(self.reader.read_tile(t).map_err(|e| e.to_string())?);
        let mut c = lock(&self.cache);
        while c.bytes + cells.len() > STORE_BYTES && !c.map.is_empty() {
            let oldest = c.map.iter().min_by_key(|(_, v)| v.1).map(|(k, _)| *k);
            if let Some(k) = oldest
                && let Some((gone, _)) = c.map.remove(&k)
            {
                c.bytes -= gone.len();
            }
        }
        c.clock += 1;
        let clock = c.clock;
        c.bytes += cells.len();
        c.map.insert(t, (Arc::clone(&cells), clock));
        Ok(cells)
    }

    /// Cells `[i0, i0 + w) × [j0, j0 + h)` (x wraps, z clamps) as physical f32.
    fn window(&self, i0: i64, j0: i64, w: usize, h: usize) -> Result<Window, String> {
        let d = self.desc();
        let g = &d.grid;
        let (cb, tile) = (d.cell_bytes(), i64::from(d.tile));
        let mut data = vec![0f32; w * h];
        let mut tile_for: Option<(usize, Arc<Vec<u8>>)> = None;
        for jj in 0..h {
            let row = &mut data[jj * w..(jj + 1) * w];
            for (ii, out) in row.iter_mut().enumerate() {
                // Columns wrap; rows clamp (Lambert) or fold through the edges (Petroff–Guyou).
                let (ci, cj) = g.cell(i0 + ii as i64, j0 + jj as i64);
                let (i, j) = (i64::from(ci), i64::from(cj));
                let t = ((j / tile) as usize) * d.tiles_x() as usize + (i / tile) as usize;
                if tile_for.as_ref().is_none_or(|(k, _)| *k != t) {
                    tile_for = Some((t, self.tile(t)?));
                }
                let bytes = &tile_for.as_ref().expect("set above").1;
                let (x0, z0, tw, _) = d.tile_rect(t);
                let at = ((j as u32 - z0) as usize * tw as usize + (i as u32 - x0) as usize) * cb;
                *out = d.physical(d.dtype.decode_scalar(&bytes[at..at + cb])) as f32;
            }
        }
        Ok(Window {
            i0,
            j0,
            w,
            h,
            r: f64::from(g.r),
            ox: g.origin_block_x as f64,
            oz: g.origin_block_z as f64,
            data,
        })
    }

    /// The whole layer as physical f32 (row-major), for the small layers (L1, L2).
    fn load_f32(path: &Path) -> Result<(GridDesc, Vec<f32>), String> {
        let reader = TilePackReader::open(path).map_err(|e| e.to_string())?;
        let d = reader.desc().clone();
        let cb = d.cell_bytes();
        let (w, h) = (d.grid.w as usize, d.grid.h as usize);
        let mut out = vec![0f32; w * h];
        for t in 0..d.tile_count() {
            let cells = reader.read_tile(t).map_err(|e| e.to_string())?;
            let (x0, z0, tw, th) = d.tile_rect(t);
            for b in 0..th as usize {
                for a in 0..tw as usize {
                    let at = (b * tw as usize + a) * cb;
                    out[(z0 as usize + b) * w + x0 as usize + a] =
                        d.physical(d.dtype.decode_scalar(&cells[at..at + cb])) as f32;
                }
            }
        }
        Ok((d.grid, out))
    }
}

/// A grid of f32 values with its frame in blocks: a coarse field the colours are drawn from.
struct Grid {
    w: usize,
    h: usize,
    r: f64,
    ox: f64,
    oz: f64,
    data: Vec<f32>,
}

/// Where a block falls on a grid: the four cells and the weights of the bilinear blend.
#[derive(Clone, Copy)]
struct Stencil {
    k: [usize; 4],
    f: [f32; 4],
}

impl Grid {
    fn new(g: &GridDesc, data: Vec<f32>) -> Grid {
        Grid {
            w: g.w as usize,
            h: g.h as usize,
            r: f64::from(g.r),
            ox: g.origin_block_x as f64,
            oz: g.origin_block_z as f64,
            data,
        }
    }

    fn core(&self, g: &GridDesc) -> CoreField {
        CoreField {
            grid: g.clone(),
            data: self.data.clone(),
        }
    }

    #[inline]
    fn stencil(&self, bx: f64, bz: f64) -> Stencil {
        let u = (bx - self.ox) / self.r;
        let v = (bz - self.oz) / self.r;
        let (fi, fj) = (u.floor(), v.floor());
        let (fu, fv) = ((u - fi) as f32, (v - fj) as f32);
        let (i, j) = (fi as i64, fj as i64);
        let w = self.w as i64;
        let wrap = |i: i64| i.rem_euclid(w) as usize;
        let clamp = |j: i64| j.clamp(0, self.h as i64 - 1) as usize;
        let (i0, i1, j0, j1) = (wrap(i), wrap(i + 1), clamp(j), clamp(j + 1));
        Stencil {
            k: [
                j0 * self.w + i0,
                j0 * self.w + i1,
                j1 * self.w + i0,
                j1 * self.w + i1,
            ],
            f: [
                (1.0 - fu) * (1.0 - fv),
                fu * (1.0 - fv),
                (1.0 - fu) * fv,
                fu * fv,
            ],
        }
    }

    /// Bilinear over the corners that are known (NaN corners drop out); NaN when none is.
    #[inline]
    fn at(&self, s: &Stencil) -> f32 {
        let (mut sum, mut wt) = (0.0f32, 0.0f32);
        for q in 0..4 {
            let v = self.data[s.k[q]];
            if v.is_finite() {
                sum += v * s.f[q];
                wt += s.f[q];
            }
        }
        if wt > 1e-6 { sum / wt } else { f32::NAN }
    }

    /// The nearest cell's value; a tie goes to the later cell, as the game's `RasterRead::nearest`
    /// (floor(u + 0.5)) has it, so the class and the cover (both read nearest) change where the
    /// game's do.
    #[inline]
    fn nearest(&self, s: &Stencil) -> f32 {
        let mut best = 0;
        for q in 1..4 {
            if s.f[q] >= s.f[best] {
                best = q;
            }
        }
        self.data[s.k[best]]
    }

    fn bilinear(&self, bx: f64, bz: f64) -> f32 {
        self.at(&self.stencil(bx, bz))
    }
}

/// Box-averaged copies of an L3 layer: level `m` has cells of `r·2^m` blocks. A cell's value is
/// the mean of the L3 cells it covers (cells with no data do not count), its point at the middle
/// of them.
struct Mips {
    levels: Vec<Grid>,
    /// The level of `levels[0]`.
    first: u32,
}

impl Mips {
    fn level(&self, m: u32) -> Option<&Grid> {
        m.checked_sub(self.first)
            .and_then(|k| self.levels.get(k as usize))
    }
}

/// One channel of a mip pass: (value, weight) of a raw physical cell value.
type Channel = fn(f32) -> (f32, f32);

/// Streams `store`'s tiles once and builds mip levels `first..=last` of each channel (a worker
/// pool of at most four threads): `result[channel]` is that channel's [`Mips`].
fn build_mips(
    store: &LayerStore,
    first: u32,
    last: u32,
    channels: &[Channel],
) -> Result<Vec<Mips>, String> {
    let d = store.desc();
    let g = &d.grid;
    let (w, h) = (g.w as usize, g.h as usize);
    let step = 1usize << first;
    let (w0, h0) = (w.div_ceil(step), h.div_ceil(step));
    let nch = channels.len();
    // Sums and weights at the first level, per channel.
    let mut sums = vec![vec![0f32; w0 * h0]; nch];
    let mut wts = vec![vec![0f32; w0 * h0]; nch];
    let next = AtomicUsize::new(0);
    let failed: Mutex<Option<String>> = Mutex::new(None);
    let merge = Mutex::new((&mut sums, &mut wts));
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get().min(4));
    let cb = d.cell_bytes();
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| {
                loop {
                    let t = next.fetch_add(1, Ordering::Relaxed);
                    if t >= d.tile_count() {
                        break;
                    }
                    let cells = match store.reader.read_tile(t) {
                        Ok(c) => c,
                        Err(e) => {
                            *lock(&failed) = Some(e.to_string());
                            break;
                        }
                    };
                    let (x0, z0, tw, th) = d.tile_rect(t);
                    let (px, pz) = (x0 as usize / step, z0 as usize / step);
                    let (pw, ph) = ((tw as usize).div_ceil(step), (th as usize).div_ceil(step));
                    let mut ps = vec![vec![0f32; pw * ph]; nch];
                    let mut pw_ = vec![vec![0f32; pw * ph]; nch];
                    for b in 0..th as usize {
                        for a in 0..tw as usize {
                            let at = (b * tw as usize + a) * cb;
                            let v = d.physical(d.dtype.decode_scalar(&cells[at..at + cb])) as f32;
                            let q = (b / step) * pw + a / step;
                            for (c, f) in channels.iter().enumerate() {
                                let (value, weight) = f(v);
                                ps[c][q] += value * weight;
                                pw_[c][q] += weight;
                            }
                        }
                    }
                    let mut m = lock(&merge);
                    for c in 0..nch {
                        for b in 0..ph {
                            for a in 0..pw {
                                let at = (pz + b) * w0 + px + a;
                                m.0[c][at] += ps[c][b * pw + a];
                                m.1[c][at] += pw_[c][b * pw + a];
                            }
                        }
                    }
                }
            });
        }
    });
    if let Some(e) = lock(&failed).take() {
        return Err(e);
    }
    let mut out = Vec::new();
    for c in 0..nch {
        let (mut s, mut wt) = (std::mem::take(&mut sums[c]), std::mem::take(&mut wts[c]));
        let (mut cw, mut ch) = (w0, h0);
        let mut levels = Vec::new();
        for m in first..=last {
            let cell = f64::from(g.r) * (1u64 << m) as f64;
            let off = f64::from(g.r) * ((1u64 << m) as f64 - 1.0) / 2.0;
            let data: Vec<f32> = s
                .iter()
                .zip(&wt)
                .map(|(&a, &b)| if b > 0.0 { a / b } else { f32::NAN })
                .collect();
            levels.push(Grid {
                w: cw,
                h: ch,
                r: cell,
                ox: g.origin_block_x as f64 + off,
                oz: g.origin_block_z as f64 + off,
                data,
            });
            if m == last {
                break;
            }
            // The next level: 2 × 2 cells into one (sums and weights add).
            let (nw, nh) = (cw.div_ceil(2), ch.div_ceil(2));
            let (mut ns, mut nwt) = (vec![0f32; nw * nh], vec![0f32; nw * nh]);
            for j in 0..ch {
                for i in 0..cw {
                    ns[(j / 2) * nw + i / 2] += s[j * cw + i];
                    nwt[(j / 2) * nw + i / 2] += wt[j * cw + i];
                }
            }
            (s, wt, cw, ch) = (ns, nwt, nw, nh);
        }
        out.push(Mips { levels, first });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Rivers
// ---------------------------------------------------------------------------------------------

/// Share of the river segments (the larger discharges) drawn at each zoom, from zoom 0; beyond the
/// table, all of them. Cartography's generalisation: the planet shows its great rivers, a block
/// scale tile every stream.
const RIVER_SHARE: [f32; 6] = [0.03, 0.07, 0.15, 0.3, 0.55, 0.85];

struct RiverSet {
    rivers: Rivers,
    /// Discharge from which a segment is drawn, per zoom.
    q_min: Vec<f32>,
}

/// Rivers rasterised into a tile: coverage and the depth (blocks) of the drawn channel.
struct Coverage {
    alpha: Vec<f32>,
    depth: Vec<f32>,
}

impl RiverSet {
    fn new(rivers: Rivers, max_zoom: u32) -> RiverSet {
        let mut q: Vec<f32> = rivers.vertices.iter().map(|v| v.q).collect();
        q.sort_unstable_by(|a, b| a.total_cmp(b));
        let q_min = (0..=max_zoom)
            .map(|z| {
                let share = RIVER_SHARE.get(z as usize).copied().unwrap_or(1.0);
                if q.is_empty() || share >= 1.0 {
                    0.0
                } else {
                    q[(((1.0 - share) * q.len() as f32) as usize).min(q.len() - 1)]
                }
            })
            .collect();
        RiverSet { rivers, q_min }
    }

    /// Rasterises the channels over `g`'s tile.
    fn draw(&self, g: &TileGeom) -> Coverage {
        let mut cov = Coverage {
            alpha: vec![0.0; TILE * TILE],
            depth: vec![0.0; TILE * TILE],
        };
        let r = &self.rivers;
        if r.vertices.len() < 2 {
            return cov;
        }
        let c = f64::from(r.c);
        let q_min = self.q_min.get(g.z as usize).copied().unwrap_or(0.0);
        // Everything within 3 pixels of the tile (a wide river's bank) is looked up.
        let m = 3.0 * g.s + 4.0;
        let (bx0, bz0) = (g.x0 - m, g.z0 - m);
        let (bx1, bz1) = (g.x0 + TILE as f64 * g.s + m, g.z0 + TILE as f64 * g.s + m);
        let segs = r.entries_in(
            bx0.floor() as i64,
            bz0.floor() as i64,
            bx1.ceil() as i64,
            bz1.ceil() as i64,
        );
        for s in segs {
            let (a, b) = (&r.vertices[s as usize], &r.vertices[s as usize + 1]);
            let wide = f64::from(a.hw.max(b.hw)) * 2.0 / g.s;
            if (a.q + b.q) * 0.5 < q_min && wide < 1.5 {
                continue;
            }
            for k in -1i64..=1 {
                let shift = k as f64 * c;
                let to_px = |x: f32, z: f32| {
                    (
                        (f64::from(x) + shift - g.x0) / g.s,
                        (f64::from(z) - g.z0) / g.s,
                    )
                };
                let (pa, pb) = (to_px(a.x, a.z), to_px(b.x, b.z));
                let hw_max = f64::from(a.hw.max(b.hw)) / g.s;
                let pad = hw_max.max(0.5) + 1.5;
                let lo_i = (pa.0.min(pb.0) - pad).floor().max(0.0) as i64;
                let hi_i = (pa.0.max(pb.0) + pad).ceil().min(TILE as f64 - 1.0) as i64;
                let lo_j = (pa.1.min(pb.1) - pad).floor().max(0.0) as i64;
                let hi_j = (pa.1.max(pb.1) + pad).ceil().min(TILE as f64 - 1.0) as i64;
                if lo_i > hi_i || lo_j > hi_j {
                    continue;
                }
                let (ex, ez) = (pb.0 - pa.0, pb.1 - pa.1);
                let len2 = ex * ex + ez * ez;
                for j in lo_j..=hi_j {
                    for i in lo_i..=hi_i {
                        let (px, pz) = (i as f64, j as f64);
                        let t = if len2 > 0.0 {
                            (((px - pa.0) * ex + (pz - pa.1) * ez) / len2).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        let (dx, dz) = (px - (pa.0 + t * ex), pz - (pa.1 + t * ez));
                        let d = (dx * dx + dz * dz).sqrt();
                        let hw = (f64::from(a.hw) + (f64::from(b.hw) - f64::from(a.hw)) * t) / g.s;
                        // At least a one pixel line, so a stream the zoom cannot resolve still shows.
                        let half = hw.max(0.5);
                        let alpha = (half + 0.5 - d).clamp(0.0, 1.0) as f32;
                        let q = j as usize * TILE + i as usize;
                        if alpha > cov.alpha[q] {
                            cov.alpha[q] = alpha;
                            cov.depth[q] = (f64::from(a.depth)
                                + (f64::from(b.depth) - f64::from(a.depth)) * t)
                                as f32;
                        }
                    }
                }
            }
        }
        cov
    }
}

// ---------------------------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------------------------

/// What is known about a run's world, for the page.
#[derive(Clone, Debug)]
pub struct Meta {
    pub c: u32,
    pub sea_level_y: f64,
    pub min_y: i32,
    pub max_y: i32,
    pub m_per_block_h: f64,
    pub m_per_block_v: f64,
    pub radius_m: f64,
    pub source: String,
    pub created_utc: String,
    /// Blocks a cell of the terrain's grid (L3) is wide.
    pub l3_r: u32,
    pub max_zoom: u32,
    pub climate: bool,
    pub monthly: bool,
    pub detail: Option<DetailSpec>,
    pub rivers: usize,
    pub lakes: bool,
    /// The world's projection (the manifest's layout block).
    pub projection: deeptime_format::projection::WorldProjection,
}

/// Climate fields on L1 and L2, smoothed as the satellite frame smooths them.
struct ClimateMaps {
    /// °C per metre of height (the monthly layers are at sea level), 0 without them.
    gamma: f64,
    /// Temperatures at sea level (with the monthly layers: lapsed per block at draw time) or at
    /// the surface (without).
    t_mean: Grid,
    t_warm: Grid,
    t_cold: Grid,
    precip: Grid,
    aridity: Grid,
    koppen: Grid,
    ela: Grid,
    herb: Grid,
    tree: Grid,
    sea_ice: Grid,
    rock: Option<[Grid; 3]>,
    ice: Option<Grid>,
}

/// Counters of the tiles rendered, per layer.
#[derive(Default)]
pub struct Timing {
    n: [AtomicU64; 4],
    us: [AtomicU64; 4],
    max_us: [AtomicU64; 4],
}

impl Timing {
    fn add(&self, l: Layer, us: u64) {
        let k = Layer::ALL.iter().position(|x| *x == l).unwrap_or(0);
        self.n[k].fetch_add(1, Ordering::Relaxed);
        self.us[k].fetch_add(us, Ordering::Relaxed);
        self.max_us[k].fetch_max(us, Ordering::Relaxed);
    }

    pub fn json(&self) -> Value {
        let mut o = serde_json::Map::new();
        for (k, l) in Layer::ALL.iter().enumerate() {
            let n = self.n[k].load(Ordering::Relaxed);
            if n > 0 {
                let total = self.us[k].load(Ordering::Relaxed);
                o.insert(
                    l.id().into(),
                    json!({
                        "tiles": n,
                        "mean_ms": total as f64 / n as f64 / 1000.0,
                        "max_ms": self.max_us[k].load(Ordering::Relaxed) as f64 / 1000.0,
                    }),
                );
            }
        }
        Value::Object(o)
    }
}

/// A run's baked world, opened for tiles.
pub struct BakeMap {
    pub meta: Meta,
    elev: LayerStore,
    amp: Option<LayerStore>,
    lakes: Option<LayerStore>,
    /// The `closed_basin` layer (HYD-100): ground below sea level the sea does not reach.
    closed: Option<LayerStore>,
    noise: Option<(DetailSpec, Noise)>,
    elev_mips: Mips,
    lake_frac: Option<Mips>,
    lake_level: Option<Mips>,
    closed_frac: Option<Mips>,
    climate: Option<ClimateMaps>,
    rivers: Option<RiverSet>,
    pal: Palette,
    look: Look,
    light: Light,
    /// The elevation ramp, in steps of 1/4 block from `lut_lo`.
    lut: Vec<[u8; 3]>,
    lut_lo: f64,
    pub timing: Timing,
}

const LUT_LO: f64 = -400.0;
const LUT_HI: f64 = 720.0;

/// A tile's heights and lakes.
struct Heights {
    /// Terrain height, blocks, `N × N` (a pixel of border).
    h: Vec<f32>,
    /// Lake surface at the pixel (NaN: no lake); empty without lakes.
    lake_level: Vec<f32>,
    /// Share of the pixel that is lake (coarse tiles); empty on fine tiles, where a lake is a
    /// pixel whose terrain is under its level.
    lake_frac: Vec<f32>,
    /// Share of the pixel in a closed basin (`closed_basin`, HYD-100: no sea there); empty
    /// without the layer.
    closed: Vec<f32>,
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    std::array::from_fn(|k| a[k] + (b[k] - a[k]) * t)
}

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

impl BakeMap {
    /// Opens `<dir>` (a world-data folder with a manifest): the terrain, its detail, the lakes and
    /// rivers, the climate and the surface rock, whichever it has, and builds the mips and the
    /// smoothed climate fields.
    pub fn open(dir: &Path) -> Result<BakeMap, String> {
        let manifest = Manifest::read(dir).map_err(|e| e.to_string())?;
        let layer = |name: &str| manifest.layer(name).map(|e| dir.join(&e.path));
        let elev_path = layer("elevation")
            .ok_or("this world has no terrain (its preset bakes no elevation)")?;
        let elev = LayerStore::open(&elev_path)?;
        let c = manifest.scale.circumference_blocks;
        let l3 = elev.desc().grid.clone();
        let zmax = max_zoom(c);
        // Mips up to the coarsest zoom's pixel size.
        let s0 = blocks_per_px(c, 0);
        let mip_last = ((s0 / f64::from(l3.r)).log2().ceil().max(0.0) as u32).max(MIP_MIN);
        let elev_mips = build_mips(&elev, MIP_MIN, mip_last, &[|v| (v, 1.0)])?
            .pop()
            .expect("one channel");

        let detail = DetailSpec::of(&manifest).map_err(|e| e.to_string())?;
        let (amp, noise) = match &detail {
            Some(spec) => {
                let p = layer(&spec.layer)
                    .ok_or("notes.bake.detail names a layer the manifest lacks")?;
                let noise = Noise::for_world(
                    spec.seed().map_err(|e| e.to_string())?,
                    c,
                    &manifest.world_projection(),
                );
                (Some(LayerStore::open(&p)?), Some((spec.clone(), noise)))
            }
            None => (None, None),
        };
        let lakes = match layer("water_level") {
            Some(p) => Some(LayerStore::open(&p)?),
            None => None,
        };
        let (mut lake_frac, mut lake_level) = (None, None);
        if let Some(l) = &lakes {
            let lg = &l.desc().grid;
            let last = ((s0 / f64::from(lg.r)).log2().ceil().max(0.0) as u32).max(MIP_MIN);
            let mut m = build_mips(
                l,
                MIP_MIN,
                last,
                &[
                    |v| (if v.is_finite() { 1.0 } else { 0.0 }, 1.0),
                    |v| {
                        if v.is_finite() { (v, 1.0) } else { (0.0, 0.0) }
                    },
                ],
            )?;
            lake_level = m.pop();
            lake_frac = m.pop();
        }
        let closed = match layer(deeptime_format::column::CLOSED_BASIN) {
            Some(p) => Some(LayerStore::open(&p)?),
            None => None,
        };
        let closed_frac = match &closed {
            Some(l) => {
                let r = f64::from(l.desc().grid.r);
                let last = ((s0 / r).log2().ceil().max(0.0) as u32).max(MIP_MIN);
                build_mips(l, MIP_MIN, last, &[|v| (f32::from(v == 1.0), 1.0)])?.pop()
            }
            None => None,
        };
        let rivers = read_rivers(dir)
            .map_err(|e| e.to_string())?
            .filter(|r| r.vertices.len() >= 2)
            .map(|r| RiverSet::new(r, zmax));
        let preset = read_json(&dir.join("preset.json"));
        let radius_m = preset
            .as_ref()
            .and_then(|p| p.pointer("/tectonics/model_radius_km"))
            .and_then(Value::as_f64)
            .map_or(6.37e6, |km| km * 1000.0);
        let ela_offset = preset
            .as_ref()
            .and_then(|p| p.pointer("/climate/ela_offset_c"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let sea = f64::from(manifest.scale.sea_level_y);
        let climate = Self::climate(dir, &manifest, ela_offset)?;
        let ramps = Ramps::builtin();
        let pal = ramps
            .ramps
            .get(satellite::RAMP)
            .map(Palette::from_ramp)
            .unwrap_or_else(Palette::builtin);
        let (_, ramp) = ramps.for_layer("elevation");
        let ramp = ramp.clone();
        let lut = (0..((LUT_HI - LUT_LO) * 4.0) as usize)
            .map(|k| ramp.colour(LUT_LO + k as f64 / 4.0))
            .collect();
        let look = Look::default();
        let meta = Meta {
            c,
            sea_level_y: sea,
            min_y: manifest.scale.min_y,
            max_y: manifest.scale.max_y,
            m_per_block_h: manifest.scale.m_per_block_h,
            m_per_block_v: manifest.scale.m_per_block_v,
            radius_m,
            source: manifest.source.clone(),
            created_utc: manifest.created_utc.clone(),
            l3_r: l3.r,
            max_zoom: zmax,
            climate: climate.is_some(),
            monthly: climate.as_ref().is_some_and(|c| c.gamma > 0.0),
            detail: detail.clone(),
            rivers: rivers.as_ref().map_or(0, |r| r.rivers.chains.len()),
            lakes: lakes.is_some(),
            projection: manifest.world_projection(),
        };
        Ok(BakeMap {
            meta,
            elev,
            amp,
            lakes,
            closed,
            noise,
            elev_mips,
            lake_frac,
            lake_level,
            closed_frac,
            climate,
            rivers,
            light: Light::new(&look),
            look,
            pal,
            lut,
            lut_lo: LUT_LO,
            timing: Timing::default(),
        })
    }

    /// The climate fields: from the monthly layers when the bake has them (format 0.2), else the
    /// annual ones, else none. `Ok(None)` when the world has no climate at all.
    fn climate(dir: &Path, m: &Manifest, ela_offset: f64) -> Result<Option<ClimateMaps>, String> {
        let path = |n: &str| m.layer(n).map(|e| dir.join(&e.path));
        let (Some(pk), Some(pp)) = (path("climate/koppen"), path("climate/p_mm")) else {
            return Ok(None);
        };
        let (g1, kop) = LayerStore::load_f32(&pk)?;
        let (_, precip) = LayerStore::load_f32(&pp)?;
        let n = kop.len();
        let t_mean_c = match path("climate/t_mean_c") {
            Some(p) => LayerStore::load_f32(&p)?.1,
            None => vec![f32::NAN; n],
        };
        let months: Vec<Vec<f32>> = (1..=12)
            .map_while(|k| path(&format!("climate/month/t_sl_c_{k:02}")))
            .map(|p| LayerStore::load_f32(&p).map(|x| x.1))
            .collect::<Result<_, _>>()?;
        let monthly = months.len() == 12 && months.iter().all(|v| v.len() == n);
        let gamma = m
            .notes
            .get("bake")
            .and_then(|b| b.pointer("/climate/monthly/lapse_c_per_km"))
            .and_then(Value::as_f64)
            .map_or(0.0, |g| g / 1000.0);
        let monthly = monthly && gamma > 0.0;
        let look = Look::default();
        let sea_code = f32::from(koppen::SEA);

        // Per L1 cell: surface temperatures (as the climate stage had them at its own height),
        // the sea-level ones, aridity P/PET and the equilibrium line.
        #[derive(Clone, Copy, Default)]
        struct Prep {
            surf: [f32; 3],
            sl: [f32; 3],
            aridity: f32,
            ela: f32,
            sea_ice: f32,
        }
        let mut prep = vec![Prep::default(); n];
        let threads = std::thread::available_parallelism().map_or(2, |n| n.get().min(4));
        let chunk = n.div_ceil(threads);
        std::thread::scope(|sc| {
            for (ci, out) in prep.chunks_mut(chunk).enumerate() {
                let (months, t_mean_c, precip, kop) = (&months, &t_mean_c, &precip, &kop);
                sc.spawn(move || {
                    for (k, p) in out.iter_mut().enumerate() {
                        let c = ci * chunk + k;
                        if !monthly {
                            let t = t_mean_c[c];
                            p.surf = [t, f32::NAN, f32::NAN];
                            p.sl = p.surf;
                            p.aridity = f32::NAN;
                            p.ela = f32::NAN;
                            continue;
                        }
                        let mut tsl = [0f64; 12];
                        for (m, v) in months.iter().enumerate() {
                            tsl[m] = f64::from(v[c]);
                        }
                        let mean_sl = tsl.iter().sum::<f64>() / 12.0;
                        // What the climate stage lapsed to its own elevation: the baked annual mean
                        // is the sea-level mean less that lapse.
                        let delta = if t_mean_c[c].is_finite() {
                            mean_sl - f64::from(t_mean_c[c])
                        } else {
                            0.0
                        };
                        let t: [f64; 12] = std::array::from_fn(|m| tsl[m] - delta);
                        let (warm, cold) = (
                            t.iter().cloned().fold(f64::MIN, f64::max),
                            t.iter().cloned().fold(f64::MAX, f64::min),
                        );
                        let (warm_sl, cold_sl) = (
                            tsl.iter().cloned().fold(f64::MIN, f64::max),
                            tsl.iter().cloned().fold(f64::MAX, f64::min),
                        );
                        p.surf = [(mean_sl - delta) as f32, warm as f32, cold as f32];
                        p.sl = [mean_sl as f32, warm_sl as f32, cold_sl as f32];
                        let annual = f64::from(precip[c]);
                        let pet = snapshot::thornthwaite(&t);
                        p.aridity = if pet > 0.0 {
                            (annual / pet).min(10.0) as f32
                        } else {
                            10.0
                        };
                        let t_ela = glaciers::ela_temperature(annual) + ela_offset;
                        p.ela = ((glaciers::summer(&tsl) - t_ela) / gamma) as f32;
                        if kop[c] == sea_code {
                            // The frame's sea-ice cover comes from the energy balance model's
                            // ice slab, which the bake does not keep. The baked sea-level
                            // temperature is the open water's, so it runs below freezing under
                            // ice; the annual mean tracks the model's ice edge: none above -1 °C,
                            // complete from -7 °C (calibrated on the run's own frames: solid
                            // ice poleward of 64°, an edge fading out by 55°).
                            p.sea_ice = smoothstep(-1.0, -7.0, mean_sl as f32);
                        }
                    }
                });
            }
        });
        let field =
            |f: &dyn Fn(&Prep) -> f32| Grid::new(&g1, prep.iter().map(f).collect::<Vec<f32>>());
        let smooth =
            |grid: &Grid, r: (usize, usize), log: bool, keep: Option<&dyn Fn(usize) -> bool>| {
                Grid::new(
                    &g1,
                    satellite::box_filter(&grid.core(&g1), r, log, keep).data,
                )
            };
        let is_sea = |k: usize| kop[k] == sea_code;
        let is_land = |k: usize| kop[k] != sea_code;
        // Temperatures smoothed over ±3 rows, as the frame map does (the energy balance model's
        // latitude bands), moisture over 3 × 3 cells in log space.
        let (surf_mean, surf_warm, surf_cold) = (
            smooth(&field(&|p| p.surf[0]), (0, 3), false, None),
            smooth(&field(&|p| p.surf[1]), (0, 3), false, None),
            smooth(&field(&|p| p.surf[2]), (0, 3), false, None),
        );
        let (sl_mean, sl_warm, sl_cold) = if monthly {
            (
                smooth(&field(&|p| p.sl[0]), (0, 3), false, None),
                smooth(&field(&|p| p.sl[1]), (0, 3), false, None),
                smooth(&field(&|p| p.sl[2]), (0, 3), false, None),
            )
        } else {
            (
                Grid::new(&g1, surf_mean.data.clone()),
                Grid::new(&g1, surf_warm.data.clone()),
                Grid::new(&g1, surf_cold.data.clone()),
            )
        };
        let precip_g = Grid::new(&g1, precip.clone());
        let precip_s = smooth(&precip_g, (1, 1), true, None);
        let aridity_s = smooth(&field(&|p| p.aridity), (1, 1), true, None);
        let ela = field(&|p| p.ela);
        let sea_ice = smooth(&field(&|p| p.sea_ice), (2, 8), false, Some(&is_sea));
        // Herb and tree cover per cell from the smoothed climate, then smoothed over land.
        let (mut herb, mut tree) = (vec![f32::NAN; n], vec![f32::NAN; n]);
        for k in 0..n {
            let mut c = Cell::at(0.0);
            c.t_mean_c = surf_mean.data[k];
            c.t_warm_c = surf_warm.data[k];
            c.t_cold_c = surf_cold.data[k];
            c.aridity = aridity_s.data[k];
            c.precip_mm = precip_s.data[k];
            c.koppen = kop[k];
            if c.has_climate() {
                (herb[k], tree[k]) = satellite::cover(&c, &look);
            }
        }
        let herb = smooth(&Grid::new(&g1, herb), (1, 1), false, Some(&is_land));
        let tree = smooth(&Grid::new(&g1, tree), (1, 1), false, Some(&is_land));

        // The surface rock's colour (L2) and the ice cover (L2).
        let pal = Palette::builtin();
        let rock = match (
            path("surface/class_top"),
            read_json(&dir.join("strata/classes.json")),
        ) {
            (Some(p), Some(classes)) => {
                let (g2, raw) = LayerStore::load_f32(&p)?;
                let names: Vec<&str> = classes["classes"]
                    .as_array()
                    .map(|a| a.iter().map(|c| c.as_str().unwrap_or("")).collect())
                    .unwrap_or_default();
                let tint: Vec<Option<[f32; 3]>> = names
                    .iter()
                    .map(|name| {
                        deeptime_core::synth::EVERY_CLASS
                            .iter()
                            .position(|c| c == name)
                            .and_then(|i| satellite::rock_tint(i as f32, &pal))
                    })
                    .collect();
                let mut ch = [
                    vec![f32::NAN; raw.len()],
                    vec![f32::NAN; raw.len()],
                    vec![f32::NAN; raw.len()],
                ];
                for (k, &v) in raw.iter().enumerate() {
                    if v.is_finite()
                        && let Some(Some(t)) = tint.get(v as usize)
                    {
                        for c in 0..3 {
                            ch[c][k] = t[c];
                        }
                    }
                }
                let smooth2 = |data: Vec<f32>| {
                    let g = Grid::new(&g2, data);
                    Grid::new(
                        &g2,
                        satellite::box_filter(&g.core(&g2), (1, 1), false, None).data,
                    )
                };
                let [a, b, c] = ch;
                Some([smooth2(a), smooth2(b), smooth2(c)])
            }
            _ => None,
        };
        let step_m = m
            .notes
            .get("bake")
            .and_then(|b| b.pointer("/climate/ice_cover/step_m"))
            .and_then(Value::as_f64)
            .unwrap_or(160.0) as f32;
        let ice = match path("climate/ice_cover") {
            Some(p) => {
                let (g2, blocks) = LayerStore::load_f32(&p)?;
                // `1 + ⌊t / step⌋` blocks for t metres of ice: the middle of the step. Not
                // blurred: the map reads the cells by the game's cover rule (`cell`), so its edge
                // is the game's, a staircase of L2 cells.
                let data: Vec<f32> = blocks
                    .iter()
                    .map(|&b| if b >= 1.0 { (b - 0.5) * step_m } else { 0.0 })
                    .collect();
                Some(Grid::new(&g2, data))
            }
            None => None,
        };
        Ok(Some(ClimateMaps {
            gamma: if monthly { gamma } else { 0.0 },
            t_mean: sl_mean,
            t_warm: sl_warm,
            t_cold: sl_cold,
            precip: precip_s,
            aridity: aridity_s,
            koppen: Grid::new(&g1, kop),
            ela,
            herb,
            tree,
            sea_ice,
            rock,
            ice,
        }))
    }
}

// ---------------------------------------------------------------------------------------------
// Heights
// ---------------------------------------------------------------------------------------------

/// Vertical exaggeration of the hill shade at `s` blocks a pixel, relative to the frame maps'
/// (which draw 16 blocks a pixel): finer tiles show less height between neighbouring pixels, so
/// the relief is lit harder as the map zooms in, to the factor below at most.
fn exaggeration_boost(s: f64) -> f64 {
    (16.0 / s).sqrt().clamp(1.0, 4.0)
}

/// The window of a layer covering block box `[lo_x, hi_x] × [lo_z, hi_z]` and a margin.
fn window_for(
    store: &LayerStore,
    (lo_x, hi_x): (f64, f64),
    (lo_z, hi_z): (f64, f64),
) -> Result<Window, String> {
    let g = &store.desc().grid;
    let r = f64::from(g.r);
    let (ox, oz) = (g.origin_block_x as f64, g.origin_block_z as f64);
    let i0 = ((lo_x - ox) / r).floor() as i64 - 1;
    let j0 = ((lo_z - oz) / r).floor() as i64 - 1;
    let i1 = ((hi_x - ox) / r).floor() as i64 + 2;
    let j1 = ((hi_z - oz) / r).floor() as i64 + 2;
    store.window(i0, j0, (i1 - i0 + 1) as usize, (j1 - j0 + 1) as usize)
}

impl BakeMap {
    /// The mip level (of the elevation) tiles of `s` blocks a pixel read, or `None` for the L3
    /// cells themselves.
    fn mip_level(&self, s: f64) -> Option<u32> {
        let m = (s / f64::from(self.meta.l3_r)).log2().floor().max(0.0) as u32;
        let last = self.elev_mips.first + self.elev_mips.levels.len() as u32 - 1;
        (m >= MIP_MIN).then_some(m.min(last))
    }

    /// The terrain of tile `g`, with its lakes.
    fn heights(&self, g: &TileGeom) -> Result<Heights, String> {
        let mut h = vec![0f32; N * N];
        let (mut lake_level, mut lake_frac) = (Vec::new(), Vec::new());
        let closed = self.closed_cover(g)?;
        let pos =
            |a: usize, b: usize| (g.x0 + (a as f64 - 1.0) * g.s, g.z0 + (b as f64 - 1.0) * g.s);
        let span_x = (g.x0 - 2.0 * g.s, g.x0 + (TILE as f64 + 1.5) * g.s);
        let span_z = (g.z0 - 2.0 * g.s, g.z0 + (TILE as f64 + 1.5) * g.s);
        if let Some(m) = self.mip_level(g.s) {
            let grid = self.elev_mips.level(m).expect("the level exists");
            for b in 0..N {
                for a in 0..N {
                    let (bx, bz) = pos(a, b);
                    h[b * N + a] = grid.bilinear(bx, bz);
                }
            }
            if let (Some(frac), Some(level), Some(store)) =
                (&self.lake_frac, &self.lake_level, &self.lakes)
            {
                let r = f64::from(store.desc().grid.r);
                let last = frac.first + frac.levels.len() as u32 - 1;
                let ml = ((g.s / r).log2().floor().max(0.0) as u32).clamp(frac.first, last);
                let (fg, lg) = (
                    frac.level(ml).expect("level"),
                    level.level(ml).expect("level"),
                );
                lake_level = vec![f32::NAN; N * N];
                lake_frac = vec![0f32; N * N];
                for b in 0..N {
                    for a in 0..N {
                        let (bx, bz) = pos(a, b);
                        let st = fg.stencil(bx, bz);
                        let f = fg.at(&st);
                        if f > 0.004 {
                            lake_frac[b * N + a] = f.min(1.0);
                            lake_level[b * N + a] = lg.nearest(&lg.stencil(bx, bz));
                        }
                    }
                }
            }
            return Ok(Heights {
                h,
                lake_level,
                lake_frac,
                closed,
            });
        }
        // Fine tiles: the L3 cells, bilinear, a box of k × k samples under each pixel when a pixel
        // is wider than a cell, and the detail term at its middle.
        let r = f64::from(self.meta.l3_r);
        let k = (g.s / r).ceil().clamp(1.0, 4.0) as usize;
        let reach = 0.5 * g.s + r;
        let win = window_for(
            &self.elev,
            (span_x.0 - reach, span_x.1 + reach),
            (span_z.0 - reach, span_z.1 + reach),
        )?;
        let amp = match &self.amp {
            Some(a) => Some(window_for(
                a,
                (span_x.0 - reach, span_x.1 + reach),
                (span_z.0 - reach, span_z.1 + reach),
            )?),
            None => None,
        };
        let off: Vec<f64> = (0..k)
            .map(|u| ((u as f64 + 0.5) / k as f64 - 0.5) * g.s)
            .collect();
        let inv = 1.0 / (k * k) as f32;
        for b in 0..N {
            for a in 0..N {
                let (bx, bz) = pos(a, b);
                let mut e = 0f32;
                if k == 1 {
                    e = win.bilinear(bx, bz);
                } else {
                    for &dz in &off {
                        for &dx in &off {
                            e += win.bilinear(bx + dx, bz + dz);
                        }
                    }
                    e *= inv;
                }
                if let (Some(amp), Some((spec, noise))) = (&amp, &self.noise) {
                    e += (f64::from(amp.bilinear(bx, bz)) * spec.noise(noise, bx, bz)) as f32;
                }
                h[b * N + a] = e;
            }
        }
        if let Some(store) = &self.lakes {
            let lw = window_for(store, span_x, span_z)?;
            lake_level = vec![f32::NAN; N * N];
            for b in 0..N {
                for a in 0..N {
                    let (bx, bz) = pos(a, b);
                    lake_level[b * N + a] = lw.nearest(bx, bz);
                }
            }
        }
        Ok(Heights {
            h,
            lake_level,
            lake_frac,
            closed,
        })
    }

    /// The share of each pixel of tile `g` (bordered, `N × N`) in a closed basin below sea level
    /// (HYD-100): the mip's mean on coarse tiles, the nearest cell on fine ones; empty without the
    /// layer.
    fn closed_cover(&self, g: &TileGeom) -> Result<Vec<f32>, String> {
        let Some(store) = &self.closed else {
            return Ok(Vec::new());
        };
        let pos =
            |a: usize, b: usize| (g.x0 + (a as f64 - 1.0) * g.s, g.z0 + (b as f64 - 1.0) * g.s);
        let mut out = vec![0f32; N * N];
        let r = f64::from(store.desc().grid.r);
        if let (Some(_), Some(frac)) = (self.mip_level(g.s), &self.closed_frac) {
            let last = frac.first + frac.levels.len() as u32 - 1;
            let ml = ((g.s / r).log2().floor().max(0.0) as u32).clamp(frac.first, last);
            let fg = frac.level(ml).expect("level");
            for b in 0..N {
                for a in 0..N {
                    let (bx, bz) = pos(a, b);
                    out[b * N + a] = fg.at(&fg.stencil(bx, bz)).clamp(0.0, 1.0);
                }
            }
        } else {
            let span_x = (g.x0 - 2.0 * g.s, g.x0 + (TILE as f64 + 1.5) * g.s);
            let span_z = (g.z0 - 2.0 * g.s, g.z0 + (TILE as f64 + 1.5) * g.s);
            let w = window_for(store, span_x, span_z)?;
            for b in 0..N {
                for a in 0..N {
                    let (bx, bz) = pos(a, b);
                    out[b * N + a] = f32::from(w.nearest(bx, bz) == 1.0);
                }
            }
        }
        Ok(out)
    }

    /// Whether pixel `k` (in the bordered tile) lies mostly in a closed basin, where no sea stands.
    #[inline]
    fn closed_at(hg: &Heights, k: usize) -> bool {
        hg.closed.get(k).is_some_and(|&c| c >= 0.5)
    }

    /// The share of pixel `(a, b)` (in the bordered tile) that is lake and its surface.
    #[inline]
    fn lake_at(&self, hg: &Heights, k: usize) -> (f32, f32) {
        if hg.lake_level.is_empty() {
            return (0.0, f32::NAN);
        }
        let lvl = hg.lake_level[k];
        if !hg.lake_frac.is_empty() {
            return (hg.lake_frac[k], lvl);
        }
        if lvl.is_finite() {
            // A lake pixel is one whose ground is under the lake's surface; the edge is a little
            // soft so a shore is not a staircase.
            (smoothstep(0.0, 0.6, lvl - hg.h[k]), lvl)
        } else {
            (0.0, lvl)
        }
    }

    /// The hill shade of a tile: heights flat over lakes, slopes in the model sphere's metres.
    fn shade_field(&self, g: &TileGeom, hg: &Heights) -> Vec<f32> {
        let mv = self.meta.m_per_block_v;
        let c = f64::from(self.meta.c);
        let metres_px = g.s * 2.0 * std::f64::consts::PI * self.meta.radius_m / c;
        let boost = exaggeration_boost(g.s);
        let flat = |k: usize| -> f32 {
            let (frac, lvl) = self.lake_at(hg, k);
            if frac > 0.5 && lvl.is_finite() {
                lvl
            } else {
                hg.h[k]
            }
        };
        let mut out = vec![1f32; TILE * TILE];
        let lambert = match self.meta.projection {
            deeptime_format::projection::WorldProjection::Lambert(p) => Some(p),
            deeptime_format::projection::WorldProjection::PetroffGuyou(_) => None,
        };
        // A Petroff–Guyou world's blocks are what a player walks: slopes in its own block metric,
        // a block `m_per_block_h` wide.
        let metres_px = if lambert.is_some() {
            metres_px
        } else {
            g.s * self.meta.m_per_block_h
        };
        for j in 0..TILE {
            let zj = g.z0 + j as f64 * g.s;
            // On the Lambert map a pixel is s·cos φ blocks of planet wide and s / cos φ high.
            let (dx, dy) = match lambert {
                Some(proj) => (
                    metres_px * proj.scale_ew(zj).max(0.15),
                    metres_px * proj.scale_ns(zj).min(1.0 / 0.15),
                ),
                None => (metres_px, metres_px),
            };
            for i in 0..TILE {
                let k = (j + 1) * N + i + 1;
                let dzdx = f64::from(flat(k + 1) - flat(k - 1)) * mv / (2.0 * dx);
                let dzdy = f64::from(flat(k + N) - flat(k - N)) * mv / (2.0 * dy);
                out[j * TILE + i] = self.light.shade_exaggerated(dzdx, dzdy, boost);
            }
        }
        out
    }

    // -----------------------------------------------------------------------------------------
    // Layers
    // -----------------------------------------------------------------------------------------

    /// Tile `g` of `layer` as RGBA8, `TILE × TILE`.
    pub fn render(&self, layer: Layer, g: &TileGeom) -> Result<Vec<u8>, String> {
        let started = Instant::now();
        let hg = self.heights(g)?;
        let cov = self.rivers.as_ref().map(|r| r.draw(g));
        let mut rgba = match layer {
            Layer::Satellite => self.paint_satellite(g, &hg, cov.as_ref()),
            Layer::Elevation => self.paint_elevation(g, &hg, cov.as_ref()),
            Layer::Water => self.paint_water(g, &hg, cov.as_ref()),
            Layer::Overlay => self.paint_overlay(&hg, cov.as_ref()),
        };
        // Past the poles there is no planet: the page's background (transparent on the overlay).
        // A Petroff–Guyou frame is all planet.
        let (zlo, zhi) = self.meta.projection.z_range();
        let beyond: [u8; 4] = if layer.opaque() { BEYOND_POLE } else { [0; 4] };
        for j in 0..TILE {
            let zj = g.z0 + j as f64 * g.s;
            if zj < zlo || zj > zhi {
                for px in rgba[j * TILE * 4..(j + 1) * TILE * 4].chunks_mut(4) {
                    px.copy_from_slice(&beyond);
                }
            }
        }
        self.timing.add(
            layer,
            started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64,
        );
        Ok(rgba)
    }

    /// The pixel's climate and rock for the satellite look.
    #[inline]
    fn cell(&self, bx: f64, bz: f64, z_m: f32) -> Cell {
        let mut c = Cell::at(z_m);
        let Some(cm) = &self.climate else { return c };
        let s1 = cm.t_mean.stencil(bx, bz);
        // The monthly layers are at sea level: the block's own height lapses them (FORMAT.md
        // "The climate at a block"), so snow lines and tree lines follow the relief.
        let lapse = (cm.gamma * f64::from(z_m.max(0.0))) as f32;
        c.t_mean_c = cm.t_mean.at(&s1) - lapse;
        c.t_warm_c = cm.t_warm.at(&s1) - lapse;
        c.t_cold_c = cm.t_cold.at(&s1) - lapse;
        c.precip_mm = cm.precip.at(&s1);
        c.aridity = cm.aridity.at(&s1);
        c.koppen = cm.koppen.nearest(&s1);
        c.ela_m = cm.ela.at(&s1);
        c.herb_cover = cm.herb.at(&s1);
        c.tree_cover = cm.tree.at(&s1);
        c.sea_ice = cm.sea_ice.at(&s1);
        if let Some(ice) = &cm.ice {
            // The cover as the game lays it: the layer's cells by the shared cover rule (covered
            // where the nearest cell is, as the class, at the covered cells' blend). Where in the
            // climate's ice the ground can hold it the bake decided per L2 cell (`bake::ice`).
            let s = ice.stencil(bx, bz);
            c.ice_m = cover_of(
                f64::from(ice.nearest(&s)),
                (0..4).map(|q| (f64::from(ice.data[s.k[q]]), f64::from(s.f[q]))),
            ) as f32;
        }
        if let Some(rock) = &cm.rock {
            let s2 = rock[0].stencil(bx, bz);
            c.rock_rgb = [rock[0].at(&s2), rock[1].at(&s2), rock[2].at(&s2)];
        }
        c
    }

    fn paint_satellite(&self, g: &TileGeom, hg: &Heights, cov: Option<&Coverage>) -> Vec<u8> {
        let (sea, mv) = (self.meta.sea_level_y, self.meta.m_per_block_v);
        let shade = self.shade_field(g, hg);
        let mut out = Vec::with_capacity(TILE * TILE * 4);
        for j in 0..TILE {
            let bz = g.z0 + j as f64 * g.s;
            for i in 0..TILE {
                let bx = g.x0 + i as f64 * g.s;
                let k = (j + 1) * N + i + 1;
                let q = j * TILE + i;
                let hh = f64::from(hg.h[k]);
                let mut z_m = ((hh - sea) * mv) as f32;
                if Self::closed_at(hg, k) {
                    z_m = z_m.max(0.0); // no sea in a closed basin (HYD-100)
                }
                let cell = self.cell(bx, bz, z_m);
                let (mut rgb, mut strength) = satellite::colour(&cell, &self.pal, &self.look);
                let (lake, lvl) = self.lake_at(hg, k);
                if lake > 0.0 {
                    let depth = ((f64::from(lvl) - hh) * mv).max(10.0) as f32;
                    let (lrgb, lstrength) =
                        satellite::colour(&Cell::at(-depth), &self.pal, &self.look);
                    rgb = mix3(rgb, lrgb, lake);
                    strength += (lstrength - strength) * lake;
                }
                let mut px = satellite::shaded(rgb, strength, shade[q]).map(f32::from);
                if let Some(cv) = cov {
                    let a = cv.alpha[q] * (1.0 - lake);
                    if a > 0.0 {
                        let depth = (f64::from(cv.depth[q]) * mv).max(8.0) as f32;
                        let (rrgb, _) = satellite::colour(&Cell::at(-depth), &self.pal, &self.look);
                        px = mix3(px, rrgb.map(|v| v * 0.92), a);
                    }
                }
                out.extend_from_slice(&px.map(|v| v.round().clamp(0.0, 255.0) as u8));
                out.push(255);
            }
        }
        out
    }

    #[inline]
    fn ramp_colour(&self, y: f64) -> [f32; 3] {
        let k = (((y - self.lut_lo) * 4.0).max(0.0) as usize).min(self.lut.len() - 1);
        self.lut[k].map(f32::from)
    }

    fn paint_elevation(&self, g: &TileGeom, hg: &Heights, cov: Option<&Coverage>) -> Vec<u8> {
        let sea = self.meta.sea_level_y;
        let shade = self.shade_field(g, hg);
        let mut out = Vec::with_capacity(TILE * TILE * 4);
        for j in 0..TILE {
            for i in 0..TILE {
                let k = (j + 1) * N + i + 1;
                let q = j * TILE + i;
                let hh = f64::from(hg.h[k]);
                // No sea in a closed basin (HYD-100): its ground below sea level is the lowest land.
                let seen = if Self::closed_at(hg, k) {
                    hh.max(sea + 0.01)
                } else {
                    hh
                };
                let mut rgb = self.ramp_colour(seen);
                let mut strength = if seen < sea { 0.4 } else { 0.8 };
                let (lake, lvl) = self.lake_at(hg, k);
                if lake > 0.0 {
                    let depth = (f64::from(lvl) - hh).max(0.0);
                    rgb = mix3(rgb, self.ramp_colour(sea - 0.01 - 6.0 * depth), lake);
                    strength = strength + (0.3 - strength) * lake;
                }
                let f = (1.0 + strength * (shade[q] - 1.0)).clamp(0.5, 1.35);
                let mut px = rgb.map(|v| v * f);
                if let Some(cv) = cov {
                    px = mix3(px, RIVER_RGB, cv.alpha[q] * (1.0 - lake));
                }
                out.extend_from_slice(&px.map(|v| v.round().clamp(0.0, 255.0) as u8));
                out.push(255);
            }
        }
        out
    }

    fn paint_water(&self, g: &TileGeom, hg: &Heights, cov: Option<&Coverage>) -> Vec<u8> {
        let (sea, mv) = (self.meta.sea_level_y, self.meta.m_per_block_v);
        let shade = self.shade_field(g, hg);
        let mut out = Vec::with_capacity(TILE * TILE * 4);
        for j in 0..TILE {
            for i in 0..TILE {
                let k = (j + 1) * N + i + 1;
                let q = j * TILE + i;
                let hh = f64::from(hg.h[k]);
                let mut z_m = ((hh - sea) * mv) as f32;
                if Self::closed_at(hg, k) {
                    z_m = z_m.max(0.0); // no sea in a closed basin (HYD-100)
                }
                let (mut rgb, mut strength) = if z_m < 0.0 {
                    (
                        mix3(SEA_SHALLOW, SEA_DEEP, smoothstep(0.0, 4500.0, -z_m)),
                        0.3,
                    )
                } else {
                    (mix3(LAND_LOW, LAND_HIGH, smoothstep(0.0, 3500.0, z_m)), 0.5)
                };
                let (lake, lvl) = self.lake_at(hg, k);
                if lake > 0.0 {
                    let depth = ((f64::from(lvl) - hh).max(0.0) * mv) as f32;
                    let lrgb = mix3(LAKE_SHALLOW, LAKE_DEEP, smoothstep(0.0, 250.0, depth));
                    rgb = mix3(rgb, lrgb, lake);
                    strength += (0.2 - strength) * lake;
                }
                let f = (1.0 + strength * (shade[q] - 1.0)).clamp(0.5, 1.35);
                let mut px = rgb.map(|v| v * f);
                if let Some(cv) = cov {
                    let a = cv.alpha[q] * (1.0 - lake);
                    if a > 0.0 {
                        px = mix3(px, river_colour(cv.depth[q]), a);
                    }
                }
                out.extend_from_slice(&px.map(|v| v.round().clamp(0.0, 255.0) as u8));
                out.push(255);
            }
        }
        out
    }

    /// Lakes and rivers alone, over a transparent ground.
    fn paint_overlay(&self, hg: &Heights, cov: Option<&Coverage>) -> Vec<u8> {
        let mv = self.meta.m_per_block_v;
        let mut out = Vec::with_capacity(TILE * TILE * 4);
        for j in 0..TILE {
            for i in 0..TILE {
                let k = (j + 1) * N + i + 1;
                let q = j * TILE + i;
                let (lake, lvl) = self.lake_at(hg, k);
                let mut rgb = LAKE_SHALLOW;
                let mut alpha = 0.0f32;
                if lake > 0.0 {
                    let depth = ((f64::from(lvl) - f64::from(hg.h[k])).max(0.0) * mv) as f32;
                    rgb = mix3(LAKE_SHALLOW, LAKE_DEEP, smoothstep(0.0, 250.0, depth));
                    alpha = lake.min(1.0) * 0.92;
                }
                if let Some(cv) = cov {
                    let a = cv.alpha[q] * (1.0 - lake);
                    if a > 0.0 {
                        rgb = mix3(rgb, river_colour(cv.depth[q]), a / (a + alpha * (1.0 - a)));
                        alpha = 1.0 - (1.0 - alpha) * (1.0 - a);
                    }
                }
                out.extend_from_slice(&rgb.map(|v| v.round().clamp(0.0, 255.0) as u8));
                out.push((alpha * 255.0).round() as u8);
            }
        }
        out
    }

    // -----------------------------------------------------------------------------------------
    // One block
    // -----------------------------------------------------------------------------------------

    /// What the bake says at block (x, z) (any lap): the terrain height, the water over it, the
    /// climate there.
    pub fn sample(&self, x: i64, z: i64) -> Result<Value, String> {
        let c = i64::from(self.meta.c);
        let wx = (x + c / 2).rem_euclid(c) - c / 2;
        let (bx, bz) = (wx as f64, z as f64);
        let r = f64::from(self.meta.l3_r);
        let g = &self.elev.desc().grid;
        let at = |store: &LayerStore| -> Result<Window, String> {
            let g = &store.desc().grid;
            let r = f64::from(g.r);
            let i0 = ((bx - g.origin_block_x as f64) / r).floor() as i64 - 1;
            let j0 = ((bz - g.origin_block_z as f64) / r).floor() as i64 - 1;
            store.window(i0, j0, 4, 4)
        };
        let _ = (r, g);
        let mut h = f64::from(at(&self.elev)?.bilinear(bx, bz));
        if let (Some(amp), Some((spec, noise))) = (&self.amp, &self.noise) {
            h += f64::from(at(amp)?.bilinear(bx, bz)) * spec.noise(noise, bx, bz);
        }
        let sea = self.meta.sea_level_y;
        let mv = self.meta.m_per_block_v;
        let lake = match &self.lakes {
            Some(l) => {
                let v = at(l)?.nearest(bx, bz);
                (v.is_finite() && h < f64::from(v)).then_some(f64::from(v))
            }
            None => None,
        };
        let river = self
            .rivers
            .as_ref()
            .and_then(|r| r.rivers.sample(wx, z).hit);
        let closed = match &self.closed {
            Some(c) => at(c)?.nearest(bx, bz) == 1.0,
            None => false,
        };
        let (kind, water_y) = if let Some(l) = lake {
            ("lake", Some(l))
        } else if h < sea - 1.0 && !closed {
            ("sea", Some(sea))
        } else if let Some(hit) = &river {
            ("river", Some(hit.w))
        } else {
            ("land", None)
        };
        let (lat, lon) = self.meta.projection.lat_lon(bx, bz);
        let mut o = json!({
            "x": wx, "z": z,
            "lat": lat,
            "lon": lon,
            "terrain_y": (h * 10.0).round() / 10.0,
            "height_m": ((h - sea) * mv).round(),
            "surface": kind,
            "water_y": water_y,
        });
        if let Some(hit) = river {
            o["river"] = json!({
                "half_width": (hit.hw * 10.0).round() / 10.0,
                "depth": (hit.depth * 10.0).round() / 10.0,
                "discharge_m3s": hit.q.round(),
            });
        }
        if let Some(cm) = &self.climate {
            let s1 = cm.t_mean.stencil(bx, bz);
            let lapse = (cm.gamma * ((h - sea) * mv).max(0.0)) as f32;
            let code = cm.koppen.nearest(&s1);
            if code.is_finite() {
                o["koppen"] = json!(koppen::CODES.get(code as usize).copied());
            }
            let t = cm.t_mean.at(&s1) - lapse;
            if t.is_finite() {
                o["t_mean_c"] = json!((f64::from(t) * 10.0).round() / 10.0);
            }
            let p = cm.precip.at(&s1);
            if p.is_finite() {
                o["precip_mm"] = json!(p.round());
            }
            for (key, grid) in [("t_warm_c", &cm.t_warm), ("t_cold_c", &cm.t_cold)] {
                let t = grid.at(&s1) - lapse;
                if t.is_finite() {
                    o[key] = json!((f64::from(t) * 10.0).round() / 10.0);
                }
            }
            for (key, grid) in [
                ("aridity", &cm.aridity),
                ("herb_cover", &cm.herb),
                ("tree_cover", &cm.tree),
            ] {
                let v = grid.at(&s1);
                if v.is_finite() && h >= sea {
                    o[key] = json!((f64::from(v) * 100.0).round() / 100.0);
                }
            }
            let ice = cm.sea_ice.at(&s1);
            if ice.is_finite() && h < sea {
                o["sea_ice"] = json!((f64::from(ice) * 100.0).round() / 100.0);
            }
        }
        Ok(o)
    }
}

const RIVER_RGB: [f32; 3] = [38.0, 112.0, 214.0];
const SEA_SHALLOW: [f32; 3] = [136.0, 186.0, 226.0];
const SEA_DEEP: [f32; 3] = [42.0, 84.0, 152.0];
const LAND_LOW: [f32; 3] = [226.0, 228.0, 216.0];
const LAND_HIGH: [f32; 3] = [170.0, 166.0, 156.0];
const LAKE_SHALLOW: [f32; 3] = [66.0, 156.0, 240.0];
const LAKE_DEEP: [f32; 3] = [22.0, 84.0, 192.0];

/// A river's colour by the depth of its channel (blocks).
fn river_colour(depth: f32) -> [f32; 3] {
    mix3(
        [52.0, 140.0, 236.0],
        [14.0, 72.0, 190.0],
        smoothstep(1.0, 12.0, depth),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::{Codec, DType, RasterRead};

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("dt-worldmap-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// The synthetic lake world (a lake of radius 40 at the origin) at circumference `c`.
    fn lake_world(c: u32, tag: &str) -> std::path::PathBuf {
        let dir = tmp(&format!("lake{c}-{tag}"));
        crate::synth::run(&crate::synth::Options {
            kind: crate::synth::Kind::LakeTest,
            strata: crate::synth::Strata::Placeholder,
            size: c,
            out: dir.clone(),
            viewer: None,
            dump_samples: None,
            codec: Codec::Lz4Block,
        })
        .unwrap();
        dir
    }

    #[test]
    fn the_pyramid_is_two_to_one_and_ends_at_a_block_a_pixel() {
        assert_eq!(max_zoom(16_384), 5);
        assert_eq!(max_zoom(32_768), 6);
        assert_eq!(max_zoom(2048), 2);
        assert_eq!(max_zoom(256), 0);
        assert_eq!(blocks_per_px(16_384, 0), 32.0);
        assert_eq!(blocks_per_px(16_384, 5), 1.0);
        assert_eq!(blocks_per_px(32_768, 6), 1.0);
        assert_eq!(pyramid_tiles(5), 2 + 8 + 32 + 128 + 512 + 2048);
        // Tiles cover the planet exactly: the first starts on the seam and the north edge, the last
        // ends on the seam's other side and the south edge.
        for z in 0..=5 {
            let first = TileGeom::new(16_384, z, 0, 0).unwrap();
            assert_eq!((first.x0, first.z0), (-8192.0, -4096.0));
            let (nx, ny) = (2u32 << z, 1u32 << z);
            let last = TileGeom::new(16_384, z, nx - 1, ny - 1).unwrap();
            assert_eq!(last.x0 + TILE as f64 * last.s, 8192.0, "zoom {z}");
            assert_eq!(last.z0 + TILE as f64 * last.s, 4096.0, "zoom {z}");
            assert!(TileGeom::new(16_384, z, nx, 0).is_none());
            assert!(TileGeom::new(16_384, z, 0, ny).is_none());
        }
        assert!(TileGeom::new(16_384, 6, 0, 0).is_none());
        for l in Layer::ALL {
            assert_eq!(Layer::parse(l.id()), Some(l));
        }
    }

    #[test]
    fn a_window_reads_what_the_raster_reader_does_across_the_seam_and_the_pole() {
        let dir = lake_world(1024, "window");
        let m = Manifest::read(&dir).unwrap();
        let path = dir.join(&m.layer("elevation").unwrap().path);
        let store = LayerStore::open(&path).unwrap();
        let loaded = TilePackReader::open(&path).unwrap().load().unwrap();
        assert_eq!(store.desc().dtype, DType::I16);
        // A window straddling the seam (columns -3 ...) and the north edge (rows -3 ...).
        let w = store.window(-3, -3, 12, 12).unwrap();
        for (bx, bz) in [
            (-512.0, -256.0),
            (-500.5, -250.25),
            (-511.0, -255.0),
            (-509.75, -254.5),
        ] {
            let a = f64::from(w.bilinear(bx, bz));
            let b = loaded.bilinear(bx, bz);
            assert!((a - b).abs() < 1e-3, "({bx}, {bz}): {a} {b}");
        }
        // The east edge reads the seam's cells.
        let e = store.window(508, 100, 10, 4).unwrap();
        let (a, b) = (
            f64::from(e.bilinear(510.0, -56.0)),
            loaded.bilinear(510.0, -56.0),
        );
        assert!((a - b).abs() < 1e-3, "{a} {b}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mips_are_box_means_placed_on_the_middle_of_their_cells() {
        let dir = lake_world(1024, "mips");
        let m = Manifest::read(&dir).unwrap();
        let path = dir.join(&m.layer("elevation").unwrap().path);
        let store = LayerStore::open(&path).unwrap();
        let loaded = TilePackReader::open(&path).unwrap().load().unwrap();
        let mips = build_mips(&store, 2, 4, &[|v| (v, 1.0)])
            .unwrap()
            .pop()
            .unwrap();
        for (m, grid) in mips.levels.iter().enumerate() {
            let level = 2 + m as u32;
            let n = 1i64 << level;
            assert_eq!(grid.w, 512 >> level);
            assert_eq!(grid.r, 2.0 * n as f64);
            // Cell (5, 3) is the mean of its n x n L3 cells.
            let (ci, cj) = (5i64, 3i64);
            let mut sum = 0.0;
            for j in 0..n {
                for i in 0..n {
                    sum += loaded.get(ci * n + i, cj * n + j);
                }
            }
            let want = (sum / (n * n) as f64) as f32;
            let got = grid.data[(cj as usize) * grid.w + ci as usize];
            assert!((want - got).abs() < 1e-3, "level {level}: {want} {got}");
            // Its point is at the middle of the cells it covers.
            let x = grid.ox + ci as f64 * grid.r;
            assert_eq!(x, -512.0 + 2.0 * (ci * n) as f64 + (n - 1) as f64);
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_synthetic_world_draws_every_layer_at_every_zoom_with_its_lake_and_wraps() {
        let dir = lake_world(2048, "draw");
        let map = BakeMap::open(&dir).unwrap();
        assert_eq!(map.meta.max_zoom, 2);
        assert!(map.meta.lakes && !map.meta.climate && map.rivers.is_none());
        for layer in Layer::ALL {
            for z in 0..=2u32 {
                for (tx, ty) in [(0u32, 0u32), ((2 << z) - 1, (1 << z) - 1), (1, 0)] {
                    let g = TileGeom::new(2048, z, tx, ty).unwrap();
                    let a = map.render(layer, &g).unwrap();
                    assert_eq!(a.len(), TILE * TILE * 4, "{layer:?} {z}/{tx}/{ty}");
                    assert_eq!(a, map.render(layer, &g).unwrap(), "deterministic");
                    if layer.opaque() {
                        assert!(a.as_chunks::<4>().0.iter().all(|p| p[3] == 255));
                    }
                }
            }
        }
        // Zoom 2 is a block a pixel: tile 4/2 holds the lake's south-east quarter at its corner.
        let g = TileGeom::new(2048, 2, 4, 2).unwrap();
        assert_eq!((g.x0, g.z0), (0.0, 0.0));
        let water = map.render(Layer::Water, &g).unwrap();
        let px = |i: usize, j: usize| {
            let k = (j * TILE + i) * 4;
            [water[k], water[k + 1], water[k + 2]]
        };
        let lake = px(10, 10);
        let land = px(200, 200);
        assert!(lake[2] > lake[0] + 60, "the lake is blue: {lake:?}");
        assert!(land[2] <= land[0] + 20, "the plain is not: {land:?}");
        let overlay = map.render(Layer::Overlay, &g).unwrap();
        assert!(overlay[(10 * TILE + 10) * 4 + 3] > 200, "lake alpha");
        assert_eq!(overlay[(200 * TILE + 200) * 4 + 3], 0, "dry land is clear");
        // The world wraps: a block one lap east answers as the block itself.
        let a = map.sample(10, 10).unwrap();
        let b = map.sample(10 + 2048, 10).unwrap();
        assert_eq!(a["surface"], "lake");
        assert_eq!(a["terrain_y"], b["terrain_y"]);
        assert_eq!(map.sample(300, 300).unwrap()["surface"], "land");
        // The closed bowls below sea level (HYD-100): dry ground and a lake at y 50, no sea.
        assert_eq!(map.sample(-200, -70).unwrap()["surface"], "land");
        let low = map.sample(-200, 70).unwrap();
        assert_eq!(
            (&low["surface"], &low["water_y"]),
            (&json!("lake"), &json!(50.0))
        );
        // The border column of the tile west of the seam's neighbour is the next tile's first.
        let west = map.heights(&TileGeom::new(2048, 2, 3, 2).unwrap()).unwrap();
        let east = map.heights(&g).unwrap();
        for j in [0usize, 40, 100, 255] {
            let wb = west.h[(j + 1) * N + TILE + 1];
            let eb = east.h[(j + 1) * N + 1];
            assert!((wb - eb).abs() < 1e-3, "row {j}: {wb} {eb}");
        }
        // Across the seam itself: the last tile's border column is the first tile's column 0.
        let last = map.heights(&TileGeom::new(2048, 2, 7, 2).unwrap()).unwrap();
        let first = map.heights(&TileGeom::new(2048, 2, 0, 2).unwrap()).unwrap();
        for j in [0usize, 50, 255] {
            let wb = last.h[(j + 1) * N + TILE + 1];
            let eb = first.h[(j + 1) * N + 1];
            assert!((wb - eb).abs() < 1e-3, "seam row {j}: {wb} {eb}");
        }
        // The sample reads the height the tile draws.
        let s = map.sample(40, 5).unwrap()["terrain_y"].as_f64().unwrap();
        assert!(
            (s - f64::from(east.h[(5 + 1) * N + 40 + 1])).abs() < 0.11,
            "{s}"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rivers_are_drawn_where_their_channels_are_and_are_one_pixel_wide_at_least() {
        use deeptime_format::rivers::{Mouth, RiverVertex};
        let mut r = Rivers::empty(2048);
        let v = |x: f32, z: f32, hw: f32| RiverVertex {
            x,
            z,
            w: 100.0,
            hw,
            depth: 3.0,
            q: 50.0,
        };
        r.vertices = vec![v(10.0, 20.0, 1.0), v(100.0, 20.0, 1.0)];
        r.chains = vec![deeptime_format::Chain {
            first: 0,
            count: 2,
            mouth: Mouth::Sea,
            into: u32::MAX,
        }];
        r.build_index();
        let set = RiverSet::new(r, 2);
        // Zoom 2 (a block a pixel): the channel is 2 pixels wide along z = 20 for x in 10..100.
        let g = TileGeom::new(2048, 2, 4, 2).unwrap();
        let cov = set.draw(&g);
        assert!(cov.alpha[20 * TILE + 50] > 0.99);
        assert!(cov.alpha[40 * TILE + 50] == 0.0);
        assert!(cov.alpha[20 * TILE + 150] == 0.0);
        // Zoom 0 (4 blocks a pixel, a channel under a pixel wide): still a line of one pixel.
        let g0 = TileGeom::new(2048, 0, 1, 0).unwrap();
        let cov0 = set.draw(&g0);
        let row = ((20.0 - g0.z0) / g0.s) as usize;
        assert!(
            (0..TILE).any(|i| cov0.alpha[row * TILE + i] > 0.9),
            "a hairline at the lowest zoom"
        );
    }
}
