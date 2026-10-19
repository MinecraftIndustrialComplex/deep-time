// SPDX-License-Identifier: LGPL-3.0-or-later
//! Crust too thick for its strength spreads (Ben, decisions log 2026-10-01: the height limit of
//! mountains must emerge from the simulation, with its dependence on gravity).
//!
//! # The mechanism
//! A range is held up by a crustal root, and the weight of the land above its surroundings,
//! ρ_c·g·h, is a stress the crust has to carry. Where it passes the crust's strength S the hot
//! lower crust flows out from under the range, and the surface relaxes sideways: convergence then
//! widens the high ground into a plateau instead of raising it (Tibet, the Altiplano). In the
//! thin-channel description (Clark & Royden 2000) the flux of lower crust is proportional to the
//! topographic gradient and to the cube of the channel's thickness, which makes the surface obey
//! a diffusion equation,
//!
//!   ∂h/∂t = ∇·(D ∇h),   D = D₀ · min(x, X)³,   x = max(0, ρ_c·g·h / S − 1),
//!
//! with h the height above sea level. Below the strength height h_s = S/(ρ_c·g) nothing moves;
//! the channel grows with the excess over it (the cube), and D₀ (`tectonics.crust_flow_km2_myr`)
//! is the diffusivity at twice the strength height. X = [`MAX_EXCESS`] bounds D for a surface
//! that some other setting pushed far above the threshold.
//!
//! Gravity enters through ρ_c·g·h and nowhere else: h_s is 5.1 km at 1 g for S = 140 MPa, 13.4 km
//! at 0.38 g, 2.5 km at 2 g.
//!
//! # What it is not
//! A cap. Nothing is clamped: a range above h_s keeps rising while uplift outruns the flow, and
//! summits stand above the plateau by what erosion cuts and uplift maintains. The plate model's
//! own elevation guard (`tectonics.max_elevation_m`, 30 km in the pipeline presets) does not
//! take part.
//!
//! # Bookkeeping
//! The flow moves crust, not the rock at the surface: the strata log, the regolith and the
//! step's erosion and deposition are untouched, and the mass residual does not see it. The
//! diffusion is conservative (Σh is kept to round-off, `diffusion::step`), implicit and solved
//! only when some cell stands above h_s. The plate model takes the result with the feedback.
//!
//! # Sources
//! Each checked against its publisher's record on 2026-10-01 (title, year, pages). The mechanism
//! is theirs; the code is not a port of anyone's.
//! - P. England & D. McKenzie 1982, "A thin viscous sheet model for continental deformation",
//!   Geophys. J. R. Astron. Soc. 70(2):295–321 (correction 73:523, 1983): crustal thickness
//!   contrasts are limited by the gravity acting on them.
//! - P. Molnar & H. Lyon-Caen 1988, "Some simple physical aspects of the support, structure, and
//!   evolution of mountain belts", GSA Special Paper 218:179–207: the work against gravity limits
//!   a range's height, and high plateaus then grow sideways.
//! - P. Bird 1991, "Lateral extrusion of lower crust from under high topography in the isostatic
//!   limit", J. Geophys. Res. 96(B6):10275–10286: lower crust flows out from under high ground.
//! - L. H. Royden et al. 1997, "Surface deformation and lower crustal flow in eastern Tibet",
//!   Science 276:788–790, and M. K. Clark & L. H. Royden 2000, "Topographic ooze: Building the
//!   eastern margin of Tibet by lower crustal flow", Geology 28(8):703–706: flow in a lower-crustal
//!   channel; the diffusion form with the cube of the channel thickness.
//! - P. Rey, O. Vanderhaeghe & C. Teyssier 2001, "Gravitational collapse of the continental crust:
//!   definition, regimes and modes", Tectonophysics 342(3–4):435–449: collapse of thickened crust.

use rayon::prelude::*;

use super::diffusion;
use super::grid::Dims;

/// The most the excess over the strength height counts for (D ≤ D₀·MAX_EXCESS³).
pub const MAX_EXCESS: f64 = 2.0;

/// `WorldState::globals`: cell-steps above the strength height, and the volume moved, km³.
pub const CELLS_GLOBAL: &str = "surface/spread_cell_steps";
pub const MOVED_GLOBAL: &str = "surface/spread_km3";

/// What one step did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spread {
    /// Cells above the strength height before the step.
    pub cells: u64,
    /// Σ of the rises (= Σ of the falls), m: × the cell area, the volume moved.
    pub moved_m: f64,
}

/// The diffusivity of the surface at height `z` above the sea, m²/yr.
pub fn diffusivity(z_above_sea: f64, strength_height_m: f64, d0_km2_myr: f64) -> f64 {
    if z_above_sea > strength_height_m && strength_height_m > 0.0 {
        let x = (z_above_sea / strength_height_m - 1.0).min(MAX_EXCESS);
        // km²/Myr = m²/yr.
        d0_km2_myr * x * x * x
    } else {
        0.0
    }
}

/// One step of the spreading over `dt_yr` on elevations `h` (m, datum of the plate model).
pub fn step(
    d: &Dims,
    h: &mut [f64],
    sea: f64,
    strength_height_m: f64,
    d0_km2_myr: f64,
    dt_yr: f64,
) -> Spread {
    step_in(
        &mut diffusion::Workspace::default(),
        d,
        h,
        sea,
        strength_height_m,
        d0_km2_myr,
        dt_yr,
    )
}

/// [`step`] with the solve's vectors in a workspace the caller keeps (P1-19).
pub fn step_in(
    ws: &mut diffusion::Workspace,
    d: &Dims,
    h: &mut [f64],
    sea: f64,
    strength_height_m: f64,
    d0_km2_myr: f64,
    dt_yr: f64,
) -> Spread {
    if d0_km2_myr.is_nan() || d0_km2_myr <= 0.0 {
        return Spread::default();
    }
    let kd: Vec<f64> = h
        .par_iter()
        .map(|&z| diffusivity(z - sea, strength_height_m, d0_km2_myr))
        .collect();
    let cells = kd.par_iter().filter(|&&k| k > 0.0).count() as u64;
    if cells == 0 {
        return Spread::default();
    }
    let before = crate::alloc::copied(h);
    diffusion::step_in(ws, d, h, &kd, dt_yr);
    let moved_m = h.iter().zip(&before).map(|(a, b)| (a - b).max(0.0)).sum();
    Spread { cells, moved_m }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 9 km block on a 1 km plain, S/(ρ g) = 5 km: the block above 5 km sinks and widens, the
    /// volume is kept, nothing under the strength height far from it moves; at 0.38 g (13.2 km)
    /// the same block does not move at all; and without the flow nothing happens.
    #[test]
    fn a_range_over_its_strength_spreads_into_a_plateau() {
        let d = Dims::new(64, 32, 5000.0, true);
        let n = d.cells();
        let block = |c: usize| (28..36).contains(&(c % 64)) && (12..20).contains(&(c / 64));
        let start: Vec<f64> = (0..n)
            .map(|c| if block(c) { 9000.0 } else { 1000.0 })
            .collect();
        let mut h = start.clone();
        let mut total = Spread::default();
        for _ in 0..10 {
            let s = step(&d, &mut h, 0.0, 5000.0, 400.0, 2.0e6);
            total.cells += s.cells;
            total.moved_m += s.moved_m;
        }
        let sum = |v: &[f64]| v.iter().sum::<f64>();
        assert!(
            (sum(&h) - sum(&start)).abs() < 1e-3 * n as f64,
            "volume kept"
        );
        let top = h.iter().cloned().fold(f64::MIN, f64::max);
        assert!(
            top < 7500.0 && top > 5000.0,
            "the block sank towards the strength height: {top}"
        );
        let wide = h.iter().filter(|&&z| z > 3000.0).count();
        assert!(
            wide > 64,
            "and widened: {wide} cells above 3 km, 64 at the start"
        );
        assert_eq!(h[0], 1000.0, "the plain far away did not move");
        assert!(total.cells > 0 && total.moved_m > 0.0);
        // Lower gravity: the same strength holds a taller range.
        let mut low = start.clone();
        assert_eq!(
            step(&d, &mut low, 0.0, 13_200.0, 400.0, 2.0e6),
            Spread::default()
        );
        assert_eq!(low, start);
        // Off.
        let mut off = start.clone();
        assert_eq!(
            step(&d, &mut off, 0.0, 5000.0, 0.0, 2.0e6),
            Spread::default()
        );
        assert_eq!(off, start);
        assert_eq!(diffusivity(10_000.0, 5000.0, 400.0), 400.0);
        assert_eq!(diffusivity(7500.0, 5000.0, 400.0), 50.0);
        assert_eq!(diffusivity(5000.0, 5000.0, 400.0), 0.0);
        assert_eq!(diffusivity(1.0e9, 5000.0, 400.0), 3200.0);
    }
}
