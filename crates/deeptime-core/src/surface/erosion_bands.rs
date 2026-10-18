// SPDX-License-Identifier: LGPL-3.0-or-later
//! Land erosion split by process and by elevation band: a diagnostic of the deep-time surface
//! stage, kept in its run totals (`notes.surface.totals.by_band`), so a run says which process
//! wears its high ground (the plateau erosion task, 2026-10-05: is the 116–144 m/Myr above 2 km
//! hillslope diffusion, stream power, the critical slope or ice?).
//!
//! Every step, each land cell (above the sea at the step's start) adds its area × time to its
//! band, by its height above the sea at the step's start, and its gross lowering by each process:
//! stream power (`spl`, the Braun–Willett/Yuan solve, which includes the critical-slope cut),
//! the critical slope's share of it (`threshold`), glacial erosion (`glacial`) and hillslope
//! diffusion (`diffusion`), plus its net lowering after all of them (`net`, deposition counted
//! against it). High ground (above 2 km) is also split by its local relief: `high_flat` where the
//! 3 × 3 neighbourhood spans less than [`FLAT_RELIEF_M`], `high_steep` elsewhere: a plateau
//! interior against a range at the map's resolution.
//!
//! All land is also split by its local relief `R_z` (`by_relief`, the erosion recalibration of
//! 2026-10-18): max − min of the cell and its four edge neighbours, the sea counted at sea level.
//! Their centres lie within 5 km of the cell's at the 4.8-km cells of the 16k planets, so this is
//! Montgomery & Brandon's (2002) relief "within a 10-km-diameter circle" in map metres, the measure
//! their erosion rates are plotted against (ledger: ERO-172).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::grid::Dims;

/// Bands: lower ends 0, 1, 2, 3 and 4 km above the sea of the step.
pub const BANDS: usize = 5;
/// The band width, m.
pub const BAND_M: f64 = 1000.0; // ledger: ERO-175 (game-choice, diagnostic class)
/// High ground for the flat / steep split, m above the sea.
pub const HIGH_M: f64 = 2000.0; // ledger: ERO-175 (game-choice, diagnostic class)
/// Relief of a 3 × 3 neighbourhood below which high ground counts as flat (a plateau), m: a
/// diagnostic class, not a physical threshold (a 4.8-km cell at 16k quick; 500 m over two cells
/// is a mean slope of about 3°).
pub const FLAT_RELIEF_M: f64 = 500.0; // ledger: ERO-175 (game-choice, diagnostic class)

/// Lower ends of the local-relief classes after the first (which starts at 0), m: a diagnostic
/// class (the breaks of Montgomery & Brandon's Fig. 4 and the 1 and 1.5 km of their relief
/// distribution, ledger: ERO-172).
pub const RELIEF_FROM_M: [f64; 7] = [100.0, 250.0, 500.0, 1000.0, 1500.0, 2000.0, 3000.0]; // ledger: ERO-175 (game-choice, diagnostic class)
/// The number of local-relief classes.
pub const RELIEF_BINS: usize = RELIEF_FROM_M.len() + 1;

/// Area × time and eroded volumes of one class of land.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessSums {
    /// Land area × time, km²·Myr.
    pub area_km2_myr: f64,
    /// Gross lowering by stream power, including the critical-slope cut, km³.
    pub spl_km3: f64,
    /// The critical slope's share of `spl_km3`, km³.
    pub threshold_km3: f64,
    /// Gross glacial erosion, km³.
    pub glacial_km3: f64,
    /// Gross lowering by hillslope diffusion, km³.
    pub diffusion_km3: f64,
    /// Net lowering after every process of the step (deposition counted against it), km³.
    pub net_km3: f64,
}

impl ProcessSums {
    /// Mean rates, m/Myr (volume over area × time).
    pub fn json(&self) -> Value {
        let r = |v: f64| {
            if self.area_km2_myr > 0.0 {
                (v / self.area_km2_myr * 1000.0 * 10.0).round() / 10.0
            } else {
                f64::NAN
            }
        };
        json!({
            "area_km2_myr": self.area_km2_myr.round(),
            "spl_m_myr": r(self.spl_km3),
            "threshold_m_myr": r(self.threshold_km3),
            "glacial_m_myr": r(self.glacial_km3),
            "diffusion_m_myr": r(self.diffusion_km3),
            "net_m_myr": r(self.net_km3),
        })
    }
}

/// The run's sums (`Totals::by_band`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ErosionBands {
    pub bands: [ProcessSums; BANDS],
    pub high_flat: ProcessSums,
    pub high_steep: ProcessSums,
    /// All land by local relief (`relief5`), classes from 0 and [`RELIEF_FROM_M`].
    pub by_relief: [ProcessSums; RELIEF_BINS],
}

/// One step's per-cell lowering by process, metres (all ≥ 0).
pub struct StepErosion<'a> {
    /// Height at the step's start, m.
    pub h0: &'a [f64],
    pub sea_m: f64,
    pub spl: &'a [f32],
    pub threshold: &'a [f32],
    pub glacial: &'a [f32],
    pub diffusion: &'a [f32],
    /// Net change of the step (end − start), m.
    pub net: &'a [f64],
    pub dt_myr: f64,
}

impl ErosionBands {
    /// Adds one step.
    pub fn add(&mut self, d: &Dims, s: &StepErosion<'_>) {
        let a_km2 = d.cell_area() / 1.0e6;
        let km3 = d.cell_area() / 1.0e9;
        for c in 0..d.cells() {
            let z = s.h0[c] - s.sea_m;
            if z <= 0.0 {
                continue;
            }
            let band = ((z / BAND_M) as usize).min(BANDS - 1);
            let add = |p: &mut ProcessSums| {
                p.area_km2_myr += a_km2 * s.dt_myr;
                p.spl_km3 += f64::from(s.spl[c]) * km3;
                p.threshold_km3 += f64::from(s.threshold[c]) * km3;
                p.glacial_km3 += f64::from(s.glacial[c]) * km3;
                p.diffusion_km3 += f64::from(s.diffusion[c]) * km3;
                p.net_km3 += -s.net[c] * km3;
            };
            add(&mut self.bands[band]);
            let rz = relief5(d, s.h0, s.sea_m, c);
            add(&mut self.by_relief[RELIEF_FROM_M.iter().filter(|&&lo| rz >= lo).count()]);
            if z >= HIGH_M {
                if relief3(d, s.h0, c) < FLAT_RELIEF_M {
                    add(&mut self.high_flat);
                } else {
                    add(&mut self.high_steep);
                }
            }
        }
    }

    /// Mean rates per band and class, m/Myr.
    pub fn json(&self) -> Value {
        let mut m = serde_json::Map::new();
        for (i, b) in self.bands.iter().enumerate() {
            m.insert(format!("from_{}", i as f64 * BAND_M), b.json());
        }
        m.insert("high_flat".into(), self.high_flat.json());
        m.insert("high_steep".into(), self.high_steep.json());
        let land: f64 = self.bands.iter().map(|b| b.area_km2_myr).sum();
        for (i, b) in self.by_relief.iter().enumerate() {
            let lo = if i == 0 { 0.0 } else { RELIEF_FROM_M[i - 1] };
            let mut v = b.json();
            v["area_share"] = json!((b.area_km2_myr / land.max(1e-30) * 1.0e4).round() / 1.0e4);
            m.insert(format!("relief_from_{lo}"), v);
        }
        Value::Object(m)
    }
}

/// Local relief of land cell `c`: max − min of it and its four edge neighbours (wrapping in x when
/// the grid does), each at least the sea level `sea` (a sea floor counts as the shore), m.
fn relief5(d: &Dims, h: &[f64], sea: f64, c: usize) -> f64 {
    let (x, y) = ((c % d.w) as i64, (c / d.w) as i64);
    let (mut lo, mut hi) = (h[c].max(sea), h[c].max(sea));
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let yy = y + dy;
        if yy < 0 || yy >= d.h as i64 {
            continue;
        }
        let mut xx = x + dx;
        if xx < 0 || xx >= d.w as i64 {
            if !d.wrap_x {
                continue;
            }
            xx = xx.rem_euclid(d.w as i64);
        }
        let v = h[yy as usize * d.w + xx as usize].max(sea);
        lo = lo.min(v);
        hi = hi.max(v);
    }
    hi - lo
}

/// Max − min of the 3 × 3 neighbourhood of `c` (wrapping in x when the grid does), m.
fn relief3(d: &Dims, h: &[f64], c: usize) -> f64 {
    let (x, y) = ((c % d.w) as i64, (c / d.w) as i64);
    let (mut lo, mut hi) = (h[c], h[c]);
    for dy in -1..=1 {
        let yy = y + dy;
        if yy < 0 || yy >= d.h as i64 {
            continue;
        }
        for dx in -1..=1 {
            let mut xx = x + dx;
            if xx < 0 || xx >= d.w as i64 {
                if !d.wrap_x {
                    continue;
                }
                xx = xx.rem_euclid(d.w as i64);
            }
            let v = h[yy as usize * d.w + xx as usize];
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    hi - lo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bands_split_by_height_and_relief() {
        let d = Dims::new(4, 3, 1000.0, true);
        let n = d.cells();
        // A flat 2.5-km plateau (band 2000 to 3000 m), one sea cell, one 1.5-km cell.
        let mut h0 = vec![2500.0; n];
        h0[0] = -10.0;
        h0[5] = 1500.0;
        let one = vec![1.0f32; n];
        let zero = vec![0.0f32; n];
        let net = vec![-2.0; n];
        let mut b = ErosionBands::default();
        b.add(
            &d,
            &StepErosion {
                h0: &h0,
                sea_m: 0.0,
                spl: &one,
                threshold: &zero,
                glacial: &zero,
                diffusion: &one,
                net: &net,
                dt_myr: 2.0,
            },
        );
        // 1 km² cells for 2 Myr: 1 m in 2 Myr is 0.5 m/Myr.
        assert!((b.bands[1].area_km2_myr - 2.0).abs() < 1e-9);
        assert!((b.bands[2].area_km2_myr - 20.0).abs() < 1e-9);
        assert_eq!(b.bands[0].area_km2_myr, 0.0);
        let j = b.json();
        assert_eq!(j["from_2000"]["spl_m_myr"], 0.5);
        assert_eq!(j["from_2000"]["net_m_myr"], 1.0);
        // The cells next to the sea cell or the 1.5-km cell are steep, the rest flat.
        assert!(b.high_steep.area_km2_myr > 0.0 && b.high_flat.area_km2_myr > 0.0);
        assert!(
            (b.high_flat.area_km2_myr + b.high_steep.area_km2_myr - 20.0).abs() < 1e-9,
            "every high cell is flat or steep"
        );
        // By relief: the sea cell's neighbours span 2,500 m to the shore (class from 2,000 m),
        // the 1.5-km cell and its other neighbours 1,000 m (from 1,000 m), the rest none.
        let total: f64 = b.by_relief.iter().map(|p| p.area_km2_myr).sum();
        assert!(
            (total - 22.0).abs() < 1e-9,
            "every land cell has a relief class"
        );
        assert!(b.by_relief[0].area_km2_myr > 0.0);
        assert!(b.by_relief[4].area_km2_myr > 0.0 && b.by_relief[6].area_km2_myr > 0.0);
        assert!(j["relief_from_1000"]["area_share"].as_f64().unwrap() > 0.0);
    }
}
