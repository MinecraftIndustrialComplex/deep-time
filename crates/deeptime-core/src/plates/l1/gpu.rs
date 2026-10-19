// SPDX-License-Identifier: LGPL-3.0-or-later
//! The plates' L1 relink on a GPU (`docs/design/gpu-compute.md` §12 item 4): for every L1 cell,
//! the nearest visible sample (from a latitude-band bucket grid instead of the k-d tree), the walk
//! to the lattice triangle holding the cell in that sample's group's frame, the test that its three
//! corners are that group's and visible, and the barycentric weights. A cell that test settles is
//! COVERED exactly as `l1.rs`'s CPU path would find it, up to ties: two samples equally near to
//! f32 rounding (about 1e-7 rad on unit vectors, under a metre on Earth), or a cell on a triangle's
//! edge to the same rounding, which Ben's decision of 2026-10-18 allows. Every other cell (no
//! visible sample within [`RADIUS`] spacings, the first group failing, a walk that did not arrive)
//! comes back marked for the CPU, which links it with today's code, so the rare paths (six and 32
//! nearest, the gap crust) need no WGSL. The layers stay on the CPU.
//!
//! The lattice mesh (points, triangles and their neighbours) is uploaded once per lattice, i.e. per
//! resampling; the visible samples' buckets, the groups' rotations and the samples' groups once per
//! call. One dispatch over the L1 grid, in bands of rows ([`deeptime_gpu::Bands`]). The pipeline
//! and the buffers are kept for the session ([`crate::compute::Ambient::kept`]).

use std::sync::{Arc, Mutex};

use deeptime_gpu::{Bands, Gpu, Kernel, Use, wgpu};
use glam::DVec3;
use rayon::prelude::*;

use super::super::Env;
use super::super::geometry::Geo;
use super::super::kinematics::quat;
use super::super::state::PlateState;
use crate::sphere::Coupling;

/// The search radius, in sample spacings: a cell whose nearest visible sample is farther is left
/// to the CPU. A covered cell lies inside a lattice triangle of visible corners, whose nearest
/// corner is well inside one spacing on a Fibonacci lattice.
const RADIUS: f64 = 1.2;
/// Steps a walk may take before the cell is left to the CPU (a walk from the nearest sample's own
/// triangle takes one or two).
const WALK_MAX: u32 = 48;
/// The marker of a cell left to the CPU in the output's triangle slot.
const NOT_HERE: u32 = u32::MAX;
const HIDDEN_BIT: u32 = 1 << 31;

const SHADER: &str = r#"
struct Band { row0: u32, row1: u32, pad0: u32, pad1: u32 }
struct P {
    w: u32, h: u32, nbands: u32, off_nlon: u32,
    off_starts: u32, walk_max: u32, pad0: u32, pad1: u32,
    dlat: f32, radius: f32, sin_r: f32, chord2: f32,
}
@group(0) @binding(0) var<uniform> p: P;
@group(0) @binding(1) var<storage, read> pts: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> tri: array<vec4<u32>>;
@group(0) @binding(3) var<storage, read> vs: array<vec2<u32>>;
@group(0) @binding(4) var<storage, read> rots: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> buckets: array<u32>;
@group(0) @binding(6) var<storage, read> items: array<vec4<u32>>;
@group(0) @binding(7) var<storage, read> axes: array<vec2<f32>>;
@group(0) @binding(8) var<storage, read_write> outl: array<u32>;
@group(1) @binding(0) var<uniform> band: Band;

const PI: f32 = 3.14159265358979;
const TAU: f32 = 6.28318530717959;
const HALF_PI: f32 = 1.57079632679490;
const NOT_HERE: u32 = 0xffffffffu;
const HIDDEN: u32 = 0x80000000u;

fn not_here(c: u32) {
    outl[5u * c] = NOT_HERE;
}

// det(a, b, q) for the edge a → b, the same number from either triangle that shares the edge
// (lower vertex first, negated), so two neighbours never both send a walk across it.
fn det_edge(a: u32, b: u32, q: vec3<f32>) -> f32 {
    if (a < b) {
        return dot(cross(pts[a].xyz, pts[b].xyz), q);
    }
    return -dot(cross(pts[b].xyz, pts[a].xyz), q);
}

// Rotates v by the unit quaternion r (x, y, z, w).
fn rotate(r: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let t = 2.0 * cross(r.xyz, v);
    return v + r.w * t + cross(r.xyz, t);
}

@compute @workgroup_size(16, 16)
fn relink(@builtin(global_invocation_id) id: vec3<u32>) {
    let j = band.row0 + id.y;
    let i = id.x;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let row = axes[j];
    let col = axes[p.h + i];
    let q = vec3<f32>(row.y * col.y, row.y * col.x, row.x);

    // The nearest visible sample within the radius: every bucket the cap of that radius touches.
    let lat = asin(clamp(q.z, -1.0, 1.0));
    let lon = atan2(q.y, q.x);
    let r = p.radius;
    let jlo = u32(clamp(i32(floor((lat - r + HALF_PI) / p.dlat)), 0, i32(p.nbands) - 1));
    let jhi = u32(clamp(i32(floor((lat + r + HALF_PI) / p.dlat)), 0, i32(p.nbands) - 1));
    var hw = 4.0;
    if (lat + r < HALF_PI && lat - r > -HALF_PI) {
        let s = p.sin_r / cos(max(abs(lat - r), abs(lat + r)));
        if (s < 1.0) {
            hw = asin(s);
        }
    }
    var best = 1.0e30;
    var bi = NOT_HERE;
    for (var bj = jlo; bj <= jhi; bj = bj + 1u) {
        let n = buckets[p.off_nlon + bj];
        let base = buckets[bj];
        var i0 = 0;
        var cnt = i32(n);
        if (hw < PI) {
            let f = f32(n) / TAU;
            i0 = i32(floor((lon - hw + PI) * f));
            let i1 = i32(floor((lon + hw + PI) * f));
            cnt = min(i1 - i0 + 1, i32(n));
        }
        for (var k = 0; k < cnt; k = k + 1) {
            let bk = base + u32(((i0 + k) % i32(n) + i32(n)) % i32(n));
            let s0 = buckets[p.off_starts + bk];
            let s1 = buckets[p.off_starts + bk + 1u];
            for (var it = s0; it < s1; it = it + 1u) {
                let e = items[it];
                let d = q - bitcast<vec3<f32>>(e.xyz);
                let d2 = dot(d, d);
                let si = e.w;
                if (d2 < best || (d2 == best && si < bi)) {
                    best = d2;
                    bi = si;
                }
            }
        }
    }
    if (bi == NOT_HERE || best > p.chord2) {
        not_here(c);
        return;
    }

    // The walk in the nearest sample's group's frame, from a triangle of that sample.
    let g = vs[bi].y & ~HIDDEN;
    let local = rotate(rots[g], q);
    var t = vs[bi].x;
    if (t == NOT_HERE) {
        not_here(c);
        return;
    }
    var arrived = false;
    for (var walk = 0u; walk < p.walk_max; walk = walk + 1u) {
        let tv = tri[2u * t];
        let tn = tri[2u * t + 1u];
        var moved = false;
        for (var s = 0u; s < 3u; s = s + 1u) {
            let e = (walk + s) % 3u;
            let a = tv[e];
            let b = tv[(e + 1u) % 3u];
            if (det_edge(a, b, local) < 0.0) {
                t = tn[e];
                moved = true;
                break;
            }
        }
        if (!moved) {
            arrived = true;
            break;
        }
    }
    if (!arrived) {
        not_here(c);
        return;
    }
    let tv = tri[2u * t];
    for (var s = 0u; s < 3u; s = s + 1u) {
        if (vs[tv[s]].y != g) {
            // Another group's corner, or a hidden one (its bit set): the CPU tries the others.
            not_here(c);
            return;
        }
    }
    let a = pts[tv.x].xyz;
    let b = pts[tv.y].xyz;
    let cc = pts[tv.z].xyz;
    let raw = vec3<f32>(dot(local, cross(b, cc)), dot(local, cross(cc, a)), dot(local, cross(a, b)));
    let wv = max(raw, vec3<f32>(0.0));
    let sum = wv.x + wv.y + wv.z;
    if (!(sum > 0.0)) {
        not_here(c);
        return;
    }
    let w = wv / sum;
    var major = tv.x;
    var wm = w.x;
    if (w.y > wm) { major = tv.y; wm = w.y; }
    if (w.z > wm) { major = tv.z; }
    outl[5u * c] = t;
    outl[5u * c + 1u] = major;
    outl[5u * c + 2u] = bitcast<u32>(w.x);
    outl[5u * c + 3u] = bitcast<u32>(w.y);
    outl[5u * c + 4u] = bitcast<u32>(w.z);
}
"#;

/// A storage buffer kept between calls and grown when a call needs more.
struct Grow {
    label: &'static str,
    buf: Option<wgpu::Buffer>,
}

impl Grow {
    const fn new(label: &'static str) -> Grow {
        Grow { label, buf: None }
    }

    fn fit(&mut self, gpu: &Gpu, bytes: u64) -> Result<&wgpu::Buffer, String> {
        let size = bytes.max(16).next_multiple_of(16);
        if self.buf.as_ref().is_none_or(|b| b.size() < size) {
            self.buf = None;
            self.buf = Some(gpu.storage(self.label, size)?);
        }
        Ok(self.buf.as_ref().expect("set above"))
    }
}

/// What stays on the device between calls, for the session: the kernel, the lattice mesh of the
/// current resampling (80 bytes a sample), and the per-call buffers at their largest size so far
/// (about 28 bytes a sample and 20 a cell).
struct Resident {
    gpu: Arc<Gpu>,
    kernel: Kernel,
    /// The lattice the mesh buffers hold (`Geo::lattice` and the point count).
    lattice: Option<([u64; 4], usize)>,
    pts: Grow,
    tri: Grow,
    vs: Grow,
    rots: Grow,
    buckets: Grow,
    items: Grow,
    axes: Grow,
    out: Grow,
    bands: Option<((usize, usize), Bands)>,
}

impl Resident {
    fn new(gpu: &Arc<Gpu>) -> Result<Resident, String> {
        let module = gpu.module("plates l1 relink", SHADER)?;
        let mut uses = vec![Use::Uniform];
        uses.extend([Use::Read; 7]);
        uses.push(Use::Write);
        let kernel = gpu.kernel(&module, "relink", &uses)?;
        Ok(Resident {
            gpu: gpu.clone(),
            kernel,
            lattice: None,
            pts: Grow::new("l1 points"),
            tri: Grow::new("l1 triangles"),
            vs: Grow::new("l1 samples"),
            rots: Grow::new("l1 rotations"),
            buckets: Grow::new("l1 buckets"),
            items: Grow::new("l1 items"),
            axes: Grow::new("l1 axes"),
            out: Grow::new("l1 links"),
            bands: None,
        })
    }
}

/// The visible samples bucketed on latitude bands of height `dlat`, each band cut into about
/// as many longitude buckets as its circumference holds: `buckets` holds the bands' first bucket
/// (`nbands + 1`), then each band's bucket count, then each bucket's first item (`nb + 1`).
struct Grid {
    nbands: u32,
    dlat: f64,
    buckets: Vec<u32>,
    off_nlon: u32,
    off_starts: u32,
    /// Each visible sample's position (f32 bits) and index.
    items: Vec<[u32; 4]>,
}

fn bucket_grid(pos: &[DVec3], visible: impl Fn(usize) -> bool + Sync, spacing: f64) -> Grid {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};
    let nbands = (PI / spacing).ceil().max(1.0) as usize;
    let dlat = PI / nbands as f64;
    let nlon: Vec<u32> = (0..nbands)
        .map(|j| {
            let mid = -FRAC_PI_2 + (j as f64 + 0.5) * dlat;
            ((TAU * mid.cos() / dlat).floor() as u32).max(1)
        })
        .collect();
    let mut first = Vec::with_capacity(nbands + 1);
    let mut nb = 0u32;
    for &n in &nlon {
        first.push(nb);
        nb += n;
    }
    first.push(nb);
    let of = |v: DVec3| -> u32 {
        let lat = v.z.clamp(-1.0, 1.0).asin();
        let lon = v.y.atan2(v.x);
        let j =
            (((lat + FRAC_PI_2) / dlat).floor() as isize).clamp(0, nbands as isize - 1) as usize;
        let n = nlon[j];
        let i = (((lon + PI) / TAU * f64::from(n)).floor() as i64).clamp(0, i64::from(n) - 1);
        first[j] + i as u32
    };
    let key: Vec<u32> = pos
        .par_iter()
        .enumerate()
        .map(|(s, &v)| if visible(s) { of(v) } else { u32::MAX })
        .collect();
    let mut starts = vec![0u32; nb as usize + 1];
    for &k in &key {
        if k != u32::MAX {
            starts[k as usize + 1] += 1;
        }
    }
    for b in 0..nb as usize {
        starts[b + 1] += starts[b];
    }
    let mut fill = starts.clone();
    let mut items = vec![[0u32; 4]; starts[nb as usize] as usize];
    for (s, &k) in key.iter().enumerate() {
        if k != u32::MAX {
            let at = &mut fill[k as usize];
            let v = pos[s];
            items[*at as usize] = [
                (v.x as f32).to_bits(),
                (v.y as f32).to_bits(),
                (v.z as f32).to_bits(),
                s as u32,
            ];
            *at += 1;
        }
    }
    let off_nlon = first.len() as u32;
    let off_starts = off_nlon + nbands as u32;
    let mut buckets = first;
    buckets.extend_from_slice(&nlon);
    buckets.extend_from_slice(&starts);
    Grid {
        nbands: nbands as u32,
        dlat,
        buckets,
        off_nlon,
        off_starts,
        items,
    }
}

/// The covered cells' links, five words a cell in the grid's order (`super::DeviceLinks`), the
/// triangle [`NOT_HERE`] where the CPU links the cell; an error when the device failed (nothing
/// is half-applied: the caller relinks every cell on the CPU).
pub fn relink(
    gpu: &Arc<Gpu>,
    state: &PlateState,
    geo: &Geo,
    coupling: &Coupling,
    pos: &[DVec3],
    env: &Env,
) -> Result<Vec<u32>, String> {
    // Kept for the session (`compute::Ambient::kept`); a caller outside a session (a test)
    // builds the pipeline and the mesh each call.
    let kept = match crate::compute::ambient() {
        Some(a) => a.kept::<Resident>(super::KERNEL),
        None => Arc::new(Mutex::new(None)),
    };
    let mut slot = kept.lock().unwrap_or_else(|p| p.into_inner());
    if slot.as_ref().is_none_or(|r| !Arc::ptr_eq(&r.gpu, gpu)) {
        *slot = None;
        *slot = Some(Resident::new(gpu)?);
    }
    let res = slot.as_mut().expect("set above");
    let r = run(res, state, geo, coupling, pos, env);
    if r.is_err() {
        // A failed call may leave buffers half-written: start clean next time.
        *slot = None;
    }
    r
}

fn run(
    res: &mut Resident,
    state: &PlateState,
    geo: &Geo,
    coupling: &Coupling,
    pos: &[DVec3],
    env: &Env,
) -> Result<Vec<u32>, String> {
    let gpu = res.gpu.clone();
    let samples = &state.samples;
    let n = samples.len();
    let grid = coupling.grid();
    let (w, h) = (grid.w as usize, grid.h as usize);
    let cells = w * h;
    if pos.len() != n || geo.points().len() != n {
        return Err(format!(
            "{} positions and {} lattice points for {n} samples",
            pos.len(),
            geo.points().len()
        ));
    }

    // The lattice mesh, once per resampling.
    let key = (geo.lattice.map(f64::to_bits), n);
    if res.lattice != Some(key) {
        res.lattice = None;
        let pts: Vec<[f32; 4]> = geo
            .points()
            .par_iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();
        let tris = geo.mesh.triangles();
        let nbrs = geo.mesh.neighbors();
        let tri: Vec<[u32; 4]> = (0..tris.len())
            .into_par_iter()
            .flat_map_iter(|t| {
                let (a, b) = (tris[t], nbrs[t]);
                [[a[0], a[1], a[2], 0], [b[0], b[1], b[2], 0]]
            })
            .collect();
        gpu.upload(res.pts.fit(&gpu, (pts.len() * 16) as u64)?, &pts);
        gpu.upload(res.tri.fit(&gpu, (tri.len() * 16) as u64)?, &tri);
        res.lattice = Some(key);
    }

    // This call's crust: each sample's triangle and group (hidden ones marked), the groups'
    // inverse rotations, the visible samples bucketed.
    let vs: Vec<[u32; 2]> = (0..n)
        .into_par_iter()
        .map(|v| {
            let s = &samples[v];
            let t = geo.mesh.vertex_triangle(v as u32).unwrap_or(NOT_HERE);
            let g = s.group | if s.is_hidden() { HIDDEN_BIT } else { 0 };
            [t, g]
        })
        .collect();
    let rots: Vec<[f32; 4]> = state
        .groups
        .iter()
        .map(|g| {
            let r = quat(g.rot).normalize().inverse();
            [r.x as f32, r.y as f32, r.z as f32, r.w as f32]
        })
        .collect();
    let spacing = env.norm.spacing_rad();
    let bg = bucket_grid(pos, |s| !samples[s].is_hidden(), spacing);
    let [rows, cols] = coupling.axes();
    let axes: Vec<[f32; 2]> = rows
        .iter()
        .chain(cols)
        .map(|&(s, c)| [s as f32, c as f32])
        .collect();
    let radius = RADIUS * spacing;
    // The cap searched is a little wider than the one accepted, so f32 rounding of the query's
    // latitude and longitude cannot skip a bucket holding an accepted sample.
    let searched = radius * 1.01 + 1e-5;
    // `struct P` of the shader: eight u32, then four f32.
    let params: [u32; 12] = [
        w as u32,
        h as u32,
        bg.nbands,
        bg.off_nlon,
        bg.off_starts,
        WALK_MAX,
        0,
        0,
        (bg.dlat as f32).to_bits(),
        (searched as f32).to_bits(),
        (searched.sin() as f32).to_bits(),
        ((4.0 * (0.5 * radius).sin().powi(2)) as f32).to_bits(),
    ];
    gpu.upload(res.vs.fit(&gpu, (vs.len() * 8) as u64)?, &vs);
    gpu.upload(res.rots.fit(&gpu, (rots.len().max(1) * 16) as u64)?, &rots);
    gpu.upload(
        res.buckets.fit(&gpu, (bg.buckets.len() * 4) as u64)?,
        &bg.buckets,
    );
    gpu.upload(
        res.items.fit(&gpu, (bg.items.len().max(1) * 16) as u64)?,
        &bg.items,
    );
    gpu.upload(res.axes.fit(&gpu, (axes.len() * 8) as u64)?, &axes);
    let out_bytes = (cells * 5 * 4) as u64;
    res.out.fit(&gpu, out_bytes)?;
    let uniform = gpu.uniform("l1 params", &params);
    if res.bands.as_ref().is_none_or(|(d, _)| *d != (w, h)) {
        res.bands = Some(((w, h), gpu.bands(w, h)));
    }
    let bands = &res.bands.as_ref().expect("set above").1;
    let group = {
        fn b(g: &Grow) -> &wgpu::Buffer {
            g.buf.as_ref().expect("fitted above")
        }
        gpu.bind(
            &res.kernel,
            &[
                &uniform,
                b(&res.pts),
                b(&res.tri),
                b(&res.vs),
                b(&res.rots),
                b(&res.buckets),
                b(&res.items),
                b(&res.axes),
                b(&res.out),
            ],
        )
    };
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    {
        let mut pass = enc.begin_compute_pass(&Default::default());
        bands.dispatch(&mut pass, &res.kernel, &group);
    }
    gpu.queue.submit([enc.finish()]);
    let mut words = vec![0u32; cells * 5];
    gpu.download(res.out.buf.as_ref().expect("fitted above"), &mut words)?;
    Ok(words)
}

#[cfg(test)]
mod tests;
