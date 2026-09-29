// SPDX-License-Identifier: LGPL-3.0-or-later
//! Continentality and the lapse rate on the DEM (plan §8.3 item 3).
//!
//! **Lapse rate.** Surface temperature = the sea-level temperature − Γ·z (Γ = `lapse_c_per_km`,
//! 6.5 °C/km by default, the standard atmosphere's) on land above sea level.
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
            .map(|i| if (30..34).contains(&i) { 3000.0 } else if land[i] { 200.0 } else { 0.0 })
            .collect();
        let basin = vec![0.5; w];
        let dsst = vec![2.0; w];
        let east = march(&land, &z, &basin, &dsst, 100.0, true, &params());
        // Westerlies: the west coast is maritime, the lee of the range dry, the interior drier.
        assert!(east.q[20] > 0.9 && east.maritime[20] > 0.9);
        assert!(east.q[36] < 0.5 * east.q[29], "rain shadow {} vs {}", east.q[36], east.q[29]);
        assert!(east.q[79] < 0.4 && east.maritime[79] < 0.05);
        assert_eq!(east.dsst[50], 2.0, "carried from the sea crossed");
        assert!((east.q[15] - 1.0).abs() < 0.01, "sea air is moist");
        // Easterlies: the east coast is the wet one, and the range's shadow falls to its west.
        let west = march(&land, &z, &basin, &dsst, 100.0, false, &params());
        assert!(west.q[79] > 0.9);
        assert!(west.q[27] < 0.5 * west.q[35]);
        // A row of land only: the floor.
        let all = march(&[true; 10], &[100.0; 10], &[0.0; 10], &[0.0; 10], 1000.0, true, &params());
        assert!(all.q.iter().all(|&x| (x - 0.15).abs() < 1e-9));
    }
}
