// SPDX-License-Identifier: LGPL-3.0-or-later
//! Lower-crustal flow: thickened continental crust spreads into the crust around it through its
//! hot lower crust, whether or not an orogeny is still raising it (the continent-budget task,
//! 2026-10-19: "mountains will 'eat' the world").
//!
//! # Why
//! A collision turns continental area into thickness (`belts.rs`). The collapse (`collapse.rs`)
//! gives it back only from quiet columns and only into room thinner than the normal column within
//! 200 km, so a belt inside a continent, ringed by crust of normal thickness, keeps its root:
//! on `earthlike_quick_16k` (supercontinent seed 2, 1000 Myr) 41 % of the continental crust ends
//! thicker than 45 km and 30 % thinner than 35 km, and the collapse moves 4.7 km³/yr of the
//! about 33 km³/yr its rule would release.
//!
//! # What happens on Earth
//! In crust thicker than about 50 km the lower crust is hot and weak, and the weight of the
//! thick column over its thinner neighbours drives it out sideways in a channel: the plateau
//! grows outwards while convergence goes on (Bird 1991; Royden et al. 1997; Clark & Royden 2000,
//! the eastern margin of Tibet built by lower-crustal flow; Rey, Teyssier & Whitney 2010:
//! "Over 15 m.y., the length scale of channel-flow extrusion is 150 km rather than 1500 km").
//! In the thin-channel description the flux of lower crust per unit width is the channel's
//! thickness cubed over its viscosity times the pressure gradient, and in the isostatic limit the
//! pressure gradient follows the gradient of crustal thickness, so the thickness obeys a
//! diffusion equation whose diffusivity grows with the cube of the channel:
//!
//!   ∂T/∂t = ∇·(K ∇T),   K = K₀ · (h / h₀)³,   h = min(max(T − T_c, 0), 2 h₀),
//!
//! with T_c = `collapse_thickness_km` (no channel in thinner crust, PLT-256), h₀ = 15 km and
//! K₀ = `crust_channel_km2_myr` the diffusivity of a 15 km channel (PLT-293).
//!
//! # The rule
//! Each plate step, on the visible continental samples of one plate (none at a trench), crust
//! moves along each lattice edge from the thicker column to the thinner, at the diffusivity of
//! the thicker column's channel (the channel is fed from the thick side), by an explicit
//! conservative scheme in as many substeps as keep it stable. The rock moved is the source's
//! own mix (pure shear). Heights follow by Airy as in the collapse (PLT-281): a source sinks by
//! its isostatic change but not below the Airy height of the crust it keeps, a receiver rises by
//! its isostatic change but not above the Airy height of its new column.
//!
//! # What it is not
//! The surface stage's plateau spreading (`surface::spread`), which moves the surface above the
//! strength height; this moves crust on the plate model's samples, below any height.
// ledger: PLT-293 (the rule and K₀), PLT-294 (h₀ and the bound).

use rayon::prelude::*;

use super::Env;
use super::collapse::{iso_height, normal_thickness_m, toward};
use super::geometry::Geo;
use super::pools::isostatic_change_m;
use super::state::{CONTINENTAL, PlateState, TRENCH, pool};

/// `WorldState::globals`: crust the lower-crustal flow moved over the run, km³.
pub const MOVED_GLOBAL: &str = "plates/channel_flow_km3";

/// The channel thickness at which K₀ holds, m (Clark & Royden 2000's channel; from memory).
// ledger: PLT-294
const CHANNEL_REF_M: f64 = 15_000.0;
/// The thickest channel counted, in [`CHANNEL_REF_M`] (K ≤ 8 K₀). ledger: PLT-294
const CHANNEL_MAX: f64 = 2.0;
/// The largest share of a column's difference to its neighbours one substep may move (the
/// explicit scheme is stable below 1/2).
const STABLE: f64 = 0.4;
/// The most substeps a plate step takes; past them the step moves less than the law would.
const MAX_SUBSTEPS: usize = 64;
/// The Laplacian's weight of one lattice edge, times its length squared: 2/3 on a hexagonal
/// lattice (∇²T ≈ (2 / 3d²) Σ (T_j − T_i)).
const HEX_WEIGHT: f64 = 2.0 / 3.0;

/// What one step did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Flow {
    /// Columns whose crust changed.
    pub columns: u32,
    /// Crust moved, km² × m (Earth-normalized).
    pub moved_km2_m: f64,
    /// Substeps taken.
    pub substeps: u32,
}

/// The diffusivity of crust of thickness `t_m` (m), km²/Myr.
fn diffusivity(t_m: f64, t_c: f64, k0: f64) -> f64 {
    let h = ((t_m - t_c) / CHANNEL_REF_M).clamp(0.0, CHANNEL_MAX);
    k0 * h * h * h
}

/// Whether the lower crust under sample `i` takes part.
fn open(state: &PlateState, i: usize) -> bool {
    let s = &state.samples[i];
    s.kind == CONTINENTAL && !s.is_hidden() && s.flags & TRENCH == 0
}

/// One plate step of the lower-crustal flow on `state` (see the module docs).
pub fn step(state: &mut PlateState, geo: &Geo, env: &Env) -> Flow {
    let k = env.k;
    let k0 = k.crust_channel_km2_myr;
    if !(k0.is_finite() && k0 > 0.0) || env.dt_myr <= 0.0 {
        return Flow::default();
    }
    let t_c = (k.collapse_thickness_km * 1000.0).max(normal_thickness_m());
    let n = state.samples.len();
    let plate: Vec<u32> = (0..n).into_par_iter().map(|i| state.plate_of(i)).collect();
    let is_open: Vec<bool> = (0..n).into_par_iter().map(|i| open(state, i)).collect();
    // The columns with a channel, and the two rings of their plate's continental crust around
    // them: crust moves only on edges between two of these, so what one side loses the other
    // gains (a column two rings out gains only if its neighbour passes T_c within the step).
    let mut ring = vec![u8::MAX; n];
    let mut front: Vec<u32> = (0..n as u32)
        .into_par_iter()
        .filter(|&i| is_open[i as usize] && f64::from(state.samples[i as usize].thickness_m) > t_c)
        .collect();
    if front.is_empty() {
        return Flow::default();
    }
    for &i in &front {
        ring[i as usize] = 0;
    }
    for r in 1..=2u8 {
        let mut next = Vec::new();
        for &i in &front {
            for (j, _) in geo.edges(i) {
                let ju = j as usize;
                if ring[ju] == u8::MAX && is_open[ju] && plate[ju] == plate[i as usize] {
                    ring[ju] = r;
                    next.push(j);
                }
            }
        }
        front = next;
    }
    let active: Vec<u32> = (0..n as u32).filter(|&i| ring[i as usize] != u8::MAX).collect();
    let mut slot = vec![u32::MAX; n];
    for (a, &i) in active.iter().enumerate() {
        slot[i as usize] = a as u32;
    }
    // Each active column's edges to active columns of its plate, with the Laplacian's weight
    // (1 / km²).
    let km_per_rad = env.norm.km(1.0);
    let nbrs: Vec<Vec<(u32, f64)>> = active
        .par_iter()
        .map(|&i| {
            geo.edges(i)
                .filter(|&(j, _)| slot[j as usize] != u32::MAX)
                .map(|(j, rad)| {
                    let d = (f64::from(rad) * km_per_rad).max(1.0e-6);
                    (slot[j as usize], HEX_WEIGHT / (d * d))
                })
                .collect()
        })
        .collect();
    let mut pools: Vec<[f64; pool::COUNT]> = active
        .iter()
        .map(|&i| state.samples[i as usize].pools.map(f64::from))
        .collect();
    let before = pools.clone();
    let thick = |p: &[f64; pool::COUNT]| p.iter().map(|x| x.max(0.0)).sum::<f64>();
    // Substeps from the fastest column at the start (a column's rate only falls as it thins).
    let rate_of = |pools: &[[f64; pool::COUNT]]| -> f64 {
        nbrs.par_iter()
            .enumerate()
            .map(|(a, e)| {
                let ta = thick(&pools[a]);
                e.iter()
                    .map(|&(b, w)| diffusivity(ta.max(thick(&pools[b as usize])), t_c, k0) * w)
                    .sum::<f64>()
            })
            .reduce(|| 0.0, f64::max)
    };
    let fastest = rate_of(&pools);
    if fastest <= 0.0 {
        return Flow::default();
    }
    let substeps = ((env.dt_myr * fastest / STABLE).ceil() as usize).clamp(1, MAX_SUBSTEPS);
    let dt = (env.dt_myr / substeps as f64).min(STABLE / fastest);
    let mut moved = 0.0f64;
    for _ in 0..substeps {
        let t: Vec<f64> = pools.par_iter().map(thick).collect();
        let (next, out): (Vec<[f64; pool::COUNT]>, Vec<f64>) = nbrs
            .par_iter()
            .enumerate()
            .map(|(a, e)| {
                let mut p = pools[a];
                let ta = t[a];
                let mut given = 0.0;
                for &(b, w) in e {
                    let b = b as usize;
                    let tb = t[b];
                    let f = dt * diffusivity(ta.max(tb), t_c, k0) * w * (tb - ta);
                    if f == 0.0 {
                        continue;
                    }
                    // Gains come in the neighbour's mix, losses go in this column's own.
                    let (src, ts) = if f > 0.0 {
                        (&pools[b], tb)
                    } else {
                        given -= f;
                        (&pools[a], ta)
                    };
                    if ts <= 0.0 {
                        continue;
                    }
                    for (x, &m) in p.iter_mut().zip(src.iter()) {
                        *x += f * m / ts;
                    }
                }
                (p, given)
            })
            .unzip();
        pools = next;
        moved += out.iter().sum::<f64>();
    }
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let mut columns = 0;
    for (a, &i) in active.iter().enumerate() {
        if pools[a] == before[a] {
            continue;
        }
        let s = &mut state.samples[i as usize];
        let pools_before = s.pools;
        for (x, &v) in s.pools.iter_mut().zip(pools[a].iter()) {
            *x = v.max(0.0) as f32;
        }
        s.thickness_m = s.pools.iter().sum();
        // Isostasy (PLT-281, PLT-278), as the collapse moves its sources and receivers.
        let z = f64::from(s.elevation_m);
        let change = isostatic_change_m(&pools_before, &s.pools, env.gravity);
        s.elevation_m = toward(z, change, iso_height(&s.pools, z_0, env.gravity)) as f32;
        columns += 1;
    }
    Flow {
        columns,
        moved_km2_m: moved * env.norm.sample_area_km2(),
        substeps: substeps as u32,
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use super::*;
    use crate::plates::collapse::tests::{Fx, crust_m, planet};

    /// A belt 26 km thicker than normal inside plate 0's continent, ringed by crust of normal
    /// thickness (no room for the collapse): its crust flows out into the ring, the volume is
    /// kept, no column ends thinner than it started or thicker than the belt, plate 1 does not
    /// change, no column stands above its Airy height, and with `crust_channel_km2_myr` 0 nothing
    /// moves.
    #[test]
    fn a_thick_belt_spreads_into_the_crust_around_it_and_keeps_its_volume() {
        let fx = Fx::new();
        let belt = |p: DVec3| p.x < -0.1 && p.y > 0.88 && p.y < 0.93;
        let mut st = planet(&fx.geo, belt, 26_000.0);
        let t0: Vec<f64> = st.samples.iter().map(|s| f64::from(s.thickness_m)).collect();
        let z0: Vec<f32> = st.samples.iter().map(|s| s.elevation_m).collect();
        let crust0 = crust_m(&st);
        let top0 = t0.iter().copied().fold(f64::MIN, f64::max);
        let mut moved = 0.0;
        for n in 1..=20 {
            moved += step(&mut st, &fx.geo, &fx.env(n)).moved_km2_m;
        }
        assert!(moved > 0.0);
        assert!(
            (crust_m(&st) - crust0).abs() < 1e-6 * crust0,
            "volume kept: {} vs {crust0}",
            crust_m(&st)
        );
        let z_0 = 0.5 * (fx.k.continent_elevation_m[0] + fx.k.continent_elevation_m[1]);
        let (mut gained, mut top) = (0, f64::MIN);
        for (i, s) in st.samples.iter().enumerate() {
            let p = fx.geo.points()[i];
            let t = f64::from(s.thickness_m);
            top = top.max(t);
            assert!(t <= top0 + 1.0, "{t} above the belt's {top0}");
            if p.x >= 0.0 {
                assert_eq!(t, t0[i], "plate 1 has no belt");
                continue;
            }
            if !belt(p) {
                assert!(t >= t0[i] - 1.0, "{t} below its {}", t0[i]);
                if t > t0[i] + 100.0 {
                    gained += 1;
                    assert!(s.elevation_m > z0[i], "a receiver rises");
                }
            }
            if s.kind == CONTINENTAL {
                let z_a = iso_height(&s.pools, z_0, 9.81);
                assert!(
                    f64::from(s.elevation_m) <= z_a.max(f64::from(z0[i])) + 1.0,
                    "{} above its Airy height {z_a}",
                    s.elevation_m
                );
            }
        }
        assert!(gained > 20, "{gained} columns gained");
        assert!(top < top0 - 1000.0, "the belt thinned: {top} of {top0}");

        let mut off = Fx::new();
        off.k.crust_channel_km2_myr = 0.0;
        let mut st = planet(&off.geo, belt, 26_000.0);
        assert_eq!(step(&mut st, &off.geo, &off.env(1)), Flow::default());
    }

    #[test]
    fn no_channel_below_the_threshold_and_the_cube_above() {
        let t_c = 50_000.0;
        assert_eq!(diffusivity(49_000.0, t_c, 1500.0), 0.0);
        assert!((diffusivity(65_000.0, t_c, 1500.0) - 1500.0).abs() < 1e-9);
        assert!((diffusivity(57_500.0, t_c, 1500.0) - 1500.0 / 8.0).abs() < 1e-9);
        assert!((diffusivity(200_000.0, t_c, 1500.0) - 1500.0 * 8.0).abs() < 1e-9);
    }
}
