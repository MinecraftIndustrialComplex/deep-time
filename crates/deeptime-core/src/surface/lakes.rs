// SPDX-License-Identifier: LGPL-3.0-or-later
//! The water standing over each cell, for the flexure (ISO-126, `surface.flexure.water_by_basin`).
//!
//! The flexure loads a cell with the water over it and restores a wet cell's plate against the
//! mantle less the water that fills its room. Before 2026-10-23 it took every cell at or under the
//! sea level as under the sea: a closed basin under sea level, behind a rim above it, carried the
//! sea's water whether or not any reached it (the Dead Sea's floor, a dry rift), and a lake above
//! the sea carried none. This module gives the flexure the surface's own water instead:
//!
//! - **The open sea** (`pflood::open_sea`'s [`BASE`] cells): the sea level.
//! - **A lake** (cells the priority flood filled by more than [`LAKE_TOL_M`], connected and at
//!   one level): the level its water balance holds, as the bake's lakes take it
//!   (`hydro::network`, HYD-100, by the same [`HydroParams::budget`](crate::hydro::HydroParams)).
//!   What runs off its catchment, and what spills from the lakes above it, flows in; a cell
//!   under its water loses what open water evaporates less the rain on it. A lake that still
//!   gains water wet to its rim is full, at the flood's level, and spills the rest; one that
//!   cannot fill stands where its losses take the inflow: its cells by elevation go under from
//!   the lowest up while the balance holds (the hypsometry). A lake whose level is at or under
//!   the sea level is open to the sea and stands at it.
//! - **Everything else**: no water ([`DRY`]), whatever its height.
//!
//! The lakes are the step's last flood's; the open sea is taken again on the elevation the flexure
//! starts from ([`at_step_end`]), so ground the step's later processes put under the sea (a rift
//! the spreading opened, a coast the sea cut) carries it.
//!
//! Simplifications (game-choice: numerical, agent 2026-10-23): one level per flooded depression,
//! so a basin whose balance level falls under a sill inside it is still one water body (its
//! water's mass is right, its share between the sub-basins is not); a lake with two outlets
//! spills over the one its last exit drains to; the levels are those of the step's last flood, on
//! the elevation it flooded.

use super::grid::SurfaceGrid;
use super::pflood::{BASE, LAKE_TOL_M, LAND};
use super::routing::Routing;

/// No water stands over the cell.
pub const DRY: f64 = f64::NEG_INFINITY;

/// Cell-steps at or under the sea level with no water over them (the sea's rule had them wet).
pub const DRY_BELOW_SEA_GLOBAL: &str = "surface/flexure_dry_below_sea_cells";
/// Cell-steps above the sea level under a lake (the sea's rule had them dry).
pub const WET_ABOVE_SEA_GLOBAL: &str = "surface/flexure_wet_above_sea_cells";
/// Lake-steps: lakes above the sea level, …
pub const LAKES_GLOBAL: &str = "surface/flexure_lakes";
/// … and of those, dry ones.
pub const DRY_LAKES_GLOBAL: &str = "surface/flexure_dry_lakes";

/// What a cell gives and takes a year, m (of water over its area).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CellWater {
    /// What runs off it as land.
    pub runoff_m_yr: f64,
    /// What it loses as open water: the evaporation less the rain on it (negative: it gains).
    pub loss_m_yr: f64,
}

/// The water levels and what they did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Levels {
    /// The level of the water over each cell, m ([`DRY`]: none).
    pub water_z: Vec<f64>,
    /// Lakes (flooded depressions above the sea level), …
    pub lakes: usize,
    /// … of which full, …
    pub full: usize,
    /// … and dry (a playa or bare floor).
    pub dry: usize,
    /// Cells at or under the sea level with no water over them (the sea's rule had them wet).
    pub dry_below_sea: usize,
    /// Cells above the sea level under a lake (the sea's rule had them dry).
    pub wet_above_sea: usize,
}

/// One lake while the balance is taken.
struct Lake {
    /// Its cells.
    cells: Vec<u32>,
    /// The flood's level: its lowest fill.
    level: f64,
    /// Exits not yet reached by the upstream pass.
    exits_left: u32,
    /// What flowed in, m³/yr.
    supply: f64,
}

// ledger: ISO-126
/// The level of the water over every cell (see the module documentation). `h`, `filled`, `kind`
/// and `r` are one flood's and routing's; `water(c)` is cell `c`'s budget; `balance` false keeps
/// every lake full (the `hydro.lake_balance` switch).
#[allow(clippy::too_many_arguments)]
pub fn water_levels<G: SurfaceGrid>(
    d: &G,
    h: &[f64],
    filled: &[f64],
    kind: &[u8],
    r: &Routing,
    sea: f64,
    balance: bool,
    water: &(dyn Fn(usize) -> CellWater + Sync),
) -> Levels {
    let n = d.cells();
    const NONE: u32 = u32::MAX;
    let is_lake = |c: usize| kind[c] == LAND && filled[c] - h[c] > LAKE_TOL_M;
    // 1. The lakes: lake cells connected at one level.
    let mut id = vec![NONE; n];
    let mut lakes: Vec<Lake> = Vec::new();
    let mut todo: Vec<u32> = Vec::new();
    for s in 0..n {
        if id[s] != NONE || !is_lake(s) {
            continue;
        }
        let k = lakes.len() as u32;
        let mut lake = Lake {
            cells: Vec::new(),
            level: f64::INFINITY,
            exits_left: 0,
            supply: 0.0,
        };
        id[s] = k;
        todo.push(s as u32);
        while let Some(c) = todo.pop() {
            let c = c as usize;
            lake.cells.push(c as u32);
            lake.level = lake.level.min(filled[c]);
            for (m, _) in d.neighbours(c) {
                if id[m] == NONE && is_lake(m) && (filled[m] - filled[c]).abs() <= LAKE_TOL_M {
                    id[m] = k;
                    todo.push(m as u32);
                }
            }
        }
        lakes.push(lake);
    }
    let rcv = |c: usize| r.rcv.get(c).map_or(c, |&x| x as usize);
    let exits = |c: usize| id[c] != NONE && (rcv(c) == c || id[rcv(c)] != id[c]);
    for c in 0..n {
        if exits(c) {
            lakes[id[c] as usize].exits_left += 1;
        }
    }

    // 2. The balance, upstream first: every cell after its donors.
    let mut level = vec![DRY; lakes.len()];
    let mut q = vec![0.0f64; n];
    let mut full = 0;
    let mut dry = 0;
    let mut resolve = |lake: &mut Lake| -> (f64, f64) {
        if lake.level <= sea + LAKE_TOL_M {
            // Open to the sea: the sea reaches it at its own level.
            return (sea, 0.0);
        }
        let loss: f64 = lake
            .cells
            .iter()
            .map(|&c| water(c as usize).loss_m_yr * d.area(c as usize))
            .sum();
        let at_full = lake.supply - loss;
        if !balance || at_full >= 0.0 {
            full += 1;
            return (lake.level, at_full.max(0.0));
        }
        // It cannot fill: its cells go under from the lowest up while the inflow, and the
        // runoff of its floor still dry, cover the losses of those under water.
        lake.cells
            .sort_by(|&a, &b| h[a as usize].total_cmp(&h[b as usize]));
        let budgets: Vec<(f64, f64)> = lake
            .cells
            .iter()
            .map(|&c| {
                let (wc, a) = (water(c as usize), d.area(c as usize));
                (wc.runoff_m_yr.max(0.0) * a, wc.loss_m_yr * a)
            })
            .collect();
        let mut g = lake.supply + budgets.iter().map(|b| b.0).sum::<f64>();
        let mut wet = 0;
        for &(run, lose) in &budgets {
            let next = g - run - lose;
            if next < 0.0 {
                break;
            }
            g = next;
            wet += 1;
        }
        if wet == 0 {
            dry += 1;
            (DRY, 0.0)
        } else {
            // The level is the shore: the lowest cell it does not cover.
            (h[lake.cells[wet.min(lake.cells.len() - 1)] as usize], 0.0)
        }
    };
    for &c in r.stack.iter().rev() {
        let c = c as usize;
        let to = rcv(c);
        if id[c] == NONE {
            q[c] += water(c).runoff_m_yr.max(0.0) * d.area(c);
            if to != c {
                q[to] += q[c];
            }
            continue;
        }
        let k = id[c] as usize;
        if !exits(c) {
            q[to] += q[c];
            continue;
        }
        let lake = &mut lakes[k];
        lake.supply += q[c];
        lake.exits_left -= 1;
        if lake.exits_left == 0 {
            let (z, out) = resolve(lake);
            level[k] = z;
            if to != c {
                q[to] += out;
            }
        }
    }

    // 3. The level over each cell.
    let mut out = Levels {
        lakes: lakes.iter().filter(|l| l.level > sea + LAKE_TOL_M).count(),
        full,
        dry,
        ..Levels::default()
    };
    out.water_z = (0..n)
        .map(|c| {
            if kind[c] == BASE {
                sea
            } else if id[c] != NONE {
                level[id[c] as usize]
            } else {
                DRY
            }
        })
        .collect();
    for (&hc, &z) in h.iter().zip(&out.water_z) {
        if hc <= sea && hc > z {
            out.dry_below_sea += 1;
        }
        if hc > sea && hc < z {
            out.wet_above_sea += 1;
        }
    }
    out
}

// ledger: ISO-126
/// The water over each cell when the flexure starts, on the step's final elevation `h`: the sea
/// level where the open sea reaches the cell on `h` (`pflood::open_sea` with the surface stage's
/// `min_cells`), else what the step's last flood had (`flood`, [`water_levels`]); the lake counts
/// are the flood's, the cells whose water differs from the sea's rule are counted on `h`.
pub fn at_step_end<G: SurfaceGrid>(
    d: &G,
    h: &[f64],
    sea: f64,
    min_cells: usize,
    flood: &Levels,
) -> Levels {
    let n = d.cells();
    let mut open = vec![LAND; n];
    super::pflood::open_sea(d, h, sea, min_cells, &mut open);
    let water_z: Vec<f64> = (0..n)
        .map(|c| {
            if open[c] == BASE {
                sea
            } else {
                flood.water_z.get(c).copied().unwrap_or(DRY)
            }
        })
        .collect();
    let (mut dry_below_sea, mut wet_above_sea) = (0, 0);
    for c in 0..n {
        if h[c] <= sea && h[c] > water_z[c] {
            dry_below_sea += 1;
        }
        if h[c] > sea && h[c] < water_z[c] {
            wet_above_sea += 1;
        }
    }
    Levels {
        water_z,
        lakes: flood.lakes,
        full: flood.full,
        dry: flood.dry,
        dry_below_sea,
        wet_above_sea,
    }
}

#[cfg(test)]
mod tests {
    use super::super::grid::Dims;
    use super::super::pflood;
    use super::super::routing;
    use super::*;

    /// The open sea's least body in the tests: the strip of sea (30 cells) is one, the bowl
    /// (25) is not.
    const SEA_CELLS: usize = 30;

    /// A flood and a routing of `h` with the sea at `sea`, as the surface stage makes them.
    fn levels(d: &Dims, h: &[f64], sea: f64, water: CellWater, balance: bool) -> Levels {
        let n = h.len();
        let mut kind = vec![LAND; n];
        pflood::open_sea(d, h, sea, SEA_CELLS, &mut kind);
        let base_z = vec![sea; n];
        let mut filled = vec![0.0; n];
        pflood::flood(d, h, &base_z, &mut kind, &mut filled);
        let mut r = routing::Routing::default();
        routing::receivers(d, &filled, &kind, &mut r);
        routing::stack(d, &kind, &mut r);
        water_levels(d, h, &filled, &kind, &r, sea, balance, &|_| water)
    }

    /// A strip of sea on the west, land rising east, and a closed bowl in the land: 20 × 10
    /// cells, the bowl 5 × 5 around (12, 5), its floor at `floor` m, its rim at 400 m.
    fn bowl(floor: f64) -> (Dims, Vec<f64>) {
        let (w, h) = (20usize, 10usize);
        let d = Dims::new(w, h, 1000.0, false);
        let z = (0..w * h)
            .map(|c| {
                let (i, j) = ((c % w) as i64, (c / w) as i64);
                if i < 3 {
                    -1000.0
                } else if (i - 12).abs() <= 2 && (j - 5).abs() <= 2 {
                    // The bowl: deeper towards its middle.
                    floor + 50.0 * ((i - 12).abs().max((j - 5).abs())) as f64
                } else {
                    400.0 + 10.0 * i as f64
                }
            })
            .collect();
        (d, z)
    }

    /// The open sea stands at the sea level and dry land carries nothing.
    #[test]
    fn the_sea_is_the_sea_and_land_is_dry() {
        let (d, z) = bowl(-500.0);
        let wet = CellWater {
            runoff_m_yr: 1.0,
            loss_m_yr: 0.0,
        };
        let out = levels(&d, &z, 0.0, wet, true);
        for c in 0..z.len() {
            if c % d.w < 3 {
                assert_eq!(out.water_z[c], 0.0, "sea cell {c}");
            } else if (c % d.w) as i64 - 12 > 2 || ((c % d.w) as i64 - 12) < -2 {
                assert_eq!(out.water_z[c], DRY, "land cell {c}");
            }
        }
    }

    /// A wet climate fills a closed bowl under sea level to its rim: its water stands above the
    /// sea, at the flood's level.
    #[test]
    fn a_wet_climate_fills_a_closed_bowl_to_its_rim() {
        let (d, z) = bowl(-500.0);
        let wet = CellWater {
            runoff_m_yr: 1.0,
            loss_m_yr: 0.0,
        };
        let out = levels(&d, &z, 0.0, wet, true);
        let mid = 5 * d.w + 12;
        let z_mid = out.water_z[mid];
        assert!(
            z_mid > 490.0 && z_mid < 490.001,
            "the rim at 490 m: {z_mid}"
        );
        assert_eq!((out.lakes, out.full, out.dry), (1, 1, 0));
        assert_eq!((out.dry_below_sea, out.wet_above_sea), (0, 0));
        // The same bowl above the sea level: its water stands where the sea's rule had none.
        let (d, z) = bowl(100.0);
        let out = levels(&d, &z, 0.0, wet, true);
        assert_eq!(out.wet_above_sea, 25);
    }

    /// A dry climate leaves a closed bowl under sea level dry: no water over it although it lies
    /// under the sea level, where the sea's rule put the sea.
    #[test]
    fn a_dry_climate_leaves_a_closed_bowl_dry() {
        let (d, z) = bowl(-500.0);
        let arid = CellWater {
            runoff_m_yr: 0.0,
            loss_m_yr: 2.0,
        };
        let out = levels(&d, &z, 0.0, arid, true);
        let mid = 5 * d.w + 12;
        assert_eq!(out.water_z[mid], DRY);
        assert_eq!((out.lakes, out.full, out.dry), (1, 0, 1));
        // The bowl's 25 cells all lie under the sea level.
        assert_eq!(out.dry_below_sea, 25);
        // Without the balance the lake is full, as every flooded lake was.
        let kept = levels(&d, &z, 0.0, arid, false);
        assert!(kept.water_z[mid] > 400.0);
    }

    /// A lake that cannot fill stands where its losses take its inflow: the bowl's floor is one
    /// cell, its first ring eight more, so an inflow that covers 1.5 cells' losses puts the floor
    /// under water with the shore at the first ring, and one that covers 9.5 the floor and the
    /// first ring, with the shore at the second.
    #[test]
    fn a_terminal_lake_stands_where_its_losses_take_its_inflow() {
        let (d, z) = bowl(-500.0);
        let n = z.len();
        let mut kind = vec![LAND; n];
        pflood::open_sea(&d, &z, 0.0, SEA_CELLS, &mut kind);
        let mut filled = vec![0.0; n];
        pflood::flood(&d, &z, &vec![0.0; n], &mut kind, &mut filled);
        let mut r = routing::Routing::default();
        routing::receivers(&d, &filled, &kind, &mut r);
        routing::stack(&d, &kind, &mut r);
        // The cells draining into the bowl, outside it (the sea's cells are filled to the sea
        // level, a kilometre over their floor, but are no lake).
        let lake = |c: usize| kind[c] == LAND && filled[c] - z[c] > LAKE_TOL_M;
        let mut into = 0usize;
        for c in 0..n {
            if lake(c) {
                continue;
            }
            let mut k = c;
            while r.rcv[k] as usize != k && !lake(k) {
                k = r.rcv[k] as usize;
            }
            if lake(k) {
                into += 1;
            }
        }
        assert!(into > 0, "the bowl has a catchment");
        let mid = 5 * d.w + 12;
        for (cells, shore, wet) in [(1.5, -450.0, 1), (9.5, -400.0, 9)] {
            // Each catchment cell gives 1 m a year; a cell under water loses `loss`.
            let loss = into as f64 / cells;
            let water = |c: usize| CellWater {
                runoff_m_yr: if lake(c) { 0.0 } else { 1.0 },
                loss_m_yr: loss,
            };
            let out = water_levels(&d, &z, &filled, &kind, &r, 0.0, true, &water);
            assert_eq!(out.water_z[mid], shore, "inflow for {cells} cells");
            let under = (0..n).filter(|&c| z[c] < out.water_z[c] && kind[c] != BASE);
            assert_eq!(under.count(), wet, "inflow for {cells} cells");
            assert_eq!((out.lakes, out.full, out.dry), (1, 0, 0));
        }
    }

    /// A lake whose flood level is at or under the sea level is open to the sea and stands at
    /// it.
    #[test]
    fn a_lake_at_the_sea_level_is_the_sea() {
        let (d, z) = bowl(-500.0);
        let n = z.len();
        let mut kind = vec![LAND; n];
        pflood::open_sea(&d, &z, 0.0, SEA_CELLS, &mut kind);
        let mut filled = vec![0.0; n];
        pflood::flood(&d, &z, &vec![0.0; n], &mut kind, &mut filled);
        let mut r = routing::Routing::default();
        routing::receivers(&d, &filled, &kind, &mut r);
        routing::stack(&d, &kind, &mut r);
        let arid = CellWater {
            runoff_m_yr: 0.0,
            loss_m_yr: 2.0,
        };
        // The flood's lake stands at 490 m, its rim; a sea at 600 m reaches it.
        let out = water_levels(&d, &z, &filled, &kind, &r, 600.0, true, &|_| arid);
        let mid = 5 * d.w + 12;
        assert_eq!(out.water_z[mid], 600.0);
        assert_eq!((out.lakes, out.full, out.dry), (0, 0, 0));
    }

    /// Ground the sea reaches after the flood (here a channel cut from the sea to a dry bowl)
    /// stands under the sea at the step's end.
    #[test]
    fn ground_the_sea_reaches_after_the_flood_is_under_the_sea() {
        let (d, mut z) = bowl(-500.0);
        let arid = CellWater {
            runoff_m_yr: 0.0,
            loss_m_yr: 2.0,
        };
        let flood = levels(&d, &z, 0.0, arid, true);
        let mid = 5 * d.w + 12;
        assert_eq!(flood.water_z[mid], DRY);
        // Unchanged ground: the flood's levels.
        let same = at_step_end(&d, &z, 0.0, SEA_CELLS, &flood);
        assert_eq!(same.water_z, flood.water_z);
        assert_eq!(same.dry_below_sea, 25);
        for i in 3..=9 {
            z[5 * d.w + i] = -100.0;
        }
        let end = at_step_end(&d, &z, 0.0, SEA_CELLS, &flood);
        assert_eq!(end.water_z[mid], 0.0);
        assert_eq!((end.dry_below_sea, end.wet_above_sea), (0, 0));
        assert_eq!((end.lakes, end.dry), (1, 1));
    }
}
