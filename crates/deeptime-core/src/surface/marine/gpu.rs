// SPDX-License-Identifier: LGPL-3.0-or-later
//! The submarine slope failure on a GPU (`docs/design/gpu-compute.md` §16, phase 3): the same
//! Jacobi gather as [`super::slope_failure`], every cell of every iteration on the device in
//! `f32`, with the heights, the sediment and the metres moved coming back to the CPU's `f64` once
//! a call.
//!
//! What the device holds: each cell's height above the sea level as it came (`base`, `f32`), the
//! change since (`acc`, two buffers: an iteration reads one and writes the other), its sediment
//! (`avail`, two buffers likewise), its share for the iteration (`scale`), the metres it moved
//! (`mv`, Σ|Δh|/2) and one bit for `sea`, 28 bytes a cell, and a read-back buffer of 12: about
//! 51 MiB on the full 16k planet's 1.34 M cells, kept for the session while the grid keeps its
//! size. The height a kernel
//! works with is `base + acc`: the change is kept apart from the height so a slope of a few
//! metres on a floor 4 km deep is not rounded to the floor's `f32` step at every iteration, and
//! the host adds the change to its `f64` heights.
//!
//! The CPU path stops when an iteration moves nothing; the device runs every iteration, which
//! gives the same result (an iteration that moves nothing leaves every input as it was, so the
//! next one moves nothing either) without a read-back per iteration. Each cell's share and change
//! are the CPU's expressions in the CPU's order; the device rounds in `f32`, and a pair's two ends
//! may round their sums differently, so matter is kept to `f32` round-off of the changes (the
//! tests bound it) rather than the CPU path's `f64` round-off.

use std::sync::{Arc, Mutex};

use deeptime_gpu::{Bands, Gpu, Kernel, Readback, Use, bytemuck, wgpu};
use rayon::prelude::*;

use super::super::grid::Dims;
use super::FailureParams;

/// The name the device path logs under.
pub(super) const KERNEL: &str = "surface:slope_failure";

/// In `auto` mode a grid smaller than this keeps the slope failure on the CPU, whose active set
/// makes a small grid's call cheaper than the device's fixed cost (80 dispatches, an upload and a
/// read-back). A forced `gpu` mode uses the device at any size (the tests on lavapipe).
/// game-choice: engineering threshold, not physical (gpu-phase3, 2026-10-19).
const DEVICE_MIN_CELLS: usize = 1 << 17;

/// The session's device for a call on `n` cells, if it should take it (`compute.rs`).
pub(super) fn session_device(n: usize) -> Option<Arc<Gpu>> {
    let a = crate::compute::ambient()?;
    if a.mode() == crate::compute::ComputeMode::Auto && n < DEVICE_MIN_CELLS {
        return None;
    }
    crate::compute::ambient_device(KERNEL)
}

/// Says once per session that the device failed and the CPU runs the call (counted).
pub(super) fn device_failed(e: &str) {
    let msg = format!("compute: {KERNEL} failed on the GPU, the CPU runs it ({e})");
    match crate::compute::ambient() {
        Some(a) => a.tell_once("surface:slope_failure failed", crate::log::Level::Warn, msg),
        None => eprintln!("{msg}"),
    }
    crate::compute::add("slope_failure_fallback_n", 1.0);
}

/// The grid, the rate and the critical drops to a side and a diagonal neighbour.
const PRELUDE: &str = r#"
struct Band { row0: u32, row1: u32, pad0: u32, pad1: u32 }
struct P {
    w: u32, h: u32, wrap: u32, pad0: u32,
    rate: f32, crit_side: f32, crit_diag: f32, pad1: f32,
}
@group(1) @binding(0) var<uniform> band: Band;
@group(0) @binding(0) var<uniform> p: P;

// The neighbour of cell (i, j) at offset (di, dj), or -1 (`Dims::neighbour`: x wraps when the
// grid does, rows clamp, a cell is not its own neighbour).
fn nbr(i: u32, j: u32, di: i32, dj: i32) -> i32 {
    let nj = i32(j) + dj;
    if (nj < 0 || nj >= i32(p.h)) { return -1; }
    var ni = i32(i) + di;
    if (ni < 0 || ni >= i32(p.w)) {
        if (p.wrap == 0u) { return -1; }
        ni = (ni + i32(p.w)) % i32(p.w);
    }
    let n = nj * i32(p.w) + ni;
    if (n == i32(j * p.w + i)) { return -1; }
    return n;
}

fn crit(k: u32) -> f32 {
    return select(p.crit_side, p.crit_diag, (k & 1u) == 1u);
}
"#;

/// Each cell's share of what it would give that its sediment allows (`share` in `marine.rs`).
const SHARE: &str = r#"
@group(0) @binding(1) var<storage, read> base: array<f32>;
@group(0) @binding(2) var<storage, read> sea: array<u32>;
@group(0) @binding(3) var<storage, read> acc: array<f32>;
@group(0) @binding(4) var<storage, read> avail: array<f32>;
@group(0) @binding(5) var<storage, read_write> scale: array<f32>;

fn wet(c: u32, z: f32) -> bool {
    return ((sea[c >> 5u] >> (c & 31u)) & 1u) == 1u && z < 0.0;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let z = base[c] + acc[c];
    if (!wet(c, z) || avail[c] <= 0.0) {
        scale[c] = 0.0;
        return;
    }
    // The neighbour offsets of `grid.rs` (OFFSETS): odd ones are diagonal.
    var di = array<i32, 8>(1, 1, 0, -1, -1, -1, 0, 1);
    var dj = array<i32, 8>(0, -1, -1, -1, 0, 1, 1, 1);
    var give = 0.0;
    for (var k = 0u; k < 8u; k++) {
        let m = nbr(i, j, di[k], dj[k]);
        if (m >= 0) {
            let zm = base[m] + acc[m];
            if (wet(u32(m), zm)) {
                give += p.rate * max(z - zm - crit(k), 0.0);
            }
        }
    }
    scale[c] = select(0.0, min(avail[c] / give, 1.0), give > 0.0);
}
"#;

/// Each cell's change from its pairs (`change` in `marine.rs`), applied: the height's change, the
/// sediment (as the CPU applies it, to a cell that moved) and the metres moved.
const CHANGE: &str = r#"
@group(0) @binding(1) var<storage, read> base: array<f32>;
@group(0) @binding(2) var<storage, read> sea: array<u32>;
@group(0) @binding(3) var<storage, read> acc: array<f32>;
@group(0) @binding(4) var<storage, read> avail: array<f32>;
@group(0) @binding(5) var<storage, read> scale: array<f32>;
@group(0) @binding(6) var<storage, read_write> acc_out: array<f32>;
@group(0) @binding(7) var<storage, read_write> avail_out: array<f32>;
@group(0) @binding(8) var<storage, read_write> mv: array<f32>;

fn wet(c: u32, z: f32) -> bool {
    return ((sea[c >> 5u] >> (c & 31u)) & 1u) == 1u && z < 0.0;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let z = base[c] + acc[c];
    // The neighbour offsets of `grid.rs` (OFFSETS): odd ones are diagonal.
    var di = array<i32, 8>(1, 1, 0, -1, -1, -1, 0, 1);
    var dj = array<i32, 8>(0, -1, -1, -1, 0, 1, 1, 1);
    var dz = 0.0;
    if (wet(c, z)) {
        for (var k = 0u; k < 8u; k++) {
            let m = nbr(i, j, di[k], dj[k]);
            if (m < 0) { continue; }
            let zm = base[m] + acc[m];
            if (!wet(u32(m), zm)) { continue; }
            let ck = crit(k);
            let out = max(z - zm - ck, 0.0);
            let inn = max(zm - z - ck, 0.0);
            dz += p.rate * (inn * scale[m] - out * scale[c]);
        }
    }
    acc_out[c] = acc[c] + dz;
    if (dz != 0.0) {
        avail_out[c] = max(avail[c] + dz, 0.0);
        mv[c] = mv[c] + abs(dz) * 0.5;
    } else {
        avail_out[c] = avail[c];
    }
}
"#;

struct Kernels {
    share: Kernel,
    change: Kernel,
}

impl Kernels {
    fn new(gpu: &Gpu) -> Result<Kernels, String> {
        let m = |label: &str, body: &str| gpu.module(label, &format!("{PRELUDE}{body}"));
        let (r, w, u) = (Use::Read, Use::Write, Use::Uniform);
        Ok(Kernels {
            share: gpu.kernel(
                &m("slope failure share", SHARE)?,
                "main",
                &[u, r, r, r, r, w],
            )?,
            change: gpu.kernel(
                &m("slope failure change", CHANGE)?,
                "main",
                &[u, r, r, r, r, r, w, w, w],
            )?,
        })
    }
}

/// The buffers of one grid size and their bind groups.
struct Grid {
    w: usize,
    h: usize,
    params: wgpu::Buffer,
    base: wgpu::Buffer,
    sea: wgpu::Buffer,
    acc: [wgpu::Buffer; 2],
    avail: [wgpu::Buffer; 2],
    // The share buffer (`scale`) lives only in the bind groups, which keep it alive.
    mv: wgpu::Buffer,
    bands: Bands,
    /// The share and change groups reading `acc[s]` and `avail[s]`.
    share: [wgpu::BindGroup; 2],
    change: [wgpu::BindGroup; 2],
    readback: Readback,
}

/// What stays on the device between calls, for the session.
struct Resident {
    gpu: Arc<Gpu>,
    k: Kernels,
    grid: Option<Grid>,
}

/// Device bytes for `n` cells: seven `f32` buffers and the sea's bits, and the read-back of three.
fn bytes(n: usize) -> u64 {
    n as u64 * 28 + n.div_ceil(32) as u64 * 4 + n as u64 * 12
}

impl Resident {
    fn fit(&mut self, w: usize, h: usize) -> Result<(), String> {
        if self.grid.as_ref().is_some_and(|g| g.w == w && g.h == h) {
            return Ok(());
        }
        self.grid = None;
        let gpu = self.gpu.clone();
        let n = w * h;
        let need = bytes(n);
        if need > gpu.budget {
            return Err(format!(
                "the slope failure's {need} bytes exceed the device budget of {} bytes",
                gpu.budget
            ));
        }
        let f = (n * 4) as u64;
        let s = |label: &str, b: u64| gpu.storage(label, b);
        let params = gpu.uniform("slope failure params", &[0u32; 8]);
        let base = s("slope failure base", f)?;
        let sea = s("slope failure sea", (n.div_ceil(32) * 4) as u64)?;
        let acc = [s("slope failure acc0", f)?, s("slope failure acc1", f)?];
        let avail = [s("slope failure avail0", f)?, s("slope failure avail1", f)?];
        let scale = s("slope failure scale", f)?;
        let mv = s("slope failure moved", f)?;
        let k = &self.k;
        let share = std::array::from_fn(|c| {
            gpu.bind(
                &k.share,
                &[&params, &base, &sea, &acc[c], &avail[c], &scale],
            )
        });
        let change = std::array::from_fn(|c| {
            gpu.bind(
                &k.change,
                &[
                    &params,
                    &base,
                    &sea,
                    &acc[c],
                    &avail[c],
                    &scale,
                    &acc[1 - c],
                    &avail[1 - c],
                    &mv,
                ],
            )
        });
        let readback = gpu.readback("slope failure read-back", f * 3)?;
        self.grid = Some(Grid {
            w,
            h,
            bands: gpu.bands(w, h),
            params,
            base,
            sea,
            acc,
            avail,
            mv,
            share,
            change,
            readback,
        });
        Ok(())
    }
}

/// [`super::slope_failure`] on `gpu`: the same arguments and result, `h` and `avail` written only
/// when the device finished (an error leaves them as they were, for the CPU to run the call).
pub(super) fn slope_failure(
    gpu: &Arc<Gpu>,
    d: &Dims,
    sea: &[bool],
    sea_level: f64,
    p: &FailureParams,
    h: &mut [f64],
    avail: &mut [f64],
) -> Result<f64, String> {
    let (w, hh) = (d.w, d.h);
    let n = d.cells();
    if gpu.is_lost() {
        return Err("the device was lost".into());
    }
    if !gpu.buffer_fits((n * 4) as u64) {
        return Err(format!(
            "a {w} x {hh} grid exceeds the adapter's largest buffer"
        ));
    }
    let slot = match crate::compute::ambient() {
        Some(a) => a.kept::<Resident>(KERNEL),
        None => Arc::new(Mutex::new(None)),
    };
    let taken = {
        let mut s = slot.lock().unwrap_or_else(|e| e.into_inner());
        s.take().filter(|r| Arc::ptr_eq(&r.gpu, gpu))
    };
    let mut res = match taken {
        Some(r) => r,
        None => Resident {
            gpu: gpu.clone(),
            k: Kernels::new(gpu)?,
            grid: None,
        },
    };
    let out = run(&mut res, d, sea, sea_level, p, h, avail);
    if out.is_ok() {
        let mut s = slot.lock().unwrap_or_else(|e| e.into_inner());
        if s.is_none() {
            *s = Some(res);
        }
    }
    out
}

fn run(
    res: &mut Resident,
    d: &Dims,
    sea: &[bool],
    sea_level: f64,
    p: &FailureParams,
    h: &mut [f64],
    avail: &mut [f64],
) -> Result<f64, String> {
    let (w, hh) = (d.w, d.h);
    let n = w * hh;
    res.fit(w, hh)?;
    let gpu = &res.gpu;
    let g = res.grid.as_ref().expect("fitted");
    // The uniform: the grid, the rate and S_c·d to a side and a diagonal neighbour (`crit` in
    // `marine.rs`, from the same f64 products).
    let words: [u32; 8] = [
        w as u32,
        hh as u32,
        u32::from(d.wrap_x),
        0,
        (p.rate as f32).to_bits(),
        ((p.critical_slope * d.step_len(0)) as f32).to_bits(),
        ((p.critical_slope * d.step_len(1)) as f32).to_bits(),
        0,
    ];
    gpu.upload(&g.params, &words[..]);
    // The height above the sea level; a height just below it that `f32` rounds to zero keeps
    // its sign, so a cell takes part on the device as it does on the CPU.
    let base: Vec<f32> = h
        .par_iter()
        .map(|&z| {
            let b = (z - sea_level) as f32;
            if b == 0.0 && z < sea_level {
                -f32::MIN_POSITIVE
            } else {
                b
            }
        })
        .collect();
    gpu.upload(&g.base, &base);
    drop(base);
    let bits: Vec<u32> = sea
        .par_chunks(32)
        .map(|s| {
            s.iter()
                .enumerate()
                .fold(0u32, |a, (b, &v)| a | (u32::from(v) << b))
        })
        .collect();
    gpu.upload(&g.sea, &bits);
    drop(bits);
    let av: Vec<f32> = avail.par_iter().map(|&a| a as f32).collect();
    gpu.upload(&g.avail[0], &av);
    drop(av);
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.clear_buffer(&g.acc[0], 0, None);
    enc.clear_buffer(&g.mv, 0, None);
    let mut cur = 0;
    {
        let mut pass = enc.begin_compute_pass(&Default::default());
        for _ in 0..p.iterations {
            g.bands.dispatch(&mut pass, &res.k.share, &g.share[cur]);
            g.bands.dispatch(&mut pass, &res.k.change, &g.change[cur]);
            cur = 1 - cur;
        }
    }
    let f = (n * 4) as u64;
    let copies = [(&g.acc[cur], f), (&g.avail[cur], f), (&g.mv, f)];
    gpu.submit_and_read(enc, &g.readback, &copies, |bytes| {
        let mut apply = |v: &[f32]| -> f64 {
            let (acc, rest) = v.split_at(n);
            let (av, mv) = rest.split_at(n);
            // A cell that never moved keeps its f64 sediment; one that did takes the device's.
            h.par_iter_mut()
                .zip(avail.par_iter_mut())
                .zip(acc.par_iter().zip(av.par_iter().zip(mv.par_iter())))
                .for_each(|((z, a), (&dz, (&ad, &m)))| {
                    if m != 0.0 || dz != 0.0 {
                        *z += f64::from(dz);
                        *a = f64::from(ad);
                    }
                });
            // The metres moved, summed per fixed block in order (the same at any thread count).
            mv.par_chunks(1 << 16)
                .map(|b| b.iter().map(|&m| f64::from(m)).sum::<f64>())
                .collect::<Vec<f64>>()
                .iter()
                .sum()
        };
        match bytemuck::try_cast_slice::<u8, f32>(bytes) {
            Ok(v) => apply(v),
            Err(_) => {
                let v: Vec<f32> = bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|b| f32::from_le_bytes(*b))
                    .collect();
                apply(&v)
            }
        }
    })
}
