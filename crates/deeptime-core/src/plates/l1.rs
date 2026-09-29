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
//! continental crust) and `regime`, and the outputs P1-04 and later stages read: `plates/*`.

use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::geometry::{Geo, Tree, cover};
use super::kinematics::{azimuth_deg, quat, tangent, to_world};
use super::pools::ocean_elevation;
use super::state::{CONTINENTAL, OCEANIC, OROGENY_NONE, PlateState, regime};
use crate::params::ScaleParams;
use crate::sphere::Coupling;
use crate::sphere::kdtree::angle_of_chord2;

/// Colours in the `plates` ramp; the layer holds plate id · 7 mod this.
pub const PLATE_COLOURS: u32 = 20;

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

/// Samples the crust onto the coupling's grid; returns `(layer name, values)` pairs.
pub fn sample(
    state: &PlateState,
    geo: &Geo,
    coupling: &mut Coupling,
    pos: &[DVec3],
    env: &Env,
    scale: &ScaleParams,
) -> Vec<(&'static str, Vec<f32>)> {
    let samples = &state.samples;
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let vis = Tree::over(pos, |i| !samples[i].is_hidden());
    let gap_rad = 0.9 * env.norm.spacing_rad();
    let tris = geo.mesh.triangles();
    let plate_of = |v: u32| state.groups[samples[v as usize].group as usize].plate;

    let info: Vec<Info> = coupling.relink(|_, q, link| {
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
                link.tri = c.t;
                link.w = c.w.map(|x| x as f32);
                link.nearest = c.major();
                return Info {
                    mode: COVERED,
                    d1: 0.0,
                    d2: 0.0,
                    j2: u32::MAX,
                };
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

    let k = env.k;
    let t = env.t_myr;
    let mesh = &geo.mesh;
    let since = (t - state.resample_myr).max(env.dt_myr);
    let (zr, zt) = (k.ridge_elevation_m, k.trench_elevation_m);
    let edge_z = |v: u32| {
        let s = &samples[v as usize];
        if s.kind == CONTINENTAL {
            ocean_elevation(since, zr, zt, k.oceanic_damping_mm_yr)
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
            alpha * zbar + (1.0 - alpha) * ocean_elevation(age, zr, zt, k.oceanic_damping_mm_yr),
            age,
        )
    };

    let links = coupling.links();
    let elevation_m = coupling.interpolate(mesh, |v| f64::from(samples[v as usize].elevation_m));
    let uplift = coupling.interpolate(mesh, |v| f64::from(samples[v as usize].uplift_mm_yr));
    let cells = links.len();
    let per_cell: Vec<[f32; 11]> = (0..cells)
        .into_par_iter()
        .map(|c| {
            let l = links[c];
            let inf = info[c];
            let v = l.nearest;
            let s = &samples[v as usize];
            let plate = plate_of(v);
            let mut z = elevation_m[c];
            let mut up = uplift[c];
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
        })
        .collect();
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
    names
        .iter()
        .enumerate()
        .map(|(f, &name)| (name, per_cell.iter().map(|v| v[f]).collect()))
        .collect()
}
