// SPDX-License-Identifier: LGPL-3.0-or-later
//! The multigrid V-cycle on a GPU (`docs/design/gpu-compute.md` §15, phase 3): the
//! preconditioner of the diffusion solve's conjugate gradients, whose `f64` vectors, operator,
//! sums and stopping rule stay on the CPU. Every level of the cycle lives on the device in `f32`,
//! as the CPU cycle keeps it on the host (`multigrid.rs`): the finest operator is uploaded once a
//! solve (east, south and diagonal, converted from the CPU's `f64`), the coarse levels are built
//! from it on the device, and each conjugate-gradient iteration uploads the residual, runs one
//! V-cycle in one submission and reads the finest correction back. The dot product the iteration
//! needs (Σ r·z in `f64`) is taken on the CPU per strip of rows, added in order, as the CPU cycle
//! takes it, so the solve stays the same at any thread count.
//!
//! The kernels do each cell's arithmetic in the CPU cycle's order, so the cycle differs from the
//! CPU's only where the device rounds differently (a fused multiply-add, a division within the
//! 2.5 ulp WGSL allows): a preconditioner need not be exact, and the conjugate gradients stop on
//! the same `f64` residual either way (`tests.rs`, `the_gpu_cycle_*`).
//!
//! What a solve keeps on the device: per level its operator (east, south, diagonal, ω/diagonal,
//! mass) and its vectors (r and two z, the sweeps alternating between them), 32 bytes a cell (28
//! on the finest, whose mass is 1), about 49.5 MiB on the full 16k planet's 1.34 M cells with the
//! coarse levels; and the read-back buffer, 4 bytes a cell (5.1 MiB). Kept for the session
//! ([`crate::compute::Ambient::kept`]) while the grid keeps its size.

use std::sync::{Arc, Mutex};

use deeptime_gpu::{Bands, Gpu, Kernel, Readback, Use, bytemuck, wgpu};
use rayon::prelude::*;

use super::super::{Operator, STRIP_ROWS, sum_zero};
use super::{COARSE_SWEEPS, COARSEST, Cycle};

/// The name the device path logs under.
pub(super) const KERNEL: &str = "surface:diffusion";

/// The uniform every kernel reads: the level its grid covers (`w`, `h`, `wrap`), the finer level
/// it is built from or restricts (`fw`, `fh`, `fwrap`, `unit`: that level's mass is 1 a cell),
/// the coarser level's width (`cw`, for the correction), ω and the coarse coupling share.
const PRELUDE: &str = r#"
struct Band { row0: u32, row1: u32, pad0: u32, pad1: u32 }
struct L {
    w: u32, h: u32, wrap: u32, unit: u32,
    fw: u32, fh: u32, fwrap: u32, cw: u32,
    omega: f32, scale: f32, pad0: u32, pad1: u32,
}
@group(1) @binding(0) var<uniform> band: Band;
"#;

/// ω / diagonal on the finest level (the coarse levels get it with their diagonal).
const WD: &str = r#"
@group(0) @binding(0) var<uniform> p: L;
@group(0) @binding(1) var<storage, read> diag: array<f32>;
@group(0) @binding(2) var<storage, read_write> wd: array<f32>;
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let j = band.row0 + id.y;
    if (id.x >= p.w || j >= band.row1) { return; }
    let c = j * p.w + id.x;
    wd[c] = p.omega / diag[c];
}
"#;

/// A coarse level's mass and face couplings from the finer level's (`Level::coarsen`).
const COARSEN: &str = r#"
@group(0) @binding(0) var<uniform> p: L;
@group(0) @binding(1) var<storage, read> feast: array<f32>;
@group(0) @binding(2) var<storage, read> fsouth: array<f32>;
@group(0) @binding(3) var<storage, read> fmass: array<f32>;
@group(0) @binding(4) var<storage, read_write> east: array<f32>;
@group(0) @binding(5) var<storage, read_write> south: array<f32>;
@group(0) @binding(6) var<storage, read_write> mass: array<f32>;
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let jc = band.row0 + id.y;
    let ic = id.x;
    if (ic >= p.w || jc >= band.row1) { return; }
    let fw = p.fw;
    let j0 = 2u * jc;
    let j1 = min(2u * jc + 2u, p.fh);
    let i0 = 2u * ic;
    let i1 = min(2u * ic + 2u, fw);
    var m = 0.0;
    for (var j = j0; j < j1; j = j + 1u) {
        for (var i = i0; i < i1; i = i + 1u) {
            if (p.unit == 1u) {
                m = m + 1.0;
            } else {
                m = m + fmass[j * fw + i];
            }
        }
    }
    let c = jc * p.w + ic;
    mass[c] = m;
    var e = 0.0;
    if (ic + 1u < p.w || p.wrap == 1u) {
        for (var j = j0; j < j1; j = j + 1u) {
            e = e + feast[j * fw + i1 - 1u];
        }
        e = p.scale * e;
    }
    east[c] = e;
    var s = 0.0;
    if (jc + 1u < p.h) {
        for (var i = i0; i < i1; i = i + 1u) {
            s = s + fsouth[(j1 - 1u) * fw + i];
        }
        s = p.scale * s;
    }
    south[c] = s;
}
"#;

/// A coarse level's diagonal (mass + east + west + south + north) and ω / diagonal.
const CDIAG: &str = r#"
@group(0) @binding(0) var<uniform> p: L;
@group(0) @binding(1) var<storage, read> east: array<f32>;
@group(0) @binding(2) var<storage, read> south: array<f32>;
@group(0) @binding(3) var<storage, read> mass: array<f32>;
@group(0) @binding(4) var<storage, read_write> diag: array<f32>;
@group(0) @binding(5) var<storage, read_write> wd: array<f32>;
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let j = band.row0 + id.y;
    let i = id.x;
    if (i >= p.w || j >= band.row1) { return; }
    let w = p.w;
    let c = j * w + i;
    var west = 0.0;
    if (i > 0u) {
        west = east[c - 1u];
    } else if (p.wrap == 1u) {
        west = east[c + w - 1u];
    }
    var north = 0.0;
    if (j > 0u) {
        north = south[c - w];
    }
    let g = mass[c] + east[c] + west + south[c] + north;
    diag[c] = g;
    wd[c] = p.omega / g;
}
"#;

/// The first sweep from z = 0: z = ω D⁻¹ r.
const FIRST: &str = r#"
@group(0) @binding(0) var<uniform> p: L;
@group(0) @binding(1) var<storage, read> r: array<f32>;
@group(0) @binding(2) var<storage, read> wd: array<f32>;
@group(0) @binding(3) var<storage, read_write> z: array<f32>;
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let j = band.row0 + id.y;
    if (id.x >= p.w || j >= band.row1) { return; }
    let c = j * p.w + id.x;
    z[c] = wd[c] * r[c];
}
"#;

/// One damped Jacobi sweep, zout = z' + ω D⁻¹ (r − A z'), with z' = zin (and, in the coarse
/// variant, plus the coarser level's correction of the cell's block). `XAT` is filled in.
const SWEEP: &str = r#"
@group(0) @binding(0) var<uniform> p: L;
@group(0) @binding(1) var<storage, read> r: array<f32>;
@group(0) @binding(2) var<storage, read> wd: array<f32>;
@group(0) @binding(3) var<storage, read> diag: array<f32>;
@group(0) @binding(4) var<storage, read> east: array<f32>;
@group(0) @binding(5) var<storage, read> south: array<f32>;
@group(0) @binding(6) var<storage, read> zin: array<f32>;
XAT
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let j = band.row0 + id.y;
    let i = id.x;
    if (i >= p.w || j >= band.row1) { return; }
    let w = p.w;
    let c = j * w + i;
    let c0 = j * w;
    let x = x_at(j, i);
    var o = diag[c] * x;
    if (i + 1u < w) { o = o - east[c] * x_at(j, i + 1u); }
    if (i > 0u) { o = o - east[c - 1u] * x_at(j, i - 1u); }
    if (p.wrap == 1u) {
        if (i == w - 1u) { o = o - east[c] * x_at(j, 0u); }
        if (i == 0u) { o = o - east[c0 + w - 1u] * x_at(j, w - 1u); }
    }
    if (j + 1u < p.h) { o = o - south[c] * x_at(j + 1u, i); }
    if (j > 0u) { o = o - south[c - w] * x_at(j - 1u, i); }
    zout[c] = x + wd[c] * (r[c] - o);
}
"#;

const XAT_PLAIN: &str = r#"
@group(0) @binding(7) var<storage, read_write> zout: array<f32>;
fn x_at(j: u32, i: u32) -> f32 {
    return zin[j * p.w + i];
}
"#;

const XAT_COARSE: &str = r#"
@group(0) @binding(7) var<storage, read> zc: array<f32>;
@group(0) @binding(8) var<storage, read_write> zout: array<f32>;
fn x_at(j: u32, i: u32) -> f32 {
    return zin[j * p.w + i] + zc[(j / 2u) * p.cw + i / 2u];
}
"#;

/// The coarse level's right-hand side: the finer level's residual r − A z summed over each block.
const RESTRICT: &str = r#"
@group(0) @binding(0) var<uniform> p: L;
@group(0) @binding(1) var<storage, read> fr: array<f32>;
@group(0) @binding(2) var<storage, read> fdiag: array<f32>;
@group(0) @binding(3) var<storage, read> feast: array<f32>;
@group(0) @binding(4) var<storage, read> fsouth: array<f32>;
@group(0) @binding(5) var<storage, read> fz: array<f32>;
@group(0) @binding(6) var<storage, read_write> rc: array<f32>;
fn az(j: u32, i: u32) -> f32 {
    let w = p.fw;
    let c = j * w + i;
    let c0 = j * w;
    var o = fdiag[c] * fz[c];
    if (i + 1u < w) { o = o - feast[c] * fz[c + 1u]; }
    if (i > 0u) { o = o - feast[c - 1u] * fz[c - 1u]; }
    if (p.fwrap == 1u) {
        if (i == w - 1u) { o = o - feast[c] * fz[c0]; }
        if (i == 0u) { o = o - feast[c0 + w - 1u] * fz[c0 + w - 1u]; }
    }
    if (j + 1u < p.fh) { o = o - fsouth[c] * fz[c + w]; }
    if (j > 0u) { o = o - fsouth[c - w] * fz[c - w]; }
    return o;
}
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let jc = band.row0 + id.y;
    let ic = id.x;
    if (ic >= p.w || jc >= band.row1) { return; }
    let j0 = 2u * jc;
    let j1 = min(2u * jc + 2u, p.fh);
    let i0 = 2u * ic;
    let i1 = min(2u * ic + 2u, p.fw);
    var acc = 0.0;
    for (var j = j0; j < j1; j = j + 1u) {
        for (var i = i0; i < i1; i = i + 1u) {
            acc = acc + (fr[j * p.fw + i] - az(j, i));
        }
    }
    rc[jc * p.w + ic] = acc;
}
"#;

struct Kernels {
    wd: Kernel,
    coarsen: Kernel,
    cdiag: Kernel,
    first: Kernel,
    sweep: Kernel,
    sweep_c: Kernel,
    restrict: Kernel,
}

impl Kernels {
    fn new(gpu: &Gpu) -> Result<Kernels, String> {
        use Use::{Read, Uniform, Write};
        let k = |label: &str, body: &str, uses: &[Use]| -> Result<Kernel, String> {
            let m = gpu.module(label, &format!("{PRELUDE}{body}"))?;
            gpu.kernel(&m, "main", uses)
        };
        let sweep = |xat: &str| SWEEP.replace("XAT", xat);
        Ok(Kernels {
            wd: k("multigrid wd", WD, &[Uniform, Read, Write])?,
            coarsen: k(
                "multigrid coarsen",
                COARSEN,
                &[Uniform, Read, Read, Read, Write, Write, Write],
            )?,
            cdiag: k(
                "multigrid diagonal",
                CDIAG,
                &[Uniform, Read, Read, Read, Write, Write],
            )?,
            first: k(
                "multigrid first sweep",
                FIRST,
                &[Uniform, Read, Read, Write],
            )?,
            sweep: k(
                "multigrid sweep",
                &sweep(XAT_PLAIN),
                &[Uniform, Read, Read, Read, Read, Read, Read, Write],
            )?,
            sweep_c: k(
                "multigrid corrected sweep",
                &sweep(XAT_COARSE),
                &[Uniform, Read, Read, Read, Read, Read, Read, Read, Write],
            )?,
            restrict: k(
                "multigrid restrict",
                RESTRICT,
                &[Uniform, Read, Read, Read, Read, Read, Write],
            )?,
        })
    }
}

/// One level on the device.
struct Lvl {
    w: usize,
    h: usize,
    params: wgpu::Buffer,
    east: wgpu::Buffer,
    south: wgpu::Buffer,
    diag: wgpu::Buffer,
    wd: wgpu::Buffer,
    mass: wgpu::Buffer,
    r: wgpu::Buffer,
    z: [wgpu::Buffer; 2],
    bands: Bands,
}

/// The bind groups of every dispatch a cycle can make, built with the levels.
struct Groups {
    /// The finest level's ω / diagonal.
    wd0: wgpu::BindGroup,
    /// Per coarse level (index 0 unused): its coarsening and its diagonal.
    coarsen: Vec<Option<(wgpu::BindGroup, wgpu::BindGroup)>>,
    /// Per level: the first sweep (into z[0]).
    first: Vec<wgpu::BindGroup>,
    /// Per level and source z: a plain sweep.
    sweep: Vec<[wgpu::BindGroup; 2]>,
    /// Per level (but the coarsest), source z and the coarser level's z: a corrected sweep.
    sweep_c: Vec<Option<[[wgpu::BindGroup; 2]; 2]>>,
    /// Per level (but the coarsest) and source z: the restriction into the next level.
    restrict: Vec<Option<[wgpu::BindGroup; 2]>>,
}

/// The level sizes of the CPU cycle's hierarchy (`Level::coarsen`): halve (rounding up) until a
/// level holds at most [`COARSEST`] cells or stops shrinking; a coarse level wraps when the fine
/// one does and it is more than one cell wide.
fn shapes(w: usize, h: usize, wrap: bool) -> Vec<(usize, usize, bool)> {
    let mut out = vec![(w, h, wrap)];
    loop {
        let &(w, h, wrap) = out.last().expect("a level");
        let (wc, hc) = (w.div_ceil(2), h.div_ceil(2));
        if w * h <= COARSEST || (wc == w && hc == h) {
            return out;
        }
        out.push((wc, hc, wrap && wc > 1));
    }
}

/// What stays on the device between solves, for the session: the kernels, the levels of the
/// last grid's size and their bind groups, and the read-back buffer.
struct Resident {
    gpu: Arc<Gpu>,
    k: Kernels,
    /// The grid and cycle the levels were built for: (w, h, wrap, ω bits, share bits).
    shape: Option<(usize, usize, bool, u32, u32)>,
    levels: Vec<Lvl>,
    groups: Option<Groups>,
    readback: Option<Readback>,
}

impl Resident {
    fn new(gpu: &Arc<Gpu>) -> Result<Resident, String> {
        Ok(Resident {
            gpu: gpu.clone(),
            k: Kernels::new(gpu)?,
            shape: None,
            levels: Vec::new(),
            groups: None,
            readback: None,
        })
    }

    /// Device bytes of the levels of a `w × h` grid and its read-back buffer.
    fn bytes(w: usize, h: usize, wrap: bool) -> u64 {
        let cells: usize = shapes(w, h, wrap).iter().map(|s| s.0 * s.1).sum();
        cells as u64 * 32 + (w * h) as u64 * 4
    }

    /// Builds the levels and bind groups for a `w × h` grid and the cycle `cy`, unless built.
    fn fit(&mut self, w: usize, h: usize, wrap: bool, cy: &Cycle) -> Result<(), String> {
        let key = (w, h, wrap, cy.omega.to_bits(), cy.scale.to_bits());
        if self.shape == Some(key) {
            return Ok(());
        }
        self.shape = None;
        self.groups = None;
        self.levels.clear();
        self.readback = None;
        let gpu = self.gpu.clone();
        let bytes = Self::bytes(w, h, wrap);
        if bytes > gpu.budget {
            return Err(format!(
                "the cycle's {bytes} bytes exceed the device budget of {} bytes",
                gpu.budget
            ));
        }
        let sh = shapes(w, h, wrap);
        for (l, &(lw, lh, lwrap)) in sh.iter().enumerate() {
            let n = (lw * lh) as u64 * 4;
            let (fw, fh, fwrap) = if l > 0 { sh[l - 1] } else { (0, 0, false) };
            let cw = sh.get(l + 1).map_or(0, |s| s.0);
            let words: [u32; 12] = [
                lw as u32,
                lh as u32,
                u32::from(lwrap),
                u32::from(l == 1),
                fw as u32,
                fh as u32,
                u32::from(fwrap),
                cw as u32,
                cy.omega.to_bits(),
                cy.scale.to_bits(),
                0,
                0,
            ];
            let s = |label: &str, b: u64| gpu.storage(label, b);
            self.levels.push(Lvl {
                w: lw,
                h: lh,
                params: gpu.uniform("multigrid level", &words),
                east: s("multigrid east", n)?,
                south: s("multigrid south", n)?,
                diag: s("multigrid diagonal", n)?,
                wd: s("multigrid wd", n)?,
                mass: s("multigrid mass", if l == 0 { 4 } else { n })?,
                r: s("multigrid r", n)?,
                z: [s("multigrid z0", n)?, s("multigrid z1", n)?],
                bands: gpu.bands(lw, lh),
            });
        }
        self.readback = Some(gpu.readback("multigrid z read-back", (w * h * 4) as u64)?);
        let lv = &self.levels;
        let k = &self.k;
        let last = lv.len() - 1;
        let groups = Groups {
            wd0: gpu.bind(&k.wd, &[&lv[0].params, &lv[0].diag, &lv[0].wd]),
            coarsen: (0..lv.len())
                .map(|l| {
                    (l > 0).then(|| {
                        let (f, c) = (&lv[l - 1], &lv[l]);
                        (
                            gpu.bind(
                                &k.coarsen,
                                &[
                                    &c.params, &f.east, &f.south, &f.mass, &c.east, &c.south,
                                    &c.mass,
                                ],
                            ),
                            gpu.bind(
                                &k.cdiag,
                                &[&c.params, &c.east, &c.south, &c.mass, &c.diag, &c.wd],
                            ),
                        )
                    })
                })
                .collect(),
            first: lv
                .iter()
                .map(|v| gpu.bind(&k.first, &[&v.params, &v.r, &v.wd, &v.z[0]]))
                .collect(),
            sweep: lv
                .iter()
                .map(|v| {
                    std::array::from_fn(|s| {
                        gpu.bind(
                            &k.sweep,
                            &[
                                &v.params,
                                &v.r,
                                &v.wd,
                                &v.diag,
                                &v.east,
                                &v.south,
                                &v.z[s],
                                &v.z[1 - s],
                            ],
                        )
                    })
                })
                .collect(),
            sweep_c: (0..lv.len())
                .map(|l| {
                    (l < last).then(|| {
                        let (v, c) = (&lv[l], &lv[l + 1]);
                        std::array::from_fn(|s| {
                            std::array::from_fn(|cc| {
                                gpu.bind(
                                    &k.sweep_c,
                                    &[
                                        &v.params,
                                        &v.r,
                                        &v.wd,
                                        &v.diag,
                                        &v.east,
                                        &v.south,
                                        &v.z[s],
                                        &c.z[cc],
                                        &v.z[1 - s],
                                    ],
                                )
                            })
                        })
                    })
                })
                .collect(),
            restrict: (0..lv.len())
                .map(|l| {
                    (l < last).then(|| {
                        let (f, c) = (&lv[l], &lv[l + 1]);
                        std::array::from_fn(|s| {
                            gpu.bind(
                                &k.restrict,
                                &[&c.params, &f.r, &f.diag, &f.east, &f.south, &f.z[s], &c.r],
                            )
                        })
                    })
                })
                .collect(),
        };
        self.groups = Some(groups);
        self.shape = Some(key);
        Ok(())
    }

    /// Records one V-cycle from z = 0 on level `l` into `pass`; returns which z holds the result
    /// (the CPU's `vcycle`, sweep for sweep).
    fn record(&self, pass: &mut wgpu::ComputePass<'_>, l: usize, nu: usize) -> usize {
        let g = self.groups.as_ref().expect("fitted");
        let (k, v) = (&self.k, &self.levels[l]);
        v.bands.dispatch(pass, &k.first, &g.first[l]);
        let mut cur = 0;
        if l + 1 == self.levels.len() {
            for _ in 1..COARSE_SWEEPS.max(2) {
                v.bands.dispatch(pass, &k.sweep, &g.sweep[l][cur]);
                cur = 1 - cur;
            }
            return cur;
        }
        for _ in 1..nu {
            v.bands.dispatch(pass, &k.sweep, &g.sweep[l][cur]);
            cur = 1 - cur;
        }
        let c = &self.levels[l + 1];
        c.bands.dispatch(
            pass,
            &k.restrict,
            &g.restrict[l].as_ref().expect("a coarser level")[cur],
        );
        let cc = self.record(pass, l + 1, nu);
        let sc = g.sweep_c[l].as_ref().expect("a coarser level");
        v.bands.dispatch(pass, &k.sweep_c, &sc[cur][cc]);
        cur = 1 - cur;
        for _ in 1..nu {
            v.bands.dispatch(pass, &k.sweep, &g.sweep[l][cur]);
            cur = 1 - cur;
        }
        cur
    }
}

/// The device cycle of one solve: the session's [`Resident`] (taken from its slot for the solve
/// and put back after it unless the device failed) and the host's `f32` residual and correction.
pub(super) struct DeviceCycle {
    res: Option<Resident>,
    slot: Arc<Mutex<Option<Resident>>>,
    w: usize,
    nu: usize,
    /// The finest level's residual, which the conjugate gradients write before each cycle.
    pub r32: Vec<f32>,
    /// The finest level's correction after the last cycle.
    pub z32: Vec<f32>,
}

impl Drop for DeviceCycle {
    fn drop(&mut self) {
        if let Some(r) = self.res.take() {
            let mut slot = self.slot.lock().unwrap_or_else(|p| p.into_inner());
            if slot.is_none() {
                *slot = Some(r);
            }
        }
    }
}

impl DeviceCycle {
    /// The levels of `op`'s grid on `gpu`: the finest operator uploaded, the coarse levels built
    /// on the device (submitted; the first cycle runs after them).
    pub(super) fn new(gpu: &Arc<Gpu>, op: &Operator, cy: Cycle) -> Result<DeviceCycle, String> {
        let slot = match crate::compute::ambient() {
            Some(a) => a.kept::<Resident>(KERNEL),
            None => Arc::new(Mutex::new(None)),
        };
        let taken = {
            let mut s = slot.lock().unwrap_or_else(|p| p.into_inner());
            s.take().filter(|r| Arc::ptr_eq(&r.gpu, gpu))
        };
        let res = match taken {
            Some(r) => r,
            None => Resident::new(gpu)?,
        };
        let mut dc = DeviceCycle {
            res: Some(res),
            slot,
            w: op.w,
            nu: cy.nu,
            r32: Vec::new(),
            z32: Vec::new(),
        };
        let r = dc.setup(op, &cy);
        if r.is_err() {
            dc.res = None;
        }
        r.map(|_| dc)
    }

    fn setup(&mut self, op: &Operator, cy: &Cycle) -> Result<(), String> {
        let (w, h) = (op.w, op.h);
        let n = w * h;
        let res = self.res.as_mut().expect("held");
        let gpu = res.gpu.clone();
        if gpu.is_lost() {
            return Err("the device was lost".into());
        }
        if !gpu.buffer_fits((n * 4) as u64) {
            return Err(format!(
                "a {w} x {h} level exceeds the adapter's largest buffer"
            ));
        }
        res.fit(w, h, op.wrap, cy)?;
        let lv0 = &res.levels[0];
        let mut tmp = crate::alloc::filled(n, 0.0f32);
        for (src, dst) in [
            (op.east, &lv0.east),
            (op.south, &lv0.south),
            (op.diag, &lv0.diag),
        ] {
            tmp.par_iter_mut()
                .zip(src.par_iter())
                .for_each(|(t, &s)| *t = s as f32);
            gpu.upload(dst, &tmp);
        }
        drop(tmp);
        let g = res.groups.as_ref().expect("fitted");
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            lv0.bands.dispatch(&mut pass, &res.k.wd, &g.wd0);
            for (l, v) in res.levels.iter().enumerate().skip(1) {
                let (coarsen, diag) = g.coarsen[l].as_ref().expect("a coarse level");
                v.bands.dispatch(&mut pass, &res.k.coarsen, coarsen);
                v.bands.dispatch(&mut pass, &res.k.cdiag, diag);
            }
        }
        gpu.queue.submit([enc.finish()]);
        self.r32 = crate::alloc::filled(n, 0.0f32);
        self.z32 = crate::alloc::filled(n, 0.0f32);
        Ok(())
    }

    /// z = M⁻¹ r on the device for the `r32` the caller has set; returns Σ `r64`·z, per strip of
    /// rows in order as the CPU cycle sums it. An error leaves the device's state to be dropped.
    pub(super) fn precondition(&mut self, r64: &[f64]) -> Result<f64, String> {
        let r = self.run(r64);
        if r.is_err() {
            self.res = None;
        }
        r
    }

    fn run(&mut self, r64: &[f64]) -> Result<f64, String> {
        let res = self.res.as_ref().ok_or("the device failed earlier")?;
        let gpu = &res.gpu;
        let lv0 = &res.levels[0];
        let n = lv0.w * lv0.h;
        gpu.upload(&lv0.r, &self.r32);
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        let fin = {
            let mut pass = enc.begin_compute_pass(&Default::default());
            res.record(&mut pass, 0, self.nu)
        };
        let rb = res.readback.as_ref().expect("fitted");
        let w = self.w;
        let z32 = &mut self.z32;
        gpu.submit_and_read(enc, rb, &[(&lv0.z[fin], (n * 4) as u64)], |bytes| {
            let fill = |src: &[f32], z32: &mut [f32]| -> f64 {
                z32.par_chunks_mut(STRIP_ROWS * w)
                    .zip(src.par_chunks(STRIP_ROWS * w))
                    .zip(r64.par_chunks(STRIP_ROWS * w))
                    .map(|((zd, zs), rs)| {
                        zd.copy_from_slice(zs);
                        let mut acc = sum_zero();
                        for (&a, &b) in rs.iter().zip(zd.iter()) {
                            acc += a * f64::from(b);
                        }
                        acc
                    })
                    .collect::<Vec<f64>>()
                    .iter()
                    .sum()
            };
            match bytemuck::try_cast_slice::<u8, f32>(bytes) {
                Ok(src) => fill(src, z32),
                Err(_) => {
                    let src: Vec<f32> = bytes
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|b| f32::from_le_bytes(*b))
                        .collect();
                    fill(&src, z32)
                }
            }
        })
    }
}
