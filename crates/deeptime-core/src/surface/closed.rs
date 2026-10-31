// SPDX-License-Identifier: LGPL-3.0-or-later
//! What a lake passes on, and internally drained (closed) basins: the surface's rivers take
//! their discharge through a lake as its water balance leaves it (`surface.closed_basins`, off
//! by default; ledger ERO-217 to ERO-220; docs/design/plateau-drainage.md).
//!
//! # Why
//! The surface stage floods every depression to its spill (`pflood.rs`) and routes it over the
//! spill to the sea (ERO-080), every drop of the water that reaches it with it: no basin on the
//! deep-time surface is internally drained, whatever its climate. On Earth a fifth of the land is
//! ("Endorheic basins occupy 20% of the Earth's land surface but they collect only about 2% of
//! global river runoff, showing that they develop mostly under arid conditions", Garcia-Castellanos
//! et al. 2003, p. 8-1), the high plateaus among it: "Low internal relief, aridity, and internal
//! drainage characterize the Puna-Altiplano Plateau and the Tibetan Plateau" and "Most coupled
//! tectonic-landscape development models prevent the development of internal drainage and so may
//! overestimate the voracity of erosion" (Sobel, Hilley & Strecker 2003, abstract). The plateau
//! interiors erode at 0.02 mm/yr (the Altiplano-Puna, Bookhagen & Strecker 2012, p. 103) and 6 to
//! 16 m/Myr (southern Tibet, Hetzel et al. 2011, p. 983), where the surface's flat high ground
//! wears at 134 to 138 m/Myr (the plateau lane's measure, 2026-10-31).
//!
//! # The rule
//! A lake's level is set by its water balance, as the flexure's lakes already take it
//! (`lakes::water_levels`, ISO-126, the bake's HYD-050 and HYD-051): the runoff of its catchment
//! (the bake's water budget, `hydro::HydroParams::budget`) against what its surface loses, the
//! open water's evaporation less the rain on it. In Langbein 1961's words (p. 4) "I + P A_L =
//! E' A_L" at equilibrium, and "In absence of significant underground flow, the level of a closed
//! lake is determined by the water balance between precipitation and evaporation" (Garcia-
//! Castellanos 2007, p. 387, eq. 12). A lake that fills to its spill passes on what its balance
//! leaves: "the water discharge leaving a lake is determined by the water balance between
//! precipitation and evaporation ... Qw(x = 0) = P . lb - E . ll" (Garcia-Castellanos 2006,
//! pp. 285-286, eq. 2). One that cannot fill passes nothing: "By definition, for a closed lake
//! Qw(x = 0) = 0" (the same page). So with the switch the drainage area the stream power reads
//! (`routing::accumulate`, ERO-082) is passed on from each lake's exit at the share of its inflow
//! that leaves it, and not at all from a closed one: below a closed basin's spill the rivers carry
//! only their own catchment's water, and the spill itself is no longer cut by the basin's.
//!
//! Inside the basin nothing changes: its depression is a lake for the solve (`spl.rs`: no
//! incision, the sediment kept up to its room, HYD-110), the rivers that reach it erode towards its
//! fill, and its ranges shed into it. What follows needs no rule of its own:
//! - the basin fills ("Channels connected to closed basins must eventually aggrade as the basin
//!   fills and base level continually rises", Sobel et al. 2003, [60]), and the eroded rock stays
//!   on the plateau's crust ("Closed drainage (or endorheism) implies that, although some
//!   precipitation occurs in the highlands, its effect in terms of mass removal from the orogen is
//!   null", Garcia-Castellanos 2007, p. 380);
//! - it opens when its balance fills it to the spill (a wetter climate, or fill that shrinks the
//!   room the water has, Garcia-Castellanos 2006, pp. 284-285), or when a river outside cuts its
//!   rim below the lake ("arid areas in close proximity to large orographic gradients may represent
//!   transitional parts of the landscape in which efficient headward erosion allows external
//!   drainage to extend into the low-precipitation highlands", Sobel et al. 2003, [58]);
//! - a wet basin fills and spills as before, less what its lake evaporates.
//!
//! # Simplifications (the agent's, 2026-10-31)
//! - The depression is the solve's lake up to its spill whatever the level of the water in it, as
//!   every lake is today; the balance's level decides only what the lake passes on. In Garcia-
//!   Castellanos 2006 (p. 288) a closed lake's surface stands below the outlet and the floor
//!   between the two receives the rivers; at 4.8-km cells and 2-Myr steps that floor is the
//!   aggrading basin of the sources, so the solve's lake stands for it.
//! - The share is the balance's (the bake's water budget), applied to the stream power's
//!   runoff-weighted area (ERO-010, the surface's own runoff): the two runoffs differ on frozen
//!   ground (the surface counts all of P as runoff below 0 °C, the bake only melted snow).
//! - What a closed lake cannot keep (sediment beyond its room) still leaves at its exit, as from any
//!   lake: the overfilled basin builds its fan at the spill.
//! - The open water's evaporation is the bake's 200 + 60 T+ mm/yr (HYD-040, HYD-041), about half
//!   what Tibet's large lakes lose (650 to 1,300 mm/yr, mean about 1,000; Wang et al. 2020) and
//!   Titicaca's 1,100 (Garcia-Castellanos 2006, p. 284): the model closes fewer high basins than
//!   their measured evaporation would. Not changed here (HYD-040 is calibrated on Earth's lakes as
//!   a whole, Zhao et al. 2022).

use rayon::prelude::*;

use super::lakes::LakeOutflow;
use super::routing::Routing;

/// The share of its drainage area each cell passes on to its receiver (`routing::
/// accumulate_passing`): 1 everywhere but at the exits of lakes that lose water, which pass their
/// lake's [`LakeOutflow::share`]; None when every share is 1. ledger: ERO-218
pub fn pass_shares(n: usize, outflow: &[LakeOutflow]) -> Option<Vec<f32>> {
    let mut pass: Option<Vec<f32>> = None;
    for lake in outflow.iter().filter(|l| l.share < 1.0) {
        let p = pass.get_or_insert_with(|| vec![1.0; n]);
        for &e in &lake.exits {
            p[e as usize] = lake.share as f32;
        }
    }
    pass
}

/// Per cell: whether its water ends in a closed basin, the cell itself an exit of a lake that
/// passes nothing or every path from it reaching one before the sea (measurement only: the
/// erosion bands' closed flat high ground, the run's closed catchment).
pub fn closed_catchment(r: &Routing, outflow: &[LakeOutflow]) -> Vec<bool> {
    let n = r.rcv.len();
    let mut out = vec![false; n];
    for lake in outflow.iter().filter(|l| l.share <= 0.0) {
        for &e in &lake.exits {
            out[e as usize] = true;
        }
    }
    // The stack holds every cell after its receiver.
    for &c in &r.stack {
        let c = c as usize;
        let to = r.rcv[c] as usize;
        if to != c && out[to] {
            out[c] = true;
        }
    }
    out
}

/// The run's counts: lakes passing nothing, lakes passing part of their inflow, and the land
/// cells whose water ends in a closed basin.
pub fn counts(outflow: &[LakeOutflow], catchment: &[bool], land: &[bool]) -> (u64, u64, u64) {
    let closed = outflow.iter().filter(|l| l.share <= 0.0).count() as u64;
    let partial = outflow
        .iter()
        .filter(|l| l.share > 0.0 && l.share < 1.0)
        .count() as u64;
    let cells = catchment
        .par_iter()
        .zip(land.par_iter())
        .filter(|(c, l)| **c && **l)
        .count() as u64;
    (closed, partial, cells)
}

#[cfg(test)]
mod tests {
    use super::super::grid::Dims;
    use super::super::lakes::{self, CellWater};
    use super::super::pflood::{self, LAND};
    use super::super::routing;
    use super::*;

    /// The spill of [`basin`]'s bowl.
    const SPILL: usize = 4 * 24 + 14;

    /// A strip of sea on the west, land rising east to a range at 1,000 m (columns 12 to 14) and
    /// behind it a basin whose floor rises 40 m a ring from 600 m at (17, 4) to the grid's east,
    /// north and south edges: 24 × 9 cells of 1 km. The range's one pass, the spill, is (14, 4) at
    /// 800 m, with (13, 4) and (12, 4) at 790 and 780 m leading west. The basin's depression is the
    /// 63 cells of columns 15 to 21; columns 22 and 23 drain into it. Returns the grid, the
    /// heights and the basin's cells (columns 15 and up).
    fn basin() -> (Dims, Vec<f64>, Vec<bool>) {
        let (w, h) = (24usize, 9usize);
        let d = Dims::new(w, h, 1000.0, false);
        let z: Vec<f64> = (0..w * h)
            .map(|c| {
                let (i, j) = ((c % w) as i64, (c / w) as i64);
                let ring = (i - 17).abs().max((j - 4).abs());
                if i < 3 {
                    -1000.0
                } else if i >= 15 {
                    600.0 + 40.0 * ring as f64
                } else if j == 4 && (12..=14).contains(&i) {
                    800.0 - 10.0 * (14 - i) as f64
                } else if (12..=14).contains(&i) {
                    1000.0
                } else {
                    20.0 * i as f64
                }
            })
            .collect();
        let basin = (0..w * h).map(|c| c % w >= 15).collect();
        (d, z, basin)
    }

    struct Routed {
        r: routing::Routing,
        outflow: Vec<LakeOutflow>,
        area: Vec<f64>,
        passing: Vec<f64>,
    }

    fn route(d: &Dims, z: &[f64], bowl: &[bool], water: CellWater, interior: CellWater) -> Routed {
        let n = z.len();
        let mut kind = vec![LAND; n];
        pflood::open_sea(d, z, 0.0, 10, &mut kind);
        let base_z = vec![0.0; n];
        let mut filled = vec![0.0; n];
        pflood::flood(d, z, &base_z, &mut kind, &mut filled);
        let mut r = routing::Routing::default();
        routing::receivers(d, &filled, &kind, &mut r);
        routing::stack(d, &kind, &mut r);
        let weight = vec![1.0f32; n];
        let mut area = Vec::new();
        routing::accumulate(d, &r, &weight, &mut area);
        let lv = lakes::water_levels(d, z, &filled, &kind, &r, 0.0, true, &|c| {
            if bowl[c] { interior } else { water }
        });
        let pass = pass_shares(n, &lv.outflow).unwrap_or_else(|| vec![1.0; n]);
        let mut passing = Vec::new();
        routing::accumulate_passing(d, &r, &weight, &pass, &mut passing);
        Routed {
            r,
            outflow: lv.outflow,
            area,
            passing,
        }
    }

    /// A humid climate: 0.5 m of runoff, a lake losing 0.1 m a year net of the rain on it.
    // ledger: ERO-219
    const WET: CellWater = CellWater {
        runoff_m_yr: 0.5,
        loss_m_yr: 0.1,
    };
    /// An arid one: 20 mm of runoff, a lake losing 1 m a year (Langbein 1961, p. 18, Table 1: the
    /// net evaporation of closed lakes in arid basins, 2.0 to 7.0 ft a year).
    // ledger: ERO-219
    const ARID: CellWater = CellWater {
        runoff_m_yr: 0.02,
        loss_m_yr: 1.0,
    };

    /// The area the lake's exits hand to the spill (`area` at each exit, summed).
    fn exits_area(got: &Routed, lake: &LakeOutflow) -> f64 {
        lake.exits.iter().map(|&e| got.area[e as usize]).sum()
    }

    /// An arid basin cannot fill its depression to the spill: its lake passes nothing, so the
    /// spill and the river below it carry only their own catchment, and every cell of the basin
    /// drains to it. ledger: ERO-217, ERO-218
    #[test]
    fn an_arid_basin_is_closed_and_passes_no_water() {
        let (d, z, basin_cells) = basin();
        let got = route(&d, &z, &basin_cells, WET, ARID);
        assert_eq!(got.outflow.len(), 1, "{:?}", got.outflow);
        let lake = &got.outflow[0];
        assert_eq!(lake.share, 0.0);
        assert_eq!(lake.cells, 63);
        assert!(lake.level_m < lake.spill_m && (lake.spill_m - 800.0).abs() < 1.0e-6);
        let held = exits_area(&got, lake);
        assert!(held >= 81.0 * d.cell_area() - 1.0, "the 81 basin cells");
        // Without the rule the spill carries the basin; with it, only its own catchment.
        assert!(got.area[SPILL] > held);
        let own = got.area[SPILL] - held;
        assert!(
            (got.passing[SPILL] - own).abs() < 1.0e-9 * got.area[SPILL],
            "{} against {}",
            got.passing[SPILL],
            own
        );
        let catchment = closed_catchment(&got.r, &got.outflow);
        for c in 0..z.len() {
            if basin_cells[c] {
                assert!(catchment[c], "basin cell {c} drains to the closed lake");
            }
        }
        assert!(!catchment[SPILL], "the spill drains to the sea");
    }

    /// A humid basin fills and spills: it passes its inflow less what its lake loses (a share
    /// under 1), and the river below carries that share of the basin's area. ledger: ERO-218
    #[test]
    fn a_humid_basin_spills_less_its_lake_s_evaporation() {
        let (d, z, basin_cells) = basin();
        let got = route(&d, &z, &basin_cells, WET, WET);
        assert_eq!(got.outflow.len(), 1, "{:?}", got.outflow);
        let lake = &got.outflow[0];
        assert_eq!(lake.level_m, lake.spill_m);
        let held = exits_area(&got, lake);
        // The inflow is the runoff of the land draining to the lake (all of it but the lake's own
        // 63 cells, at 0.5 m), the loss 63 cells at 0.1 m.
        let land = (held / d.cell_area()).round() - 63.0;
        let (inflow, loss) = (land * 0.5, 63.0 * 0.1);
        assert!(land > 12.0 && inflow > loss, "{land}");
        assert!(
            (lake.share - (inflow - loss) / inflow).abs() < 1.0e-9,
            "{} against {}",
            lake.share,
            (inflow - loss) / inflow
        );
        let want = got.area[SPILL] - (1.0 - lake.share) * held;
        // The share goes to the accumulation as an f32.
        assert!((got.passing[SPILL] - want).abs() < 1.0e-6 * want);
        assert!(closed_catchment(&got.r, &got.outflow).iter().all(|&c| !c));
    }

    /// With every share 1 the passing accumulation is the plain one to the bit.
    #[test]
    fn passing_everything_is_the_plain_accumulation() {
        let (d, z, bowl) = basin();
        let n = z.len();
        let got = route(&d, &z, &bowl, WET, WET);
        let mut again = Vec::new();
        routing::accumulate_passing(&d, &got.r, &vec![1.0; n], &vec![1.0; n], &mut again);
        assert_eq!(
            again.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            got.area.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
    }
}
