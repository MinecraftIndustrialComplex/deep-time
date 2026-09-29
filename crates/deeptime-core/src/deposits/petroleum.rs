// SPDX-License-Identifier: LGPL-3.0-or-later
//! Petroleum systems and coal rank (plan §8.6 step 7): Lopatin's time–temperature index,
//! TTI = Σ Δt_n · 2ⁿ with Δt in Myr and n the 10 °C interval of the layer's temperature
//! (n = 0 for 100–110 °C, n = −1 for 90–100 °C, …; Waples 1980, AAPG Bulletin 64:916), per cell
//! of the organic-rich layer (`deeptime:burial_temp_c`). Oil generation starts at the type's
//! `tti_onset`, peaks at `tti_peak` and ends at `tti_end`.
//!
//! A **kitchen** is a connected region of mature source rock (TTI ≥ onset, TOC ≥ the type's
//! minimum). A **trap** cell has structural closure (from the frame its trap field first passed
//! the threshold), a seal and a reservoir. A trap is **charged** when a kitchen within the
//! migration range was still generating after the trap formed: "charge in order", the petroleum
//! system's critical moment (Magoon & Dow 1994). Charged traps become the petroleum type's
//! favourability, so tracts, counts, positions and sizes follow the common pipeline.

use deeptime_format::GridDesc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::cluster::{cell_xz, distance};
use super::fields::{FieldFrame, Values};
use super::tracts::{Tract, find_tracts};
use super::types::PetroleumRules;

/// A layer counts as generating only while at least this hot: TTI accrues at every temperature
/// (the index keeps halving per 10 °C below 100 °C), but a cooled, inverted basin stops charging
/// traps. A design threshold (catagenesis spans roughly 50–150 °C; little oil forms below ≈80 °C).
pub const GEN_MIN_TEMP_C: f64 = 80.0;

/// TTI increment for `dt_myr` at `temp_c` (Waples 1980's interval form).
pub fn tti_increment(temp_c: f64, dt_myr: f64) -> f64 {
    let n = ((temp_c - 100.0) / 10.0).floor();
    dt_myr * 2f64.powf(n)
}

/// Per-cell maturity and trap timing.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct CellPet {
    pub tti: f32,
    /// Times (years) TTI crossed the onset and the peak, and the last frame it generated
    /// (TTI rising inside [onset, end)); NaN until then.
    pub t_onset: f32,
    pub t_peak: f32,
    pub t_last_gen: f32,
    /// Time the trap field first passed the threshold; NaN if never.
    pub t_trap: f32,
    pub toc_max: f32,
}

impl Default for CellPet {
    fn default() -> Self {
        CellPet {
            tti: 0.0,
            t_onset: f32::NAN,
            t_peak: f32::NAN,
            t_last_gen: f32::NAN,
            t_trap: f32::NAN,
            toc_max: 0.0,
        }
    }
}

/// Folds one frame into the maturity accumulators. `rules` gives the petroleum field names; a
/// coal type passes only its temperature field (`trap`/`toc` then read as zero).
pub fn accumulate(
    acc: &mut [CellPet],
    frame: &FieldFrame,
    temp: &str,
    rules: Option<&PetroleumRules>,
) {
    let Some(tv) = frame.values(temp) else { return };
    let zero = Values::Uniform(0.0);
    let (trap, toc) = match rules {
        Some(r) => (
            frame.values(&r.trap_field).unwrap_or(zero),
            frame.values(&r.source_field).unwrap_or(zero),
        ),
        None => (zero, zero),
    };
    let (onset, peak, end, trap_th) = rules.map_or(
        (f64::INFINITY, f64::INFINITY, f64::INFINITY, f64::INFINITY),
        |r| (r.tti_onset, r.tti_peak, r.tti_end, r.trap_threshold),
    );
    let t = frame.t_years as f32;
    let dt = frame.dt_myr();
    acc.par_iter_mut().enumerate().for_each(|(c, a)| {
        let before = f64::from(a.tti);
        let temp = f64::from(tv.at(c));
        let after = before + tti_increment(temp, dt);
        a.tti = after as f32;
        if a.t_onset.is_nan() && after >= onset {
            a.t_onset = t;
        }
        if a.t_peak.is_nan() && after >= peak {
            a.t_peak = t;
        }
        if after >= onset && before < end && temp >= GEN_MIN_TEMP_C {
            a.t_last_gen = t;
        }
        if a.t_trap.is_nan() && f64::from(trap.at(c)) >= trap_th {
            a.t_trap = t;
        }
        a.toc_max = a.toc_max.max(toc.at(c));
    });
}

/// A kitchen.
#[derive(Clone, Debug, PartialEq)]
pub struct Kitchen {
    pub tract: Tract,
    /// Earliest onset and latest generation among its cells (years).
    pub t_onset: f64,
    pub t_last_gen: f64,
    pub tti_max: f64,
    pub toc_mean: f64,
    /// The cell nearest its area-weighted centre.
    pub centre: u32,
}

/// The kitchens of a run, largest first among equals in scan order, and how many connected
/// regions of mature source rock were too small to count (`PetroleumRules::min_kitchen_km2`): on a
/// real history most of them are a cell or two of locally hot rock, which cannot charge a trap.
pub fn kitchens(
    grid: &GridDesc,
    acc: &[CellPet],
    r: &PetroleumRules,
    m_per_block_h: f64,
) -> (Vec<Kitchen>, usize) {
    let all = mature_regions(grid, acc, r, m_per_block_h);
    let found = all.len();
    let kept: Vec<Kitchen> = all
        .into_iter()
        .filter(|k| k.tract.area_km2 >= r.min_kitchen_km2)
        .collect();
    let dropped = found - kept.len();
    (kept, dropped)
}

fn mature_regions(
    grid: &GridDesc,
    acc: &[CellPet],
    r: &PetroleumRules,
    m_per_block_h: f64,
) -> Vec<Kitchen> {
    let mature: Vec<f64> = acc
        .iter()
        .map(|a| {
            if f64::from(a.tti) >= r.tti_onset && f64::from(a.toc_max) >= r.min_source {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    find_tracts(grid, &mature, 0.5, m_per_block_h)
        .into_iter()
        .map(|tract| {
            let cells = &tract.cells;
            let t_onset = cells
                .iter()
                .map(|c| f64::from(acc[*c as usize].t_onset))
                .fold(f64::INFINITY, f64::min);
            let t_last_gen = cells
                .iter()
                .map(|c| f64::from(acc[*c as usize].t_last_gen))
                .filter(|t| t.is_finite())
                .fold(f64::NEG_INFINITY, f64::max);
            let tti_max = cells
                .iter()
                .map(|c| f64::from(acc[*c as usize].tti))
                .fold(0.0, f64::max);
            let toc_mean = cells
                .iter()
                .map(|c| f64::from(acc[*c as usize].toc_max))
                .sum::<f64>()
                / cells.len() as f64;
            let c = grid.circumference() as f64;
            // The cell with the smallest summed distance to a sample of the others.
            let step = (cells.len() / 64).max(1);
            let centre = *cells
                .iter()
                .min_by(|a, b| {
                    let s = |x: u32| -> f64 {
                        cells
                            .iter()
                            .step_by(step)
                            .map(|o| distance(c, cell_xz(grid, x), cell_xz(grid, *o)))
                            .sum()
                    };
                    s(**a).total_cmp(&s(**b))
                })
                .expect("a tract has cells");
            Kitchen {
                tract,
                t_onset,
                t_last_gen,
                tti_max,
                toc_mean,
                centre,
            }
        })
        .collect()
}

/// The charge of one trap cell: the kitchen that charged it and when.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Charge {
    pub kitchen: usize,
    pub t_charge: f64,
    pub migration_km: f64,
}

/// Whether trap cell `cell` (formed at `t_trap`) was charged by a kitchen within range.
pub fn charge(
    grid: &GridDesc,
    kitchens: &[Kitchen],
    cell: u32,
    t_trap: f64,
    r: &PetroleumRules,
    m_per_block_h: f64,
) -> Option<Charge> {
    let c = grid.circumference() as f64;
    let here = cell_xz(grid, cell);
    let mut best: Option<Charge> = None;
    for (k, kit) in kitchens.iter().enumerate() {
        if t_trap.is_nan() || t_trap > kit.t_last_gen {
            continue;
        }
        let km = kit
            .tract
            .cells
            .iter()
            .map(|o| distance(c, here, cell_xz(grid, *o)))
            .fold(f64::INFINITY, f64::min)
            * m_per_block_h
            / 1000.0;
        if km > r.migration_km {
            continue;
        }
        let ch = Charge {
            kitchen: k,
            t_charge: t_trap.max(kit.t_onset),
            migration_km: km,
        };
        if best.is_none_or(|b| km < b.migration_km) {
            best = Some(ch);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> PetroleumRules {
        PetroleumRules {
            tti_onset: 15.0,
            tti_peak: 75.0,
            tti_end: 160.0,
            migration_km: 60.0,
            source_field: "deeptime:toc".into(),
            temperature_field: "deeptime:temp".into(),
            trap_field: "deeptime:trap".into(),
            seal_field: "deeptime:seal".into(),
            reservoir_field: "deeptime:res".into(),
            trap_threshold: 0.3,
            seal_threshold: 0.3,
            reservoir_threshold: 0.3,
            min_source: 1.0,
            min_kitchen_km2: 0.0,
        }
    }

    #[test]
    fn tti_intervals() {
        assert_eq!(tti_increment(105.0, 10.0), 10.0);
        assert_eq!(tti_increment(115.0, 10.0), 20.0);
        assert_eq!(tti_increment(95.0, 10.0), 5.0);
        assert_eq!(tti_increment(100.0, 1.0), 1.0);
    }

    /// A source heated past the onset, a trap that forms while it still generates (charged) and
    /// one that forms after generation stopped (dry).
    #[test]
    fn charge_in_order() {
        let grid = GridDesc::for_planet("L1", 512, 16).unwrap(); // 32 × 16
        let n = grid.cells();
        let r = rules();
        let mut acc = vec![CellPet::default(); n];
        let kitchen_cells = [8 * 32 + 10, 8 * 32 + 11];
        let early_trap = 8 * 32 + 12; // one cell (4.8 km) away
        let late_trap = 8 * 32 + 13;
        for (k, (t_myr, temp)) in [
            (-100.0, 90.0),
            (-90.0, 112.0),
            (-80.0, 118.0),
            (-70.0, 60.0),
            (-60.0, 60.0),
        ]
        .iter()
        .enumerate()
        {
            let mut f = FieldFrame {
                t_years: t_myr * 1.0e6,
                dt_years: 1.0e7,
                ..FieldFrame::default()
            };
            let mut temp_v = vec![20.0f32; n];
            let mut toc = vec![0.0f32; n];
            let mut trap = vec![0.0f32; n];
            for c in kitchen_cells {
                temp_v[c] = *temp;
                toc[c] = 3.0;
            }
            if k >= 1 {
                trap[early_trap] = 1.0;
            }
            if k >= 3 {
                trap[late_trap] = 1.0;
            }
            f.fields.insert("deeptime:temp".into(), temp_v);
            f.fields.insert("deeptime:toc".into(), toc);
            f.fields.insert("deeptime:trap".into(), trap);
            accumulate(&mut acc, &f, "deeptime:temp", Some(&r));
        }
        let a = acc[kitchen_cells[0]];
        // 10·2⁻¹ + 10·2 + 10·2 + 2·10·2⁻⁴ = 46.25: past the onset (at −90), not the peak; the
        // cooled frames add a little TTI but no generation.
        assert!((a.tti - 46.25).abs() < 1e-4, "{}", a.tti);
        assert_eq!(a.t_onset, -90.0e6);
        assert!(a.t_peak.is_nan());
        assert_eq!(a.t_last_gen, -80.0e6);
        let (ks, dropped) = kitchens(&grid, &acc, &r, 300.0);
        assert_eq!((ks.len(), dropped), (1, 0));
        // A minimum area drops the kitchen (two cells of ≈ 2 km²… at 300 m blocks: 2 · 23 km²).
        let small = PetroleumRules {
            min_kitchen_km2: 1.0e6,
            ..r.clone()
        };
        let (none, dropped) = kitchens(&grid, &acc, &small, 300.0);
        assert_eq!((none.len(), dropped), (0, 1));
        assert_eq!(ks[0].t_onset, -90.0e6);
        let early = charge(
            &grid,
            &ks,
            early_trap as u32,
            f64::from(acc[early_trap].t_trap),
            &r,
            300.0,
        );
        let early = early.expect("charged");
        assert_eq!(early.t_charge, -90.0e6);
        assert!((early.migration_km - 4.8).abs() < 1e-9);
        assert!(
            charge(
                &grid,
                &ks,
                late_trap as u32,
                f64::from(acc[late_trap].t_trap),
                &r,
                300.0
            )
            .is_none()
        );
    }
}
