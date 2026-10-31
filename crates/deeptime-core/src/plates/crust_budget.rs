// SPDX-License-Identifier: LGPL-3.0-or-later
//! The continental crust budget: measurement only, no rule reads it (build log 2026-10-19,
//! "mountains eat the world": where the continents' area and volume go, and why their land
//! drowns).
//!
//! # What it measures
//! The stock of continental crust over a run: its area (visible continental samples, as a share
//! of the planet) and its volume (Σ thickness, visible and hidden crust apart, km³
//! Earth-normalized); the land (visible samples above the plate datum, 0 m, and above the
//! climate's sea level); the dry share of the continental crust (above the datum, `dry`, the
//! stock the parts and the moves book, and above the sea, `dry_sl`); its thickness distribution,
//! each band with its elevation (the freeboard), its Airy height
//! ([`super::collapse::iso_height`]) and its share above the sea;
//! the hypsometry (the land's share above 2 km) and the median distance from land to the coast.
//!
//! # How
//! Every process of a plate step is bracketed as for the gap budget (`gap_budget.rs`, the same
//! [`Part`]s): after it, the change of each stock goes to that process, and the changes of state
//! of single samples are counted (ocean floor made continental and back, with the elevation and
//! thickness of crust made continental; continental crust hidden under another plate or
//! surfacing; dry continental crust drowned, wet continental crust emerging). The surface stage's
//! feedback goes to the `erosion` part (all of its change: erosion, deposition, isostasy,
//! flexure, the sea), a resampling to `resample_blend`. A snapshot every
//! [`super::GAP_SNAPSHOT_MYR`] and at the end. With [`MAPS_ENV`] set to a folder, every snapshot
//! also writes the visible samples there (`t<Myr>.bin`, little-endian f32 records of
//! [`MAP_FIELDS`]: latitude and longitude in degrees, elevation (m), thickness (m), kind (0
//! oceanic, 1 continental), orogeny (0 none, 1 Andean, 2 Himalayan), the orogeny's age (Myr), the
//! island-arc and continental-arc magma (m), the plate group, the flags: the sample's, with
//! [`MAP_FRONT_SUBDUCTION`] or [`MAP_FRONT_COLLISION`] added on the overriding side of the step's
//! fronts as the dynamics record's boundary mix classed them, when the record is on and the
//! snapshot is not at a resampling) for
//! `tools/review/continent-budget.py`, and the same samples' plate ids to
//! `plate/t<Myr>.bin` (little-endian u32, one per record; [`CrustBudget::write_plates`]) for the
//! front lane's boundary geometry (`tools/review/pool/measure_boundgeom.py`), their sea-floor
//! ages to `age/t<Myr>.bin` (little-endian f32, one per record, NaN on continental crust;
//! [`CrustBudget::write_ages`]) for the floor-age lane (`tools/review/floorage/`), and their rock
//! pools to `pools/t<Myr>.bin` ([`POOL_FIELDS`]). Each snapshot also splits the land by the origin of its
//! crust (`cb_origin`: island arc, collision, Andean margin, the rest).
//!
//! Off unless [`ON_ENV`] is set; it changes nothing the model does.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use glam::DVec3;
use rayon::prelude::*;

use super::collapse::iso_height;
use super::gap_budget::{NAMES, PARTS, Part};
use super::geometry::Tree;
use super::state::{OCEANIC, OROGENY_ANDEAN, OROGENY_HIMALAYAN, Sample, TRENCH, pool};

/// Set to switch the budget on.
pub const ON_ENV: &str = "DEEPTIME_CRUST_BUDGET";
/// Set to a folder to have every snapshot write the visible samples there.
pub const MAPS_ENV: &str = "DEEPTIME_CRUST_BUDGET_MAPS";

/// Whether the environment asks for the budget (`PlatesStage::new` reads it).
pub fn enabled() -> bool {
    std::env::var_os(ON_ENV).is_some()
}

/// The stocks, in order: visible continental samples, hidden continental samples, their volumes
/// (Σ thickness less the ocean crust under crust made continental, `Sample::own_thickness_m`, m;
/// ledger PLT-331), visible samples above 0 m, visible continental samples above 0 m, Σ
/// elevation of the visible continental samples (m), visible samples, and the felsic rock (the
/// conserved pools, `pool::CONSERVED`) of every sample, visible and hidden, and of the oceanic
/// ones alone (m): a part's change of the first is felsic rock made or destroyed, of the second
/// felsic rock moved onto or off the ocean floor (the loss budget against Stern & Scholl 2010,
/// `tools/review/loss-budget.py`).
const STOCKS: [&str; 14] = [
    "cont",
    "cont_hidden",
    "vol",
    "vol_hidden",
    "land",
    "dry",
    "z_cont",
    "vis",
    "felsic",
    "felsic_ocean",
    "sed_tr",
    "sed_ov",
    "sed_oh",
    "sed_c",
];
const NT: usize = STOCKS.len();
const CONT: usize = 0;
const CONT_HIDDEN: usize = 1;
const VOL: usize = 2;
const VOL_HIDDEN: usize = 3;
const LAND: usize = 4;
const DRY: usize = 5;
const FELSIC: usize = 8;
const FELSIC_OCEAN: usize = 9;
/// The sediment and sedimentary rock (`pool::SEDIMENT`, `pool::SEDIMENTARY`, m): on the visible
/// ocean floor at the trenches (the TRENCH flag: the incoming plate at its front this step), on
/// all the visible ocean floor, on the hidden ocean floor (gone under, until a resampling), and
/// on continental crust (visible and hidden). Measurement only (the crust-budget lane's trench
/// trace, 2026-10-31): the parts' changes of these are the sediment's mass balance.
const SED_TR: usize = 10;
const SED_OV: usize = 11;
const SED_OH: usize = 12;
const SED_C: usize = 13;
/// Set to print the sediment stocks' changes by part every step (`cb_step` lines).
pub const STEPS_ENV: &str = "DEEPTIME_CRUST_BUDGET_STEPS";
/// The globals each `cb_step` line gives the step's change of (km³ Earth-normalized).
const STEP_GLOBALS: [(&str, &str); 6] = [
    ("sed_trench", super::ridges::SEDIMENT_REACHING_TRENCH_GLOBAL),
    ("sed_subducted", super::ridges::SEDIMENT_SUBDUCTED_GLOBAL),
    ("accreted", super::ridges::ACCRETED_GLOBAL),
    ("slab", super::ridges::SLAB_GLOBAL),
    ("forearc_c", super::subduction::FOREARC_CONTINENTAL_GLOBAL),
    ("forearc_o", super::subduction::FOREARC_OTHER_GLOBAL),
];

/// The changes of state counted per sample: ocean floor made continental (count, Σ elevation,
/// Σ thickness of the continental crust made, m), continental crust made oceanic, hidden,
/// surfacing, drowned, emerging; and the ocean floor's own crust kept under the crust made
/// continental (Σ `Sample::basement`, m).
const MOVES: [&str; 9] = [
    "o2c", "o2c_z", "o2c_t", "c2o", "hide", "surface", "drown", "emerge", "o2c_b",
];
const NX: usize = MOVES.len();

const BIT_CONT: u8 = 1;
const BIT_HIDDEN: u8 = 2;
const BIT_DRY: u8 = 4;

/// The thickness bands of the snapshots, km.
// ledger: PLT-300 (game-choice: reporting bands, read by no rule; agent, 2026-10-19).
pub const EDGES_KM: [f64; 10] = [
    0.0,
    20.0,
    25.0,
    30.0,
    35.0,
    40.0,
    45.0,
    55.0,
    70.0,
    f64::INFINITY,
];

/// The globals each snapshot copies (cumulative over the run, km³ or km² Earth-normalized).
const GLOBALS: [(&str, &str); 99] = [
    ("flow_steps", super::flow::STEPS_GLOBAL),
    ("flow_substeps", super::flow::SUBSTEPS_GLOBAL),
    ("flow_short_myr", super::flow::SHORT_GLOBAL),
    ("flow_edge", super::flow::EDGE_GLOBAL),
    ("foundered", super::collapse::FOUNDERED_GLOBAL),
    ("rate_lost", super::rate::LOST_GLOBAL),
    ("crossed", super::belts::CROSSED_GLOBAL),
    ("kept", super::belts::KEPT_GLOBAL),
    ("crossed_km2", super::belts::CROSSED_AREA_GLOBAL),
    ("front_km_myr", super::belts::FRONT_GLOBAL),
    ("crossed_above", super::belts::ABOVE_GLOBAL),
    ("crossed_recent", super::belts::RECENT_GLOBAL),
    ("kept_wedge", super::belts::WEDGE_GLOBAL),
    ("crossed_restack", super::belts::RESTACK_GLOBAL),
    ("noroom_sliver", super::belts::SLIVER_GLOBAL),
    ("noroom_capped", super::belts::CAPPED_GLOBAL),
    ("noroom_clipped", super::belts::CLIPPED_GLOBAL),
    ("noroom_stays", super::belts::STAYS_GLOBAL),
    ("taper_toe", super::belts::TOE_GLOBAL),
    // Terranes docking stripped to their cover (ledger: PLT-770; zero without the switch).
    ("stripped", super::belts::STRIPPED_GLOBAL),
    // The two wedges' retro plateau beyond the belt's reach (ledger: PLT-740; zero without the
    // switch).
    ("plateau", super::belts::PLATEAU_GLOBAL),
    // The overriding plate's share (ledger: PLT-652; zero without the switch).
    ("upper", super::belts::UPPER_GLOBAL),
    ("upper_km2", super::belts::UPPER_AREA_GLOBAL),
    ("upper_most_km2", super::belts::UPPER_MOST_GLOBAL),
    ("accreted", super::ridges::ACCRETED_GLOBAL),
    ("stacked", super::ridges::STACKED_CONSUMED_GLOBAL),
    (
        "accreted_continent",
        super::ridges::ACCRETED_HIDDEN_CONTINENT_GLOBAL,
    ),
    ("dropped", super::ridges::DROPPED_GLOBAL),
    ("mafic_dropped", super::ridges::MAFIC_DROPPED_GLOBAL),
    ("collapse_moved", super::collapse::MOVED_GLOBAL),
    ("collapse_gained_km2", super::collapse::GAINED_GLOBAL),
    ("collapse_held", super::collapse::HELD_GLOBAL),
    ("collapse_unplaced", super::collapse::UNPLACED_GLOBAL),
    ("channel_flow", super::flow::MOVED_GLOBAL),
    ("settle_rise", super::settle::RISE_GLOBAL),
    ("settle_fall", super::settle::FALL_GLOBAL),
    ("arc_c", super::subduction::ARC_CONTINENTAL_GLOBAL),
    ("arc_o", super::subduction::ARC_OCEANIC_GLOBAL),
    ("cordillera", super::subduction::CORDILLERA_GLOBAL),
    ("cone_felsic", super::CONE_FELSIC_GLOBAL),
    ("front_o", super::subduction::ARC_FRONT_OCEANIC_GLOBAL),
    ("front_c", super::subduction::ARC_FRONT_CONTINENTAL_GLOBAL),
    ("front_g_full", super::subduction::FRONT_G_FULL_GLOBAL),
    ("front_g_normal", super::subduction::FRONT_G_NORMAL_GLOBAL),
    (
        "front_gc_full",
        super::subduction::FRONT_G_CONTINENTAL_FULL_GLOBAL,
    ),
    (
        "front_gc_normal",
        super::subduction::FRONT_G_CONTINENTAL_NORMAL_GLOBAL,
    ),
    ("transition_moved", super::transition::MOVED_GLOBAL),
    ("margins_left", super::transition::MARGINS_LEFT_GLOBAL),
    ("sed_subducted", super::ridges::SEDIMENT_SUBDUCTED_GLOBAL),
    // Measurement only: the floor's sediment reaching the trenches, before the share the slab
    // takes, by the crust on top (the sediment diagnosis, tools/review/seddiag/).
    ("sed_trench", super::ridges::SEDIMENT_REACHING_TRENCH_GLOBAL),
    (
        "sed_trench_c",
        super::ridges::SEDIMENT_REACHING_TRENCH_CONTINENTAL_GLOBAL,
    ),
    (
        "sed_trench_o",
        super::ridges::SEDIMENT_REACHING_TRENCH_OCEANIC_GLOBAL,
    ),
    // The arcs' fronts' length × dt × normal convergence (PLT-665): over front_c and front_o,
    // the mean v_n at the arcs.
    (
        "front_n_c",
        super::subduction::ARC_FRONT_NORMAL_CONTINENTAL_GLOBAL,
    ),
    (
        "front_n_o",
        super::subduction::ARC_FRONT_NORMAL_OCEANIC_GLOBAL,
    ),
    // The arcs' fronts younger than the island arcs' early pulse (PLT-670): over front_c and
    // front_o, the share of the fronts the pulse acts on.
    (
        "front_young_c",
        super::subduction::ARC_FRONT_YOUNG_CONTINENTAL_GLOBAL,
    ),
    (
        "front_young_o",
        super::subduction::ARC_FRONT_YOUNG_OCEANIC_GLOBAL,
    ),
    ("forearc_c", super::subduction::FOREARC_CONTINENTAL_GLOBAL),
    ("forearc_o", super::subduction::FOREARC_OTHER_GLOBAL),
    ("forearc_short", super::subduction::FOREARC_SHORT_GLOBAL),
    (
        "forearc_front_c",
        super::subduction::FOREARC_FRONT_CONTINENTAL_GLOBAL,
    ),
    (
        "forearc_front_o",
        super::subduction::FOREARC_FRONT_OCEANIC_GLOBAL,
    ),
    ("hotspot_magma", super::hotspots::MAGMA_GLOBAL),
    ("rate_capped", super::rate::CAPPED_GLOBAL),
    ("accreted_visible", super::ridges::ACCRETED_VISIBLE_GLOBAL),
    ("cone_mafic", super::CONE_MAFIC_GLOBAL),
    ("slab", super::ridges::SLAB_GLOBAL),
    // The fronts' lengths as the arcs and the slab pull count them (ledger: PLT-530, PLT-554).
    (
        "front_straight",
        super::subduction::ARC_FRONT_STRAIGHT_GLOBAL,
    ),
    ("arc_band", super::subduction::ARC_BAND_GLOBAL),
    ("slab_lattice", super::forces::SLAB_FRONT_LATTICE_GLOBAL),
    ("slab_normal", super::forces::SLAB_FRONT_NORMAL_GLOBAL),
    ("slab_straight", super::forces::SLAB_FRONT_STRAIGHT_GLOBAL),
    // The additions against Stern & Scholl 2010's Table 1 (ledger: PLT-761, PLT-760): the felsic
    // cones the arcs counted inside their rates, by the arc's law; the hotspots' basalt on
    // continental crust and the cones they counted inside theirs; the intra-oceanic arcs' front.
    (
        "arc_cones_c",
        super::subduction::ARC_CONES_CONTINENTAL_GLOBAL,
    ),
    ("arc_cones_o", super::subduction::ARC_CONES_OCEANIC_GLOBAL),
    ("arc_s", super::subduction::ARC_SUTURE_GLOBAL),
    (
        "front_io",
        super::subduction::ARC_FRONT_INTRA_OCEANIC_GLOBAL,
    ),
    ("hotspot_magma_c", super::hotspots::MAGMA_CONTINENTAL_GLOBAL),
    ("hotspot_cones", super::hotspots::CONES_GLOBAL),
    ("hotspot_cones_c", super::hotspots::CONES_CONTINENTAL_GLOBAL),
    // Where the arcs' magma went by law, setting and crust (measurement, the trace of PLT-760).
    ("split_c_io_cont", super::subduction::ARC_SPLIT_GLOBALS[0]),
    ("split_c_io_io_no", super::subduction::ARC_SPLIT_GLOBALS[1]),
    ("split_c_io_io_yes", super::subduction::ARC_SPLIT_GLOBALS[2]),
    ("split_c_other", super::subduction::ARC_SPLIT_GLOBALS[3]),
    ("split_s_io", super::subduction::ARC_SPLIT_GLOBALS[4]),
    ("split_s_other", super::subduction::ARC_SPLIT_GLOBALS[5]),
    ("split_o_cont", super::subduction::ARC_SPLIT_GLOBALS[6]),
    // The hotspot lane (ledger: PLT-795, PLT-799, PLT-800): the plume heads' large igneous provinces, all
    // and on continental crust, their count, and the volcanic rifted margins' continental crust
    // and gap points.
    ("lip", super::hotspots::LIP_GLOBAL),
    ("lip_c", super::hotspots::LIP_CONTINENTAL_GLOBAL),
    ("lip_dome_c", super::hotspots::LIP_DOME_CONTINENTAL_GLOBAL),
    ("lip_n", super::hotspots::LIP_COUNT_GLOBAL),
    ("vrm", super::ridges::VOLCANIC_MARGIN_GLOBAL),
    ("vrm_n", super::ridges::VOLCANIC_MARGIN_SAMPLES_GLOBAL),
    // The cone budget (ledger: PLT-803, PLT-804): the cones the continental crust took; the
    // cones the tails netted by their hotspot, and on continental cells; the tails' rock; what
    // dead tails' cones handed that none netted; the hotspots' cones at full, as handed, and
    // handed on continental cells (the volcano stage).
    ("cone_c", super::CONE_CONTINENTAL_GLOBAL),
    ("hot_by_id", super::hotspots::CONES_BY_ID_GLOBAL),
    (
        "hot_by_id_c",
        super::hotspots::CONES_BY_ID_CONTINENTAL_GLOBAL,
    ),
    ("tail_rock", super::hotspots::TAIL_ROCK_GLOBAL),
    ("hot_unnetted", super::hotspots::CONES_UNNETTED_GLOBAL),
    (
        "vh_built",
        crate::surface::volcanoes::HOTSPOT_CONES_BUILT_GLOBAL,
    ),
    (
        "vh_handed",
        crate::surface::volcanoes::HOTSPOT_CONES_HANDED_GLOBAL,
    ),
    (
        "vh_handed_c",
        crate::surface::volcanoes::HOTSPOT_CONES_HANDED_CONTINENTAL_GLOBAL,
    ),
];

/// Earth's radius, km: the distances to the coast are Earth-normalized.
const EARTH_KM: f64 = 6371.0;

/// The origins the snapshots split the land by (see [`origin`]).
const ORIGINS: usize = 4;
const ORIGIN_NAMES: [&str; ORIGINS] = ["island_arc", "collision", "andean", "other"];

/// Where a sample's crust came from: island-arc crust (more than 1 km of island-arc magma, more
/// than of continental-arc magma, as [`super::dynamics::is_island_arc`]), else a collision's belt,
/// else an Andean margin, else the rest (measurement only).
fn origin(s: &Sample) -> usize {
    if s.arc_oceanic_m >= 1000.0 && s.arc_oceanic_m > s.arc_continental_m {
        0
    } else if s.orogeny == OROGENY_HIMALAYAN {
        1
    } else if s.orogeny == OROGENY_ANDEAN {
        2
    } else {
        3
    }
}

fn bits(s: &Sample) -> u8 {
    let mut b = 0;
    if s.is_continental() {
        b |= BIT_CONT;
    }
    if s.is_hidden() {
        b |= BIT_HIDDEN;
    }
    if s.elevation_m > 0.0 {
        b |= BIT_DRY;
    }
    b
}

fn stocks(samples: &[Sample]) -> [f64; NT] {
    samples
        .par_iter()
        .fold(
            || [0.0f64; NT],
            |mut a, s| {
                let t = f64::from(s.own_thickness_m());
                let dry = s.elevation_m > 0.0;
                match (s.is_continental(), s.is_hidden()) {
                    (true, false) => {
                        a[CONT] += 1.0;
                        a[VOL] += t;
                        a[DRY] += f64::from(u8::from(dry));
                        a[6] += f64::from(s.elevation_m);
                    }
                    (true, true) => {
                        a[CONT_HIDDEN] += 1.0;
                        a[VOL_HIDDEN] += t;
                    }
                    _ => {}
                }
                if !s.is_hidden() {
                    a[LAND] += f64::from(u8::from(dry));
                    a[7] += 1.0;
                }
                let felsic: f64 = s.pools[..pool::CONSERVED]
                    .iter()
                    .map(|&x| f64::from(x))
                    .sum();
                a[FELSIC] += felsic;
                if !s.is_continental() {
                    a[FELSIC_OCEAN] += felsic;
                }
                let sed =
                    f64::from(s.pools[pool::SEDIMENT]) + f64::from(s.pools[pool::SEDIMENTARY]);
                match (s.is_continental(), s.is_hidden()) {
                    (true, _) => a[SED_C] += sed,
                    (false, true) => a[SED_OH] += sed,
                    (false, false) => {
                        a[SED_OV] += sed;
                        if s.flags & TRENCH != 0 {
                            a[SED_TR] += sed;
                        }
                    }
                }
                a
            },
        )
        .reduce(
            || [0.0; NT],
            |mut a, b| {
                for (x, y) in a.iter_mut().zip(b) {
                    *x += y;
                }
                a
            },
        )
}

/// The budget (`PlatesStage`, while [`ON_ENV`] is set).
#[derive(Clone, Debug)]
pub struct CrustBudget {
    z_0: f64,
    gravity: f64,
    /// The stocks at the last booking.
    last: [f64; NT],
    /// Every sample's state bits at the last booking.
    bits: Vec<u8>,
    /// Per part, the change of each stock over the run.
    parts: Vec<[f64; NT]>,
    /// Per part, the changes of state over the run.
    moves: Vec<[f64; NX]>,
    /// The snapshots' lines.
    lines: Vec<String>,
    maps: Option<PathBuf>,
    /// With [`STEPS_ENV`], the parts' changes and the step globals at the last step's end.
    steps: Option<(Vec<[f64; NT]>, [f64; STEP_GLOBALS.len()])>,
    /// The collision fronts by what stands on either side (`dock_tally.rs`, measurement only),
    /// printed after the budget's lines.
    pub dock: super::dock_tally::DockTally,
}

impl CrustBudget {
    /// Starts the budget on the run's first samples.
    pub fn start(samples: &[Sample], z_0: f64, gravity: f64) -> CrustBudget {
        CrustBudget {
            z_0,
            gravity,
            last: stocks(samples),
            bits: samples.par_iter().map(bits).collect(),
            parts: vec![[0.0; NT]; PARTS],
            moves: vec![[0.0; NX]; PARTS],
            lines: Vec::new(),
            maps: std::env::var_os(MAPS_ENV).map(PathBuf::from),
            steps: std::env::var_os(STEPS_ENV)
                .map(|_| (vec![[0.0; NT]; PARTS], [f64::NAN; STEP_GLOBALS.len()])),
            dock: Default::default(),
        }
    }

    /// Whether no snapshot was taken yet.
    pub fn fresh(&self) -> bool {
        self.lines.is_empty()
    }

    /// Books the stocks' changes since the last booking to `part`, and starts the samples' states
    /// afresh: for a resampling, whose new lattice has the same number of samples at other places,
    /// so sample i before and after is not one piece of crust. (Until 2026-10-20 a resampling was
    /// booked with [`CrustBudget::book`], which compared sample i of the old lattice with sample i
    /// of the new: about c (1 - c) of the samples counted as made oceanic or continental at each
    /// resampling, the "15.9 of the planet's worth" of the build log's 2026-10-20 entry.)
    pub fn book_lattice(&mut self, samples: &[Sample], part: Part) {
        let now = stocks(samples);
        let p = part as usize;
        for (k, x) in self.parts[p].iter_mut().enumerate() {
            *x += now[k] - self.last[k];
        }
        self.last = now;
        self.bits = samples.par_iter().map(bits).collect();
    }

    /// Books every change since the last booking to `part`.
    pub fn book(&mut self, samples: &[Sample], part: Part) {
        if samples.len() != self.bits.len() {
            // A resampling: the samples are new, their changes of state are not single moves.
            self.book_lattice(samples, part);
            return;
        }
        let now = stocks(samples);
        let p = part as usize;
        for (k, x) in self.parts[p].iter_mut().enumerate() {
            *x += now[k] - self.last[k];
        }
        self.last = now;
        let moved = self
            .bits
            .par_iter_mut()
            .zip(samples.par_iter())
            .fold(
                || [0.0f64; NX],
                |mut m, (b, s)| {
                    let (was, now) = (*b, bits(s));
                    if was == now {
                        return m;
                    }
                    *b = now;
                    let (c0, c1) = (was & BIT_CONT != 0, now & BIT_CONT != 0);
                    let (h0, h1) = (was & BIT_HIDDEN != 0, now & BIT_HIDDEN != 0);
                    let (d0, d1) = (was & BIT_DRY != 0, now & BIT_DRY != 0);
                    if !c0 && c1 {
                        m[0] += 1.0;
                        m[1] += f64::from(s.elevation_m);
                        m[2] += f64::from(s.own_thickness_m());
                        m[8] += f64::from(s.basement());
                    }
                    if c0 && !c1 {
                        m[3] += 1.0;
                    }
                    if c0 && c1 {
                        if !h0 && h1 {
                            m[4] += 1.0;
                        }
                        if h0 && !h1 {
                            m[5] += 1.0;
                        }
                        if !h0 && !h1 {
                            if d0 && !d1 {
                                m[6] += 1.0;
                            }
                            if !d0 && d1 {
                                m[7] += 1.0;
                            }
                        }
                    }
                    m
                },
            )
            .reduce(
                || [0.0; NX],
                |mut a, b| {
                    for (x, y) in a.iter_mut().zip(b) {
                        *x += y;
                    }
                    a
                },
            );
        for (x, y) in self.moves[p].iter_mut().zip(moved) {
            *x += y;
        }
    }

    /// With [`STEPS_ENV`], at the end of each plate step: a `cb_step` line with the sediment
    /// stocks ([`SED_TR`] to [`SED_C`]) now and their changes by part over the step, and the
    /// step's change of [`STEP_GLOBALS`], km³ Earth-normalized (sample area `area_km2`). The
    /// first step's globals are their totals so far.
    pub fn end_step(&mut self, t_myr: f64, area_km2: f64, globals: &BTreeMap<String, f64>) {
        let Some((prev, g_prev)) = self.steps.as_mut() else {
            return;
        };
        let km3 = area_km2 / 1000.0;
        let mut line = format!(
            "\ncb_step t={t_myr:.1} stock=[tr={:.5e},ov={:.5e},oh={:.5e},c={:.5e}]",
            self.last[SED_TR] * km3,
            self.last[SED_OV] * km3,
            self.last[SED_OH] * km3,
            self.last[SED_C] * km3,
        );
        for (name, k) in [
            ("d_tr", SED_TR),
            ("d_ov", SED_OV),
            ("d_oh", SED_OH),
            ("d_c", SED_C),
        ] {
            let row: Vec<String> = (0..PARTS)
                .filter(|&p| self.parts[p][k] != prev[p][k])
                .map(|p| format!("{}:{:.4e}", NAMES[p], (self.parts[p][k] - prev[p][k]) * km3))
                .collect();
            let _ = write!(line, " {name}=[{}]", row.join(","));
        }
        let _ = write!(line, " g=[");
        for (j, (name, key)) in STEP_GLOBALS.iter().enumerate() {
            let now = globals.get(*key).copied().unwrap_or(0.0);
            let was = if g_prev[j].is_nan() { 0.0 } else { g_prev[j] };
            let _ = write!(
                line,
                "{}{name}:{:.4e}",
                if j > 0 { "," } else { "" },
                now - was
            );
            g_prev[j] = now;
        }
        line.push(']');
        prev.clone_from(&self.parts);
        self.lines.push(line);
    }

    /// Whether the snapshots write maps ([`MAPS_ENV`]).
    pub fn maps_on(&self) -> bool {
        self.maps.is_some()
    }

    /// Records the budget at `t_myr`: the samples at `pos`, a sample's area `area_km2`
    /// (Earth-normalized), the climate's sea level `sea_m`, the world's globals. `fronts` (one
    /// per sample, the boundary mix's classes, `dynamics::boundary_mix`) marks the fronts in the
    /// map's flags ([`MAP_FRONT_SUBDUCTION`], [`MAP_FRONT_COLLISION`]).
    #[allow(clippy::too_many_arguments)]
    pub fn snapshot(
        &mut self,
        samples: &[Sample],
        pos: &[DVec3],
        t_myr: f64,
        area_km2: f64,
        sea_m: f64,
        globals: &BTreeMap<String, f64>,
        fronts: Option<&[u8]>,
    ) {
        let (z_0, g) = (self.z_0, self.gravity);
        let s = stocks(samples);
        let vis = s[7].max(1.0);
        let sea = sea_m as f32;
        let visible = |i: usize| !samples[i].is_hidden();
        let land = |i: usize| visible(i) && samples[i].elevation_m > sea;
        let n_land_sl = (0..samples.len()).filter(|&i| land(i)).count() as f64;
        let above_2km = (0..samples.len())
            .filter(|&i| land(i) && samples[i].elevation_m > sea + 2000.0)
            .count() as f64;
        let ocean_land = (0..samples.len())
            .filter(|&i| land(i) && !samples[i].is_continental())
            .count() as f64;
        // The dry share of the continental crust at the sea (`dry_sl`), beside the stock's `dry`
        // at the datum (the sea off the datum, plates.md 2026-10-21): the share Earth's 0.71
        // (HYP-012) is counted at.
        let cont_land = (0..samples.len())
            .filter(|&i| land(i) && samples[i].is_continental())
            .count() as f64;
        // The distance from each piece of land to the nearest visible sea floor.
        let coast_p50 = if pos.len() == samples.len() && n_land_sl > 0.0 {
            let sea_tree = Tree::over(pos, |i| visible(i) && samples[i].elevation_m <= sea);
            let mut d: Vec<f64> = (0..samples.len())
                .into_par_iter()
                .filter(|&i| land(i))
                .filter_map(|i| sea_tree.nearest(pos[i]))
                .map(|(_, c2)| 2.0 * (0.5 * c2.sqrt()).min(1.0).asin() * EARTH_KM)
                .collect();
            d.sort_by(f64::total_cmp);
            d.get(d.len() / 2).copied().unwrap_or(f64::NAN)
        } else {
            f64::NAN
        };
        // The land by where its crust came from (the land-shape task, 2026-10-20): island-arc
        // crust grown continental, a collision's belt, an Andean margin, the rest. Per origin: its
        // share of the land, its share above 2 km and its median distance to the coast.
        let origin_line = if pos.len() == samples.len() && n_land_sl > 0.0 {
            let sea_tree = Tree::over(pos, |i| visible(i) && samples[i].elevation_m <= sea);
            let mut d: [Vec<f64>; ORIGINS] = Default::default();
            let mut high = [0.0f64; ORIGINS];
            for i in (0..samples.len()).filter(|&i| land(i)) {
                let o = origin(&samples[i]);
                if samples[i].elevation_m > sea + 2000.0 {
                    high[o] += 1.0;
                }
                if let Some((_, c2)) = sea_tree.nearest(pos[i]) {
                    d[o].push(2.0 * (0.5 * c2.sqrt()).min(1.0).asin() * EARTH_KM);
                }
            }
            let mut out = format!("\ncb_origin t={t_myr:.0}");
            for (o, v) in d.iter_mut().enumerate() {
                v.sort_by(f64::total_cmp);
                let n = v.len() as f64;
                let _ = write!(
                    out,
                    " {}=[land={:.4} above_2km={:.4} coast_p50_km={:.0}]",
                    ORIGIN_NAMES[o],
                    n / n_land_sl,
                    high[o] / n.max(1.0),
                    v.get(v.len() / 2).copied().unwrap_or(f64::NAN),
                );
            }
            out
        } else {
            String::new()
        };
        let mut line = format!(
            "\ncb_t t={t_myr:.0} vis={:.0} cont={:.4} cont_hidden={:.4} vol_km3={:.4e} \
             vol_hidden_km3={:.4e} land0={:.4} land={:.4} sea_m={sea_m:.0} dry={:.4} dry_sl={:.4} \
             above_2km={:.4} ocean_land={:.4} coast_p50_km={coast_p50:.0} mean_t_km={:.2} \
             felsic_km3={:.4e} felsic_ocean_km3={:.4e}",
            s[7],
            s[CONT] / vis,
            s[CONT_HIDDEN] / vis,
            s[VOL] * area_km2 / 1000.0,
            s[VOL_HIDDEN] * area_km2 / 1000.0,
            s[LAND] / vis,
            n_land_sl / vis,
            s[DRY] / s[CONT].max(1.0),
            cont_land / s[CONT].max(1.0),
            above_2km / n_land_sl.max(1.0),
            ocean_land / n_land_sl.max(1.0),
            s[VOL] / s[CONT].max(1.0) / 1000.0,
            s[FELSIC] * area_km2 / 1000.0,
            s[FELSIC_OCEAN] * area_km2 / 1000.0,
        );
        line += &origin_line;
        // The thickness bands of the visible continental crust.
        let mut bands: Vec<Vec<(f32, f32, f32)>> = vec![Vec::new(); EDGES_KM.len() - 1];
        for x in samples
            .iter()
            .filter(|x| x.is_continental() && !x.is_hidden())
        {
            let t = f64::from(x.thickness_m) / 1000.0;
            let b = (0..bands.len())
                .find(|&b| t < EDGES_KM[b + 1])
                .unwrap_or(bands.len() - 1);
            bands[b].push((
                x.elevation_m,
                x.thickness_m,
                iso_height(&x.pools, z_0, g) as f32,
            ));
        }
        let _ = write!(line, "\ncb_bins t={t_myr:.0}");
        for (b, v) in bands.iter_mut().enumerate() {
            v.sort_by(|a, b| a.0.total_cmp(&b.0));
            let n = v.len() as f64;
            let q = |f: f64| {
                v.get(((v.len().max(1) - 1) as f64 * f).round() as usize)
                    .map_or(f64::NAN, |x| f64::from(x.0))
            };
            let vol: f64 = v.iter().map(|x| f64::from(x.1)).sum();
            let iso = v.iter().map(|x| f64::from(x.2)).sum::<f64>() / n.max(1.0);
            let dry = v.iter().filter(|x| x.0 > sea).count() as f64 / n.max(1.0);
            let _ = write!(
                line,
                " [{}-{}km a={:.4} v={:.4} z10/50/90={:.0}/{:.0}/{:.0} iso={:.0} dry={:.3}]",
                EDGES_KM[b],
                EDGES_KM[b + 1].min(999.0),
                n / s[CONT].max(1.0),
                vol / s[VOL].max(1.0),
                q(0.1),
                q(0.5),
                q(0.9),
                iso,
                dry,
            );
        }
        // The parts' changes over the run so far: area as a share of the planet, volume in
        // 10^6 km³, land and dry continental crust as shares of the planet.
        let _ = write!(line, "\ncb_parts t={t_myr:.0}");
        for (name, k, scale) in [
            ("cont", CONT, 1.0 / vis),
            ("cont_hidden", CONT_HIDDEN, 1.0 / vis),
            ("vol", VOL, area_km2 / 1000.0 / 1.0e6),
            ("vol_hidden", VOL_HIDDEN, area_km2 / 1000.0 / 1.0e6),
            ("land", LAND, 1.0 / vis),
            ("dry", DRY, 1.0 / vis),
            ("felsic", FELSIC, area_km2 / 1000.0 / 1.0e6),
            ("felsic_ocean", FELSIC_OCEAN, area_km2 / 1000.0 / 1.0e6),
            ("sed_tr", SED_TR, area_km2 / 1000.0 / 1.0e6),
            ("sed_ov", SED_OV, area_km2 / 1000.0 / 1.0e6),
            ("sed_oh", SED_OH, area_km2 / 1000.0 / 1.0e6),
            ("sed_c", SED_C, area_km2 / 1000.0 / 1.0e6),
        ] {
            let row: Vec<String> = (0..PARTS)
                .filter(|&p| self.parts[p][k] != 0.0)
                .map(|p| format!("{}:{:.5}", NAMES[p], self.parts[p][k] * scale))
                .collect();
            let _ = write!(line, " {name}=[{}]", row.join(","));
        }
        let _ = write!(line, "\ncb_moves t={t_myr:.0}");
        for (name, m) in NAMES.iter().zip(&self.moves) {
            if m.iter().all(|&x| x == 0.0) {
                continue;
            }
            let _ = write!(
                line,
                " {}=[o2c={:.5} o2c_z={:.0} o2c_t_km={:.1} o2c_b_km={:.2} c2o={:.5} hide={:.5} surface={:.5} drown={:.5} emerge={:.5}]",
                name,
                m[0] / vis,
                m[1] / m[0].max(1.0),
                m[2] / m[0].max(1.0) / 1000.0,
                m[8] / m[0].max(1.0) / 1000.0,
                m[3] / vis,
                m[4] / vis,
                m[5] / vis,
                m[6] / vis,
                m[7] / vis,
            );
        }
        let _ = write!(line, "\ncb_globals t={t_myr:.0}");
        for (name, key) in GLOBALS {
            let _ = write!(
                line,
                " {name}={:.4e}",
                globals.get(key).copied().unwrap_or(0.0)
            );
        }
        self.lines.push(line);
        if let Some(dir) = &self.maps {
            write_map(dir, samples, pos, t_myr, fronts);
        }
    }

    /// With [`MAPS_ENV`] set, writes the plate of each visible sample (the records of the map the
    /// snapshot at `t_myr` wrote, in order) to `<dir>/plate/t<Myr>.bin` as little-endian u32
    /// (measurement only; a failure is reported and ignored).
    pub fn write_plates(&self, samples: &[Sample], plate_of: impl Fn(usize) -> u32, t_myr: f64) {
        let Some(dir) = &self.maps else { return };
        let mut buf: Vec<u8> = Vec::with_capacity(samples.len() * 4);
        for (i, s) in samples.iter().enumerate() {
            if !s.is_hidden() {
                buf.extend_from_slice(&plate_of(i).to_le_bytes());
            }
        }
        let dir = dir.join("plate");
        let path = dir.join(format!("t{:04.0}.bin", t_myr.round()));
        if let Err(e) = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, &buf)) {
            eprintln!("crust_budget: cannot write {}: {e}", path.display());
        }
    }

    /// With [`MAPS_ENV`] set, writes the sea-floor age of each visible sample (the records of the
    /// map the snapshot at `t_myr` wrote, in order; [`age_records`]) to `<dir>/age/t<Myr>.bin` as
    /// little-endian f32 (measurement only; a failure is reported and ignored).
    pub fn write_ages(&self, samples: &[Sample], t_myr: f64) {
        let Some(dir) = &self.maps else { return };
        let mut buf: Vec<u8> = Vec::with_capacity(samples.len() * 4);
        for a in age_records(samples, t_myr) {
            buf.extend_from_slice(&a.to_le_bytes());
        }
        let dir = dir.join("age");
        let path = dir.join(format!("t{:04.0}.bin", t_myr.round()));
        if let Err(e) = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, &buf)) {
            eprintln!("crust_budget: cannot write {}: {e}", path.display());
        }
    }

    /// The summary lines for `deeptime run` and `stage`.
    pub fn lines(&self) -> String {
        let mut out = format!(
            "\ncrust_budget z0_m={:.0} stocks={} moves={} parts={}",
            self.z_0,
            STOCKS.join(","),
            MOVES.join(","),
            NAMES.join(",")
        );
        for l in &self.lines {
            out += l;
        }
        out + &self.dock.lines()
    }
}

/// The age file's record of each visible sample, in the map's order: the sea-floor age at `t_myr`
/// ([`Sample::age_myr`], Myr) of oceanic crust, NaN of continental crust. The ages are the ones
/// `dynamics::ocean_ages` sorts for the dynamics record's age quantiles.
pub fn age_records(samples: &[Sample], t_myr: f64) -> Vec<f32> {
    samples
        .iter()
        .filter(|s| !s.is_hidden())
        .map(|s| {
            if s.kind == OCEANIC {
                s.age_myr(t_myr) as f32
            } else {
                f32::NAN
            }
        })
        .collect()
}

/// The f32 fields of a map record, in order.
pub const MAP_FIELDS: [&str; 11] = [
    "lat",
    "lon",
    "z",
    "thickness",
    "kind",
    "orogeny",
    "orogeny_age",
    "arc_oceanic_m",
    "arc_continental_m",
    "group",
    "flags",
];

/// The map's flag (above the sample flags' eight bits) of a sample on the overriding side of a
/// subduction front, as the dynamics record's boundary mix classed it (`dynamics::mix`'s
/// ocean-ocean and ocean-continent classes): the front Bird 2003 calls a subduction zone (SUB).
pub const MAP_FRONT_SUBDUCTION: u16 = 256;
/// … of a front whose subducting side is continental crust (`dynamics::mix::COLLISION`): Bird's
/// continental convergent boundary (CCB).
pub const MAP_FRONT_COLLISION: u16 = 512;

/// A map record's flags: the sample's own and, from the boundary mix's class (`fronts`, the
/// class plus one), the front's.
fn map_flags(s: &Sample, front: u8) -> u16 {
    let front = match front.checked_sub(1).map(usize::from) {
        Some(super::dynamics::mix::OCEAN_OCEAN | super::dynamics::mix::OCEAN_CONTINENT) => {
            MAP_FRONT_SUBDUCTION
        }
        Some(super::dynamics::mix::COLLISION) => MAP_FRONT_COLLISION,
        _ => 0,
    };
    u16::from(s.flags) | front
}

/// Writes the visible samples to `<dir>/t<Myr>.bin` (measurement only; a failure is reported and
/// ignored).
fn write_map(
    dir: &std::path::Path,
    samples: &[Sample],
    pos: &[DVec3],
    t_myr: f64,
    fronts: Option<&[u8]>,
) {
    if pos.len() != samples.len() {
        return;
    }
    let fronts = fronts.filter(|f| f.len() == samples.len());
    let mut buf: Vec<u8> = Vec::with_capacity(samples.len() * 4 * MAP_FIELDS.len());
    for (i, (s, p)) in samples.iter().zip(pos).enumerate() {
        if s.is_hidden() {
            continue;
        }
        let p = p.normalize_or_zero();
        let lat = p.z.clamp(-1.0, 1.0).asin().to_degrees();
        let lon = p.y.atan2(p.x).to_degrees();
        for x in [
            lat as f32,
            lon as f32,
            s.elevation_m,
            s.thickness_m,
            f32::from(u8::from(s.is_continental())),
            f32::from(s.orogeny),
            (t_myr - f64::from(s.orogeny_myr)) as f32,
            s.arc_oceanic_m,
            s.arc_continental_m,
            s.group as f32,
            f32::from(map_flags(s, fronts.map_or(0, |f| f[i]))),
        ] {
            buf.extend_from_slice(&x.to_le_bytes());
        }
    }
    let path = dir.join(format!("t{:04.0}.bin", t_myr.round()));
    if let Err(e) = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&path, &buf)) {
        eprintln!("crust_budget: cannot write {}: {e}", path.display());
    }
    write_pools(dir, samples, t_myr);
}

/// The f32 fields of a pools record (`pools/t<Myr>.bin`, the map's records in order): the rock
/// pools ([`pool::NAMES`]), the basement ([`Sample::basement`]) and when the crust formed.
pub const POOL_FIELDS: [&str; pool::COUNT + 2] = [
    "sediment",
    "sedimentary",
    "metamorphic",
    "felsic_plutonic",
    "felsic_volcanic",
    "mafic_volcanic",
    "mafic_plutonic",
    "basement",
    "formed_myr",
];

/// Writes the visible samples' rock pools to `<dir>/pools/t<Myr>.bin`, little-endian f32 records
/// of [`POOL_FIELDS`] in the map's order (the drowning diagnosis: what the thick ocean crust is
/// made of; measurement only, a failure is reported and ignored).
fn write_pools(dir: &std::path::Path, samples: &[Sample], t_myr: f64) {
    let mut buf: Vec<u8> = Vec::with_capacity(samples.len() * 4 * POOL_FIELDS.len());
    for s in samples.iter().filter(|s| !s.is_hidden()) {
        for x in s.pools.iter().copied().chain([s.basement(), s.formed_myr]) {
            buf.extend_from_slice(&x.to_le_bytes());
        }
    }
    let dir = dir.join("pools");
    let path = dir.join(format!("t{:04.0}.bin", t_myr.round()));
    if let Err(e) = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, &buf)) {
        eprintln!("crust_budget: cannot write {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::pools::continental_column;
    use crate::plates::state::{CONTINENTAL, HIDDEN, OCEANIC};

    fn column(kind: u8, z: f32) -> Sample {
        let mut s = Sample::blank(0);
        s.kind = kind;
        s.pools = continental_column();
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = z;
        s
    }

    /// A map record's flags carry the boundary mix's fronts above the sample's own eight bits: a
    /// subduction front (ocean-ocean, ocean-continent) and a collision front apart, nothing for
    /// the other classes or no class.
    #[test]
    fn the_map_flags_mark_the_fronts() {
        use crate::plates::dynamics::mix;
        let mut s = column(CONTINENTAL, 500.0);
        s.flags = crate::plates::state::ARC;
        let own = u16::from(s.flags);
        let f = |class: usize| map_flags(&s, class as u8 + 1);
        assert_eq!(f(mix::OCEAN_OCEAN), own | MAP_FRONT_SUBDUCTION);
        assert_eq!(f(mix::OCEAN_CONTINENT), own | MAP_FRONT_SUBDUCTION);
        assert_eq!(f(mix::COLLISION), own | MAP_FRONT_COLLISION);
        for class in [mix::RIDGE, mix::RIFT, mix::OTHER] {
            assert_eq!(f(class), own, "class {class}");
        }
        assert_eq!(map_flags(&s, 0), own);
    }

    /// The age file has one record per visible sample, NaN for each visible continental sample,
    /// and its oceanic records, sorted, are the dynamics record's sea-floor ages.
    #[test]
    fn the_age_records_are_the_ocean_ages() {
        use crate::plates::dynamics::ocean_ages;
        use crate::plates::geometry::Geo;
        use crate::plates::rifting::tests::{N, one_plate};
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let t = 640.0;
        let mut st = one_plate(&geo, 5, |p| {
            let mut s = column(u8::from(p.y > 0.5), 0.0);
            s.formed_myr = (t as f32) * (0.5 + 0.5 * p.x as f32) + 30.0 * p.z as f32;
            if p.z > 0.8 {
                s.flags |= HIDDEN;
            }
            s
        });
        st.t_myr = t;
        let rec = age_records(&st.samples, t);
        let visible = st.samples.iter().filter(|s| !s.is_hidden());
        assert_eq!(rec.len(), visible.clone().count());
        let continental = visible.filter(|s| s.kind != OCEANIC).count();
        assert!(continental > 0 && continental < rec.len(), "{continental}");
        assert_eq!(rec.iter().filter(|a| a.is_nan()).count(), continental);
        let mut ocean: Vec<f32> = rec.into_iter().filter(|a| !a.is_nan()).collect();
        ocean.sort_unstable_by(f32::total_cmp);
        let want: Vec<f32> = ocean_ages(&st, t).into_iter().map(|a| a as f32).collect();
        assert!(want.iter().any(|&a| a > 0.0) && want.contains(&0.0));
        assert_eq!(ocean, want);
    }

    /// Each change lands in the part it was booked to, and the moves are counted once.
    #[test]
    fn changes_land_in_their_parts() {
        let mut samples = vec![
            column(CONTINENTAL, 500.0),
            column(CONTINENTAL, 300.0),
            column(OCEANIC, -3000.0),
            column(CONTINENTAL, 800.0),
        ];
        let mut b = CrustBudget::start(&samples, 525.0, 9.81);
        samples[1].elevation_m = -100.0;
        b.book(&samples, Part::Erosion);
        samples[0].flags |= HIDDEN;
        b.book(&samples, Part::Subduction);
        samples[2].kind = CONTINENTAL;
        b.book(&samples, Part::Collapse);
        assert_eq!(b.moves[Part::Erosion as usize][6], 1.0, "drowned");
        assert_eq!(b.moves[Part::Subduction as usize][4], 1.0, "hidden");
        assert_eq!(b.moves[Part::Collapse as usize][0], 1.0, "made continental");
        assert_eq!(b.moves[Part::Collapse as usize][1], -3000.0);
        assert_eq!(b.parts[Part::Subduction as usize][CONT], -1.0);
        assert_eq!(b.parts[Part::Subduction as usize][CONT_HIDDEN], 1.0);
        assert_eq!(b.parts[Part::Erosion as usize][DRY], -1.0);
        let pos = vec![DVec3::X, DVec3::Y, DVec3::Z, -DVec3::X];
        b.snapshot(&samples, &pos, 2.0, 1.0, 0.0, &BTreeMap::new(), None);
        let out = b.lines();
        assert!(out.contains("cb_t t=2"), "{out}");
        assert!(out.contains("collapse=[o2c="), "{out}");
        assert!(out.contains("cb_origin t=2 island_arc=[land="), "{out}");
    }

    /// The land's origins: island-arc crust before a collision, a collision before an Andean margin.
    #[test]
    fn origins_in_order() {
        let mut s = column(CONTINENTAL, 500.0);
        assert_eq!(ORIGIN_NAMES[origin(&s)], "other");
        s.orogeny = OROGENY_ANDEAN;
        assert_eq!(ORIGIN_NAMES[origin(&s)], "andean");
        s.orogeny = OROGENY_HIMALAYAN;
        assert_eq!(ORIGIN_NAMES[origin(&s)], "collision");
        s.arc_oceanic_m = 8000.0;
        s.arc_continental_m = 2000.0;
        assert_eq!(ORIGIN_NAMES[origin(&s)], "island_arc");
    }
}
