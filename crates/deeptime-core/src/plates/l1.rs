// SPDX-License-Identifier: LGPL-3.0-or-later
//! The plate model sampled onto L1 through the coupling (plan §5.2, P1-02's [`Coupling`]).
//!
//! The coupling is built on the lattice at each resampling, when every group is unrotated. Before
//! each L1 sampling, [`Coupling::relink`] moves every cell's link onto the crust as it is now: the
//! cell's direction is located in the frames of the rigid groups of its nearest visible samples,
//! and the first group whose lattice triangle holds it (all three corners that group's, none
//! subducted) gives the triangle and barycentric weights. Cells on a plate's boundary strip take
//! their nearest visible sample; cells farther than 0.9 sample spacings from any visible sample lie
//! in a gap that opened since the last resampling and get fresh ridge crust exactly as the next
//! resampling will make it (`ridges.rs`). Every cell is relinked each time, from a k-d-tree query,
//! so the result does not depend on the history of links (a resumed run samples identically); a
//! 1024 × 512 grid takes a few milliseconds per call.
//!
//! Scalar fields interpolate barycentrically; categorical ones (plate, crust type, orogeny type,
//! regime) and the fold trend take the nearest sample. Layers (world.fields): the keyframe layers
//! `elevation` (block y), `plates` (a colour index of the plate id), `crust_age` (Myr, −1 on
//! continental crust) and `regime`, and the outputs P1-04 and later stages read: `plates/*`
//! (P1-07 added the rock pools, `plates/pool/*`, interpolated, for the basement columns).
//!
//! On a GPU (`docs/design/gpu-compute.md` §12): when the session's compute mode gives a device
//! ([`crate::compute::ambient_device`]), the relink of the cells the nearest visible sample's
//! group covers runs there ([`gpu`]); the CPU below links every other cell (boundary strips,
//! gaps, anything the device could not settle) exactly as without one, and writes the layers.

use std::sync::Arc;
use std::time::Instant;

use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::geometry::{Geo, Tree, cover};
use super::kinematics::{azimuth_deg, quat, tangent, to_world};
use super::pools::floor_at;
use super::state::{CONTINENTAL, OCEANIC, OROGENY_NONE, PlateState, regime};
use crate::params::ScaleParams;
use crate::sphere::Coupling;
use crate::sphere::kdtree::angle_of_chord2;

/// Colours in the `plates` ramp; the layer holds plate id · 7 mod this.
pub const PLATE_COLOURS: u32 = 20;

#[cfg(feature = "gpu")]
mod gpu;

/// What the relink runs on besides the CPU: a wgpu device with the `gpu` feature, nothing
/// without it.
#[cfg(feature = "gpu")]
type Device = deeptime_gpu::Gpu;
#[cfg(not(feature = "gpu"))]
type Device = std::convert::Infallible;

/// The name the device path logs under and the compute gauge's prefix.
const KERNEL: &str = "plates:write_l1";

const COVERED: u8 = 0;
const NEAREST: u8 = 1;
const GAP: u8 = 2;

#[derive(Clone, Copy)]
struct Info {
    mode: u8,
    d1: f32,
    d2: f32,
    j2: u32,
}

/// Samples the crust onto the coupling's grid; returns `(layer name, values)` pairs. The one
/// dispatch point of the GPU path: the session's device, if any, links the covered cells.
pub fn sample(
    state: &PlateState,
    geo: &Geo,
    coupling: &mut Coupling,
    pos: &[DVec3],
    env: &Env,
    scale: &ScaleParams,
) -> Vec<(&'static str, Vec<f32>)> {
    #[cfg(feature = "gpu")]
    let device = crate::compute::ambient_device(KERNEL);
    #[cfg(not(feature = "gpu"))]
    let device: Option<Arc<Device>> = None;
    sample_with(state, geo, coupling, pos, env, scale, device.as_ref())
}

/// The device's links, five words a cell in the grid's order (triangle, nearest sample, the bits
/// of the three f32 weights), the triangle [`DeviceLinks::CPU`] where the CPU links the cell:
/// 20 bytes a cell on the host while the relink runs, decoded as each cell is linked.
#[cfg_attr(not(feature = "gpu"), allow(dead_code))]
struct DeviceLinks(Vec<u32>);

#[cfg_attr(not(feature = "gpu"), allow(dead_code))]
impl DeviceLinks {
    const CPU: u32 = u32::MAX;

    fn get(&self, c: usize) -> Option<crate::sphere::CellLink> {
        let o = &self.0[5 * c..5 * c + 5];
        (o[0] != Self::CPU).then(|| crate::sphere::CellLink {
            tri: o[0],
            nearest: o[1],
            w: [o[2], o[3], o[4]].map(f32::from_bits),
        })
    }

    fn len(&self) -> usize {
        self.0.len() / 5
    }

    /// The cells left to the CPU.
    fn left(&self) -> usize {
        self.0
            .as_chunks::<5>()
            .0
            .iter()
            .filter(|o| o[0] == Self::CPU)
            .count()
    }
}

/// The covered cells' links from `device`, or None when there is no device or it failed (logged
/// once; the CPU then links every cell).
fn on_device(
    device: Option<&Arc<Device>>,
    state: &PlateState,
    geo: &Geo,
    coupling: &Coupling,
    pos: &[DVec3],
    env: &Env,
) -> Option<DeviceLinks> {
    #[cfg(feature = "gpu")]
    {
        let g = device?;
        let started = Instant::now();
        let r = gpu::relink(g, state, geo, coupling, pos, env);
        crate::compute::lap("l1_device", started);
        match r {
            Ok(words) => Some(DeviceLinks(words)),
            Err(e) => {
                let msg =
                    format!("compute: {KERNEL} failed on the GPU, the CPU links every cell ({e})");
                match crate::compute::ambient() {
                    Some(a) => a.tell_once("plates:write_l1 failed", crate::log::Level::Warn, msg),
                    None => eprintln!("{msg}"),
                }
                crate::compute::add("l1_fallback_n", 1.0);
                None
            }
        }
    }
    #[cfg(not(feature = "gpu"))]
    {
        let _ = (state, geo, coupling, pos, env);
        device.map(|d| match **d {})
    }
}

/// [`sample`] with the device given (tests force it).
pub(super) fn sample_with(
    state: &PlateState,
    geo: &Geo,
    coupling: &mut Coupling,
    pos: &[DVec3],
    env: &Env,
    scale: &ScaleParams,
    device: Option<&Arc<Device>>,
) -> Vec<(&'static str, Vec<f32>)> {
    let started = Instant::now();
    let samples = &state.samples;
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    // The visible samples' k-d tree beside the device's work (the CPU's cells need it).
    let (vis, from_device) = rayon::join(
        || Tree::over(pos, |i| !samples[i].is_hidden()),
        || on_device(device, state, geo, coupling, pos, env),
    );
    crate::compute::lap("l1_tree_and_device", started);
    let relinked = Instant::now();
    let gap_rad = 0.9 * env.norm.spacing_rad();
    let tris = geo.mesh.triangles();
    let plate_of = |v: u32| state.groups[samples[v as usize].group as usize].plate;

    let covered = |link: &mut crate::sphere::CellLink, c: super::geometry::Cover| {
        link.tri = c.t;
        link.w = c.w.map(|x| x as f32);
        link.nearest = c.major();
        Info {
            mode: COVERED,
            d1: 0.0,
            d2: 0.0,
            j2: u32::MAX,
        }
    };
    let info: Vec<Info> = coupling.relink(|c, q, link| {
        if let Some(l) = from_device.as_ref().and_then(|d| d.get(c)) {
            *link = l;
            return Info {
                mode: COVERED,
                d1: 0.0,
                d2: 0.0,
                j2: u32::MAX,
            };
        }
        // Most cells lie inside their nearest visible sample's group: the nearest sample alone
        // (one query a third the cost of six) settles them, with the same first cover the six
        // would try (P1-19). The rest take the full search below.
        let first = vis.nearest(q);
        if let Some((v, _)) = first {
            let g = samples[v as usize].group;
            if let Some(c) = cover(geo, state, g, rots[g as usize], q, v)
                && c.tri.iter().all(|&x| !samples[x as usize].is_hidden())
            {
                return covered(link, c);
            }
        }
        let nn = vis.nearest_n(q, 6);
        let Some(&(j, d2)) = nn.first() else {
            return Info {
                mode: GAP,
                d1: f32::INFINITY,
                d2: f32::INFINITY,
                j2: u32::MAX,
            };
        };
        let mut tried = [u32::MAX; 6];
        let mut nt = 0;
        // The nearest sample's group failed above (unless a tie gave the six another first).
        if first.is_some_and(|(v, _)| v == j) {
            tried[0] = samples[j as usize].group;
            nt = 1;
        }
        for &(v, _) in &nn {
            let g = samples[v as usize].group;
            if tried[..nt].contains(&g) {
                continue;
            }
            tried[nt] = g;
            nt += 1;
            if let Some(c) = cover(geo, state, g, rots[g as usize], q, v)
                && c.tri.iter().all(|&x| !samples[x as usize].is_hidden())
            {
                return covered(link, c);
            }
        }
        // Nearest visible sample only: a one-hot weight on its corner of one of its triangles.
        let t = geo.mesh.vertex_triangle(j).unwrap_or(0);
        let corner = tris[t as usize].iter().position(|&x| x == j).unwrap_or(0);
        link.tri = t;
        link.w = [0.0; 3];
        link.w[corner] = 1.0;
        link.nearest = j;
        let d1 = angle_of_chord2(d2);
        if d1 <= gap_rad {
            return Info {
                mode: NEAREST,
                d1: d1 as f32,
                d2: 0.0,
                j2: u32::MAX,
            };
        }
        let p1 = plate_of(j);
        let second = nn
            .iter()
            .find(|&&(v, _)| plate_of(v) != p1)
            .copied()
            .or_else(|| {
                vis.nearest_n(q, 32)
                    .into_iter()
                    .find(|&(v, _)| plate_of(v) != p1)
            });
        match second {
            Some((v, dv)) => Info {
                mode: GAP,
                d1: d1 as f32,
                d2: angle_of_chord2(dv) as f32,
                j2: v,
            },
            None => Info {
                mode: GAP,
                d1: d1 as f32,
                d2: f32::INFINITY,
                j2: u32::MAX,
            },
        }
    });
    crate::compute::lap("l1_relink", relinked);
    if let Some(d) = &from_device {
        crate::compute::add("l1_cpu_cells_n", d.left() as f64);
        crate::compute::add("l1_cells_n", d.len() as f64);
    }
    let layered = Instant::now();

    let k = env.k;
    let t = env.t_myr;
    let since = (t - state.resample_myr).max(env.dt_myr);
    let edge_z = |v: u32| {
        let s = &samples[v as usize];
        if s.kind == CONTINENTAL {
            floor_at(k, since)
        } else {
            f64::from(s.elevation_m)
        }
    };
    // Gap cells: crust as the next resampling will make it.
    let gap_of = |c: &Info, nearest: u32| -> (f64, f64) {
        let z1 = edge_z(nearest);
        let (alpha, zbar) = if c.j2 != u32::MAX && c.d2.is_finite() {
            let (d1, d2) = (f64::from(c.d1), f64::from(c.d2));
            let s = (d1 + d2).max(1e-12);
            (
                ((d2 - d1) / s).clamp(0.0, 1.0),
                (d2 * z1 + d1 * edge_z(c.j2)) / s,
            )
        } else {
            (1.0, z1)
        };
        let age = alpha * since;
        (
            alpha * zbar + (1.0 - alpha) * floor_at(k, age),
            age,
        )
    };

    let links = coupling.links();
    // Every interpolated field in one pass (P1-17; each field's values are those of its own
    // `Coupling::interpolate`): elevation, uplift, oxidation, heat flow, thickness, the
    // continental weight and heat flow (the bake's `meta/heat_flow_mw_m2`: interpolated over the
    // continental corners only, so a coast cell does not mix in the sea floor's age law, up to
    // 300 mW/m², from an oceanic corner of its triangle), then the rock pools.
    const ELEV: usize = 0;
    const UPLIFT: usize = 1;
    const OXID: usize = 2;
    const HEAT: usize = 3;
    const THICK: usize = 4;
    const CONT_W: usize = 5;
    const CONT_HEAT: usize = 6;
    const POOL0: usize = 7;
    let continental = |v: u32| samples[v as usize].kind == CONTINENTAL;
    const INTERP: usize = POOL0 + POOL_FIELDS.len();
    let value = |v: u32| {
        let s = &samples[v as usize];
        let cont = continental(v);
        let mut out = [0.0f64; POOL0 + POOL_FIELDS.len()];
        out[ELEV] = f64::from(s.elevation_m);
        out[UPLIFT] = f64::from(s.uplift_mm_yr);
        out[OXID] = f64::from(s.oxidation);
        out[HEAT] = f64::from(s.heat_flow_mw_m2);
        out[THICK] = f64::from(s.thickness_m);
        out[CONT_W] = if cont { 1.0 } else { 0.0 };
        out[CONT_HEAT] = if cont {
            f64::from(s.heat_flow_mw_m2)
        } else {
            0.0
        };
        for k in 0..POOL_FIELDS.len() {
            out[POOL0 + k] = f64::from(s.pools[k]);
        }
        out
    };
    // A cell's interpolated fields, as `Coupling::interpolate_each` gives them, computed where
    // the cell's layers are written (P1-19: every cell's row of them was collected first, about
    // 150 MB a step on the full 16k planet).
    let interp_at = |c: usize| -> [f64; INTERP] {
        let l = &links[c];
        let t = tris[l.tri as usize];
        let v = [value(t[0]), value(t[1]), value(t[2])];
        std::array::from_fn(|f| (0..3).map(|k| f64::from(l.w[k]) * v[k][f]).sum())
    };
    let cells = links.len();
    // The deposit engine's inputs and the surface's basement (P1-04): the nearest sample's flags,
    // top rock pool and trench values; the rest interpolated.
    let extra = |c: usize, interp: &[f64; INTERP]| -> [f32; 12] {
        let s = &samples[links[c].nearest as usize];
        if info[c].mode == GAP {
            let age = gap_of(&info[c], links[c].nearest).1;
            return [
                0.0,
                super::state::pool::MAFIC_VOLCANIC as f32,
                0.0,
                0.0,
                0.0,
                0.0,
                super::pools::ocean_heat_flow(age) as f32,
                7000.0,
                s.group as f32,
                f32::NAN,
                0.0,
                0.0,
            ];
        }
        let top = TOP_ORDER
            .iter()
            .copied()
            .find(|&k| s.pools[k] > TOP_POOL_M)
            .map_or(255.0, |k| k as f32);
        [
            f32::from(s.flags),
            top,
            interp[OXID] as f32,
            s.convergence_mm_yr,
            s.obliquity_deg,
            s.slab_age_myr,
            interp[HEAT] as f32,
            interp[THICK] as f32,
            s.group as f32,
            if s.kind == CONTINENTAL && interp[CONT_W] > 1.0e-6 {
                (interp[CONT_HEAT] / interp[CONT_W]) as f32
            } else {
                f32::NAN
            },
            s.collision_mm_yr,
            s.shortening_km,
        ]
    };
    // The rock pools, interpolated like the other thicknesses (P1-07's basement columns,
    // `structure::columns`: the nearest sample's pools changed every other L1 cell and built
    // columns of alternating crust); gap cells get the fresh ridge crust the next resampling makes.
    // The rock pools come from the corners of the nearest sample's kind only (a coast cell of
    // ocean crust does not take a continent's granite, gneiss and platform cover from a
    // continental corner; the resampling keeps them apart too, ledger: PLT-293); the loose
    // sediment from all three.
    let pools = |c: usize, interp: &[f64; INTERP]| -> [f32; POOL_FIELDS.len()] {
        if info[c].mode == GAP {
            return super::pools::oceanic_column();
        }
        let l = &links[c];
        let t = tris[l.tri as usize];
        let kind = samples[l.nearest as usize].kind;
        let mine = |kk: usize| samples[t[kk] as usize].kind == kind;
        // Each weight at least 0, so the subset never extrapolates. The weights are not negative
        // today (the CPU's barycentric weights are clamped, `sphere::hull::barycentric`, the
        // device's too, `l1/gpu.rs`, and a NEAREST link is one-hot); the clamp keeps it so.
        let w = |kk: usize| f64::from(l.w[kk]).max(0.0);
        let sw: f64 = (0..3).filter(|&kk| mine(kk)).map(w).sum();
        std::array::from_fn(|k| {
            use super::state::pool;
            let v = if k == pool::SEDIMENT || sw <= 1.0e-9 {
                interp[POOL0 + k]
            } else {
                (0..3)
                    .filter(|&kk| mine(kk))
                    .map(|kk| w(kk) * f64::from(samples[t[kk] as usize].pools[k]))
                    .sum::<f64>()
                    / sw
            };
            v.max(0.0) as f32
        })
    };
    let per_cell = |c: usize, interp: &[f64; INTERP]| -> [f32; 11] {
        let l = links[c];
        let inf = info[c];
        let v = l.nearest;
        let s = &samples[v as usize];
        let plate = plate_of(v);
        let mut z = interp[ELEV];
        let mut up = interp[UPLIFT];
        let mut kind = f64::from(s.kind);
        let mut orogeny = f64::from(s.orogeny);
        let mut orogeny_age = if s.orogeny == OROGENY_NONE {
            f64::NAN
        } else {
            (t - f64::from(s.orogeny_myr)).max(0.0)
        };
        let mut reg = f64::from(s.regime);
        let mut age = if s.kind == CONTINENTAL {
            -1.0
        } else {
            let tri = tris[l.tri as usize];
            let (mut sw, mut sa) = (0.0, 0.0);
            for kk in 0..3 {
                let x = &samples[tri[kk] as usize];
                if x.kind == OCEANIC {
                    sw += f64::from(l.w[kk]);
                    sa += f64::from(l.w[kk]) * x.age_myr(t);
                }
            }
            if sw > 0.0 { sa / sw } else { s.age_myr(t) }
        };
        if inf.mode == GAP {
            let (gz, ga) = gap_of(&inf, v);
            z = gz;
            age = ga;
            up = 0.0;
            kind = f64::from(OCEANIC);
            orogeny = 0.0;
            orogeny_age = f64::NAN;
            reg = f64::from(regime::RIDGE);
        }
        // Fold trend: the axis perpendicular to the shortening direction, in [0°, 180°).
        let rot = rots[s.group as usize];
        let fold = to_world(rot, s.fold);
        let q = rot * geo.points()[v as usize];
        let trend = if fold.length_squared() > 1e-12 {
            let axis = q.cross(tangent(q, fold));
            azimuth_deg(q, axis).rem_euclid(180.0)
        } else {
            f64::NAN
        };
        [
            (f64::from(scale.sea_level_y) + z / scale.m_per_block_v) as f32,
            ((plate * 7) % PLATE_COLOURS) as f32,
            age as f32,
            reg as f32,
            z as f32,
            up as f32,
            kind as f32,
            orogeny as f32,
            orogeny_age as f32,
            trend as f32,
            plate as f32,
        ]
    };
    let names: [&'static str; 11] = [
        "elevation",
        "plates",
        "crust_age",
        "regime",
        "plates/elevation_m",
        "plates/uplift_mm_yr",
        "plates/crust_type",
        "plates/orogeny_type",
        "plates/orogeny_age_myr",
        "plates/fold_trend_deg",
        "plates/plate_id",
    ];
    let extra_names: [&'static str; 12] = [
        "plates/flags",
        "plates/top_pool",
        "plates/oxidation",
        "plates/convergence_mm_yr",
        "plates/obliquity_deg",
        "plates/slab_age_myr",
        "plates/heat_flow_mw_m2",
        "plates/thickness_m",
        // The rigid group of the cell's nearest sample: which piece of crust is under the cell. A
        // group keeps its number when a collision hands it to another plate, so the surface stage
        // can tell crust that changed plate from another plate's crust arriving (`surface/advect.rs`).
        "plates/group_id",
        // The heat flow of continental crust alone, NaN on oceanic crust and in gaps (above).
        "plates/continental_heat_flow_mw_m2",
        // The footprint of an active collision this step: its belt, its wedge and its foreland
        // (`plates/flags` carries FORELAND for the last): the convergence across the front, mm/yr,
        // and the collision's shortening so far, km; 0 elsewhere and in gaps. The structure stage
        // reads them (`belts.rs`, `structure::deform::emplace`).
        "plates/collision_mm_yr",
        "plates/shortening",
    ];
    // Every layer written in one pass over the cells, each block of cells on one thread (P1-19:
    // the cells' values were collected per group, then each layer drawn out of them in a pass of
    // its own, 30 passes over the L1 every step).
    const FIELDS: usize = 11 + 12 + POOL_FIELDS.len();
    const BLOCK: usize = 1 << 12;
    let mut layers: Vec<Vec<f32>> = (0..FIELDS).map(|_| vec![0.0; cells]).collect();
    let mut blocks: Vec<Vec<&mut [f32]>> = (0..cells.div_ceil(BLOCK))
        .map(|_| Vec::with_capacity(FIELDS))
        .collect();
    for layer in &mut layers {
        for (b, part) in layer.chunks_mut(BLOCK).enumerate() {
            blocks[b].push(part);
        }
    }
    blocks
        .into_par_iter()
        .enumerate()
        .for_each(|(b, mut parts)| {
            for i in 0..parts[0].len() {
                let c = b * BLOCK + i;
                let iv = interp_at(c);
                let values = per_cell(c, &iv)
                    .into_iter()
                    .chain(extra(c, &iv))
                    .chain(pools(c, &iv))
                    .enumerate();
                for (f, x) in values {
                    parts[f][i] = x;
                }
            }
        });
    crate::compute::lap("l1_layers", layered);
    names
        .into_iter()
        .chain(extra_names)
        .chain(POOL_FIELDS)
        .zip(layers)
        .collect()
}

/// The L1 fields holding the rock pools (interpolated), metres, in [`pool`] order
/// (`plates/pool/<name>_m`; P1-07 builds the basement below the strata log from them).
///
/// [`pool`]: super::state::pool
pub const POOL_FIELDS: [&str; super::state::pool::COUNT] = [
    "plates/pool/sediment_m",
    "plates/pool/sedimentary_m",
    "plates/pool/metamorphic_m",
    "plates/pool/felsic_plutonic_m",
    "plates/pool/felsic_volcanic_m",
    "plates/pool/mafic_volcanic_m",
    "plates/pool/mafic_plutonic_m",
];

/// The pools in the order erosion meets them from the top (the surface's basement rock,
/// `plates/top_pool`): the first holding more than [`TOP_POOL_M`].
pub const TOP_ORDER: [usize; 7] = [
    super::state::pool::SEDIMENT,
    super::state::pool::SEDIMENTARY,
    super::state::pool::FELSIC_VOLCANIC,
    super::state::pool::MAFIC_VOLCANIC,
    super::state::pool::METAMORPHIC,
    super::state::pool::FELSIC_PLUTONIC,
    super::state::pool::MAFIC_PLUTONIC,
];

/// A pool thinner than this does not count as the top rock, metres.
pub const TOP_POOL_M: f32 = 50.0;
