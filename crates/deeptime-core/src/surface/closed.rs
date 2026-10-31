// SPDX-License-Identifier: LGPL-3.0-or-later
//! What a lake passes on, and internally drained (closed) basins: the surface's rivers take
//! their discharge through a lake as its water balance leaves it (`surface.closed_basins`, on
//! by default since 2026-10-31; ledger ERO-217 to ERO-220; docs/design/plateau-drainage.md).
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
//! - it opens when a wetter climate's balance fills it to the spill, or when a river outside cuts
//!   its rim below the lake ("arid areas in close proximity to large orographic gradients may
//!   represent transitional parts of the landscape in which efficient headward erosion allows
//!   external drainage to extend into the low-precipitation highlands", Sobel et al. 2003, [58]);
//! - a wet basin fills and spills as before, less what its lake evaporates.
//!
//! One rule of its own: a basin its sediment fills opens ("either a climate change to wetter
//! climatic conditions, or the sediment overfill of the lake (both inducing lake level rise) or a
//! basin capture undertaken by an adjacent river can trigger drainage opening", Garcia-Castellanos
//! 2006, pp. 284-285). The balance's level rises with the fill only in its last part (HYD-110
//! raises every lake cell by the same share of its depth, so the lake keeps its cells, and the
//! area its losses need, until the room is nearly gone), and a closed lake passes no water, so the
//! sediment past its room left at the spill with only the spill's own water, settled there and
//! dammed it (the closed-basins review, 2026-10-31). [`open_filled`]: a lake the step's sediment
//! fills to its spill has no room for water either, so it passes all of it, and the step is solved
//! again with its area passed on; the spill is then cut by the basin's water and the sediment
//! past the room is carried on with it. Opening a lake sends its water and its sediment on to the
//! lakes below it, which may fill in turn, so the step is opened and solved again until no lake
//! opens ([`open_until_stable`]): a chain of lakes ends within its depth.
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
//! - A lake the step's sediment fills is open for the whole step (the step is solved again with
//!   its water passed on), though it filled at some time within it; the flexure's water over it
//!   stays the balance's level of the step's start.
//! - A lake is any depression the flood fills by more than `LAKE_TOL_M` (1 cm), one cell or
//!   more, as the solve's lakes are; the bake draws only lakes of 1 km² and a block deep
//!   (`hydro/network.rs`), which is about what the world shows, not the water. A shallow closed
//!   pit has little room, so the step's sediment fills it and it opens ([`open_filled`]); one the
//!   sediment cannot reach stays closed, as a playa would.
//! - The open water's evaporation is the bake's 200 + 60 T+ mm/yr (HYD-040, HYD-041), about half
//!   what Tibet's large lakes lose (650 to 1,300 mm/yr, mean about 1,000; Wang et al. 2020) and
//!   Titicaca's 1,100 (Garcia-Castellanos 2006, p. 284): the model closes fewer high basins than
//!   their measured evaporation would. Not changed here (HYD-040 is calibrated on Earth's lakes as
//!   a whole, Zhao et al. 2022).

use rayon::prelude::*;

use super::grid::SurfaceGrid;
use super::lakes::LakeOutflow;
use super::routing::{self, Routing};

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

/// The drainage area with the lakes' shares passed on: the plain accumulation when every lake
/// passes all of its inflow, else [`routing::accumulate_passing`] with [`pass_shares`].
/// ledger: ERO-218
pub fn accumulate<G: SurfaceGrid>(
    d: &G,
    r: &Routing,
    weight: &[f32],
    outflow: &[LakeOutflow],
    area: &mut Vec<f64>,
) {
    match pass_shares(d.cells(), outflow) {
        Some(pass) => routing::accumulate_passing(d, r, weight, &pass, area),
        None => routing::accumulate(d, r, weight, area),
    }
}

/// How close to the flood's level a lake's exit must stand after the solve to count as filled, m
/// (rounding: the solve raises a filled lake's cells to h0 + 1.0 × (fill − h0)).
const FILLED_TOL_M: f64 = 1.0e-6; // ledger: ERO-217 (numerical)

/// Opens the lakes the step's sediment filled to their spill: a lake passing less than all its
/// inflow (share under 1) whose every exit the solve left at the flood's level, its room taken
/// (`spl.rs`, HYD-110: the lake's cells rise by one share of their depth, all of it once the
/// sediment reaching the lake fills its room). A lake with no room holds no water, so it passes
/// all of it ("the sediment overfill of the lake (both inducing lake level rise) ... can trigger
/// drainage opening", Garcia-Castellanos 2006, pp. 284-285; that source alone). Without it the
/// sediment past the room left at the spill with only the spill's own water, settled there and
/// dammed the spill higher: the next step's flood found a new closed lake behind it (the
/// closed-basins review, 2026-10-31). `h` is the solve's result; returns how many lakes it
/// opened (their share set to 1). One call sees only the lakes this solve filled: see
/// [`open_until_stable`]. ledger: ERO-217
pub fn open_filled(outflow: &mut [LakeOutflow], h: &[f64], filled: &[f64]) -> usize {
    let mut opened = 0;
    for lake in outflow.iter_mut().filter(|l| l.share < 1.0) {
        let full = !lake.exits.is_empty()
            && lake
                .exits
                .iter()
                .all(|&e| filled[e as usize] - h[e as usize] <= FILLED_TOL_M);
        if full {
            lake.share = 1.0;
            opened += 1;
        }
    }
    opened
}

/// Opens the filled lakes and solves again until no lake opens; returns how many it opened in
/// all. `open` is [`open_filled`] on the state's solve (`state` holds the solve's result), and
/// `resolve` accumulates the area again with the new shares ([`accumulate`]) and solves the step
/// again into the state. Opening a lake adds its water and its sediment to the lakes below it,
/// so a lower lake whose room lay between its supplies before and after the upper one opened
/// fills only in the second solve; one call of [`open_filled`] left it closed, its overflow
/// settling on its spill (the chain review, 2026-10-31). Each round opens a lake or ends the
/// loop, and an opened lake is never a candidate again, so it ends within the number of lakes;
/// that bounds the rounds. ledger: ERO-217
pub fn open_until_stable<S>(
    outflow: &mut [LakeOutflow],
    state: &mut S,
    mut open: impl FnMut(&S, &mut [LakeOutflow]) -> usize,
    mut resolve: impl FnMut(&mut S, &[LakeOutflow]),
) -> usize {
    let mut total = 0;
    for _ in 0..outflow.len() {
        let opened = open(state, outflow);
        if opened == 0 {
            break;
        }
        total += opened;
        resolve(state, outflow);
    }
    total
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
        kind: Vec<u8>,
        filled: Vec<f64>,
    }

    fn route(d: &Dims, z: &[f64], bowl: &[bool], water: CellWater, interior: CellWater) -> Routed {
        route_with(d, z, bowl, water, interior, &|_| {})
    }

    /// [`route`], `fix` changing the receivers before the stack is built.
    fn route_with(
        d: &Dims,
        z: &[f64],
        bowl: &[bool],
        water: CellWater,
        interior: CellWater,
        fix: &dyn Fn(&mut routing::Routing),
    ) -> Routed {
        let n = z.len();
        let mut kind = vec![LAND; n];
        pflood::open_sea(d, z, 0.0, 10, &mut kind);
        let base_z = vec![0.0; n];
        let mut filled = vec![0.0; n];
        pflood::flood(d, z, &base_z, &mut kind, &mut filled);
        let mut r = routing::Routing::default();
        routing::receivers(d, &filled, &kind, &mut r);
        fix(&mut r);
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
            kind,
            filled,
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

    /// With shares below 1 the passing accumulation is its definition, area_i = w_i a_i + Σ over
    /// its donors d of pass_d × area_d, summed donors first.
    #[test]
    fn passing_with_shares_is_the_sum_over_donors() {
        let (d, z, bowl) = basin();
        let n = z.len();
        let got = route(&d, &z, &bowl, WET, WET);
        let mut rng = crate::sphere::SplitMix64::new(11);
        let weight: Vec<f32> = (0..n).map(|_| 0.05 + rng.next_f64() as f32).collect();
        let pass: Vec<f32> = (0..n).map(|_| rng.next_f64() as f32).collect();
        let mut fast = Vec::new();
        routing::accumulate_passing(&d, &got.r, &weight, &pass, &mut fast);
        let mut want: Vec<f64> = (0..n)
            .map(|c| f64::from(weight[c]) * d.cell_area())
            .collect();
        for &c in got.r.stack.iter().rev() {
            let (c, to) = (c as usize, got.r.rcv[c as usize] as usize);
            if to != c {
                want[to] += f64::from(pass[c]) * want[c];
            }
        }
        for c in 0..n {
            assert!(
                (fast[c] - want[c]).abs() <= 1.0e-9 * want[c].abs().max(1.0),
                "cell {c}: {} against {}",
                fast[c],
                want[c]
            );
        }
    }

    /// A lake cell draining onto a shoal of its own lake (ground at the fill, so not a lake
    /// cell) that drains back in is no exit: the lake has its one way out over the spill, and the
    /// spill carries its inflow's share once. ledger: ERO-218
    #[test]
    fn a_lake_draining_over_its_own_shoal_has_one_way_out() {
        let (d, mut z, bowl) = basin();
        let w = 24;
        let (shoal, onto, back) = (4 * w + 19, 4 * w + 20, 4 * w + 18);
        z[shoal] = 800.0;
        let got = route_with(&d, &z, &bowl, WET, WET, &|r| {
            r.rcv[onto] = shoal as u32;
            r.rcv[shoal] = back as u32;
        });
        assert_eq!(got.r.rcv[onto] as usize, shoal);
        assert!(
            got.filled[shoal] - z[shoal] <= pflood::LAKE_TOL_M,
            "the shoal is no lake cell"
        );
        assert_eq!(got.outflow.len(), 1, "{:?}", got.outflow);
        let lake = &got.outflow[0];
        assert!(!lake.exits.contains(&(onto as u32)), "{:?}", lake.exits);
        assert!(
            lake.exits
                .iter()
                .all(|&e| got.r.rcv[e as usize] as usize == SPILL),
            "every exit drains over the spill: {:?}",
            lake.exits
        );
        assert!(lake.share > 0.0 && lake.share < 1.0);
        let held = exits_area(&got, lake);
        let want = got.area[SPILL] - (1.0 - lake.share) * held;
        assert!(
            (got.passing[SPILL] - want).abs() < 1.0e-6 * want,
            "{} against {want}",
            got.passing[SPILL]
        );
    }

    /// Two basins in a chain: [`basin`]'s bowl, and east of a ridge a second bowl spilling into
    /// it over a pass at 1,000 m. 32 × 9 cells of 1 km; the upper bowl's depression is the 49 cells
    /// within three rings of (26, 4). Returns the grid, the heights, the upper basin's cells
    /// (columns 23 and up) and the pass.
    fn chain() -> (Dims, Vec<f64>, Vec<bool>, usize) {
        let (w, h) = (32usize, 9usize);
        let d = Dims::new(w, h, 1000.0, false);
        let z: Vec<f64> = (0..w * h)
            .map(|c| {
                let (i, j) = ((c % w) as i64, (c / w) as i64);
                let ring = |x: i64| (i - x).abs().max((j - 4).abs());
                if i < 3 {
                    -1000.0
                } else if i >= 23 {
                    900.0 + 30.0 * ring(26) as f64
                } else if i == 22 {
                    if j == 4 { 1000.0 } else { 1200.0 }
                } else if i >= 15 {
                    600.0 + 40.0 * ring(18) as f64
                } else if j == 4 && (12..=14).contains(&i) {
                    800.0 - 10.0 * (14 - i) as f64
                } else if (12..=14).contains(&i) {
                    1000.0
                } else {
                    20.0 * i as f64
                }
            })
            .collect();
        let upper = (0..w * h).map(|c| c % w >= 23).collect();
        (d, z, upper, 4 * w + 22)
    }

    /// The lower lake takes what the upper one passes on and passes its own share of the whole:
    /// the spill below carries the upper basin's area at both shares, the lower one's at its own;
    /// an arid upper basin passes none, and the lower one, fed by its own land alone, passes a
    /// smaller share of that.
    /// ledger: ERO-217, ERO-218
    #[test]
    fn a_chain_of_lakes_passes_each_share_once() {
        let (d, z, upper, pass) = chain();
        let spill = 4 * 32 + 14;
        let got = route(&d, &z, &upper, WET, WET);
        assert_eq!(got.outflow.len(), 2, "{:?}", got.outflow);
        let by_cells = |k: usize| {
            got.outflow
                .iter()
                .find(|l| l.cells == k)
                .unwrap_or_else(|| panic!("a lake of {k} cells: {:?}", got.outflow))
        };
        let (a, b) = (by_cells(49), by_cells(63));
        assert!(
            a.exits
                .iter()
                .all(|&e| got.r.rcv[e as usize] as usize == pass)
        );
        assert!(a.share > 0.0 && a.share < 1.0, "{}", a.share);
        assert!(b.share > 0.0 && b.share < 1.0, "{}", b.share);
        let (held_a, held_b) = (exits_area(&got, a), exits_area(&got, b));
        let want = got.area[spill] - held_b + b.share * (held_b - (1.0 - a.share) * held_a);
        assert!(
            (got.passing[spill] - want).abs() < 1.0e-6 * want,
            "{} against {want}",
            got.passing[spill]
        );
        // The upper basin arid: it passes nothing, and the lower one, fed by its own land alone,
        // passes a smaller share of that alone.
        let wet_share = b.share;
        let dry = route(&d, &z, &upper, WET, ARID);
        let a = dry
            .outflow
            .iter()
            .find(|l| l.cells == 49)
            .expect("the upper lake");
        let b = dry
            .outflow
            .iter()
            .find(|l| l.cells == 63)
            .expect("the lower lake");
        assert_eq!(a.share, 0.0);
        assert!(
            b.share > 0.0 && b.share < wet_share,
            "{} against {wet_share}",
            b.share
        );
        let (held_a, held_b) = (exits_area(&dry, a), exits_area(&dry, b));
        let want = dry.area[spill] - held_b + b.share * (held_b - held_a);
        assert!(
            (dry.passing[spill] - want).abs() < 1.0e-6 * want,
            "{} against {want}",
            dry.passing[spill]
        );
    }

    /// A closed basin its sediment fills opens (Garcia-Castellanos 2006, pp. 284-285): a 0.5 m
    /// deep arid basin behind a pass at 500 m, below a range that sheds far more than its room in
    /// one 2-Myr step. Closed, the sediment past the room settles on the spill, which carries only
    /// its own water, and dams it; opened ([`open_filled`]), the spill carries the basin's water
    /// and the sediment goes on down the valley. ledger: ERO-217
    #[test]
    fn a_closed_basin_its_sediment_fills_opens() {
        let (w, h) = (20usize, 9usize);
        let d = Dims::new(w, h, 1000.0, false);
        let z: Vec<f64> = (0..w * h)
            .map(|c| {
                let (i, j) = ((c % w) as i64, c / w);
                match i {
                    0..=1 => -1000.0,
                    2..=9 => 400.0 + 10.0 * i as f64,
                    10 if j == 4 => 500.0,
                    10 => 520.0,
                    11..=13 => 499.5,
                    _ => 600.0 + 100.0 * (i - 14) as f64,
                }
            })
            .collect();
        let n = z.len();
        let spill = 4 * w + 10;
        let basin: Vec<bool> = (0..n).map(|c| c % w >= 11).collect();
        let got = route(&d, &z, &basin, WET, ARID);
        let mut outflow = got.outflow.clone();
        let lake = outflow
            .iter()
            .find(|l| l.cells == 27)
            .expect("the basin's lake");
        assert_eq!(lake.share, 0.0, "the arid basin is closed");
        let p = super::super::spl::SplParams {
            m: 1.0,
            n: 2.0,
            g: 1.0,
            dt: 2.0e6,
            sc: 0.15,
            max_iter: 6,
            tol_m: 0.01,
        };
        let kf = vec![1.5e-9; n];
        let base_z = vec![0.0; n];
        let solve = |area: &[f64]| {
            let input = super::super::spl::SplInput {
                kind: &got.kind,
                h0: &z,
                filled: &got.filled,
                base_z: &base_z,
                area,
                kf: &kf,
            };
            let (mut out, mut qin) = (vec![0.0; n], vec![0.0; n]);
            super::super::spl::solve(&d, &got.r, &input, &p, &mut out, &mut qin);
            out
        };
        let closed_h = solve(&got.passing);
        assert_eq!(
            open_filled(&mut outflow, &closed_h, &got.filled),
            1,
            "filled to its spill"
        );
        assert!(pass_shares(n, &outflow).is_none(), "every lake passes all");
        let open_h = solve(&got.area);
        let (dammed, opened) = (closed_h[spill] - z[spill], open_h[spill] - z[spill]);
        eprintln!("the spill's rise: closed {dammed:.1} m, opened {opened:.1} m");
        assert!(
            dammed > 50.0,
            "closed, the overflow dams the spill: {dammed}"
        );
        assert!(
            opened < 0.25 * dammed,
            "opened, it goes on: {opened} against {dammed}"
        );
    }

    /// Two lakes of one depression the shoals at its fill split, a route from one into the other
    /// and back into the first: no exit where it enters the second, whose water comes back. A
    /// ridge of ground at 800 m, the spill's height, across [`basin`]'s bowl makes two lakes at
    /// that level (columns 15 to 17 and 19 to 21); an east cell is sent into a west cell, and that
    /// into another east cell. Walking on only through lakes at the same level (within
    /// `LAKE_TOL_M`), as the solve's lakes are taken (`spl::lake_links`). ledger: ERO-218
    #[test]
    fn a_route_through_a_lake_at_its_own_level_and_back_is_no_exit() {
        let (d, mut z, bowl) = basin();
        let w = 24;
        for j in 0..9 {
            z[j * w + 18] = 800.0;
        }
        let plain = route(&d, &z, &bowl, WET, WET);
        assert_eq!(plain.outflow.len(), 2, "{:?}", plain.outflow);
        assert!(
            (plain.outflow[0].spill_m - plain.outflow[1].spill_m).abs() <= pflood::LAKE_TOL_M,
            "one level: {:?}",
            plain.outflow
        );
        let at = |i: usize, j: usize| j * w + i;
        let (w1, e1) = (at(16, 4), at(20, 4));
        // An east cell whose way down passes neither of the two (no loop of receivers).
        let passes = |from: usize| {
            let mut x = from;
            loop {
                if x == e1 || x == w1 {
                    return true;
                }
                let next = plain.r.rcv[x] as usize;
                if next == x {
                    return false;
                }
                x = next;
            }
        };
        let e2 = (19..=21)
            .flat_map(|i| (1..8).map(move |j| (i, j)))
            .map(|(i, j)| at(i, j))
            .find(|&c| c != e1 && !passes(c))
            .expect("an east cell off the two's way");
        let got = route_with(&d, &z, &bowl, WET, WET, &|r| {
            r.rcv[e1] = w1 as u32;
            r.rcv[w1] = e2 as u32;
        });
        assert_eq!(got.outflow.len(), 2, "{:?}", got.outflow);
        assert!(
            got.outflow
                .iter()
                .all(|l| !l.exits.contains(&(e1 as u32)) && !l.exits.contains(&(w1 as u32))),
            "the loop through the other lake is no exit: {:?}",
            got.outflow
        );
        // The lakes still have their ways out: the real ones, over the spill.
        assert!(got.outflow.iter().all(|l| !l.exits.is_empty()));
    }

    /// The stream power's solve over [`route`]'s routing with `area`, 2 Myr, K 1.5e-9 (the
    /// arrays of [`a_closed_basin_its_sediment_fills_opens`]).
    fn solve_step(d: &Dims, got: &Routed, z: &[f64], area: &[f64]) -> Vec<f64> {
        let n = z.len();
        let p = super::super::spl::SplParams {
            m: 1.0,
            n: 2.0,
            g: 1.0,
            dt: 2.0e6,
            sc: 0.15,
            max_iter: 6,
            tol_m: 0.01,
        };
        let (kf, base_z) = (vec![1.5e-9; n], vec![0.0; n]);
        let input = super::super::spl::SplInput {
            kind: &got.kind,
            h0: z,
            filled: &got.filled,
            base_z: &base_z,
            area,
            kf: &kf,
        };
        let (mut out, mut qin) = (vec![0.0; n], vec![0.0; n]);
        super::super::spl::solve(d, &got.r, &input, &p, &mut out, &mut qin);
        out
    }

    /// Two arid basins in a chain, the upper's pass spilling into the lower, and a range behind
    /// the upper that sheds far more than its room. Closed, the upper basin takes the sediment
    /// it can hold and its pass dams up to the critical slope, so the lower basin gets the rest;
    /// the upper opens ([`open_filled`]) and the lower gets the pass's share too. The lower
    /// basin's floor is set so that its room lies between the two supplies (measured on a basin
    /// too deep to fill): it is not full after the first solve and is full after the second. One
    /// round of opening leaves it closed, its overflow settling on its spill and damming it;
    /// [`open_until_stable`] opens it in the second round and the spill is cut by the basin's
    /// water. ledger: ERO-217
    #[test]
    fn a_lower_basin_the_opened_upper_one_fills_is_opened_in_turn() {
        let (w, h) = (24usize, 9usize);
        let d = Dims::new(w, h, 1000.0, false);
        let n = w * h;
        // The heights, the lower basin's floor given.
        let heights = |floor: f64| -> Vec<f64> {
            (0..n)
                .map(|c| {
                    let (i, j) = ((c % w) as i64, c / w);
                    match i {
                        0..=1 => -1000.0,
                        2..=9 => 400.0 + 10.0 * i as f64,
                        10 if j == 4 => 500.0,
                        10 => 520.0,
                        // The lower basin, its spill at 500 m.
                        11..=13 => floor,
                        // The pass between the basins, over the lower one's floor.
                        14 if j == 4 => 510.0,
                        14 => 515.0,
                        // The upper basin, half a metre under its pass, and the range behind it.
                        15..=17 => 509.5,
                        _ => 610.0 + 100.0 * (i - 18) as f64,
                    }
                })
                .collect()
        };
        let (spill, pass) = (4 * w + 10, 4 * w + 14);
        let bowl: Vec<bool> = (0..n).map(|c| c % w >= 11).collect();
        let weight = vec![1.0f32; n];
        // What the lower basin holds of the sediment after a solve, m³.
        let held = |h: &[f64], z: &[f64]| -> f64 {
            (0..n)
                .filter(|c| (11..=13).contains(&(c % w)))
                .map(|c| (h[c] - z[c]) * d.cell_area())
                .sum()
        };
        // The supply of a lower basin too deep to fill: before the upper opens (it is closed, its
        // pass dammed) and after (its sediment and water go on).
        let deep = heights(200.0);
        let got = route(&d, &deep, &bowl, WET, ARID);
        let first = solve_step(&d, &got, &deep, &got.passing);
        let mut once = got.outflow.clone();
        assert_eq!(open_filled(&mut once, &first, &got.filled), 1);
        let mut area = Vec::new();
        accumulate(&d, &got.r, &weight, &once, &mut area);
        let (before, after) = (
            held(&first, &deep),
            held(&solve_step(&d, &got, &deep, &area), &deep),
        );
        assert!(
            after > before + 5.0e7,
            "the opened upper basin adds to the lower's supply: {before:.3e} and {after:.3e}"
        );
        // The lower basin's room between the two.
        let room = before + 0.5 * (after - before);
        let depth = room / (27.0 * d.cell_area());
        eprintln!(
            "the lower basin's supply {before:.3e} then {after:.3e} m3, its room {room:.3e} ({depth:.1} m)"
        );

        let z = heights(500.0 - depth);
        let got = route(&d, &z, &bowl, WET, ARID);
        assert_eq!(got.outflow.len(), 2, "{:?}", got.outflow);
        assert!(got.outflow.iter().all(|l| l.share == 0.0), "both closed");
        let column = |k: usize| got.outflow[k].exits[0] as usize % w;
        let lower = (0..2).find(|&k| column(k) == 11).expect("the lower");
        let upper = (0..2).find(|&k| column(k) == 15).expect("the upper");
        let full = |h: &[f64], l: &LakeOutflow| {
            l.exits
                .iter()
                .all(|&e| got.filled[e as usize] - h[e as usize] <= FILLED_TOL_M)
        };

        // The first solve fills the upper basin only.
        let first = solve_step(&d, &got, &z, &got.passing);
        assert!(
            full(&first, &got.outflow[upper]),
            "the range fills the upper"
        );
        assert!(!full(&first, &got.outflow[lower]), "and not the lower");

        // One round, as before: the upper opens, the lower fills in the next solve, stays
        // closed, and dams its spill.
        let mut once = got.outflow.clone();
        assert_eq!(open_filled(&mut once, &first, &got.filled), 1);
        assert_eq!(once[upper].share, 1.0);
        accumulate(&d, &got.r, &weight, &once, &mut area);
        let second = solve_step(&d, &got, &z, &area);
        assert!(
            full(&second, &once[lower]),
            "the opened upper fills the lower"
        );
        assert_eq!(once[lower].share, 0.0, "one round leaves it closed");
        let dammed = second[spill] - z[spill];

        // To the end: the lower opens in the second round and the spill is cut.
        let mut outflow = got.outflow.clone();
        let mut state = first.clone();
        let opened = open_until_stable(
            &mut outflow,
            &mut state,
            |h, outflow| open_filled(outflow, h, &got.filled),
            |h, outflow| {
                accumulate(&d, &got.r, &weight, outflow, &mut area);
                *h = solve_step(&d, &got, &z, &area);
            },
        );
        assert_eq!(opened, 2, "both lakes opened");
        assert!(pass_shares(n, &outflow).is_none(), "every lake passes all");
        let cut = state[spill] - z[spill];
        eprintln!(
            "the lower spill's rise: one round {dammed:.1} m, to the end {cut:.1} m (the pass {:.1} m)",
            state[pass] - z[pass]
        );
        assert!(
            dammed > 20.0,
            "one round, the overflow dams the spill: {dammed}"
        );
        assert!(
            cut < 0.25 * dammed,
            "opened, it goes on: {cut} against {dammed}"
        );
    }
}
