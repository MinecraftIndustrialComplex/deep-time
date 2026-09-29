// SPDX-License-Identifier: LGPL-3.0-or-later
//! Thermal erosion for the amplifier (plan §8.2, P1-06): rock heaped steeper than its critical
//! slope slumps downhill.
//!
//! Ported from the pass of Schott et al. (2024), "Terrain Amplification using Multi-scale
//! Erosion" (ACM TOG 43(4)), whose code is
//! `H-Schott/MultiScaleErosion` (MIT, commit 964a3d5, `data/shaders/thermal.glsl`; see NOTICE.md):
//! a cell compares itself with its neighbours, a neighbour more than `tan(angle)` below it takes
//! matter from it and one that far above gives matter to it, and the angle carries a smooth noise
//! (their `noisifiedAngle`, ×0.9…1.4) so the talus is not one clean slope everywhere.
//!
//! **What changed.** The shader moves a fixed sliver `ε·cell area` per neighbour per iteration
//! (600 to 6,000 iterations at their resolutions); here a pair moves `rate ×` its *excess* over
//! the threshold, so a few dozen iterations relax a slope to its critical angle. It is written
//! as a gather (each cell adds what its neighbours give it and subtracts what it gives), and the
//! two ends of a pair compute the same amount from the higher cell's threshold, so **matter is
//! conserved exactly** (up to f32 rounding) wherever both cells exist. The shader wraps in both
//! axes; the planet wraps in x only, so the poles' first and last rows have no northern or
//! southern neighbour, as everywhere in `surface`.
//!
//! Drama (the slider of plan §8.2, 0..=2, default 1) scales the critical slope
//! ([`critical_slope`]): `S_c · (0.25 + 0.75 · drama)`. The floor keeps drama 0 a smooth world
//! instead of a plane; drama 2 allows slopes 1.75 times the default.

use super::amplify::window::{NONE, STEP, Win, map_cells};

/// The critical slope at slider `drama` (0..=2) from the drama-1 slope `base`.
pub fn critical_slope(base: f64, drama: f64) -> f64 {
    base * (0.25 + 0.75 * drama.clamp(0.0, 2.0))
}

/// One Jacobi iteration of thermal erosion on `z` (m) into `out`.
///
/// `tmult[c]` multiplies the critical slope `slope` at cell `c` (the noise); a pair (a, b) with
/// `z_a − z_b > slope · tmult[a] · d` moves `rate · (z_a − z_b − slope · tmult[a] · d)` metres
/// from a to b. `dx` is the cell size, m. Stable for `rate ≤ 0.06`.
pub fn step(win: &Win, z: &[f32], tmult: &[f32], slope: f32, rate: f32, dx: f32, out: &mut [f32]) {
    map_cells(win, out, |i, j, o| {
        let c = j * win.w + i;
        let zc = z[c];
        let mut nb = [NONE; 8];
        win.nbrs(i, j, &mut nb);
        let mut acc = 0.0f32;
        for (k, &n) in nb.iter().enumerate() {
            if n == NONE {
                continue;
            }
            let n = n as usize;
            let d = STEP[k] * dx;
            let give = (zc - z[n]) - slope * tmult[c] * d;
            if give > 0.0 {
                acc -= rate * give;
            }
            let take = (z[n] - zc) - slope * tmult[n] * d;
            if take > 0.0 {
                acc += rate * take;
            }
        }
        *o = zc + acc;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drama_scales_the_critical_slope() {
        assert_eq!(critical_slope(0.6, 1.0), 0.6);
        assert!((critical_slope(0.6, 0.0) - 0.15).abs() < 1e-12);
        assert!((critical_slope(0.6, 2.0) - 1.05).abs() < 1e-12);
        assert_eq!(
            critical_slope(0.6, 5.0),
            critical_slope(0.6, 2.0),
            "clamped"
        );
    }

    #[test]
    fn a_spike_slumps_and_matter_is_kept() {
        let win = Win::full(16, 9);
        let n = win.cells();
        let mut z = vec![100.0f32; n];
        z[4 * 16 + 8] = 400.0;
        z[4 * 16 + 15] = 300.0; // beside the seam: its neighbours wrap
        let tm = vec![1.0f32; n];
        let before: f64 = z.iter().map(|&v| f64::from(v)).sum();
        let mut out = vec![0.0f32; n];
        for _ in 0..40 {
            step(&win, &z, &tm, 0.5, 0.05, 100.0, &mut out);
            std::mem::swap(&mut z, &mut out);
        }
        let after: f64 = z.iter().map(|&v| f64::from(v)).sum();
        assert!((before - after).abs() < 1e-2, "{before} {after}");
        assert!(
            z[4 * 16 + 8] < 250.0,
            "the spike came down: {}",
            z[4 * 16 + 8]
        );
        // Nothing is left steeper than a little over the threshold between the spike and its
        // neighbours (40 iterations relax most of the excess).
        let drop = z[4 * 16 + 8] - z[4 * 16 + 9];
        assert!(drop < 0.5 * 100.0 * 2.0, "{drop}");
        // The seam spike's mass spread to both sides of the seam.
        assert!(z[4 * 16] > 100.0 && z[4 * 16 + 14] > 100.0);
        // Flat ground never moves.
        let flat = vec![7.0f32; n];
        step(&win, &flat, &tm, 0.5, 0.05, 100.0, &mut out);
        assert_eq!(out, flat);
    }
}
