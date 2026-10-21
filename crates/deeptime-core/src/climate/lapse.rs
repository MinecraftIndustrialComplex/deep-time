// SPDX-License-Identifier: LGPL-3.0-or-later
//! Continentality and the lapse rate on the DEM (plan §8.3 item 3).
//!
//! **Lapse rate.** Surface temperature = the sea-level temperature + the plateau's warmth − Γ·z
//! (Γ = `lapse_c_per_km`, 6.5 °C/km by default, the standard atmosphere's) on land above sea
//! level. The plateau's warmth (the mass-elevation effect, [`mass_elevation_k`] ×
//! [`mass_elevation_base`]) is how much warmer than the free air at the same height the air over
//! broad high ground is: a few kelvin in the sunlit months over a plateau hundreds of kilometres
//! wide, nothing over the sea and little over a lone peak.
//!
//! **Along the wind.** Each row is marched in the direction the air travels (east under
//! westerlies, west under easterlies; the row wraps, so the march goes round twice from a sea cell
//! and keeps the second lap). The air carries:
//!
//! - moisture q (1 = maritime): over the sea it recovers toward 1 over `ocean_recharge_km`; over
//!   land it decays toward `moisture_floor` over `moisture_decay_km` (the research note: rain
//!   reaches about 2,000 km inland from warm coasts), lengthened by (1 + `recycling` × the row's
//!   belt wetness, P_belt(φ)/2,400 mm) where the land's own evaporation returns much of the rain
//!   (rainforests: a third to a half of the Amazon's rain is recycled);
//! - maritime influence m on temperature: 1 over the sea, decaying over land over
//!   `maritime_decay_km` (west coasts under the westerlies stay mild, interiors get the EBM's land
//!   climate);
//! - the basin position and the sea-surface anomaly of the last sea it crossed (`belts.rs`), whose
//!   effect inland scales with m;
//! - the highest barrier B it crossed, which relaxes toward the ground over `barrier_recover_km`:
//!   downwind of a range the moisture is q·e^(−(B − z)/`vapour_scale_m`) — the rain shadow (the
//!   research note: rain is blocked by relief of 1,000 m or more).
//!
//! Distances are the true east–west widths of the row's cells on the planet, converted to Earth
//! kilometres (planetary-scale climate is Earth-normalized; `params.rs`).

/// What the air brings to each cell of a row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Carried {
    /// Moisture available for precipitation, 0–1, after the rain shadow.
    pub q: Vec<f64>,
    /// Maritime influence on temperature, 0–1.
    pub maritime: Vec<f64>,
    /// Basin position of the last sea crossed (`belts::basin_position`).
    pub basin: Vec<f64>,
    /// Sea-surface anomaly of the last sea crossed, °C.
    pub dsst: Vec<f64>,
}

/// Lengths of the march, in the same unit as the cell width passed to [`march`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarchParams {
    pub recharge: f64,
    pub moisture_decay: f64,
    pub floor: f64,
    pub maritime_decay: f64,
    pub vapour_scale_m: f64,
    pub barrier_recover: f64,
}

/// Marches one row. `z` is the ground above sea level (0 at sea), `basin` and `dsst` the sea's
/// values (read at sea cells), `ds` the width of one cell.
pub fn march(
    land: &[bool],
    z: &[f64],
    basin: &[f64],
    dsst: &[f64],
    ds: f64,
    eastward: bool,
    p: &MarchParams,
) -> Carried {
    let w = land.len();
    let mut out = Carried {
        q: vec![0.0; w],
        maritime: vec![0.0; w],
        basin: vec![0.0; w],
        dsst: vec![0.0; w],
    };
    if w == 0 {
        return out;
    }
    let start = land.iter().position(|&l| !l).unwrap_or(0);
    let (e_rech, e_moist, e_mar, e_bar) = (
        (-ds / p.recharge).exp(),
        (-ds / p.moisture_decay).exp(),
        (-ds / p.maritime_decay).exp(),
        (-ds / p.barrier_recover).exp(),
    );
    let (mut q, mut m, mut a, mut t, mut b) = if land[start] {
        (p.floor, 0.0, 0.0, 0.0, z[start])
    } else {
        (1.0, 1.0, basin[start], dsst[start], 0.0)
    };
    for lap in 0..2 {
        for k in 0..w {
            let off = if eastward { k } else { w - k };
            let i = (start + off) % w;
            if land[i] {
                q = p.floor + (q - p.floor) * e_moist;
                m *= e_mar;
                b = z[i] + (b - z[i]).max(0.0) * e_bar;
            } else {
                q = 1.0 - (1.0 - q) * e_rech;
                m = 1.0;
                a = basin[i];
                t = dsst[i];
                b *= e_rech;
            }
            if lap == 1 {
                out.q[i] = q * (-(b - z[i]).max(0.0) / p.vapour_scale_m).exp();
                out.maritime[i] = m;
                out.basin[i] = a;
                out.dsst[i] = t;
            }
        }
    }
    out
}

// --- The plateau's warmth (the mass-elevation effect; docs/science/mass-elevation-effect.md).

/// Half-width of the box, run twice, over which the ground's height above the sea is averaged
/// into the base height of [`mass_elevation_base`], Earth km (a fetch over which sunlit ground
/// heats the boundary layer's air: about 10 m/s for 8 hours).
pub const MEE_FOOTPRINT_KM: f64 = 150.0; // ledger: CLM-225
/// The warmth per metre of base height per unit of the month's relative sunlight, K/m (fitted to
/// 28 station-months of Tibet, the Altiplano, the Rockies and Ethiopia).
pub const MEE_K_PER_M: f64 = 2.43e-3; // ledger: CLM-226
/// The month's relative sunlight at which the warmth changes sign (270 W/m² on Earth).
pub const MEE_Q0: f64 = 0.792; // ledger: CLM-226
/// The relative sunlight's range the law was fitted over (Tibet's January to its July); beyond it
/// the law holds its ends.
pub const MEE_Q_MIN: f64 = 0.60; // ledger: CLM-227
pub const MEE_Q_MAX: f64 = 1.42; // ledger: CLM-227
/// Ground under this much land ice (m) heats no air: the per-cell climate's ice test (percell.rs),
/// so that both see the same ice (game-choice: a threshold for "covered", not a physical value).
pub const MEE_ICE_M: f32 = 1.0; // ledger: CLM-228

/// The plateau's warmth per metre of base height, K/m: how much warmer than the free air at the
/// same height the air over a broad plateau is, in a month whose daily-mean sunlight at the top of
/// the atmosphere is `q_rel` times the planet's mean (S₀/4). Negative in the dark months, when the
/// high ground cools the air instead (Pepin & Seidel 2005).
pub fn mass_elevation_k(q_rel: f64) -> f64 {
    MEE_K_PER_M * (q_rel.clamp(MEE_Q_MIN, MEE_Q_MAX) - MEE_Q0)
}

/// The base height the plateau's warmth grows with, m: the height above the sea of the bare
/// ground around the cell, averaged over the footprint (the sea and ground under land ice counted
/// at 0: neither heats the air from below), and no higher than the cell's own ground. So a peak
/// takes its surroundings' base, a lowland beside a plateau its own small height, the sea and an
/// ice sheet's interior 0, and a glacier on a bare plateau the plateau's base. `z` is the ground
/// above the sea (0 at sea), `ice` the land ice's thickness (m; `None`: none), `r` the box's
/// half-width in cells.
pub fn mass_elevation_base(
    d: &crate::surface::grid::Dims,
    z: &[f64],
    ice: Option<&[f32]>,
    r: usize,
) -> Vec<f64> {
    let mut zb: Vec<f64> = match ice {
        Some(v) if v.len() == z.len() => z
            .iter()
            .zip(v)
            .map(|(&zc, &t)| if t >= MEE_ICE_M { 0.0 } else { zc })
            .collect(),
        _ => z.to_vec(),
    };
    crate::surface::grid::box_blur(d, &mut zb, r, 2);
    for (b, &zc) in zb.iter_mut().zip(z) {
        *b = b.min(zc).max(0.0);
    }
    zb
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> MarchParams {
        MarchParams {
            recharge: 600.0,
            moisture_decay: 2500.0,
            floor: 0.15,
            maritime_decay: 1200.0,
            vapour_scale_m: 2500.0,
            barrier_recover: 1500.0,
        }
    }

    #[test]
    fn air_dries_inland_and_behind_ranges() {
        // Sea 0..20, a 6,000-km continent 20..80 (100 km cells) with a 3-km range at 30..34.
        let w = 100;
        let land: Vec<bool> = (0..w).map(|i| (20..80).contains(&i)).collect();
        let z: Vec<f64> = (0..w)
            .map(|i| {
                if (30..34).contains(&i) {
                    3000.0
                } else if land[i] {
                    200.0
                } else {
                    0.0
                }
            })
            .collect();
        let basin = vec![0.5; w];
        let dsst = vec![2.0; w];
        let east = march(&land, &z, &basin, &dsst, 100.0, true, &params());
        // Westerlies: the west coast is maritime, the lee of the range dry, the interior drier.
        assert!(east.q[20] > 0.9 && east.maritime[20] > 0.9);
        assert!(
            east.q[36] < 0.5 * east.q[29],
            "rain shadow {} vs {}",
            east.q[36],
            east.q[29]
        );
        assert!(east.q[79] < 0.4 && east.maritime[79] < 0.05);
        assert_eq!(east.dsst[50], 2.0, "carried from the sea crossed");
        assert!((east.q[15] - 1.0).abs() < 0.01, "sea air is moist");
        // Easterlies: the east coast is the wet one, and the range's shadow falls to its west.
        let west = march(&land, &z, &basin, &dsst, 100.0, false, &params());
        assert!(west.q[79] > 0.9);
        assert!(west.q[27] < 0.5 * west.q[35]);
        // A row of land only: the floor.
        let all = march(
            &[true; 10],
            &[100.0; 10],
            &[0.0; 10],
            &[0.0; 10],
            1000.0,
            true,
            &params(),
        );
        assert!(all.q.iter().all(|&x| (x - 0.15).abs() < 1e-9));
    }

    /// CLM-225 to CLM-228: the plateau's base height is the bare high ground's around a cell,
    /// so a broad plateau takes its full height, a lone peak little, the sea and an ice sheet's
    /// interior nothing, and a glacier on a bare plateau the plateau's base.
    #[test]
    fn the_plateaus_base_is_its_broad_bare_high_ground() {
        use crate::surface::grid::Dims;
        // 50-km cells, a footprint of 3 cells: a 1,500-km plateau at 4.5 km in a 4,000-km sea,
        // and a lone 4-km peak one cell wide in a 200-m lowland.
        let (w, h, r) = (80, 80, 3);
        let d = Dims::new(w, h, 50_000.0, true);
        let mut z = vec![0.0; w * h];
        for j in 10..40 {
            for i in 10..40 {
                z[j * w + i] = 4500.0;
            }
        }
        for j in 50..75 {
            for i in 50..75 {
                z[j * w + i] = 200.0;
            }
        }
        z[62 * w + 62] = 4000.0;
        let zb = mass_elevation_base(&d, &z, None, r);
        let at = |v: &[f64], j: usize, i: usize| v[j * w + i];
        assert!((at(&zb, 25, 25) - 4500.0).abs() < 1e-6, "the plateau's interior");
        assert_eq!(at(&zb, 5, 5), 0.0, "the sea");
        assert!(at(&zb, 10, 25) < 0.75 * 4500.0, "the plateau's edge: {}", at(&zb, 10, 25));
        let peak = at(&zb, 62, 62);
        assert!(peak < 300.0, "a lone peak takes its lowland's base: {peak}");
        assert!(at(&zb, 62, 55) <= 200.0, "no lowland above its own ground");
        // Ice on the plateau's west half: its interior has no base, a glacier cell inside the
        // bare east half keeps the plateau's.
        let mut ice = vec![0.0f32; w * h];
        for j in 10..40 {
            for i in 10..25 {
                ice[j * w + i] = 800.0;
            }
        }
        ice[25 * w + 33] = 200.0;
        let zi = mass_elevation_base(&d, &z, Some(&ice), r);
        assert!(at(&zi, 25, 14) < 1.0, "an ice sheet's interior: {}", at(&zi, 25, 14));
        assert!(at(&zi, 25, 33) > 0.9 * 4500.0, "a glacier on bare ground: {}", at(&zi, 25, 33));
        // A thin dusting (under a metre) is bare ground.
        let thin = vec![0.5f32; w * h];
        assert_eq!(mass_elevation_base(&d, &z, Some(&thin), r), zb);
    }

    /// CLM-226, CLM-227: the warmth per metre is positive in the sunlit months, negative in the
    /// dark ones, and held at the fitted range's ends.
    #[test]
    fn the_plateaus_warmth_follows_the_months_sunlight() {
        assert!(mass_elevation_k(1.4) > 1.4e-3, "a mid-latitude July");
        assert!(mass_elevation_k(MEE_Q0).abs() < 1e-12);
        assert!(mass_elevation_k(0.65) < 0.0, "a dark month");
        assert_eq!(mass_elevation_k(0.0), mass_elevation_k(MEE_Q_MIN));
        assert_eq!(mass_elevation_k(2.0), mass_elevation_k(MEE_Q_MAX));
        // Tibet's July (q about 1.41) at Nagqu's base of 4,780 m: about 7 K over the free air
        // (observed 5 to 6); its January (q about 0.67): about 1.4 K under it (observed 1.5 to 3).
        let jul = mass_elevation_k(1.41) * 4780.0;
        let jan = mass_elevation_k(0.67) * 4780.0;
        assert!((6.0..8.0).contains(&jul) && (-2.0..-1.0).contains(&jan), "{jul} {jan}");
    }
}
