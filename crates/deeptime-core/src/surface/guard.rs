// SPDX-License-Identifier: LGPL-3.0-or-later
//! The surface's divergence guard (build log 2026-10-18, the surface divergence): a run whose
//! surface has left any planet's range of heights stops with an error that says so, instead of
//! baking peaks of 10²⁹ m for hours.
//!
//! The bound is twelve times Earth's whole relief, from Everest's summit to the floor of the
//! Challenger Deep (19.8 km), measured from the sea: 237 km. Below Earth's gravity it grows as
//! g_E/g, as the height a crust of given strength holds does (ISO-021), and it does not shrink
//! above it (the plate model's own ceiling, ISO-025, does not scale with gravity). It is a stop,
//! not a cap: nothing is clamped, and a surface inside it is not touched. Real relief is a
//! twelfth of it. The plate model now and then lifts a patch by tens of km in one step, and a
//! stiff plate holds part of the root behind it up for a step or two: transients of up to 112 km
//! were measured on the 1:300 presets, before the fix and after it (build log 2026-10-18), and a
//! sound step could reach about 7 x the plate model's 30 km ceiling (the whole thickening held
//! up, 1 / (1 - k) = 7), 210 km. The bound sits above both; a diverging surface grows past it
//! within a few steps.

use rayon::prelude::*;

use crate::error::CoreError;
use crate::plates::forces::G_EARTH;

/// Earth's highest summit above the sea, m: Everest (Sagarmatha, Qomolangma), the snow surface,
/// in the International Height Reference System, as Nepal and China announced it on
/// 8 December 2020.
pub const EVEREST_M: f64 = 8848.86; // ledger: ISO-071
/// The ocean's deepest point below the sea, m: the Challenger Deep in the Mariana Trench.
pub const CHALLENGER_DEEP_M: f64 = 10_925.0; // ledger: ISO-071
/// How many times Earth's relief a cell may stand from the sea before the run stops.
pub const RELIEF_MULTIPLE: f64 = 12.0; // ledger: ISO-072 (game-choice: a margin, agent 2026-10-18)

/// The farthest a cell may stand from the sea, up or down, m, at surface gravity `g` (m/s²).
// ledger: ISO-072
pub fn bound_m(gravity_m_s2: f64) -> f64 {
    RELIEF_MULTIPLE * (EVEREST_M + CHALLENGER_DEEP_M) * (G_EARTH / gravity_m_s2).max(1.0)
}

/// `WorldState::globals`: the farthest any cell stood from the sea after a step, m (measurement:
/// how close a run came to [`bound_m`]) …
pub const WORST_GLOBAL: &str = "surface/guard_worst_m";
/// … and the steps on which the farthest cell stood more than [`NEAR_RELIEFS`] Earth reliefs from
/// the sea, one global each.
pub const NEAR_GLOBALS: [&str; 4] = [
    "surface/guard_steps_over_1_relief",
    "surface/guard_steps_over_2_reliefs",
    "surface/guard_steps_over_3_reliefs",
    "surface/guard_steps_over_6_reliefs",
];
/// The Earth reliefs (Everest to the Challenger Deep, at the planet's gravity as [`bound_m`]
/// scales it) the near counts use: a twelfth, a sixth, a quarter and a half of the bound.
// ledger: ISO-073 (game-choice: reporting bands, read by no rule; agent, 2026-10-18).
pub const NEAR_RELIEFS: [f64; 4] = [1.0, 2.0, 3.0, 6.0];

/// Stops the step at `t_myr` when a cell of `h` (m, `w` cells to a row) is not finite or stands
/// farther than [`bound_m`] from the sea; otherwise returns how far the farthest cell stands from
/// the sea, m.
pub fn check(
    w: usize,
    h: &[f64],
    sea_level_m: f64,
    gravity_m_s2: f64,
    t_myr: f64,
) -> Result<f64, CoreError> {
    let bound = bound_m(gravity_m_s2);
    let off = |z: f64| {
        if z.is_finite() {
            (z - sea_level_m).abs()
        } else {
            f64::INFINITY
        }
    };
    let Some((c, worst)) = h
        .par_iter()
        .enumerate()
        .map(|(c, &z)| (c, off(z)))
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
    else {
        return Ok(0.0);
    };
    if worst <= bound {
        return Ok(worst);
    }
    let w = w.max(1);
    let ground = if worst.is_finite() {
        format!("stands {:.0} m from the sea", h[c] - sea_level_m)
    } else {
        format!("is {}", h[c])
    };
    Err(CoreError::InvalidState(format!(
        "the surface diverged at {t_myr:.1} Myr: cell ({}, {}) {ground}, past the bound of \
         {bound:.0} m ({RELIEF_MULTIPLE} times Earth's relief from Everest's summit to the \
         Challenger Deep, at {gravity_m_s2} m/s²). A step of the surface is unstable at this \
         preset; see docs/science/flexure.md, \"The surface guard\"",
        c % w,
        c / w,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bound is twelve Earth reliefs at Earth's gravity, grows below it and holds above it; a
    /// surface inside it passes, a cell past it or not finite stops the run and is named.
    #[test]
    fn a_surface_past_any_planets_relief_stops_the_run() {
        let earth = bound_m(G_EARTH);
        assert!((earth - 237_286.3).abs() < 0.1, "{earth}");
        assert!((bound_m(0.38 * G_EARTH) - earth / 0.38).abs() < 1e-6);
        assert_eq!(bound_m(2.0 * G_EARTH), earth);
        let (w, sea) = (4, 40.0);
        let mut h = vec![
            -11_000.0, 8_849.0, 30_000.0, -5000.0, 0.0, 2000.0, 300.0, 1.0,
        ];
        assert_eq!(check(w, &h, sea, G_EARTH, -10.0).unwrap(), 29_960.0);
        h[6] = 250_000.0;
        h[5] = -240_000.0;
        let e = check(w, &h, sea, G_EARTH, -10.0).unwrap_err().to_string();
        assert!(e.contains("diverged at -10.0 Myr"), "{e}");
        assert!(e.contains("cell (2, 1) stands 249960 m"), "{e}");
        // The same height is inside the bound on a small, light planet.
        assert!(check(w, &h, sea, 0.3 * G_EARTH, -10.0).is_ok());
        h[1] = f64::NAN;
        let e = check(w, &h, sea, 0.3 * G_EARTH, 0.0)
            .unwrap_err()
            .to_string();
        assert!(e.contains("cell (1, 0) is NaN"), "{e}");
    }
}
