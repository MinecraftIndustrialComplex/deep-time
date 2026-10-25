// SPDX-License-Identifier: LGPL-3.0-or-later
//! Run statistics of the plate model: the plan's statistical targets (oceanic crust mostly younger
//! than ~250 Myr; a few large plates plus small ones), the CLI's summary line, and the continental
//! margins by tectonic setting (P1-27, [`margins`]).

use glam::DVec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::geometry::Tree;
use super::normalise::Norm;
use super::state::{CONTINENTAL, Counters, PlateState, regime};
use crate::sphere::kdtree::angle_of_chord2;

/// A snapshot of the crust's statistics (visible samples).
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    /// Plates alive.
    pub plates: usize,
    /// (plate id, fraction of the planet's surface), largest first.
    pub areas: Vec<(u32, f64)>,
    /// Oceanic fraction of the surface.
    pub ocean_frac: f64,
    /// Oceanic crust age percentiles, Myr (area-weighted: samples are equal-area).
    pub ocean_age_p50_myr: f64,
    pub ocean_age_p90_myr: f64,
    pub ocean_age_max_myr: f64,
    /// Fraction of oceanic crust younger than 250 Myr.
    pub ocean_younger_250: f64,
    /// Smallest oceanic crust age seen (must be ≥ 0).
    pub ocean_age_min_myr: f64,
    /// Samples (visible or hidden) per alive plate sum to the sample count, and every sample's
    /// plate is alive: no sample in two plates and none in none.
    pub partition_ok: bool,
    /// The run's event counters.
    pub counters: Counters,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let i = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// The statistics of `state` at its current time.
pub fn summary(state: &PlateState) -> Summary {
    let t = state.t_myr;
    let n = state.samples.len();
    let mut per_plate = vec![0usize; state.plates.len()];
    let mut all = vec![0usize; state.plates.len()];
    let mut ages = Vec::new();
    let mut visible = 0usize;
    for (i, s) in state.samples.iter().enumerate() {
        let p = state.plate_of(i) as usize;
        all[p] += 1;
        if s.is_hidden() {
            continue;
        }
        visible += 1;
        per_plate[p] += 1;
        if s.kind != CONTINENTAL {
            ages.push(t - f64::from(s.formed_myr));
        }
    }
    ages.sort_by(f64::total_cmp);
    let mut areas: Vec<(u32, f64)> = state
        .alive()
        .map(|p| {
            (
                p.id,
                per_plate[p.id as usize] as f64 / visible.max(1) as f64,
            )
        })
        .collect();
    areas.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let partition_ok = state
        .samples
        .iter()
        .enumerate()
        .all(|(i, _)| state.plates[state.plate_of(i) as usize].alive)
        && state.alive().map(|p| all[p.id as usize]).sum::<usize>() == n;
    Summary {
        plates: state.alive().count(),
        areas,
        ocean_frac: ages.len() as f64 / visible.max(1) as f64,
        ocean_age_p50_myr: percentile(&ages, 0.5),
        ocean_age_p90_myr: percentile(&ages, 0.9),
        ocean_age_max_myr: ages.last().copied().unwrap_or(f64::NAN),
        ocean_younger_250: if ages.is_empty() {
            1.0
        } else {
            ages.iter().filter(|&&a| a < 250.0).count() as f64 / ages.len() as f64
        },
        ocean_age_min_myr: ages.first().copied().unwrap_or(0.0),
        partition_ok,
        counters: state.counters.clone(),
    }
}

/// A coastal sample this close (Earth-normalized km) to a subduction front, on either plate, is on
/// an active margin (P1-27, design choice: Earth's continental arcs stand 150–300 km from their
/// trench, and a continent this close to a trench on the subducting side is about to collide).
pub const MARGIN_ACTIVE_KM: f64 = 400.0;

/// Landmasses (connected continental crust) smaller than this are fragments, not continents:
/// island arcs and slivers, whose coasts are not counted as continental margins, km²
/// (Earth-normalized; design choice: about Great Britain, so Earth's continents and its largest
/// islands count).
pub const MARGIN_MIN_LANDMASS_KM2: f64 = 200_000.0;

/// Neighbours within this many sample spacings are adjacent (the continent–ocean boundary, a plate
/// boundary): the lattice's nearest neighbours sit about one spacing apart.
const MARGIN_ADJACENT_SPACINGS: f64 = 1.5;

/// The continental margins at one moment, by tectonic setting (P1-27). The margin is the
/// continent–ocean boundary: every visible continental sample with visible oceanic crust (of any
/// plate) within 1.5 sample spacings, one spacing of coastline each. Each is, in this order:
/// **collision** (regime collision), **subduction** (a confirmed subduction front within
/// [`MARGIN_ACTIVE_KM`], on either plate), **boundary** (another plate's crust adjacent: a rifted,
/// divergent or transform margin) or **passive** (no plate boundary near: crust and ocean ride one
/// plate). Earth's present-day margins are the reference (STATUS.md, P1-27).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    /// Time, Myr relative to world creation.
    #[serde(deserialize_with = "nan_if_null")]
    pub t_myr: f64,
    /// Margin length, km (Earth-normalized: one sample spacing per coastal sample).
    #[serde(deserialize_with = "nan_if_null")]
    pub coast_km: f64,
    /// Coastal samples, and of them …
    pub coast: u32,
    /// … at a subduction front (both plates' sides), …
    pub subduction: u32,
    /// … (of those, the ones whose continent overrides the front: Andean margins) …
    pub andean: u32,
    /// … (of those, fronts where one plate was split off by a subduction initiation) …
    pub initiated: u32,
    /// … in a collision belt, …
    pub collision: u32,
    /// … at another plate boundary (rift, divergent or transform), …
    pub boundary: u32,
    /// … and passive.
    pub passive: u32,
    /// Coastal samples whose regime (the `regime` layer) is arc, back-arc, trench or collision.
    pub regime_active: u32,
    /// Coastal samples whose regime is arc.
    pub regime_arc: u32,
    /// Median age of the oceanic crust next to passive margins, Myr (how long margins stay
    /// passive: the floor beside a rifted margin is as old as the ocean).
    #[serde(deserialize_with = "nan_if_null")]
    pub passive_floor_age_p50_myr: f64,
    /// Polarity reversals so far (the run's counter).
    pub reversals: u32,
    /// Coastal samples with another plate's crust adjacent (a plate boundary at the coast).
    pub at_boundary: u32,
    /// Median distance from a subduction margin to its front, km (Earth-normalized).
    #[serde(deserialize_with = "nan_if_null")]
    pub front_km_p50: f64,
    /// Samples on the overriding side of a live subduction front (its length in spacings), …
    pub front_over: u32,
    /// … of which continental crust (the overriding plate's continent is at the front).
    pub front_over_continental: u32,
    /// Landmasses of at least [`MARGIN_MIN_LANDMASS_KM2`] (continents and large islands).
    pub landmasses: u32,
    /// Coastal samples of smaller fragments (island arcs, slivers), not counted above.
    pub fragment_coast: u32,
    /// Continental crust on fragments, as a share of all visible continental crust.
    #[serde(deserialize_with = "nan_if_null")]
    pub fragment_crust_share: f64,
}

/// JSON has no NaN (serde_json writes it as null): read null back as NaN.
fn nan_if_null<'de, D: serde::Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    Ok(Option::<f64>::deserialize(d)?.unwrap_or(f64::NAN))
}

impl Margins {
    fn share(&self, n: u32) -> f64 {
        f64::from(n) / f64::from(self.coast.max(1))
    }

    /// Share of the margin that is active (subduction or collision).
    pub fn active_share(&self) -> f64 {
        self.share(self.subduction + self.collision)
    }

    /// Share at a subduction front.
    pub fn subduction_share(&self) -> f64 {
        self.share(self.subduction)
    }

    /// Share at another plate boundary.
    pub fn boundary_share(&self) -> f64 {
        self.share(self.boundary)
    }

    /// Share that is passive.
    pub fn passive_share(&self) -> f64 {
        self.share(self.passive)
    }

    /// Share the regime layer shows as arc, back-arc, trench or collision.
    pub fn regime_active_share(&self) -> f64 {
        self.share(self.regime_active)
    }
}

/// Per-sample margin classes of [`margin_classes`] (the `deeptime margins` map).
pub mod margin_class {
    /// Oceanic crust, or hidden.
    pub const OCEAN: u8 = 0;
    /// Continental crust away from the coast.
    pub const INLAND: u8 = 1;
    pub const PASSIVE: u8 = 2;
    /// Subduction margin whose continent overrides the front (Andean).
    pub const ANDEAN: u8 = 3;
    /// Subduction margin on the subducting plate (the continent nears a trench).
    pub const SUBDUCTING: u8 = 4;
    pub const COLLISION: u8 = 5;
    /// Another plate boundary at the coast (rift, divergent, transform).
    pub const BOUNDARY: u8 = 6;
    /// Coast of a landmass below the size of a continent (an island arc, a sliver).
    pub const FRAGMENT: u8 = 7;
    /// Added to a sample's class when it lies on the overriding side of a live subduction front.
    pub const FRONT: u8 = 16;
}

/// The continental margins of `state` now (see [`Margins`]). `pos` are the samples' current
/// positions; `initiated(plate)` says whether a plate was split off by a subduction initiation;
/// a contact whose slab is shorter than `incipient_km` is no subduction zone yet
/// (`forces::incipient_km`, PLT-614).
pub fn margins(
    state: &PlateState,
    pos: &[DVec3],
    norm: &Norm,
    initiated: &(dyn Fn(u32) -> bool + Sync),
    incipient_km: f64,
) -> Margins {
    margin_classes(state, pos, norm, initiated, incipient_km).0
}

/// [`margins`] and every sample's [`margin_class`].
pub fn margin_classes(
    state: &PlateState,
    pos: &[DVec3],
    norm: &Norm,
    initiated: &(dyn Fn(u32) -> bool + Sync),
    incipient_km: f64,
) -> (Margins, Vec<u8>) {
    let t = state.t_myr;
    let samples = &state.samples;
    let plate_of: Vec<u32> = (0..samples.len())
        .into_par_iter()
        .map(|i| state.plate_of(i))
        .collect();
    let vis = Tree::over(pos, |i| !samples[i].is_hidden());
    let adjacent = MARGIN_ADJACENT_SPACINGS * norm.spacing_rad();
    // Confirmed subduction zones seen in the last step or carried through a resampling (ledger:
    // PLT-550), past their incipient stage (PLT-614).
    let slab_km: std::collections::BTreeMap<(u32, u32), f64> = state
        .slabs
        .iter()
        .map(|s| ((s.under, s.over), s.km))
        .collect();
    let live: Vec<(u32, u32)> = state
        .contacts
        .iter()
        .filter(|c| c.onset_event.is_some() && (c.carried || c.last_step + 1 >= state.step))
        .map(|c| (c.under, c.over))
        .filter(|key| slab_km.get(key).copied().unwrap_or(0.0) >= incipient_km)
        .collect();
    let omega: Vec<DVec3> = state
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let contact_of = |a: u32, b: u32| {
        live.iter()
            .position(|&(u, o)| (u == a && o == b) || (u == b && o == a))
    };
    // Per visible sample: on the coast, the plate boundary it touches (if any), the age of the
    // oldest adjacent oceanic crust, and the live contact it fronts.
    struct Near {
        coast: bool,
        other_plate: bool,
        floor_age: f64,
        front: Option<usize>,
        over: bool,
    }
    let land = |i: usize| samples[i].kind == CONTINENTAL && !samples[i].is_hidden();
    // One query per visible sample serves both its margin and its landmass links (P1-19: two
    // identical queries before).
    let (near, links): (Vec<Near>, Vec<Vec<u32>>) = (0..samples.len())
        .into_par_iter()
        .map(|i| {
            let s = &samples[i];
            let mut out = Near {
                coast: false,
                other_plate: false,
                floor_age: f64::NAN,
                front: None,
                over: false,
            };
            if s.is_hidden() {
                return (out, Vec::new());
            }
            let nn = vis.nearest_n(pos[i], 8);
            // Landmasses: connected continental crust (any plate), linked to adjacent samples.
            let links: Vec<u32> = if land(i) {
                nn.iter()
                    .filter(|&&(j, d2)| {
                        j as usize > i && land(j as usize) && angle_of_chord2(d2) <= adjacent
                    })
                    .map(|&(j, _)| j)
                    .collect()
            } else {
                Vec::new()
            };
            for (j, d2) in nn {
                if j as usize == i || angle_of_chord2(d2) > adjacent {
                    continue;
                }
                let o = &samples[j as usize];
                if s.kind == CONTINENTAL && o.kind != CONTINENTAL {
                    out.coast = true;
                    let a = o.age_myr(t);
                    if out.floor_age.is_nan() || a > out.floor_age {
                        out.floor_age = a;
                    }
                }
                let pj = plate_of[j as usize];
                if pj != plate_of[i] {
                    out.other_plate = true;
                    // A front where the two plates close in on each other (the model's own test,
                    // `subduction::fronts_and_uplift`): a contact's diverging stretches are none.
                    let x = pos[i];
                    let rel =
                        norm.velocity_mm_yr(omega[pj as usize] - omega[plate_of[i] as usize], x);
                    let out_dir =
                        (pos[j as usize] - x * pos[j as usize].dot(x)).normalize_or_zero();
                    if rel.dot(out_dir) < 0.0
                        && let Some(c) = contact_of(plate_of[i], pj)
                    {
                        out.front.get_or_insert(c);
                        out.over |= live[c].1 == plate_of[i];
                    }
                }
            }
            (out, links)
        })
        .unzip();
    // Landmasses: connected continental crust (any plate), by union–find over adjacent samples.
    let n = samples.len();
    let mut parent: Vec<u32> = (0..n as u32).collect();
    fn root(parent: &mut [u32], mut i: u32) -> u32 {
        while parent[i as usize] != i {
            parent[i as usize] = parent[parent[i as usize] as usize];
            i = parent[i as usize];
        }
        i
    }
    for (i, l) in links.iter().enumerate() {
        for &j in l {
            let (a, b) = (root(&mut parent, i as u32), root(&mut parent, j));
            if a != b {
                parent[a.max(b) as usize] = a.min(b);
            }
        }
    }
    let mut size = vec![0u32; n];
    let roots: Vec<u32> = (0..n as u32).map(|i| root(&mut parent, i)).collect();
    for i in 0..n {
        if land(i) {
            size[roots[i] as usize] += 1;
        }
    }
    let min_land = (MARGIN_MIN_LANDMASS_KM2 / norm.sample_area_km2()).ceil() as u32;
    let fragment = |i: usize| size[roots[i] as usize] < min_land;
    let fronts: Vec<u32> = (0..samples.len() as u32)
        .filter(|&i| near[i as usize].front.is_some())
        .collect();
    let front_pos: Vec<DVec3> = fronts.iter().map(|&i| pos[i as usize]).collect();
    let front_tree = Tree::over(&front_pos, |_| true);
    let active = norm.rad(MARGIN_ACTIVE_KM);
    let (mut crust, mut crust_fragments) = (0u32, 0u32);
    let mut landmasses = 0u32;
    for i in 0..n {
        if land(i) {
            crust += 1;
            if fragment(i) {
                crust_fragments += 1;
            }
            if roots[i] as usize == i && size[i] >= min_land {
                landmasses += 1;
            }
        }
    }
    let mut m = Margins {
        t_myr: t,
        reversals: state.counters.reversals,
        landmasses,
        fragment_crust_share: f64::from(crust_fragments) / f64::from(crust.max(1)),
        ..Margins::default()
    };
    let mut passive_ages = Vec::new();
    let mut front_km = Vec::new();
    let mut class: Vec<u8> = samples
        .iter()
        .map(|s| {
            if s.kind == CONTINENTAL && !s.is_hidden() {
                margin_class::INLAND
            } else {
                margin_class::OCEAN
            }
        })
        .collect();
    for (i, nb) in near.iter().enumerate() {
        if nb.over {
            class[i] += margin_class::FRONT;
            m.front_over += 1;
            if samples[i].kind == CONTINENTAL {
                m.front_over_continental += 1;
            }
        }
        if !nb.coast {
            continue;
        }
        if fragment(i) {
            m.fragment_coast += 1;
            class[i] = margin_class::FRAGMENT | (class[i] & margin_class::FRONT);
            continue;
        }
        if nb.other_plate {
            m.at_boundary += 1;
        }
        let s = &samples[i];
        m.coast += 1;
        match s.regime {
            regime::ARC => {
                m.regime_active += 1;
                m.regime_arc += 1;
            }
            regime::BACK_ARC | regime::TRENCH | regime::COLLISION => m.regime_active += 1,
            _ => {}
        }
        let front = if fronts.is_empty() {
            None
        } else {
            front_tree
                .nearest(pos[i])
                .filter(|&(_, d2)| angle_of_chord2(d2) <= active)
                .map(|(f, d2)| {
                    front_km.push(norm.km(angle_of_chord2(d2)));
                    near[fronts[f as usize] as usize].front.expect("a front")
                })
        };
        let front_bit = class[i] & margin_class::FRONT;
        let code = if s.regime == regime::COLLISION {
            m.collision += 1;
            margin_class::COLLISION
        } else if let Some(c) = front {
            let (under, over) = live[c];
            m.subduction += 1;
            if initiated(under) || initiated(over) {
                m.initiated += 1;
            }
            if over == plate_of[i] {
                m.andean += 1;
                margin_class::ANDEAN
            } else {
                margin_class::SUBDUCTING
            }
        } else if nb.other_plate {
            m.boundary += 1;
            margin_class::BOUNDARY
        } else {
            m.passive += 1;
            if nb.floor_age.is_finite() {
                passive_ages.push(nb.floor_age);
            }
            margin_class::PASSIVE
        };
        class[i] = code | front_bit;
    }
    passive_ages.sort_by(f64::total_cmp);
    m.passive_floor_age_p50_myr = percentile(&passive_ages, 0.5);
    front_km.sort_by(f64::total_cmp);
    m.front_km_p50 = percentile(&front_km, 0.5);
    m.coast_km = f64::from(m.coast) * norm.km(norm.spacing_rad());
    (m, class)
}

impl Summary {
    /// Plates covering at least `frac` of the surface.
    pub fn plates_at_least(&self, frac: f64) -> usize {
        self.areas.iter().filter(|a| a.1 >= frac).count()
    }

    /// The CLI's line (plan P1-03: `plates_final=<n> ocean_age_p90_myr=<a> …`).
    pub fn line(&self) -> String {
        let big = self.areas.first().map_or(0.0, |a| a.1);
        format!(
            "plates_final={} ocean_age_p90_myr={:.1} ocean_age_p50_myr={:.1} \
             ocean_younger_250={:.3} ocean_frac={:.3} largest_plate_frac={:.3} \
             plates_over_5pct={} plates_under_1pct={} rifts={} initiations={} reversals={} \
             subductions={} collisions={} sutures={} healed={} accretions={} ridges={} births={} \
             deaths={} lips={} resamplings={} emerged={} buoyant={} reversals_confirmed={} \
             transferences={}",
            self.plates,
            self.ocean_age_p90_myr,
            self.ocean_age_p50_myr,
            self.ocean_younger_250,
            self.ocean_frac,
            big,
            self.plates_at_least(0.05),
            self.areas.iter().filter(|a| a.1 < 0.01).count(),
            self.counters.rifts,
            self.counters.initiations,
            self.counters.reversals,
            self.counters.subduction_onsets,
            self.counters.collisions,
            self.counters.sutures,
            self.counters.healed,
            self.counters.accretions,
            self.counters.ridges,
            self.counters.plate_births,
            self.counters.plate_deaths,
            self.counters.lips,
            self.counters.resamplings,
            self.counters.emerged,
            self.counters.buoyant,
            self.counters.reversals_confirmed,
            self.counters.transferences
        )
    }
}
