// SPDX-License-Identifier: LGPL-3.0-or-later
//! Child deposits (plan §8.6 step 6): supergene blankets over exposed parents and placers
//! downstream of eroded ones.
//!
//! **Supergene** (Sillitoe 2005, restated in `ore_genesis_modeling.md` F and Q5): the parent's top
//! within a few hundred metres of the surface, uplift having lowered the water table, an eligible
//! (semi-arid to pluvial) climate, and at least 0.5 Myr of uninterrupted slow erosion — the
//! type's `weathering` rules give the numbers. The blanket takes the metal leached from the
//! parent's top `leach_depth_m` and concentrates it k-fold (k in the type's `grade_mult`, 2–3×
//! hypogene): its tonnage is the leached tonnage / k at k × the parent's grade, so copper is
//! conserved, and the leached part of the parent becomes barren cap.
//!
//! **Placers**: an eroded parent with a resistate product (gold) sheds a placer downstream,
//! following the present drainage (P1-04's receivers) until the channel gradient flattens below the
//! type's threshold or the transport distance runs out (gold travels a few km). The placer's size
//! comes from its own grade–tonnage model, capped so it holds at most `capture_fraction` of the
//! resistate metal the parent lost.

use deeptime_format::GridDesc;

use super::cluster::{cell_xz, distance};
use super::fields::FieldFrame;
use super::types::{PlacerRules, Weathering};

/// The climate and erosion of a site's column in one frame (pass 2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColumnSample {
    pub t_years: f64,
    pub dt_years: f64,
    pub erosion_m_myr: f64,
    pub deposition_m_myr: f64,
    pub water_table_m: f64,
    pub precip_mm: f64,
}

/// When a stable supergene window ending at the present began, and how long it lasted (Myr),
/// if the rules are met: scanning back from the newest frame while the column stays stable.
pub fn supergene_window(
    rules: &Weathering,
    parent_top_depth_m: f64,
    series: &[ColumnSample],
) -> Option<(f64, f64)> {
    if parent_top_depth_m > rules.max_top_depth_m {
        return None;
    }
    let mut stable_myr = 0.0;
    let mut start = None;
    for s in series.iter().rev() {
        let ok = s.erosion_m_myr <= rules.max_erosion_m_myr
            && s.water_table_m >= rules.min_water_table_m
            && (rules.precip_mm[0]..=rules.precip_mm[1]).contains(&s.precip_mm);
        if !ok {
            break;
        }
        stable_myr += s.dt_years / 1.0e6;
        start = Some(s.t_years - s.dt_years);
    }
    match start {
        Some(t) if stable_myr >= rules.min_stable_myr => Some((t, stable_myr)),
        _ => None,
    }
}

/// A blanket from a parent: (tonnage t, grade as mass fraction, leached parent tonnage t).
pub fn blanket(
    parent_tonnage_t: f64,
    parent_grade: f64,
    leached_fraction: f64,
    k: f64,
) -> (f64, f64, f64) {
    let leached = parent_tonnage_t * leached_fraction.clamp(0.0, 1.0);
    let k = k.max(1.0);
    (leached / k, (parent_grade * k).min(1.0), leached)
}

/// Where a placer settles: walking the receivers from `start` until the gradient to the next
/// cell drops below the rules' threshold, the distance exceeds the transport range, or the walk
/// reaches a sink or the sea. Returns the cell and the distance travelled (km).
pub fn placer_cell(
    grid: &GridDesc,
    receivers: Option<&[u32]>,
    elevation_m: &[f32],
    start: u32,
    rules: &PlacerRules,
    m_per_block_h: f64,
) -> (u32, f64) {
    let Some(rec) = receivers else {
        return (start, 0.0);
    };
    let c = grid.circumference() as f64;
    let mut cell = start;
    let mut travelled = 0.0;
    for _ in 0..grid.cells() {
        let next = rec[cell as usize];
        if next == cell || elevation_m[next as usize] < 0.0 {
            break;
        }
        let step_km =
            distance(c, cell_xz(grid, cell), cell_xz(grid, next)) * m_per_block_h / 1000.0;
        if travelled + step_km > rules.transport_km {
            break;
        }
        let gradient = f64::from(elevation_m[cell as usize] - elevation_m[next as usize]) / step_km;
        travelled += step_km;
        cell = next;
        if gradient < rules.flatten_gradient_m_km {
            break;
        }
    }
    (cell, travelled)
}

/// The column samples of `cells` in a frame (pass 2).
pub fn sample(frame: &FieldFrame, cells: &[u32]) -> Vec<ColumnSample> {
    use super::fields::{DEPOSITION, EROSION, PRECIP, WATER_TABLE};
    let v = |name: &str, c: u32| {
        frame
            .values(name)
            .map_or(0.0, |v| f64::from(v.at(c as usize)))
    };
    cells
        .iter()
        .map(|&c| ColumnSample {
            t_years: frame.t_years,
            dt_years: frame.dt_years,
            erosion_m_myr: v(EROSION, c),
            deposition_m_myr: v(DEPOSITION, c),
            water_table_m: v(WATER_TABLE, c),
            precip_mm: v(PRECIP, c),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Weathering {
        Weathering {
            max_top_depth_m: 300.0,
            min_water_table_m: 100.0,
            precip_mm: [150.0, 1500.0],
            min_stable_myr: 0.5,
            max_erosion_m_myr: 60.0,
            grade_mult: [2.0, 3.0],
            leach_depth_m: 250.0,
        }
    }

    fn sample_at(t_myr: f64, erosion: f64, wt: f64, p: f64) -> ColumnSample {
        ColumnSample {
            t_years: t_myr * 1.0e6,
            dt_years: 1.0e7,
            erosion_m_myr: erosion,
            deposition_m_myr: 0.0,
            water_table_m: wt,
            precip_mm: p,
        }
    }

    /// Acceptance: a shallow parent under a dry, slowly eroding, deep-water-table column enriches;
    /// a deep parent, a wet climate or fast erosion now do not.
    #[test]
    fn supergene_documented_cases() {
        let series = vec![
            sample_at(-30.0, 200.0, 50.0, 300.0),
            sample_at(-20.0, 20.0, 180.0, 300.0),
            sample_at(-10.0, 15.0, 200.0, 350.0),
            sample_at(0.0, 10.0, 220.0, 320.0),
        ];
        let (t0, myr) = supergene_window(&rules(), 120.0, &series).unwrap();
        assert_eq!(myr, 30.0);
        assert_eq!(t0, -30.0e6);
        assert!(
            supergene_window(&rules(), 900.0, &series).is_none(),
            "too deep"
        );
        let mut wet = series.clone();
        wet[3].precip_mm = 3000.0;
        assert!(
            supergene_window(&rules(), 120.0, &wet).is_none(),
            "too wet now"
        );
        let mut fast = series.clone();
        fast[3].erosion_m_myr = 300.0;
        assert!(
            supergene_window(&rules(), 120.0, &fast).is_none(),
            "eroding too fast"
        );
        // Copper is conserved: 100 Mt × 20 % leached at 0.5 % → 10 Mt at 1.0 % (k = 2).
        let (t, g, leached) = blanket(100.0e6, 0.005, 0.2, 2.0);
        assert!((t - 10.0e6).abs() < 1.0 && (g - 0.01).abs() < 1e-12);
        assert!((t * g - leached * 0.005).abs() < 1e-3);
    }

    #[test]
    fn placers_follow_the_drainage_until_it_flattens() {
        let grid = GridDesc::for_planet("L1", 512, 16).unwrap(); // 32 × 16, 4.8 km cells
        let n = grid.cells();
        // A slope down the row 8 eastwards: steep for three cells, then flat.
        let mut elev = vec![500.0f32; n];
        let row = 8 * 32;
        for i in 0..32 {
            elev[row + i] = match i {
                0 => 900.0,
                1 => 600.0,
                2 => 300.0,
                3 => 295.0,
                _ => 290.0 - i as f32,
            };
        }
        let rec: Vec<u32> = (0..n as u32)
            .map(|c| {
                if (c as usize) >= row && (c as usize) < row + 31 {
                    c + 1
                } else {
                    c
                }
            })
            .collect();
        let r = PlacerRules {
            transport_km: 50.0,
            capture_fraction: 0.5,
            flatten_gradient_m_km: 10.0,
        };
        let (cell, km) = placer_cell(&grid, Some(&rec), &elev, row as u32, &r, 300.0);
        assert_eq!(cell as usize, row + 3, "stops where the gradient flattens");
        assert!((km - 3.0 * 4.8).abs() < 1e-9);
        let short = PlacerRules {
            transport_km: 5.0,
            ..r.clone()
        };
        assert_eq!(
            placer_cell(&grid, Some(&rec), &elev, row as u32, &short, 300.0).0 as usize,
            row + 1
        );
        assert_eq!(placer_cell(&grid, None, &elev, 7, &r, 300.0), (7, 0.0));
    }
}
